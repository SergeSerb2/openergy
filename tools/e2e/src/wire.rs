//! Original raw probe builders from docs/re/03-core-protocol.md, not the codec.
use crate::model::{Failure, Result, Step};
use crate::util::{check_deadline, hex, io_budget, wait_io};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::{Duration, Instant};

pub struct Raw {
    pub listener: Option<TcpListener>,
    pub stream: Option<TcpStream>,
    /// Bytes read from `stream` but not yet consumed by a step. Bulk reads keep
    /// a probe from paying two syscalls per tiny frame while it drains backlog.
    pub pending: VecDeque<u8>,
}
impl Raw {
    pub fn accept(&mut self, timeout: Duration) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| Failure::runner("raw node has no stream or listener"))?;
        listener.set_nonblocking(true)?;
        let deadline = Instant::now() + timeout;
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(true)?;
                    stream.set_nodelay(true)?;
                    self.stream = Some(stream);
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(Failure::product(format!("raw accept: {e}"))),
            }
        }
    }
    pub fn socket(&mut self, timeout: Duration) -> Result<&mut TcpStream> {
        self.accept(timeout)?;
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| Failure::runner("missing raw socket"))?;
        stream.set_nonblocking(true)?;
        Ok(stream)
    }
    pub fn send(&mut self, data: &[u8], step: &Step, timeout: Duration) -> Result<()> {
        let repeat = step.number("repeat", 1);
        if repeat > 1_000_000 || data.len().saturating_mul(repeat as usize) > 64 * 1024 * 1024 {
            return Err(Failure::runner("raw send exceeds bounded probe budget"));
        }
        let stream = self.socket(timeout)?;
        let deadline = Instant::now() + timeout;
        for _ in 0..repeat {
            let mut offset = 0;
            while offset < data.len() {
                check_deadline(deadline, "raw send")?;
                match stream.write(&data[offset..]) {
                    Ok(0) => return Err(Failure::product("raw write returned zero")),
                    Ok(count) => offset += count,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => wait_io(deadline, "raw send")?,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(e)
                        if step.flag("allow_closed")
                            && matches!(
                                e.kind(),
                                std::io::ErrorKind::ConnectionReset
                                    | std::io::ErrorKind::ConnectionAborted
                                    | std::io::ErrorKind::BrokenPipe
                            ) =>
                    {
                        return Ok(())
                    }
                    Err(e) => return Err(Failure::product(format!("raw send: {e}"))),
                }
            }
        }
        if step.flag("shutdown_write") {
            stream.shutdown(Shutdown::Write)?;
        }
        Ok(())
    }
    pub fn expect(&mut self, data: &[u8], timeout: Duration, framed: bool, scan: bool) -> Result<()> {
        self.socket(timeout)?;
        let started = Instant::now();
        let deadline = started + timeout;
        let (mut frames, mut bytes) = (0usize, 0usize);
        // Hosted CI keeps only the console log, so say how far the scan got.
        let context = |e: Failure, frames: usize, bytes: usize| Failure {
            message: format!(
                "{} after {frames} frames / {bytes} bytes in {} ms",
                e.message,
                started.elapsed().as_millis()
            ),
            ..e
        };
        loop {
            let actual = if framed {
                let header = self.take(4, deadline).map_err(|e| context(e, frames, bytes))?;
                let len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
                if len > 4 * 1024 * 1024 {
                    return Err(Failure::product("peer sent oversized frame"));
                }
                self.take(len, deadline).map_err(|e| context(e, frames, bytes))?
            } else {
                self.take(data.len(), deadline).map_err(|e| context(e, frames, bytes))?
            };
            frames += 1;
            bytes += actual.len();
            if actual.starts_with(data) {
                return Ok(());
            }
            if !scan || Instant::now() >= deadline {
                return Err(Failure::product(format!(
                    "raw expectation mismatch ({} received bytes, {} expected bytes)",
                    actual.len(),
                    data.len()
                )));
            }
        }
    }
    pub fn expect_absent(&mut self, codes: &[String], timeout: Duration) -> Result<()> {
        let mut bytes: Vec<u8> = self.pending.drain(..).collect();
        let stream = self.socket(timeout)?;
        let deadline = Instant::now() + timeout;
        let mut buffer = [0; 4096];
        while Instant::now() < deadline {
            match stream.read(&mut buffer) {
                Ok(0) => return Err(Failure::product("peer closed during absence check")),
                Ok(n) => bytes.extend_from_slice(&buffer[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(e) => return Err(e.into()),
            }
            while bytes.len() >= 4 {
                let n = u32::from_be_bytes(bytes[..4].try_into().map_err(|_| Failure::runner("header"))?) as usize;
                if !(4..=4 * 1024 * 1024).contains(&n) {
                    return Err(Failure::product("invalid frame length"));
                }
                if bytes.len() < n + 4 {
                    break;
                }
                if codes.iter().any(|code| bytes[4..8] == *code.as_bytes()) {
                    return Err(Failure::product("forbidden frame received"));
                }
                bytes.drain(..n + 4);
            }
        }
        if !bytes.is_empty() {
            return Err(Failure::product("partial frame at absence deadline"));
        }
        Ok(())
    }
    pub fn expect_closed(&mut self, timeout: Duration, reject_bytes: bool) -> Result<()> {
        let deadline = Instant::now() + timeout;
        let buffered = !self.pending.is_empty();
        self.pending.clear();
        if buffered && reject_bytes {
            return Err(Failure::product("unexpected bytes before FIN"));
        }
        let stream = self.socket(timeout)?;
        let mut buf = [0; 4096];
        loop {
            check_deadline(deadline, "raw close")?;
            match stream.read(&mut buf) {
                Ok(n) if n > 0 && reject_bytes => return Err(Failure::product("unexpected bytes before FIN")),
                Ok(0) => return Ok(()),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::BrokenPipe
                    ) =>
                {
                    return Ok(())
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => wait_io(deadline, "raw close")?,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(Failure::product(format!("expected TCP close: {e}"))),
                _ if Instant::now() >= deadline => return Err(Failure::product("peer kept hostile connection open")),
                _ => {}
            }
        }
    }
}
fn string(out: &mut Vec<u8>, s: &str) -> Result<()> {
    let len = u32::try_from(s.len()).map_err(|_| Failure::runner("wire string too long"))?;
    out.extend(len.to_be_bytes());
    out.extend(s.as_bytes());
    Ok(())
}
pub fn build(step: &Step) -> Result<Vec<u8>> {
    if let Some(text) = step.fields.get("hex").and_then(serde_json::Value::as_str) {
        return hex(text);
    }
    let name = step.text("builder")?;
    let mut payload = Vec::new();
    match name {
        "hello" | "hello_back" => {
            let product = step.string("product", "Synergy");
            if product.len() != 7 {
                return Err(Failure::runner("hello product must be 7 bytes"));
            }
            payload.extend(product.as_bytes());
            payload.extend((step.number("major", 1) as u16).to_be_bytes());
            payload.extend((step.number("minor", 8) as u16).to_be_bytes());
            if name == "hello_back" {
                string(&mut payload, &step.string("name", "client"))?;
            }
        }
        "screen_info" => {
            payload.extend(b"DINF");
            for value in [
                0,
                0,
                step.number("width", 1920) as i16,
                step.number("height", 1080) as i16,
                0,
                500,
                500,
            ] {
                payload.extend(value.to_be_bytes());
            }
        }
        "code" | "unknown" => {
            let code = step.string("code", "ZZZZ");
            if code.len() != 4 {
                return Err(Failure::runner("wire code must have 4 bytes"));
            }
            payload.extend(code.as_bytes());
            payload.extend(hex(&step.string("payload_hex", ""))?);
        }
        "options" => {
            payload.extend(b"DSOP");
            let values = step.value("values");
            let values = values
                .as_array()
                .ok_or_else(|| Failure::runner("options requires values array"))?;
            payload.extend((values.len() as u32).to_be_bytes());
            for value in values {
                payload.extend(
                    (value
                        .as_i64()
                        .ok_or_else(|| Failure::runner("option must be integer"))? as u32)
                        .to_be_bytes(),
                );
            }
        }
        "oversized" => return Ok(u32::MAX.to_be_bytes().to_vec()),
        "garbage" => return hex("00000003616263"),
        _ => return Err(Failure::runner(format!("unknown raw builder {name}"))),
    }
    if step.flag("payload_only") {
        return Ok(payload);
    }
    let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
    frame.extend(payload);
    Ok(frame)
}

impl Raw {
    /// Consume exactly `len` bytes. Reads are bulk and block in the kernel
    /// (bounded by the deadline), so draining a large backlog costs a few
    /// syscalls instead of two per tiny frame plus a timer sleep per refill.
    fn take(&mut self, len: usize, deadline: Instant) -> Result<Vec<u8>> {
        if self.pending.len() < len {
            let stream = self
                .stream
                .as_mut()
                .ok_or_else(|| Failure::runner("missing raw socket"))?;
            stream.set_nonblocking(false)?;
            let filled = fill(stream, &mut self.pending, len, deadline);
            stream.set_nonblocking(true)?;
            filled?;
        }
        Ok(self.pending.drain(..len).collect())
    }
}
fn fill(stream: &mut TcpStream, pending: &mut VecDeque<u8>, len: usize, deadline: Instant) -> Result<()> {
    let mut chunk = vec![0; 64 * 1024];
    while pending.len() < len {
        // macOS rejects socket options with EINVAL once both directions are shut
        // down; the next call then reports EOF or EPIPE without blocking.
        let _ = stream.set_read_timeout(io_budget(deadline, "raw read")?);
        match stream.read(&mut chunk) {
            Ok(0) => return Err(Failure::product("raw TCP closed inside expected bytes")),
            Ok(count) => pending.extend(&chunk[..count]),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(e) => return Err(Failure::product(format!("raw read: {e}"))),
        }
    }
    Ok(())
}
