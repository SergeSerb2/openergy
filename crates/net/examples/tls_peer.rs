//! Real transport peer. Sends handshake/control frames only; never touches input,
//! cursor, clipboard, platform APIs, or production paths/ports.
use openergy_net::{Fingerprint, NetCommand, NetConfig, NetDriver, NetEvent, Priority, TransportMode};
use std::{
    net::SocketAddr,
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("tls_peer: {error}");
        std::process::exit(1);
    }
}
struct Trace;
impl log::Log for Trace {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &log::Record<'_>) {
        eprintln!("{} {}", record.level(), record.args());
    }
    fn flush(&self) {}
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("OPENERGY_NET_TRACE").is_some() {
        log::set_logger(&Trace).map_err(|_| "cannot install trace logger")?;
        log::set_max_level(log::LevelFilter::Trace);
    }
    let mut args = std::env::args().skip(1);
    let role = args.next().ok_or("usage: tls_peer server|client --dir D --addr 127.0.0.1:PORT [--pin FP] [--approve-after-ms N] [--seconds N] [--plain] [--messages] [--hello-only] [--hold-ms N]")?;
    if role != "server" && role != "client" {
        return Err("role must be server or client".into());
    }
    let mut dir = None;
    let mut addr = None;
    let mut pin = None;
    let mut approve_after = None;
    let mut seconds = 5;
    let mut plain = false;
    let mut messages = false;
    let mut hello_only = false;
    let mut hold_ms = 30_000;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--dir" => dir = Some(PathBuf::from(args.next().ok_or("missing dir")?)),
            "--addr" => addr = Some(args.next().ok_or("missing addr")?.parse::<SocketAddr>()?),
            "--pin" => pin = Some(Fingerprint::parse(&args.next().ok_or("missing pin")?).ok_or("bad pin")?),
            "--approve-after-ms" => {
                approve_after = Some(Duration::from_millis(
                    args.next().ok_or("missing milliseconds")?.parse()?,
                ))
            }
            "--seconds" => seconds = args.next().ok_or("missing seconds")?.parse()?,
            "--hold-ms" => hold_ms = args.next().ok_or("missing milliseconds")?.parse()?,
            "--plain" => plain = true,
            "--messages" => messages = true,
            "--hello-only" => hello_only = true,
            _ => return Err(format!("unknown flag {flag}").into()),
        }
    }
    let addr = addr.ok_or("--addr required")?;
    if !addr.ip().is_loopback()
        || !(24970..=24979).contains(&addr.port())
            && !(24920..=24929).contains(&addr.port())
            && !(24980..=24989).contains(&addr.port())
            && !(25080..=25099).contains(&addr.port())
    {
        return Err("example requires loopback and assigned integration port ranges".into());
    }
    let config = NetConfig {
        tls_enabled: !plain,
        tls_dir: dir.ok_or("--dir required")?.join("tls"),
        identity_name: format!("WP4-{role}"),
        held_timeout: Duration::from_millis(hold_ms),
        ..NetConfig::default()
    };
    let (mut driver, handle) = NetDriver::new(config)?;
    if !messages {
        driver.set_transport_mode(TransportMode::Frames)?;
    }
    if let Some(fp) = driver.identity_fingerprint() {
        println!("identity {}", fp.to_synergy_line());
    }
    let start = Instant::now();
    let mut out = Vec::new();
    let mut held = Vec::new();
    let mut sent = false;
    let mut received = 0;
    let mut connected = false;
    driver.execute(
        if role == "server" {
            NetCommand::Listen { addr }
        } else if let Some(fingerprint) = pin {
            NetCommand::ConnectPinned {
                host: addr.ip().to_string(),
                port: addr.port(),
                fingerprint,
            }
        } else {
            NetCommand::Connect {
                host: addr.ip().to_string(),
                port: addr.port(),
            }
        },
        &mut out,
    );
    let deadline = start + Duration::from_secs(seconds);
    while Instant::now() < deadline {
        for event in std::mem::take(&mut out) {
            match event {
                NetEvent::Listening { addr } => println!("LISTENING {addr}"),
                NetEvent::UntrustedPeer { conn, fingerprint, .. } => {
                    println!(
                        "HELD conn={} elapsed_ms={} delivered={received}",
                        conn.0,
                        start.elapsed().as_millis()
                    );
                    held.push((fingerprint, Instant::now()));
                }
                NetEvent::Connected {
                    conn, tls, tls_version, ..
                } => {
                    connected = true;
                    println!(
                        "CONNECTED conn={} tls={tls} version={tls_version:?} cipher={:?} elapsed_ms={}",
                        conn.0,
                        driver.tls_cipher(conn),
                        start.elapsed().as_millis()
                    );
                    if role == "server" {
                        let command = if messages {
                            NetCommand::Send {
                                conn,
                                msg: openergy_protocol::Message::Hello {
                                    product: "Synergy".into(),
                                    version: openergy_protocol::ProtocolVersion::V1_8,
                                },
                                priority: Priority::Control,
                            }
                        } else {
                            NetCommand::SendFrame {
                                conn,
                                payload: b"Synergy\x00\x01\x00\x08".to_vec(),
                                priority: Priority::Control,
                            }
                        };
                        handle.send(command)?;
                        sent = true;
                    }
                }
                NetEvent::Frame { conn, payload } => {
                    received += 1;
                    println!(
                        "FRAME conn={} bytes={} elapsed_ms={}",
                        conn.0,
                        payload.len(),
                        start.elapsed().as_millis()
                    );
                    if payload == b"Synergy\x00\x01\x00\x08" {
                        println!("HELLO 0000000b53796e6572677900010008");
                    }
                    if role == "client" && !sent && !hello_only {
                        let mut payload = b"Synergy\x00\x01\x00\x08".to_vec();
                        payload.extend_from_slice(&12u32.to_be_bytes());
                        payload.extend_from_slice(b"openergy-mac");
                        handle.send(NetCommand::SendFrame {
                            conn,
                            payload,
                            priority: Priority::Control,
                        })?;
                        sent = true;
                    }
                }
                NetEvent::Message { conn, msg } => {
                    received += 1;
                    // Only message kinds, never input/clipboard contents.
                    let hello = matches!(msg, openergy_protocol::Message::Hello { .. });
                    println!("MESSAGE conn={} hello={hello}", conn.0);
                    if hello && role == "client" && !sent && !hello_only {
                        handle.send(NetCommand::Send {
                            conn,
                            msg: openergy_protocol::Message::HelloBack {
                                product: "Synergy".into(),
                                version: openergy_protocol::ProtocolVersion::V1_8,
                                name: "openergy-mac".into(),
                            },
                            priority: Priority::Control,
                        })?;
                        sent = true;
                    }
                }
                NetEvent::Disconnected { reason, .. } => {
                    println!("DISCONNECTED {reason:?} delivered={received}");
                    return Ok(());
                }
                NetEvent::ConnectFailed { error, .. } => {
                    println!("CONNECT_FAILED {error} delivered={received}");
                    return Ok(());
                }
                NetEvent::ListenFailed { error, .. } | NetEvent::Error { message: error, .. } => {
                    return Err(error.into())
                }
                _ => {}
            }
        }
        if let Some(delay) = approve_after {
            for (fp, since) in &mut held {
                if since.elapsed() >= delay {
                    println!(
                        "APPROVE elapsed_ms={} delivered={received}",
                        start.elapsed().as_millis()
                    );
                    driver.approve_held(fp, &mut out)?;
                    *since = deadline + delay;
                }
            }
        }
        driver.poll(Some(Duration::from_millis(20)), &mut out)?;
    }
    handle.send(NetCommand::Shutdown)?;
    for _ in 0..20 {
        driver.poll(Some(Duration::from_millis(20)), &mut out)?;
    }
    println!("DONE connected={connected} delivered={received}");
    if !connected && held.is_empty() {
        return Err("no connection completed".into());
    }
    Ok(())
}
