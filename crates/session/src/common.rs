use crate::{clipboard::Transfers, layout::Rect, notify, Action, CloseReason, ConnId};
use openergy_config::{Config, Corner};
use openergy_platform::{InputEvent, Modifiers};
use openergy_protocol::{options as o, ClipboardId, Message, ProtocolVersion};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct PeerSnapshot {
    pub conn: ConnId,
    pub name: String,
    pub connected: bool,
    pub product: String,
    pub version: ProtocolVersion,
    pub screen: Rect,
    pub rtt_ms: Option<f64>,
    pub secure_input_app: Option<String>,
    pub languages: Vec<String>,
    pub active_language: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct SessionSnapshot {
    pub active_screen: Option<String>,
    pub cursor: Option<(i32, i32)>,
    pub peers: Vec<PeerSnapshot>,
    pub connected: bool,
    pub remote_active: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Hello,
    Info,
    Ready,
}
#[derive(Default, Clone)]
pub(crate) struct Ledger {
    keys: BTreeMap<(u16, u16), (u16, u16)>,
    pub buttons: BTreeSet<u8>,
    pub modifiers: u16,
}
impl Ledger {
    fn key_id(key: u16, button: u16) -> (u16, u16) {
        // Button zero has no physical identity; distinguish these keys by KeyID.
        (button, if button == 0 { key } else { 0 })
    }
    pub fn has_key(&self, key: u16, button: u16) -> bool {
        self.keys.contains_key(&Self::key_id(key, button))
    }
    pub fn observe(&mut self, ev: &InputEvent) {
        match *ev {
            InputEvent::KeyDown { key, modifiers, button } => {
                self.keys.insert(Self::key_id(key, button), (key, modifiers.0));
                self.modifiers = modifiers.0;
            }
            InputEvent::KeyRepeat { modifiers, .. } => self.modifiers = modifiers.0,
            InputEvent::KeyUp { key, modifiers, button } => {
                self.keys.remove(&Self::key_id(key, button));
                self.modifiers = modifiers.0;
            }
            InputEvent::MouseDown { button } => {
                self.buttons.insert(button);
            }
            InputEvent::MouseUp { button } => {
                self.buttons.remove(&button);
            }
            _ => {}
        }
    }
    pub fn release_remote(&mut self, conn: ConnId, out: &mut Vec<Action>) {
        for ((button, _), (key, modifiers)) in std::mem::take(&mut self.keys) {
            out.push(Action::SendControl {
                conn,
                msg: Message::KeyUp { key, modifiers, button },
            });
        }
        for button in std::mem::take(&mut self.buttons) {
            out.push(Action::SendControl {
                conn,
                msg: Message::MouseUp { button },
            });
        }
        self.modifiers = 0;
    }
    pub fn clear(&mut self) {
        self.keys.clear();
        self.buttons.clear();
        self.modifiers = 0;
    }
}
#[derive(Clone)]
pub(crate) struct Options {
    pub raw: BTreeMap<u32, i32>,
    pub heartbeat: u64,
    pub clipboard: bool,
    pub limit: usize,
    pub relative: bool,
    pub locked: bool,
    pub disable_lock: bool,
    pub needs: u16,
    /// Per-lock half-duplex flags for the receiving screen, in Caps, Num,
    /// Scroll order (DSOP HDCL/HDNL/HDSL).
    pub half_duplex: [bool; 3],
}
impl Default for Options {
    fn default() -> Self {
        Self {
            raw: BTreeMap::new(),
            heartbeat: 3000,
            clipboard: true,
            limit: i32::MAX as usize * 1024,
            relative: false,
            locked: false,
            disable_lock: false,
            needs: 0,
            half_duplex: [false; 3],
        }
    }
}
impl Options {
    pub fn apply(&mut self, pairs: &[(u32, i32)]) {
        let half_duplex_before = self.half_duplex;
        for &(id, v) in pairs {
            self.raw.insert(id, v);
            match id {
                o::HEARTBEAT => self.heartbeat = (v.max(0) as u64).min(3_600_000),
                o::CLIPBOARD_SHARING => self.clipboard = v != 0,
                o::CLIPBOARD_SHARING_SIZE => self.limit = (v.max(0) as usize).saturating_mul(1024),
                o::RELATIVE_MOUSE_MOVES => self.relative = v != 0,
                o::DEFAULT_LOCK_TO_SCREEN_STATE => self.locked = v != 0,
                o::DISABLE_LOCK_TO_SCREEN => self.disable_lock = v != 0,
                o::HALF_DUPLEX_CAPS_LOCK => self.half_duplex[0] = v != 0,
                o::HALF_DUPLEX_NUM_LOCK => self.half_duplex[1] = v != 0,
                o::HALF_DUPLEX_SCROLL_LOCK => self.half_duplex[2] = v != 0,
                o::SWITCH_NEEDS_SHIFT | o::SWITCH_NEEDS_CONTROL | o::SWITCH_NEEDS_ALT => {
                    let mask = match id {
                        o::SWITCH_NEEDS_SHIFT => Modifiers::SHIFT,
                        o::SWITCH_NEEDS_CONTROL => Modifiers::CONTROL,
                        _ => Modifiers::ALT,
                    };
                    if v != 0 {
                        self.needs |= mask;
                    } else {
                        self.needs &= !mask;
                    }
                }
                _ => {}
            }
        }
        if self.half_duplex != half_duplex_before {
            log::info!(
                "half-duplex locks: caps={} num={} scroll={}",
                self.half_duplex[0],
                self.half_duplex[1],
                self.half_duplex[2]
            );
        }
    }
    /// True when `key` is a lock key whose receiving screen is half-duplex:
    /// Caps Lock, Num Lock, or Scroll Lock with its flag set.
    pub fn is_half_duplex_lock(&self, key: u16) -> bool {
        match key {
            0xefe5 => self.half_duplex[0],
            0xef7f => self.half_duplex[1],
            0xef14 => self.half_duplex[2],
            _ => false,
        }
    }
    /// Target modifier (1 Shift .. 6 AltGr) for a source modifier, from the
    /// screen's modifier map. Identity when unmapped; None when mapped to
    /// "none", which disables that modifier.
    fn modifier_target(&self, source: u8) -> Option<u8> {
        let id = match source {
            1 => o::MOD_SHIFT,
            2 => o::MOD_CTRL,
            3 => o::MOD_ALT,
            4 => o::MOD_META,
            5 => o::MOD_SUPER,
            _ => o::MOD_ALTGR,
        };
        match self.raw.get(&id) {
            None => Some(source),
            Some(t) if (1..=6).contains(t) => Some(*t as u8),
            Some(_) => None,
        }
    }
    /// Modifier keys follow the same map as the modifier mask, as in the
    /// reference client: with "ctrl = super", Control_L arrives as Super_L.
    /// The left or right side is kept; AltGr has one side. None drops a
    /// modifier key that maps to "none".
    pub fn mapped_key(&self, key: u16) -> Option<u16> {
        let (source, side) = match key {
            0xefe1 => (1, 0),
            0xefe2 => (1, 1),
            0xefe3 => (2, 0),
            0xefe4 => (2, 1),
            0xefe9 => (3, 0),
            0xefea => (3, 1),
            0xefe7 => (4, 0),
            0xefe8 => (4, 1),
            0xefeb => (5, 0),
            0xefec => (5, 1),
            0xef7e => (6, 0),
            _ => return Some(key),
        };
        Some(match (self.modifier_target(source)?, side) {
            (1, s) => 0xefe1 + s,
            (2, s) => 0xefe3 + s,
            (3, s) => 0xefe9 + s,
            (4, s) => 0xefe7 + s,
            (5, s) => 0xefeb + s,
            _ => 0xef7e,
        })
    }
    pub fn mapped_mask(&self, mask: u16) -> u16 {
        let mut result = mask & !0x3f;
        for (bit, id) in [
            (1, o::MOD_SHIFT),
            (2, o::MOD_CTRL),
            (4, o::MOD_ALT),
            (8, o::MOD_META),
            (16, o::MOD_SUPER),
            (32, o::MOD_ALTGR),
        ] {
            if mask & bit != 0 {
                let target = self.raw.get(&id).copied().unwrap_or(match bit {
                    1 => 1,
                    2 => 2,
                    4 => 3,
                    8 => 4,
                    16 => 5,
                    _ => 6,
                });
                if (1..=6).contains(&target) {
                    result |= 1u16 << (target - 1);
                }
            }
        }
        result
    }
}
#[derive(Clone)]
pub(crate) struct Peer {
    pub name: String,
    pub product: String,
    pub version: ProtocolVersion,
    pub phase: Phase,
    pub screen: Rect,
    pub fingerprint: Option<String>,
    pub since: u64,
    pub last_rx: u64,
    pub last_alive: u64,
    pub last_sent: u64,
    pub ping: Option<u64>,
    pub rtt: Option<f64>,
    pub options: Options,
    pub ledger: Ledger,
    pub transfers: Transfers,
    pub unknown: BTreeSet<[u8; 4]>,
    pub secure_input: Option<String>,
    pub warned: bool,
    pub languages: Vec<[u8; 2]>,
    pub active_language: Option<[u8; 2]>,
    pub declined: bool,
    unknown_saturated: bool,
    pub language_warned: bool,
    language_logged: bool,
    /// This peer sent a CCLP for clipboard id 1 on this connection, which
    /// marks it PRIMARY-capable for the server relay rule.
    pub sent_primary_grab: bool,
    pub primary_drop_logged: bool,
}
impl Peer {
    pub fn new(now: u64) -> Self {
        Self {
            name: String::new(),
            product: "Synergy".into(),
            version: ProtocolVersion::V1_8,
            phase: Phase::Hello,
            screen: Rect::default(),
            fingerprint: None,
            since: now,
            last_rx: now,
            last_alive: now,
            last_sent: now,
            ping: None,
            rtt: None,
            options: Options::default(),
            ledger: Ledger::default(),
            transfers: Transfers::default(),
            unknown: BTreeSet::new(),
            secure_input: None,
            warned: false,
            languages: Vec::new(),
            active_language: None,
            declined: false,
            unknown_saturated: false,
            language_warned: false,
            language_logged: false,
            sent_primary_grab: false,
            primary_drop_logged: false,
        }
    }
    pub fn next_deadline(&self, now: u64, server: bool) -> u64 {
        if self.transfers.pending() {
            return self.transfers.deadline(now);
        }
        if self.phase != Phase::Ready {
            return self.since.saturating_add(2001).max(now.saturating_add(100));
        }
        if self.options.heartbeat == 0 {
            return now.saturating_add(1000);
        }
        let last = if server { self.last_rx } else { self.last_alive };
        let dead = last.saturating_add(self.options.heartbeat.saturating_mul(3));
        if server {
            self.last_sent.saturating_add(self.options.heartbeat).min(dead)
        } else {
            dead
        }
    }
    pub fn snapshot(&self, conn: ConnId) -> PeerSnapshot {
        PeerSnapshot {
            conn,
            name: self.name.clone(),
            connected: self.phase == Phase::Ready,
            product: self.product.clone(),
            version: self.version,
            screen: self.screen,
            rtt_ms: self.rtt,
            secure_input_app: self.secure_input.clone(),
            languages: self
                .languages
                .iter()
                .map(|l| String::from_utf8_lossy(l).into_owned())
                .collect(),
            active_language: self.active_language.map(|l| String::from_utf8_lossy(&l).into_owned()),
        }
    }
    pub fn alive(&mut self, now: u64) {
        self.last_alive = now;
        if let Some(sent) = self.ping.take() {
            self.rtt = Some(now.saturating_sub(sent) as f64);
        }
    }
    /// PRIMARY traffic arrives where it cannot be applied (flag off or a
    /// single-clipboard receiver). Drop it; note it once per connection.
    pub fn drop_primary(&mut self) {
        if !self.primary_drop_logged {
            self.primary_drop_logged = true;
            log::info!("PRIMARY selection (clipboard id 1) dropped; sharing is off or unsupported here");
        }
    }
    pub fn decline(&mut self, bytes: usize, out: &mut Vec<Action>) {
        if !self.declined {
            self.declined = true;
            log::warn!("file/drag transfer declined: bytes={bytes}");
            notice("transfer_unsupported", "File and drag transfer are disabled", out);
        }
    }
    pub fn observe_language(
        &mut self,
        message: &Message,
        enabled: bool,
        local: &openergy_platform::KeyboardLanguages,
        out: &mut Vec<Action>,
    ) {
        let lang = match message {
            Message::KeyDown { lang: Some(lang), .. } if self.version.minor >= 8 => lang,
            Message::KeyRepeat { lang, .. } => lang,
            _ => return,
        };
        if lang.is_empty() {
            return;
        }
        if !enabled {
            if !self.language_logged {
                self.language_logged = true;
                log::info!("peer keyboard language ignored; input.sync_language is off");
            }
            return;
        }
        let Some(code) = openergy_platform::keyboard_language(lang) else {
            if !self.language_warned {
                self.language_warned = true;
                log::info!("invalid peer keyboard language ignored");
            }
            return;
        };
        if self.active_language == Some(code) {
            return;
        }
        self.active_language = Some(code);
        if !local.available.is_empty() && !local.available.contains(&code) {
            notify(
                openergy_ipc::events::LANGUAGE_SYNC,
                serde_json::json!({"language": String::from_utf8_lossy(&code), "result": "no_match"}),
                out,
            );
            if !self.language_logged {
                self.language_logged = true;
                log::info!("peer keyboard language has no installed layout match");
            }
        } else {
            out.push(Action::SetKeyboardLanguage(code));
        }
    }
    pub fn unknown(&mut self, code: [u8; 4]) {
        if self.unknown.contains(&code) {
            return;
        }
        if self.unknown.len() < 64 {
            self.unknown.insert(code);
            log::warn!("skipping unknown frame code={code:02x?}");
        } else if !self.unknown_saturated {
            self.unknown_saturated = true;
            log::warn!("unknown-code log limit reached; further unknown frames are skipped");
        }
    }
    pub fn tick(&mut self, conn: ConnId, now: u64, server: bool, out: &mut Vec<Action>) -> bool {
        if self.phase != Phase::Ready {
            if now.saturating_sub(self.since) > 2000 && !self.warned {
                self.warned = true;
                log::warn!("slow session setup");
            }
            return now.saturating_sub(self.since) >= 10_000;
        }
        let hb = self.options.heartbeat;
        let last = if server { self.last_rx } else { self.last_alive };
        if hb > 0 && now.saturating_sub(last) >= hb.saturating_mul(3) {
            return true;
        }
        if server && hb > 0 && now.saturating_sub(self.last_sent) >= hb {
            send(conn, Message::KeepAlive, out);
            self.last_sent = now;
            if server {
                self.ping.get_or_insert(now);
            }
        }
        self.transfers.tick(conn, now, out);
        false
    }
}
/// Clipboard id 1 (PRIMARY) moves only under the opt-in config flag;
/// id 0 is always allowed by this check (enabled/send/receive gate it).
pub(crate) fn primary_allowed(cfg: &Config, id: ClipboardId) -> bool {
    id.0 == 0 || cfg.clipboard.share_primary_selection
}
pub(crate) fn send(conn: ConnId, msg: Message, out: &mut Vec<Action>) {
    out.push(Action::Send { conn, msg });
}
pub(crate) fn close(conn: ConnId, reason: CloseReason, out: &mut Vec<Action>) {
    out.push(Action::Close { conn, reason });
}
pub(crate) fn notice(code: &str, message: &str, out: &mut Vec<Action>) {
    notify(
        "error",
        serde_json::json!({"code":code,"message":message,"unix_ms":null}),
        out,
    );
}
pub(crate) fn i16c(v: i32) -> i16 {
    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}
pub(crate) fn screen_info(r: Rect, cursor: (i32, i32)) -> Message {
    Message::ScreenInfo {
        x: i16c(r.x),
        y: i16c(r.y),
        w: i16c(r.w.max(1)),
        h: i16c(r.h.max(1)),
        warp: 0,
        mx: i16c(cursor.0),
        my: i16c(cursor.1),
    }
}
pub(crate) fn config_options(cfg: &Config, server: bool) -> Vec<(u32, i32)> {
    let mask = cfg.switch.dead_corners.iter().fold(0, |m, c| {
        m | match c {
            Corner::TopLeft => 1,
            Corner::TopRight => 2,
            Corner::BottomLeft => 4,
            Corner::BottomRight => 8,
        }
    });
    vec![
        (
            o::DEFAULT_LOCK_TO_SCREEN_STATE,
            i32::from(cfg.options.default_lock_to_screen),
        ),
        (o::DISABLE_LOCK_TO_SCREEN, i32::from(cfg.options.disable_lock_to_screen)),
        (o::SWITCH_NEEDS_SHIFT, i32::from(cfg.options.switch_needs_shift)),
        (o::SWITCH_NEEDS_CONTROL, i32::from(cfg.options.switch_needs_control)),
        (o::SWITCH_NEEDS_ALT, i32::from(cfg.options.switch_needs_alt)),
        (o::WIN32_KEEP_FOREGROUND, i32::from(cfg.options.win32_keep_foreground)),
        (
            o::HEARTBEAT,
            if server {
                cfg.server.heartbeat_ms
            } else {
                cfg.client.heartbeat_ms
            }
            .min(i32::MAX as u64) as i32,
        ),
        (o::CLIPBOARD_SHARING, i32::from(cfg.clipboard.enabled)),
        (
            o::CLIPBOARD_SHARING_SIZE,
            (cfg.clipboard.max_bytes / 1024).min(i32::MAX as usize) as i32,
        ),
        (o::RELATIVE_MOUSE_MOVES, i32::from(cfg.input.relative_mouse)),
        (o::SWITCH_CORNERS, mask),
        (
            o::SWITCH_CORNER_SIZE,
            cfg.switch.corner_size_px.min(i32::MAX as u32) as i32,
        ),
        (o::SWITCH_DELAY, cfg.switch.delay_ms.min(i32::MAX as u64) as i32),
        (
            o::SWITCH_DOUBLE_TAP,
            cfg.switch.double_tap_ms.min(i32::MAX as u64) as i32,
        ),
    ]
}
pub(crate) fn wire_input(ev: &InputEvent, minor: u16, lang: Option<&str>) -> Message {
    match *ev {
        InputEvent::KeyDown { key, modifiers, button } => Message::KeyDown {
            key,
            modifiers: modifiers.0,
            button,
            lang: lang.filter(|_| minor >= 8).map(str::to_owned),
        },
        InputEvent::KeyRepeat {
            key,
            modifiers,
            count,
            button,
        } => Message::KeyRepeat {
            key,
            modifiers: modifiers.0,
            count: count.max(0) as u16,
            button,
            lang: lang.unwrap_or_default().into(),
        },
        InputEvent::KeyUp { key, modifiers, button } => Message::KeyUp {
            key,
            modifiers: modifiers.0,
            button,
        },
        InputEvent::MouseDown { button } => Message::MouseDown { button },
        InputEvent::MouseUp { button } => Message::MouseUp { button },
        InputEvent::MouseMoveAbs { x, y } => Message::MouseMoveAbs { x: i16c(x), y: i16c(y) },
        InputEvent::MouseMoveRel { dx, dy } => Message::MouseMoveRel {
            dx: i16c(dx),
            dy: i16c(dy),
        },
        InputEvent::MouseWheel { dx, dy } => Message::MouseWheel {
            dx: i16c(dx),
            dy: i16c(dy),
        },
    }
}
pub(crate) fn input_message(msg: Message, opts: &Options) -> Option<InputEvent> {
    Some(match msg {
        Message::KeyDown {
            key, modifiers, button, ..
        } => InputEvent::KeyDown {
            key: opts.mapped_key(key)?,
            modifiers: Modifiers(opts.mapped_mask(modifiers)),
            button,
        },
        Message::KeyRepeat {
            key,
            modifiers,
            count,
            button,
            ..
        } => InputEvent::KeyRepeat {
            key: opts.mapped_key(key)?,
            modifiers: Modifiers(opts.mapped_mask(modifiers)),
            count: count.min(i16::MAX as u16) as i16,
            button,
        },
        Message::KeyUp { key, modifiers, button } => InputEvent::KeyUp {
            key: opts.mapped_key(key)?,
            modifiers: Modifiers(opts.mapped_mask(modifiers)),
            button,
        },
        Message::MouseDown { button } if button > 0 => InputEvent::MouseDown { button },
        Message::MouseUp { button } if button > 0 => InputEvent::MouseUp { button },
        Message::MouseMoveAbs { x, y } => InputEvent::MouseMoveAbs {
            x: i32::from(x),
            y: i32::from(y),
        },
        Message::MouseMoveRel { dx, dy } => InputEvent::MouseMoveRel {
            dx: i32::from(dx),
            dy: i32::from(dy),
        },
        Message::MouseWheel { dx, dy } => InputEvent::MouseWheel {
            dx: i32::from(dx),
            dy: i32::from(dy),
        },
        _ => return None,
    })
}

pub(crate) fn screen_options(cfg: &Config, screen_name: &str) -> Vec<(u32, i32)> {
    let mut options = config_options(cfg, true);
    if let Some(screen) = cfg.screens.iter().find(|s| s.name == screen_name) {
        if screen.os.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("windows")) {
            options.push((o::WIN32_KEEP_FOREGROUND, 1));
        }
        if let Some(opts) = &screen.options {
            for (id, v) in [
                (o::HALF_DUPLEX_CAPS_LOCK, opts.half_duplex_caps_lock),
                (o::HALF_DUPLEX_NUM_LOCK, opts.half_duplex_num_lock),
                (o::HALF_DUPLEX_SCROLL_LOCK, opts.half_duplex_scroll_lock),
                (o::XTEST_XINERAMA_UNAWARE, opts.xtest_is_xinerama_unaware),
                (o::PRESERVE_FOCUS, opts.preserve_focus),
            ] {
                if let Some(v) = v {
                    options.push((id, i32::from(v)));
                }
            }
            if let Some(corners) = &opts.dead_corners {
                let mask = corners.iter().fold(0, |mask, c| {
                    mask | match c {
                        Corner::TopLeft => 1,
                        Corner::TopRight => 2,
                        Corner::BottomLeft => 4,
                        Corner::BottomRight => 8,
                    }
                });
                options.push((o::SWITCH_CORNERS, mask));
            }
            if let Some(size) = opts.corner_size_px {
                options.push((o::SWITCH_CORNER_SIZE, size.min(i32::MAX as u32) as i32));
            }
        }
        if let Some(remap) = &screen.modifiers {
            for (id, value) in [
                (o::MOD_SHIFT, &remap.shift),
                (o::MOD_CTRL, &remap.ctrl),
                (o::MOD_ALT, &remap.alt),
                (o::MOD_ALTGR, &remap.altgr),
                (o::MOD_META, &remap.meta),
                (o::MOD_SUPER, &remap.super_),
            ] {
                if let Some(value) = value {
                    let target = match value.to_ascii_lowercase().as_str() {
                        "none" => 0,
                        "shift" => 1,
                        "ctrl" | "control" => 2,
                        "alt" => 3,
                        "meta" => 4,
                        "super" | "command" => 5,
                        "altgr" => 6,
                        _ => continue,
                    };
                    options.push((id, target));
                }
            }
        }
    }
    options
}
