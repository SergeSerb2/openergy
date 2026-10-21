use crate::{clipboard, common::*, layout::Rect, notify, Action, ClientEvent, CloseReason, ConnId};
use openergy_config::Config;
use openergy_platform::{ClipboardData, InputEvent, Modifiers};
use openergy_protocol::{ClipboardId, Message};
use std::sync::Arc;

pub struct Client {
    config: Arc<Config>,
    screen_name: String,
    peer: Option<(ConnId, Peer)>,
    now: u64,
    screen: Rect,
    cursor: (i32, i32),
    active: bool,
    ignore_mouse: bool,
    sequence: u32,
    clip_sequences: [u32; 2],
    local_owner: [bool; 2],
    ledger: Ledger,
}
impl Client {
    pub fn new(config: Arc<Config>, screen_name: String) -> Self {
        Self {
            config,
            screen_name,
            peer: None,
            now: 0,
            screen: Rect::default(),
            cursor: (0, 0),
            active: false,
            ignore_mouse: true,
            sequence: 0,
            clip_sequences: [0; 2],
            local_owner: [false; 2],
            ledger: Ledger::default(),
        }
    }
    pub fn next_deadline_ms(&self) -> u64 {
        self.peer
            .as_ref()
            .map(|(_, p)| p.next_deadline(self.now, false))
            .unwrap_or(self.now + 1000)
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn screen_name(&self) -> &str {
        &self.screen_name
    }
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            active_screen: self.active.then(|| self.screen_name.clone()),
            cursor: Some(self.cursor),
            peers: self.peer.iter().map(|(c, p)| p.snapshot(*c)).collect(),
            connected: self.peer.as_ref().is_some_and(|(_, p)| p.phase == Phase::Ready),
            remote_active: self.active,
        }
    }
    fn leave(&mut self, out: &mut Vec<Action>) {
        self.active = false;
        self.ledger.clear();
        out.push(Action::ReleaseAllLocal);
        out.push(Action::HideLocalCursor(false));
        if let Some((_, p)) = &mut self.peer {
            p.secure_input = None;
        }
    }
    fn disconnect(&mut self, out: &mut Vec<Action>) {
        self.leave(out);
        if let Some((c, _)) = self.peer.take() {
            close(c, CloseReason::Normal, out);
            notify("peer.disconnected", serde_json::json!({}), out);
        }
    }
    pub fn handle(&mut self, ev: ClientEvent, out: &mut Vec<Action>) {
        match ev {
            ClientEvent::ServerConnected { conn } => {
                self.leave(out);
                let mut p = Peer::new(self.now);
                p.name = "server".into();
                p.options.heartbeat = self.config.client.heartbeat_ms;
                self.peer = Some((conn, p));
                self.ignore_mouse = true;
            }
            ClientEvent::ServerMessage { conn, msg } => {
                if self.peer.as_ref().is_some_and(|(c, _)| *c == conn) {
                    self.message(msg, out);
                }
            }
            ClientEvent::ServerDisconnected { conn, .. } => {
                if self.peer.as_ref().is_some_and(|(c, _)| *c == conn) {
                    self.leave(out);
                    self.peer = None;
                    notify("peer.disconnected", serde_json::json!({}), out);
                }
            }
            ClientEvent::LocalScreen { rect, cursor } => {
                self.screen = rect;
                self.cursor = rect.clamp(cursor.0, cursor.1);
            }
            ClientEvent::DisplayChanged => {
                if let Some((c, p)) = &self.peer {
                    if p.phase != Phase::Hello {
                        send(*c, screen_info(self.screen, self.cursor), out);
                        self.ignore_mouse = true;
                    }
                }
            }
            ClientEvent::PermissionLost => {
                notice("permission_denied", "Input permission lost", out);
                self.disconnect(out);
            }
            ClientEvent::TimerTick { now_ms } => {
                self.now = now_ms;
                if let Some((c, p)) = &mut self.peer {
                    if p.tick(*c, now_ms, false, out) {
                        log::warn!("server heartbeat/setup timeout");
                        self.disconnect(out);
                    }
                }
            }
            ClientEvent::ConfigChanged(cfg) => {
                let was_active = self.active;
                self.leave(out);
                self.active = was_active;
                self.config = cfg;
                self.screen_name = self.config.name.clone();
                if let Some((c, p)) = &mut self.peer {
                    p.transfers = Default::default();
                    if p.phase == Phase::Ready {
                        send(*c, Message::ResetOptions, out);
                        send(
                            *c,
                            Message::SetOptions {
                                options: config_options(&self.config, false),
                            },
                            out,
                        );
                    }
                }
            }
            ClientEvent::LocalClipboardChanged { id } => {
                if id.0 < 2 && self.config.clipboard.enabled && self.config.clipboard.send {
                    if let Some((c, p)) = &mut self.peer {
                        if p.phase == Phase::Ready && p.version.minor >= 6 {
                            if let Some(v) = self.local_owner.get_mut(id.0 as usize) {
                                *v = true;
                            }
                            if let Some(v) = self.clip_sequences.get_mut(id.0 as usize) {
                                *v = self.sequence;
                            }
                            p.transfers.cancel(id);
                            send(
                                *c,
                                Message::ClipboardGrab {
                                    id,
                                    sequence: self.sequence,
                                },
                                out,
                            );
                            out.push(Action::ReadClipboard {
                                id,
                                max_bytes: self.config.clipboard.max_bytes.min(clipboard::HARD_CAP),
                            });
                        }
                    }
                }
            }
            ClientEvent::ClipboardPayload { id, data } => self.payload(id, data, out),
            ClientEvent::Shutdown => {
                self.leave(out);
                if let Some((c, _)) = &self.peer {
                    send(*c, Message::Bye, out);
                }
                self.peer = None;
            }
            ClientEvent::ReconnectTimer => {}
        }
    }
    fn message(&mut self, msg: Message, out: &mut Vec<Action>) {
        let Some((conn, p)) = &mut self.peer else { return };
        let c = *conn;
        p.last_rx = self.now;
        if let Message::Unknown { code, .. } = msg {
            p.unknown(code);
            return;
        }
        if matches!(
            msg,
            Message::Bye
                | Message::ErrorBusy
                | Message::ErrorBad
                | Message::ErrorUnknown
                | Message::ErrorIncompatible { .. }
        ) {
            if matches!(msg, Message::ErrorBusy) {
                log::warn!("server rejected duplicate name (EBSY); retry with backoff");
                notice("busy", "Server name is busy; reconnect will back off", out);
            } else if !matches!(msg, Message::Bye) {
                notice("protocol_rejected", "Server rejected session setup", out);
            }
            self.disconnect(out);
            return;
        }
        if p.phase == Phase::Hello {
            if let Message::Hello { product, version } = msg {
                if version.major != 1 || version.minor > 8 || !(product == "Synergy" || product == "Barrier") {
                    send(c, Message::ErrorIncompatible { major: 1, minor: 8 }, out);
                    self.disconnect(out);
                    return;
                }
                p.product = product.clone();
                p.version = version;
                p.phase = Phase::Info;
                send(
                    c,
                    Message::HelloBack {
                        product,
                        version,
                        name: self.screen_name.clone(),
                    },
                    out,
                );
            } else {
                self.disconnect(out);
            }
            return;
        }
        match msg {
            Message::QueryInfo => {
                send(c, screen_info(self.screen, self.cursor), out);
                self.ignore_mouse = true;
            }
            Message::InfoAck => self.ignore_mouse = false,
            Message::LanguageSync { languages } if p.phase == Phase::Info => {
                if languages.len() % 2 != 0 {
                    log::warn!("odd language list ignored");
                } else {
                    p.languages = languages.as_bytes().chunks_exact(2).map(|c| [c[0], c[1]]).collect();
                }
            }
            Message::KeepAlive => {
                p.alive(self.now);
                send(c, Message::KeepAlive, out);
                p.last_sent = self.now;
            }
            Message::ResetOptions => {
                p.options = Options::default();
                p.last_alive = self.now;
            }
            Message::SetOptions { options } => {
                p.options.apply(&options);
                p.last_alive = self.now;
                if p.phase != Phase::Ready {
                    p.phase = Phase::Ready;
                    send(
                        c,
                        Message::SetOptions {
                            options: config_options(&self.config, false),
                        },
                        out,
                    );
                    notify("peer.connected", serde_json::json!({"name":"server"}), out);
                }
            }
            Message::Enter {
                x,
                y,
                sequence,
                modifiers,
            } if p.phase == Phase::Ready => {
                self.ledger.clear();
                out.push(Action::ReleaseAllLocal);
                self.active = true;
                self.sequence = sequence;
                self.cursor = self.screen.clamp(i32::from(x), i32::from(y));
                self.ledger.modifiers = modifiers;
                out.push(Action::HideLocalCursor(false));
                out.push(Action::Inject(InputEvent::MouseMoveAbs {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }));
                for (mask, key, button) in [
                    (Modifiers::SHIFT, 0xefe1, 0x38),
                    (Modifiers::CONTROL, 0xefe3, 0x3b),
                    (Modifiers::ALT, 0xefe9, 0x3a),
                    (Modifiers::SUPER, 0xefeb, 0x37),
                ] {
                    if modifiers & mask != 0 {
                        let ev = InputEvent::KeyDown {
                            key,
                            modifiers: Modifiers(modifiers),
                            button,
                        };
                        self.ledger.observe(&ev);
                        out.push(Action::Inject(ev));
                    }
                }
            }
            Message::Leave => self.leave(out),
            Message::ClipboardGrab { id, sequence: _ } if id.0 < 2 => {
                if let Some(v) = self.local_owner.get_mut(id.0 as usize) {
                    *v = false;
                }
                if let Some(v) = self.clip_sequences.get_mut(id.0 as usize) {
                    *v = 0;
                }
                p.transfers.cancel(id);
            }
            Message::ClipboardData { id, sequence, chunk }
                if id.0 < 2 && self.config.clipboard.enabled && self.config.clipboard.receive =>
            {
                if self.clip_sequences.get(id.0 as usize).is_some_and(|s| sequence >= *s) {
                    match p
                        .transfers
                        .receive(id, sequence, chunk, self.config.clipboard.max_bytes, self.now)
                    {
                        Ok(Some(data)) => {
                            if let Some(v) = self.local_owner.get_mut(id.0 as usize) {
                                *v = false;
                            }
                            clipboard::transferred(&data, "in", out);
                            out.push(Action::SetClipboard { id, data });
                        }
                        Ok(None) => {}
                        Err(e) => log::warn!("discarded clipboard: {e:?}"),
                    }
                }
            }
            Message::SecureInputNotification { name } => {
                p.secure_input = (!name.is_empty()).then_some(name);
                log::info!("peer secure-input status changed");
            }
            Message::ScreenSaver { on } => notify("peer.screensaver", serde_json::json!({"on":on}), out),
            Message::FileTransfer { payload, .. } => p.decline(payload.len(), out),
            Message::DragInfo { payload, .. } => p.decline(payload.len(), out),
            other => {
                if self.active && p.phase == Phase::Ready {
                    if let Some(mut ev) = input_message(other, &p.options) {
                        match ev {
                            InputEvent::MouseMoveAbs { x, y } => {
                                if self.ignore_mouse {
                                    return;
                                }
                                self.cursor = self.screen.clamp(x, y);
                                ev = InputEvent::MouseMoveAbs {
                                    x: self.cursor.0,
                                    y: self.cursor.1,
                                };
                            }
                            InputEvent::MouseMoveRel { dx, dy } => {
                                if self.ignore_mouse {
                                    return;
                                }
                                self.cursor = self
                                    .screen
                                    .clamp(self.cursor.0.saturating_add(dx), self.cursor.1.saturating_add(dy));
                            }
                            _ => {}
                        }
                        self.ledger.observe(&ev);
                        out.push(Action::Inject(ev));
                    }
                }
            }
        }
    }
    fn payload(&mut self, id: ClipboardId, data: ClipboardData, out: &mut Vec<Action>) {
        if id.0 > 1 || !self.local_owner.get(id.0 as usize).copied().unwrap_or(false) {
            return;
        }
        let Some((_, p)) = &mut self.peer else { return };
        if p.phase != Phase::Ready || !p.options.clipboard {
            return;
        }
        let limit = p.options.limit.min(self.config.clipboard.max_bytes);
        match clipboard::marshal(&data, self.config.clipboard.max_bytes) {
            Ok(bytes) if bytes.len() < limit => {
                p.transfers.enqueue(id, self.sequence, Arc::new(bytes));
                clipboard::transferred(&data, "out", out);
            }
            Ok(bytes) => clipboard::blocked(bytes.len(), limit, &p.name, out),
            Err(_) => clipboard::blocked(data.total_bytes(), limit, &p.name, out),
        }
    }
}
