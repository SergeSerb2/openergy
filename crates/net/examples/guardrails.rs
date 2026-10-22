//! Exercises real failed connects, a stalled socket, connect deadline, and a
//! cross-thread wake. No platform code or production/LAN sockets are used.
use openergy_net::{NetCommand, NetConfig, NetDriver, NetEvent, Priority, TransportMode};
use openergy_session::DisconnectReason;
use std::{
    net::{TcpListener, TcpStream},
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() {
    if let Err(e) = run() {
        eprintln!("guardrails: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args().nth(1).ok_or("guardrails DIR")?);
    let (mut driver, handle) = NetDriver::new(NetConfig {
        tls_enabled: false,
        tls_dir: root.join("tls"),
        ..NetConfig::default()
    })?;
    driver.set_transport_mode(TransportMode::Frames)?;
    let mut out = Vec::new();
    driver.execute(
        NetCommand::Connect {
            host: "127.0.0.1".into(),
            port: 24978,
        },
        &mut out,
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while !out.iter().any(|e| matches!(e, NetEvent::ConnectFailed { .. })) && Instant::now() < deadline {
        driver.poll(Some(Duration::from_millis(20)), &mut out)?;
    }
    if out
        .iter()
        .filter(|e| matches!(e, NetEvent::ConnectFailed { .. }))
        .count()
        != 1
        || out.iter().any(|e| matches!(e, NetEvent::Connected { .. }))
    {
        return Err("failed connect did not produce exactly one ConnectFailed".into());
    }
    println!("REFUSED exactly_one_ConnectFailed=PASS");
    out.clear();
    driver.execute(
        NetCommand::Listen {
            addr: ([127, 0, 0, 1], 24976).into(),
        },
        &mut out,
    );
    // TCP client intentionally stops reading. It is a real socket backpressure probe.
    let _socket = TcpStream::connect(("127.0.0.1", 24976))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let conn = loop {
        driver.poll(Some(Duration::from_millis(10)), &mut out)?;
        if let Some(conn) = out.iter().find_map(|e| match e {
            NetEvent::Connected { conn, .. } => Some(*conn),
            _ => None,
        }) {
            break conn;
        }
        if Instant::now() > deadline {
            return Err("accept timed out".into());
        }
    };
    out.clear();
    let start = Instant::now();
    for _ in 0..5 {
        let mut payload = b"SLOW".to_vec();
        payload.resize(1024 * 1024, 0);
        driver.execute(
            NetCommand::SendFrame {
                conn,
                payload,
                priority: Priority::Input,
            },
            &mut out,
        );
    }
    while start.elapsed() < Duration::from_secs(4)
        && !out.iter().any(|e| {
            matches!(
                e,
                NetEvent::Disconnected {
                    reason: DisconnectReason::SlowPeer,
                    ..
                }
            )
        })
    {
        driver.poll(Some(Duration::from_millis(25)), &mut out)?;
    }
    if !out.iter().any(|e| {
        matches!(
            e,
            NetEvent::Disconnected {
                reason: DisconnectReason::SlowPeer,
                ..
            }
        )
    }) {
        return Err("no SlowPeer close".into());
    }
    println!(
        "SLOW_PEER queued_nonbulk_over_4MiB close_ms={} reason=SlowPeer",
        start.elapsed().as_millis()
    );
    driver.execute(NetCommand::Shutdown, &mut out);
    out.clear();
    let blocker = TcpListener::bind(("127.0.0.1", 24977))?;
    driver.reconfigure(NetConfig {
        tls_dir: root.join("tls"),
        ..NetConfig::default()
    })?;
    driver.set_connect_timeout(Duration::from_millis(120))?;
    driver.execute(
        NetCommand::Connect {
            host: "127.0.0.1".into(),
            port: 24977,
        },
        &mut out,
    );
    let start = Instant::now();
    while !out.iter().any(|e| matches!(e, NetEvent::ConnectFailed { .. })) && start.elapsed() < Duration::from_secs(1) {
        driver.poll(Some(Duration::from_secs(1)), &mut out)?;
    }
    if out
        .iter()
        .filter(|e| matches!(e, NetEvent::ConnectFailed { .. }))
        .count()
        != 1
        || start.elapsed() > Duration::from_millis(500)
    {
        return Err("TLS handshake deadline failed".into());
    }
    println!(
        "HANDSHAKE_DEADLINE close_ms={} exactly_one_ConnectFailed=PASS",
        start.elapsed().as_millis()
    );
    blocker.set_nonblocking(true)?;
    let (_accepted, _) = blocker
        .accept()
        .map_err(|_| "loopback connector did not reach its listener")?;
    println!("TCP loopback_listener_accepted=PASS");
    drop(blocker);
    out.clear();
    driver.execute(NetCommand::Shutdown, &mut out);
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(40));
        handle.send(NetCommand::Listen {
            addr: ([127, 0, 0, 1], 24976).into(),
        })
    });
    let start = Instant::now();
    out.clear();
    // Drain old socket notifications before timing a truly blocking poll.
    driver.poll(Some(Duration::ZERO), &mut out)?;
    out.clear();
    while !out.iter().any(|e| matches!(e, NetEvent::Listening { .. })) && start.elapsed() < Duration::from_secs(2) {
        driver.poll(Some(Duration::from_secs(2)), &mut out)?;
    }
    thread.join().map_err(|_| "command producer failed")??;
    if !out.iter().any(|e| matches!(e, NetEvent::Listening { .. })) || start.elapsed() > Duration::from_millis(500) {
        return Err("cross-thread wake failed".into());
    }
    println!(
        "CROSS_THREAD_WAKER wake_ms={} reused_driver=PASS",
        start.elapsed().as_millis()
    );
    driver.execute(NetCommand::Shutdown, &mut out);
    driver.reconfigure(NetConfig {
        tls_enabled: false,
        tls_dir: root.join("tls"),
        ..NetConfig::default()
    })?;
    driver.execute(
        NetCommand::Listen {
            addr: ([127, 0, 0, 1], 24976).into(),
        },
        &mut out,
    );
    let mut writer = TcpStream::connect(("127.0.0.1", 24976))?;
    writer.set_write_timeout(Some(Duration::from_millis(100)))?;
    let producer = std::thread::spawn(move || {
        use std::io::Write;
        let frame = b"\x00\x00\x00\x04CNOP";
        let bytes = frame.repeat(4096);
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(7) && writer.write_all(&bytes).is_ok() {
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    let start = Instant::now();
    let mut frames = 0u64;
    let mut flood = false;
    while start.elapsed() < Duration::from_secs(7) && !flood {
        out.clear();
        driver.poll(Some(Duration::from_millis(20)), &mut out)?;
        frames += out.iter().filter(|e| matches!(e, NetEvent::Frame { .. })).count() as u64;
        flood = out.iter().any(|e| {
            matches!(
                e,
                NetEvent::Disconnected {
                    reason: DisconnectReason::Flood,
                    ..
                }
            )
        });
    }
    driver.execute(NetCommand::Shutdown, &mut out);
    producer.join().map_err(|_| "frame producer failed")?;
    if !flood {
        return Err("flood watchdog did not close the real sender".into());
    }
    println!(
        "FLOOD valid_frames={frames} close_ms={} reason=Flood",
        start.elapsed().as_millis()
    );
    Ok(())
}
