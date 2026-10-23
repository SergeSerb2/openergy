//! Real NetDriver commands and private loopback sockets on TCP 25100..=25119.
//! No resolver replacement, global DNS changes, discovery, TLS, or native backend.
//! A nanosecond deadline expires real localhost jobs without a slow-DNS fixture.
use openergy_net::{ConnectErrorKind, NetCommand, NetConfig, NetDriver, NetError, NetEvent, Priority, TransportMode};
use openergy_session::ConnId;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEALTHY: u16 = 25116;
const REFUSED: u16 = 25117;
const INBOUND: u16 = 25118;
const LIMIT: Duration = Duration::from_secs(3);

fn config(root: &Path) -> NetConfig {
    NetConfig {
        tls_enabled: false,
        tls_dir: root.join("tls"),
        mdns: None,
        ..NetConfig::default()
    }
}
fn connect(host: &str, port: u16) -> NetCommand {
    NetCommand::Connect {
        host: host.into(),
        port,
    }
}
fn poll_until(driver: &mut NetDriver, out: &mut Vec<NetEvent>, done: impl Fn(&[NetEvent]) -> bool) -> Result<()> {
    let started = Instant::now();
    while !done(out) && started.elapsed() < LIMIT {
        driver.poll(Some(Duration::from_millis(1)), out)?;
    }
    if !done(out) {
        return Err(format!("event deadline: {out:?}").into());
    }
    Ok(())
}
fn connected(out: &[NetEvent]) -> Option<ConnId> {
    out.iter().find_map(|event| match event {
        NetEvent::Connected { conn, .. } => Some(*conn),
        _ => None,
    })
}
fn accept(driver: &mut NetDriver, listener: &TcpListener) -> Result<TcpStream> {
    let started = Instant::now();
    let (peer, addr) = loop {
        match listener.accept() {
            Ok(peer) => break peer,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock && started.elapsed() < LIMIT => {
                let mut events = Vec::new();
                driver.poll(Some(Duration::from_micros(100)), &mut events)?;
                if !events.is_empty() {
                    return Err(format!("events while waiting for accept: {events:?}").into());
                }
            }
            Err(error) => return Err(error.into()),
        }
    };
    if !addr.ip().is_loopback() {
        return Err("non-loopback peer".into());
    }
    peer.set_nonblocking(true)?;
    Ok(peer)
}
fn no_accept(listeners: &[TcpListener]) -> Result<()> {
    for listener in listeners {
        match listener.accept() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            result => return Err(format!("stale job opened TCP: {result:?}").into()),
        }
    }
    Ok(())
}
fn failure(out: &[NetEvent], port: u16, kind: ConnectErrorKind) -> bool {
    let matching: Vec<_> = out
        .iter()
        .filter_map(|event| match event {
            NetEvent::ConnectFailed {
                port: actual, error, ..
            } if *actual == port => Some(error),
            _ => None,
        })
        .collect();
    matching.len() == 1 && ConnectErrorKind::from_message(matching[0]) == kind
}
fn exchange(driver: &mut NetDriver, conn: ConnId, peer: &mut TcpStream) -> Result<()> {
    let mut out = Vec::new();
    driver.execute_checked(
        NetCommand::SendFrame {
            conn,
            payload: b"PASS".to_vec(),
            priority: Priority::Control,
        },
        &mut out,
    )?;
    let started = Instant::now();
    let mut received = Vec::new();
    while received.len() < 8 && started.elapsed() < LIMIT {
        driver.poll(Some(Duration::from_millis(1)), &mut out)?;
        let mut buf = [0; 8];
        match peer.read(&mut buf) {
            Ok(0) => return Err("peer closed during frame exchange".into()),
            Ok(n) => received.extend_from_slice(&buf[..n]),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
    }
    if received != b"\0\0\0\x04PASS" {
        return Err(format!("wrong outbound frame: {received:?}").into());
    }
    peer.write_all(b"\0\0\0\x04BACK")?;
    poll_until(driver, &mut out, |events| {
        events
            .iter()
            .any(|event| matches!(event, NetEvent::Frame { conn: id, payload } if *id == conn && payload == b"BACK"))
    })
}

fn queue_round(
    driver: &mut NetDriver,
    stale: &[TcpListener],
    healthy: &TcpListener,
    round: usize,
    expire: bool,
) -> Result<()> {
    let mut out = Vec::new();
    driver.set_connect_timeout(if expire { Duration::from_nanos(1) } else { LIMIT })?;
    let start = Instant::now();
    for port in 25100..=25115 {
        driver.execute_checked(connect("localhost", port), &mut out)?;
    }
    if !out.is_empty() {
        return Err(format!("16-job admission failed: {out:?}").into());
    }
    let admitted_us = start.elapsed().as_micros();
    driver.execute_checked(connect("localhost", HEALTHY), &mut out)?;
    if out.len() != 1 || !matches!(&out[0], NetEvent::ConnectFailed { error, .. } if error == "16-connection limit") {
        return Err(format!("17th pending job was not rejected: {out:?}").into());
    }
    out.clear();
    if expire {
        driver.poll(Some(Duration::ZERO), &mut out)?;
    } else {
        driver.execute_checked(
            if round.is_multiple_of(2) {
                NetCommand::Shutdown
            } else {
                NetCommand::DisconnectAll
            },
            &mut out,
        )?;
    }
    let mut obsolete = std::mem::take(&mut out);
    for port in 25100..=25115 {
        let errors: Vec<_> = obsolete
            .iter()
            .filter_map(|event| match event {
                NetEvent::ConnectFailed {
                    port: actual, error, ..
                } if *actual == port => Some(error),
                _ => None,
            })
            .collect();
        if errors.len() != 1
            || if expire {
                ConnectErrorKind::from_message(errors[0]) != ConnectErrorKind::Timeout
            } else {
                errors[0] != "shutdown"
            }
        {
            return Err(format!("expected exactly one stale-job failure per port: {obsolete:?}").into());
        }
    }
    if obsolete.len() != 16 {
        return Err(format!("unexpected stale-job events: {obsolete:?}").into());
    }
    driver.set_connect_timeout(LIMIT)?;
    let recovery = Instant::now();
    let mut rejects = 0;
    loop {
        driver.execute_checked(connect("localhost", HEALTHY), &mut out)?;
        if out.is_empty() {
            break;
        }
        if out.len() != 1 || !matches!(&out[0], NetEvent::ConnectFailed { error, .. } if error == "DNS request limit") {
            return Err(format!("unexpected recovery admission error: {out:?}").into());
        }
        rejects += 1;
        if recovery.elapsed() >= LIMIT || rejects >= 128 {
            return Err("DNS budget did not recover".into());
        }
        out.clear();
        driver.poll(Some(Duration::from_micros(100)), &mut obsolete)?;
    }
    let admission_us = recovery.elapsed().as_micros();
    if !expire && rejects == 0 {
        return Err("cancellation released outstanding slots before completion accounting".into());
    }
    poll_until(driver, &mut out, |events| connected(events).is_some())?;
    let connected_us = recovery.elapsed().as_micros();
    let conn = connected(&out).ok_or("missing healthy connection")?;
    let mut peer = accept(driver, healthy).map_err(|error| {
        format!(
            "healthy accept failed: {error}; events={out:?}; conn_info={:?}",
            driver.conn_info(conn)
        )
    })?;
    exchange(driver, conn, &mut peer)?;
    let roundtrip_us = recovery.elapsed().as_micros();
    // Process every accepted old completion before checking late-result behavior.
    for _ in 0..20 {
        driver.poll(Some(Duration::from_millis(1)), &mut obsolete)?;
    }
    if obsolete.len() != 16 {
        return Err(format!("late result emitted extra events: {obsolete:?}").into());
    }
    no_accept(stale)?;
    driver.execute_checked(NetCommand::Shutdown, &mut out)?;
    println!(
        "QUEUE mode={} round={round} jobs=16 admission_us={admitted_us} overflow_rejected=PASS dns_budget_rejections={rejects} recovery_admission_us={admission_us} recovery_connected_us={connected_us} recovery_roundtrip_us={roundtrip_us} exactly_one_failure=PASS stale_tcp=NONE frame_roundtrip=PASS",
        if expire { "deadline" } else if round.is_multiple_of(2) { "shutdown" } else { "disconnect_all" },
    );
    Ok(())
}

fn contracts(root: &Path, healthy: &TcpListener) -> Result<bool> {
    let (mut driver, handle) = NetDriver::new(config(root))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    let waker = driver.waker();
    let mut out = Vec::new();
    // Embedded NUL exercises the real std/OS resolver input failure without public DNS traffic.
    driver.execute_checked(connect("invalid\0hostname", REFUSED), &mut out)?;
    poll_until(&mut driver, &mut out, |events| !events.is_empty())?;
    if out.len() != 1 || !failure(&out, REFUSED, ConnectErrorKind::Dns) {
        return Err(format!("DNS failure classification: {out:?}").into());
    }
    println!("FAILURE case=invalid_hostname events={out:?} classified_dns=PASS");
    out.clear();
    // Confirm the assigned refusal port is free; never disconnect another owner.
    drop(TcpListener::bind(("127.0.0.1", REFUSED))?);
    driver.execute_checked(connect("localhost", REFUSED), &mut out)?;
    poll_until(&mut driver, &mut out, |events| {
        events
            .iter()
            .any(|event| matches!(event, NetEvent::ConnectFailed { .. } | NetEvent::Disconnected { .. }))
    })?;
    let refused = out.len() == 1 && failure(&out, REFUSED, ConnectErrorKind::Refused);
    println!(
        "FAILURE case=refused_localhost events={out:?} classified_refused={}",
        if refused { "PASS" } else { "FAIL" }
    );
    out.clear();
    // Real listener and inbound socket verify the original handle survives both resets.
    let inbound = ([127, 0, 0, 1], INBOUND).into();
    handle.send(NetCommand::Listen { addr: inbound })?;
    poll_until(&mut driver, &mut out, |events| {
        events.iter().any(|event| matches!(event, NetEvent::Listening { .. }))
    })?;
    out.clear();
    let _first = TcpStream::connect(inbound)?;
    poll_until(&mut driver, &mut out, |events| connected(events).is_some())?;
    out.clear();
    handle.send(NetCommand::DisconnectAll)?;
    poll_until(&mut driver, &mut out, |events| {
        events
            .iter()
            .any(|event| matches!(event, NetEvent::Disconnected { .. }))
    })?;
    out.clear();
    let _second = TcpStream::connect(inbound)?;
    poll_until(&mut driver, &mut out, |events| connected(events).is_some())?;
    out.clear();
    handle.send(NetCommand::Shutdown)?;
    poll_until(&mut driver, &mut out, |events| {
        events
            .iter()
            .any(|event| matches!(event, NetEvent::Disconnected { .. }))
    })?;
    if TcpStream::connect_timeout(&inbound, Duration::from_millis(100)).is_ok() {
        return Err("Shutdown retained listener".into());
    }
    out.clear();
    driver.reconfigure(config(&root.join("reconfigured")))?;
    let mut invalid = config(&root.join("invalid"));
    invalid.max_frame_len = 0;
    if driver.reconfigure(invalid).is_ok() || !Arc::ptr_eq(&waker, &driver.waker()) {
        return Err("reconfigure validation or waker reuse failed".into());
    }
    handle.send(connect("localhost", HEALTHY))?;
    poll_until(&mut driver, &mut out, |events| connected(events).is_some())?;
    let conn = connected(&out).ok_or("original handle failed after reconfigure")?;
    let mut peer = accept(&mut driver, healthy)?;
    exchange(&mut driver, conn, &mut peer)?;
    out.clear();
    handle.send(NetCommand::Shutdown)?;
    poll_until(&mut driver, &mut out, |events| {
        events
            .iter()
            .any(|event| matches!(event, NetEvent::Disconnected { .. }))
    })?;
    out.clear();
    driver.execute_checked(connect("127.0.0.1", HEALTHY), &mut out)?;
    poll_until(&mut driver, &mut out, |events| connected(events).is_some())?;
    let mut peer = accept(&mut driver, healthy)?;
    exchange(
        &mut driver,
        connected(&out).ok_or("numeric recovery failed")?,
        &mut peer,
    )?;
    drop(driver);
    if handle.send(connect("localhost", HEALTHY)) != Err(NetError::Shutdown) {
        return Err("old handle admitted work after driver drop".into());
    }
    println!("CONTRACTS same_driver=PASS same_handle=PASS same_waker=PASS disconnect_all_keeps_listener=PASS shutdown_closes_listener=PASS reconfigure=PASS invalid_reconfigure_recovery=PASS localhost=PASS numeric_loopback=PASS driver_drop=PASS");
    Ok(refused)
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().ok_or("dns_queue_cancel_c4806a3d NEW_ISOLATED_DIR")?);
    if args.next().is_some() || !root.is_absolute() || root.exists() {
        return Err("requires a new absolute isolated directory".into());
    }
    std::fs::create_dir(&root)?;
    let mut stale = Vec::new();
    for port in 25100..=25115 {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        stale.push(listener);
    }
    let healthy = TcpListener::bind(("127.0.0.1", HEALTHY))?;
    healthy.set_nonblocking(true)?;
    let (mut driver, _) = NetDriver::new(config(&root.join("queue")))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    println!("SCOPE resolver=real_std_to_socket_addrs host=localhost ports=25100..25118 discovery=OFF tls=OFF native=NONE slow_lookup_control=UNAVAILABLE");
    for round in 0..4 {
        queue_round(&mut driver, &stale, &healthy, round, false)?;
    }
    for round in 0..4 {
        queue_round(&mut driver, &stale, &healthy, round, true)?;
    }
    drop(driver);
    let refused = contracts(&root.join("contracts"), &healthy)?;
    no_accept(&stale)?;
    println!(
        "RESULT queue_and_lifecycle=PASS refused_localhost_control={} slow_dns_head_of_line_regression=NOT_MEASURED",
        if refused { "PASS" } else { "FAIL" }
    );
    if !refused {
        return Err("real localhost refusal control did not emit a typed refused failure; see raw events".into());
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("dns_queue_cancel_c4806a3d: {error}");
        std::process::exit(1);
    }
}
