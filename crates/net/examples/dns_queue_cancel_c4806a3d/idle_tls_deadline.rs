//! One separate caller workflow. Deliberate caller idle is not product grace.
use super::*;
use openergy_net::NetHandle;

const BUDGET: Duration = Duration::from_millis(500);
const PREPARE: Duration = Duration::from_millis(250);
const RUNTIME: Duration = Duration::from_secs(12);
const HELLO_BYTES: usize = 16 * 1024;

struct Journal {
    epoch: Instant,
    calls: usize,
    caller: Vec<NetEvent>,
    peer: Vec<NetEvent>,
}
impl Journal {
    fn ns(&self, at: Instant) -> u128 {
        at.duration_since(self.epoch).as_nanos()
    }
    fn call(&mut self, driver: &mut NetDriver, peer: bool, phase: &str, command: Option<NetCommand>) -> Result<Call> {
        if self.calls >= MAX_CALLS || self.epoch.elapsed() >= RUNTIME {
            return Err("bounded call/runtime limit".into());
        }
        let side = if peer { "peer" } else { "caller" };
        println!("OP index={} side={side} phase={phase} command={command:?}", self.calls);
        let events = if peer { &mut self.peer } else { &mut self.caller };
        let before = events.len();
        let enter = Instant::now();
        let result = match command {
            Some(command) => driver.execute_checked(command, events),
            None => driver.poll(Some(Duration::from_millis(1)), events),
        };
        let returned = Instant::now();
        for (index, event) in events.iter().enumerate().skip(before) {
            println!("EVENT call={} side={side} index={index} value={event:?}", self.calls);
        }
        println!(
            "CALL index={} side={side} phase={phase} enter_ns={} return_ns={} result={result:?}",
            self.calls,
            self.ns(enter),
            self.ns(returned)
        );
        self.calls += 1;
        result?;
        if self.caller.len() + self.peer.len() > MAX_EVENTS {
            return Err("bounded public event limit".into());
        }
        Ok(Call { enter, returned })
    }
    fn submit(&self, handle: &NetHandle, phase: &str, command: NetCommand) -> Result<()> {
        println!("HANDLE phase={phase} command={command:?}");
        let enter = Instant::now();
        let result = handle.send(command);
        let returned = Instant::now();
        println!(
            "HANDLE_CALL phase={phase} enter_ns={} return_ns={} result={result:?}",
            self.ns(enter),
            self.ns(returned)
        );
        result?;
        Ok(())
    }
}

fn no_accept(listener: &TcpListener) -> Result<()> {
    match listener.accept() {
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(()),
        result => Err(format!("unexpected additional owned TCP accept: {result:?}").into()),
    }
}
fn timeout(events: &[NetEvent], addr: SocketAddr) -> bool {
    matches!(events, [NetEvent::ConnectFailed { host, port, error }]
        if host == "127.0.0.1" && *port == addr.port()
        && error.starts_with("connect_timeout:")
        && ConnectErrorKind::from_message(error) == ConnectErrorKind::Timeout)
}
fn trace_conn(addr: SocketAddr, command: &Call, cutoff: Instant) -> Option<ConnId> {
    let state = OBSERVER.0.lock().expect("observer lock");
    if state.1 {
        return None;
    }
    let records: Vec<_> = state.0.iter().map(transition).collect::<Option<_>>()?;
    let [attempt, ready] = records.as_slice() else {
        return None;
    };
    (attempt.phase == "attempt"
        && attempt.detail == Some("true")
        && ready.phase == "tcp_ready"
        && ready.detail == Some("true")
        && attempt.conn == ready.conn
        && attempt.addr == addr
        && ready.addr == addr
        && attempt.deadline == ready.deadline
        && attempt.at >= command.enter
        && attempt.at <= command.returned
        && attempt.at <= ready.at
        && ready.at < cutoff)
        .then_some(ConnId(attempt.conn))
}

fn case(root: &Path) -> Result<Verdict> {
    log::set_logger(&OBSERVER).map_err(|error| format!("observer initialization: {error}"))?;
    log::set_max_level(log::LevelFilter::Trace);
    *OBSERVER.0.lock().expect("observer lock") = (Vec::new(), false);
    let mut journal = Journal {
        epoch: Instant::now(),
        calls: 0,
        caller: Vec::new(),
        peer: Vec::new(),
    };
    println!("SCOPE case=idle_tls_deadline_only epoch={:?} caller_budget_ns={} prepare_limit_ns={} runtime_limit_ns={} max_tcp_connections=2 max_calls={MAX_CALLS} max_events={MAX_EVENTS} max_traces={MAX_TRACE} max_hello_bytes={HELLO_BYTES} discovery=OFF native=NONE", journal.epoch, BUDGET.as_nanos(), PREPARE.as_nanos(), RUNTIME.as_nanos());
    let mut caller_config = config(&root.join("caller"));
    caller_config.tls_enabled = true;
    let (mut caller, handle) = NetDriver::new(caller_config)?;
    caller.set_transport_mode(TransportMode::Frames)?;
    caller.set_connect_timeout(BUDGET)?;
    let waker = caller.waker();
    let mut peer_config = config(&root.join("peer"));
    peer_config.tls_enabled = true;
    peer_config.require_client_trust = false;
    let (mut peer, _) = NetDriver::new(peer_config)?;
    peer.set_transport_mode(TransportMode::Frames)?;
    peer.set_connect_timeout(BUDGET)?;
    let fingerprint = peer.identity_fingerprint().ok_or("owned TLS peer identity missing")?;
    let stalled_addr = SocketAddr::from(([127, 0, 0, 1], 25119));
    let healthy_addr = SocketAddr::from(([127, 0, 0, 1], INBOUND));
    let mut listener = None;
    let mut client = None;
    let mut owned_addrs = Vec::new();
    let mut qualified = false;
    let result = (|| -> Result<()> {
        #[cfg(unix)]
        let owned = bound_socket::listen(bound_socket::bind(stalled_addr)?)?;
        #[cfg(windows)]
        let owned = bound_socket::listen_owned(bound_socket::bind(stalled_addr)?)?;
        owned_addrs.push(stalled_addr);
        owned.set_nonblocking(true)?;
        println!(
            "OWNER stalled_listener={stalled_addr} local_addr={:?}",
            owned.local_addr()?
        );
        listener = Some(owned);
        let owned = listener.as_ref().ok_or("stalled listener missing")?;
        match TcpListener::bind(stalled_addr) {
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                println!("OWNER competing_bind={error:?} exclusive=PASS")
            }
            result => return Err(format!("exclusive stalled owner unavailable: {result:?}").into()),
        }
        journal.call(
            &mut peer,
            true,
            "listen",
            Some(NetCommand::Listen { addr: healthy_addr }),
        )?;
        if !matches!(journal.peer.as_slice(), [NetEvent::Listening { addr }] if *addr == healthy_addr) {
            return Err("healthy listener setup not exact".into());
        }
        owned_addrs.push(healthy_addr);
        let command = journal.call(
            &mut caller,
            false,
            "pending_execute_checked",
            Some(NetCommand::ConnectPinned {
                host: "127.0.0.1".into(),
                port: stalled_addr.port(),
                fingerprint,
            }),
        )?;
        let lower = command.enter + BUDGET;
        let upper = command.returned + BUDGET;
        let cutoff = command.enter + PREPARE;
        println!(
            "DEADLINE execute_entry_ns={} execute_return_ns={} earliest_ns={} latest_ns={} prepare_cutoff_ns={}",
            journal.ns(command.enter),
            journal.ns(command.returned),
            journal.ns(lower),
            journal.ns(upper),
            journal.ns(cutoff)
        );
        let mut acceptor = rustls::server::Acceptor::default();
        let mut bytes = Vec::new();
        let pending_conn = loop {
            if Instant::now() >= cutoff || !journal.caller.is_empty() {
                return Err("pending TLS precondition unavailable before preparation cutoff".into());
            }
            if client.is_none() {
                match owned.accept() {
                    Ok((stream, source)) if source.ip().is_loopback() => {
                        stream.set_nonblocking(true)?;
                        println!(
                            "ACCEPT at_ns={} listener={stalled_addr} source={source} local={:?}",
                            journal.ns(Instant::now()),
                            stream.local_addr()?
                        );
                        client = Some(stream);
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    result => return Err(format!("owned stalled accept: {result:?}").into()),
                }
            }
            if let Some(stream) = &mut client {
                no_accept(owned)?;
                let mut buf = [0; 4096];
                match stream.read(&mut buf) {
                    Ok(0) => return Err("stalled client closed before ClientHello".into()),
                    Ok(n) => {
                        println!(
                            "PEER_BYTES at_ns={} offset={} bytes={:02x?} server_bytes_written=0",
                            journal.ns(Instant::now()),
                            bytes.len(),
                            &buf[..n]
                        );
                        if bytes.len() + n > HELLO_BYTES {
                            return Err("bounded ClientHello capacity".into());
                        }
                        bytes.extend_from_slice(&buf[..n]);
                        let mut input = io::Cursor::new(&buf[..n]);
                        while input.position() < n as u64 {
                            acceptor.read_tls(&mut input)?;
                        }
                        if let Some(accepted) = acceptor
                            .accept()
                            .map_err(|(error, _)| format!("real ClientHello parse: {error}"))?
                        {
                            let at = Instant::now();
                            let conn = trace_conn(stalled_addr, &command, cutoff)
                                .ok_or("pending connection trace identity unavailable")?;
                            let info = caller.conn_info(conn).ok_or("pending caller connection missing")?;
                            println!("CLIENT_HELLO at_ns={} bytes={} conn={conn:?} info={info:?} server_name={:?} cipher_suites={:?} server_bytes_written=0", journal.ns(at), bytes.len(), accepted.client_hello().server_name(), accepted.client_hello().cipher_suites());
                            if at >= cutoff
                                || info.peer != stalled_addr
                                || !info.outbound
                                || info.tls_version.is_some()
                                || info.fingerprint.is_some()
                                || info.certificate_verified
                                || !journal.caller.is_empty()
                            {
                                return Err("pending TLS proof did not finish well before earliest deadline".into());
                            }
                            println!("PRECONDITION verdict=PASS conn={conn:?} at_ns={} earliest_deadline_margin_ns={} public_events=0 genuine_client_hello=true", journal.ns(at), lower.duration_since(at).as_nanos());
                            break conn;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    Err(error) => return Err(error.into()),
                }
            }
            let poll = journal.call(&mut caller, false, "prepare_pending_tls", None)?;
            if poll.returned >= cutoff {
                return Err("preparation poll exceeded conservative cutoff".into());
            }
        };
        let idle_enter = Instant::now();
        if idle_enter >= cutoff {
            return Err("caller idle did not start before conservative preparation cutoff".into());
        }
        qualified = true;
        println!(
            "IDLE_ENTER at_ns={} until_latest_deadline_ns={} caller_polling=NONE product_grace=NONE",
            journal.ns(idle_enter),
            journal.ns(upper)
        );
        std::thread::sleep(upper.saturating_duration_since(Instant::now()));
        let idle_return = Instant::now();
        println!("IDLE_RETURN at_ns={} no_poll_between=true", journal.ns(idle_return));
        let first = journal.call(&mut caller, false, "first_after_expiry", None)?;
        if idle_enter >= lower
            || first.enter < upper
            || !timeout(&journal.caller, stalled_addr)
            || caller.conn_info(pending_conn).is_some()
        {
            return Err("FIRST_ELIGIBLE_POLL_FAIL: expected exactly one matching Timeout and removal of same pending connection".into());
        }
        println!("FIRST_ELIGIBLE_POLL verdict=PASS conn={pending_conn:?} enter_ns={} latest_deadline_ns={} matching_timeout_count=1 pending_connection_removed=true", journal.ns(first.enter), journal.ns(upper));
        let quiet = Instant::now();
        while quiet.elapsed() < QUIET {
            journal.call(&mut caller, false, "late_events", None)?;
            no_accept(owned)?;
            if !timeout(&journal.caller, stalled_addr) {
                return Err("late duplicate/wrong failed-request event".into());
            }
        }
        let stream = client.as_mut().ok_or("stalled accepted stream missing")?;
        let mut buf = [0; 4096];
        match stream.read(&mut buf) {
            Ok(0) => println!(
                "FAILED_CLIENT_EOF at_ns={} local={:?} peer={:?} server_bytes_written=0",
                journal.ns(Instant::now()),
                stream.local_addr()?,
                stream.peer_addr()?
            ),
            result => {
                return Err(format!("failed client not closed or extra bytes after ClientHello: {result:?}").into())
            }
        }
        journal.submit(
            &handle,
            "recovery_connect_same_handle",
            NetCommand::ConnectPinned {
                host: "127.0.0.1".into(),
                port: healthy_addr.port(),
                fingerprint,
            },
        )?;
        let start = Instant::now();
        let mut sent = false;
        let mut replied = false;
        let mut quiet = None;
        loop {
            journal.call(&mut caller, false, "recovery", None)?;
            journal.call(&mut peer, true, "recovery", None)?;
            no_accept(owned)?;
            if start.elapsed() >= LIMIT {
                return Err("same caller recovery deadline".into());
            }
            if let (Some(conn), Some(peer_conn)) = (connected(&journal.caller), connected(&journal.peer)) {
                if conn == pending_conn {
                    return Err("reused failed connection ID".into());
                }
                if !sent {
                    journal.submit(
                        &handle,
                        "PASS_same_handle",
                        NetCommand::SendFrame {
                            conn,
                            payload: b"PASS".to_vec(),
                            priority: Priority::Control,
                        },
                    )?;
                    sent = true;
                }
                if journal.peer.iter().any(|event| matches!(event, NetEvent::Frame { conn, payload } if *conn == peer_conn && payload == b"PASS")) && !replied {
                    journal.call(&mut peer, true, "BACK", Some(NetCommand::SendFrame { conn: peer_conn, payload: b"BACK".to_vec(), priority: Priority::Control }))?;
                    replied = true;
                }
                if journal.caller.iter().any(|event| matches!(event, NetEvent::Frame { conn: actual, payload } if *actual == conn && payload == b"BACK"))
                    && quiet.get_or_insert_with(Instant::now).elapsed() >= QUIET { break; }
            }
        }
        let conn = connected(&journal.caller).ok_or("healthy caller missing")?;
        let peer_conn = connected(&journal.peer).ok_or("healthy peer missing")?;
        if !matches!(journal.caller.as_slice(), [NetEvent::ConnectFailed { .. }, NetEvent::Connected { conn: actual, peer: addr, fingerprint: Some(fp), tls: true, tls_version: Some(_), .. }, NetEvent::Frame { conn: frame_conn, payload }]
            if *actual == conn && *addr == healthy_addr && *fp == fingerprint && *frame_conn == conn && payload == b"BACK")
            || !matches!(journal.peer.as_slice(), [NetEvent::Listening { addr }, NetEvent::Connected { conn: actual, peer: source, fingerprint: Some(fp), tls: true, tls_version: Some(_), .. }, NetEvent::Frame { conn: frame_conn, payload }]
                if *addr == healthy_addr && *actual == peer_conn && source.ip().is_loopback() && Some(*fp) == caller.identity_fingerprint() && *frame_conn == peer_conn && payload == b"PASS")
            || !timeout(&journal.caller[..1], stalled_addr)
            || !Arc::ptr_eq(&waker, &caller.waker())
        {
            return Err("recovery transcript/cardinality/identity mismatch".into());
        }
        println!("RECOVERY verdict=PASS failed_conn={pending_conn:?} caller_conn={conn:?} peer_conn={peer_conn:?} same_driver=true same_handle=true same_waker=true timeout_budget_unchanged_ns={} reset=NONE reconfigure=NONE caller_events=3 peer_events=3 exact_PASS_BACK=true", BUDGET.as_nanos());
        Ok(())
    })();
    // Always close the drivers and owned sockets, including first-failure paths.
    println!("WORKFLOW result={result:?} qualified={qualified}");
    let caller_conn = connected(&journal.caller);
    let peer_conn = connected(&journal.peer);
    let caller_before = journal.caller.len();
    let peer_before = journal.peer.len();
    let cleanup_caller = journal.call(&mut caller, false, "cleanup_shutdown", Some(NetCommand::Shutdown));
    let cleanup_peer = journal.call(&mut peer, true, "cleanup_shutdown", Some(NetCommand::Shutdown));
    let exact_shutdown = |events: &[NetEvent], conn: Option<ConnId>| match conn {
        Some(conn) => {
            matches!(events, [NetEvent::Disconnected { conn: actual, reason: openergy_session::DisconnectReason::Shutdown }] if *actual == conn)
        }
        None => events.is_empty(),
    };
    let mut cleanup_ok = cleanup_caller.is_ok()
        && cleanup_peer.is_ok()
        && exact_shutdown(&journal.caller[caller_before..], caller_conn)
        && exact_shutdown(&journal.peer[peer_before..], peer_conn);
    drop(caller);
    drop(peer);
    drop(handle);
    drop(waker);
    drop(client);
    drop(listener);
    for addr in owned_addrs {
        match bound_socket::bind(addr) {
            Ok(owner) => {
                println!("CLEANUP_BIND addr={addr} verdict=PASS");
                drop(owner);
            }
            Err(error) => {
                println!("CLEANUP_BIND addr={addr} verdict=FAIL error={error:?}");
                cleanup_ok = false;
            }
        }
    }
    let state = OBSERVER.0.lock().expect("observer lock");
    for trace in &state.0 {
        println!("TRACE at_ns={} {}", journal.ns(trace.at), trace.message);
    }
    println!("TRANSCRIPT caller={:?} peer={:?}", journal.caller, journal.peer);
    println!(
        "CLEANUP verdict={} trace_overflow={} elapsed_ns={} all_owned_drivers_handles_listeners_clients_dropped=true",
        if cleanup_ok { "PASS" } else { "FAIL" },
        state.1,
        journal.epoch.elapsed().as_nanos()
    );
    Ok(if !cleanup_ok || state.1 || journal.epoch.elapsed() >= RUNTIME {
        Verdict::Fail
    } else if result.is_ok() {
        Verdict::Pass
    } else if qualified {
        Verdict::Fail
    } else {
        Verdict::Unverified
    })
}

pub(super) fn run(root: &Path) -> Result<i32> {
    let verdict = match case(root) {
        Ok(verdict) => verdict,
        Err(error) => {
            println!("SETUP verdict=UNVERIFIED error={error:?}");
            Verdict::Unverified
        }
    };
    println!("RESULT scope=idle_tls_deadline_only verdict={} continuous_strict_deadline=NOT_MEASURED full_acceptance=NOT_REQUESTED original_UNVERIFIED_gates=UNCHANGED", verdict.label());
    Ok(match verdict {
        Verdict::Pass => 0,
        Verdict::Unverified => 2,
        Verdict::Fail => 1,
    })
}
