use crate::{
    adapter,
    events::{EventBus, EventQueue},
    ipc_server, lifetime,
    logger::{unix_ms, Logger},
    workers::{self, CaptureCommand, ClipboardCommand, ClipboardEvent, FileCommand, FileEvent},
    VERSION,
};
use openergy_config::{Config, FileConflict, FileReceiveMode, Role};
use openergy_ipc::{self as ipc, methods as m, Event, IpcError, Request, Response, Status};
use openergy_net::{Fingerprint, NetCommand, NetDriver, NetEvent, Priority};
use openergy_platform::{ClipboardData, Grant, InputEvent, InputSink, Permissions, ScreenRect, ScreenSystem};
use openergy_protocol::{Chunk, ClipboardId, FileDataChunk, Message};
use openergy_session::{
    file::{self as sfile, Reason as FileReason},
    layout::Rect,
    Action, Client, ClientEvent, ConnId, Server, ServerEvent, SessionSnapshot,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    time::{Duration, Instant},
};

pub enum IoCommand {
    Ipc {
        req: Request,
        reply: Sender<Response>,
        events: Arc<EventQueue>,
    },
}
struct Connection {
    address: String,
    fingerprint: Option<String>,
    tls: bool,
    tls_version: Option<String>,
    certificate_verified: bool,
    since: u64,
}
#[derive(Clone)]
enum Machine {
    Off,
    Server(Server),
    Client(Client),
}
impl Machine {
    fn new(cfg: Arc<Config>) -> Self {
        match cfg.role {
            Role::Off => Self::Off,
            Role::Server => Self::Server(Server::new(cfg)),
            Role::Client => Self::Client(Client::new(cfg.clone(), cfg.name.clone())),
        }
    }
    fn connected(&self) -> bool {
        match self {
            Self::Server(s) => s.is_connected(),
            Self::Client(c) => c.is_connected(),
            Self::Off => false,
        }
    }
    fn remote_active(&self) -> bool {
        match self {
            Self::Server(s) => s.remote_active(),
            Self::Client(c) => c.remote_active(),
            Self::Off => false,
        }
    }
    fn snapshot(&self) -> SessionSnapshot {
        match self {
            Self::Off => SessionSnapshot::default(),
            Self::Server(s) => s.snapshot(),
            Self::Client(c) => c.snapshot(),
        }
    }
    fn tick(&mut self, now: u64, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::TimerTick { now_ms: now }, out),
            Self::Client(c) => c.handle(ClientEvent::TimerTick { now_ms: now }, out),
            _ => {}
        }
    }
    fn shutdown(&mut self, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::Shutdown, out),
            Self::Client(c) => c.handle(ClientEvent::Shutdown, out),
            _ => {}
        }
    }
    fn geometry(&mut self, screens: &[ScreenRect], cursor: (i32, i32), changed: bool, out: &mut Vec<Action>) {
        let rect = display_rect(screens);
        let displays = screens
            .iter()
            .map(|d| Rect {
                x: d.x,
                y: d.y,
                w: d.w,
                h: d.h,
            })
            .collect::<Vec<_>>();
        match self {
            Self::Server(s) => {
                s.handle(ServerEvent::LocalScreen { rect, cursor, displays }, out);
                if changed {
                    s.handle(ServerEvent::DisplayChanged, out);
                }
            }
            Self::Client(c) => {
                c.handle(ClientEvent::LocalScreen { rect, cursor, displays }, out);
                if changed {
                    c.handle(ClientEvent::DisplayChanged, out);
                }
            }
            _ => {}
        }
    }
    fn languages(&mut self, languages: openergy_platform::KeyboardLanguages, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::LocalLanguages(languages), out),
            Self::Client(c) => c.handle(ClientEvent::LocalLanguages(languages), out),
            Self::Off => {}
        }
    }
    fn clipboard_changed(&mut self, id: ClipboardId, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::LocalClipboardChanged { id }, out),
            Self::Client(c) => c.handle(ClientEvent::LocalClipboardChanged { id }, out),
            _ => {}
        }
    }
    fn payload(&mut self, id: ClipboardId, data: ClipboardData, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::ClipboardPayload { id, data }, out),
            Self::Client(c) => c.handle(ClientEvent::ClipboardPayload { id, data }, out),
            _ => {}
        }
    }
    fn clipboard_read_failed(&mut self, id: ClipboardId, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(ServerEvent::ClipboardReadFailed { id }, out),
            Self::Client(c) => c.handle(ClientEvent::ClipboardReadFailed { id }, out),
            Self::Off => {}
        }
    }
    fn alloc_xfer(&mut self) -> u64 {
        match self {
            Self::Server(s) => s.alloc_xfer(),
            Self::Client(c) => c.alloc_xfer(),
            Self::Off => 0,
        }
    }
    fn next_xfer(&self) -> u64 {
        match self {
            Self::Server(s) => s.next_xfer(),
            Self::Client(c) => c.next_xfer(),
            Self::Off => 1,
        }
    }
    fn set_next_xfer(&mut self, next: u64) {
        match self {
            Self::Server(s) => s.set_next_xfer(next),
            Self::Client(c) => c.set_next_xfer(next),
            Self::Off => {}
        }
    }
}
// The driver admits `BULK_WINDOW` of Bulk wire bytes per connection.
// Chunks the queue cannot admit yet wait here, gated on the driver's
// queued-bytes accessor, instead of aborting the transfer. Clipboard
// aborts after BULK_STALL_MS without progress; file streams get the
// longer max(30 s, 3 x heartbeat) rule (design §6.2). Entries are
// tagged per stream so one abort never drops the other stream.
const BULK_GATE_BYTES: usize = openergy_net::BULK_WINDOW;
const BULK_STALL_MS: u64 = 2_000;
const BULK_DEFERRED_CAP: usize = 16 * 1024 * 1024;
/// Initial file chunks of downstream credit (design §6.1): one bulk
/// window of 32 KiB slices.
const FILE_INITIAL_CREDIT: u32 = (openergy_net::BULK_WINDOW / (32 * 1024)) as u32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BulkTag {
    Clipboard(ClipboardId),
    File(u64),
}

#[derive(Default)]
struct BulkBacklog {
    chunks: VecDeque<(BulkTag, Message, usize)>,
    bytes: usize,
    /// Last chunk the driver accepted, or the first defer if none yet.
    progress_ms: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SendStage {
    Queued,
    Hashing,
    Active,
}
struct SendRec {
    conn: ConnId,
    peer_name: String,
    fingerprint: Option<String>,
    path: PathBuf,
    total_len: u64,
    wire: u32,
    stage: SendStage,
}
struct PendingOffer {
    conn: ConnId,
    name: String,
    total_len: u64,
    sha256: [u8; 32],
    expires_ms: u64,
    accepting: bool,
}
#[derive(Default)]
struct FileRuntime {
    receive_dir: PathBuf,
    /// False when the configured receive_dir could not be created or
    /// resolved: offers are rejected instead of landing in a fallback
    /// directory the user did not select.
    receive_dir_ok: bool,
    sends: BTreeMap<u64, SendRec>,
    send_queue: BTreeMap<ConnId, VecDeque<u64>>,
    active_send: BTreeMap<ConnId, u64>,
    pending_offers: BTreeMap<u64, PendingOffer>,
    /// Inbound transfers the session created: xfer -> (conn, total).
    recvs: BTreeMap<u64, (ConnId, u64)>,
    wire_alloc: BTreeMap<ConnId, u32>,
    progress: BTreeMap<u64, u64>,
    progress_sent_ms: BTreeMap<u64, u64>,
}

struct Runtime {
    cfg: Config,
    path: PathBuf,
    dir: PathBuf,
    platform: String,
    started: Instant,
    driver: NetDriver,
    machine: Machine,
    sink: Box<dyn InputSink>,
    screens: Box<dyn ScreenSystem>,
    control: Option<openergy_platform::virtual_backend::VirtualControl>,
    notices: adapter::Notices,
    /// Sleep and screen lock suspend independently; the session resumes
    /// only once both have cleared (see `suspended`).
    asleep: bool,
    locked: bool,
    resume_enabled: bool,
    close_at: Option<u64>,
    capture: workers::Capture,
    clipboard: workers::ClipboardWorker,
    file_worker: workers::FileWorker,
    files: FileRuntime,
    bus: Arc<EventBus>,
    logger: Arc<Logger>,
    state: String,
    connections: BTreeMap<ConnId, Connection>,
    pending: BTreeMap<ConnId, ipc::PendingTrust>,
    expected_pin: Option<(String, Fingerprint)>,
    discovery: Value,
    last_error: Option<ipc::LastError>,
    last_transfer: Option<ipc::ClipboardTransfer>,
    last_blocked: Option<ipc::ClipboardBlocked>,
    last_file_completed: Option<ipc::FileCompleted>,
    last_file_blocked: Option<ipc::FileBlocked>,
    next_xfer: u64,
    events_in: u64,
    events_out: u64,
    attempt: u64,
    retry: Option<u64>,
    healthy_since: Option<u64>,
    enabled: bool,
    status_dirty: bool,
    last_status: Value,
    last_status_ms: u64,
    last_health: u64,
    languages: openergy_platform::KeyboardLanguages,
    language_unsupported_logged: bool,
    last_network_sample: u64,
    network_set: Option<crate::interfaces::Addresses>,
    permissions: Permissions,
    local_secure: Option<String>,
    display_changed: Arc<AtomicBool>,
    preventing_sleep: bool,
    displays: Vec<ScreenRect>,
    rng: u64,
    stop: Arc<AtomicBool>,
    actions: Vec<Action>,
    net_events: Vec<NetEvent>,
    deferred_bulk: BTreeMap<ConnId, BulkBacklog>,
}
impl Runtime {
    fn now(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
    fn suspended(&self) -> bool {
        self.asleep || self.locked
    }
    fn error(&mut self, code: &str, message: impl Into<String>) {
        let message = message.into();
        self.status_dirty = true;
        log::warn!("{code}: {message}");
        self.last_error = Some(ipc::LastError {
            code: code.into(),
            message,
            unix_ms: Some(unix_ms()),
        });
    }
    fn clip_enabled(&self, on: bool) -> ClipboardCommand {
        ClipboardCommand::Enabled {
            clipboard: on,
            primary: self.cfg.clipboard.share_primary_selection,
        }
    }
    fn publish(&self, event: &str, data: Value) {
        self.bus.publish(Event {
            event: event.into(),
            data,
        });
    }
    fn bulk_has_room(&self, conn: ConnId, wire: usize) -> bool {
        // No driver connection keeps the old fire-and-forget outcome; the
        // disconnect path discards the backlog and the session transfer.
        self.driver
            .conn_info(conn)
            .is_none_or(|info| info.queued_bytes.saturating_add(wire) <= BULK_GATE_BYTES)
    }
    fn defer_bulk(&mut self, conn: ConnId, tag: BulkTag, msg: Message, wire: usize) {
        let queued = self.deferred_bulk.get(&conn).map_or(0, |backlog| backlog.bytes);
        if queued.saturating_add(wire) > BULK_DEFERRED_CAP {
            log::warn!("bulk backlog over cap for connection {}; aborting transfer", conn.0);
            self.abort_tagged(conn, tag);
            return;
        }
        let now = self.now();
        let backlog = self.deferred_bulk.entry(conn).or_insert_with(|| BulkBacklog {
            progress_ms: now,
            ..BulkBacklog::default()
        });
        backlog.bytes = backlog.bytes.saturating_add(wire);
        backlog.chunks.push_back((tag, msg, wire));
    }
    /// Drop deferred chunks for one stream without touching session
    /// state (the session already cancelled or superseded it).
    fn drop_deferred_tag(&mut self, conn: ConnId, tag: BulkTag) {
        let empty = if let Some(backlog) = self.deferred_bulk.get_mut(&conn) {
            backlog.chunks.retain(|(queued, _, _)| *queued != tag);
            backlog.bytes = backlog.chunks.iter().map(|(_, _, wire)| *wire).sum();
            backlog.chunks.is_empty()
        } else {
            false
        };
        if empty {
            self.deferred_bulk.remove(&conn);
        }
    }
    fn drop_deferred(&mut self, conn: ConnId, id: ClipboardId) {
        self.drop_deferred_tag(conn, BulkTag::Clipboard(id));
    }
    fn drop_deferred_file(&mut self, xfer: u64) {
        let conns = self.deferred_bulk.keys().copied().collect::<Vec<_>>();
        for conn in conns {
            self.drop_deferred_tag(conn, BulkTag::File(xfer));
        }
    }
    fn abort_tagged(&mut self, conn: ConnId, tag: BulkTag) {
        match tag {
            BulkTag::Clipboard(id) => self.abort_bulk(conn, id),
            BulkTag::File(xfer) => self.abort_file_bulk(conn, xfer),
        }
    }
    fn abort_file_bulk(&mut self, conn: ConnId, xfer: u64) {
        self.drop_deferred_tag(conn, BulkTag::File(xfer));
        self.driver.cancel_file(conn);
        self.error(
            "file_queue_full",
            "File transfer stopped because the network queue is full.",
        );
        match &mut self.machine {
            Machine::Server(s) => s.handle(ServerEvent::FileBulkAborted { xfer }, &mut self.actions),
            Machine::Client(c) => c.handle(ClientEvent::FileBulkAborted { xfer }, &mut self.actions),
            Machine::Off => {}
        }
    }
    fn grant_file_credit(&mut self, xfer: u64, chunks: u32) {
        match &mut self.machine {
            Machine::Server(s) => s.handle(ServerEvent::FileCredit { xfer, chunks }, &mut self.actions),
            Machine::Client(c) => c.handle(ClientEvent::FileCredit { xfer, chunks }, &mut self.actions),
            Machine::Off => {}
        }
    }
    fn peer_name_for(&self, conn: ConnId) -> String {
        self.machine
            .snapshot()
            .peers
            .into_iter()
            .find(|p| p.conn == conn)
            .map(|p| p.name)
            .unwrap_or_else(|| "server".into())
    }
    fn next_rand_u32(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 16) as u32
    }
    fn alloc_wire(&mut self, conn: ConnId) -> u32 {
        if !self.files.wire_alloc.contains_key(&conn) {
            // Random start per connection (design §4.1), skipping zero.
            let seed = self.next_rand_u32() ^ (unix_ms() as u32);
            self.files.wire_alloc.insert(conn, if seed == 0 { 1 } else { seed });
        }
        let entry = self.files.wire_alloc.get_mut(&conn).map_or(1, |v| *v);
        let wire = if entry == 0 { 1 } else { entry };
        let next = wire.wrapping_add(1);
        self.files.wire_alloc.insert(conn, if next == 0 { 1 } else { next });
        wire
    }
    fn worker_file(&mut self, cmd: FileCommand) -> bool {
        self.file_worker.commands.try_send(cmd).is_ok()
    }
    /// Start the next queued send for one connection, if idle.
    fn start_next_send(&mut self, conn: ConnId) {
        if self.files.active_send.contains_key(&conn) {
            return;
        }
        loop {
            let next = self.files.send_queue.get_mut(&conn).and_then(|queue| queue.pop_front());
            let Some(xfer) = next else { return };
            let Some(rec) = self.files.sends.get_mut(&xfer) else {
                continue;
            };
            rec.stage = SendStage::Hashing;
            let path = rec.path.clone();
            self.files.active_send.insert(conn, xfer);
            if !self.worker_file(FileCommand::HashFile { xfer, path }) {
                self.fail_send_local(xfer, "io_error");
            }
            return;
        }
    }
    /// Terminal cleanup for one outbound transfer.
    fn finish_send(&mut self, xfer: u64) {
        let Some(rec) = self.files.sends.remove(&xfer) else {
            return;
        };
        self.files.progress.remove(&xfer);
        self.files.progress_sent_ms.remove(&xfer);
        self.drop_deferred_file(xfer);
        let _ = self.worker_file(FileCommand::DropSend { xfer });
        if let Some(queue) = self.files.send_queue.get_mut(&rec.conn) {
            queue.retain(|queued| *queued != xfer);
        }
        if self.files.active_send.get(&rec.conn) == Some(&xfer) {
            self.files.active_send.remove(&rec.conn);
            self.start_next_send(rec.conn);
        }
    }
    fn fail_send_local(&mut self, xfer: u64, reason: &str) {
        let peer = self
            .files
            .sends
            .get(&xfer)
            .map(|rec| rec.peer_name.clone())
            .unwrap_or_default();
        self.publish(
            "file.failed",
            json!({"xfer":xfer,"peer":peer,"direction":"out","reason":reason,"unix_ms":unix_ms()}),
        );
        self.finish_send(xfer);
    }
    fn decide_offer(&mut self, xfer: u64, accept: bool, reason: FileReason) {
        match &mut self.machine {
            Machine::Server(s) => s.handle(
                ServerEvent::FileOfferDecided {
                    xfer,
                    accept,
                    reason,
                    resume_offset: 0,
                },
                &mut self.actions,
            ),
            Machine::Client(c) => c.handle(
                ClientEvent::FileOfferDecided {
                    xfer,
                    accept,
                    reason,
                    resume_offset: 0,
                },
                &mut self.actions,
            ),
            Machine::Off => {}
        }
    }
    fn cancel_session_file(&mut self, xfer: u64) {
        match &mut self.machine {
            Machine::Server(s) => s.handle(ServerEvent::FileCancelRequested { xfer }, &mut self.actions),
            Machine::Client(c) => c.handle(ClientEvent::FileCancelRequested { xfer }, &mut self.actions),
            Machine::Off => {}
        }
    }
    fn begin_receive(&mut self, xfer: u64) {
        let Some(offer) = self.files.pending_offers.get_mut(&xfer) else {
            return;
        };
        // A duplicate accept while staging initializes must not enqueue
        // a second BeginReceive: the worker would answer Busy for it,
        // and handling that result would drop the live transfer record.
        if offer.accepting {
            return;
        }
        offer.accepting = true;
        let cmd = FileCommand::BeginReceive {
            xfer,
            name: offer.name.clone(),
            total_len: offer.total_len,
            sha256: offer.sha256,
            on_conflict_reject: self.cfg.file.on_conflict == FileConflict::Reject,
        };
        if !self.worker_file(cmd) {
            self.files.pending_offers.remove(&xfer);
            self.files.recvs.remove(&xfer);
            self.decide_offer(xfer, false, FileReason::IoError);
        }
    }
    fn handle_offered(&mut self, xfer: u64, conn: ConnId, kind: u8, name: String, total_len: u64, sha256: [u8; 32]) {
        if !self.files.receive_dir_ok {
            self.decide_offer(xfer, false, FileReason::IoError);
            return;
        }
        let peer_name = self.peer_name_for(conn);
        self.files.recvs.insert(xfer, (conn, total_len));
        let expires_ms = self.now().saturating_add(self.cfg.file.offer_timeout());
        self.files.pending_offers.insert(
            xfer,
            PendingOffer {
                conn,
                name: name.clone(),
                total_len,
                sha256,
                expires_ms,
                accepting: false,
            },
        );
        if self.cfg.file.receive_mode == FileReceiveMode::Auto {
            self.begin_receive(xfer);
        } else {
            self.publish(
                "file.offered",
                json!({
                    "xfer": xfer,
                    "peer": peer_name,
                    "name": name,
                    "bytes": total_len,
                    "kind": kind,
                    "expires_unix_ms": unix_ms().saturating_add(self.cfg.file.offer_timeout()),
                }),
            );
        }
    }
    fn expire_offers(&mut self) {
        let now = self.now();
        let expired = self
            .files
            .pending_offers
            .iter()
            .filter(|(_, offer)| !offer.accepting && now >= offer.expires_ms)
            .map(|(xfer, _)| *xfer)
            .collect::<Vec<_>>();
        for xfer in expired {
            self.files.pending_offers.remove(&xfer);
            self.files.recvs.remove(&xfer);
            self.decide_offer(xfer, false, FileReason::Timeout);
        }
    }
    fn handle_file_event(&mut self, event: FileEvent) {
        match event {
            FileEvent::Hashed {
                xfer,
                name,
                total_len,
                sha256,
            } => {
                let Some(rec) = self.files.sends.get(&xfer) else { return };
                let (conn, peer_name, fingerprint) = (rec.conn, rec.peer_name.clone(), rec.fingerprint.clone());
                let wire = self.alloc_wire(conn);
                if let Some(rec) = self.files.sends.get_mut(&xfer) {
                    rec.wire = wire;
                    rec.total_len = total_len;
                    rec.stage = SendStage::Active;
                }
                match &mut self.machine {
                    Machine::Server(s) => s.handle(
                        ServerEvent::FileSendRequested {
                            xfer,
                            conn,
                            transfer: wire,
                            peer_name,
                            fingerprint,
                            kind: 0,
                            name,
                            total_len,
                            sha256,
                        },
                        &mut self.actions,
                    ),
                    Machine::Client(c) => c.handle(
                        ClientEvent::FileSendRequested {
                            xfer,
                            conn,
                            transfer: wire,
                            peer_name,
                            fingerprint,
                            kind: 0,
                            name,
                            total_len,
                            sha256,
                        },
                        &mut self.actions,
                    ),
                    Machine::Off => {}
                }
                self.grant_file_credit(xfer, FILE_INITIAL_CREDIT);
            }
            FileEvent::HashFailed { xfer, reason } => {
                if self.files.sends.contains_key(&xfer) {
                    self.fail_send_local(xfer, reason);
                }
            }
            FileEvent::ChunkReady { xfer, bytes } => match &mut self.machine {
                Machine::Server(s) => s.handle(ServerEvent::FileChunkReady { xfer, bytes }, &mut self.actions),
                Machine::Client(c) => c.handle(ClientEvent::FileChunkReady { xfer, bytes }, &mut self.actions),
                Machine::Off => {}
            },
            FileEvent::ChunkFailed { xfer } => match &mut self.machine {
                Machine::Server(s) => s.handle(
                    ServerEvent::FileChunkFailed {
                        xfer,
                        reason: FileReason::IoError,
                    },
                    &mut self.actions,
                ),
                Machine::Client(c) => c.handle(
                    ClientEvent::FileChunkFailed {
                        xfer,
                        reason: FileReason::IoError,
                    },
                    &mut self.actions,
                ),
                Machine::Off => {}
            },
            FileEvent::ReceiveReady { xfer, ok, reason } => {
                self.files.pending_offers.remove(&xfer);
                if !ok {
                    self.files.recvs.remove(&xfer);
                }
                self.decide_offer(xfer, ok, reason);
            }
            FileEvent::StageResult { xfer, ok, reason } => match &mut self.machine {
                Machine::Server(s) => s.handle(ServerEvent::FileStageResult { xfer, ok, reason }, &mut self.actions),
                Machine::Client(c) => c.handle(ClientEvent::FileStageResult { xfer, ok, reason }, &mut self.actions),
                Machine::Off => {}
            },
            FileEvent::FinishResult { xfer, ok, reason } => match &mut self.machine {
                Machine::Server(s) => s.handle(ServerEvent::FileFinishResult { xfer, ok, reason }, &mut self.actions),
                Machine::Client(c) => c.handle(ClientEvent::FileFinishResult { xfer, ok, reason }, &mut self.actions),
                Machine::Off => {}
            },
        }
    }
    /// Drop every daemon-side file record. Used when the session
    /// machine is recreated or the connection stack resets, where
    /// session-side terminal notices may never arrive.
    fn reset_file_state(&mut self, reason: &str) {
        let sends = self.files.sends.keys().copied().collect::<Vec<_>>();
        for xfer in sends {
            self.fail_send_local(xfer, reason);
        }
        let recvs = self.files.recvs.keys().copied().collect::<Vec<_>>();
        for xfer in recvs {
            let _ = self.worker_file(FileCommand::DropReceive { xfer, keep_part: false });
        }
        self.files.sends.clear();
        self.files.send_queue.clear();
        self.files.active_send.clear();
        self.files.pending_offers.clear();
        self.files.recvs.clear();
        self.files.progress.clear();
        self.files.progress_sent_ms.clear();
        let conns = self.deferred_bulk.keys().copied().collect::<Vec<_>>();
        for conn in conns {
            if let Some(backlog) = self.deferred_bulk.get_mut(&conn) {
                backlog
                    .chunks
                    .retain(|(tag, _, _)| matches!(tag, BulkTag::Clipboard(_)));
                backlog.bytes = backlog.chunks.iter().map(|(_, _, wire)| *wire).sum();
            }
        }
        self.deferred_bulk.retain(|_, backlog| !backlog.chunks.is_empty());
    }
    fn save_xfer_counter(&mut self) {
        self.next_xfer = self.machine.next_xfer();
    }
    fn seed_xfer_counter(&mut self) {
        let next = self.next_xfer;
        self.machine.set_next_xfer(next);
    }
    fn abort_bulk(&mut self, conn: ConnId, id: ClipboardId) {
        self.drop_deferred(conn, id);
        match &mut self.machine {
            Machine::Server(s) => s.abort_clipboard(conn, id),
            Machine::Client(c) => c.abort_clipboard(conn, id),
            Machine::Off => {}
        }
        self.driver.cancel_clipboard(conn);
        let blocked = ipc::ClipboardBlocked {
            bytes: None,
            limit: None,
            peer: self.connections.get(&conn).map(|c| c.address.clone()),
            unix_ms: Some(unix_ms()),
            selection: Some(selection_name(id).into()),
        };
        self.publish(
            "clipboard.blocked",
            serde_json::to_value(&blocked).unwrap_or(Value::Null),
        );
        self.last_blocked = Some(blocked);
        self.error(
            "clipboard_queue_full",
            "Clipboard transfer stopped because the network queue is full.",
        );
    }
    fn fail_bulk(&mut self, conn: ConnId, id: ClipboardId, error: openergy_net::NetError) {
        self.error("network_failed", format!("Can't send data: {error}."));
        self.abort_bulk(conn, id);
    }
    fn fail_tagged(&mut self, conn: ConnId, tag: BulkTag, error: openergy_net::NetError) {
        self.error("network_failed", format!("Can't send data: {error}."));
        self.abort_tagged(conn, tag);
    }
    fn send_clipboard(&mut self, conn: ConnId, id: ClipboardId, sequence: u32, chunk: Chunk) {
        // A Start supersedes any transfer the session cancelled for this
        // clipboard (newer grab/local change): drop its unsent chunks so
        // megabytes of obsolete data cannot drain ahead of the fresh one.
        if matches!(chunk, Chunk::Start { .. }) {
            self.drop_deferred(conn, id);
        }
        let wire = openergy_session::clipboard::chunk_wire_bytes(&chunk);
        let msg = Message::ClipboardData { id, sequence, chunk };
        // Chunk order is load-bearing: behind a backlog nothing overtakes.
        if self.deferred_bulk.contains_key(&conn) || !self.bulk_has_room(conn, wire) {
            self.defer_bulk(conn, BulkTag::Clipboard(id), msg, wire);
            return;
        }
        let retry = msg.clone();
        match self.driver.execute_checked(
            NetCommand::Send {
                conn,
                msg,
                priority: Priority::Bulk,
            },
            &mut self.net_events,
        ) {
            Ok(()) => {}
            // WouldBlock, not failure: the head chunk stays queued and the
            // next loop tick retries it once the queue drains.
            Err(openergy_net::NetError::QueueFull) => self.defer_bulk(conn, BulkTag::Clipboard(id), retry, wire),
            Err(error) => self.fail_bulk(conn, id, error),
        }
    }
    fn send_file_chunk(&mut self, conn: ConnId, transfer: u32, chunk: FileDataChunk) {
        let Some(&xfer) = self
            .files
            .active_send
            .get(&conn)
            .filter(|xfer| self.files.sends.get(xfer).is_some_and(|rec| rec.wire == transfer))
        else {
            log::warn!("file chunk for unknown active send dropped");
            return;
        };
        let is_data = matches!(chunk, FileDataChunk::Data(_));
        let wire = sfile::file_chunk_wire_bytes(&chunk);
        let msg = Message::FileData { transfer, chunk };
        if self.deferred_bulk.contains_key(&conn) || !self.bulk_has_room(conn, wire) {
            self.defer_bulk(conn, BulkTag::File(xfer), msg, wire);
            return;
        }
        let retry = msg.clone();
        match self.driver.execute_checked(
            NetCommand::Send {
                conn,
                msg,
                priority: Priority::Bulk,
            },
            &mut self.net_events,
        ) {
            Ok(()) => {
                if is_data {
                    self.grant_file_credit(xfer, 1);
                }
            }
            Err(openergy_net::NetError::QueueFull) => self.defer_bulk(conn, BulkTag::File(xfer), retry, wire),
            Err(error) => self.fail_tagged(conn, BulkTag::File(xfer), error),
        }
    }
    fn drain_bulk(&mut self) {
        if self.deferred_bulk.is_empty() {
            return;
        }
        let now = self.now();
        let heartbeat = if self.cfg.role == Role::Server {
            self.cfg.server.heartbeat_ms
        } else {
            self.cfg.client.heartbeat_ms
        };
        let file_stall_ms = sfile::inbound_expiry_ms(heartbeat);
        let stalled = self
            .deferred_bulk
            .iter()
            .filter_map(|(conn, backlog)| {
                let (tag, _, _) = backlog.chunks.front()?;
                let limit = match tag {
                    BulkTag::Clipboard(_) => BULK_STALL_MS,
                    BulkTag::File(_) => file_stall_ms,
                };
                (now.saturating_sub(backlog.progress_ms) >= limit).then_some((*conn, *tag))
            })
            .collect::<Vec<_>>();
        for (conn, tag) in stalled {
            log::warn!("bulk queue stalled for connection {}; aborting transfer", conn.0);
            self.abort_tagged(conn, tag);
        }
        let conns = self.deferred_bulk.keys().copied().collect::<Vec<_>>();
        for conn in conns {
            while let Some((tag, msg, wire)) = self
                .deferred_bulk
                .get(&conn)
                .and_then(|backlog| backlog.chunks.front().cloned())
            {
                if !self.bulk_has_room(conn, wire) {
                    break;
                }
                let file_data_xfer = match (&tag, &msg) {
                    (
                        BulkTag::File(xfer),
                        Message::FileData {
                            chunk: FileDataChunk::Data(_),
                            ..
                        },
                    ) => Some(*xfer),
                    _ => None,
                };
                match self.driver.execute_checked(
                    NetCommand::Send {
                        conn,
                        msg,
                        priority: Priority::Bulk,
                    },
                    &mut self.net_events,
                ) {
                    Ok(()) => {
                        let empty = if let Some(backlog) = self.deferred_bulk.get_mut(&conn) {
                            backlog.chunks.pop_front();
                            backlog.bytes = backlog.bytes.saturating_sub(wire);
                            backlog.progress_ms = now;
                            backlog.chunks.is_empty()
                        } else {
                            false
                        };
                        if empty {
                            self.deferred_bulk.remove(&conn);
                        }
                        // Admission returns one chunk of file credit.
                        if let Some(xfer) = file_data_xfer {
                            self.grant_file_credit(xfer, 1);
                        }
                    }
                    Err(openergy_net::NetError::QueueFull) => break,
                    Err(error) => {
                        self.fail_tagged(conn, tag, error);
                        break;
                    }
                }
            }
        }
    }
    fn execute(&mut self, cmd: NetCommand) {
        let send = match &cmd {
            NetCommand::Send { conn, msg, priority } => Some((
                *conn,
                matches!(msg, Message::KeyUp { .. } | Message::MouseUp { .. }),
                (*priority == Priority::Input).then(|| msg.clone()),
            )),
            _ => None,
        };
        if let Err(error) = self.driver.execute_checked(cmd, &mut self.net_events) {
            let code = if error == openergy_net::NetError::QueueFull {
                "network_queue_full"
            } else {
                "network_failed"
            };
            self.error(code, format!("Can't send data: {error}."));
            if let Some((conn, lost_up, input)) = send {
                if lost_up {
                    // Retry the lost release in the reserved control path, then leave
                    // only this peer. Other peers and their ledgers stay intact.
                    if let Some(msg) = input {
                        self.driver.execute(
                            NetCommand::Send {
                                conn,
                                msg,
                                priority: Priority::Control,
                            },
                            &mut self.net_events,
                        );
                    }
                    match &mut self.machine {
                        Machine::Server(s) => s.handle(
                            ServerEvent::ClientDisconnected {
                                conn,
                                reason: openergy_session::DisconnectReason::SlowPeer,
                            },
                            &mut self.actions,
                        ),
                        Machine::Client(c) => c.handle(
                            ClientEvent::ServerDisconnected {
                                conn,
                                reason: openergy_session::DisconnectReason::SlowPeer,
                            },
                            &mut self.actions,
                        ),
                        Machine::Off => {}
                    }
                    self.actions.push(Action::Close {
                        conn,
                        reason: openergy_session::CloseReason::SlowPeer,
                    });
                }
            }
        }
    }
    fn status(&self) -> Status {
        let mut s = Status::idle(VERSION, std::process::id(), &self.platform);
        s.daemon.uptime_s = self.started.elapsed().as_secs();
        s.role = self.cfg.role.as_str().into();
        s.state = self.state.clone();
        s.name = Some(self.cfg.name.clone());
        let snapshot = self.machine.snapshot();
        s.active_screen = snapshot.active_screen;
        s.cursor.screen = s.active_screen.clone();
        if let Some((x, y)) = snapshot.cursor {
            s.cursor.x = Some(x);
            s.cursor.y = Some(y);
        }
        let rect = display_rect(&self.displays);
        s.local_screen.w = Some(rect.w);
        s.local_screen.h = Some(rect.h);
        s.local_screen.displays = self
            .displays
            .iter()
            .map(|r| ipc::DisplayInfo {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                scale: r.scale,
            })
            .collect();
        s.server.listen = (self.cfg.role == Role::Server).then(|| self.cfg.server.listen.clone());
        s.client.remote = (!self.cfg.client.remote.is_empty()).then(|| self.cfg.client.remote.clone());
        s.client.attempt = Some(self.attempt);
        s.client.next_retry_ms = self.retry.map(|t| t.saturating_sub(self.now()));
        s.peers = snapshot
            .peers
            .iter()
            .map(|p| {
                let conn = self.connections.get(&p.conn);
                ipc::PeerInfo {
                    name: p.name.clone(),
                    address: conn.map(|c| c.address.clone()),
                    role: Some(
                        if self.cfg.role == Role::Server {
                            "client"
                        } else {
                            "server"
                        }
                        .into(),
                    ),
                    connected: p.connected,
                    since_unix_ms: conn.map(|c| c.since),
                    rtt_ms: p.rtt_ms,
                    protocol: ipc::PeerProtocol {
                        product: Some(p.product.clone()),
                        major: Some(p.version.major),
                        minor: Some(p.version.minor),
                    },
                    screen: ipc::PeerScreen {
                        w: (self.cfg.role == Role::Server).then_some(p.screen.w),
                        h: (self.cfg.role == Role::Server).then_some(p.screen.h),
                    },
                    tls: ipc::PeerTls {
                        enabled: conn.is_some_and(|c| c.tls),
                        version: conn.and_then(|c| c.tls_version.clone()),
                        fingerprint: conn.and_then(|c| c.fingerprint.clone()),
                        trusted: conn.filter(|c| c.tls).map(|c| c.certificate_verified),
                    },
                    secure_input_app: p.secure_input_app.clone(),
                    languages: p.languages.clone(),
                    active_language: p.active_language.clone(),
                    file: ipc::PeerFile {
                        capability: p.file_capability.clone(),
                        max_bytes: p.file_max_bytes,
                    },
                }
            })
            .collect();
        s.tls.enabled = self.cfg.tls.enabled;
        s.tls.fingerprint = self.driver.identity_fingerprint().map(|fp| fp.to_colon_hex());
        let p = serde_json::to_value(&self.permissions).unwrap_or(Value::Null);
        s.permissions.accessibility = p["accessibility"].as_str().unwrap_or("unknown").into();
        s.permissions.input_monitoring = p["input_monitoring"].as_str().unwrap_or("unknown").into();
        s.permissions.post_events = p["post_events"].as_str().unwrap_or("unknown").into();
        s.secure_input.local_app = self.local_secure.clone();
        s.clipboard.enabled = self.cfg.clipboard.enabled;
        // PRIMARY exists on the X11 backend and on the virtual backend's
        // second slot; macOS/Windows have one system clipboard only.
        s.clipboard.primary_enabled = self.cfg.clipboard.enabled
            && self.cfg.clipboard.share_primary_selection
            && matches!(self.platform.as_str(), "x11" | "virtual");
        s.clipboard.last_transfer = self.last_transfer.clone();
        s.clipboard.last_blocked = self.last_blocked.clone();
        s.file.enabled = self.cfg.file.enabled;
        s.file.receive_dir = self.files.receive_dir.to_string_lossy().into_owned();
        s.file.last_completed = self.last_file_completed.clone();
        s.file.last_blocked = self.last_file_blocked.clone();
        for (&xfer, rec) in &self.files.sends {
            s.file.active.push(ipc::FileActive {
                xfer,
                peer: rec.peer_name.clone(),
                direction: "out".into(),
                kind: 0,
                bytes_done: self.files.progress.get(&xfer).copied().unwrap_or(0),
                bytes_total: rec.total_len,
                state: match rec.stage {
                    SendStage::Queued => "queued",
                    SendStage::Hashing => "hashing",
                    SendStage::Active => "sending",
                }
                .into(),
            });
        }
        for (&xfer, (conn, total)) in &self.files.recvs {
            s.file.active.push(ipc::FileActive {
                xfer,
                peer: self.peer_name_for(*conn),
                direction: "in".into(),
                kind: 0,
                bytes_done: self.files.progress.get(&xfer).copied().unwrap_or(0),
                bytes_total: *total,
                state: if self.files.pending_offers.contains_key(&xfer) {
                    "offered".into()
                } else {
                    "receiving".into()
                },
            });
        }
        s.file.active.sort_by_key(|a| a.xfer);
        s.pending_trust = self.pending.values().cloned().collect();
        s.last_error = self.last_error.clone();
        s.stats.rss_bytes = lifetime::rss_bytes();
        s.stats.events_in = self.events_in;
        s.stats.events_out = self.events_out;
        s
    }
    fn apply_actions(&mut self) {
        self.status_dirty |= !self.actions.is_empty();
        let mut count = 0;
        while !self.actions.is_empty() && count < 1024 {
            let mut actions = std::mem::take(&mut self.actions);
            count += actions.len();
            for action in actions.drain(..) {
                match action {
                    Action::SendControl { conn, msg } => {
                        self.events_out += 1;
                        self.execute(NetCommand::Send {
                            conn,
                            msg,
                            priority: Priority::Control,
                        });
                    }
                    Action::Send { conn, msg } => {
                        let priority = if matches!(
                            msg,
                            Message::KeyDown { .. }
                                | Message::KeyUp { .. }
                                | Message::KeyRepeat { .. }
                                | Message::MouseDown { .. }
                                | Message::MouseUp { .. }
                                | Message::MouseMoveAbs { .. }
                                | Message::MouseMoveRel { .. }
                                | Message::MouseWheel { .. }
                                | Message::Enter { .. }
                                | Message::Leave
                        ) {
                            Priority::Input
                        } else {
                            Priority::Control
                        };
                        self.events_out += 1;
                        self.execute(NetCommand::Send { conn, msg, priority });
                    }
                    Action::SendClipboardChunk {
                        conn,
                        id,
                        sequence,
                        chunk,
                    } => self.send_clipboard(conn, id, sequence, chunk),
                    Action::SendFileChunk { conn, transfer, chunk } => self.send_file_chunk(conn, transfer, chunk),
                    Action::ReadFileChunk { xfer, offset, len } => {
                        if !self.worker_file(FileCommand::ReadChunk { xfer, offset, len }) {
                            match &mut self.machine {
                                Machine::Server(s) => s.handle(
                                    ServerEvent::FileChunkFailed {
                                        xfer,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Client(c) => c.handle(
                                    ClientEvent::FileChunkFailed {
                                        xfer,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Off => {}
                            }
                        }
                    }
                    Action::StageFileChunk { xfer, offset, bytes } => {
                        if !self.worker_file(FileCommand::Stage { xfer, offset, bytes }) {
                            match &mut self.machine {
                                Machine::Server(s) => s.handle(
                                    ServerEvent::FileStageResult {
                                        xfer,
                                        ok: false,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Client(c) => c.handle(
                                    ClientEvent::FileStageResult {
                                        xfer,
                                        ok: false,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Off => {}
                            }
                        }
                    }
                    Action::FinishFile { xfer } => {
                        if !self.worker_file(FileCommand::Finish { xfer }) {
                            match &mut self.machine {
                                Machine::Server(s) => s.handle(
                                    ServerEvent::FileFinishResult {
                                        xfer,
                                        ok: false,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Client(c) => c.handle(
                                    ClientEvent::FileFinishResult {
                                        xfer,
                                        ok: false,
                                        reason: FileReason::IoError,
                                    },
                                    &mut self.actions,
                                ),
                                Machine::Off => {}
                            }
                        }
                    }
                    Action::DropFile { xfer, keep_part } => {
                        let _ = self.worker_file(FileCommand::DropReceive { xfer, keep_part });
                    }
                    Action::PurgeFileBulk { conn, xfer } => {
                        self.drop_deferred_tag(conn, BulkTag::File(xfer));
                        self.driver.cancel_file(conn);
                    }
                    Action::FileOffered {
                        xfer,
                        conn,
                        kind,
                        name,
                        total_len,
                        sha256,
                    } => self.handle_offered(xfer, conn, kind, name, total_len, sha256),
                    Action::Close { conn, .. } => self.execute(NetCommand::Close { conn }),
                    Action::ReplaceConnection { .. } => {}
                    Action::SetKeyboardLanguage(code) => {
                        let lang = String::from_utf8_lossy(&code);
                        let result = match self.screens.set_keyboard_language(&lang) {
                            Ok(()) => "applied",
                            Err(openergy_platform::PlatformError::NotImplemented) => {
                                if !self.language_unsupported_logged {
                                    self.language_unsupported_logged = true;
                                    log::info!("keyboard language sync unsupported");
                                }
                                "unsupported"
                            }
                            Err(e) => {
                                self.error("language_sync_failed", e.to_string());
                                "unsupported"
                            }
                        };
                        self.publish(
                            ipc::events::LANGUAGE_SYNC,
                            json!({"language":lang,"result":result,"attempted":self.platform == "windows"}),
                        );
                    }
                    Action::Inject(ev) => {
                        if let Err(e) = self.sink.inject(&ev) {
                            self.error("injection_failed", e.to_string());
                        }
                    }
                    Action::ReleaseAllLocal => self.sink.release_all(),
                    Action::HideLocalCursor(hidden) => {
                        self.sink.set_cursor_visible(!hidden);
                        self.capture.remote.store(hidden, Ordering::Relaxed);
                        let _ = self.capture.commands.try_send(CaptureCommand::Captured(hidden));
                    }
                    Action::WarpLocalCursor { x, y } => {
                        let _ = self.capture.commands.try_send(CaptureCommand::Warp(x, y));
                    }
                    Action::ReadClipboard { id, max_bytes } => {
                        if self
                            .clipboard
                            .commands
                            .try_send(ClipboardCommand::Read { id, max: max_bytes })
                            .is_err()
                        {
                            self.machine.clipboard_read_failed(id, &mut self.actions);
                            self.error("clipboard_busy", "Clipboard worker queue is full");
                        }
                    }
                    Action::SetClipboardShared { id, data } => {
                        if self
                            .clipboard
                            .commands
                            .try_send(ClipboardCommand::Write { id, data })
                            .is_err()
                        {
                            self.error("clipboard_busy", "Clipboard worker queue is full");
                        }
                    }
                    Action::SetClipboard { id, data } => {
                        if self
                            .clipboard
                            .commands
                            .try_send(ClipboardCommand::Write {
                                id,
                                data: Arc::new(data),
                            })
                            .is_err()
                        {
                            self.error("clipboard_busy", "Clipboard worker queue is full");
                        }
                    }
                    Action::Notify(mut event) => {
                        if let Some(object) = event.data.as_object_mut() {
                            if object.get("unix_ms").is_some_and(Value::is_null) {
                                object.insert("unix_ms".into(), json!(unix_ms()));
                            }
                        }
                        let mut publish_event = true;
                        match event.event.as_str() {
                            "clipboard.blocked" => {
                                self.last_blocked = serde_json::from_value(event.data.clone()).ok();
                            }
                            "clipboard.transfer" => {
                                self.last_transfer = serde_json::from_value(event.data.clone()).ok();
                            }
                            "file.progress" => {
                                // Coalesce to at most 4 per second per
                                // transfer; status keeps every update.
                                if let Some(xfer) = event.data.get("xfer").and_then(Value::as_u64) {
                                    if let Some(done) = event.data.get("bytes_done").and_then(Value::as_u64) {
                                        self.files.progress.insert(xfer, done);
                                    }
                                    let now = self.now();
                                    let last = self.files.progress_sent_ms.get(&xfer).copied().unwrap_or(0);
                                    if now.saturating_sub(last) >= 250 || last == 0 {
                                        self.files.progress_sent_ms.insert(xfer, now);
                                    } else {
                                        publish_event = false;
                                    }
                                }
                            }
                            "file.completed" => {
                                self.last_file_completed = serde_json::from_value(event.data.clone()).ok();
                                if let Some(xfer) = event.data.get("xfer").and_then(Value::as_u64) {
                                    self.files.recvs.remove(&xfer);
                                    self.files.progress.remove(&xfer);
                                    self.files.progress_sent_ms.remove(&xfer);
                                    if self.files.sends.contains_key(&xfer) {
                                        self.finish_send(xfer);
                                    }
                                }
                            }
                            "file.failed" => {
                                if let Some(xfer) = event.data.get("xfer").and_then(Value::as_u64) {
                                    self.files.recvs.remove(&xfer);
                                    self.files.pending_offers.remove(&xfer);
                                    self.files.progress.remove(&xfer);
                                    self.files.progress_sent_ms.remove(&xfer);
                                    if self.files.sends.contains_key(&xfer) {
                                        self.finish_send(xfer);
                                    }
                                }
                            }
                            "file.blocked" => {
                                self.last_file_blocked = serde_json::from_value(event.data.clone()).ok();
                            }
                            "error" => {
                                let mut error: Option<ipc::LastError> = serde_json::from_value(event.data.clone()).ok();
                                if let Some(error) = &mut error {
                                    let server = remote(&self.cfg.client.remote, None)
                                        .map(|(host, _)| host)
                                        .unwrap_or_else(|_| "The server".into());
                                    error.message = match error.code.as_str() {
                                        "busy" => format!(
                                            "{server} says a computer named {} is already connected; retrying.",
                                            self.cfg.name
                                        ),
                                        "unknown_screen" => {
                                            format!("{server} has no screen named {} in its layout.", self.cfg.name)
                                        }
                                        "protocol_error"
                                            if error.message == "The server rejected the connection protocol." =>
                                        {
                                            format!("{server} rejected the connection protocol.")
                                        }
                                        _ => error.message.clone(),
                                    };
                                }
                                self.last_error = error;
                            }
                            _ => {}
                        }
                        if publish_event {
                            self.bus.publish(event);
                        }
                    }
                }
            }
            if self.actions.is_empty() {
                self.actions = actions;
            }
        }
        self.update_sleep_assertion();
    }
    fn update_sleep_assertion(&mut self) {
        let on = self.enabled && self.cfg.input.prevent_sleep && self.machine.remote_active();
        if on != self.preventing_sleep {
            self.preventing_sleep = on;
            self.screens.prevent_sleep(on);
        }
    }
    fn input(&mut self, event: InputEvent) {
        if !self.enabled || self.suspended() {
            return;
        }
        self.events_in += 1;
        if let Machine::Server(s) = &mut self.machine {
            s.handle(ServerEvent::LocalInput(event), &mut self.actions);
        }
    }
    fn notice(&mut self, notice: adapter::Notice) {
        use adapter::Notice;
        let is_sleep = matches!(notice, Notice::Sleep);
        let is_wake = matches!(notice, Notice::Wake);
        match notice {
            Notice::Languages => self.refresh_health(),
            Notice::Sleep | Notice::Lock => {
                let was_suspended = self.suspended();
                if is_sleep {
                    self.asleep = true;
                } else {
                    self.locked = true;
                }
                // Side effects fire only on the combined unsuspended ->
                // suspended transition, not when the second flag sets.
                if was_suspended {
                    return;
                }
                self.resume_enabled = self.enabled;
                self.enabled = false;
                self.retry = None;
                self.deferred_bulk.clear();
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::StopListening);
                self.close_at = Some(self.now() + 250);
                self.state = "idle".into();
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
                let _ = self.clipboard.commands.try_send(self.clip_enabled(false));
            }
            Notice::Wake | Notice::Unlock => {
                let was_suspended = self.suspended();
                if is_wake {
                    self.asleep = false;
                } else {
                    self.locked = false;
                }
                // Resume only when both flags have cleared; a Wake while
                // still locked (or an Unlock while still asleep) resumes
                // nothing.
                if was_suspended && !self.suspended() {
                    self.displays = self.screens.screens();
                    self.permissions = self.screens.permissions();
                    self.sink.permissions_changed(&self.permissions);
                    self.local_secure = self.screens.secure_input_app();
                    self.enabled = self.resume_enabled;
                    self.close_at = None;
                    self.execute(NetCommand::Shutdown);
                    self.process_net();
                    self.attempt = 0;
                    self.retry = None;
                    self.save_xfer_counter();
                    self.reset_file_state("io_error");
                    self.machine = Machine::new(Arc::new(self.cfg.clone()));
                    self.seed_xfer_counter();
                    self.machine
                        .geometry(&self.displays, self.screens.cursor_pos(), false, &mut self.actions);
                    self.start_connection();
                    let _ = self
                        .capture
                        .commands
                        .try_send(CaptureCommand::Enable(self.enabled && self.cfg.role == Role::Server));
                    let _ = self
                        .clipboard
                        .commands
                        .try_send(self.clip_enabled(self.enabled && self.cfg.clipboard.enabled));
                } else if !was_suspended && self.cfg.role == Role::Client && self.retry.is_some() {
                    self.attempt = 0;
                    self.retry = Some(self.now());
                }
            }
            Notice::Displays => self.display_changed.store(true, Ordering::Release),
            Notice::Permission | Notice::CaptureStopped | Notice::CaptureError => {
                if self.suspended() || !self.enabled || self.cfg.role != Role::Server {
                    return;
                }
                if let Machine::Server(s) = &mut self.machine {
                    s.handle(ServerEvent::CaptureOverflow, &mut self.actions);
                }
                self.error("capture_unavailable", "Platform capture stopped or lost permission");
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
                self.capture.retry.store(true, Ordering::Release);
                self.refresh_health();
            }
            Notice::NotificationError => {
                match &mut self.machine {
                    Machine::Server(s) => s.handle(ServerEvent::CaptureOverflow, &mut self.actions),
                    Machine::Client(c) => c.handle(ClientEvent::PermissionLost, &mut self.actions),
                    _ => {}
                }
                self.error(
                    "platform_notifications",
                    "Platform lifecycle notifications are unavailable",
                );
            }
        }
    }
    fn reconnect(&mut self) {
        if !self.enabled || self.cfg.role != Role::Client {
            return;
        }
        self.attempt = self.attempt.saturating_add(1);
        // A host that reboots or wakes should be back within seconds. A LAN
        // connect attempt is cheap, so the cap stays short (Synergy retries
        // every second).
        let base = 250u64
            .saturating_mul(1u64 << self.attempt.saturating_sub(1).min(7))
            .min(5_000);
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        let delay = base * (80 + self.rng % 41) / 100;
        self.retry = Some(self.now() + delay);
        self.state = "reconnecting".into();
        let reason = if self.last_error.as_ref().is_some_and(|e| e.code == "busy") {
            "EBSY"
        } else {
            "connection loss"
        };
        log::warn!("{reason}: reconnect attempt={} delay_ms={delay}", self.attempt);
    }
    fn start_connection(&mut self) {
        self.languages = self.screens.keyboard_languages();
        self.machine.languages(self.languages.clone(), &mut self.actions);
        // Connecting is idempotent, including a TLS connection waiting for approval.
        if (self.cfg.role == Role::Client
            && (matches!(self.state.as_str(), "connecting" | "connected")
                || (self.state == "reconnecting" && self.retry.is_none())
                || !self.pending.is_empty()))
            || (self.cfg.role == Role::Server && matches!(self.state.as_str(), "starting" | "listening" | "connected"))
        {
            return;
        }
        self.retry = None;
        if !self.enabled {
            return;
        }
        match self.cfg.role {
            Role::Off => self.state = "idle".into(),
            Role::Server => {
                self.state = "starting".into();
                match self.cfg.server.listen.parse() {
                    Ok(addr) => self.execute(NetCommand::Listen { addr }),
                    Err(e) => {
                        self.state = "error".into();
                        self.error("invalid_address", format!("listen: {e}"));
                    }
                }
            }
            Role::Client => match remote(&self.cfg.client.remote, None) {
                Ok((host, port)) => {
                    self.state = if self.attempt == 0 {
                        "connecting"
                    } else {
                        "reconnecting"
                    }
                    .into();
                    if let Some((_, fingerprint)) = self
                        .expected_pin
                        .as_ref()
                        .filter(|(address, _)| address == &self.cfg.client.remote)
                    {
                        self.execute(NetCommand::ConnectPinned {
                            host,
                            port,
                            fingerprint: *fingerprint,
                        });
                    } else {
                        self.execute(NetCommand::Connect { host, port });
                    }
                }
                Err(e) => {
                    self.state = "error".into();
                    self.error("invalid_address", e);
                }
            },
        }
    }
    fn process_net(&mut self) {
        let mut rounds = 0;
        while !self.net_events.is_empty() && rounds < 32 {
            rounds += 1;
            let events = std::mem::take(&mut self.net_events);
            for event in events {
                self.status_dirty = true;
                match event {
                    NetEvent::Listening { addr } => {
                        self.state = "listening".into();
                        log::info!("listening on {addr}");
                    }
                    NetEvent::ListenFailed { addr, error } => {
                        self.state = "error".into();
                        log::warn!("listener {addr}: {error}");
                        self.error(
                            "listen_failed",
                            format!("Can't listen at {addr}. Check the port and listen address."),
                        );
                    }
                    NetEvent::ConnectFailed { host, port, error } => {
                        if error != "shutdown" && self.enabled {
                            log::warn!("connector: {host}:{port}: {error}");
                            let kind = openergy_net::ConnectErrorKind::from_message(&error);
                            let address = host_port(&host, port);
                            let message = match kind {
                                openergy_net::ConnectErrorKind::Refused => {
                                    format!("No server is listening at {address}.")
                                }
                                openergy_net::ConnectErrorKind::Timeout => format!("No answer from {address}."),
                                openergy_net::ConnectErrorKind::Unreachable => format!("Can't reach {address}."),
                                openergy_net::ConnectErrorKind::Dns => format!("Can't find host {host}."),
                                openergy_net::ConnectErrorKind::PinMismatch => {
                                    "The server's identity changed — check its fingerprint.".into()
                                }
                                openergy_net::ConnectErrorKind::Tls => {
                                    format!("Can't establish a secure connection to {address}.")
                                }
                                openergy_net::ConnectErrorKind::Other => format!("Can't connect to {address}."),
                            };
                            self.error(kind.code(), message);
                            self.reconnect();
                        }
                    }
                    NetEvent::Connected {
                        conn,
                        peer,
                        fingerprint,
                        tls,
                        tls_version,
                    } => {
                        self.pending.remove(&conn);
                        self.language_unsupported_logged = false;
                        self.connections.insert(
                            conn,
                            Connection {
                                address: peer.to_string(),
                                fingerprint: fingerprint.map(|f| f.to_colon_hex()),
                                tls,
                                tls_version,
                                certificate_verified: self
                                    .driver
                                    .conn_info(conn)
                                    .is_some_and(|c| c.certificate_verified),
                                since: unix_ms(),
                            },
                        );
                        if !tls {
                            log::warn!("plaintext connection: {peer}");
                        }
                        let verified = self.connections.get(&conn).is_some_and(|c| c.certificate_verified);
                        match &mut self.machine {
                            Machine::Server(s) => {
                                s.handle(
                                    ServerEvent::ConnectionMetadata {
                                        conn,
                                        fingerprint: fingerprint.map(|f| f.to_colon_hex()),
                                        certificate_verified: verified,
                                    },
                                    &mut self.actions,
                                );
                                s.handle(ServerEvent::ClientConnected { conn }, &mut self.actions);
                            }
                            Machine::Client(c) => {
                                c.handle(
                                    ClientEvent::ConnectionMetadata {
                                        conn,
                                        fingerprint: fingerprint.map(|f| f.to_colon_hex()),
                                        certificate_verified: verified,
                                    },
                                    &mut self.actions,
                                );
                                c.handle(ClientEvent::ServerConnected { conn }, &mut self.actions);
                            }
                            Machine::Off => self.execute(NetCommand::Close { conn }),
                        }
                    }
                    NetEvent::Message { conn, msg } => {
                        self.events_in += 1;
                        match &mut self.machine {
                            Machine::Server(s) => s.handle(ServerEvent::ClientMessage { conn, msg }, &mut self.actions),
                            Machine::Client(c) => c.handle(ClientEvent::ServerMessage { conn, msg }, &mut self.actions),
                            _ => {}
                        }
                    }
                    NetEvent::Disconnected { conn, reason } => {
                        self.pending.remove(&conn);
                        self.connections.remove(&conn);
                        self.deferred_bulk.remove(&conn);
                        self.files.wire_alloc.remove(&conn);
                        self.files.pending_offers.retain(|_, offer| offer.conn != conn);
                        self.files.recvs.retain(|_, (c, _)| *c != conn);
                        match &mut self.machine {
                            Machine::Server(s) => {
                                s.handle(ServerEvent::ClientDisconnected { conn, reason }, &mut self.actions)
                            }
                            Machine::Client(c) => {
                                c.handle(ClientEvent::ServerDisconnected { conn, reason }, &mut self.actions)
                            }
                            _ => {}
                        }
                        if self.cfg.role == Role::Client {
                            self.healthy_since = None;
                            self.reconnect();
                        }
                    }
                    NetEvent::UntrustedPeer {
                        conn,
                        fingerprint,
                        addr,
                    } => {
                        let pending = ipc::PendingTrust {
                            fingerprint: fingerprint.to_colon_hex(),
                            address: Some(addr.to_string()),
                            name: None,
                            expires_unix_ms: Some(unix_ms() + 30_000),
                        };
                        self.publish("peer.untrusted", serde_json::to_value(&pending).unwrap_or(Value::Null));
                        self.pending.insert(conn, pending);
                        self.retry = None;
                        self.error("untrusted_peer", "Peer fingerprint needs approval");
                    }
                    NetEvent::Discovered(peers) => {
                        self.discovery=json!(peers.into_iter().map(|p|json!({"name":p.name,"host":p.host,"port":p.port,"fingerprint_hint":p.fingerprint_hint})).collect::<Vec<_>>());
                        self.publish("discovery.changed", self.discovery.clone());
                    }
                    NetEvent::Error { conn, message } => {
                        log::warn!("transport error peer={conn:?}: {message}");
                        self.error("network_failed", format!("Network operation failed: {message}."));
                    }
                    NetEvent::Frame { conn, .. } => {
                        log::warn!("unexpected diagnostic frame from peer {}", conn.0);
                        self.error("protocol_error", "An unexpected diagnostic frame was received.");
                    }
                }
            }
            self.apply_actions();
        }
        let connected = self.machine.connected();
        if self.cfg.role == Role::Client && connected {
            if self.state != "connected" {
                self.last_error = None;
            }
            self.state = "connected".into();
            self.retry = None;
            let now = self.now();
            let since = *self.healthy_since.get_or_insert(now);
            if now.saturating_sub(since) >= 60_000 {
                self.attempt = 0;
            }
        } else if self.cfg.role == Role::Server && self.enabled && connected {
            if self.state != "connected" {
                self.last_error = None;
            }
            self.state = "connected".into();
        } else if self.cfg.role == Role::Server && self.enabled && self.state == "connected" {
            self.state = "listening".into();
        }
    }
    fn refresh_health(&mut self) {
        let languages = self.screens.keyboard_languages();
        if languages != self.languages {
            self.languages = languages.clone();
            self.machine.languages(languages, &mut self.actions);
        }
        let secure = self.screens.secure_input_app();
        if secure != self.local_secure {
            self.status_dirty = true;
            self.local_secure = secure.clone();
            if let Machine::Server(s) = &mut self.machine {
                s.handle(ServerEvent::LocalSecureInput { app: secure }, &mut self.actions);
            }
        }
        let displays = self.screens.screens();
        let changed = displays != self.displays;
        self.status_dirty |= changed;
        self.displays = displays;
        self.machine
            .geometry(&self.displays, self.screens.cursor_pos(), changed, &mut self.actions);
        let permissions = self.screens.permissions();
        self.sink.permissions_changed(&permissions);
        let denied = matches!(permissions.accessibility, Grant::Denied)
            || matches!(permissions.input_monitoring, Grant::Denied)
            || matches!(permissions.post_events, Grant::Denied);
        if permissions != self.permissions {
            self.status_dirty = true;
            self.permissions = permissions;
            if denied {
                match &mut self.machine {
                    Machine::Server(s) => s.handle(ServerEvent::PermissionLost, &mut self.actions),
                    Machine::Client(c) => c.handle(ClientEvent::PermissionLost, &mut self.actions),
                    _ => {}
                }
            }
            if !denied {
                let _ = self
                    .capture
                    .commands
                    .try_send(CaptureCommand::Enable(self.enabled && self.cfg.role == Role::Server));
            }
            self.publish(
                "permissions.changed",
                serde_json::to_value(&self.permissions).unwrap_or(Value::Null),
            );
        }
        if !denied
            && !self.suspended()
            && self.enabled
            && self.cfg.role == Role::Server
            && self.capture.retry.swap(false, Ordering::AcqRel)
            && self.capture.commands.try_send(CaptureCommand::Enable(true)).is_err()
        {
            self.capture.retry.store(true, Ordering::Release);
        }
        if self.capture.overflow.swap(false, Ordering::Relaxed) {
            if let Machine::Server(s) = &mut self.machine {
                s.handle(ServerEvent::CaptureOverflow, &mut self.actions);
            }
            self.error("capture_overflow", "Capture channel full; event passed to the local OS");
        }
    }
    fn apply_config(&mut self, mut cfg: Config, force_reconnect: bool) -> Result<(), String> {
        normalize(&mut cfg);
        cfg.validate().map_err(|e| e.to_string())?;
        if self.platform == "macos" && cfg.macos != self.cfg.macos {
            return Err("Changing macos.click_numbering requires a daemon restart; stop the daemon, edit the config, then start it again".into());
        }
        let new_receive_dir = if cfg.file.receive_dir != self.cfg.file.receive_dir {
            Some(resolve_receive_dir(&cfg, &self.dir)?)
        } else {
            None
        };
        let reconnect = force_reconnect
            || cfg.role != self.cfg.role
            || cfg.server != self.cfg.server
            || cfg.client.remote != self.cfg.client.remote
            || cfg.tls != self.cfg.tls
            || cfg.name != self.cfg.name;
        let mut durability_warning = None;
        if reconnect {
            let listen = if cfg.role == Role::Server {
                Some(cfg.server.listen.parse().map_err(|_| "Invalid listen address")?)
            } else {
                None
            };
            self.driver
                .reconfigure_transaction(
                    adapter::net_config(&cfg, &self.dir, self.platform == "virtual"),
                    listen,
                    || {
                        durability_warning = write_config(&self.path, &cfg).map_err(openergy_net::NetError::Failed)?;
                        Ok(())
                    },
                )
                .map_err(|error| error.to_string())?;
            self.machine.shutdown(&mut self.actions);
            self.apply_actions();
            self.execute(NetCommand::DisconnectAll);
            self.process_net();
            self.connections.clear();
            self.pending.clear();
            self.deferred_bulk.clear();
            self.save_xfer_counter();
            self.reset_file_state("io_error");
            self.cfg = cfg;
            self.machine = Machine::new(Arc::new(self.cfg.clone()));
            self.seed_xfer_counter();
            self.enabled = self.cfg.role != Role::Off && (self.cfg.role != Role::Client || self.cfg.client.autoconnect);
            self.attempt = 0;
            self.retry = None;
            self.healthy_since = None;
            self.state = "idle".into();
            self.machine
                .geometry(&self.displays, self.screens.cursor_pos(), false, &mut self.actions);
            self.start_connection();
        } else {
            let previous = self.machine.clone();
            let action_count = self.actions.len();
            let cfg = Arc::new(cfg);
            match &mut self.machine {
                Machine::Server(s) => s.handle(ServerEvent::ConfigChanged(cfg.clone()), &mut self.actions),
                Machine::Client(c) => c.handle(ClientEvent::ConfigChanged(cfg.clone()), &mut self.actions),
                Machine::Off => {}
            }
            // Apply to the live state before saving. Platform/network effects
            // remain queued until persistence commits; rollback preserves peers.
            match write_config(&self.path, &cfg) {
                Ok(warning) => durability_warning = warning,
                Err(error) => {
                    self.machine = previous;
                    self.actions.truncate(action_count);
                    return Err(error);
                }
            }
            // The session dropped its transfers on a clipboard policy
            // change; held chunks must not resurrect them.
            if cfg.clipboard != self.cfg.clipboard {
                self.deferred_bulk.clear();
            }
            if cfg.file != self.cfg.file {
                // Session reset already cancelled live transfers.
                // Fail sends that never reached the session and drop
                // held file chunks plus pending offers (§7.3).
                let queued = self
                    .files
                    .sends
                    .iter()
                    .filter(|(_, rec)| rec.stage != SendStage::Active)
                    .map(|(xfer, _)| *xfer)
                    .collect::<Vec<_>>();
                for xfer in queued {
                    self.fail_send_local(xfer, "disabled");
                }
                self.files.pending_offers.clear();
                let conns = self.deferred_bulk.keys().copied().collect::<Vec<_>>();
                for conn in conns {
                    if let Some(backlog) = self.deferred_bulk.get_mut(&conn) {
                        backlog
                            .chunks
                            .retain(|(tag, _, _)| matches!(tag, BulkTag::Clipboard(_)));
                        backlog.bytes = backlog.chunks.iter().map(|(_, _, wire)| *wire).sum();
                    }
                }
                self.deferred_bulk.retain(|_, backlog| !backlog.chunks.is_empty());
            }
            self.cfg = Arc::unwrap_or_clone(cfg);
        }
        if let Some(root) = new_receive_dir {
            self.files.receive_dir = root.clone();
            self.files.receive_dir_ok = true;
            let _ = self.worker_file(FileCommand::SetRoot { root });
        }
        self.machine
            .geometry(&self.displays, self.screens.cursor_pos(), false, &mut self.actions);
        let _ = self
            .capture
            .commands
            .try_send(CaptureCommand::Enable(self.enabled && self.cfg.role == Role::Server));
        let _ = self
            .clipboard
            .commands
            .try_send(self.clip_enabled(self.enabled && self.cfg.clipboard.enabled));
        self.update_sleep_assertion();
        if let Some(warning) = durability_warning {
            // Rename has committed. Keep disk and live state aligned and report
            // uncertain crash durability instead of rolling back only live state.
            self.error("config_durability_uncertain", warning);
        }
        self.publish(
            "config.changed",
            json!({"config":self.cfg.to_json(),"derived_links":openergy_config::derive_links(&self.cfg.screens)}),
        );
        Ok(())
    }
    fn dispatch(&mut self, req: &Request, queue: &Arc<EventQueue>) -> Result<Value, IpcError> {
        self.status_dirty |= req.method != m::STATUS_GET;
        let invalid = |msg: &str| IpcError::new("invalid_params", msg);
        match req.method.as_str() {
            m::STATUS_GET => Ok(self.status().to_json()),
            m::CONFIG_GET => Ok(
                json!({"config":self.cfg.to_json(),"derived_links":openergy_config::derive_links(&self.cfg.screens)}),
            ),
            m::CONFIG_PATCH => {
                let patch = req.params.get("patch").ok_or_else(|| invalid("patch is required"))?;
                let (mut cfg, warnings) = openergy_config::apply_merge_patch(&self.cfg, patch)
                    .map_err(|e| IpcError::new("invalid_config", e.to_string()))?;
                normalize(&mut cfg);
                self.apply_config(cfg, false)
                    .map_err(|e| IpcError::new("config_apply_failed", e))?;
                Ok(json!({"config":self.cfg.to_json(),"warnings":warnings}))
            }
            m::CONFIG_SET => {
                let text = req
                    .params
                    .get("toml")
                    .or_else(|| req.params.get("text"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("toml text is required"))?;
                let cfg = Config::load_toml(text).map_err(|e| IpcError::new("invalid_config", e.to_string()))?;
                self.apply_config(cfg, false)
                    .map_err(|e| IpcError::new("config_apply_failed", e))?;
                Ok(json!({"config":self.cfg.to_json(),"warnings":[]}))
            }
            m::CONFIG_IMPORT_SYNERGY => {
                let path = req
                    .params
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("path is required"))?;
                let (cfg, report) = openergy_config::import_synergy_dir(Path::new(path))
                    .map_err(|e| IpcError::new("import_failed", e.to_string()))?;
                let result = json!({"config":cfg.to_json(),"warnings":report.warnings});
                if req.params.get("apply").and_then(Value::as_bool) == Some(true) {
                    self.apply_config(cfg, false)
                        .map_err(|e| IpcError::new("config_apply_failed", e))?;
                }
                Ok(result)
            }
            m::CONNECTION_CONNECT => {
                if let Some(host) = req.params.get("host").and_then(Value::as_str) {
                    let port = req
                        .params
                        .get("port")
                        .and_then(Value::as_u64)
                        .map(|p| u16::try_from(p).map_err(|_| invalid("port must fit u16")))
                        .transpose()?;
                    let (host, port) = remote(host, port).map_err(|e| invalid(&e))?;
                    let mut cfg = self.cfg.clone();
                    cfg.role = Role::Client;
                    cfg.client.remote = host_port(&host, port);
                    cfg.client.autoconnect = true;
                    let previous_pin = self.expected_pin.clone();
                    let pinned = if let Some(text) = req.params.get("fingerprint") {
                        let fingerprint = text
                            .as_str()
                            .and_then(Fingerprint::parse)
                            .ok_or_else(|| invalid("fingerprint must be a SHA-256 fingerprint"))?;
                        // Explicit pins are scoped to this address for this daemon's
                        // lifetime; retries retain the pin and cannot use plaintext.
                        if !cfg.tls.enabled {
                            return Err(invalid("an explicit fingerprint requires TLS"));
                        }
                        self.expected_pin = Some((cfg.client.remote.clone(), fingerprint));
                        true
                    } else {
                        false
                    };
                    if let Err(error) = self.apply_config(cfg, pinned) {
                        self.expected_pin = previous_pin;
                        return Err(IpcError::new("connect_failed", error));
                    }
                }
                self.enabled = true;
                self.start_connection();
                let _ = self
                    .capture
                    .commands
                    .try_send(CaptureCommand::Enable(self.cfg.role == Role::Server));
                let _ = self
                    .clipboard
                    .commands
                    .try_send(self.clip_enabled(self.cfg.clipboard.enabled));
                Ok(json!({"connecting":true}))
            }
            m::CONNECTION_DISCONNECT => {
                self.enabled = false;
                self.retry = None;
                self.deferred_bulk.clear();
                self.reset_file_state("io_error");
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::Shutdown);
                self.state = "idle".into();
                self.connections.clear();
                self.pending.clear();
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
                let _ = self.clipboard.commands.try_send(self.clip_enabled(false));
                Ok(json!({"disconnected":true}))
            }
            m::CONNECTION_RESTART => {
                self.enabled = false;
                self.deferred_bulk.clear();
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::Shutdown);
                self.process_net();
                self.save_xfer_counter();
                self.reset_file_state("io_error");
                self.machine = Machine::new(Arc::new(self.cfg.clone()));
                self.seed_xfer_counter();
                self.machine
                    .geometry(&self.displays, self.screens.cursor_pos(), false, &mut self.actions);
                self.enabled = true;
                self.retry = None;
                self.state = "idle".into();
                self.start_connection();
                let _ = self
                    .capture
                    .commands
                    .try_send(CaptureCommand::Enable(self.cfg.role == Role::Server));
                let _ = self
                    .clipboard
                    .commands
                    .try_send(self.clip_enabled(self.cfg.clipboard.enabled));
                Ok(json!({"restarting":true}))
            }
            m::TRUST_LIST => Ok(
                json!({"servers":self.driver.trust_store().servers().iter().map(Fingerprint::to_colon_hex).collect::<Vec<_>>(),"clients":self.driver.trust_store().clients().iter().map(Fingerprint::to_colon_hex).collect::<Vec<_>>()}),
            ),
            m::TRUST_APPROVE | m::TRUST_REJECT | m::TRUST_REVOKE => {
                let text = req
                    .params
                    .get("fingerprint")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("fingerprint is required"))?;
                let fp = Fingerprint::parse(text).ok_or_else(|| invalid("invalid SHA-256 fingerprint"))?;
                match req.method.as_str() {
                    m::TRUST_APPROVE => {
                        if self.pending.values().any(|p| p.fingerprint == fp.to_colon_hex()) {
                            self.driver
                                .approve_held(&fp, &mut self.net_events)
                                .map_err(|e| IpcError::new("trust_failed", e.to_string()))?;
                        } else {
                            let store = self.driver.trust_store_mut();
                            let result = if self.cfg.role == Role::Server {
                                store.try_trust_client(fp)
                            } else {
                                store.try_trust_server(fp)
                            };
                            result.map_err(|e| IpcError::new("trust_failed", e.to_string()))?;
                        }
                    }
                    m::TRUST_REJECT => self.driver.reject_held(&fp, &mut self.net_events),
                    _ => {
                        self.driver
                            .trust_store_mut()
                            .try_revoke(&fp)
                            .map_err(|e| IpcError::new("trust_failed", e.to_string()))?;
                        if self.expected_pin.as_ref().is_some_and(|(_, pin)| *pin == fp) {
                            self.expected_pin = None;
                        }
                        self.driver.reject_held(&fp, &mut self.net_events);
                        let conns = self
                            .connections
                            .iter()
                            .filter_map(|(&c, info)| {
                                (info.fingerprint.as_deref() == Some(&fp.to_colon_hex())).then_some(c)
                            })
                            .collect::<Vec<_>>();
                        for conn in conns {
                            // Revocation must cut off application reads before the
                            // IPC reply, including connections authenticated by an
                            // explicit pin. Graceful close can still read peer input.
                            self.execute(NetCommand::Reject { conn });
                        }
                    }
                }
                self.pending.retain(|_, p| p.fingerprint != fp.to_colon_hex());
                Ok(json!({"ok":true}))
            }
            m::DISCOVERY_LIST => Ok(self.discovery.clone()),
            m::EVENTS_SUBSCRIBE => {
                queue.status.store(true, Ordering::Relaxed);
                queue.push(Event {
                    event: "status.changed".into(),
                    data: self.status().to_json(),
                });
                Ok(json!({"subscribed":true}))
            }
            m::LOGS_TAIL => Ok(json!(self
                .logger
                .tail(req.params.get("lines").and_then(Value::as_u64).unwrap_or(50) as usize))),
            m::LOGS_SUBSCRIBE => {
                queue.logs.store(true, Ordering::Relaxed);
                Ok(json!({"subscribed":true}))
            }
            m::LOGS_UNSUBSCRIBE => {
                queue.logs.store(false, Ordering::Relaxed);
                Ok(json!({"subscribed":false}))
            }
            m::PERMISSIONS_REQUEST => {
                let kind = req
                    .params
                    .get("kind")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("kind is required"))?;
                if !["accessibility", "input_monitoring", "post_events"].contains(&kind) {
                    return Err(invalid("unknown permission kind"));
                }
                let kind = match kind {
                    "accessibility" => openergy_platform::PermissionKind::Accessibility,
                    "input_monitoring" => openergy_platform::PermissionKind::InputMonitoring,
                    _ => openergy_platform::PermissionKind::PostEvents,
                };
                let grant = self
                    .screens
                    .request_permission(kind)
                    .map_err(|e| IpcError::new("permission_request_failed", e.to_string()))?;
                self.refresh_health();
                Ok(json!({"grant":grant,"permissions":self.screens.permissions()}))
            }
            m::PERMISSIONS_OPEN_SETTINGS => Err(IpcError::new(
                "settings_unavailable",
                "Open permission settings in the native app",
            )),
            m::DAEMON_STOP => {
                self.stop.store(true, Ordering::Relaxed);
                Ok(json!({"stopping":true}))
            }
            m::FILE_SEND => {
                let peer = req
                    .params
                    .get("peer")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("peer is required"))?;
                let path = req
                    .params
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid("path is required"))?;
                if !self.cfg.file.enabled || !self.cfg.file.send {
                    return Err(IpcError::new("file_disabled", "File sending is disabled"));
                }
                // Resolve the screen name ONCE, now. The binding
                // (connection + certificate) travels with the xfer
                // and is re-validated before any offer.
                let snapshot = self.machine.snapshot();
                let Some(found) = snapshot.peers.iter().find(|p| p.connected && p.name == peer) else {
                    return Err(IpcError::new("peer_unknown", "No connected peer with that name"));
                };
                let conn = found.conn;
                let peer_cap = found.file_max_bytes.unwrap_or(sfile::FILE_HARD_CAP);
                let verified = self
                    .connections
                    .get(&conn)
                    .is_some_and(|c| c.tls && c.certificate_verified);
                if !self.cfg.tls.enabled || !verified {
                    return Err(IpcError::new(
                        "tls_required",
                        "File transfer requires a verified TLS peer",
                    ));
                }
                if found.file_capability != "openergy" {
                    return Err(IpcError::new(
                        "peer_no_file_support",
                        "That peer cannot receive Openergy files",
                    ));
                }
                let fingerprint = self.connections.get(&conn).and_then(|c| c.fingerprint.clone());
                let path_buf = PathBuf::from(path);
                let link_meta =
                    std::fs::symlink_metadata(&path_buf).map_err(|e| IpcError::new("io_error", e.to_string()))?;
                if link_meta.file_type().is_symlink() {
                    return Err(IpcError::new("is_symlink", "Symlinks cannot be sent"));
                }
                if link_meta.is_dir() {
                    return Err(IpcError::new("is_directory", "Folders cannot be sent"));
                }
                if !link_meta.is_file() {
                    return Err(IpcError::new("not_a_file", "Only regular files can be sent"));
                }
                let limit = self.cfg.file.max_bytes.min(sfile::FILE_HARD_CAP).min(peer_cap);
                if link_meta.len() > limit {
                    let blocked = ipc::FileBlocked {
                        peer: Some(peer.into()),
                        bytes: Some(link_meta.len()),
                        limit: Some(limit),
                        reason: "too_large".into(),
                        unix_ms: Some(unix_ms()),
                    };
                    self.publish("file.blocked", serde_json::to_value(&blocked).unwrap_or(Value::Null));
                    self.last_file_blocked = Some(blocked);
                    return Err(IpcError::new("too_large", "File exceeds the size limit"));
                }
                let xfer = self.machine.alloc_xfer();
                if xfer == 0 {
                    return Err(IpcError::new("peer_unknown", "No session is running"));
                }
                self.files.sends.insert(
                    xfer,
                    SendRec {
                        conn,
                        peer_name: peer.into(),
                        fingerprint,
                        path: path_buf,
                        total_len: link_meta.len(),
                        wire: 0,
                        stage: SendStage::Queued,
                    },
                );
                self.files.send_queue.entry(conn).or_default().push_back(xfer);
                self.start_next_send(conn);
                Ok(json!({"xfer": xfer}))
            }
            m::FILE_ANSWER => {
                let xfer = req
                    .params
                    .get("xfer")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| invalid("xfer is required"))?;
                let accept = req
                    .params
                    .get("accept")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| invalid("accept is required"))?;
                if !self.files.pending_offers.contains_key(&xfer) {
                    // An accept for a transfer that already left the
                    // pending state because an earlier accept landed is
                    // a benign retry: the chosen resolution stands.
                    if accept && self.files.recvs.contains_key(&xfer) {
                        return Ok(json!({"ok": true}));
                    }
                    return Err(IpcError::new("unknown_xfer", "No pending offer with that handle"));
                }
                if accept {
                    self.begin_receive(xfer);
                } else {
                    self.files.pending_offers.remove(&xfer);
                    self.files.recvs.remove(&xfer);
                    self.decide_offer(xfer, false, FileReason::UserRejected);
                }
                Ok(json!({"ok": true}))
            }
            m::FILE_CANCEL => {
                let xfer = req
                    .params
                    .get("xfer")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| invalid("xfer is required"))?;
                if let Some(rec) = self.files.sends.get(&xfer) {
                    if rec.stage == SendStage::Active {
                        self.cancel_session_file(xfer);
                    } else {
                        self.fail_send_local(xfer, "cancelled");
                    }
                    return Ok(json!({"ok": true}));
                }
                if self.files.pending_offers.contains_key(&xfer) {
                    self.files.pending_offers.remove(&xfer);
                    self.files.recvs.remove(&xfer);
                    self.decide_offer(xfer, false, FileReason::Cancelled);
                    return Ok(json!({"ok": true}));
                }
                if self.files.recvs.contains_key(&xfer) {
                    self.cancel_session_file(xfer);
                    return Ok(json!({"ok": true}));
                }
                Err(IpcError::new("unknown_xfer", "No transfer with that handle"))
            }
            method if method.starts_with("virtual.") => self.virtual_request(method, &req.params),
            _ => Err(IpcError::new(
                "unknown_method",
                format!("unknown method {}", req.method),
            )),
        }
    }
    fn virtual_request(&mut self, method: &str, params: &Value) -> Result<Value, IpcError> {
        let control = self
            .control
            .as_ref()
            .ok_or_else(|| IpcError::new("virtual_unavailable", "virtual.* requires --platform virtual"))?;
        let result = crate::adapter::virtual_request(control, method, params);
        if result.is_ok()
            && (matches!(method, m::VIRTUAL_SET_SCREENS | m::VIRTUAL_SET_PERMISSIONS)
                || (method == m::VIRTUAL_INJECT && params.get("keyboard_languages").is_some()))
        {
            self.refresh_health();
        }
        result
    }
}
fn selection_name(id: ClipboardId) -> &'static str {
    if id.0 == 1 {
        "primary"
    } else {
        "clipboard"
    }
}
fn display_rect(screens: &[ScreenRect]) -> Rect {
    let x = screens.iter().map(|s| s.x).min().unwrap_or(0);
    let y = screens.iter().map(|s| s.y).min().unwrap_or(0);
    let right = screens.iter().map(|s| s.x.saturating_add(s.w)).max().unwrap_or(1920);
    let bottom = screens.iter().map(|s| s.y.saturating_add(s.h)).max().unwrap_or(1080);
    Rect {
        x,
        y,
        w: right.saturating_sub(x).clamp(1, i16::MAX as i32),
        h: bottom.saturating_sub(y).clamp(1, i16::MAX as i32),
    }
}
fn host_port(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}
fn remote(text: &str, override_port: Option<u16>) -> Result<(String, u16), String> {
    if text.is_empty() {
        return Err("remote host is empty".into());
    }
    if let Ok(addr) = text.parse::<std::net::SocketAddr>() {
        return Ok((addr.ip().to_string(), override_port.unwrap_or(addr.port())));
    }
    if let Some((host, port)) = text.rsplit_once(':') {
        if !host.contains(':') {
            let port = override_port
                .or_else(|| port.parse().ok())
                .ok_or("invalid remote port")?;
            if port == 0 {
                return Err("port must be positive".into());
            }
            return Ok((host.into(), port));
        }
    }
    Ok((text.into(), override_port.unwrap_or(24800)))
}
/// Resolve `file.receive_dir` once per config apply (design §7.2):
/// expand `~`, require an absolute result, create `0700`, canonicalize.
fn resolve_receive_dir(cfg: &Config, state_dir: &Path) -> Result<PathBuf, String> {
    let raw = cfg.file.receive_dir.trim();
    let path = if raw.is_empty() {
        state_dir.join("Received")
    } else if let Some(rest) = raw
        .strip_prefix("~/")
        .or_else(|| if raw == "~" { Some("") } else { None })
    {
        let home = std::env::var_os("HOME").ok_or("file.receive_dir uses ~ but HOME is unset")?;
        PathBuf::from(home).join(rest)
    } else {
        let p = PathBuf::from(raw);
        if p.is_absolute() {
            p
        } else {
            state_dir.join(p)
        }
    };
    std::fs::create_dir_all(&path).map_err(|e| format!("file.receive_dir: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700));
    }
    Ok(path.canonicalize().unwrap_or(path))
}

fn normalize(cfg: &mut Config) {
    let listen = &cfg.server.listen;
    let host = if let Ok(addr) = listen.parse::<std::net::SocketAddr>() {
        addr.ip().to_string()
    } else if listen.parse::<std::net::IpAddr>().is_ok() || listen.starts_with('[') {
        listen.clone()
    } else {
        listen
            .rsplit_once(':')
            .map(|(host, _)| host.to_string())
            .unwrap_or_else(|| listen.clone())
    };
    cfg.server.listen = host_port(&host, cfg.server.port);
    if cfg.name.is_empty() {
        cfg.name = host_name();
    }
}
fn host_name() -> String {
    let name = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_default();
    #[cfg(unix)]
    let mut name = name;
    #[cfg(unix)]
    if name.is_empty() {
        let mut bytes = [0u8; 256];
        // Safety: gethostname writes at most the fixed output buffer length and has no side effects.
        if unsafe { libc::gethostname(bytes.as_mut_ptr().cast(), bytes.len()) } == 0 {
            let length = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
            name = String::from_utf8_lossy(bytes.get(..length).unwrap_or_default()).into_owned();
        }
    }
    let name = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-._".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    if name.is_empty() {
        "openergy".into()
    } else {
        name
    }
}

fn write_config(path: &Path, cfg: &Config) -> Result<Option<String>, String> {
    use std::io::Write;
    let text = cfg.to_toml();
    if text.len() as u64 > crate::MAX_CONFIG_FILE_LEN {
        return Err("config exceeds 16 MiB (16777216 bytes)".into());
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    let parent = std::fs::File::open(parent).map_err(|e| e.to_string())?;
    let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut created = false;
    let result = (|| {
        let mut file = options.open(&tmp).map_err(|e| e.to_string())?;
        created = true;
        file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Err(error) = parent.sync_all() {
            return Ok(Some(format!(
                "Config was saved, but syncing its directory failed; crash durability is uncertain: {error}"
            )));
        }
        Ok(None)
    })();
    if result.is_err() && created {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

pub fn run_daemon(
    socket_path: PathBuf,
    mut config: Config,
    config_path: Option<PathBuf>,
    platform: String,
    parent_pid: Option<i64>,
    startup_error: Option<String>,
) -> Result<i32, String> {
    normalize(&mut config);
    config.validate().map_err(|e| format!("config: {e}"))?;
    if matches!(platform.as_str(), "macos" | "x11" | "windows")
        && std::env::var_os("OPENERGY_ALLOW_PORT_24800").is_none()
    {
        // Startup is still single-threaded. Explicit lab denials and virtual
        // backends retain the transport's protected-port guard.
        std::env::set_var("OPENERGY_ALLOW_PORT_24800", "1");
    }
    let path = config_path.unwrap_or_else(crate::default_config_path);
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let (listener, _lock) = ipc_server::bind(&socket_path)?;
    let bus = Arc::new(EventBus::default());
    let logger = Logger::new(&dir, bus.clone());
    logger.install();
    let parts = adapter::platform(&platform, &config).map_err(|e| e.to_string())?;
    let (mut backend, control, notices) = (parts.backend, parts.control, parts.notices);
    let (driver, _net_handle) =
        NetDriver::new(adapter::net_config(&config, &dir, platform == "virtual")).map_err(|e| e.to_string())?;
    let waker = driver.waker();
    let display_changed = Arc::new(AtomicBool::new(false));
    let flag = display_changed.clone();
    let display_waker = waker.clone();
    backend.screens.on_display_changed(Box::new(move || {
        flag.store(true, Ordering::Release);
        let _ = display_waker.wake();
    }));
    let capture = workers::capture(backend.source, waker.clone());
    let clipboard = workers::clipboard(backend.clipboard, waker.clone());
    let (receive_dir, receive_dir_ok) = match resolve_receive_dir(&config, &dir) {
        Ok(dir) => (dir, true),
        Err(error) => {
            // Startup keeps running off-role style, but receiving
            // fails closed: offers are rejected until the configured
            // directory resolves, never redirected to a fallback.
            log::warn!("{error}; file receiving disabled until file.receive_dir is usable");
            (dir.join("Received"), false)
        }
    };
    let file_worker = workers::file_worker(receive_dir.clone(), waker.clone());
    let stop = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::sync_channel(128);
    let accept = ipc_server::serve(listener, socket_path.clone(), tx, waker, bus.clone(), stop.clone());
    let displays = backend.screens.screens();
    let permissions = backend.screens.permissions();
    let machine = Machine::new(Arc::new(config.clone()));
    let enabled = config.role != Role::Off && (config.role != Role::Client || config.client.autoconnect);
    let mut rt = Runtime {
        cfg: config,
        path,
        dir,
        platform,
        started: Instant::now(),
        driver,
        machine,
        sink: backend.sink,
        screens: backend.screens,
        control,
        notices,
        asleep: false,
        locked: false,
        resume_enabled: false,
        close_at: None,
        capture,
        clipboard,
        file_worker,
        files: FileRuntime {
            receive_dir,
            receive_dir_ok,
            ..FileRuntime::default()
        },
        bus,
        logger,
        state: "idle".into(),
        connections: BTreeMap::new(),
        pending: BTreeMap::new(),
        expected_pin: None,
        discovery: json!([]),
        last_error: None,
        last_transfer: None,
        last_blocked: None,
        last_file_completed: None,
        last_file_blocked: None,
        next_xfer: (unix_ms() ^ (u64::from(std::process::id()) << 32) ^ 0x5EED_u64).max(1),
        events_in: 0,
        events_out: 0,
        attempt: 0,
        retry: None,
        healthy_since: None,
        enabled,
        status_dirty: true,
        last_status: Value::Null,
        last_status_ms: 0,
        last_health: 0,
        languages: openergy_platform::KeyboardLanguages::default(),
        language_unsupported_logged: false,
        last_network_sample: 0,
        network_set: None,
        permissions,
        local_secure: None,
        display_changed,
        preventing_sleep: false,
        displays,
        rng: unix_ms() ^ u64::from(std::process::id()),
        stop: stop.clone(),
        actions: Vec::new(),
        net_events: Vec::new(),
        deferred_bulk: BTreeMap::new(),
    };
    lifetime::install_signals();
    let parent = lifetime::ParentWatch::new(parent_pid);
    // A fallback (role=off) start still tells ctl/app why it is off.
    if let Some(error) = startup_error {
        rt.error("invalid_config", error);
    }
    rt.seed_xfer_counter();
    rt.machine
        .geometry(&rt.displays, rt.screens.cursor_pos(), false, &mut rt.actions);
    let _ = rt
        .capture
        .commands
        .try_send(CaptureCommand::Enable(enabled && rt.cfg.role == Role::Server));
    let _ = rt.clipboard.commands.try_send(ClipboardCommand::Enabled {
        clipboard: enabled && rt.cfg.clipboard.enabled,
        primary: rt.cfg.clipboard.share_primary_selection,
    });
    rt.update_sleep_assertion();
    rt.start_connection();
    log::info!("openergy {VERSION} daemon listening on {}", socket_path.display());
    'running: while !stop.load(Ordering::Relaxed) && !lifetime::SIGNALLED.load(Ordering::Relaxed) && !parent.exited() {
        for notice in (rt.notices)() {
            rt.notice(notice);
        }
        if rt.close_at.is_some_and(|deadline| rt.now() >= deadline) {
            rt.close_at = None;
            rt.execute(NetCommand::Shutdown);
            rt.process_net();
        }
        while let Ok(packet) = rt.capture.events.try_recv() {
            rt.status_dirty = true;
            if let Some(movement) = packet.movement {
                rt.input(movement);
            }
            rt.input(packet.event);
        }
        let movement = rt.capture.movement.try_lock().ok().and_then(|mut s| s.take());
        if let Some(movement) = movement {
            rt.input(movement);
        }
        // Input gets the first turn. A busy IPC observer must not keep a
        // captured move behind an unbounded request drain.
        for _ in 0..8 {
            let Ok(IoCommand::Ipc { req, reply, events }) = rx.try_recv() else {
                break;
            };
            let response = match rt.dispatch(&req, &events) {
                Ok(value) => Response::ok(req.id, value),
                Err(error) => Response::err(req.id, error),
            };
            rt.apply_actions();
            rt.process_net();
            let _ = reply.send(response);
            // Acknowledge public stop before entering the full shutdown path.
            // Its request wake may already be consumed; do not poll again first.
            if stop.load(Ordering::Relaxed) {
                break 'running;
            }
        }
        let failed_reads = rt.clipboard.failed_reads.swap(0, Ordering::AcqRel);
        for id in 0..2 {
            if failed_reads & (1 << id) != 0 {
                rt.machine.clipboard_read_failed(ClipboardId(id), &mut rt.actions);
                rt.error("clipboard_busy", "Clipboard result queue is full");
            }
        }
        while let Ok(event) = rt.clipboard.events.try_recv() {
            match event {
                ClipboardEvent::Changed(id) => rt.machine.clipboard_changed(id, &mut rt.actions),
                ClipboardEvent::Payload { id, data } => rt.machine.payload(id, data, &mut rt.actions),
                ClipboardEvent::ReadUnavailable { id } => rt.machine.clipboard_read_failed(id, &mut rt.actions),
                ClipboardEvent::Failure { read_id, error } => {
                    if let Some(id) = read_id {
                        rt.machine.clipboard_read_failed(id, &mut rt.actions);
                    }
                    rt.error("clipboard_failed", error);
                }
                ClipboardEvent::Blocked { id, bytes, limit } => {
                    rt.machine.clipboard_read_failed(id, &mut rt.actions);
                    let blocked = ipc::ClipboardBlocked {
                        bytes: Some(bytes),
                        limit: Some(limit),
                        peer: Some("local".into()),
                        unix_ms: Some(unix_ms()),
                        selection: Some(selection_name(id).into()),
                    };
                    rt.publish(
                        "clipboard.blocked",
                        serde_json::to_value(&blocked).unwrap_or(Value::Null),
                    );
                    rt.last_blocked = Some(blocked);
                }
            }
        }
        let now = rt.now();
        while let Ok(event) = rt.file_worker.events.try_recv() {
            rt.handle_file_event(event);
        }
        rt.expire_offers();
        rt.machine.tick(now, &mut rt.actions);
        rt.apply_actions();
        rt.process_net();
        rt.drain_bulk();
        let health_interval = if matches!(rt.permissions.accessibility, Grant::Denied | Grant::Unknown)
            || matches!(rt.permissions.input_monitoring, Grant::Denied | Grant::Unknown)
            || matches!(rt.permissions.post_events, Grant::Denied | Grant::Unknown)
        {
            250
        } else {
            2000
        };
        if rt.display_changed.swap(false, Ordering::AcqRel) || now.saturating_sub(rt.last_health) >= health_interval {
            rt.last_health = now;
            rt.refresh_health();
            rt.apply_actions();
        }
        if rt.cfg.role == Role::Client
            && rt.state == "reconnecting"
            && rt.retry.is_some()
            && now.saturating_sub(rt.last_network_sample) >= 2000
        {
            rt.last_network_sample = now;
            if let Some(addresses) = crate::interfaces::snapshot() {
                let changed = rt.network_set.as_ref().is_some_and(|previous| *previous != addresses);
                rt.network_set = Some(addresses);
                if changed {
                    log::info!("network interfaces changed; retry configured server");
                    rt.attempt = 0;
                    rt.retry = Some(now);
                }
            }
        }
        if rt.retry.is_some_and(|t| now >= t) {
            rt.start_connection();
        }
        if rt.status_dirty && now.saturating_sub(rt.last_status_ms) >= 100 {
            let status = rt.status().to_json();
            // File progress bytes change on every sample; they have a
            // dedicated coalesced event, so they must not republish the
            // whole status (and flood slow subscribers) by themselves.
            let mut fingerprint = status.clone();
            if let Some(active) = fingerprint.pointer_mut("/file/active").and_then(Value::as_array_mut) {
                for entry in active {
                    if let Some(done) = entry.get_mut("bytes_done") {
                        *done = json!(0);
                    }
                }
            }
            if fingerprint != rt.last_status {
                rt.last_status = fingerprint;
                rt.publish("status.changed", status);
            }
            rt.last_status_ms = now;
            rt.status_dirty = false;
        }
        let now = rt.now();
        let session_deadline = match &rt.machine {
            Machine::Server(s) => s.next_deadline_ms(),
            Machine::Client(c) => c.next_deadline_ms(),
            _ => now + 1000,
        };
        let mut deadline = session_deadline.min(rt.last_health + health_interval);
        if rt.status_dirty {
            deadline = deadline.min(rt.last_status_ms + 100);
        }
        if let Some(retry) = rt.retry {
            deadline = deadline.min(retry);
        }
        if !rt.deferred_bulk.is_empty() {
            deadline = deadline.min(now.saturating_add(5));
        }
        if let Some(expires) = rt
            .files
            .pending_offers
            .values()
            .filter(|offer| !offer.accepting)
            .map(|offer| offer.expires_ms)
            .min()
        {
            deadline = deadline.min(expires);
        }
        let timeout = Duration::from_millis(deadline.saturating_sub(now).clamp(1, 2000));
        if let Err(e) = rt.driver.poll(Some(timeout), &mut rt.net_events) {
            rt.error("network_failed", e.to_string());
            break;
        }
        rt.process_net();
    }
    log::info!("shutdown: leave and goodbye");
    rt.enabled = false;
    rt.machine.shutdown(&mut rt.actions);
    rt.apply_actions();
    let flush = Instant::now();
    while flush.elapsed() < Duration::from_millis(250) {
        let _ = rt.driver.poll(Some(Duration::from_millis(10)), &mut rt.net_events);
        rt.net_events.clear();
    }
    rt.screens.prevent_sleep(false);
    rt.sink.release_all();
    rt.sink.set_cursor_visible(true);
    rt.execute(NetCommand::Shutdown);
    stop.store(true, Ordering::Relaxed);
    let _ = rt.capture.commands.send(CaptureCommand::Stop);
    let _ = rt.clipboard.commands.send(ClipboardCommand::Stop);
    let _ = rt.file_worker.commands.send(FileCommand::Stop);
    let _ = accept.join();
    let _ = rt.capture.thread.join();
    let _ = rt.clipboard.thread.join();
    let _ = rt.file_worker.thread.join();
    log::info!("daemon stopped");
    #[cfg(windows)]
    lifetime::SHUTDOWN_COMPLETE.store(true, Ordering::Release);
    Ok(0)
}
