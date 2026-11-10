//! Original raw probe builders from docs/re/03-core-protocol.md, not the codec.
use crate::model::{Failure, Result, Step};
use crate::util::{check_deadline, hex, io_budget, wait_io};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::{Duration, Instant};

/// Plain TCP, or a TLS 1.3 client with a self-signed certificate the
/// test server trusts by seeded pin. TLS exists so hostile-wire file
/// scenarios can pass the daemon's verified-TLS gate honestly.
pub enum Transport {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}
impl Transport {
    fn sock(&mut self) -> &mut TcpStream {
        match self {
            Self::Plain(stream) => stream,
            Self::Tls(stream) => &mut stream.sock,
        }
    }
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> std::io::Result<()> {
        self.sock().set_nonblocking(nonblocking)
    }
    pub fn set_nodelay(&mut self, nodelay: bool) -> std::io::Result<()> {
        self.sock().set_nodelay(nodelay)
    }
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> std::io::Result<()> {
        self.sock().set_read_timeout(timeout)
    }
    pub fn shutdown_write(&mut self) -> std::io::Result<()> {
        self.sock().shutdown(Shutdown::Write)
    }
}
impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf),
            Self::Tls(stream) => stream.read(buf),
        }
    }
}
impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.write(buf),
            Self::Tls(stream) => stream.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

pub struct Raw {
    pub listener: Option<TcpListener>,
    pub stream: Option<Transport>,
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
                    self.stream = Some(Transport::Plain(stream));
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => return Err(Failure::product(format!("raw accept: {e}"))),
            }
        }
    }
    pub fn socket(&mut self, timeout: Duration) -> Result<&mut Transport> {
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
            stream.shutdown_write()?;
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
                            // A TLS peer that drops TCP without
                            // close_notify reads as unexpected EOF.
                            | std::io::ErrorKind::UnexpectedEof
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
fn fill(stream: &mut Transport, pending: &mut VecDeque<u8>, len: usize, deadline: Instant) -> Result<()> {
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

/// Splits one DER element into (tag, contents, rest).
fn der(input: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, input) = input.split_first()?;
    let (&first, input) = input.split_first()?;
    let (len, input) = match first {
        0..=0x7f => (first as usize, input),
        0x81..=0x84 => {
            let n = (first & 0x7f) as usize;
            let (bytes, rest) = (input.get(..n)?, input.get(n..)?);
            (bytes.iter().fold(0usize, |acc, b| acc << 8 | *b as usize), rest)
        }
        _ => return None,
    };
    Some((tag, input.get(..len)?, input.get(len..)?))
}
fn spki(cert: &[u8]) -> Option<&[u8]> {
    let (0x30, cert, _) = der(cert)? else { return None };
    let (0x30, tbs, _) = der(cert)? else { return None };
    let mut rest = tbs;
    if rest.first() == Some(&0xa0) {
        rest = der(rest)?.2;
    }
    for _ in 0..5 {
        rest = der(rest)?.2;
    }
    let (0x30, _, after) = der(rest)? else { return None };
    Some(&rest[..rest.len() - after.len()])
}

#[derive(Debug)]
struct AnyServerCert {
    provider: std::sync::Arc<rustls::crypto::CryptoProvider>,
}
impl rustls::client::danger::ServerCertVerifier for AnyServerCert {
    fn verify_server_cert(
        &self,
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Test-only: the daemon gates application bytes on its own pin
        // trust state; this probe accepts any server certificate.
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _msg: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Err(rustls::Error::General("tls1.2 disabled in raw probe".into()))
    }
    fn verify_tls13_signature(
        &self,
        msg: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        let spki =
            spki(cert.as_ref()).ok_or(rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding))?;
        rustls::crypto::verify_tls13_signature_with_raw_key(
            msg,
            &rustls::pki_types::SubjectPublicKeyInfoDer::from(spki),
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

/// Wrap a connected socket in TLS 1.3 with the probe's client
/// certificate and complete the handshake before returning.
pub fn tls_client(stream: TcpStream, cert_der: &[u8], key_der: &[u8]) -> Result<Transport> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| Failure::runner(format!("tls versions: {e}")))?
        .dangerous()
        .with_custom_certificate_verifier(std::sync::Arc::new(AnyServerCert { provider }))
        .with_client_auth_cert(
            vec![CertificateDer::from(cert_der.to_vec())],
            PrivateKeyDer::Pkcs8(rustls::pki_types::PrivatePkcs8KeyDer::from(key_der.to_vec())),
        )
        .map_err(|e| Failure::runner(format!("tls client cert: {e}")))?;
    let name = ServerName::try_from("openergy.e2e")
        .map_err(|e| Failure::runner(format!("tls server name: {e}")))?
        .to_owned();
    let conn = rustls::ClientConnection::new(std::sync::Arc::new(config), name)
        .map_err(|e| Failure::runner(format!("tls connection: {e}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut owned = rustls::StreamOwned::new(conn, stream);
    while owned.conn.is_handshaking() {
        owned
            .conn
            .complete_io(&mut owned.sock)
            .map_err(|e| Failure::product(format!("raw TLS handshake: {e}")))?;
    }
    owned.sock.set_read_timeout(None)?;
    Ok(Transport::Tls(Box::new(owned)))
}
