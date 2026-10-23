//! Supplementary native Windows diagnosis. Not the original acceptance probe.
//! Compiles against the exact verified 07adb0a native library; source stays frozen.
//! Runs real private NetDriver commands and loopback sockets only.
use openergy_net::{ConnectErrorKind, NetCommand, NetConfig, NetDriver, NetError, NetEvent, Priority, TransportMode};
use openergy_session::ConnId;
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEALTHY: u16 = 25116;
const REFUSED: u16 = 25117;
const INBOUND: u16 = 25118;
const LIMIT: Duration = Duration::from_secs(3);


use std::os::windows::io::{AsRawSocket, FromRawSocket, RawSocket};

#[link(name = "ws2_32")]
extern "system" {
    fn socket(family: i32, kind: i32, protocol: i32) -> RawSocket;
    fn setsockopt(socket: RawSocket, level: i32, name: i32, value: *const u8, len: i32) -> i32;
    fn getsockopt(socket: RawSocket, level: i32, name: i32, value: *mut u8, len: *mut i32) -> i32;
    fn bind(socket: RawSocket, address: *const u8, len: i32) -> i32;
    fn listen(socket: RawSocket, backlog: i32) -> i32;
    fn WSAGetLastError() -> i32;
}
#[repr(C)]
struct AddressV4 { family: u16, port: u16, address: [u8; 4], zero: [u8; 8] }
#[repr(C)]
struct AddressV6 { family: u16, port: u16, flow: u32, address: [u8; 16], scope: u32 }
fn wsa_result(code: i32) -> io::Result<()> {
    if code == 0 { Ok(()) } else { Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() })) }
}
fn unused_exclusive(addr: SocketAddr) -> Result<()> {
    if !addr.ip().is_loopback() || ![25117, 25119].contains(&addr.port()) {
        return Err("exclusive control outside assigned loopback refusal ports".into());
    }
    // Real std resolver runs before this call and initializes Winsock.
    let raw = unsafe { socket(if addr.is_ipv4() { 2 } else { 23 }, 1, 6) };
    if raw == RawSocket::MAX { return Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() }).into()); }
    // RAII closes only this owned socket on every return path.
    let listener = unsafe { TcpListener::from_raw_socket(raw) };
    let exclusive: i32 = 1;
    wsa_result(unsafe { setsockopt(raw, 0xffff, -5, (&exclusive as *const i32).cast(), 4) })?;
    let mut observed: i32 = 0;
    let mut len: i32 = 4;
    wsa_result(unsafe { getsockopt(raw, 0xffff, -5, (&mut observed as *mut i32).cast(), &mut len) })?;
    if observed != 1 || len != 4 || listener.as_raw_socket() != raw { return Err("exclusive socket option mismatch".into()); }
    match addr {
        SocketAddr::V4(addr) => {
            let value = AddressV4 { family: 2, port: addr.port().to_be(), address: addr.ip().octets(), zero: [0; 8] };
            wsa_result(unsafe { bind(raw, (&value as *const AddressV4).cast(), std::mem::size_of::<AddressV4>() as i32) })?;
        }
        SocketAddr::V6(addr) => {
            let value = AddressV6 { family: 23, port: addr.port().to_be(), flow: 0, address: addr.ip().octets(), scope: 0 };
            wsa_result(unsafe { bind(raw, (&value as *const AddressV6).cast(), std::mem::size_of::<AddressV6>() as i32) })?;
        }
    }
    wsa_result(unsafe { listen(raw, 1) })?;
    match TcpListener::bind(addr) {
        Err(error) if error.kind() == io::ErrorKind::AddrInUse || error.kind() == io::ErrorKind::PermissionDenied => {
            println!("EXCLUSIVE_BIND addr={addr} SO_EXCLUSIVEADDRUSE={observed} duplicate_kind={:?} duplicate_errno={:?} result=PASS", error.kind(), error.raw_os_error());
        }
        result => return Err(format!("exclusive bind did not reject duplicate at {addr}: {result:?}").into()),
    }
    drop(listener);
    Ok(())
}
fn resolver(port: u16) -> Result<()> {
    let addresses: Vec<_> = ("localhost", port).to_socket_addrs()?.collect();
    println!("RESOLVER host=localhost addresses={addresses:?}");
    if addresses.is_empty() || addresses.iter().any(|addr| !addr.ip().is_loopback()) { return Err("localhost did not resolve only to loopback".into()); }
    Ok(())
}
fn direct_os() -> Result<()> {
    let mut short_failures = 0;
    let mut long_failures = 0;
    for port in [25117, 25119] {
        resolver(port)?;
        for host in ["127.0.0.1", "::1"] {
            let addr = SocketAddr::new(host.parse()?, port);
            for window_ms in [500, 6000] {
                unused_exclusive(addr)?;
                let window = Duration::from_millis(window_ms);
                let started = Instant::now();
                let outcome = TcpStream::connect_timeout(&addr, window);
                let elapsed = started.elapsed();
                let refused = matches!(&outcome, Err(error) if error.kind() == io::ErrorKind::ConnectionRefused) && elapsed <= window;
                match &outcome {
                    Ok(_) => println!("OS_CONNECT addr={addr} window_ms={window_ms} elapsed_us={} outcome=CONNECTED refused_check=FAIL", elapsed.as_micros()),
                    Err(error) => println!("OS_CONNECT addr={addr} window_ms={window_ms} elapsed_us={} kind={:?} errno={:?} error={error:?} refused_check={}", elapsed.as_micros(), error.kind(), error.raw_os_error(), if refused { "PASS" } else { "FAIL" }),
                }
                if !refused { if window_ms == 500 { short_failures += 1; } else { long_failures += 1; } }
                drop(outcome);
            }
        }
    }
    println!("OS_RESULT short_window_failures={short_failures} long_window_failures={long_failures} original_probe=UNCHANGED_FAIL windows_acceptance=NOT_GRANTED");
    if short_failures != 0 || long_failures != 0 { return Err("direct OS matrix has strict ConnectionRefused failures; see separate short/long observations".into()); }
    Ok(())
}
fn driver_refusals(root: &Path) -> Result<()> {
    let (mut driver, _) = NetDriver::new(config(root))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    driver.set_connect_timeout(Duration::from_secs(5))?;
    let mut failures = 0;
    for port in [25117, 25119] {
        resolver(port)?;
        for host in ["127.0.0.1", "::1"] { unused_exclusive(SocketAddr::new(host.parse()?, port))?; }
        for host in ["127.0.0.1", "::1", "localhost", "localhost", "localhost"] {
            let mut out = Vec::new();
            let started = Instant::now();
            let mut first_event_us = None;
            driver.execute_checked(connect(host, port), &mut out)?;
            // Keep the original 5-second connect deadline and a full 6-second
            // event window, including observations after the first event.
            while started.elapsed() < Duration::from_secs(6) {
                driver.poll(Some(Duration::from_millis(1)), &mut out)?;
                if first_event_us.is_none() && !out.is_empty() { first_event_us = Some(started.elapsed().as_micros()); }
            }
            let no_connected = !out.iter().any(|event| matches!(event, NetEvent::Connected { .. }));
            let typed = matches!(out.as_slice(), [NetEvent::ConnectFailed { host: actual, port: actual_port, error }]
                if actual == host && *actual_port == port && error.starts_with("connect_refused:") && ConnectErrorKind::from_message(error) == ConnectErrorKind::Refused);
            let passed = typed && no_connected;
            println!("DRIVER_REFUSAL host={host} port={port} connect_deadline_ms=5000 observation_ms=6000 first_event_us={first_event_us:?} elapsed_us={} events={out:?} exact_typed_connect_refused={} no_false_connected={} exactly_one_event={} result={}", started.elapsed().as_micros(), if typed { "PASS" } else { "FAIL" }, if no_connected { "PASS" } else { "FAIL" }, if out.len() == 1 { "PASS" } else { "FAIL" }, if passed { "PASS" } else { "FAIL" });
            if !passed { failures += 1; }
        }
    }
    let mut out = Vec::new();
    driver.execute_checked(NetCommand::Shutdown, &mut out)?;
    println!("DRIVER_RESULT cases=10 failures={failures} original_os_prerequisite=BYPASSED_ONLY_FOR_THIS_SUPPLEMENT windows_acceptance=NOT_GRANTED");
    if failures != 0 { return Err("supplementary NetDriver refusal matrix failed".into()); }
    Ok(())
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or("expected os, driver, or contracts mode")?;
    let root = PathBuf::from(args.next().ok_or("expected NEW_ABSOLUTE_PRIVATE_DIRECTORY")?);
    if args.next().is_some() || !root.is_absolute() || root.exists() { return Err("requires new absolute private runtime directory".into()); }
    std::fs::create_dir(&root)?;
    println!("SUPPLEMENT source=07adb0a3cdc4cff1aa77ce31dbe078a48db56bd5 mode={mode} discovery=OFF tls=OFF native=NONE original_probe=UNCHANGED_FAIL");
    resolver(25117)?;
    match mode.as_str() {
        "os" => direct_os(),
        "driver" => driver_refusals(&root),
        "contracts" => {
            let healthy = TcpListener::bind(("127.0.0.1", HEALTHY))?;
            healthy.set_nonblocking(true)?;
            contracts(&root, &healthy)
        }
        _ => Err("unknown diagnostic mode".into()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("windows_dns_supplement: {error}");
        std::process::exit(1);
    }
}

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


fn contracts(root: &Path, healthy: &TcpListener) -> Result<()> {
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
    driver.set_connect_timeout(LIMIT)?;
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
    driver.execute_checked(NetCommand::Shutdown, &mut out)?;
    let healthy_v6 = TcpListener::bind(("::1", HEALTHY))?;
    healthy_v6.set_nonblocking(true)?;
    for host in ["::1", "localhost"] {
        out.clear();
        driver.execute_checked(connect(host, HEALTHY), &mut out)?;
        poll_until(&mut driver, &mut out, |events| connected(events).is_some())?;
        let conn = connected(&out).ok_or("IPv6 healthy connection missing")?;
        let peer_addr = driver.conn_info(conn).ok_or("IPv6 connection info missing")?.peer;
        let listener = if peer_addr.is_ipv6() { &healthy_v6 } else { healthy };
        if peer_addr != listener.local_addr()? {
            return Err(format!("unexpected healthy peer: {peer_addr}").into());
        }
        let mut peer = accept(&mut driver, listener)?;
        exchange(&mut driver, conn, &mut peer)?;
        println!("HEALTHY host={host} peer={peer_addr} frame_roundtrip=PASS");
        driver.execute_checked(NetCommand::Shutdown, &mut out)?;
    }
    drop(driver);
    if handle.send(connect("localhost", HEALTHY)) != Err(NetError::Shutdown) {
        return Err("old handle admitted work after driver drop".into());
    }
    println!("CONTRACTS same_driver=PASS same_handle=PASS same_waker=PASS disconnect_all_keeps_listener=PASS shutdown_closes_listener=PASS reconfigure=PASS invalid_reconfigure_recovery=PASS localhost=PASS numeric_loopback=PASS driver_drop=PASS");
    Ok(())
}

