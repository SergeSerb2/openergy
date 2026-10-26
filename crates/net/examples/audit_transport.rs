//! End-to-end transport probes over real loopback sockets. No desktop backend.
use openergy_net::{Fingerprint, NetCommand, NetConfig, NetDriver, NetEvent, Priority, TransportMode};
use openergy_session::{ConnId, DisconnectReason};
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("audit_transport: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let case = args.next().ok_or("audit_transport CASE DIR PORT")?;
    let dir = PathBuf::from(args.next().ok_or("missing isolated DIR")?);
    let port: u16 = args.next().ok_or("missing PORT")?.parse()?;
    if args.next().is_some() || !(25080..=25099).contains(&port) {
        return Err("requires an assigned port in 25080..=25099".into());
    }
    let (mut driver, _) = NetDriver::new(NetConfig {
        tls_enabled: case == "pinned-reconfigure",
        tls_dir: dir.join("tls"),
        ..NetConfig::default()
    })?;
    driver.set_transport_mode(TransportMode::Frames)?;
    driver.set_heartbeat_interval(Duration::from_millis(if case == "close-deadline" { 4000 } else { 120 }))?;
    let mut out = Vec::new();
    if case == "guard-denied" || case == "guard-allowed" {
        // Run only against a validation copy whose protected port was redirected
        // to this assigned loopback port. The example never uses port 24800.
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        if case == "guard-denied" {
            for host in ["127.0.0.1", "localhost"] {
                out.clear();
                driver.execute(
                    NetCommand::Connect {
                        host: host.into(),
                        port,
                    },
                    &mut out,
                );
                let deadline = Instant::now() + Duration::from_secs(2);
                while out.is_empty() && Instant::now() < deadline {
                    driver.poll(Some(Duration::from_millis(10)), &mut out)?;
                }
                if out.len() != 1
                    || !matches!(&out[0], NetEvent::ConnectFailed { error, .. }
                    if error == &openergy_net::NetError::ProductionPortDenied.to_string())
                    || listener.accept().is_ok()
                {
                    return Err("outbound guard did not deny the redirected numeric/DNS endpoint before TCP".into());
                }
            }
        } else {
            driver.execute(
                NetCommand::Connect {
                    host: "localhost".into(),
                    port,
                },
                &mut out,
            );
            let deadline = Instant::now() + Duration::from_secs(2);
            let conn = loop {
                driver.poll(Some(Duration::from_millis(10)), &mut out)?;
                if let Some(conn) = out.iter().find_map(|event| match event {
                    NetEvent::Connected { conn, .. } => Some(*conn),
                    _ => None,
                }) {
                    break conn;
                }
                if Instant::now() >= deadline {
                    return Err("explicit guard opt-in did not connect to the redirected endpoint".into());
                }
            };
            let (mut peer, _) = loop {
                match listener.accept() {
                    Ok(peer) => break peer,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                        driver.poll(Some(Duration::from_millis(10)), &mut out)?;
                    }
                    Err(error) => return Err(error.into()),
                }
            };
            driver.execute_checked(
                NetCommand::SendFrame {
                    conn,
                    payload: b"PASS".to_vec(),
                    priority: Priority::Control,
                },
                &mut out,
            )?;
            while driver.conn_info(conn).is_some_and(|info| info.queued_bytes != 0) && Instant::now() < deadline {
                driver.poll(Some(Duration::from_millis(10)), &mut out)?;
            }
            peer.set_nonblocking(false)?;
            peer.set_read_timeout(Some(Duration::from_secs(1)))?;
            let mut bytes = [0; 8];
            peer.read_exact(&mut bytes)?;
            if &bytes != b"\x00\x00\x00\x04PASS" {
                return Err("opt-in frame did not cross the real redirected socket".into());
            }
        }
    } else if case == "pinned-plain" || case == "pinned-reconfigure" {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        driver.execute(
            NetCommand::ConnectPinned {
                host: if case == "pinned-reconfigure" {
                    "localhost"
                } else {
                    "127.0.0.1"
                }
                .into(),
                port,
                fingerprint: Fingerprint([0; 32]),
            },
            &mut out,
        );
        if case == "pinned-reconfigure" {
            driver.reconfigure(NetConfig {
                tls_enabled: false,
                tls_dir: dir.join("tls"),
                ..NetConfig::default()
            })?;
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while !out.iter().any(|event| matches!(event, NetEvent::ConnectFailed { .. })) && Instant::now() < deadline {
            driver.poll(Some(Duration::from_millis(10)), &mut out)?;
        }
        if !out.iter().any(|event| matches!(event, NetEvent::ConnectFailed { .. })) || listener.accept().is_ok() {
            return Err("explicit certificate pin opened a plaintext connection".into());
        }
    } else {
        driver.execute(
            NetCommand::Listen {
                addr: ([127, 0, 0, 1], port).into(),
            },
            &mut out,
        );
        let mut peer = TcpStream::connect(("127.0.0.1", port))?;
        let deadline = Instant::now() + Duration::from_secs(2);
        let conn: ConnId = loop {
            driver.poll(Some(Duration::from_millis(10)), &mut out)?;
            if let Some(conn) = out.iter().find_map(|event| match event {
                NetEvent::Connected { conn, .. } => Some(*conn),
                _ => None,
            }) {
                break conn;
            }
            if Instant::now() >= deadline {
                return Err("loopback accept timed out".into());
            }
        };
        out.clear();
        match case.as_str() {
            "priority" => {
                for (priority, payload) in [(Priority::Control, b"CTRL"), (Priority::Input, b"INPT")] {
                    driver.execute_checked(
                        NetCommand::SendFrame {
                            conn,
                            payload: payload.to_vec(),
                            priority,
                        },
                        &mut out,
                    )?;
                }
                driver.poll(Some(Duration::ZERO), &mut out)?;
                peer.set_read_timeout(Some(Duration::from_secs(1)))?;
                let mut frames = [0; 16];
                peer.read_exact(&mut frames)?;
                if &frames[4..8] != b"INPT" || &frames[12..16] != b"CTRL" {
                    return Err("established Control traffic overtook queued Input".into());
                }
            }
            "control-pressure" | "close-deadline" => {
                // A stopped real reader and five large Control frames fill the queues.
                for _ in 0..5 {
                    let mut payload = b"CTRL".to_vec();
                    payload.resize(1024 * 1024, 0);
                    driver.execute_checked(
                        NetCommand::SendFrame {
                            conn,
                            payload,
                            priority: Priority::Control,
                        },
                        &mut out,
                    )?;
                }
                let start = Instant::now();
                let expected = if case == "close-deadline" {
                    driver.execute(NetCommand::Close { conn }, &mut out);
                    DisconnectReason::Shutdown
                } else {
                    DisconnectReason::SlowPeer
                };
                loop {
                    driver.poll(Some(Duration::from_millis(10)), &mut out)?;
                    if out
                        .iter()
                        .any(|event| matches!(event, NetEvent::Disconnected { reason, .. } if *reason == expected))
                    {
                        break;
                    }
                    if start.elapsed() >= Duration::from_millis(if case == "close-deadline" { 2400 } else { 650 }) {
                        return Err(format!("{case} did not close within its fixed deadline").into());
                    }
                    // Repeated requests must not extend either watchdog.
                    if case == "close-deadline" {
                        driver.execute(NetCommand::Close { conn }, &mut out);
                    } else {
                        driver.execute_checked(
                            NetCommand::SendFrame {
                                conn,
                                payload: b"CTRL".to_vec(),
                                priority: Priority::Control,
                            },
                            &mut out,
                        )?;
                    }
                }
                println!("{case} close_ms={}", start.elapsed().as_millis());
            }
            _ => return Err("unknown CASE".into()),
        }
    }
    driver.execute(NetCommand::Shutdown, &mut out);
    println!("{case}=PASS");
    Ok(())
}
