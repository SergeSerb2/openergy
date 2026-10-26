use crate::{
    discovery::{Discovery, DiscoveryScope},
    identity::failed,
    tls::TlsFactory,
    Fingerprint, Identity, NetCommand, NetError, NetEvent, NetHandle, Priority, TrustStore,
};
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token, Waker,
};
use openergy_protocol::{Decoder, Message, MAX_FRAME_LEN};
use openergy_session::{ConnId, DisconnectReason};
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io::{self, Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, ToSocketAddrs},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub const BULK_SLICE: usize = 32 * 1024;
const MAX_CONNECTIONS: usize = 16;
const HIGH_WATER: usize = 4 * 1024 * 1024;
const MAILBOX_BYTES: usize = 8 * 1024 * 1024;
const BULK_WINDOW: usize = 256 * 1024;
const WAKE: Token = Token(0);
const LISTENER: Token = Token(1);
const TRUST_POLL: Duration = Duration::from_millis(500);
const CLOSE_TIMEOUT: Duration = Duration::from_millis(250);
// A byte budget alone admits tens of thousands of tiny messages in one turn.
// Yield after this many events, finishing the current <=16 KiB read so decoder
// and TLS state stay intact. Large-frame throughput keeps the existing byte cap.
const READ_EVENT_BATCH: usize = 1024;

pub struct NetConfig {
    pub tls_enabled: bool,
    pub tls_dir: PathBuf,
    pub identity_name: String,
    pub require_client_trust: bool,
    pub held_timeout: Duration,
    pub max_frame_len: u32,
    pub mdns: Option<MdnsConfig>,
}
impl Default for NetConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            tls_dir: crate::default_config_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("tls"),
            identity_name: "computer".into(),
            require_client_trust: true,
            held_timeout: Duration::from_secs(30),
            max_frame_len: MAX_FRAME_LEN,
            mdns: None,
        }
    }
}
pub struct MdnsConfig {
    pub advertise: Option<(String, u16)>,
    pub browse: bool,
    pub loopback_only: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMode {
    Messages,
    Frames,
}
#[derive(Clone, Debug)]
pub struct ConnInfo {
    pub peer: SocketAddr,
    pub outbound: bool,
    pub tls_version: Option<String>,
    pub fingerprint: Option<Fingerprint>,
    pub queued_bytes: usize,
    /// True only after a matching explicit pin or trust-store certificate.
    pub certificate_verified: bool,
}

#[derive(Default)]
struct Budget {
    bytes: AtomicUsize,
    bulk: Mutex<HashMap<ConnId, usize>>,
}
struct Reservation {
    budget: Arc<Budget>,
    bytes: usize,
    bulk: Option<(ConnId, usize)>,
}
impl Reservation {
    fn new(budget: &Arc<Budget>, bytes: usize, bulk: Option<(ConnId, usize)>, control: bool) -> Result<Self, NetError> {
        budget
            .bytes
            .try_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                n.checked_add(bytes)
                    .filter(|v| *v <= MAILBOX_BYTES + if control { 1024 * 1024 } else { 0 })
            })
            .map_err(|_| NetError::QueueFull)?;
        if let Some((conn, wire_bytes)) = bulk {
            let mut bulk = budget.bulk.lock().unwrap_or_else(|e| e.into_inner());
            let current = bulk.entry(conn).or_default();
            if *current + wire_bytes > BULK_WINDOW {
                budget.bytes.fetch_sub(bytes, Ordering::AcqRel);
                return Err(NetError::QueueFull);
            }
            *current += wire_bytes;
        }
        Ok(Self {
            budget: budget.clone(),
            bytes,
            bulk,
        })
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.budget.bytes.fetch_sub(self.bytes, Ordering::AcqRel);
        if let Some((conn, wire_bytes)) = self.bulk {
            let mut bulk = self.budget.bulk.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(bytes) = bulk.get_mut(&conn) {
                *bytes -= wire_bytes;
                if *bytes == 0 {
                    bulk.remove(&conn);
                }
            }
        }
    }
}
struct Packet {
    bytes: Vec<u8>,
    offset: usize,
    priority: Priority,
    absolute: bool,
    handshake_end: bool,
    input_barrier: bool,
    order: u64,
    raw: bool,
    _reservation: Reservation,
}
// Tiny frames still retain a Packet and a VecDeque slot. Allow room for
// geometric queue growth as well, so byte limits cannot admit a million
// eight-byte keepalive packets while omitting their much larger metadata.
const PACKET_OVERHEAD: usize = 2 * std::mem::size_of::<Packet>();
impl Packet {
    fn queued_cost(&self) -> usize {
        self.bytes.len() - self.offset + PACKET_OVERHEAD
    }
}
enum Command {
    Public(NetCommand, Reservation),
    Packet(ConnId, Packet),
}
pub(crate) struct Mailbox {
    sender: SyncSender<Command>,
    waker: Arc<Waker>,
    budget: Arc<Budget>,
    closed: AtomicBool,
}
impl Mailbox {
    fn prepare(&self, cmd: NetCommand) -> Result<Command, NetError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(NetError::Shutdown);
        }
        let (conn, bytes, priority, absolute, handshake_end, input_barrier, raw) = match cmd {
            NetCommand::Send { conn, msg, priority } => {
                let mut bytes = Vec::new();
                openergy_protocol::encode(&msg, &mut bytes).map_err(failed)?;
                let absolute = priority == Priority::Input && matches!(msg, Message::MouseMoveAbs { .. });
                let end = matches!(msg, Message::SetOptions { .. });
                let input_barrier = priority == Priority::Control
                    && matches!(
                        msg,
                        Message::KeyUp { .. } | Message::MouseUp { .. } | Message::Enter { .. } | Message::Leave
                    );
                (conn, bytes, priority, absolute, end, input_barrier, false)
            }
            NetCommand::SendFrame {
                conn,
                payload,
                priority,
            } => {
                if payload.len() < 4 || payload.len() > MAX_FRAME_LEN as usize {
                    return Err(NetError::InvalidFrame);
                }
                let mut bytes = Vec::with_capacity(payload.len() + 4);
                bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                bytes.extend_from_slice(&payload);
                (conn, bytes, priority, false, false, false, true)
            }
            other => {
                return Ok(Command::Public(
                    other,
                    Reservation::new(&self.budget, 128, None, false)?,
                ))
            }
        };
        if bytes.len() < 8
            || bytes.len() > MAX_FRAME_LEN as usize + 4
            || (priority == Priority::Bulk && bytes.len() > BULK_SLICE)
        {
            return Err(NetError::InvalidFrame);
        }
        let reservation = Reservation::new(
            &self.budget,
            bytes.capacity() + PACKET_OVERHEAD,
            (priority == Priority::Bulk).then_some((conn, bytes.len())),
            priority == Priority::Control,
        )?;
        Ok(Command::Packet(
            conn,
            Packet {
                bytes,
                offset: 0,
                priority,
                absolute,
                handshake_end,
                input_barrier,
                order: 0,
                raw,
                _reservation: reservation,
            },
        ))
    }
    pub(crate) fn submit(&self, cmd: NetCommand) -> Result<(), NetError> {
        let command = self.prepare(cmd)?;
        self.sender.try_send(command).map_err(|e| match e {
            mpsc::TrySendError::Full(_) => NetError::QueueFull,
            mpsc::TrySendError::Disconnected(_) => NetError::Shutdown,
        })?;
        self.waker.wake().map_err(failed)
    }
}

#[derive(Default)]
struct Queues {
    input: VecDeque<Packet>,
    control: VecDeque<Packet>,
    bulk: VecDeque<Packet>,
    active: Option<Packet>,
    high_bytes: usize,
    next_order: u64,
    control_barriers: VecDeque<u64>,
}
impl Queues {
    fn push(&mut self, mut packet: Packet) {
        packet.order = self.next_order;
        self.next_order = self.next_order.wrapping_add(1);
        if packet.input_barrier {
            self.control_barriers.push_back(packet.order);
        }
        let queue = match packet.priority {
            Priority::Input => &mut self.input,
            Priority::Control => &mut self.control,
            Priority::Bulk => &mut self.bulk,
        };
        if packet.priority == Priority::Input && packet.absolute && queue.back().is_some_and(|p| p.absolute) {
            if let Some(old) = queue.pop_back() {
                self.high_bytes -= old.queued_cost();
            }
        }
        if packet.priority != Priority::Bulk {
            self.high_bytes += packet.queued_cost();
        }
        queue.push_back(packet);
    }
    fn choose(&mut self, handshake_done: bool, slow: bool) {
        if self.active.is_some() {
            return;
        }
        // Safety releases and screen transitions must follow older Input and
        // precede newer Input. Unrelated Control traffic may still be overtaken.
        // The byte budget bounds live packets well below half the u64 sequence space.
        let barrier_first = self.input.front().is_some_and(|input| {
            self.control_barriers
                .front()
                .is_some_and(|order| input.order.wrapping_sub(*order) < (1u64 << 63))
        });
        self.active = if slow || !handshake_done || barrier_first {
            self.pop_control()
        } else {
            self.input
                .pop_front()
                .or_else(|| self.pop_control())
                .or_else(|| self.bulk.pop_front())
        };
    }
    fn pop_control(&mut self) -> Option<Packet> {
        let packet = self.control.pop_front()?;
        if packet.input_barrier {
            self.control_barriers.pop_front();
        }
        Some(packet)
    }
    fn len(&self) -> usize {
        self.input
            .iter()
            .chain(&self.control)
            .chain(&self.bulk)
            .map(|p| p.bytes.len() - p.offset)
            .sum::<usize>()
            + self.active.as_ref().map_or(0, |p| p.bytes.len() - p.offset)
    }
    fn can_write(&self, handshake_done: bool, slow: bool) -> bool {
        self.active.is_some()
            || !self.control.is_empty()
            || (handshake_done && !slow && (!self.input.is_empty() || !self.bulk.is_empty()))
    }
    fn has_data(&self) -> bool {
        self.active.is_some() || !self.input.is_empty() || !self.control.is_empty() || !self.bulk.is_empty()
    }
    fn discard_low(&mut self) {
        self.high_bytes -= self.input.iter().map(Packet::queued_cost).sum::<usize>();
        self.input.clear();
        self.bulk.clear();
    }
}
struct Connection {
    id: ConnId,
    stream: TcpStream,
    peer: SocketAddr,
    outbound: bool,
    host: String,
    port: u16,
    connecting: bool,
    alternate_addresses: VecDeque<SocketAddr>,
    client_hello_delay: Duration,
    client_hello_at: Option<Instant>,
    deadline: Instant,
    tls: Option<rustls::Connection>,
    expected: Option<Fingerprint>,
    fingerprint: Option<Fingerprint>,
    version: Option<String>,
    cipher: Option<String>,
    require_trust: bool,
    trust_dir: PathBuf,
    held_timeout: Duration,
    held: Option<Instant>,
    announced: bool,
    established: bool,
    read_ready: bool,
    write_blocked: bool,
    queues: Queues,
    decoder: Decoder,
    frames: Vec<u8>,
    max_frame: u32,
    handshake_done: bool,
    failure: Option<String>,
    rate_start: Instant,
    rate_count: u64,
    flood_since: Option<Instant>,
    registered: bool,
    slow: Option<Instant>,
    close_at: Option<Instant>,
    close_notify: bool,
}
impl Connection {
    fn token(&self) -> Token {
        Token(self.id.0 as usize)
    }
    fn retry_address(&mut self, registry: &mio::Registry) -> Result<bool, NetError> {
        while let Some(addr) = self.alternate_addresses.pop_front() {
            NetDriver::check_port(addr.port())?;
            let result = (|| {
                let mut stream = TcpStream::connect(addr)?;
                registry.register(&mut stream, self.token(), Interest::READABLE.add(Interest::WRITABLE))?;
                if self.registered {
                    let _ = registry.deregister(&mut self.stream);
                }
                self.stream = stream;
                self.peer = addr;
                self.registered = true;
                self.read_ready = false;
                self.write_blocked = false;
                self.failure = None;
                Ok::<_, io::Error>(())
            })();
            if result.is_ok() {
                log::trace!(target: "openergy_net::connect_observer",
                    "phase=fallback conn={} addr={} deadline={:?}", self.id.0, self.peer, self.deadline);
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn interest(&self) -> Interest {
        if self.connecting
            || (self.client_hello_at.is_none_or(|d| d <= Instant::now())
                && self.tls.as_ref().is_some_and(|t| t.wants_write()))
            || (self.established
                && self.held.is_none()
                && self.queues.can_write(self.handshake_done, self.slow.is_some()))
        {
            Interest::READABLE.add(Interest::WRITABLE)
        } else {
            Interest::READABLE
        }
    }
    fn sync_registration(&mut self, registry: &mio::Registry) -> io::Result<()> {
        let suspend = self.held.is_some() && !self.tls.as_ref().is_some_and(|t| t.wants_write());
        if suspend {
            if self.registered {
                registry.deregister(&mut self.stream)?;
                self.registered = false;
            }
        } else {
            let token = self.token();
            let interest = self.interest();
            if self.registered {
                registry.reregister(&mut self.stream, token, interest)?;
            } else {
                registry.register(&mut self.stream, token, interest)?;
                self.registered = true;
            }
        }
        Ok(())
    }
    fn info(&self) -> ConnInfo {
        ConnInfo {
            peer: self.peer,
            outbound: self.outbound,
            tls_version: self.version.clone(),
            fingerprint: self.fingerprint,
            queued_bytes: self.queues.len(),
            certificate_verified: self.tls.is_some()
                && self.fingerprint.is_some_and(|fp| {
                    self.expected == Some(fp)
                        || TrustStore::load(&self.trust_dir).is_ok_and(|store| {
                            if self.outbound {
                                store.is_trusted_server(&fp)
                            } else {
                                store.is_trusted_client(&fp)
                            }
                        })
                }),
        }
    }
    fn trust_without_store(&self) -> Option<bool> {
        if !self.require_trust {
            return Some(true);
        }
        let Some(fp) = self.fingerprint else {
            return Some(false);
        };
        if self.expected == Some(fp) {
            return Some(true);
        }
        None
    }
    fn trusted_by(&self, store: &TrustStore) -> bool {
        self.fingerprint.is_some_and(|fp| {
            if self.outbound {
                store.is_trusted_server(&fp)
            } else {
                store.is_trusted_client(&fp)
            }
        })
    }
    fn is_trusted(&self) -> Result<bool, NetError> {
        if let Some(trusted) = self.trust_without_store() {
            return Ok(trusted);
        }
        let store = TrustStore::load(&self.trust_dir).map_err(failed)?;
        Ok(self.trusted_by(&store))
    }
    fn announce(&mut self, out: &mut Vec<NetEvent>) {
        self.held = None;
        self.established = true;
        self.announced = true;
        log::warn!(
            "connection {} established: TLS={:?}, cipher={:?}",
            self.id.0,
            self.version,
            self.cipher
        );
        out.push(NetEvent::Connected {
            conn: self.id,
            peer: self.peer,
            fingerprint: self.fingerprint,
            tls: self.tls.is_some(),
            tls_version: self.version.clone(),
        });
    }
    fn finish_handshake(&mut self, out: &mut Vec<NetEvent>) -> Result<(), NetError> {
        if self.established || self.held.is_some() {
            return Ok(());
        }
        if let Some(tls) = &self.tls {
            if tls.is_handshaking() {
                return Ok(());
            }
            self.fingerprint = tls
                .peer_certificates()
                .and_then(|c| c.first())
                .map(|c| Fingerprint::from_der(c.as_ref()));
            self.version = tls.protocol_version().map(|v| format!("{v:?}"));
            self.cipher = tls.negotiated_cipher_suite().map(|c| format!("{:?}", c.suite()));
            if self.require_trust && self.fingerprint.is_none() {
                return Err(failed("peer omitted its certificate"));
            }
            if !self.is_trusted()? {
                let fp = self.fingerprint.ok_or_else(|| failed("peer certificate is missing"))?;
                self.held = Some(Instant::now() + self.held_timeout);
                self.announced = true;
                out.push(NetEvent::UntrustedPeer {
                    conn: self.id,
                    fingerprint: fp,
                    addr: self.peer,
                });
                return Ok(());
            }
        } else {
            log::warn!("PLAINTEXT connection {}: TLS explicitly disabled", self.id.0);
        }
        self.announce(out);
        Ok(())
    }
    fn feed(&mut self, bytes: &[u8], mode: TransportMode, out: &mut Vec<NetEvent>) -> Result<(), DisconnectReason> {
        if mode == TransportMode::Messages {
            self.decoder.feed(bytes).map_err(|_| DisconnectReason::BadFrame)?;
            while let Some(message) = self.decoder.next_message() {
                let msg = message.map_err(|_| DisconnectReason::BadFrame)?;
                if matches!(msg, Message::SetOptions { .. }) {
                    self.handshake_done = true;
                }
                self.rate_count = self.rate_count.saturating_add(1);
                out.push(NetEvent::Message { conn: self.id, msg });
            }
        } else {
            self.frames.extend_from_slice(bytes);
            let mut consumed = 0;
            while let Some(header) = self.frames.get(consumed..consumed + 4) {
                let mut prefix = [0; 4];
                prefix.copy_from_slice(header);
                let len = u32::from_be_bytes(prefix);
                if len < 4 || len > self.max_frame {
                    return Err(DisconnectReason::BadFrame);
                }
                let end = consumed + 4 + len as usize;
                let Some(payload) = self.frames.get(consumed + 4..end) else {
                    break;
                };
                self.rate_count = self.rate_count.saturating_add(1);
                out.push(NetEvent::Frame {
                    conn: self.id,
                    payload: payload.to_owned(),
                });
                consumed = end;
            }
            if consumed > 0 {
                self.frames.drain(..consumed);
            }
            if self.frames.len() > self.max_frame as usize + 4 {
                return Err(DisconnectReason::BadFrame);
            }
        }
        Ok(())
    }
    fn read(&mut self, mode: TransportMode, out: &mut Vec<NetEvent>) -> Result<(), DisconnectReason> {
        if self.held.is_some() {
            return Ok(());
        }
        let mut budget: usize = 256 * 1024;
        let mut buffer = [0u8; 16 * 1024];
        let initial_events = out.len();
        while budget > 0 && out.len() - initial_events < READ_EVENT_BATCH {
            if self.tls.is_some() {
                if self.established {
                    loop {
                        let count = match self
                            .tls
                            .as_mut()
                            .ok_or(DisconnectReason::TlsFailure)?
                            .reader()
                            .read(&mut buffer)
                        {
                            Ok(0) => return Err(DisconnectReason::ClosedByPeer),
                            Ok(n) => n,
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(_) => return Err(DisconnectReason::TlsFailure),
                        };
                        self.feed(&buffer[..count], mode, out)?;
                        budget = budget.saturating_sub(count);
                        if budget == 0 || out.len() - initial_events >= READ_EVENT_BATCH {
                            break;
                        }
                    }
                }
                if budget == 0 || out.len() - initial_events >= READ_EVENT_BATCH {
                    break;
                }
                if self.read_ready {
                    let count = {
                        let tls = self.tls.as_mut().ok_or(DisconnectReason::TlsFailure)?;
                        match tls.read_tls(&mut self.stream) {
                            Ok(n) => {
                                log::trace!("connection {} read {n} TLS record bytes", self.id.0);
                                n
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                self.read_ready = false;
                                0
                            }
                            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                            Err(_) => return Err(DisconnectReason::TlsFailure),
                        }
                    };
                    if count == 0 && self.read_ready {
                        return Err(DisconnectReason::ClosedByPeer);
                    }
                    if count > 0 {
                        budget = budget.saturating_sub(count);
                        let tls = self.tls.as_mut().ok_or(DisconnectReason::TlsFailure)?;
                        if let Err(error) = tls.process_new_packets() {
                            log::warn!("TLS connection {} failed: {error}", self.id.0);
                            self.failure = Some(error.to_string());
                            // A fatal alert is best effort; never serialize application bytes.
                            let _ = tls.write_tls(&mut self.stream);
                            return Err(DisconnectReason::TlsFailure);
                        }
                    }
                    self.finish_handshake(out).map_err(|_| DisconnectReason::TlsFailure)?;
                    if self.held.is_some() {
                        return Ok(());
                    }
                }
                if !self.read_ready {
                    break;
                }
            } else {
                if !self.read_ready {
                    break;
                }
                match self.stream.read(&mut buffer) {
                    Ok(0) => return Err(DisconnectReason::ClosedByPeer),
                    Ok(n) => {
                        budget -= n.min(budget);
                        self.feed(&buffer[..n], mode, out)?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        self.read_ready = false;
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => return Err(DisconnectReason::ClosedByPeer),
                }
            }
        }
        Ok(())
    }
    fn flush_tls(&mut self) -> Result<(), DisconnectReason> {
        if let Some(tls) = &mut self.tls {
            while tls.wants_write() && !self.write_blocked {
                match tls.write_tls(&mut self.stream) {
                    Ok(0) => return Err(DisconnectReason::ClosedByPeer),
                    Ok(n) => {
                        log::trace!("connection {} wrote {n} TLS record bytes", self.id.0);
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => self.write_blocked = true,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => return Err(DisconnectReason::TlsFailure),
                }
            }
        }
        Ok(())
    }
    fn write(&mut self) -> Result<(), DisconnectReason> {
        if self.client_hello_at.is_some_and(|deadline| Instant::now() < deadline) {
            return Ok(());
        }
        self.client_hello_at = None;
        self.flush_tls()?;
        if !self.established
            || self.held.is_some()
            || self.write_blocked
            || self.tls.as_ref().is_some_and(|t| t.wants_write())
        {
            return Ok(());
        }
        // One whole <=32KiB bulk frame per poll iteration. A partly-written frame
        // must finish before another frame; interleaving its bytes corrupts framing.
        let mut budget = 64 * 1024;
        let mut bulk_done = false;
        while budget > 0 {
            if bulk_done
                && self.queues.active.is_none()
                && self.queues.input.is_empty()
                && self.queues.control.is_empty()
            {
                break;
            }
            self.queues.choose(self.handshake_done, self.slow.is_some());
            let Some(packet) = self.queues.active.as_mut() else {
                break;
            };
            if packet.priority == Priority::Bulk && bulk_done {
                break;
            }
            let end = packet.bytes.len().min(packet.offset + budget);
            let bytes = &packet.bytes[packet.offset..end];
            let result = match &mut self.tls {
                Some(tls) => tls.writer().write(bytes),
                None => self.stream.write(bytes),
            };
            match result {
                Ok(0) => return Err(DisconnectReason::ClosedByPeer),
                Ok(n) => {
                    packet.offset += n;
                    budget -= n;
                    if packet.priority != Priority::Bulk {
                        self.queues.high_bytes -= n;
                    }
                    let done = packet.offset == packet.bytes.len();
                    if done {
                        if packet.priority != Priority::Bulk {
                            self.queues.high_bytes -= PACKET_OVERHEAD;
                        }
                        bulk_done |= packet.priority == Priority::Bulk;
                        self.handshake_done |= packet.handshake_end;
                        self.queues.active = None;
                    }
                    self.flush_tls()?;
                    if self.write_blocked {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    self.write_blocked = true;
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(DisconnectReason::ClosedByPeer),
            }
        }
        if self.close_at.is_some() && !self.queues.has_data() && !self.close_notify {
            self.close_notify = true;
            if let Some(tls) = &mut self.tls {
                tls.send_close_notify();
            }
            self.flush_tls()?;
        }
        Ok(())
    }
}

struct PreparedConfig {
    factory: Option<TlsFactory>,
    fingerprint: Option<Fingerprint>,
    discovery: Option<Discovery>,
}
struct NewConnection {
    peer: SocketAddr,
    outbound: bool,
    host: String,
    tls: Option<rustls::Connection>,
    expected: Option<Fingerprint>,
    deadline: Instant,
}
struct ResolveJob {
    id: u64,
    host: String,
    port: u16,
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
}
struct ResolveResult {
    id: u64,
    addresses: Result<Vec<SocketAddr>, String>,
}
struct Pending {
    host: String,
    port: u16,
    expected: Option<Fingerprint>,
    deadline: Instant,
    cancelled: Arc<AtomicBool>,
}

pub struct NetDriver {
    poll: Poll,
    events: Events,
    config: NetConfig,
    factory: Option<TlsFactory>,
    fingerprint: Option<Fingerprint>,
    trust: TrustStore,
    listener: Option<TcpListener>,
    conns: BTreeMap<ConnId, Connection>,
    next_id: u64,
    commands: Receiver<Command>,
    handle: NetHandle,
    mode: TransportMode,
    connect_timeout: Duration,
    client_hello_delay: Duration,
    heartbeat: Duration,
    last_trust: Instant,
    trust_dirty: bool,
    discovery: Option<Discovery>,
    resolver: Option<SyncSender<ResolveJob>>,
    resolved_sender: mpsc::Sender<ResolveResult>,
    resolved: Receiver<ResolveResult>,
    pending: BTreeMap<u64, Pending>,
    next_resolve: u64,
    dns_outstanding: usize,
}
impl NetDriver {
    pub fn new(config: NetConfig) -> Result<(Self, NetHandle), NetError> {
        let poll = Poll::new().map_err(failed)?;
        let waker = Arc::new(Waker::new(poll.registry(), WAKE).map_err(failed)?);
        let (sender, commands) = mpsc::sync_channel(256);
        let handle = NetHandle {
            inner: Arc::new(Mailbox {
                sender,
                waker,
                budget: Arc::new(Budget::default()),
                closed: AtomicBool::new(false),
            }),
        };
        let (resolved_sender, resolved) = mpsc::channel();
        let trust = TrustStore::load(&config.tls_dir).map_err(failed)?;
        let PreparedConfig {
            factory,
            fingerprint,
            discovery,
        } = Self::prepare_config(&config)?;
        let driver = Self {
            poll,
            events: Events::with_capacity(256),
            config,
            factory,
            fingerprint,
            trust,
            listener: None,
            conns: BTreeMap::new(),
            next_id: 2,
            commands,
            handle: handle.clone(),
            mode: TransportMode::Messages,
            connect_timeout: Duration::from_secs(10),
            // The inherited Synergy/Deskflow accept path can lose readiness when
            // ClientHello arrives in the accept callback. The lab independently
            // requires 100ms here (tools/interop-lab/smoke.py). This is a timer,
            // never a sleep on the io thread; established input has no delay.
            client_hello_delay: Duration::from_millis(100),
            heartbeat: Duration::from_secs(3),
            last_trust: Instant::now(),
            trust_dirty: false,
            discovery,
            resolver: None,
            resolved_sender,
            resolved,
            pending: BTreeMap::new(),
            next_resolve: 0,
            dns_outstanding: 0,
        };
        Ok((driver, handle))
    }
    fn prepare_config(config: &NetConfig) -> Result<PreparedConfig, NetError> {
        if !(4..=MAX_FRAME_LEN).contains(&config.max_frame_len) {
            return Err(failed("max_frame_len must be 4..=4194304"));
        }
        if config.held_timeout.is_zero() || config.held_timeout > Duration::from_secs(300) {
            return Err(failed("held_timeout must be positive and <=300 seconds"));
        }
        let identity = if config.tls_enabled || config.mdns.as_ref().is_some_and(|m| m.advertise.is_some()) {
            Some(Identity::try_load_or_create_in(&config.tls_dir, &config.identity_name)?)
        } else {
            None
        };
        let fingerprint = identity.as_ref().map(|id| id.fingerprint);
        let factory = if config.tls_enabled {
            Some(TlsFactory::new(
                identity.ok_or_else(|| failed("identity missing"))?,
                config.require_client_trust,
            )?)
        } else {
            None
        };
        let discovery = if let Some(options) = &config.mdns {
            let mut discovery = Discovery::with_browse(
                if options.loopback_only {
                    DiscoveryScope::Loopback
                } else {
                    DiscoveryScope::Lan
                },
                5353,
                options.browse,
            )?;
            if let Some((name, port)) = &options.advertise {
                discovery.advertise(
                    name,
                    *port,
                    fingerprint.ok_or_else(|| failed("advertising requires identity"))?,
                )?;
            }
            Some(discovery)
        } else {
            None
        };
        Ok(PreparedConfig {
            factory,
            fingerprint,
            discovery,
        })
    }
    pub fn waker(&self) -> Arc<Waker> {
        self.handle.inner.waker.clone()
    }
    pub fn trust_store(&self) -> &TrustStore {
        &self.trust
    }
    pub fn trust_store_mut(&mut self) -> &mut TrustStore {
        self.trust_dirty = true;
        &mut self.trust
    }
    pub fn identity_fingerprint(&self) -> Option<Fingerprint> {
        self.fingerprint
    }
    pub fn conn_info(&self, conn: ConnId) -> Option<ConnInfo> {
        self.conns.get(&conn).map(Connection::info)
    }
    pub fn tls_cipher(&self, conn: ConnId) -> Option<&str> {
        self.conns.get(&conn).and_then(|c| c.cipher.as_deref())
    }
    pub fn reconfigure(&mut self, config: NetConfig) -> Result<(), NetError> {
        let trust = TrustStore::load(&config.tls_dir).map_err(failed)?;
        let PreparedConfig {
            factory,
            fingerprint,
            discovery,
        } = Self::prepare_config(&config)?;
        self.config = config;
        self.trust = trust;
        self.factory = factory;
        self.fingerprint = fingerprint;
        self.discovery = discovery;
        Ok(())
    }
    /// Apply a transport/listener transaction. Existing peers remain alive on
    /// any preparation or persistence error. The callback runs after live apply.
    pub fn reconfigure_transaction(
        &mut self,
        config: NetConfig,
        listen: Option<SocketAddr>,
        persist: impl FnOnce() -> Result<(), NetError>,
    ) -> Result<(), NetError> {
        let trust = TrustStore::load(&config.tls_dir).map_err(failed)?;
        let PreparedConfig {
            factory,
            fingerprint,
            discovery,
        } = Self::prepare_config(&config)?;
        let keep_listener = self.listener.as_ref().and_then(|l| l.local_addr().ok()) == listen;
        let mut replacement = if !keep_listener {
            if let Some(addr) = listen {
                Self::check_port(addr.port())?;
                let mut listener = TcpListener::bind(addr).map_err(failed)?;
                self.poll
                    .registry()
                    .register(&mut listener, LISTENER, Interest::READABLE)
                    .map_err(failed)?;
                Some(listener)
            } else {
                None
            }
        } else {
            None
        };
        let old_listener = if keep_listener {
            None
        } else {
            std::mem::replace(&mut self.listener, replacement.take())
        };
        let old_config = std::mem::replace(&mut self.config, config);
        let old_trust = std::mem::replace(&mut self.trust, trust);
        let old_factory = std::mem::replace(&mut self.factory, factory);
        let old_fingerprint = std::mem::replace(&mut self.fingerprint, fingerprint);
        let old_discovery = std::mem::replace(&mut self.discovery, discovery);
        if let Err(error) = persist() {
            if !keep_listener {
                if let Some(mut listener) = self.listener.take() {
                    let _ = self.poll.registry().deregister(&mut listener);
                }
                self.listener = old_listener;
            }
            self.config = old_config;
            self.trust = old_trust;
            self.factory = old_factory;
            self.fingerprint = old_fingerprint;
            self.discovery = old_discovery;
            return Err(error);
        }
        if let Some(mut old_listener) = old_listener {
            let _ = self.poll.registry().deregister(&mut old_listener);
        }
        Ok(())
    }
    /// Diagnostic frame mode can be enabled only before opening any connections.
    pub fn set_transport_mode(&mut self, mode: TransportMode) -> Result<(), NetError> {
        if !self.conns.is_empty() || !self.pending.is_empty() {
            return Err(failed("cannot change framing mode with live connections"));
        }
        self.mode = mode;
        Ok(())
    }
    /// Applies to future outbound TLS connections. Zero is useful for peers
    /// known to handle an immediate ClientHello; the compatibility default is 100ms.
    pub fn set_client_hello_delay(&mut self, delay: Duration) -> Result<(), NetError> {
        if delay > Duration::from_secs(1) {
            return Err(failed("ClientHello delay must be <=1 second"));
        }
        self.client_hello_delay = delay;
        Ok(())
    }
    pub fn set_connect_timeout(&mut self, timeout: Duration) -> Result<(), NetError> {
        if timeout.is_zero() || timeout > Duration::from_secs(300) {
            return Err(failed("connect timeout must be positive and <=300 seconds"));
        }
        self.connect_timeout = timeout;
        Ok(())
    }
    pub fn set_heartbeat_interval(&mut self, interval: Duration) -> Result<(), NetError> {
        if interval.is_zero() || interval > Duration::from_secs(300) {
            return Err(failed("heartbeat interval must be positive and <=300 seconds"));
        }
        self.heartbeat = interval;
        Ok(())
    }
    pub fn execute(&mut self, cmd: NetCommand, out: &mut Vec<NetEvent>) {
        if let Err(error) = self.execute_checked(cmd, out) {
            out.push(NetEvent::Error {
                conn: None,
                message: error.to_string(),
            });
        }
    }
    /// Preserve typed send failures for the session owner.
    pub fn execute_checked(&mut self, cmd: NetCommand, out: &mut Vec<NetEvent>) -> Result<(), NetError> {
        if let NetCommand::Send {
            conn,
            priority: Priority::Input,
            ..
        } = &cmd
        {
            if self.conns.get(conn).is_some_and(|c| c.queues.high_bytes >= HIGH_WATER) {
                return Err(NetError::QueueFull);
            }
        }
        let command = if matches!(cmd, NetCommand::Send { .. } | NetCommand::SendFrame { .. }) {
            self.handle.inner.prepare(cmd)
        } else {
            // Same-thread control must remain available at the mailbox byte limit,
            // especially Shutdown/Close while the peer is already backpressured.
            Reservation::new(&self.handle.inner.budget, 0, None, true)
                .map(|reservation| Command::Public(cmd, reservation))
        };
        let command = command?;
        if let Command::Packet(conn, packet) = &command {
            if packet.priority == Priority::Input
                && self
                    .conns
                    .get(conn)
                    .is_some_and(|c| c.queues.high_bytes.saturating_add(packet.queued_cost()) > HIGH_WATER)
            {
                return Err(NetError::QueueFull);
            }
        }
        self.command(command, out);
        Ok(())
    }
    pub fn cancel_clipboard(&mut self, conn: ConnId) {
        if let Some(connection) = self.conns.get_mut(&conn) {
            connection.queues.bulk.clear();
        }
    }

    fn command(&mut self, command: Command, out: &mut Vec<NetEvent>) {
        match command {
            Command::Packet(id, packet) => {
                let Some(c) = self.conns.get_mut(&id) else {
                    out.push(NetEvent::Error {
                        conn: Some(id),
                        message: "connection no longer exists".into(),
                    });
                    return;
                };
                if packet.raw && self.mode != TransportMode::Frames {
                    out.push(NetEvent::Error {
                        conn: Some(id),
                        message: "SendFrame requires explicit diagnostic frame mode".into(),
                    });
                    return;
                }
                if packet.bytes.len() > c.max_frame as usize + 4 {
                    out.push(NetEvent::Error {
                        conn: Some(id),
                        message: "frame exceeds connection limit".into(),
                    });
                    return;
                }
                if packet.priority != Priority::Control && (c.close_at.is_some() || c.slow.is_some()) {
                    out.push(NetEvent::Error {
                        conn: Some(id),
                        message: "connection is closing or unhealthy".into(),
                    });
                    return;
                }
                c.queues.push(packet);
                if c.queues.high_bytes > HIGH_WATER && c.slow.is_none() {
                    log::warn!(
                        "connection {} exceeded 4 MiB input/control queue budget; closing SlowPeer after heartbeat",
                        id.0
                    );
                    c.slow = Some(Instant::now() + self.heartbeat);
                    c.queues.discard_low();
                }
                if let Err(e) = c.sync_registration(self.poll.registry()) {
                    out.push(NetEvent::Error {
                        conn: Some(id),
                        message: e.to_string(),
                    });
                    self.remove(id, DisconnectReason::Other, out);
                }
            }
            Command::Public(cmd, _reservation) => match cmd {
                NetCommand::Listen { addr } => {
                    if let Err(e) = self.listen(addr) {
                        out.push(NetEvent::ListenFailed {
                            addr,
                            error: e.to_string(),
                        });
                    } else if let Some(listener) = &self.listener {
                        match listener.local_addr() {
                            Ok(addr) => out.push(NetEvent::Listening { addr }),
                            Err(e) => out.push(NetEvent::ListenFailed {
                                addr,
                                error: e.to_string(),
                            }),
                        }
                    }
                }
                NetCommand::Connect { host, port } => self.connect(host, port, None, out),
                NetCommand::ConnectPinned {
                    host,
                    port,
                    fingerprint,
                } => self.connect(host, port, Some(fingerprint), out),
                NetCommand::Approve { conn } => {
                    if let Some(fp) = self.conns.get(&conn).and_then(|c| c.held.and(c.fingerprint)) {
                        if let Err(e) = self.approve_held(&fp, out) {
                            out.push(NetEvent::Error {
                                conn: Some(conn),
                                message: e.to_string(),
                            });
                        }
                    }
                }
                NetCommand::Reject { conn } => self.remove(conn, DisconnectReason::TlsFailure, out),
                NetCommand::Close { conn } => self.close(conn, out),
                NetCommand::ReloadTrust => {
                    self.refresh_trust(out);
                }
                command @ (NetCommand::Shutdown | NetCommand::DisconnectAll) => {
                    if matches!(command, NetCommand::Shutdown) {
                        if let Some(mut listener) = self.listener.take() {
                            let _ = self.poll.registry().deregister(&mut listener);
                        }
                    }
                    for (_, p) in std::mem::take(&mut self.pending) {
                        p.cancelled.store(true, Ordering::Release);
                        out.push(NetEvent::ConnectFailed {
                            host: p.host,
                            port: p.port,
                            error: "shutdown".into(),
                        });
                    }
                    let ids: Vec<_> = self.conns.keys().copied().collect();
                    for id in ids {
                        self.remove(id, DisconnectReason::Shutdown, out);
                    }
                }
                NetCommand::StopListening => {
                    if let Some(mut listener) = self.listener.take() {
                        let _ = self.poll.registry().deregister(&mut listener);
                    }
                }
                NetCommand::Send { .. } | NetCommand::SendFrame { .. } => {}
            },
        }
    }
    fn listen(&mut self, addr: SocketAddr) -> Result<(), NetError> {
        Self::check_port(addr.port())?;
        if self.listener.as_ref().and_then(|l| l.local_addr().ok()) == Some(addr) {
            return Ok(());
        }
        let mut listener = TcpListener::bind(addr).map_err(failed)?;
        if let Some(mut old) = self.listener.take() {
            self.poll.registry().deregister(&mut old).map_err(failed)?;
        }
        self.poll
            .registry()
            .register(&mut listener, LISTENER, Interest::READABLE)
            .map_err(failed)?;
        self.listener = Some(listener);
        Ok(())
    }
    pub(crate) fn check_port(port: u16) -> Result<(), NetError> {
        if port == 24800 && std::env::var_os("OPENERGY_ALLOW_PORT_24800").as_deref() != Some(std::ffi::OsStr::new("1"))
        {
            Err(NetError::ProductionPortDenied)
        } else {
            Ok(())
        }
    }
    fn connect(&mut self, host: String, port: u16, expected: Option<Fingerprint>, out: &mut Vec<NetEvent>) {
        let result = (|| {
            Self::check_port(port)?;
            if expected.is_some() && self.factory.is_none() {
                return Err(failed("tls_pin_mismatch: a certificate pin requires TLS to be enabled"));
            }
            if self.conns.len() + self.pending.len() >= MAX_CONNECTIONS {
                return Err(failed("16-connection limit"));
            }
            if let Ok(ip) = host.parse::<IpAddr>() {
                return self.add_outbound(
                    host.clone(),
                    SocketAddr::new(ip, port),
                    expected,
                    Instant::now() + self.connect_timeout,
                );
            }
            if self.dns_outstanding >= MAX_CONNECTIONS {
                return Err(failed("DNS request limit"));
            }
            self.start_resolver()?;
            let id = self.next_resolve;
            self.next_resolve = self
                .next_resolve
                .checked_add(1)
                .ok_or_else(|| failed("DNS id exhausted"))?;
            let deadline = Instant::now() + self.connect_timeout;
            let cancelled = Arc::new(AtomicBool::new(false));
            self.resolver
                .as_ref()
                .ok_or_else(|| failed("DNS worker unavailable"))?
                .try_send(ResolveJob {
                    id,
                    host: host.clone(),
                    port,
                    deadline,
                    cancelled: cancelled.clone(),
                })
                .map_err(failed)?;
            self.pending.insert(
                id,
                Pending {
                    host: host.clone(),
                    port,
                    expected,
                    deadline,
                    cancelled,
                },
            );
            self.dns_outstanding += 1;
            Ok(())
        })();
        if let Err(e) = result {
            out.push(NetEvent::ConnectFailed {
                host,
                port,
                error: e.to_string(),
            });
        }
    }
    fn start_resolver(&mut self) -> Result<(), NetError> {
        if self.resolver.is_some() {
            return Ok(());
        }
        let (sender, receiver) = mpsc::sync_channel::<ResolveJob>(MAX_CONNECTIONS);
        let results = self.resolved_sender.clone();
        let waker = self.waker();
        // getaddrinfo can block. One lazy, bounded worker keeps it off the io loop.
        // Cancelled/expired queued jobs still send one completion to release their
        // outstanding slot. An active OS lookup cannot be cancelled and keeps its
        // slot until it returns. Dropping the driver also cancels queued work.
        std::thread::Builder::new()
            .name("openergy-dns".into())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    let addresses = if job.cancelled.load(Ordering::Acquire) {
                        Err("shutdown".into())
                    } else if job.deadline <= Instant::now() {
                        Err("connect_timeout: no response before deadline".into())
                    } else {
                        (job.host.as_str(), job.port)
                            .to_socket_addrs()
                            .map(|a| a.collect())
                            .map_err(|e| format!("dns_failed: {e}"))
                    };
                    if results.send(ResolveResult { id: job.id, addresses }).is_err() {
                        break;
                    }
                    let _ = waker.wake();
                }
            })
            .map_err(failed)?;
        self.resolver = Some(sender);
        Ok(())
    }
    fn add_outbound(
        &mut self,
        host: String,
        addr: SocketAddr,
        expected: Option<Fingerprint>,
        deadline: Instant,
    ) -> Result<(), NetError> {
        // Check again after DNS: explicit lab denial can change while resolving.
        Self::check_port(addr.port())?;
        // Resolution may finish after reconfigure disabled TLS. Preserve the pin
        // requirement across that asynchronous boundary before opening a socket.
        if expected.is_some() && self.factory.is_none() {
            return Err(failed("tls_pin_mismatch: a certificate pin requires TLS to be enabled"));
        }
        let stream = TcpStream::connect(addr).map_err(crate::connect_error::io_failure)?;
        let tls = self.factory.as_ref().map(|f| f.client(&host, expected)).transpose()?;
        self.add_connection(
            stream,
            NewConnection {
                peer: addr,
                outbound: true,
                host,
                tls,
                expected,
                deadline,
            },
        )
    }
    fn add_connection(&mut self, mut stream: TcpStream, params: NewConnection) -> Result<(), NetError> {
        let NewConnection {
            peer,
            outbound,
            host,
            tls,
            expected,
            deadline,
        } = params;
        // On Darwin, socket options on a refused pending connect can return
        // EINVAL before SO_ERROR is read. Wait for completion for outbound TCP.
        if !outbound {
            stream.set_nodelay(true).map_err(failed)?;
        }
        let id = ConnId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .filter(|v| *v <= usize::MAX as u64)
            .ok_or_else(|| failed("connection id exhausted"))?;
        self.poll
            .registry()
            .register(
                &mut stream,
                Token(id.0 as usize),
                Interest::READABLE.add(Interest::WRITABLE),
            )
            .map_err(failed)?;
        self.conns.insert(
            id,
            Connection {
                id,
                stream,
                peer,
                outbound,
                host,
                port: peer.port(),
                connecting: outbound,
                alternate_addresses: VecDeque::new(),
                client_hello_delay: self.client_hello_delay,
                client_hello_at: None,
                deadline,
                tls,
                expected,
                fingerprint: None,
                version: None,
                cipher: None,
                require_trust: self.config.tls_enabled && (outbound || self.config.require_client_trust),
                trust_dir: self.config.tls_dir.clone(),
                held_timeout: self.config.held_timeout,
                held: None,
                announced: false,
                established: false,
                read_ready: false,
                write_blocked: false,
                queues: Queues::default(),
                decoder: Decoder::new(self.config.max_frame_len),
                frames: Vec::new(),
                max_frame: self.config.max_frame_len,
                handshake_done: self.mode == TransportMode::Frames,
                failure: None,
                rate_start: Instant::now(),
                rate_count: 0,
                flood_since: None,
                registered: true,
                slow: None,
                close_at: None,
                close_notify: false,
            },
        );
        log::trace!(target: "openergy_net::connect_observer",
            "phase=attempt conn={} addr={} outbound={} deadline={:?}", id.0, peer, outbound, deadline);
        Ok(())
    }
    fn accept(&mut self, out: &mut Vec<NetEvent>) {
        loop {
            let Some(listener) = &mut self.listener else {
                return;
            };
            match listener.accept() {
                Ok((stream, peer)) => {
                    if self.conns.len() + self.pending.len() >= MAX_CONNECTIONS {
                        drop(stream);
                        continue;
                    }
                    let result = self
                        .factory
                        .as_ref()
                        .map(TlsFactory::server)
                        .transpose()
                        .and_then(|tls| {
                            self.add_connection(
                                stream,
                                NewConnection {
                                    peer,
                                    outbound: false,
                                    host: peer.ip().to_string(),
                                    tls,
                                    expected: None,
                                    deadline: Instant::now() + self.connect_timeout,
                                },
                            )
                        });
                    if let Err(e) = result {
                        out.push(NetEvent::Error {
                            conn: None,
                            message: e.to_string(),
                        });
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    out.push(NetEvent::Error {
                        conn: None,
                        message: e.to_string(),
                    });
                    return;
                }
            }
        }
    }
    fn close(&mut self, id: ConnId, out: &mut Vec<NetEvent>) {
        let Some(c) = self.conns.get_mut(&id) else {
            return;
        };
        if !c.established || c.held.is_some() {
            self.remove(id, DisconnectReason::Shutdown, out);
            return;
        }
        c.queues.discard_low();
        c.close_at.get_or_insert_with(|| Instant::now() + CLOSE_TIMEOUT);
    }
    fn remove(&mut self, id: ConnId, reason: DisconnectReason, out: &mut Vec<NetEvent>) {
        if let Some(mut c) = self.conns.remove(&id) {
            let _ = self.poll.registry().deregister(&mut c.stream);
            let _ = c.stream.shutdown(Shutdown::Both);
            if c.outbound && !c.announced {
                out.push(NetEvent::ConnectFailed {
                    host: c.host,
                    port: c.port,
                    error: c.failure.take().unwrap_or_else(|| match reason {
                        DisconnectReason::TlsFailure => "tls_error: TLS handshake failed".into(),
                        DisconnectReason::Shutdown => "shutdown".into(),
                        _ => "connect_failed: connection closed".into(),
                    }),
                });
                if reason == DisconnectReason::Shutdown {
                    out.push(NetEvent::Disconnected { conn: id, reason });
                }
            } else {
                out.push(NetEvent::Disconnected { conn: id, reason });
            }
        }
    }
    pub fn approve_held(&mut self, fp: &Fingerprint, out: &mut Vec<NetEvent>) -> Result<(), NetError> {
        let ids: Vec<_> = self
            .conns
            .iter()
            .filter(|(_, c)| c.held.is_some() && c.fingerprint == Some(*fp))
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            if let Some(c) = self.conns.get_mut(&id) {
                if c.held.is_some_and(|deadline| deadline <= Instant::now()) {
                    self.remove(id, DisconnectReason::TlsFailure, out);
                    continue;
                }
                let mut store = TrustStore::load(&c.trust_dir).map_err(failed)?;
                if c.outbound {
                    store.try_trust_server(*fp).map_err(failed)?;
                } else {
                    store.try_trust_client(*fp).map_err(failed)?;
                }
                c.announce(out);
                c.read_ready = true;
            }
        }
        self.trust.reload().map_err(failed)?;
        Ok(())
    }
    pub fn reject_held(&mut self, fp: &Fingerprint, out: &mut Vec<NetEvent>) {
        let ids: Vec<_> = self
            .conns
            .iter()
            .filter(|(_, c)| c.held.is_some() && c.fingerprint == Some(*fp))
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.remove(id, DisconnectReason::TlsFailure, out);
        }
    }
    fn refresh_trust(&mut self, out: &mut Vec<NetEvent>) {
        let read_error = self.trust.reload().err().map(|error| error.to_string());
        if let Some(message) = &read_error {
            out.push(NetEvent::Error {
                conn: None,
                message: message.clone(),
            });
        }
        let active = |c: &Connection| {
            c.held.is_some_and(|deadline| deadline > Instant::now()) || (c.established && c.require_trust)
        };
        let mut decisions = Vec::new();
        let mut other_dirs = Vec::new();
        for (&id, c) in &self.conns {
            if !active(c) {
                continue;
            }
            if let Some(trusted) = c.trust_without_store() {
                decisions.push((id, Ok(trusted)));
            } else if c.trust_dir.as_path() == self.trust.dir() {
                // A failed reload must not fall back to the last good store.
                decisions.push((
                    id,
                    match &read_error {
                        Some(error) => Err(failed(error)),
                        None => Ok(c.trusted_by(&self.trust)),
                    },
                ));
            } else if !other_dirs.contains(&c.trust_dir) {
                other_dirs.push(c.trust_dir.clone());
            }
        }
        // Old connections can retain another directory after reconfiguration.
        // Read each directory once per pass and drop it before reading the next;
        // do not retain whole stores for every distinct connection directory.
        for dir in other_dirs {
            let store = TrustStore::load(&dir).map_err(|error| error.to_string());
            for (&id, c) in &self.conns {
                if active(c) && c.trust_dir == dir && c.trust_without_store().is_none() {
                    decisions.push((id, store.as_ref().map(|store| c.trusted_by(store)).map_err(failed)));
                }
            }
        }
        let mut failures = Vec::new();
        for (id, trusted) in decisions {
            let Some(c) = self.conns.get_mut(&id) else { continue };
            if c.held.is_some() && c.held.is_some_and(|d| d > Instant::now()) {
                match trusted {
                    Ok(true) => {
                        c.announce(out);
                        c.read_ready = true;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        out.push(NetEvent::Error {
                            conn: Some(id),
                            message: e.to_string(),
                        });
                        failures.push(id);
                    }
                }
            } else if c.established && c.require_trust {
                match trusted {
                    Ok(true) => {}
                    Ok(false) => failures.push(id),
                    Err(error) => {
                        out.push(NetEvent::Error {
                            conn: Some(id),
                            message: error.to_string(),
                        });
                        failures.push(id);
                    }
                }
            }
        }
        for id in failures {
            self.remove(id, DisconnectReason::TlsFailure, out);
        }
        self.trust_dirty = false;
        self.last_trust = Instant::now();
    }
    fn maintenance(&mut self, out: &mut Vec<NetEvent>) {
        while let Ok(result) = self.resolved.try_recv() {
            self.dns_outstanding = self.dns_outstanding.saturating_sub(1);
            let Some(p) = self.pending.remove(&result.id) else {
                continue;
            };
            let error = if p.deadline <= Instant::now() {
                Some("connect_timeout: no response before deadline".into())
            } else {
                match result.addresses {
                    Ok(addresses) => {
                        let mut addresses = addresses.into_iter();
                        let mut error = Some("dns_failed: hostname has no usable address".into());
                        while let Some(addr) = addresses.next() {
                            let id = ConnId(self.next_id);
                            match self.add_outbound(p.host.clone(), addr, p.expected, p.deadline) {
                                Ok(()) => {
                                    if let Some(c) = self.conns.get_mut(&id) {
                                        c.alternate_addresses = addresses.collect();
                                    }
                                    error = None;
                                    break;
                                }
                                Err(e) => error = Some(e.to_string()),
                            }
                        }
                        error
                    }
                    Err(e) => Some(e),
                }
            };
            if let Some(error) = error {
                out.push(NetEvent::ConnectFailed {
                    host: p.host,
                    port: p.port,
                    error,
                });
            }
        }
        let expired: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, p)| p.deadline <= Instant::now())
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            if let Some(p) = self.pending.remove(&id) {
                p.cancelled.store(true, Ordering::Release);
                out.push(NetEvent::ConnectFailed {
                    host: p.host,
                    port: p.port,
                    error: "connect_timeout: no response before deadline".into(),
                });
            }
        }
        let now = Instant::now();
        let mut expired = Vec::new();
        for (id, c) in &mut self.conns {
            if c.rate_start.elapsed() >= Duration::from_secs(1) {
                if c.rate_count as f64 / c.rate_start.elapsed().as_secs_f64() > 20_000.0 {
                    c.flood_since.get_or_insert(c.rate_start);
                } else {
                    c.flood_since = None;
                }
                c.rate_start = now;
                c.rate_count = 0;
            }
            let reason = if c
                .flood_since
                .is_some_and(|d| now.duration_since(d) >= Duration::from_secs(5))
            {
                Some(DisconnectReason::Flood)
            } else if c.slow.is_some_and(|d| now >= d) {
                Some(DisconnectReason::SlowPeer)
            } else if !c.established && c.held.is_none() && now >= c.deadline {
                c.failure = Some("connect_timeout: no response before deadline".into());
                Some(DisconnectReason::Other)
            } else if c.held.is_some_and(|d| now >= d) {
                Some(DisconnectReason::TlsFailure)
            } else if c.close_at.is_some_and(|d| now >= d) {
                Some(DisconnectReason::Shutdown)
            } else {
                None
            };
            if let Some(reason) = reason {
                expired.push((*id, reason));
            }
        }
        for (id, reason) in expired {
            self.remove(id, reason, out);
        }
        if self.trust_dirty
            || (self.conns.values().any(|c| c.held.is_some()) && self.last_trust.elapsed() >= TRUST_POLL)
        {
            self.refresh_trust(out);
        }
        if let Some(discovery) = &mut self.discovery {
            if let Some(peers) = discovery.poll() {
                out.push(NetEvent::Discovered(peers));
            }
            for message in discovery.errors() {
                out.push(NetEvent::Error { conn: None, message });
            }
        }
    }
    fn wait_time(&self, requested: Option<Duration>) -> Option<Duration> {
        let now = Instant::now();
        let mut wait = requested;
        let mut cap = |duration: Duration| {
            wait = Some(wait.map_or(duration, |v| v.min(duration)));
        };
        for c in self.conns.values() {
            if !c.established && c.held.is_none() {
                cap(c.deadline.saturating_duration_since(now));
            }
            if let Some(d) = c.held {
                cap(d.saturating_duration_since(now));
                cap(TRUST_POLL.saturating_sub(self.last_trust.elapsed()));
            }
            if let Some(d) = c.slow {
                cap(d.saturating_duration_since(now));
            }
            if let Some(d) = c.close_at {
                cap(d.saturating_duration_since(now));
            }
            if let Some(d) = c.client_hello_at {
                cap(d.saturating_duration_since(now));
            }
            if !c.connecting
                && c.held.is_none()
                && (c.read_ready
                    || (!c.write_blocked
                        && ((c.client_hello_at.is_none_or(|d| d <= now)
                            && c.tls.as_ref().is_some_and(|t| t.wants_write()))
                            || (c.established && c.queues.can_write(c.handshake_done, c.slow.is_some())))))
            {
                cap(Duration::ZERO);
            }
        }
        for p in self.pending.values() {
            cap(p.deadline.saturating_duration_since(now));
        }
        if self.discovery.is_some() {
            cap(Duration::from_secs(1));
        }
        wait
    }
    /// One bounded io iteration. Events append to `out`; the caller owns draining it.
    pub fn poll(&mut self, timeout: Option<Duration>, out: &mut Vec<NetEvent>) -> Result<(), NetError> {
        let start = Instant::now();
        let initial = out.len();
        for _ in 0..256 {
            let Ok(command) = self.commands.try_recv() else {
                break;
            };
            self.command(command, out);
        }
        self.maintenance(out);
        let remaining = timeout.map(|t| t.saturating_sub(start.elapsed()));
        let wait = if out.len() != initial {
            Some(Duration::ZERO)
        } else {
            self.wait_time(remaining)
        };
        match self.poll.poll(&mut self.events, wait) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(failed(e)),
        }
        let ready: Vec<_> = self
            .events
            .iter()
            .map(|e| {
                (
                    e.token(),
                    e.is_readable(),
                    e.is_writable(),
                    e.is_error() || e.is_read_closed() || e.is_write_closed(),
                )
            })
            .collect();
        for (token, readable, writable, closed) in ready {
            if token == WAKE {
                for _ in 0..256 {
                    let Ok(command) = self.commands.try_recv() else {
                        break;
                    };
                    self.command(command, out);
                }
            } else if token == LISTENER {
                self.accept(out);
            } else if let Some(c) = self.conns.get_mut(&ConnId(token.0 as u64)) {
                c.read_ready |= readable || closed;
                if writable {
                    c.write_blocked = false;
                }
                if c.connecting && (readable || writable || closed) {
                    // A retry keeps its token, so another event in this batch
                    // can describe the previous socket. Empty SO_ERROR alone
                    // does not prove the new nonblocking connect completed.
                    let completion = c.stream.take_error().and_then(|error| match error {
                        Some(error) => Err(error),
                        None => c.stream.peer_addr().map(|_| ()),
                    });
                    match completion {
                        Ok(()) => {
                            c.connecting = false;
                            log::trace!(target: "openergy_net::connect_observer",
                                "phase=tcp_ready conn={} addr={} tls={} deadline={:?}",
                                c.id.0, c.peer, c.tls.is_some(), c.deadline);
                            let _ = c.stream.set_nodelay(true);
                            if c.tls.is_some() {
                                c.client_hello_at = Some(Instant::now() + c.client_hello_delay);
                            }
                        }
                        Err(error) if error.kind() == io::ErrorKind::NotConnected => continue,
                        #[cfg(unix)]
                        Err(error) if error.raw_os_error() == Some(libc::EINPROGRESS) => continue,
                        Err(error) => {
                            log::trace!(target: "openergy_net::connect_observer",
                                "phase=failed conn={} addr={} kind={:?} deadline={:?}",
                                c.id.0, c.peer, error.kind(), c.deadline);
                            c.failure = Some(match c.retry_address(self.poll.registry()) {
                                Ok(true) => continue,
                                Err(error) => error.to_string(),
                                Ok(false) => crate::connect_error::io_failure(error).to_string(),
                            });
                            self.remove(ConnId(token.0 as u64), DisconnectReason::Other, out);
                            continue;
                        }
                    }
                }
            }
        }
        let ids: Vec<_> = self.conns.keys().copied().collect();
        for id in ids {
            let Some(c) = self.conns.get_mut(&id) else {
                continue;
            };
            if c.connecting {
                continue;
            }
            let result = c
                .finish_handshake(out)
                .map_err(|_| DisconnectReason::TlsFailure)
                .and_then(|_| c.read(self.mode, out))
                .and_then(|_| c.write());
            if let Err(reason) = result {
                self.remove(id, reason, out);
                continue;
            }
            let done = c.close_notify && !c.queues.has_data() && !c.tls.as_ref().is_some_and(|t| t.wants_write());
            if done {
                self.remove(id, DisconnectReason::Shutdown, out);
                continue;
            }
            if let Err(e) = c.sync_registration(self.poll.registry()) {
                out.push(NetEvent::Error {
                    conn: Some(id),
                    message: e.to_string(),
                });
                self.remove(id, DisconnectReason::Other, out);
            }
        }
        self.maintenance(out);
        Ok(())
    }
}
impl Drop for NetDriver {
    fn drop(&mut self) {
        self.handle.inner.closed.store(true, Ordering::Release);
        for pending in self.pending.values() {
            pending.cancelled.store(true, Ordering::Release);
        }
    }
}
