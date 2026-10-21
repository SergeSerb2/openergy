use crate::{
    adapter,
    events::{EventBus, EventQueue},
    ipc_server, lifetime,
    logger::{unix_ms, Logger},
    workers::{self, CaptureCommand, ClipboardCommand, ClipboardEvent},
    VERSION,
};
use openergy_config::{Config, Role};
use openergy_ipc::{self as ipc, methods as m, Event, IpcError, Request, Response, Status};
use openergy_net::{Fingerprint, NetCommand, NetDriver, NetEvent, Priority};
use openergy_platform::{ClipboardData, Grant, InputEvent, InputSink, Permissions, ScreenRect, ScreenSystem};
use openergy_protocol::{ClipboardId, Message};
use openergy_session::{layout::Rect, Action, Client, ClientEvent, ConnId, Server, ServerEvent, SessionSnapshot};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
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
    fn geometry(&mut self, rect: Rect, cursor: (i32, i32), changed: bool, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => {
                s.handle(ServerEvent::LocalScreen { rect, cursor }, out);
                if changed {
                    s.handle(ServerEvent::DisplayChanged, out);
                }
            }
            Self::Client(c) => {
                c.handle(ClientEvent::LocalScreen { rect, cursor }, out);
                if changed {
                    c.handle(ClientEvent::DisplayChanged, out);
                }
            }
            _ => {}
        }
    }
    fn clipboard_changed(&mut self, out: &mut Vec<Action>) {
        match self {
            Self::Server(s) => s.handle(
                ServerEvent::LocalClipboardChanged {
                    id: ClipboardId::CLIPBOARD,
                },
                out,
            ),
            Self::Client(c) => c.handle(
                ClientEvent::LocalClipboardChanged {
                    id: ClipboardId::CLIPBOARD,
                },
                out,
            ),
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
    suspended: bool,
    resume_enabled: bool,
    close_at: Option<u64>,
    capture: workers::Capture,
    clipboard: workers::ClipboardWorker,
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
}
impl Runtime {
    fn now(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
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
    fn publish(&self, event: &str, data: Value) {
        self.bus.publish(Event {
            event: event.into(),
            data,
        });
    }
    fn execute(&mut self, cmd: NetCommand) {
        let send = match &cmd {
            NetCommand::Send { conn, msg, priority } => Some((
                *conn,
                matches!(msg, Message::KeyUp { .. } | Message::MouseUp { .. }),
                if let Message::ClipboardData { id, .. } = msg {
                    Some(*id)
                } else {
                    None
                },
                (*priority == Priority::Input).then(|| msg.clone()),
            )),
            _ => None,
        };
        if let Err(error) = self.driver.execute_checked(cmd, &mut self.net_events) {
            self.error("network_queue_full", format!("Can't send data: {error}."));
            if let Some((conn, lost_up, clipboard, input)) = send {
                if let Some(id) = clipboard {
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
                } else if lost_up {
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
        s.clipboard.last_transfer = self.last_transfer.clone();
        s.clipboard.last_blocked = self.last_blocked.clone();
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
                    } => self.execute(NetCommand::Send {
                        conn,
                        msg: Message::ClipboardData { id, sequence, chunk },
                        priority: Priority::Bulk,
                    }),
                    Action::Close { conn, .. } => self.execute(NetCommand::Close { conn }),
                    Action::ReplaceConnection { .. } => {}
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
                            self.error("clipboard_busy", "Clipboard worker queue is full");
                        }
                    }
                    Action::SetClipboard { id: _, data } => {
                        if self
                            .clipboard
                            .commands
                            .try_send(ClipboardCommand::Write { data })
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
                        match event.event.as_str() {
                            "clipboard.blocked" => {
                                self.last_blocked = serde_json::from_value(event.data.clone()).ok();
                            }
                            "clipboard.transfer" => {
                                self.last_transfer = serde_json::from_value(event.data.clone()).ok();
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
                                        "protocol_error" => format!("{server} rejected the connection protocol."),
                                        _ => error.message.clone(),
                                    };
                                }
                                self.last_error = error;
                            }
                            _ => {}
                        }
                        self.bus.publish(event);
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
        if !self.enabled || self.suspended {
            return;
        }
        self.events_in += 1;
        if let Machine::Server(s) = &mut self.machine {
            s.handle(ServerEvent::LocalInput(event), &mut self.actions);
        }
    }
    fn notice(&mut self, notice: adapter::Notice) {
        use adapter::Notice;
        match notice {
            Notice::Sleep | Notice::Lock => {
                if self.suspended {
                    return;
                }
                self.suspended = true;
                self.resume_enabled = self.enabled;
                self.enabled = false;
                self.retry = None;
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::StopListening);
                self.close_at = Some(self.now() + 250);
                self.state = "idle".into();
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
                let _ = self.clipboard.commands.try_send(ClipboardCommand::Enabled(false));
            }
            Notice::Wake | Notice::Unlock => {
                if self.suspended {
                    self.suspended = false;
                    self.displays = self.screens.screens();
                    self.permissions = self.screens.permissions();
                    self.local_secure = self.screens.secure_input_app();
                    self.enabled = self.resume_enabled;
                    self.close_at = None;
                    self.execute(NetCommand::Shutdown);
                    self.process_net();
                    self.attempt = 0;
                    self.retry = None;
                    self.machine = Machine::new(Arc::new(self.cfg.clone()));
                    self.machine.geometry(
                        display_rect(&self.displays),
                        self.screens.cursor_pos(),
                        false,
                        &mut self.actions,
                    );
                    self.start_connection();
                    let _ = self
                        .capture
                        .commands
                        .try_send(CaptureCommand::Enable(self.enabled && self.cfg.role == Role::Server));
                    let _ = self
                        .clipboard
                        .commands
                        .try_send(ClipboardCommand::Enabled(self.enabled && self.cfg.clipboard.enabled));
                } else if self.cfg.role == Role::Client && self.retry.is_some() {
                    self.attempt = 0;
                    self.retry = Some(self.now());
                }
            }
            Notice::Displays => self.display_changed.store(true, Ordering::Release),
            Notice::Permission | Notice::CaptureStopped | Notice::CaptureError => {
                if self.suspended || !self.enabled || self.cfg.role != Role::Server {
                    return;
                }
                if let Machine::Server(s) = &mut self.machine {
                    s.handle(ServerEvent::CaptureOverflow, &mut self.actions);
                }
                self.error("capture_unavailable", "Platform capture stopped or lost permission");
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
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
        let base = 250u64
            .saturating_mul(1u64 << self.attempt.saturating_sub(1).min(7))
            .min(30_000);
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
        // Connecting is idempotent, including a TLS connection waiting for approval.
        if (self.cfg.role == Role::Client
            && (matches!(self.state.as_str(), "connecting" | "connected") || !self.pending.is_empty()))
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
                        match &mut self.machine {
                            Machine::Server(s) => {
                                s.handle(
                                    ServerEvent::ConnectionMetadata {
                                        conn,
                                        fingerprint: fingerprint.map(|f| f.to_colon_hex()),
                                    },
                                    &mut self.actions,
                                );
                                s.handle(ServerEvent::ClientConnected { conn }, &mut self.actions);
                            }
                            Machine::Client(c) => c.handle(ClientEvent::ServerConnected { conn }, &mut self.actions),
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
            self.state = "connected".into();
            self.last_error = None;
        } else if self.cfg.role == Role::Server && self.enabled && self.state == "connected" {
            self.state = "listening".into();
        }
    }
    fn refresh_health(&mut self) {
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
        self.machine.geometry(
            display_rect(&self.displays),
            self.screens.cursor_pos(),
            changed,
            &mut self.actions,
        );
        let permissions = self.screens.permissions();
        self.sink.permissions_changed(&permissions);
        if permissions != self.permissions {
            self.status_dirty = true;
            let denied = matches!(permissions.accessibility, Grant::Denied)
                || matches!(permissions.input_monitoring, Grant::Denied)
                || matches!(permissions.post_events, Grant::Denied);
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
        if self.capture.overflow.swap(false, Ordering::Relaxed) {
            if let Machine::Server(s) = &mut self.machine {
                s.handle(ServerEvent::CaptureOverflow, &mut self.actions);
            }
            self.error("capture_overflow", "Capture channel full; event passed to the local OS");
        }
    }
    fn apply_config(&mut self, mut cfg: Config) -> Result<(), String> {
        normalize(&mut cfg);
        cfg.validate().map_err(|e| e.to_string())?;
        let reconnect = cfg.role != self.cfg.role
            || cfg.server != self.cfg.server
            || cfg.client.remote != self.cfg.client.remote
            || cfg.tls != self.cfg.tls
            || cfg.name != self.cfg.name;
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
                    || write_config(&self.path, &cfg).map_err(openergy_net::NetError::Failed),
                )
                .map_err(|error| error.to_string())?;
            self.machine.shutdown(&mut self.actions);
            self.apply_actions();
            self.execute(NetCommand::DisconnectAll);
            self.process_net();
            self.connections.clear();
            self.pending.clear();
            self.cfg = cfg;
            self.machine = Machine::new(Arc::new(self.cfg.clone()));
            self.enabled = self.cfg.role != Role::Off && (self.cfg.role != Role::Client || self.cfg.client.autoconnect);
            self.attempt = 0;
            self.retry = None;
            self.healthy_since = None;
            self.state = "idle".into();
            self.machine.geometry(
                display_rect(&self.displays),
                self.screens.cursor_pos(),
                false,
                &mut self.actions,
            );
            self.start_connection();
        } else {
            // Session-only changes cannot fail after strict config validation.
            // Preserve the live machine until the atomic save succeeds.
            write_config(&self.path, &cfg)?;
            self.cfg = cfg;
            let cfg = Arc::new(self.cfg.clone());
            match &mut self.machine {
                Machine::Server(s) => s.handle(ServerEvent::ConfigChanged(cfg), &mut self.actions),
                Machine::Client(c) => c.handle(ClientEvent::ConfigChanged(cfg), &mut self.actions),
                Machine::Off => {}
            }
        }
        self.machine.geometry(
            display_rect(&self.displays),
            self.screens.cursor_pos(),
            false,
            &mut self.actions,
        );
        let _ = self
            .capture
            .commands
            .try_send(CaptureCommand::Enable(self.enabled && self.cfg.role == Role::Server));
        let _ = self
            .clipboard
            .commands
            .try_send(ClipboardCommand::Enabled(self.enabled && self.cfg.clipboard.enabled));
        self.update_sleep_assertion();
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
                self.apply_config(cfg)
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
                self.apply_config(cfg)
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
                    self.apply_config(cfg)
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
                    if let Some(text) = req.params.get("fingerprint") {
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
                        self.enabled = false;
                        self.machine.shutdown(&mut self.actions);
                        self.apply_actions();
                        self.execute(NetCommand::Shutdown);
                        self.process_net();
                        self.state = "idle".into();
                    }
                    self.apply_config(cfg).map_err(|e| IpcError::new("connect_failed", e))?;
                }
                self.enabled = true;
                self.retry = None;
                self.start_connection();
                let _ = self
                    .capture
                    .commands
                    .try_send(CaptureCommand::Enable(self.cfg.role == Role::Server));
                let _ = self
                    .clipboard
                    .commands
                    .try_send(ClipboardCommand::Enabled(self.cfg.clipboard.enabled));
                Ok(json!({"connecting":true}))
            }
            m::CONNECTION_DISCONNECT => {
                self.enabled = false;
                self.retry = None;
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::Shutdown);
                self.state = "idle".into();
                self.connections.clear();
                self.pending.clear();
                let _ = self.capture.commands.try_send(CaptureCommand::Enable(false));
                let _ = self.clipboard.commands.try_send(ClipboardCommand::Enabled(false));
                Ok(json!({"disconnected":true}))
            }
            m::CONNECTION_RESTART => {
                self.enabled = false;
                self.machine.shutdown(&mut self.actions);
                self.apply_actions();
                self.execute(NetCommand::Shutdown);
                self.process_net();
                self.machine = Machine::new(Arc::new(self.cfg.clone()));
                self.machine.geometry(
                    display_rect(&self.displays),
                    self.screens.cursor_pos(),
                    false,
                    &mut self.actions,
                );
                self.enabled = true;
                self.retry = None;
                self.start_connection();
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
                                store.trust_client(fp)
                            } else {
                                store.trust_server(fp)
                            };
                            result.map_err(|e| IpcError::new("trust_failed", e.to_string()))?;
                        }
                    }
                    m::TRUST_REJECT => self.driver.reject_held(&fp, &mut self.net_events),
                    _ => {
                        self.driver
                            .trust_store_mut()
                            .revoke(&fp)
                            .map_err(|e| IpcError::new("trust_failed", e.to_string()))?;
                        self.driver.reject_held(&fp, &mut self.net_events);
                        let conns = self
                            .connections
                            .iter()
                            .filter_map(|(&c, info)| {
                                (info.fingerprint.as_deref() == Some(&fp.to_colon_hex())).then_some(c)
                            })
                            .collect::<Vec<_>>();
                        for conn in conns {
                            self.execute(NetCommand::Close { conn });
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
        if result.is_ok() && matches!(method, m::VIRTUAL_SET_SCREENS | m::VIRTUAL_SET_PERMISSIONS) {
            self.refresh_health();
        }
        result
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
    let mut name = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_default();
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

fn write_config(path: &Path, cfg: &Config) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&tmp).map_err(|e| e.to_string())?;
        file.write_all(cfg.to_toml().as_bytes()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
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
) -> Result<i32, String> {
    #[cfg(not(unix))]
    {
        let _ = (socket_path, config, config_path, platform, parent_pid);
        Err("named-pipe IPC is unavailable in this build".into())
    }
    #[cfg(unix)]
    {
        normalize(&mut config);
        let path = config_path.unwrap_or_else(crate::default_config_path);
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
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
            suspended: false,
            resume_enabled: false,
            close_at: None,
            capture,
            clipboard,
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
        };
        lifetime::install_signals();
        let parent = lifetime::ParentWatch::new(parent_pid);
        rt.machine.geometry(
            display_rect(&rt.displays),
            rt.screens.cursor_pos(),
            false,
            &mut rt.actions,
        );
        let _ = rt
            .capture
            .commands
            .try_send(CaptureCommand::Enable(enabled && rt.cfg.role == Role::Server));
        let _ = rt
            .clipboard
            .commands
            .try_send(ClipboardCommand::Enabled(enabled && rt.cfg.clipboard.enabled));
        rt.update_sleep_assertion();
        rt.start_connection();
        log::info!("openergy {VERSION} daemon listening on {}", socket_path.display());
        while !stop.load(Ordering::Relaxed) && !lifetime::SIGNALLED.load(Ordering::Relaxed) && !parent.exited() {
            for notice in (rt.notices)() {
                rt.notice(notice);
            }
            if rt.close_at.is_some_and(|deadline| rt.now() >= deadline) {
                rt.close_at = None;
                rt.execute(NetCommand::Shutdown);
                rt.process_net();
            }
            while let Ok(packet) = rt.capture.events.try_recv() {
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
            }
            while let Ok(event) = rt.clipboard.events.try_recv() {
                match event {
                    ClipboardEvent::Changed => rt.machine.clipboard_changed(&mut rt.actions),
                    ClipboardEvent::Payload { id, data } => rt.machine.payload(id, data, &mut rt.actions),
                    ClipboardEvent::Failure(error) => rt.error("clipboard_failed", error),
                    ClipboardEvent::Blocked { bytes, limit } => {
                        let blocked = ipc::ClipboardBlocked {
                            bytes: Some(bytes),
                            limit: Some(limit),
                            peer: Some("local".into()),
                            unix_ms: Some(unix_ms()),
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
            rt.machine.tick(now, &mut rt.actions);
            rt.apply_actions();
            rt.process_net();
            let health_interval = if rt.permissions.accessibility != Grant::Granted
                || rt.permissions.input_monitoring != Grant::Granted
                || rt.permissions.post_events != Grant::Granted
            {
                250
            } else {
                2000
            };
            if rt.display_changed.swap(false, Ordering::AcqRel) || now.saturating_sub(rt.last_health) >= health_interval
            {
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
                if status != rt.last_status {
                    rt.last_status = status.clone();
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
        let _ = std::os::unix::net::UnixStream::connect(&socket_path);
        let _ = rt.capture.commands.send(CaptureCommand::Stop);
        let _ = rt.clipboard.commands.send(ClipboardCommand::Stop);
        let _ = accept.join();
        let _ = rt.capture.thread.join();
        let _ = rt.clipboard.thread.join();
        log::info!("daemon stopped");
        Ok(0)
    }
}
