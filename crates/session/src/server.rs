use crate::{
    clipboard::{self},
    common::*,
    layout::{Layout, Rect},
    notify, Action, CloseReason, ConnId, ServerEvent,
};
use openergy_config::{Config, Role};
use openergy_platform::{ClipboardData, InputEvent};
use openergy_protocol::{ClipboardId, Message, ProtocolVersion};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Default)]
struct Slot {
    sequence: u32,
    owner: Option<ConnId>,
    data: Option<Arc<Vec<u8>>>,
    available: bool,
    readback: bool,
}
pub struct Server {
    config: Arc<Config>,
    peers: BTreeMap<ConnId, Peer>,
    metadata: BTreeMap<ConnId, Option<String>>,
    layout: Layout,
    primary: usize,
    active: usize,
    cursor: (i32, i32),
    last_motion: (i32, i32),
    now: u64,
    sequence: u32,
    local: Ledger,
    opts: Options,
    slots: [Slot; 2],
}
impl Server {
    pub fn new(config: Arc<Config>) -> Self {
        let primary = config.screens.iter().position(|s| s.name == config.name).unwrap_or(0);
        let layout = Layout::from_config(&config).unwrap_or_else(|e| {
            log::warn!("invalid layout: {e:?}");
            Layout::empty()
        });
        let cursor = layout.rect(primary).center();
        let mut opts = Options::default();
        opts.apply(&config_options(&config, true));
        Self {
            config,
            peers: BTreeMap::new(),
            metadata: BTreeMap::new(),
            layout,
            primary,
            active: primary,
            cursor,
            last_motion: (0, 0),
            now: 0,
            sequence: 0,
            local: Ledger::default(),
            opts,
            slots: [Slot::default(), Slot::default()],
        }
    }
    pub fn next_deadline_ms(&self) -> u64 {
        self.peers
            .values()
            .map(|p| p.next_deadline(self.now, true))
            .min()
            .unwrap_or(self.now + 1000)
            .min(self.layout.next_deadline(self.now))
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn remote_active(&self) -> bool {
        self.active != self.primary
    }
    pub fn is_connected(&self) -> bool {
        self.peers.values().any(|p| p.phase == Phase::Ready)
    }
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            active_screen: Some(self.name(self.active).into()),
            cursor: Some(self.cursor),
            peers: self.peers.iter().map(|(&c, p)| p.snapshot(c)).collect(),
            connected: self.peers.values().any(|p| p.phase == Phase::Ready),
            remote_active: self.active != self.primary,
        }
    }
    fn name(&self, s: usize) -> &str {
        self.config
            .screens
            .get(s)
            .map(|s| s.name.as_str())
            .unwrap_or(&self.config.name)
    }
    fn screen(&self, name: &str) -> Option<usize> {
        self.config
            .screens
            .iter()
            .position(|s| s.name == name || s.aliases.iter().any(|a| a == name))
    }
    fn connection(&self, s: usize) -> Option<ConnId> {
        let name = self.name(s);
        self.peers
            .iter()
            .find(|(_, p)| p.phase == Phase::Ready && p.name == name)
            .map(|(&c, _)| c)
    }
    fn leave(&mut self, out: &mut Vec<Action>) {
        if self.active != self.primary {
            if let Some(conn) = self.connection(self.active) {
                if let Some(p) = self.peers.get_mut(&conn) {
                    p.ledger.release_remote(conn, out);
                    p.secure_input = None;
                }
                send(conn, Message::Leave, out);
            }
        }
        self.active = self.primary;
        self.local.clear();
        self.layout.reset();
        self.cursor = self.layout.rect(self.primary).center();
        out.push(Action::ReleaseAllLocal);
        out.push(Action::HideLocalCursor(false));
        out.push(Action::WarpLocalCursor {
            x: self.cursor.0,
            y: self.cursor.1,
        });
    }
    fn remove(&mut self, conn: ConnId, out: &mut Vec<Action>) {
        if self.connection(self.active) == Some(conn) {
            self.leave(out);
        }
        if let Some(p) = self.peers.remove(&conn) {
            notify("peer.disconnected", serde_json::json!({"name":p.name}), out);
            for slot in &mut self.slots {
                if slot.owner == Some(conn) {
                    slot.owner = None;
                    slot.data = None;
                }
            }
        }
        self.metadata.remove(&conn);
    }
    fn fail(&mut self, conn: ConnId, msg: Message, out: &mut Vec<Action>) {
        send(conn, msg, out);
        self.remove(conn, out);
        close(conn, CloseReason::BadFrame, out);
    }
    pub fn handle(&mut self, ev: ServerEvent, out: &mut Vec<Action>) {
        match ev {
            ServerEvent::ClientConnected { conn } => {
                let mut p = Peer::new(self.now);
                p.fingerprint = self.metadata.remove(&conn).flatten();
                p.options.heartbeat = self.config.server.heartbeat_ms;
                self.peers.insert(conn, p);
                send(
                    conn,
                    Message::Hello {
                        product: if self.config.options.barrier {
                            "Barrier"
                        } else {
                            "Synergy"
                        }
                        .into(),
                        version: ProtocolVersion {
                            major: 1,
                            minor: if self.config.options.barrier { 6 } else { 8 },
                        },
                    },
                    out,
                );
            }
            ServerEvent::ConnectionMetadata { conn, fingerprint } => {
                if let Some(p) = self.peers.get_mut(&conn) {
                    p.fingerprint = fingerprint;
                } else {
                    self.metadata.insert(conn, fingerprint);
                }
            }
            ServerEvent::ClientMessage { conn, msg } => self.message(conn, msg, out),
            ServerEvent::ClientDisconnected { conn, .. } => self.remove(conn, out),
            ServerEvent::LocalInput(ev) => self.input(ev, out),
            ServerEvent::LocalScreen { rect, cursor } => {
                if self.layout.rect(self.primary) != rect {
                    self.layout.set_rect(self.primary, rect);
                }
                if self.active == self.primary {
                    self.cursor = rect.clamp(cursor.0, cursor.1);
                }
            }
            ServerEvent::DisplayChanged => {
                self.leave(out);
            }
            ServerEvent::LocalClipboardChanged { id } => {
                if id.0 < 2 && self.config.clipboard.enabled && self.config.clipboard.send {
                    if let Some(slot) = self.slots.get_mut(id.0 as usize) {
                        slot.sequence = self.sequence;
                        slot.owner = None;
                        slot.data = None;
                        slot.available = false;
                        slot.readback = false;
                    }
                    for p in self.peers.values_mut() {
                        p.transfers.cancel(id);
                    }
                    out.push(Action::ReadClipboard {
                        id,
                        max_bytes: self.config.clipboard.max_bytes.min(clipboard::HARD_CAP),
                    });
                }
            }
            ServerEvent::ClipboardPayload { id, data } => self.payload(None, id, data, out),
            ServerEvent::TimerTick { now_ms } => {
                self.now = now_ms;
                let dead = self
                    .peers
                    .iter_mut()
                    .filter_map(|(&c, p)| p.tick(c, now_ms, true, out).then_some(c))
                    .collect::<Vec<_>>();
                // Transfers own their shared blobs until End is queued. Keep
                // only the platform's current clipboard after the final sender.
                for slot in &mut self.slots {
                    if slot.data.as_ref().is_some_and(|data| Arc::strong_count(data) == 1) {
                        slot.data = None;
                    }
                }
                for c in dead {
                    log::warn!("peer heartbeat/setup timeout");
                    self.remove(c, out);
                    close(c, CloseReason::Normal, out);
                }
                if self.last_motion != (0, 0) && self.layout.waiting() {
                    self.motion(
                        self.cursor.0,
                        self.cursor.1,
                        self.last_motion.0,
                        self.last_motion.1,
                        out,
                    );
                }
            }
            ServerEvent::ConfigChanged(cfg) => {
                self.leave(out);
                self.config = cfg;
                self.primary = self
                    .config
                    .screens
                    .iter()
                    .position(|s| s.name == self.config.name)
                    .unwrap_or(0);
                self.active = self.primary;
                self.layout = Layout::from_config(&self.config).unwrap_or_else(|_| Layout::empty());
                self.opts = Options::default();
                self.opts.apply(&config_options(&self.config, true));
                let invalid = self
                    .peers
                    .iter()
                    .filter_map(|(&c, p)| self.screen(&p.name).is_none().then_some(c))
                    .collect::<Vec<_>>();
                for c in invalid {
                    self.remove(c, out);
                    close(c, CloseReason::Normal, out);
                }
                for (&c, p) in &mut self.peers {
                    if p.phase == Phase::Ready {
                        send(c, Message::ResetOptions, out);
                        send(
                            c,
                            Message::SetOptions {
                                options: screen_options(&self.config, &p.name),
                            },
                            out,
                        );
                    }
                }
                if self.config.role != Role::Server {
                    self.shutdown(out);
                }
            }
            ServerEvent::CaptureOverflow => self.leave(out),
            ServerEvent::PermissionLost => {
                self.leave(out);
                notice("permission_denied", "Input permission lost", out);
            }
            ServerEvent::LocalSecureInput { app } => {
                if let Some(name) = app {
                    let name = name
                        .chars()
                        .scan(0usize, |len, c| {
                            *len += c.len_utf8();
                            (*len <= 255).then_some(c)
                        })
                        .collect::<String>();
                    for (&conn, p) in &self.peers {
                        if p.phase == Phase::Ready && p.version.minor >= 7 {
                            send(conn, Message::SecureInputNotification { name: name.clone() }, out);
                        }
                    }
                }
            }
            ServerEvent::LocalScreenSaver { on } => {
                for (&conn, p) in &self.peers {
                    if p.phase == Phase::Ready {
                        send(conn, Message::ScreenSaver { on }, out);
                    }
                }
            }
            ServerEvent::Shutdown => self.shutdown(out),
        }
    }
    fn shutdown(&mut self, out: &mut Vec<Action>) {
        self.leave(out);
        for &c in self.peers.keys() {
            send(c, Message::Bye, out);
        }
        self.peers.clear();
    }
    fn message(&mut self, conn: ConnId, msg: Message, out: &mut Vec<Action>) {
        let Some(p) = self.peers.get_mut(&conn) else { return };
        p.last_rx = self.now;
        if let Message::Unknown { code, .. } = msg {
            p.unknown(code);
            return;
        }
        if matches!(msg, Message::Bye) {
            log::info!("peer goodbye");
            self.remove(conn, out);
            close(conn, CloseReason::Normal, out);
            return;
        }
        if p.phase == Phase::Hello {
            if let Message::HelloBack { product, version, name } = msg {
                if version.major != 1 || version.minor > 8 {
                    self.fail(conn, Message::ErrorIncompatible { major: 1, minor: 8 }, out);
                    return;
                }
                p.product = product;
                p.version = version;
                p.name = name;
                p.phase = Phase::Info;
                send(conn, Message::QueryInfo, out);
                if version.minor >= 8 {
                    send(
                        conn,
                        Message::LanguageSync {
                            languages: String::new(),
                        },
                        out,
                    );
                }
            } else {
                self.fail(conn, Message::ErrorBad, out);
            }
            return;
        }
        if let Message::ScreenInfo { x, y, w, h, .. } = msg {
            if w <= 0 || h <= 0 {
                self.fail(conn, Message::ErrorBad, out);
                return;
            }
            p.screen = Rect {
                x: i32::from(x),
                y: i32::from(y),
                w: i32::from(w),
                h: i32::from(h),
            };
            send(conn, Message::InfoAck, out);
            if p.phase == Phase::Info {
                let name = p.name.clone();
                let fingerprint = p.fingerprint.clone();
                let Some(screen) = self.screen(&name) else {
                    self.fail(conn, Message::ErrorUnknown, out);
                    return;
                };
                let canonical = self.name(screen).to_string();
                let existing = self
                    .peers
                    .iter()
                    .find(|(c, p)| **c != conn && p.phase == Phase::Ready && p.name == canonical)
                    .map(|(&c, p)| (c, p.last_rx, p.options.heartbeat, p.fingerprint.clone()));
                if let Some((old, last, hb, fp)) = existing {
                    if (hb > 0 && self.now.saturating_sub(last) >= hb)
                        || fingerprint.as_ref().is_some_and(|n| Some(n) == fp.as_ref())
                    {
                        self.remove(old, out);
                        close(old, CloseReason::Replaced, out);
                        out.push(Action::ReplaceConnection { old, new: conn });
                    } else {
                        log::warn!("healthy duplicate name rejected (EBSY)");
                        self.fail(conn, Message::ErrorBusy, out);
                        return;
                    }
                }
                if let Some(p) = self.peers.get_mut(&conn) {
                    p.name = canonical.clone();
                    p.phase = Phase::Ready;
                    self.layout.set_rect(screen, p.screen);
                }
                send(conn, Message::ResetOptions, out);
                send(
                    conn,
                    Message::SetOptions {
                        options: screen_options(&self.config, &canonical),
                    },
                    out,
                );
                notify("peer.connected", serde_json::json!({"name":canonical}), out);
                self.push_slots(conn, out);
            } else if let Some(screen) = self.peers.get(&conn).and_then(|p| self.screen(&p.name)) {
                if let Some(p) = self.peers.get(&conn) {
                    self.layout.set_rect(screen, p.screen);
                }
            }
            return;
        }
        let Some(p) = self.peers.get_mut(&conn) else { return };
        match msg {
            Message::KeepAlive => {
                p.alive(self.now);
            }
            Message::SetOptions { options } => {
                p.options.apply(&options);
            }
            Message::ResetOptions => {
                p.options = Options::default();
                p.last_alive = self.now;
            }
            Message::ClipboardGrab { id, sequence } if id.0 == 0 && self.config.clipboard.enabled => {
                if let Some(slot) = self.slots.get_mut(id.0 as usize) {
                    if sequence >= slot.sequence {
                        slot.sequence = sequence;
                        slot.owner = Some(conn);
                        slot.data = None;
                        slot.available = false;
                        slot.readback = false;
                        for peer in self.peers.values_mut() {
                            peer.transfers.cancel(id);
                        }
                    }
                }
            }
            Message::ClipboardData { id, sequence, chunk }
                if id.0 == 0 && self.config.clipboard.enabled && self.config.clipboard.receive =>
            {
                let valid = self
                    .slots
                    .get(id.0 as usize)
                    .is_some_and(|s| s.owner == Some(conn) && sequence >= s.sequence);
                if valid {
                    let declared = if let openergy_protocol::Chunk::Start { total_len } = &chunk {
                        Some(*total_len as usize)
                    } else {
                        None
                    };
                    match p
                        .transfers
                        .receive(id, sequence, chunk, self.config.clipboard.max_bytes, self.now)
                    {
                        Ok(Some(data)) => self.payload(Some(conn), id, data, out),
                        Ok(None) => {}
                        Err(clipboard::ClipboardError::TooLarge) => clipboard::blocked(
                            declared.unwrap_or(0),
                            self.config.clipboard.max_bytes.min(clipboard::HARD_CAP),
                            &p.name,
                            out,
                        ),
                        Err(e) => {
                            log::warn!("discarded clipboard: {e:?}");
                        }
                    }
                }
            }
            Message::SecureInputNotification { name } => p.secure_input = (!name.is_empty()).then_some(name),
            Message::FileTransfer { payload, .. } => p.decline(payload.len(), out),
            Message::DragInfo { payload, .. } => p.decline(payload.len(), out),
            Message::Noop | Message::ScreenSaver { .. } | Message::LanguageSync { .. } => {}
            _ => {}
        }
    }
    fn input(&mut self, ev: InputEvent, out: &mut Vec<Action>) {
        let hotkey = if let InputEvent::KeyDown { key, modifiers, .. } = &ev {
            self.config.hotkeys.lock_to_screen.as_deref().is_some_and(|chord| {
                let mut mask = 0;
                let mut code = None;
                for part in chord.split('+').map(str::trim) {
                    match part.to_ascii_lowercase().as_str() {
                        "ctrl" | "control" => mask |= 2,
                        "alt" => mask |= 4,
                        "shift" => mask |= 1,
                        "super" | "command" | "cmd" => mask |= 16,
                        k => {
                            let mut chars = k.chars();
                            if let (Some(c), None) = (chars.next(), chars.next()) {
                                code = Some(c as u16);
                            }
                        }
                    }
                }
                code == Some(*key) && modifiers.0 & 0x3f == mask
            })
        } else {
            false
        };
        if (matches!(ev, InputEvent::KeyDown { key: 0xef14, .. }) || hotkey) && !self.opts.disable_lock {
            self.opts.locked = !self.opts.locked;
            self.layout.reset();
        }
        if hotkey {
            return;
        }
        self.local.observe(&ev);
        match ev {
            InputEvent::MouseMoveAbs { x, y } => {
                let dx = x.saturating_sub(self.cursor.0);
                let dy = y.saturating_sub(self.cursor.1);
                self.motion(x, y, dx, dy, out);
            }
            InputEvent::MouseMoveRel { dx, dy } => {
                if self.active != self.primary && self.opts.locked && !self.opts.disable_lock && self.opts.relative {
                    if let Some(c) = self.connection(self.active) {
                        send(
                            c,
                            Message::MouseMoveRel {
                                dx: i16c(dx),
                                dy: i16c(dy),
                            },
                            out,
                        );
                    }
                } else {
                    self.motion(
                        self.cursor.0.saturating_add(dx),
                        self.cursor.1.saturating_add(dy),
                        dx,
                        dy,
                        out,
                    );
                }
            }
            _ => {
                if let Some(c) = self.connection(self.active) {
                    if let Some(p) = self.peers.get_mut(&c) {
                        p.ledger.observe(&ev);
                        send(c, wire_input(&ev, p.version.minor, &self.config), out);
                    }
                }
            }
        }
    }
    fn motion(&mut self, x: i32, y: i32, dx: i32, dy: i32, out: &mut Vec<Action>) {
        self.last_motion = (dx, dy);
        let can_switch = self.local.buttons.is_empty()
            && (!self.opts.locked || self.opts.disable_lock)
            && (self.local.modifiers & self.opts.needs) == self.opts.needs;
        if can_switch {
            if let Some((target, tx, ty)) = self.layout.switch_target(self.active, x, y, dx, dy, self.now) {
                if target == self.primary || self.connection(target).is_some() {
                    if let Some(c) = self.connection(self.active) {
                        if let Some(p) = self.peers.get_mut(&c) {
                            p.ledger.release_remote(c, out);
                        }
                        send(c, Message::Leave, out);
                    } else {
                        out.push(Action::ReleaseAllLocal);
                    }
                    self.active = target;
                    self.cursor = (tx, ty);
                    self.sequence = self.sequence.wrapping_add(1);
                    self.last_motion = (0, 0);
                    if let Some(c) = self.connection(target) {
                        send(
                            c,
                            Message::Enter {
                                x: i16c(tx),
                                y: i16c(ty),
                                sequence: self.sequence,
                                modifiers: self.local.modifiers,
                            },
                            out,
                        );
                        out.push(Action::HideLocalCursor(true));
                        self.push_slots(c, out);
                    } else {
                        out.push(Action::HideLocalCursor(false));
                        out.push(Action::WarpLocalCursor { x: tx, y: ty });
                    }
                    return;
                }
            }
        } else {
            self.layout.reset();
        }
        self.cursor = self.layout.rect(self.active).clamp(x, y);
        if let Some(c) = self.connection(self.active) {
            send(
                c,
                Message::MouseMoveAbs {
                    x: i16c(self.cursor.0),
                    y: i16c(self.cursor.1),
                },
                out,
            );
        }
    }
    fn push_slots(&mut self, conn: ConnId, out: &mut Vec<Action>) {
        let Some(p) = self.peers.get_mut(&conn) else { return };
        if p.version.minor < 6 || !p.options.clipboard || !self.config.clipboard.send {
            return;
        }
        for (i, s) in self.slots.iter_mut().enumerate() {
            if s.owner != Some(conn) {
                if let Some(bytes) = &s.data {
                    let limit = p.options.limit.min(self.config.clipboard.max_bytes);
                    if bytes.len() > limit {
                        clipboard::blocked(bytes.len(), limit, &p.name, out);
                    } else {
                        send(
                            conn,
                            Message::ClipboardGrab {
                                id: ClipboardId(i as u8),
                                sequence: 0,
                            },
                            out,
                        );
                        p.transfers.enqueue(ClipboardId(i as u8), 0, bytes.clone());
                    }
                } else if s.available && !s.readback {
                    s.readback = true;
                    out.push(Action::ReadClipboard {
                        id: ClipboardId(i as u8),
                        max_bytes: self.config.clipboard.max_bytes.min(clipboard::HARD_CAP),
                    });
                }
            }
        }
    }
    fn payload(&mut self, owner: Option<ConnId>, id: ClipboardId, data: ClipboardData, out: &mut Vec<Action>) {
        let Some(slot) = self.slots.get_mut(id.0 as usize) else {
            return;
        };
        let readback = owner.is_none() && slot.readback;
        let owner = if readback { slot.owner } else { owner };
        slot.readback = false;
        if slot.owner != owner {
            return;
        }
        match clipboard::marshal(&data, self.config.clipboard.max_bytes) {
            Ok(bytes) => {
                slot.data = Some(Arc::new(bytes));
                slot.available = true;
            }
            Err(_) => {
                clipboard::blocked(data.total_bytes(), self.config.clipboard.max_bytes, "local", out);
                return;
            }
        }
        if owner.is_some() && !readback {
            clipboard::transferred(&data, "in", out);
            out.push(Action::SetClipboard { id, data });
        } else {
            clipboard::transferred(&data, "out", out);
        }
        let conns = self
            .peers
            .keys()
            .copied()
            .filter(|c| Some(*c) != owner)
            .collect::<Vec<_>>();
        for c in conns {
            self.push_slots(c, out);
        }
    }
}
