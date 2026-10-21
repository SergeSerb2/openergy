//! Two-process TLS queue demonstration with diagnostic frames, not input events.
use openergy_net::{NetCommand, NetConfig, NetDriver, NetError, NetEvent, Priority, TransportMode, BULK_SLICE};
use openergy_session::ConnId;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const TOTAL: usize = 10 * 1024 * 1024;
fn main() {
    if let Err(e) = run() {
        eprintln!("priority_peer: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let role = args.next().ok_or("priority_peer server|client DIR PORT")?;
    if role != "server" && role != "client" {
        return Err("invalid role".into());
    }
    let dir = PathBuf::from(args.next().ok_or("missing DIR")?);
    let port: u16 = args.next().ok_or("missing PORT")?.parse()?;
    if !(24970..=24979).contains(&port) || args.next().is_some() {
        return Err("requires WP4 loopback port".into());
    }
    let (mut driver, handle) = NetDriver::new(NetConfig {
        tls_dir: dir.join("tls"),
        identity_name: role.clone(),
        ..NetConfig::default()
    })?;
    driver.set_transport_mode(TransportMode::Frames)?;
    let mut events = Vec::new();
    driver.execute(
        if role == "server" {
            NetCommand::Listen {
                addr: ([127, 0, 0, 1], port).into(),
            }
        } else {
            NetCommand::Connect {
                host: "127.0.0.1".into(),
                port,
            }
        },
        &mut events,
    );
    let start = Instant::now();
    let mut connection: Option<ConnId> = None;
    let mut sent = 0;
    let mut input_count = 0;
    let mut received = 0;
    let mut bulk_since_input = 0;
    let mut seen_input = false;
    let mut max_between = 0;
    let mut latencies = Vec::new();
    let mut done = false;
    while start.elapsed() < Duration::from_secs(20) {
        for event in std::mem::take(&mut events) {
            match event {
                NetEvent::Listening { addr } => println!("LISTENING {addr}"),
                NetEvent::Connected { conn, .. } => connection = Some(conn),
                NetEvent::Frame { conn, mut payload } => match payload.get(..4) {
                    Some(b"BULK") => {
                        received += payload.len() - 4;
                        if seen_input {
                            bulk_since_input += 1;
                        }
                    }
                    Some(b"INPT") => {
                        if seen_input {
                            max_between = max_between.max(bulk_since_input);
                        }
                        seen_input = true;
                        bulk_since_input = 0;
                        input_count += 1;
                        payload[..4].copy_from_slice(b"ECHO");
                        handle.send(NetCommand::SendFrame {
                            conn,
                            payload,
                            priority: Priority::Control,
                        })?;
                    }
                    Some(b"ECHO") => {
                        if let Some(stamp) = payload.get(4..12).and_then(|b| <[u8; 8]>::try_from(b).ok()) {
                            latencies
                                .push((start.elapsed().as_micros() as u64).saturating_sub(u64::from_be_bytes(stamp)));
                        }
                    }
                    Some(b"DONE") => {
                        println!("RECEIVED bulk_bytes={received} input_frames={input_count} max_bulk_frames_between_input={max_between} max_slice={BULK_SLICE}");
                        if received != TOTAL || input_count < 100 || max_between > 1 {
                            return Err("queue ordering or transfer-size failure".into());
                        }
                        handle.send(NetCommand::SendFrame {
                            conn,
                            payload: b"PASS".to_vec(),
                            priority: Priority::Control,
                        })?;
                        done = true;
                    }
                    Some(b"PASS") => {
                        latencies.sort_unstable();
                        let p99 = latencies.get(latencies.len() * 99 / 100).copied().unwrap_or(0);
                        println!(
                            "SENT bulk_bytes={sent} input_frames={input_count} echo_samples={} p99_echo_us={p99}",
                            latencies.len()
                        );
                        return Ok(());
                    }
                    _ => return Err("unexpected diagnostic frame".into()),
                },
                NetEvent::UntrustedPeer { .. } => return Err("pre-seed trust for both peers".into()),
                NetEvent::Disconnected { .. } if done => return Ok(()),
                NetEvent::Disconnected { reason, .. } => return Err(format!("disconnected: {reason:?}").into()),
                NetEvent::Error { message, .. }
                | NetEvent::ConnectFailed { error: message, .. }
                | NetEvent::ListenFailed { error: message, .. } => return Err(message.into()),
                _ => {}
            }
        }
        if role == "server" && !done {
            if let Some(conn) = connection {
                if sent < TOTAL || driver.conn_info(conn).is_some_and(|c| c.queued_bytes > 0) {
                    let mut payload = b"INPT".to_vec();
                    payload.extend_from_slice(&(start.elapsed().as_micros() as u64).to_be_bytes());
                    handle.send(NetCommand::SendFrame {
                        conn,
                        payload,
                        priority: Priority::Input,
                    })?;
                    input_count += 1;
                    loop {
                        let n = (TOTAL - sent).min(BULK_SLICE - 8);
                        if n == 0 {
                            break;
                        }
                        let mut payload = b"BULK".to_vec();
                        payload.resize(n + 4, 0);
                        match handle.send(NetCommand::SendFrame {
                            conn,
                            payload,
                            priority: Priority::Bulk,
                        }) {
                            Ok(()) => sent += n,
                            Err(NetError::QueueFull) => break,
                            Err(e) => return Err(e.into()),
                        }
                    }
                } else {
                    handle.send(NetCommand::SendFrame {
                        conn,
                        payload: b"DONE".to_vec(),
                        priority: Priority::Control,
                    })?;
                    done = true;
                }
            }
        }
        driver.poll(Some(Duration::from_millis(10)), &mut events)?;
        if role == "client" && done {
            for _ in 0..3 {
                driver.poll(Some(Duration::from_millis(10)), &mut events)?;
            }
            return Ok(());
        }
    }
    Err("priority demo timed out".into())
}
