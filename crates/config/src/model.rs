use crate::*;
use serde_json::Value;
use std::collections::BTreeSet;

fn invalid(message: impl Into<String>) -> ConfigError {
    ConfigError::Validation(message.into())
}

impl Config {
    pub fn load_toml(text: &str) -> Result<Self, ConfigError> {
        let cfg: Self = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn to_toml(&self) -> String {
        // Every model field has a TOML representation; optional fields are omitted.
        toml::to_string_pretty(self).unwrap_or_default()
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let cfg: Self = serde_json::from_value(value.clone()).map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(invalid(format!("unsupported config version {}", self.version)));
        }
        if !self.name.is_empty() && !valid_name(&self.name) {
            return Err(invalid("name: invalid screen name"));
        }
        if self.role != Role::Off && self.name.is_empty() {
            return Err(invalid("name: required when role is server or client"));
        }
        if self.server.port == 0 {
            return Err(invalid("server.port: must be in 1..=65535"));
        }
        address_port(&self.server.listen).map_err(|e| invalid(format!("server.listen: {e}")))?;
        if !self.client.remote.is_empty() {
            address_port(&self.client.remote).map_err(|e| invalid(format!("client.remote: {e}")))?;
        }
        for (key, value) in [
            ("server.heartbeat_ms", self.server.heartbeat_ms),
            ("client.heartbeat_ms", self.client.heartbeat_ms),
            ("switch.delay_ms", self.switch.delay_ms),
            ("switch.double_tap_ms", self.switch.double_tap_ms),
        ] {
            if value > i32::MAX as u64 {
                return Err(invalid(format!("{key}: exceeds signed 32-bit wire range")));
            }
        }
        if self.clipboard.max_bytes as u128 > i32::MAX as u128 * 1024 {
            return Err(invalid("clipboard.max_bytes: exceeds signed 32-bit KiB wire range"));
        }
        if !self.input.scroll_sensitivity.is_finite() || self.input.scroll_sensitivity <= 0.0 {
            return Err(invalid("input.scroll_sensitivity: must be finite and positive"));
        }
        check_corner_size(self.switch.corner_size_px)?;
        if let Some(stroke) = &self.hotkeys.lock_to_screen {
            validate_stroke(stroke)?;
        }
        let mut names = BTreeSet::new();
        let canonical: BTreeSet<&str> = self.screens.iter().map(|s| s.name.as_str()).collect();
        for screen in &self.screens {
            for name in std::iter::once(&screen.name).chain(&screen.aliases) {
                if !valid_name(name) || !names.insert(name) {
                    return Err(invalid(format!("screen: duplicate or invalid name {name:?}")));
                }
            }
            if let Some(rect) = screen.rect {
                if ![rect.x, rect.y, rect.w, rect.h].iter().all(|v| v.is_finite())
                    || rect.x < 0.0
                    || rect.y < 0.0
                    || rect.w <= 0.0
                    || rect.h <= 0.0
                    || rect.x + rect.w > i32::MAX as f64
                    || rect.y + rect.h > i32::MAX as f64
                {
                    return Err(invalid(format!(
                        "screen {:?}.rect: need nonnegative finite origins and positive sizes within i32 bounds",
                        screen.name
                    )));
                }
            }
            if let Some(options) = &screen.options {
                if let Some(size) = options.corner_size_px {
                    check_corner_size(size)?;
                }
            }
            if let Some(remap) = &screen.modifiers {
                for target in [
                    &remap.shift,
                    &remap.ctrl,
                    &remap.alt,
                    &remap.altgr,
                    &remap.meta,
                    &remap.super_,
                ]
                .into_iter()
                .flatten()
                {
                    if !["none", "shift", "ctrl", "control", "alt", "altgr", "meta", "super"]
                        .contains(&target.to_ascii_lowercase().as_str())
                    {
                        return Err(invalid(format!("screen {:?}.modifiers: invalid target", screen.name)));
                    }
                }
            }
            if let Some(links) = &screen.links {
                for edge in EDGES {
                    if let Some(target) = direct(links, edge) {
                        if !target.is_empty() && !canonical.contains(target) {
                            return Err(invalid(format!(
                                "screen {:?}.links: unknown target {target:?}",
                                screen.name
                            )));
                        }
                        if !target.is_empty() && links.ranges.iter().any(|l| l.edge == edge) {
                            return Err(invalid("links: full-edge and ranged overrides cannot share an edge"));
                        }
                    }
                }
                for link in &links.ranges {
                    if !canonical.contains(link.to.as_str()) {
                        return Err(invalid(format!(
                            "screen {:?}.links.ranges: unknown target",
                            screen.name
                        )));
                    }
                    check_range(link.from_range)?;
                    check_range(link.to_range)?;
                }
            }
        }
        if self.role == Role::Server && !self.screens.is_empty() && !canonical.contains(self.name.as_str()) {
            return Err(invalid("name: server must be a canonical screen in screen[]"));
        }
        for (i, a) in self.screens.iter().enumerate() {
            for b in &self.screens[i + 1..] {
                if let (Some(a_rect), Some(b_rect)) = (a.rect, b.rect) {
                    if overlaps(a_rect, b_rect) {
                        return Err(invalid(format!("screen rects overlap: {:?} and {:?}", a.name, b.name)));
                    }
                }
            }
        }
        let links = derive_links(&self.screens);
        for (i, a) in links.iter().enumerate() {
            check_range(a.from_range)?;
            check_range(a.to_range)?;
            for b in &links[i + 1..] {
                if a.from == b.from
                    && a.edge == b.edge
                    && a.from_range.0 < b.from_range.1
                    && b.from_range.0 < a.from_range.1
                {
                    return Err(invalid(format!(
                        "links: overlapping source ranges for {:?} {:?}",
                        a.from, a.edge
                    )));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn warnings(&self) -> Vec<String> {
        let mut warnings = crate::layout::warnings(&self.screens);
        if let Ok(Some(port)) = address_port(&self.server.listen) {
            if port != self.server.port {
                warnings.push("server.port overrides the port in server.listen".into());
            }
        }
        if !self.tls.enabled {
            warnings.push("TLS is disabled; traffic is plaintext".into());
        }
        if self.server.heartbeat_ms == 0 || self.client.heartbeat_ms == 0 {
            warnings.push("A zero heartbeat disables liveness timers".into());
        }
        if self.clipboard.max_bytes % 1024 != 0 {
            warnings.push("clipboard.max_bytes is rounded down to whole KiB for Synergy peers".into());
        }
        if self.role == Role::Client && self.client.remote.is_empty() {
            warnings.push("client.remote is empty; choose a server before connecting".into());
        }
        warnings
    }
}

fn check_corner_size(size: u32) -> Result<(), ConfigError> {
    if size > i32::MAX as u32 {
        return Err(invalid("corner_size_px: exceeds signed 32-bit wire range"));
    }
    Ok(())
}

pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

// Interface key names from docs/re/03-core-protocol.md Appendix A, not reference code.
const KEY_NAMES: &str = "AltGr Alt_L Alt_R AppMail AppMedia AppUser1 AppUser2 AudioDown AudioMute AudioNext AudioPlay AudioPrev AudioStop AudioUp BackSpace Begin Break Cancel CapsLock Clear Control_L Control_R Delete Down Eject End Escape Execute F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 F20 F21 F22 F23 F24 F25 F26 F27 F28 F29 F30 F31 F32 F33 F34 F35 Find Help Henkan Home Hyper_L Hyper_R Insert KP_0 KP_1 KP_2 KP_3 KP_4 KP_5 KP_6 KP_7 KP_8 KP_9 KP_Add KP_Begin KP_Decimal KP_Delete KP_Divide KP_Down KP_End KP_Enter KP_Equal KP_F1 KP_F2 KP_F3 KP_F4 KP_Home KP_Insert KP_Left KP_Multiply KP_PageDown KP_PageUp KP_Right KP_Separator KP_Space KP_Subtract KP_Tab KP_Up Left LeftTab Linefeed Menu Meta_L Meta_R NumLock PageDown PageUp Pause Print Redo Return Right ScrollLock Select ShiftLock Shift_L Shift_R Sleep Super_L Super_R SysReq Tab Undo Up WWWBack WWWFavorites WWWForward WWWHome WWWRefresh WWWSearch WWWStop Zenkaku Space Exclaim DoubleQuote Number Dollar Percent Ampersand Apostrophe ParenthesisL ParenthesisR Asterisk Plus Comma Minus Period Slash Colon Semicolon Less Equal Greater Question At BracketL Backslash BracketR Circumflex Underscore Grave BraceL Bar BraceR Tilde";
pub(crate) fn canonical_key(key: &str) -> Option<&str> {
    if key.chars().count() == 1 && key.chars().all(|c| c as u32 <= u16::MAX as u32 && !c.is_control()) {
        return Some(key);
    }
    if key.len() == 6 && key.starts_with("\\u") && key.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit) {
        return Some(key);
    }
    KEY_NAMES.split_whitespace().find(|name| name.eq_ignore_ascii_case(key))
}

pub(crate) fn validate_stroke(stroke: &str) -> Result<(), ConfigError> {
    let mouse = stroke.starts_with("mousebutton(");
    let stroke = if mouse {
        stroke
            .strip_prefix("mousebutton(")
            .and_then(|s| s.strip_suffix(')'))
            .ok_or_else(|| invalid("hotkeys.lock_to_screen: invalid mouse trigger"))?
    } else {
        stroke
    };
    if stroke.is_empty()
        || stroke.split('+').any(str::is_empty)
        || stroke
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || "#=(),;:".contains(c))
    {
        return Err(invalid("hotkeys.lock_to_screen: invalid stroke"));
    }
    let mut parts = stroke.split('+').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_some() {
            if !["ctrl", "control", "shift", "alt", "meta", "super", "altgr"]
                .contains(&part.to_ascii_lowercase().as_str())
            {
                return Err(invalid("hotkeys.lock_to_screen: unknown modifier"));
            }
        } else if !mouse && canonical_key(part).is_none() {
            return Err(invalid("hotkeys.lock_to_screen: unknown key name"));
        } else if mouse && part.parse::<u8>().ok().filter(|v| *v != 0).is_none() {
            return Err(invalid("hotkeys.lock_to_screen: mouse button must be in 1..=255"));
        }
    }
    Ok(())
}

pub(crate) fn check_range((start, end): (u8, u8)) -> Result<(), ConfigError> {
    if start >= end || end > 100 {
        return Err(invalid("links: require 0 <= start < end <= 100"));
    }
    Ok(())
}

/// Parse syntax only. This never resolves a host or opens a network socket.
pub(crate) fn address_port(address: &str) -> Result<Option<u16>, String> {
    if address.is_empty()
        || address
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "#=;\"\\".contains(c))
    {
        return Err("invalid or empty address".into());
    }
    let port = if let Some(rest) = address.strip_prefix('[') {
        let (host, suffix) = rest.split_once(']').ok_or("unclosed IPv6 address")?;
        host.parse::<std::net::Ipv6Addr>().map_err(|_| "invalid IPv6 address")?;
        if suffix.is_empty() {
            None
        } else {
            Some(suffix.strip_prefix(':').ok_or("invalid IPv6 port")?)
        }
    } else if address.parse::<std::net::Ipv6Addr>().is_ok() {
        None
    } else if let Some((host, port)) = address.split_once(':') {
        if host.is_empty() || !host.chars().all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)) {
            return Err("invalid host".into());
        }
        Some(port)
    } else {
        if !address.chars().all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)) {
            return Err("invalid host".into());
        }
        None
    };
    port.map(|p| {
        p.parse::<u16>()
            .ok()
            .filter(|n| *n != 0)
            .ok_or_else(|| "port must be in 1..=65535".into())
    })
    .transpose()
}

/// RFC 7386: an object patch first replaces a nonobject target with {}.
pub fn merge_patch(target: &mut Value, patch: &Value) {
    if let Value::Object(patch) = patch {
        if !target.is_object() {
            *target = Value::Object(Default::default());
        }
        if let Value::Object(target) = target {
            for (key, value) in patch {
                if value.is_null() {
                    target.remove(key);
                } else {
                    merge_patch(target.entry(key.clone()).or_insert(Value::Null), value);
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}

pub fn apply_merge_patch(cfg: &Config, patch: &Value) -> Result<(Config, Vec<String>), ConfigError> {
    let mut value = cfg.to_json();
    merge_patch(&mut value, patch);
    let next = Config::from_json(&value)?;
    let warnings = next.warnings();
    Ok((next, warnings))
}

pub(crate) const EDGES: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];
pub(crate) fn direct(links: &LinksCfg, edge: Edge) -> Option<&str> {
    match edge {
        Edge::Left => &links.left,
        Edge::Right => &links.right,
        Edge::Top => &links.top,
        Edge::Bottom => &links.bottom,
    }
    .as_deref()
}
pub(crate) fn overlaps(a: LayoutRect, b: LayoutRect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}
