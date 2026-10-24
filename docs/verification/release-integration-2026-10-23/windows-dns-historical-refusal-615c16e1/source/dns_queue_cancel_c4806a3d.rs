//! Real NetDriver commands and owned loopback sockets on TCP 25100..=25119.
//! Short OS timing, fixture qualification, API deadlines, and observation are separate.
//! No slow-DNS fixture, resolver replacement, native backend, or discovery.
use openergy_net::{ConnectErrorKind, NetCommand, NetConfig, NetDriver, NetError, NetEvent, Priority, TransportMode};
use openergy_session::ConnId;
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEALTHY: u16 = 25116;
const REFUSED: u16 = 25117;
const INBOUND: u16 = 25118;
const LIMIT: Duration = Duration::from_secs(3);
const OBSERVE: Duration = Duration::from_secs(6);
const TRACE_TARGET: &str = "openergy_net::connect_observer";
const MAX_TRACE: usize = 256;
const MAX_CALLS: usize = 8192;
const MAX_EVENTS: usize = 64;
const MAX_BASE_EVENTS: usize = 4096;
const QUIET: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Verdict {
    Pass,
    Unverified,
    Fail,
}
impl Verdict {
    fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Unverified => "UNVERIFIED",
            Self::Fail => "FAIL",
        }
    }
}
struct Trace {
    at: Instant,
    message: String,
}
struct Observer(Mutex<(Vec<Trace>, bool)>);
static OBSERVER: Observer = Observer(Mutex::new((Vec::new(), false)));
impl log::Log for Observer {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target() == TRACE_TARGET && metadata.level() == log::Level::Trace
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let at = Instant::now();
            let mut state = self.0.lock().expect("observer lock");
            if state.0.len() < MAX_TRACE {
                state.0.push(Trace {
                    at,
                    message: record.args().to_string(),
                });
            } else {
                state.1 = true;
            }
        }
    }
    fn flush(&self) {}
}
fn field<'a>(trace: &'a Trace, name: &str) -> Option<&'a str> {
    let mut values = trace.message.split_whitespace().filter_map(|part| {
        let (key, value) = part.split_once('=')?;
        (key == name).then_some(value)
    });
    let value = values.next()?;
    (!value.is_empty() && values.next().is_none()).then_some(value)
}
struct Transition<'a> {
    at: Instant,
    phase: &'a str,
    conn: u64,
    addr: SocketAddr,
    deadline: &'a str,
    detail: Option<&'a str>,
}
fn transition(trace: &Trace) -> Option<Transition<'_>> {
    let phase = field(trace, "phase")?;
    let conn = field(trace, "conn")?.parse().ok().filter(|id| *id > 0)?;
    let addr = field(trace, "addr")?.parse().ok()?;
    // Deadline is the final Debug field at all four allowed product trace sites.
    // Require a single complete value; never compare two absent fields. Its
    // identity corroborates transitions; public command/poll brackets still
    // determine caller timing independently.
    if trace.message.matches("deadline=").count() != 1 {
        return None;
    }
    let deadline = trace.message.split_once("deadline=")?.1;
    let body = deadline.strip_prefix("Instant {")?.strip_suffix('}')?.trim();
    let mut depth = 0_i32;
    for byte in deadline.bytes() {
        match byte {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return None;
        }
    }
    if depth != 0
        || body.split(',').any(|part| {
            part.split_once(':')
                .is_none_or(|(key, value)| key.trim().is_empty() || !value.bytes().any(|byte| byte.is_ascii_digit()))
        })
    {
        return None;
    }
    let detail = match phase {
        "attempt" => Some(
            field(trace, "outbound")?
                .parse::<bool>()
                .ok()
                .map(|value| if value { "true" } else { "false" })?,
        ),
        "tcp_ready" => Some(
            field(trace, "tls")?
                .parse::<bool>()
                .ok()
                .map(|value| if value { "true" } else { "false" })?,
        ),
        "failed" => Some(field(trace, "kind")?),
        "fallback" => None,
        _ => return None,
    };
    Some(Transition {
        at: trace.at,
        phase,
        conn,
        addr,
        deadline,
        detail,
    })
}
struct Call {
    enter: Instant,
    returned: Instant,
}
struct Observation {
    calls: Vec<Call>, // execute_checked first; every later entry is a real poll.
    events: Vec<(usize, NetEvent)>,
    traces: Vec<Trace>,
    trace_overflow: bool,
    budget: Duration,
    peer_accepted: Option<Instant>,
    peer_bytes: usize,
}
impl Observation {
    fn lower(&self) -> Instant {
        self.calls[0].enter + self.budget
    }
    fn upper(&self) -> Instant {
        self.calls[0].returned + self.budget
    }
    fn exact(&self, host: &str, port: u16, kind: ConnectErrorKind, prefix: &str) -> bool {
        self.events.len() == 1
            && matches!(&self.events[0].1,
            NetEvent::ConnectFailed { host: actual_host, port: actual_port, error }
            if actual_host == host && *actual_port == port && error.starts_with(prefix)
                && ConnectErrorKind::from_message(error) == kind)
    }
    fn refusal(&self, host: &str, port: u16) -> Verdict {
        if !self.exact(host, port, ConnectErrorKind::Refused, "connect_refused:") {
            return Verdict::Fail;
        }
        let call = &self.calls[self.events[0].0];
        if call.returned < self.lower() {
            Verdict::Pass
        } else if call.enter >= self.upper() {
            Verdict::Fail
        } else {
            Verdict::Unverified
        } // Receipt interval overlaps the caller deadline.
    }
    fn timeout(&self, host: &str, port: u16) -> Verdict {
        if self.exact(host, port, ConnectErrorKind::Refused, "connect_refused:") {
            // Only a genuine in-budget refusal is legitimate. It still does
            // not exercise expiry; a definitely late refusal is a failure.
            return self.refusal(host, port).max(Verdict::Unverified);
        }
        if !self.exact(host, port, ConnectErrorKind::Timeout, "connect_timeout:") {
            return Verdict::Fail;
        }
        let index = self.events[0].0;
        let call = &self.calls[index];
        if index == 0 || call.returned < self.lower() {
            return Verdict::Fail;
        }
        // No event may be deferred past the first actively driven poll known to
        // enter after expiry. A poll spanning the timing bracket is uncertain.
        let first_eligible = self
            .calls
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, call)| call.enter >= self.upper())
            .map(|(i, _)| i);
        if first_eligible.is_some_and(|first| index > first) {
            return Verdict::Fail;
        }
        if call.enter >= self.upper() {
            Verdict::Pass
        } else {
            Verdict::Unverified
        }
    }
    fn report(&self, case: &str) {
        let epoch = self.calls[0].enter;
        println!(
            "CLOCK case={case} epoch={epoch:?} earliest_deadline={:?} latest_deadline={:?}",
            self.lower(),
            self.upper()
        );
        println!("TIMING case={case} clock=monotonic_ns lower_deadline_ns={} upper_deadline_ns={} observation_ns={} trace_overflow={} accepted_ns={:?} peer_bytes={}",
            self.lower().duration_since(epoch).as_nanos(), self.upper().duration_since(epoch).as_nanos(),
            OBSERVE.as_nanos(), self.trace_overflow,
            self.peer_accepted.map(|at| at.duration_since(epoch).as_nanos()), self.peer_bytes);
        for (index, call) in self.calls.iter().enumerate() {
            println!(
                "CALL case={case} index={index} op={} enter_ns={} return_ns={}",
                if index == 0 { "execute_checked" } else { "poll" },
                call.enter.duration_since(epoch).as_nanos(),
                call.returned.duration_since(epoch).as_nanos()
            );
        }
        for (index, event) in &self.events {
            println!("EVENT case={case} call={index} value={event:?}");
        }
        for trace in &self.traces {
            println!(
                "TRACE case={case} at_ns={} {}",
                trace.at.duration_since(epoch).as_nanos(),
                trace.message
            );
        }
    }
}
fn observe(
    driver: &mut NetDriver,
    host: &str,
    port: u16,
    budget: Duration,
    listener: Option<&TcpListener>,
) -> Result<Observation> {
    driver.set_connect_timeout(budget)?;
    *OBSERVER.0.lock().expect("observer lock") = (Vec::new(), false);
    let mut out = Vec::new();
    let enter = Instant::now();
    driver.execute_checked(connect(host, port), &mut out)?;
    let mut calls = vec![Call {
        enter,
        returned: Instant::now(),
    }];
    let mut events: Vec<_> = out.drain(..).map(|event| (0, event)).collect();
    let mut peer: Option<TcpStream> = None;
    let mut peer_accepted = None;
    let mut peer_bytes = 0;
    while enter.elapsed() < OBSERVE {
        if calls.len() >= MAX_CALLS || events.len() > MAX_EVENTS {
            return Err("finite observation capacity exceeded".into());
        }
        if let Some(listener) = listener {
            match listener.accept() {
                Ok((stream, addr)) if peer.is_none() && addr.ip().is_loopback() => {
                    stream.set_nonblocking(true)?;
                    peer = Some(stream);
                    peer_accepted = Some(Instant::now());
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                result => return Err(format!("unexpected owned peer accept: {result:?}").into()),
            }
        }
        if let Some(peer) = &mut peer {
            let mut bytes = [0; 4096];
            match peer.read(&mut bytes) {
                Ok(n) => peer_bytes += n,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
        let poll_enter = Instant::now();
        driver.poll(
            Some(Duration::from_millis(10).min(OBSERVE.saturating_sub(enter.elapsed()))),
            &mut out,
        )?;
        let returned = Instant::now();
        let index = calls.len();
        calls.push(Call {
            enter: poll_enter,
            returned,
        });
        events.extend(out.drain(..).map(|event| (index, event)));
    }
    let (traces, trace_overflow) = std::mem::take(&mut *OBSERVER.0.lock().expect("observer lock"));
    if events.len() > MAX_EVENTS {
        return Err("finite event capacity exceeded".into());
    }
    Ok(Observation {
        calls,
        events,
        traces,
        trace_overflow,
        budget,
        peer_accepted,
        peer_bytes,
    })
}
#[cfg(unix)]
mod bound_socket {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    pub(super) type Owner = OwnedFd;
    fn result(code: i32) -> io::Result<()> {
        if code == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    pub(super) fn bind(addr: SocketAddr) -> Result<Owner> {
        let raw = unsafe {
            libc::socket(
                if addr.is_ipv4() { libc::AF_INET } else { libc::AF_INET6 },
                libc::SOCK_STREAM,
                0,
            )
        };
        if raw < 0 {
            return Err(io::Error::last_os_error().into());
        }
        // Own the socket immediately, including all setup failure paths.
        let owner = unsafe { OwnedFd::from_raw_fd(raw) };
        let zero: i32 = 0;
        result(unsafe {
            libc::setsockopt(
                raw,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                (&zero as *const i32).cast(),
                std::mem::size_of::<i32>() as libc::socklen_t,
            )
        })?;
        if addr.is_ipv6() {
            let one: i32 = 1;
            result(unsafe {
                libc::setsockopt(
                    raw,
                    libc::IPPROTO_IPV6,
                    libc::IPV6_V6ONLY,
                    (&one as *const i32).cast(),
                    std::mem::size_of::<i32>() as libc::socklen_t,
                )
            })?;
            let mut observed: i32 = 0;
            let mut len = std::mem::size_of::<i32>() as libc::socklen_t;
            result(unsafe {
                libc::getsockopt(
                    raw,
                    libc::IPPROTO_IPV6,
                    libc::IPV6_V6ONLY,
                    (&mut observed as *mut i32).cast(),
                    &mut len,
                )
            })?;
            if observed != 1 {
                return Err("IPv6 refusal owner is not IPv6-only".into());
            }
        }
        match addr {
            SocketAddr::V4(addr) => {
                let mut value: libc::sockaddr_in = unsafe { std::mem::zeroed() };
                value.sin_family = libc::AF_INET as _;
                value.sin_port = addr.port().to_be();
                value.sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
                #[cfg(target_vendor = "apple")]
                {
                    value.sin_len = std::mem::size_of_val(&value) as u8;
                }
                result(unsafe {
                    libc::bind(
                        raw,
                        (&value as *const libc::sockaddr_in).cast(),
                        std::mem::size_of_val(&value) as libc::socklen_t,
                    )
                })?;
            }
            SocketAddr::V6(addr) => {
                let mut value: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
                value.sin6_family = libc::AF_INET6 as _;
                value.sin6_port = addr.port().to_be();
                value.sin6_addr.s6_addr = addr.ip().octets();
                value.sin6_scope_id = addr.scope_id();
                #[cfg(target_vendor = "apple")]
                {
                    value.sin6_len = std::mem::size_of_val(&value) as u8;
                }
                result(unsafe {
                    libc::bind(
                        raw,
                        (&value as *const libc::sockaddr_in6).cast(),
                        std::mem::size_of_val(&value) as libc::socklen_t,
                    )
                })?;
            }
        }
        // No listen(). Qualification must establish actual refusal; Darwin may time out.
        Ok(owner)
    }
    pub(super) fn listen(owner: Owner) -> Result<TcpListener> {
        result(unsafe { libc::listen(owner.as_raw_fd(), 1) })?;
        Ok(TcpListener::from(owner))
    }
}
#[cfg(windows)]
mod bound_socket {
    use super::*;
    use std::os::windows::io::{AsRawSocket, FromRawSocket, OwnedSocket, RawSocket};
    pub(super) type Owner = OwnedSocket;
    #[link(name = "ws2_32")]
    extern "system" {
        fn socket(family: i32, kind: i32, protocol: i32) -> RawSocket;
        fn setsockopt(socket: RawSocket, level: i32, name: i32, value: *const u8, len: i32) -> i32;
        fn getsockopt(socket: RawSocket, level: i32, name: i32, value: *mut u8, len: *mut i32) -> i32;
        #[link_name = "bind"]
        fn socket_bind(socket: RawSocket, address: *const u8, len: i32) -> i32;
        fn listen(socket: RawSocket, backlog: i32) -> i32;
        fn WSAGetLastError() -> i32;
    }
    #[repr(C)]
    struct AddressV4 {
        family: u16,
        port: u16,
        address: [u8; 4],
        zero: [u8; 8],
    }
    #[repr(C)]
    struct AddressV6 {
        family: u16,
        port: u16,
        flow: u32,
        address: [u8; 16],
        scope: u32,
    }
    fn result(code: i32) -> io::Result<()> {
        if code == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() }))
        }
    }
    fn option(raw: RawSocket, level: i32, name: i32) -> Result<()> {
        let one: i32 = 1;
        result(unsafe { setsockopt(raw, level, name, (&one as *const i32).cast(), 4) })?;
        let mut observed: i32 = 0;
        let mut len: i32 = 4;
        result(unsafe { getsockopt(raw, level, name, (&mut observed as *mut i32).cast(), &mut len) })?;
        if observed != 1 || len != 4 {
            return Err("owned socket option mismatch".into());
        }
        Ok(())
    }
    pub(super) fn bind(addr: SocketAddr) -> Result<Owner> {
        // The real std resolver initializes Winsock before fixture construction.
        let raw = unsafe { socket(if addr.is_ipv4() { 2 } else { 23 }, 1, 6) };
        if raw == RawSocket::MAX {
            return Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() }).into());
        }
        let owner = unsafe { OwnedSocket::from_raw_socket(raw) };
        option(raw, 0xffff, -5)?; // SO_EXCLUSIVEADDRUSE
        if addr.is_ipv6() {
            option(raw, 41, 27)?;
        } // IPPROTO_IPV6, IPV6_V6ONLY
        match addr {
            SocketAddr::V4(addr) => {
                let value = AddressV4 {
                    family: 2,
                    port: addr.port().to_be(),
                    address: addr.ip().octets(),
                    zero: [0; 8],
                };
                result(unsafe {
                    socket_bind(
                        raw,
                        (&value as *const AddressV4).cast(),
                        std::mem::size_of_val(&value) as i32,
                    )
                })?;
            }
            SocketAddr::V6(addr) => {
                let value = AddressV6 {
                    family: 23,
                    port: addr.port().to_be(),
                    flow: 0,
                    address: addr.ip().octets(),
                    scope: addr.scope_id(),
                };
                result(unsafe {
                    socket_bind(
                        raw,
                        (&value as *const AddressV6).cast(),
                        std::mem::size_of_val(&value) as i32,
                    )
                })?;
            }
        }
        Ok(owner)
    }
    pub(super) fn listen_owned(owner: Owner) -> Result<TcpListener> {
        result(unsafe { listen(owner.as_raw_socket(), 1) })?;
        Ok(TcpListener::from(owner))
    }
}
struct BoundRefusal {
    addr: SocketAddr,
    owner: bound_socket::Owner,
}
struct RefusalFixtures {
    sockets: Vec<BoundRefusal>,
    localhost: Vec<(u16, Vec<SocketAddr>)>,
}
impl RefusalFixtures {
    fn new() -> Self {
        let mut sockets = Vec::new();
        let mut localhost = Vec::new();
        for port in [REFUSED, 25119] {
            let resolved = match resolve_localhost(port) {
                Ok(addresses) => addresses,
                Err(error) => {
                    println!("FIXTURE_SETUP port={port} verdict=UNVERIFIED resolver_error={error:?}");
                    Vec::new()
                }
            };
            println!("RESOLVER host=localhost addresses={resolved:?} order_for_product=NOT_INFERRED");
            let mut addresses = vec![
                SocketAddr::from(([127, 0, 0, 1], port)),
                SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
            ];
            for addr in &resolved {
                if addr.ip().is_loopback() && !addresses.contains(addr) {
                    addresses.push(*addr);
                }
            }
            localhost.push((port, resolved));
            for addr in addresses {
                let owner = match bound_socket::bind(addr) {
                    Ok(owner) => owner,
                    Err(error) => {
                        println!(
                            "FIXTURE_SETUP addr={addr} verdict=UNVERIFIED exclusive_owner=UNAVAILABLE error={error:?}"
                        );
                        continue;
                    }
                };
                match TcpListener::bind(addr) {
                    Err(error)
                        if error.kind() == io::ErrorKind::AddrInUse
                            || error.kind() == io::ErrorKind::PermissionDenied =>
                    {
                        println!("EXCLUSIVE_BIND addr={addr} ownership=RETAINED listening=false ipv6_only={} duplicate_kind={:?} duplicate_errno={:?} result=PASS", addr.is_ipv6(), error.kind(), error.raw_os_error());
                        sockets.push(BoundRefusal { addr, owner });
                    }
                    result => {
                        println!("EXCLUSIVE_BIND addr={addr} verdict=UNVERIFIED ownership=REJECTED competing_bind={result:?} independent_flows=CONTINUE");
                        // Drop only these owned sockets. Never connect through
                        // a rejected reservation, including hostname fallback.
                    }
                }
            }
        }
        Self { sockets, localhost }
    }
    fn owns(&self, addr: SocketAddr) -> bool {
        self.sockets.iter().any(|socket| socket.addr == addr)
    }
    fn available(&self, host: &str, port: u16) -> bool {
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            return self.owns(SocketAddr::new(ip, port));
        }
        host == "localhost" && hostname_owned(port, &self.localhost, |addr| self.owns(addr))
    }
    fn tls_listener(&mut self) -> Option<TcpListener> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 25119));
        let Some(index) = self.sockets.iter().position(|socket| socket.addr == addr) else {
            println!("TLS_HANDOFF addr={addr} verdict=UNVERIFIED reason=NO_EXCLUSIVE_RESERVATION");
            return None;
        };
        let socket = self.sockets.remove(index);
        #[cfg(unix)]
        let result = bound_socket::listen(socket.owner);
        #[cfg(windows)]
        let result = bound_socket::listen_owned(socket.owner);
        match result.and_then(|listener| {
            listener.set_nonblocking(true)?;
            Ok(listener)
        }) {
            Ok(listener) => {
                println!(
                    "TLS_HANDOFF addr={addr} same_owned_socket=true release_or_rebind=NONE ipv6_owner_retained={}",
                    self.owns(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 25119)))
                );
                Some(listener)
            }
            Err(error) => {
                println!("TLS_HANDOFF addr={addr} verdict=UNVERIFIED error={error:?} client_command=NOT_EXECUTED");
                None
            }
        }
    }
}

fn os_control(addr: SocketAddr, budget: Duration, label: &str) -> bool {
    let started = Instant::now();
    let result = TcpStream::connect_timeout(&addr, budget);
    let elapsed = started.elapsed();
    let refused = matches!(&result, Err(error) if error.kind() == io::ErrorKind::ConnectionRefused);
    let within_budget = elapsed <= budget;
    println!(
        "OS_CONTROL label={label} addr={addr} budget_ms={} elapsed_ns={} result={result:?} actual_refusal={refused} within_budget={within_budget}",
        budget.as_millis(),
        elapsed.as_nanos()
    );
    refused && within_budget
}
fn qualify(fixtures: &RefusalFixtures, port: u16) -> Result<bool> {
    let mut qualified = fixtures.available("127.0.0.1", port)
        && fixtures.available("::1", port)
        && fixtures.available("localhost", port);
    for socket in fixtures.sockets.iter().filter(|socket| socket.addr.port() == port) {
        let _short = os_control(socket.addr, Duration::from_millis(500), "strict_short_observation");
        qualified &= os_control(socket.addr, OBSERVE, "bounded_fixture_reference");
    }
    println!("FIXTURE port={port} qualified={qualified} ownership=RETAINED_NON_LISTENING");
    Ok(qualified)
}
fn refusal_controls(driver: &mut NetDriver, fixtures: &RefusalFixtures) -> Result<Verdict> {
    let mut verdict = Verdict::Pass;
    for port in [REFUSED, 25119] {
        let qualified = qualify(fixtures, port)?;
        for (index, host) in ["127.0.0.1", "::1", "localhost", "localhost", "localhost"]
            .into_iter()
            .enumerate()
        {
            let case = format!("refusal_{port}_{index}");
            if !fixtures.available(host, port) {
                println!("REFUSAL case={case} host={host} port={port} executed=false verdict=UNVERIFIED reason=NO_EXCLUSIVE_OWNER_FOR_ALL_ATTEMPTS");
                verdict = verdict.max(Verdict::Unverified);
                continue;
            }
            let observed = observe(driver, host, port, Duration::from_secs(5), None)?;
            let exact_refusal = observed.refusal(host, port);
            // An unqualified fixture cannot prove refusal. Keep the strict
            // refusal predicate visible; a Timeout never satisfies it. Still
            // enforce caller timing and reject wrong/duplicate/stale events
            // independently, even when refusal coverage is unavailable.
            let caller_deadline = if observed.exact(host, port, ConnectErrorKind::Refused, "connect_refused:") {
                exact_refusal
            } else {
                observed.timeout(host, port)
            };
            let result = if qualified {
                exact_refusal
            } else {
                caller_deadline.max(Verdict::Unverified)
            };
            observed.report(&case);
            println!(
                "REFUSAL case={case} host={host} port={port} caller_ms=5000 qualified={qualified} exact_refusal_predicate={} independent_caller_deadline={} verdict={}",
                exact_refusal.label(), caller_deadline.label(), result.label()
            );
            verdict = verdict.max(result);
        }
    }
    driver.set_connect_timeout(LIMIT)?;
    Ok(verdict)
}
// The peer is a second real NetDriver. The affected caller is always `driver`;
// it is never recreated or reconfigured after the preceding failure phase.
fn deadline_recovery(driver: &mut NetDriver, root: &Path, case: &str, tls: bool) -> Result<()> {
    let mut peer_config = config(root);
    peer_config.tls_enabled = tls;
    peer_config.require_client_trust = false; // Private owned peer; caller pins its identity.
    let (mut peer, _) = NetDriver::new(peer_config)?;
    peer.set_transport_mode(TransportMode::Frames)?;
    peer.set_connect_timeout(LIMIT)?;
    driver.set_transport_mode(TransportMode::Frames)?;
    driver.set_connect_timeout(LIMIT)?;
    let addr = SocketAddr::from(([127, 0, 0, 1], INBOUND));
    let mut peer_events = Vec::new();
    peer.execute_checked(NetCommand::Listen { addr }, &mut peer_events)?;
    if !matches!(peer_events.as_slice(), [NetEvent::Listening { addr: actual }] if *actual == addr) {
        return Err(format!("recovery peer did not own listener: {peer_events:?}").into());
    }
    let fingerprint = peer.identity_fingerprint();
    let command = if tls {
        NetCommand::ConnectPinned {
            host: "127.0.0.1".into(),
            port: INBOUND,
            fingerprint: fingerprint.ok_or("owned TLS peer has no identity")?,
        }
    } else {
        connect("127.0.0.1", INBOUND)
    };
    let mut events = Vec::new();
    let started = Instant::now();
    driver.execute_checked(command, &mut events)?;
    let mut sent = false;
    let mut replied = false;
    let mut quiet = None;
    loop {
        driver.poll(Some(Duration::from_millis(1)), &mut events)?;
        peer.poll(Some(Duration::from_millis(1)), &mut peer_events)?;
        if events.len() > MAX_EVENTS || peer_events.len() > MAX_EVENTS || started.elapsed() > LIMIT {
            return Err(
                format!("recovery capacity/deadline case={case}: caller={events:?} peer={peer_events:?}").into(),
            );
        }
        if let (Some(conn), Some(peer_conn)) = (connected(&events), connected(&peer_events)) {
            if !sent {
                if !matches!(events.first(), Some(NetEvent::Connected { peer, tls: actual_tls, fingerprint: actual_fp, .. })
                    if *peer == addr && *actual_tls == tls && (!tls || *actual_fp == fingerprint))
                    || !matches!(peer_events.get(1), Some(NetEvent::Connected { peer, tls: actual_tls, .. })
                        if peer.ip().is_loopback() && *actual_tls == tls)
                {
                    return Err(format!("wrong recovery connection: {events:?} {peer_events:?}").into());
                }
                driver.execute_checked(
                    NetCommand::SendFrame {
                        conn,
                        payload: b"PASS".to_vec(),
                        priority: Priority::Control,
                    },
                    &mut events,
                )?;
                sent = true;
            }
            let received = peer_events.iter().any(|event| {
                matches!(event,
                NetEvent::Frame { conn, payload } if *conn == peer_conn && payload == b"PASS")
            });
            if received && !replied {
                peer.execute_checked(
                    NetCommand::SendFrame {
                        conn: peer_conn,
                        payload: b"BACK".to_vec(),
                        priority: Priority::Control,
                    },
                    &mut peer_events,
                )?;
                replied = true;
            }
            if received
                && events.iter().any(|event| {
                    matches!(event,
                NetEvent::Frame { conn: actual, payload } if *actual == conn && payload == b"BACK")
                })
                && quiet.get_or_insert_with(Instant::now).elapsed() >= Duration::from_millis(100)
            {
                break;
            }
        }
    }
    let conn = connected(&events).ok_or("recovery caller missing")?;
    let peer_conn = connected(&peer_events).ok_or("recovery peer missing")?;
    // Inspect the whole recovery transcript, including old-request events. No
    // prior failure, stale acceptance, duplicate Connected, or duplicate frame
    // is permitted. The quiet interval actively drives both real reactors.
    if !matches!(events.as_slice(), [NetEvent::Connected { .. }, NetEvent::Frame { conn: actual, payload }]
        if *actual == conn && payload == b"BACK")
        || !matches!(peer_events.as_slice(), [NetEvent::Listening { .. }, NetEvent::Connected { .. }, NetEvent::Frame { conn: actual, payload }]
            if *actual == peer_conn && payload == b"PASS")
    {
        return Err(format!("extra/stale recovery events case={case}: caller={events:?} peer={peer_events:?}").into());
    }
    driver.execute_checked(NetCommand::Shutdown, &mut events)?;
    peer.execute_checked(NetCommand::Shutdown, &mut peer_events)?;
    for _ in 0..20 {
        driver.poll(Some(Duration::from_millis(1)), &mut events)?;
        peer.poll(Some(Duration::from_millis(1)), &mut peer_events)?;
    }
    if !matches!(events.as_slice(), [NetEvent::Connected { .. }, NetEvent::Frame { .. },
        NetEvent::Disconnected { conn: actual, reason: openergy_session::DisconnectReason::Shutdown }] if *actual == conn)
        || !matches!(peer_events.as_slice(), [NetEvent::Listening { .. }, NetEvent::Connected { .. }, NetEvent::Frame { .. },
            NetEvent::Disconnected { conn: actual, reason: openergy_session::DisconnectReason::Shutdown }] if *actual == peer_conn)
    {
        return Err(format!(
            "extra/stale recovery shutdown events case={case}: caller={events:?} peer={peer_events:?}"
        )
        .into());
    }
    println!("RECOVERY case={case} same_driver=PASS reconfigure=NONE tls={tls} pin_matches_owned_peer={} exact_frames=PASS no_extra_or_stale_events=PASS elapsed_ns={} caller_events={events:?} peer_events={peer_events:?}", tls, started.elapsed().as_nanos());
    Ok(())
}

fn transitions(observed: &Observation) -> Option<Vec<Transition<'_>>> {
    if observed.trace_overflow {
        return None;
    }
    observed.traces.iter().map(transition).collect()
}
fn fallback_evidence(observed: &Observation, fixtures: &RefusalFixtures) -> bool {
    let Some(records) = transitions(observed) else {
        return false;
    };
    let [first, failed, second] = records.as_slice() else {
        return false;
    };
    first.phase == "attempt"
        && first.detail == Some("true")
        && failed.phase == "failed"
        && failed.detail == Some("ConnectionRefused")
        && second.phase == "fallback"
        && first.conn == failed.conn
        && first.conn == second.conn
        && first.addr == failed.addr
        && first.addr.is_ipv4() != second.addr.is_ipv4()
        && fixtures.owns(first.addr)
        && fixtures.owns(second.addr)
        && first.addr.port() == REFUSED
        && second.addr.port() == REFUSED
        && first.deadline == failed.deadline
        && first.deadline == second.deadline
        && first.at >= observed.calls[0].enter
        && first.at <= failed.at
        && failed.at <= second.at
        && second.at < observed.lower()
}
fn tls_evidence(observed: &Observation, expected: SocketAddr) -> bool {
    let Some(records) = transitions(observed) else {
        return false;
    };
    let [attempt, ready] = records.as_slice() else {
        return false;
    };
    attempt.phase == "attempt"
        && attempt.detail == Some("true")
        && ready.phase == "tcp_ready"
        && ready.detail == Some("true")
        && attempt.conn == ready.conn
        && attempt.addr == expected
        && ready.addr == expected
        && attempt.deadline == ready.deadline
        && attempt.at >= observed.calls[0].enter
        && attempt.at <= ready.at
        && ready.at < observed.lower()
        && observed.peer_accepted.is_some_and(|at| at < observed.lower())
        && observed.peer_bytes > 0
}
fn deadline_controls(root: &Path, driver: &mut NetDriver, fixtures: &mut RefusalFixtures) -> Result<Verdict> {
    let short_result = if fixtures.available("127.0.0.1", REFUSED) {
        let short = observe(driver, "127.0.0.1", REFUSED, Duration::from_millis(500), None)?;
        let result = short.timeout("127.0.0.1", REFUSED);
        short.report("short_tcp");
        println!(
            "DEADLINE case=short_tcp phase=tcp caller_ms=500 verdict={}",
            result.label()
        );
        result
    } else {
        println!("DEADLINE case=short_tcp executed=false verdict=UNVERIFIED reason=NO_EXCLUSIVE_OWNER");
        Verdict::Unverified
    };
    deadline_recovery(driver, &root.join("short-tcp-recovery"), "short_tcp", false)?;

    let fallback_result = if fixtures.available("localhost", REFUSED) {
        let fallback = observe(driver, "localhost", REFUSED, LIMIT, None)?;
        let preconditions = fallback_evidence(&fallback, fixtures);
        let result = fallback.timeout("localhost", REFUSED).max(if preconditions {
            Verdict::Pass
        } else {
            Verdict::Unverified
        });
        fallback.report("fallback_original_deadline");
        println!("DEADLINE case=fallback_original_deadline phase=tcp caller_ms=3000 actual_attempt_preconditions={preconditions} verdict={}", result.label());
        result
    } else {
        println!("DEADLINE case=fallback_original_deadline executed=false verdict=UNVERIFIED reason=NO_EXCLUSIVE_OWNER_FOR_ALL_ATTEMPTS");
        Verdict::Unverified
    };
    deadline_recovery(
        driver,
        &root.join("fallback-recovery"),
        "fallback_original_deadline",
        false,
    )?;

    // finish_handshake leaves established=false while TLS handshakes. Both
    // maintenance and wait_time use that same original connect deadline.
    // TCP is already accepted: this phase proves pending TLS only, never a
    // pending TCP handshake or slow DNS. Same-socket listen retains ownership.
    let tls_result = if let Some(listener) = fixtures.tls_listener() {
        let mut tls_config = config(&root.join("pending-tls"));
        tls_config.tls_enabled = true;
        let (mut tls_driver, _) = NetDriver::new(tls_config)?;
        let tls = observe(
            &mut tls_driver,
            "127.0.0.1",
            25119,
            Duration::from_millis(500),
            Some(&listener),
        )?;
        let preconditions = tls_evidence(&tls, SocketAddr::from(([127, 0, 0, 1], 25119)));
        let result = tls.timeout("127.0.0.1", 25119).max(if preconditions {
            Verdict::Pass
        } else {
            Verdict::Unverified
        });
        tls.report("short_tls_establishment");
        println!("DEADLINE case=short_tls_establishment phase=tls_after_tcp_accept caller_ms=500 pending_peer={preconditions} verdict={}", result.label());
        // Recovery uses exact public events and frames, not traces. It never
        // grants deadline/fallback coverage from missing trace identity.
        deadline_recovery(
            &mut tls_driver,
            &root.join("tls-recovery-peer"),
            "short_tls_establishment",
            true,
        )?;
        result
    } else {
        println!(
            "DEADLINE case=short_tls_establishment executed=false verdict=UNVERIFIED reason=NO_OWNED_TLS_LISTENER"
        );
        Verdict::Unverified
    };
    driver.set_connect_timeout(LIMIT)?;
    Ok(short_result.max(fallback_result).max(tls_result))
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
fn resolve_localhost(port: u16) -> Result<Vec<SocketAddr>> {
    let mut addresses = Vec::new();
    for addr in ("localhost", port).to_socket_addrs()? {
        if !addr.ip().is_loopback() || addr.port() != port {
            return Err(
                format!("rejected localhost endpoint before use: {addr}, expected loopback port {port}").into(),
            );
        }
        if !addresses.contains(&addr) {
            addresses.push(addr);
        }
        if addresses.len() > 16 {
            return Err("localhost fixture address capacity exceeded".into());
        }
    }
    if addresses.is_empty() {
        return Err("localhost has no loopback address".into());
    }
    Ok(addresses)
}
fn hostname_owned(port: u16, captured: &[(u16, Vec<SocketAddr>)], owns: impl Fn(SocketAddr) -> bool) -> bool {
    let Some((_, expected)) = captured.iter().find(|(actual, _)| *actual == port) else {
        return false;
    };
    let current = match resolve_localhost(port) {
        Ok(addresses) => addresses,
        Err(error) => {
            println!("HOSTNAME_GUARD port={port} verdict=UNVERIFIED error={error:?}");
            return false;
        }
    };
    let safe = !expected.is_empty()
        && current.len() == expected.len()
        && current.iter().all(|addr| expected.contains(addr) && owns(*addr));
    if !safe {
        println!("HOSTNAME_GUARD port={port} verdict=UNVERIFIED captured={expected:?} current={current:?} reason=INCOMPLETE_OWNERSHIP_OR_CHANGED_ADDRESSES");
    }
    safe
}
struct BaseListener {
    addr: SocketAddr,
    listener: TcpListener,
}
struct BaseFixtures {
    listeners: Vec<BaseListener>,
    localhost: Vec<(u16, Vec<SocketAddr>)>,
}
impl BaseFixtures {
    fn new() -> Self {
        let mut fixtures = Self {
            listeners: Vec::new(),
            localhost: Vec::new(),
        };
        // Complete ownership is established before any NetDriver hostname
        // command. No wildcard address or resolver replacement is used.
        for port in 25100..=HEALTHY {
            let captured = resolve_localhost(port).unwrap_or_else(|error| {
                println!("BASE_FIXTURE port={port} hostname=UNVERIFIED error={error:?}");
                Vec::new()
            });
            println!("BASE_RESOLVER port={port} captured={captured:?}");
            let mut addresses = vec![
                SocketAddr::from(([127, 0, 0, 1], port)),
                SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port)),
            ];
            for addr in &captured {
                if !addresses.contains(addr) {
                    addresses.push(*addr);
                }
            }
            fixtures.localhost.push((port, captured));
            for addr in addresses {
                #[cfg(unix)]
                let setup = bound_socket::bind(addr).and_then(bound_socket::listen);
                #[cfg(windows)]
                let setup = bound_socket::bind(addr).and_then(bound_socket::listen_owned);
                let setup = setup.and_then(|listener| {
                    listener.set_nonblocking(true)?;
                    Ok(listener)
                });
                match setup {
                    Ok(listener) => match TcpListener::bind(addr) {
                        Err(error)
                            if matches!(error.kind(), io::ErrorKind::AddrInUse | io::ErrorKind::PermissionDenied) =>
                        {
                            println!("BASE_OWNER addr={addr} listening=true retained=true ipv6_only={} duplicate_rejected=true", addr.is_ipv6());
                            fixtures.listeners.push(BaseListener { addr, listener });
                        }
                        result => println!(
                            "BASE_OWNER addr={addr} verdict=UNVERIFIED exclusivity=REJECTED competing_bind={result:?}"
                        ),
                    },
                    Err(error) => {
                        println!("BASE_OWNER addr={addr} verdict=UNVERIFIED ownership=UNAVAILABLE error={error:?}")
                    }
                }
            }
        }
        fixtures
    }
    fn owns(&self, addr: SocketAddr) -> bool {
        self.listeners.iter().any(|entry| entry.addr == addr)
    }
    fn available(&self, host: &str, port: u16) -> bool {
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            return self.owns(SocketAddr::new(ip, port));
        }
        host == "localhost" && hostname_owned(port, &self.localhost, |addr| self.owns(addr))
    }
    fn no_accept(&self, include_healthy: bool) -> Result<()> {
        for entry in &self.listeners {
            if include_healthy || entry.addr.port() != HEALTHY {
                match entry.listener.accept() {
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    result => {
                        return Err(format!("unexpected/stale TCP accept at owned {}: {result:?}", entry.addr).into())
                    }
                }
            }
        }
        Ok(())
    }
    fn accept(&self, driver: &mut NetDriver, events: &mut Vec<NetEvent>, expected: SocketAddr) -> Result<TcpStream> {
        if !self.owns(expected) {
            return Err("accept requested on an unowned endpoint".into());
        }
        let baseline = events.len();
        let started = Instant::now();
        loop {
            let mut accepted = None;
            for entry in &self.listeners {
                match entry.listener.accept() {
                    Ok((peer, source)) if entry.addr == expected && accepted.is_none() && source.ip().is_loopback() => {
                        peer.set_nonblocking(true)?;
                        println!("BASE_ACCEPT listener={} source={source}", entry.addr);
                        accepted = Some(peer);
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    result => return Err(format!("unexpected/stale owned accept at {}: {result:?}", entry.addr).into()),
                }
            }
            if let Some(peer) = accepted {
                self.no_accept(true)?;
                return Ok(peer);
            }
            if started.elapsed() >= LIMIT {
                return Err(format!("owned accept deadline; transcript={events:?}").into());
            }
            poll_base(driver, events, Duration::from_micros(100))?;
            exact_len(events, baseline, "waiting for owned accept")?;
        }
    }
}
fn poll_base(driver: &mut NetDriver, events: &mut Vec<NetEvent>, budget: Duration) -> Result<()> {
    let start = events.len();
    let result = driver.poll(Some(budget), events);
    record_base_events(events, start);
    if let Err(error) = result {
        return Err(format!("base poll error: {error}; transcript={events:?}").into());
    }
    if events.len() > MAX_BASE_EVENTS {
        return Err(format!("base transcript capacity exceeded: {events:?}").into());
    }
    Ok(())
}
fn record_base_events(events: &[NetEvent], start: usize) {
    for (index, event) in events.iter().enumerate().skip(start) {
        println!("BASE_EVENT index={index} value={event:?}");
    }
}
fn execute_base(driver: &mut NetDriver, command: NetCommand, events: &mut Vec<NetEvent>) -> Result<()> {
    let start = events.len();
    let result = driver.execute_checked(command, events);
    record_base_events(events, start);
    if let Err(error) = result {
        return Err(format!("base command error: {error}; transcript={events:?}").into());
    }
    if events.len() > MAX_BASE_EVENTS {
        return Err(format!("base transcript capacity exceeded: {events:?}").into());
    }
    Ok(())
}
fn quiet_public(
    driver: &mut NetDriver,
    events: &mut Vec<NetEvent>,
    fixtures: &BaseFixtures,
    extra: Option<&TcpListener>,
) -> Result<()> {
    let expected = events.len();
    let start = Instant::now();
    while start.elapsed() < QUIET {
        poll_base(driver, events, Duration::from_millis(1))?;
        exact_len(events, expected, "independent quiet polling")?;
        fixtures.no_accept(true)?;
        if let Some(listener) = extra {
            match listener.accept() {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                result => {
                    return Err(format!("unexpected connection on owned lifecycle proof listener: {result:?}").into())
                }
            }
        }
    }
    Ok(())
}
fn exact_len(events: &[NetEvent], expected: usize, phase: &str) -> Result<()> {
    if events.len() != expected {
        return Err(format!(
            "{phase}: unexpected public events; expected total {expected}, complete transcript={events:?}"
        )
        .into());
    }
    Ok(())
}
fn wait_append(driver: &mut NetDriver, events: &mut Vec<NetEvent>, start: usize) -> Result<()> {
    let began = Instant::now();
    while events.len() == start && began.elapsed() < LIMIT {
        poll_base(driver, events, Duration::from_millis(1))?;
    }
    if events.len() == start {
        return Err(format!("missing public event: {events:?}").into());
    }
    Ok(())
}
fn connected(out: &[NetEvent]) -> Option<ConnId> {
    out.iter().find_map(|event| {
        if let NetEvent::Connected { conn, .. } = event {
            Some(*conn)
        } else {
            None
        }
    })
}
fn single_connected(
    driver: &NetDriver,
    events: &[NetEvent],
    start: usize,
    expected: impl Fn(SocketAddr) -> bool,
    outbound: bool,
) -> Result<(ConnId, SocketAddr)> {
    let [NetEvent::Connected {
        conn,
        peer,
        fingerprint: None,
        tls: false,
        tls_version: None,
    }] = &events[start..]
    else {
        return Err(format!("expected exactly one plaintext Connected: {events:?}").into());
    };
    if !expected(*peer)
        || events[..start]
            .iter()
            .any(|event| matches!(event, NetEvent::Connected { conn: old, .. } if old == conn))
    {
        return Err(format!("wrong/stale Connected identity: {events:?}").into());
    }
    let info = driver.conn_info(*conn).ok_or("Connected has no live connection info")?;
    if info.peer != *peer || info.outbound != outbound {
        return Err(format!("wrong connection metadata: {info:?}").into());
    }
    Ok((*conn, *peer))
}
fn read_peer(peer: &mut TcpStream, bytes: &mut Vec<u8>, closed: bool) -> Result<bool> {
    let mut buffer = [0; 64];
    loop {
        match peer.read(&mut buffer) {
            Ok(0) if closed => return Ok(true),
            Ok(0) => return Err("owned peer closed while frame exchange was active".into()),
            Ok(n) => {
                bytes.extend_from_slice(&buffer[..n]);
                if bytes.len() > 8 {
                    return Err(format!("extra outbound frame bytes: {bytes:?}").into());
                }
                if closed {
                    return Err(format!("outbound bytes after expected shutdown: {bytes:?}").into());
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) => return Err(error.into()),
        }
    }
}
fn exchange(
    driver: &mut NetDriver,
    events: &mut Vec<NetEvent>,
    conn: ConnId,
    peer: &mut TcpStream,
    fixtures: &BaseFixtures,
    case: &str,
) -> Result<()> {
    let start = events.len();
    execute_base(
        driver,
        NetCommand::SendFrame {
            conn,
            payload: b"PASS".to_vec(),
            priority: Priority::Control,
        },
        events,
    )?;
    exact_len(events, start, "SendFrame admission")?;
    let began = Instant::now();
    let mut received = Vec::new();
    let mut replied = false;
    let mut quiet = None;
    loop {
        poll_base(driver, events, Duration::from_millis(1))?;
        read_peer(peer, &mut received, false)?;
        if !b"\0\0\0\x04PASS".starts_with(&received) {
            return Err(format!("wrong outbound frame bytes: {received:?}").into());
        }
        fixtures.no_accept(true)?;
        if received.len() == 8 && !replied {
            peer.write_all(b"\0\0\0\x04BACK")?;
            replied = true;
        }
        match &events[start..] {
            [] => {}
            [NetEvent::Frame { conn: actual, payload }] if *actual == conn && payload == b"BACK" && replied => {
                if quiet.get_or_insert_with(Instant::now).elapsed() >= QUIET {
                    break;
                }
            }
            _ => return Err(format!("extra/wrong events during frame exchange case={case}: {events:?}").into()),
        }
        if began.elapsed() >= LIMIT {
            return Err(format!("frame exchange deadline case={case}: bytes={received:?} events={events:?}").into());
        }
    }
    if received != b"\0\0\0\x04PASS" {
        return Err(format!("incomplete outbound frame: {received:?}").into());
    }
    println!("BASE_FRAME case={case} conn={} outbound={received:?} inbound=BACK exact_event_count=1 quiet_ms=100 extra_bytes=NONE", conn.0);
    Ok(())
}
fn finish_peer(
    driver: &mut NetDriver,
    events: &mut Vec<NetEvent>,
    start: usize,
    conn: ConnId,
    peer: &mut TcpStream,
    fixtures: &BaseFixtures,
) -> Result<()> {
    if !matches!(&events[start..], [NetEvent::Disconnected { conn: actual, reason: openergy_session::DisconnectReason::Shutdown }] if *actual == conn)
    {
        return Err(format!("wrong/duplicate lifecycle disconnection: {events:?}").into());
    }
    if driver.conn_info(conn).is_some() {
        return Err("disconnected connection is still live".into());
    }
    let expected = events.len();
    let began = Instant::now();
    let mut bytes = Vec::new();
    let mut saw_eof = false;
    let mut quiet = None;
    loop {
        poll_base(driver, events, Duration::from_millis(1))?;
        exact_len(events, expected, "post-disconnect quiet polling")?;
        saw_eof |= read_peer(peer, &mut bytes, true)?;
        fixtures.no_accept(true)?;
        if saw_eof && quiet.get_or_insert_with(Instant::now).elapsed() >= QUIET {
            break;
        }
        if began.elapsed() >= LIMIT {
            return Err("owned peer did not close within lifecycle deadline".into());
        }
    }
    Ok(())
}
fn base_client(
    driver: &mut NetDriver,
    handle: Option<&openergy_net::NetHandle>,
    events: &mut Vec<NetEvent>,
    fixtures: &BaseFixtures,
    host: &str,
    case: &str,
) -> Result<Verdict> {
    if !fixtures.available(host, HEALTHY) {
        println!(
            "BASE_CASE case={case} host={host} executed=false verdict=UNVERIFIED reason=INCOMPLETE_ENDPOINT_OWNERSHIP"
        );
        quiet_public(driver, events, fixtures, None)?;
        return Ok(Verdict::Unverified);
    }
    let start = events.len();
    if let Some(handle) = handle {
        handle.send(connect(host, HEALTHY))?;
    } else {
        execute_base(driver, connect(host, HEALTHY), events)?;
    }
    wait_append(driver, events, start)?;
    let numeric = host.parse::<std::net::IpAddr>().ok();
    let (conn, addr) = single_connected(
        driver,
        events,
        start,
        |addr| fixtures.owns(addr) && addr.port() == HEALTHY && numeric.is_none_or(|ip| ip == addr.ip()),
        true,
    )?;
    let mut peer = fixtures.accept(driver, events, addr)?;
    exchange(driver, events, conn, &mut peer, fixtures, case)?;
    let start = events.len();
    if let Some(handle) = handle {
        handle.send(NetCommand::Shutdown)?;
        wait_append(driver, events, start)?;
    } else {
        execute_base(driver, NetCommand::Shutdown, events)?;
    }
    finish_peer(driver, events, start, conn, &mut peer, fixtures)?;
    println!("BASE_CASE case={case} host={host} peer={addr} verdict=PASS");
    Ok(Verdict::Pass)
}
fn queue_round(
    driver: &mut NetDriver,
    events: &mut Vec<NetEvent>,
    fixtures: &BaseFixtures,
    round: usize,
    expire: bool,
) -> Result<Verdict> {
    let mode = if expire {
        "deadline"
    } else if round.is_multiple_of(2) {
        "shutdown"
    } else {
        "disconnect_all"
    };
    if !(25100..=HEALTHY).all(|port| fixtures.available("localhost", port)) {
        println!("QUEUE mode={mode} round={round} executed=false required_jobs=16 verdict=UNVERIFIED reason=INCOMPLETE_HOSTNAME_OWNERSHIP");
        quiet_public(driver, events, fixtures, None)?;
        return Ok(Verdict::Unverified);
    }
    fixtures.no_accept(true)?;
    let baseline = events.len();
    driver.set_connect_timeout(if expire { Duration::from_nanos(1) } else { LIMIT })?;
    let start = Instant::now();
    for port in 25100..=25115 {
        execute_base(driver, connect("localhost", port), events)?;
    }
    exact_len(events, baseline, "16-job admission")?;
    let admitted_us = start.elapsed().as_micros();
    execute_base(driver, connect("localhost", HEALTHY), events)?;
    if !matches!(&events[baseline..], [NetEvent::ConnectFailed { host, port: HEALTHY, error }] if host == "localhost" && error == "16-connection limit")
    {
        return Err(format!("17th pending job was not exactly rejected: {events:?}").into());
    }
    let obsolete_start = events.len();
    if expire {
        poll_base(driver, events, Duration::ZERO)?;
    } else {
        execute_base(
            driver,
            if round.is_multiple_of(2) {
                NetCommand::Shutdown
            } else {
                NetCommand::DisconnectAll
            },
            events,
        )?;
    }
    let obsolete_end = events.len();
    if obsolete_end - obsolete_start != 16 {
        return Err(format!("wrong obsolete event count: {events:?}").into());
    }
    for port in 25100..=25115 {
        let matching: Vec<_> = events[obsolete_start..obsolete_end]
            .iter()
            .filter_map(|event| match event {
                NetEvent::ConnectFailed {
                    host,
                    port: actual,
                    error,
                } if host == "localhost" && *actual == port => Some(error),
                _ => None,
            })
            .collect();
        if matching.len() != 1
            || if expire {
                !matching[0].starts_with("connect_timeout:")
                    || ConnectErrorKind::from_message(matching[0]) != ConnectErrorKind::Timeout
            } else {
                matching[0] != "shutdown"
            }
        {
            return Err(format!("wrong obsolete failure identity/cardinality: {events:?}").into());
        }
    }
    fixtures.no_accept(true)?;
    driver.set_connect_timeout(LIMIT)?;
    let recovery = Instant::now();
    let mut rejects = 0;
    loop {
        if !fixtures.available("localhost", HEALTHY) {
            println!("QUEUE mode={mode} round={round} admitted_jobs=16 obsolete_failures=16 verdict=UNVERIFIED reason=HEALTHY_ADDRESS_SET_CHANGED_BEFORE_RECOVERY");
            quiet_public(driver, events, fixtures, None)?;
            return Ok(Verdict::Unverified);
        }
        let before = events.len();
        execute_base(driver, connect("localhost", HEALTHY), events)?;
        if events.len() == before {
            break;
        }
        if !matches!(&events[before..], [NetEvent::ConnectFailed { host, port: HEALTHY, error }] if host == "localhost" && error == "DNS request limit")
        {
            return Err(format!("unexpected recovery admission events: {events:?}").into());
        }
        rejects += 1;
        if recovery.elapsed() >= LIMIT || rejects >= 128 {
            return Err("DNS budget did not recover".into());
        }
        let before = events.len();
        poll_base(driver, events, Duration::from_micros(100))?;
        exact_len(events, before, "obsolete completion accounting")?;
        fixtures.no_accept(true)?;
    }
    let admission_us = recovery.elapsed().as_micros();
    if !expire && rejects == 0 {
        return Err("cancellation released outstanding slots before completion accounting".into());
    }
    let connected_start = events.len();
    wait_append(driver, events, connected_start)?;
    let (conn, addr) = single_connected(
        driver,
        events,
        connected_start,
        |addr| fixtures.owns(addr) && addr.port() == HEALTHY,
        true,
    )?;
    let connected_us = recovery.elapsed().as_micros();
    let mut peer = fixtures.accept(driver, events, addr)?;
    exchange(
        driver,
        events,
        conn,
        &mut peer,
        fixtures,
        &format!("queue_{mode}_{round}"),
    )?;
    let roundtrip_us = recovery.elapsed().as_micros();
    let close_start = events.len();
    execute_base(driver, NetCommand::Shutdown, events)?;
    finish_peer(driver, events, close_start, conn, &mut peer, fixtures)?;
    fixtures.no_accept(true)?;
    println!("QUEUE mode={mode} round={round} jobs=16 admission_us={admitted_us} overflow_rejected=PASS dns_budget_rejections={rejects} recovery_admission_us={admission_us} recovery_connected_us={connected_us} recovery_roundtrip_us={roundtrip_us} exactly_one_failure=PASS stale_tcp_all_families=NONE frame_roundtrip=PASS peer={addr} verdict=PASS transcript_events={}", events.len()-baseline);
    Ok(Verdict::Pass)
}
struct ContractResult {
    base: Verdict,
    refusal: Verdict,
}
fn contracts(root: &Path, fixtures: &BaseFixtures, refused_fixtures: &RefusalFixtures) -> Result<ContractResult> {
    let (mut driver, handle) = NetDriver::new(config(root))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    let waker = driver.waker();
    let mut events = Vec::new();
    execute_base(&mut driver, connect("invalid\0hostname", REFUSED), &mut events)?;
    wait_append(&mut driver, &mut events, 0)?;
    if !matches!(events.as_slice(), [NetEvent::ConnectFailed { host, port: REFUSED, error }] if host == "invalid\0hostname" && error.starts_with("dns_failed:") && ConnectErrorKind::from_message(error) == ConnectErrorKind::Dns)
    {
        return Err(format!("wrong DNS failure identity/cardinality: {events:?}").into());
    }
    println!("FAILURE case=invalid_hostname events={events:?} classified_dns=PASS");
    let refusal = refusal_controls(&mut driver, refused_fixtures)?;
    let mut base = Verdict::Pass;
    let inbound = SocketAddr::from(([127, 0, 0, 1], INBOUND));
    let start = events.len();
    handle.send(NetCommand::Listen { addr: inbound })?;
    wait_append(&mut driver, &mut events, start)?;
    if matches!(&events[start..], [NetEvent::Listening { addr }] if *addr == inbound) {
        for (case, command) in [
            ("inbound_disconnect_all", NetCommand::DisconnectAll),
            ("inbound_shutdown", NetCommand::Shutdown),
        ] {
            // The same driver still owns its listener until the explicit
            // Shutdown below. Connect only after its exact Listening result.
            let mut peer = TcpStream::connect_timeout(&inbound, LIMIT)?;
            let expected = peer.local_addr()?;
            peer.set_nonblocking(true)?;
            let start = events.len();
            wait_append(&mut driver, &mut events, start)?;
            let (conn, _) = single_connected(&driver, &events, start, |addr| addr == expected, false)?;
            exchange(&mut driver, &mut events, conn, &mut peer, fixtures, case)?;
            if case == "inbound_shutdown" {
                println!("LIFECYCLE disconnect_all_keeps_listener=PASS proof=SECOND_REAL_ACCEPT_SAME_DRIVER");
            }
            let start = events.len();
            handle.send(command)?;
            wait_append(&mut driver, &mut events, start)?;
            finish_peer(&mut driver, &mut events, start, conn, &mut peer, fixtures)?;
        }
        // A real replacement bind/listen proves the original listener closed,
        // without contacting a released (and therefore unowned) TCP endpoint.
        match TcpListener::bind(inbound) {
            Ok(proof) => {
                proof.set_nonblocking(true)?;
                quiet_public(&mut driver, &mut events, fixtures, Some(&proof))?;
                println!("LIFECYCLE shutdown_closes_listener=PASS proof=OWNED_REPLACEMENT_BIND addr={inbound}");
                drop(proof);
            }
            Err(error) => {
                println!("LIFECYCLE shutdown_closes_listener=UNVERIFIED replacement_bind_error={error:?}");
                base = Verdict::Unverified;
            }
        }
    } else if matches!(&events[start..], [NetEvent::ListenFailed { addr, .. }] if *addr == inbound) {
        println!("LIFECYCLE inbound_cases=UNVERIFIED reason=OWNED_DRIVER_LISTENER_UNAVAILABLE");
        base = Verdict::Unverified;
    } else {
        return Err(format!("wrong/duplicate listener result: {events:?}").into());
    }
    driver.reconfigure(config(&root.join("reconfigured")))?;
    let mut invalid = config(&root.join("invalid"));
    invalid.max_frame_len = 0;
    if driver.reconfigure(invalid).is_ok() || !Arc::ptr_eq(&waker, &driver.waker()) {
        return Err("reconfigure validation or waker reuse failed".into());
    }
    let hostname = base_client(
        &mut driver,
        Some(&handle),
        &mut events,
        fixtures,
        "localhost",
        "original_handle_after_reconfigure",
    )?;
    base = base.max(hostname);
    let mut handle_network = hostname == Verdict::Pass;
    if hostname == Verdict::Unverified {
        let host = if fixtures.available("127.0.0.1", HEALTHY) {
            "127.0.0.1"
        } else {
            "::1"
        };
        let numeric = base_client(
            &mut driver,
            Some(&handle),
            &mut events,
            fixtures,
            host,
            "independent_numeric_original_handle",
        )?;
        handle_network |= numeric == Verdict::Pass;
        base = base.max(numeric);
    }
    for (host, case) in [
        ("127.0.0.1", "numeric_v4"),
        ("::1", "numeric_v6"),
        ("localhost", "healthy_localhost"),
    ] {
        base = base.max(base_client(&mut driver, None, &mut events, fixtures, host, case)?);
    }
    quiet_public(&mut driver, &mut events, fixtures, None)?;
    println!("BASE_TRANSCRIPT driver=lifecycle events={events:?}");
    drop(driver);
    if handle.send(connect("127.0.0.1", HEALTHY)) != Err(NetError::Shutdown) {
        return Err("old handle admitted work after driver drop".into());
    }
    println!("CONTRACTS same_driver={} same_handle={} same_waker=PASS reset_listener_details=SEE_LIFECYCLE_RESULTS reconfigure=PASS invalid_reconfigure_recovery=PASS driver_drop=PASS base_coverage={}", if events.iter().any(|event| matches!(event, NetEvent::Frame { .. })) { "PASS" } else { "UNVERIFIED" }, if handle_network { "PASS" } else { "UNVERIFIED" }, base.label());
    Ok(ContractResult { base, refusal })
}

fn run() -> Result<i32> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().ok_or("dns_queue_cancel_c4806a3d NEW_ISOLATED_DIR")?);
    let mut refusal_only = false;
    let mut attempts = false;
    for arg in args {
        match arg.as_str() {
            "--refusal-only" if !refusal_only => refusal_only = true,
            "--observe-attempts" if !attempts => attempts = true,
            _ => return Err("unknown or repeated probe option".into()),
        }
    }
    if !root.is_absolute() || root.exists() {
        return Err("requires a new absolute isolated directory".into());
    }
    std::fs::create_dir(&root)?;
    if attempts {
        log::set_logger(&OBSERVER).map_err(|error| format!("observer initialization: {error}"))?;
        log::set_max_level(log::LevelFilter::Trace);
    }
    println!("OBSERVER enabled={attempts} finite_traces={MAX_TRACE} finite_calls={MAX_CALLS} finite_events={MAX_EVENTS} enabled_cost=monotonic_clock_mutex_format_allocation disabled_product_format_allocation=NONE");
    let mut fixtures = RefusalFixtures::new();
    if refusal_only {
        let (mut driver, _) = NetDriver::new(config(&root))?;
        driver.set_transport_mode(TransportMode::Frames)?;
        let refused = refusal_controls(&mut driver, &fixtures)?;
        println!(
            "RESULT scope=refusal_only refusal_matrix={} full_acceptance=NOT_REQUESTED",
            refused.label()
        );
        return Ok(match refused {
            Verdict::Pass => 0,
            Verdict::Unverified => 2,
            Verdict::Fail => 1,
        });
    }
    let base = BaseFixtures::new();
    let (mut driver, _) = NetDriver::new(config(&root.join("queue")))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    println!("SCOPE resolver=real_std_to_socket_addrs host=localhost ports=25100..25119 discovery=OFF tls=only_owned_pending_establishment_case native=NONE slow_lookup_control=UNAVAILABLE");
    println!("BASE_TRANSCRIPT_START driver=queue");
    let mut queue_events = Vec::new();
    let mut queues = Verdict::Pass;
    for expire in [false, true] {
        for round in 0..4 {
            queues = queues.max(queue_round(&mut driver, &mut queue_events, &base, round, expire)?);
        }
    }
    if queues == Verdict::Unverified {
        for host in ["127.0.0.1", "::1"] {
            queues = queues.max(base_client(
                &mut driver,
                None,
                &mut queue_events,
                &base,
                host,
                "independent_numeric_after_unavailable_queue",
            )?);
        }
    }
    quiet_public(&mut driver, &mut queue_events, &base, None)?;
    println!("BASE_TRANSCRIPT driver=queue events={queue_events:?}");
    drop(driver);
    println!("BASE_TRANSCRIPT_START driver=lifecycle");
    let contracts = contracts(&root.join("contracts"), &base, &fixtures)?;
    base.no_accept(true)?;
    let deadlines = deadline_controls(
        &root,
        &mut NetDriver::new(config(&root.join("deadlines")))?.0,
        &mut fixtures,
    )?;
    base.no_accept(true)?;
    let acceptance = queues.max(contracts.base).max(contracts.refusal).max(deadlines);
    println!("RESULT queue_coverage={} lifecycle_coverage={} refusal_matrix={} requested_deadlines={} full_acceptance={} slow_dns_head_of_line_regression=NOT_MEASURED cache_expiry_runtime=UNVERIFIED", queues.label(), contracts.base.label(), contracts.refusal.label(), deadlines.label(), acceptance.label());
    Ok(match acceptance {
        Verdict::Pass => 0,
        Verdict::Unverified => 2,
        Verdict::Fail => 1,
    })
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("dns_queue_cancel_c4806a3d: {error}");
            std::process::exit(1);
        }
    }
}
