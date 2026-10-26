use crate::{
    model::{address_port, check_range, valid_name, validate_stroke},
    *,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

pub(crate) fn parse_error(line: usize, message: impl AsRef<str>) -> ConfigError {
    ConfigError::Parse(format!("synergy.conf line {line}: {}", message.as_ref()))
}
pub(crate) fn blank_screen(name: String) -> ScreenCfg {
    ScreenCfg {
        name,
        rect: None,
        links: None,
        modifiers: None,
        options: None,
        os: None,
        aliases: Vec::new(),
    }
}
pub(crate) fn disconnected() -> LinksCfg {
    LinksCfg {
        left: Some(String::new()),
        right: Some(String::new()),
        top: Some(String::new()),
        bottom: Some(String::new()),
        ranges: Vec::new(),
    }
}
pub fn import_synergy_conf(text: &str) -> Result<(Config, ImportReport), ConfigError> {
    let (cfg, mut report) = overlay_conf(text, None)?;
    cfg.validate()?;
    report.warnings.extend(cfg.warnings());
    Ok((cfg, report))
}

// A directory import must apply its final core INI identity before validation:
// authoritative classic screens can replace the names generated from db.json.
pub(crate) fn overlay_conf(text: &str, base: Option<Config>) -> Result<(Config, ImportReport), ConfigError> {
    let had_base = base.is_some();
    let mut cfg = base.unwrap_or_default();
    let old_screens = std::mem::take(&mut cfg.screens);
    let mut report = ImportReport::default();
    let mut section = String::new();
    let mut sections = BTreeSet::new();
    let mut current: Option<String> = None;
    let mut aliases: Vec<(usize, String, String)> = Vec::new();
    let mut links: Vec<(usize, String, RangedLink)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix("section:") {
            let name = name.trim();
            if !section.is_empty() {
                return Err(parse_error(line_no, "nested section"));
            }
            if !["screens", "aliases", "links", "options"].contains(&name) || !sections.insert(name.to_string()) {
                return Err(parse_error(line_no, "unknown or duplicate section"));
            }
            section = name.into();
            current = None;
            continue;
        }
        if line == "end" {
            if section.is_empty() {
                return Err(parse_error(line_no, "end outside section"));
            }
            section.clear();
            current = None;
            continue;
        }
        if section.is_empty() {
            return Err(parse_error(line_no, "data outside section"));
        }
        if section != "options" {
            if let Some(name) = line.strip_suffix(':') {
                let name = name.trim();
                if !valid_name(name) {
                    return Err(parse_error(line_no, "invalid screen name"));
                }
                current = Some(name.into());
                if section == "screens" {
                    if cfg.screens.iter().any(|s| s.name == name) {
                        return Err(parse_error(line_no, "duplicate screen"));
                    }
                    let mut screen = old_screens
                        .iter()
                        .find(|s| s.name == name)
                        .cloned()
                        .unwrap_or_else(|| blank_screen(name.into()));
                    screen.links = Some(disconnected());
                    cfg.screens.push(screen);
                }
                continue;
            }
        }
        match section.as_str() {
            "aliases" => {
                let source = current
                    .clone()
                    .ok_or_else(|| parse_error(line_no, "alias needs a canonical header"))?;
                if !valid_name(line) {
                    return Err(parse_error(line_no, "invalid alias"));
                }
                aliases.push((line_no, source, line.into()));
            }
            "links" => {
                let source = current
                    .clone()
                    .ok_or_else(|| parse_error(line_no, "link needs a canonical header"))?;
                let (key, value) = assignment(line, line_no)?;
                let (direction, from_range) = interval(key, line_no, &mut report)?;
                let edge = match direction {
                    "left" => Edge::Left,
                    "right" => Edge::Right,
                    "up" => Edge::Top,
                    "down" => Edge::Bottom,
                    _ => return Err(parse_error(line_no, "invalid direction")),
                };
                let (target, to_range) = interval(value, line_no, &mut report)?;
                if !valid_name(target) {
                    return Err(parse_error(line_no, "invalid link target"));
                }
                links.push((
                    line_no,
                    source,
                    RangedLink {
                        edge,
                        from_range,
                        to: target.into(),
                        to_range,
                    },
                ));
            }
            "screens" => {
                let source = current
                    .as_ref()
                    .ok_or_else(|| parse_error(line_no, "screen option needs a header"))?;
                let screen = cfg
                    .screens
                    .iter_mut()
                    .find(|s| &s.name == source)
                    .ok_or_else(|| parse_error(line_no, "unknown screen"))?;
                let (key, value) = assignment(line, line_no)?;
                screen_option(screen, key, value, line_no, &mut report)?;
            }
            "options" => {
                let (key, value) = assignment(line, line_no)?;
                global_option(&mut cfg, key, value, line_no, &mut report)?;
            }
            _ => return Err(parse_error(line_no, "invalid section")),
        }
    }
    if !section.is_empty() {
        return Err(parse_error(text.lines().count(), "missing end"));
    }
    if cfg.screens.is_empty() {
        return Err(parse_error(1, "screens section must contain at least one screen"));
    }
    let mut canonical: BTreeMap<String, String> =
        cfg.screens.iter().map(|s| (s.name.clone(), s.name.clone())).collect();
    for screen in &cfg.screens {
        for alias in &screen.aliases {
            canonical.insert(alias.clone(), screen.name.clone());
        }
    }
    let mut file_aliases = BTreeSet::new();
    for (line, source, alias) in aliases {
        let screen = cfg
            .screens
            .iter_mut()
            .find(|s| s.name == source)
            .ok_or_else(|| parse_error(line, "alias header is not a canonical screen"))?;
        if !file_aliases.insert(alias.clone()) || cfg_name_collision(&canonical, &alias, &source) {
            return Err(parse_error(line, "duplicate alias"));
        }
        canonical.insert(alias.clone(), source);
        if !screen.aliases.contains(&alias) {
            screen.aliases.push(alias);
        }
    }
    for (line, source, mut link) in links {
        let target = canonical
            .get(&link.to)
            .ok_or_else(|| parse_error(line, "unknown link target"))?;
        link.to = target.clone();
        let screen = cfg
            .screens
            .iter_mut()
            .find(|s| s.name == source)
            .ok_or_else(|| parse_error(line, "link source must be a canonical screen"))?;
        let ranges = &mut screen.links.get_or_insert_with(disconnected).ranges;
        if ranges.iter().any(|other| {
            other.edge == link.edge && other.from_range.0 < link.from_range.1 && link.from_range.0 < other.from_range.1
        }) {
            return Err(parse_error(line, "overlapping source-edge intervals"));
        }
        ranges.push(link);
    }
    if !had_base {
        cfg.role = Role::Server;
        cfg.name = cfg.screens[0].name.clone();
        report.warnings.push(
            "synergy.conf has no local identity: selected its first screen as server; confirm role and name".into(),
        );
    }
    Ok((cfg, report))
}
fn cfg_name_collision(canonical: &BTreeMap<String, String>, alias: &str, source: &str) -> bool {
    canonical
        .get(alias)
        .is_some_and(|owner| owner != source || alias == source)
}
fn assignment(line: &str, number: usize) -> Result<(&str, &str), ConfigError> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| parse_error(number, "expected key = value"))?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() || value.is_empty() {
        return Err(parse_error(number, "empty key or value"));
    }
    Ok((key, value))
}
fn interval<'a>(text: &'a str, line: usize, report: &mut ImportReport) -> Result<(&'a str, (u8, u8)), ConfigError> {
    let Some((name, rest)) = text.split_once('(') else {
        return Ok((text.trim(), (0, 100)));
    };
    let rest = rest
        .strip_suffix(')')
        .ok_or_else(|| parse_error(line, "unclosed interval"))?;
    let (start, end) = rest
        .split_once(',')
        .ok_or_else(|| parse_error(line, "interval needs two endpoints"))?;
    let number = |s: &str| {
        s.trim()
            .parse::<f64>()
            .map_err(|_| parse_error(line, "invalid interval endpoint"))
    };
    let (start, end) = (number(start)?, number(end)?);
    if !start.is_finite() || !end.is_finite() || start < 0.0 || start >= end || end > 100.0 {
        return Err(parse_error(line, "invalid interval range"));
    }
    if start.fract() != 0.0 || end.fract() != 0.0 {
        report.warnings.push(format!(
            "synergy.conf line {line}: fractional endpoints rounded to integer percentages"
        ));
    }
    let range = (start.round() as u8, end.round() as u8);
    check_range(range).map_err(|_| parse_error(line, "interval collapses at integer precision"))?;
    Ok((name.trim(), range))
}
pub(crate) fn boolean(value: &str, line: usize) -> Result<bool, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(parse_error(line, "expected true or false")),
    }
}
pub(crate) fn integer(value: &str, line: usize) -> Result<u32, ConfigError> {
    let value: i32 = value
        .parse()
        .map_err(|_| parse_error(line, "expected signed 32-bit integer"))?;
    u32::try_from(value).map_err(|_| parse_error(line, "expected nonnegative value"))
}
pub(crate) fn corners(value: &str, line: usize) -> Result<Vec<Corner>, ConfigError> {
    let mut remaining = value.trim();
    let mut mask = 0u8;
    let mut first = true;
    while !remaining.is_empty() {
        let remove = remaining.starts_with('-');
        if remaining.starts_with(['+', '-']) {
            remaining = remaining[1..].trim_start();
        } else if !first {
            return Err(parse_error(line, "corner terms require + or -"));
        }
        let terms = [
            ("bottom-right", 8),
            ("bottom-left", 4),
            ("top-right", 2),
            ("top-left", 1),
            ("bottom", 12),
            ("right", 10),
            ("left", 5),
            ("top", 3),
            ("none", 0),
            ("all", 15),
        ];
        let (term, bits) = terms
            .into_iter()
            .find(|(term, _)| remaining.starts_with(term))
            .ok_or_else(|| parse_error(line, "invalid corner expression"))?;
        if remove {
            mask &= !bits;
        } else {
            mask |= bits;
        }
        remaining = remaining[term.len()..].trim_start();
        first = false;
    }
    if first {
        return Err(parse_error(line, "empty corner expression"));
    }
    Ok([
        (1, Corner::TopLeft),
        (2, Corner::TopRight),
        (4, Corner::BottomLeft),
        (8, Corner::BottomRight),
    ]
    .into_iter()
    .filter_map(|(bit, c)| (mask & bit != 0).then_some(c))
    .collect())
}
fn screen_option(
    screen: &mut ScreenCfg,
    key: &str,
    value: &str,
    line: usize,
    report: &mut ImportReport,
) -> Result<(), ConfigError> {
    if ["shift", "ctrl", "alt", "altgr", "meta", "super"].contains(&key) {
        let target = value.to_ascii_lowercase();
        if !["none", "shift", "ctrl", "control", "alt", "altgr", "meta", "super"].contains(&target.as_str()) {
            return Err(parse_error(line, "invalid modifier target"));
        }
        let remap = screen.modifiers.get_or_insert_with(ModifierRemap::default);
        let slot = match key {
            "shift" => &mut remap.shift,
            "ctrl" => &mut remap.ctrl,
            "alt" => &mut remap.alt,
            "altgr" => &mut remap.altgr,
            "meta" => &mut remap.meta,
            _ => &mut remap.super_,
        };
        *slot = Some(target);
        return Ok(());
    }
    let options = screen.options.get_or_insert_with(ScreenOptions::default);
    match key {
        "halfDuplexCapsLock" => options.half_duplex_caps_lock = Some(boolean(value, line)?),
        "halfDuplexNumLock" => options.half_duplex_num_lock = Some(boolean(value, line)?),
        "halfDuplexScrollLock" => options.half_duplex_scroll_lock = Some(boolean(value, line)?),
        "xtestIsXineramaUnaware" => options.xtest_is_xinerama_unaware = Some(boolean(value, line)?),
        "preserveFocus" => options.preserve_focus = Some(boolean(value, line)?),
        "switchCorners" => options.dead_corners = Some(corners(value, line)?),
        "switchCornerSize" => options.corner_size_px = Some(integer(value, line)?),
        _ => report
            .warnings
            .push(format!("synergy.conf line {line}: unsupported screen option {key:?}")),
    }
    Ok(())
}
fn switch_time(value: &str, line: usize, report: &mut ImportReport) -> Result<u64, ConfigError> {
    let number = value
        .parse::<i32>()
        .map_err(|_| parse_error(line, "expected signed 32-bit milliseconds"))?;
    if number < 0 {
        report.warnings.push(format!(
            "synergy.conf line {line}: negative switch timer maps to disabled (0 ms)"
        ));
    }
    Ok(number.max(0) as u64)
}
fn global_option(
    cfg: &mut Config,
    key: &str,
    value: &str,
    line: usize,
    report: &mut ImportReport,
) -> Result<(), ConfigError> {
    match key {
        "heartbeat" => {
            let n = integer(value, line)? as u64;
            cfg.server.heartbeat_ms = n;
            cfg.client.heartbeat_ms = n;
        }
        "relativeMouseMoves" => cfg.input.relative_mouse = boolean(value, line)?,
        "clipboardSharing" => cfg.clipboard.enabled = boolean(value, line)?,
        "clipboardSharingSize" => {
            cfg.clipboard.max_bytes = (integer(value, line)? as usize)
                .checked_mul(1024)
                .ok_or_else(|| parse_error(line, "clipboard size overflow"))?
        }
        "switchDelay" => cfg.switch.delay_ms = switch_time(value, line, report)?,
        "switchDoubleTap" => cfg.switch.double_tap_ms = switch_time(value, line, report)?,
        "switchCorners" => cfg.switch.dead_corners = corners(value, line)?,
        "switchCornerSize" => cfg.switch.corner_size_px = integer(value, line)?,
        "win32KeepForeground" => cfg.options.win32_keep_foreground = boolean(value, line)?,
        "disableLockToScreen" => cfg.options.disable_lock_to_screen = boolean(value, line)?,
        "defaultLockToScreenState" => cfg.options.default_lock_to_screen = boolean(value, line)?,
        "switchNeedsShift" => cfg.options.switch_needs_shift = boolean(value, line)?,
        "switchNeedsControl" => cfg.options.switch_needs_control = boolean(value, line)?,
        "switchNeedsAlt" => cfg.options.switch_needs_alt = boolean(value, line)?,
        "protocol" => {
            cfg.options.barrier = match value.to_ascii_lowercase().as_str() {
                "synergy" => false,
                "barrier" => true,
                _ => return Err(parse_error(line, "protocol must be synergy or barrier")),
            }
        }
        "address" => {
            if let Some(port) = address_port(value).map_err(|_| parse_error(line, "invalid address"))? {
                cfg.server.port = port;
            }
            cfg.server.listen = value.into();
        }
        _ if key.starts_with("keystroke(") || key.starts_with("mousebutton(") || key.starts_with("connect(") => {
            import_hotkey(cfg, key, value, line, report)?
        }
        _ => report
            .warnings
            .push(format!("synergy.conf line {line}: unsupported option {key:?}")),
    }
    Ok(())
}
fn normalize_stroke(stroke: &str) -> String {
    stroke
        .split('+')
        .map(|part| match part.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" => "ctrl".into(),
            "shift" => "shift".into(),
            "alt" => "alt".into(),
            "meta" => "meta".into(),
            "super" => "super".into(),
            "altgr" => "altgr".into(),
            _ => part.trim().to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}
fn core_stroke(stroke: &str) -> String {
    stroke
        .split('+')
        .map(|part| match part.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" => "Control".into(),
            "shift" => "Shift".into(),
            "alt" => "Alt".into(),
            "meta" => "Meta".into(),
            "super" => "Super".into(),
            "altgr" => "AltGr".into(),
            key if key.starts_with('f') && key[1..].parse::<u8>().ok().is_some_and(|n| (1..=35).contains(&n)) => {
                key.to_ascii_uppercase()
            }
            _ => crate::model::canonical_key(part.trim())
                .unwrap_or(part.trim())
                .to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}
fn import_hotkey(
    cfg: &mut Config,
    trigger: &str,
    actions: &str,
    line: usize,
    report: &mut ImportReport,
) -> Result<(), ConfigError> {
    let stroke = if let Some(stroke) = trigger.strip_prefix("keystroke(").and_then(|s| s.strip_suffix(')')) {
        normalize_stroke(stroke)
    } else if trigger.starts_with("mousebutton(") && trigger.ends_with(')') {
        format!("mousebutton({})", normalize_stroke(&trigger[12..trigger.len() - 1]))
    } else {
        if !trigger.starts_with("connect(") || !trigger.ends_with(')') {
            return Err(parse_error(line, "malformed hotkey condition"));
        }
        report.warnings.push(format!(
            "synergy.conf line {line}: connect trigger has no Openergy mapping"
        ));
        return Ok(());
    };
    validate_stroke(&stroke).map_err(|_| parse_error(line, "invalid hotkey trigger"))?;
    // Action separators apply only at nesting level zero; commas can occur in action arguments.
    let mut depth = 0i32;
    let mut start = 0;
    let mut release = false;
    for (index, c) in actions.char_indices().chain(std::iter::once((actions.len(), ','))) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return Err(parse_error(line, "unbalanced action"));
                }
            }
            _ => {}
        }
        if (c == ',' || c == ';') && depth == 0 {
            let action = actions[start..index].trim();
            if !action.is_empty() {
                if !release && ["lockCursorToScreen(toggle)", "lockCursorToScreen()"].contains(&action) {
                    if cfg.hotkeys.lock_to_screen.as_ref().is_some_and(|old| old != &stroke) {
                        report.warnings.push(format!(
                            "synergy.conf line {line}: lock hotkey replaced; only one lock binding is supported"
                        ));
                    }
                    cfg.hotkeys.lock_to_screen = Some(stroke.clone());
                } else {
                    report.warnings.push(format!(
                        "synergy.conf line {line}: {} action {} has no Openergy mapping",
                        if release { "release" } else { "press" },
                        action.split('(').next().unwrap_or("unknown")
                    ));
                }
            }
            release |= c == ';';
            start = index + c.len_utf8();
        }
    }
    if depth != 0 {
        return Err(parse_error(line, "unbalanced action"));
    }
    Ok(())
}

fn corner_string(corners: &[Corner]) -> String {
    let mut out = String::from("none");
    for c in corners {
        out.push_str(match c {
            Corner::TopLeft => " +top-left",
            Corner::TopRight => " +top-right",
            Corner::BottomLeft => " +bottom-left",
            Corner::BottomRight => " +bottom-right",
        });
    }
    out
}
pub fn export_synergy_conf(cfg: &Config) -> Result<String, ConfigError> {
    cfg.validate()?;
    if cfg.screens.is_empty() {
        return Err(ConfigError::Validation("export requires at least one screen".into()));
    }
    let mut out = String::from("section: screens\n");
    for screen in &cfg.screens {
        let _ = writeln!(out, "    {}:", screen.name);
        if let Some(m) = &screen.modifiers {
            for (name, target) in [
                ("shift", &m.shift),
                ("ctrl", &m.ctrl),
                ("alt", &m.alt),
                ("altgr", &m.altgr),
                ("meta", &m.meta),
                ("super", &m.super_),
            ] {
                if let Some(target) = target {
                    let _ = writeln!(out, "        {name} = {target}");
                }
            }
        }
        if let Some(o) = &screen.options {
            for (name, value) in [
                ("halfDuplexCapsLock", o.half_duplex_caps_lock),
                ("halfDuplexNumLock", o.half_duplex_num_lock),
                ("halfDuplexScrollLock", o.half_duplex_scroll_lock),
                ("xtestIsXineramaUnaware", o.xtest_is_xinerama_unaware),
                ("preserveFocus", o.preserve_focus),
            ] {
                if let Some(value) = value {
                    let _ = writeln!(out, "        {name} = {value}");
                }
            }
            if let Some(c) = &o.dead_corners {
                let _ = writeln!(out, "        switchCorners = {}", corner_string(c));
            }
            if let Some(size) = o.corner_size_px {
                let _ = writeln!(out, "        switchCornerSize = {size}");
            }
        }
    }
    out.push_str("end\n\n");
    if cfg.screens.iter().any(|s| !s.aliases.is_empty()) {
        out.push_str("section: aliases\n");
        for s in &cfg.screens {
            if !s.aliases.is_empty() {
                let _ = writeln!(out, "    {}:", s.name);
                for alias in &s.aliases {
                    let _ = writeln!(out, "        {alias}");
                }
            }
        }
        out.push_str("end\n\n");
    }
    out.push_str("section: links\n");
    let links = derive_links(&cfg.screens);
    for screen in &cfg.screens {
        let _ = writeln!(out, "    {}:", screen.name);
        for link in links.iter().filter(|l| l.from == screen.name) {
            let edge = match link.edge {
                Edge::Left => "left",
                Edge::Right => "right",
                Edge::Top => "up",
                Edge::Bottom => "down",
            };
            let _ = writeln!(
                out,
                "        {edge}({},{}) = {}({},{})",
                link.from_range.0, link.from_range.1, link.to, link.to_range.0, link.to_range.1
            );
        }
    }
    out.push_str("end\n\nsection: options\n");
    let address = if cfg.server.listen.starts_with('[') {
        cfg.server
            .listen
            .split(']')
            .next()
            .map(|s| format!("{s}]:{}", cfg.server.port))
            .unwrap_or_default()
    } else if cfg.server.listen.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]:{}", cfg.server.listen, cfg.server.port)
    } else {
        format!(
            "{}:{}",
            cfg.server.listen.split(':').next().unwrap_or("0.0.0.0"),
            cfg.server.port
        )
    };
    let heartbeat = if cfg.role == Role::Client {
        cfg.client.heartbeat_ms
    } else {
        cfg.server.heartbeat_ms
    };
    let _ = writeln!(
        out,
        "    # protocol: {} (select the peer binary dialect at startup)\n    address = {address}\n    heartbeat = {heartbeat}",
        if cfg.options.barrier { "barrier" } else { "synergy" }
    );
    for (key, value) in [
        ("relativeMouseMoves", cfg.input.relative_mouse),
        ("clipboardSharing", cfg.clipboard.enabled),
        ("win32KeepForeground", cfg.options.win32_keep_foreground),
        ("disableLockToScreen", cfg.options.disable_lock_to_screen),
        ("defaultLockToScreenState", cfg.options.default_lock_to_screen),
        ("switchNeedsShift", cfg.options.switch_needs_shift),
        ("switchNeedsControl", cfg.options.switch_needs_control),
        ("switchNeedsAlt", cfg.options.switch_needs_alt),
    ] {
        if cfg.options.barrier
            && matches!(
                key,
                "disableLockToScreen"
                    | "defaultLockToScreenState"
                    | "switchNeedsShift"
                    | "switchNeedsControl"
                    | "switchNeedsAlt"
            )
        {
            continue;
        }
        let _ = writeln!(out, "    {key} = {value}");
    }
    if !cfg.options.barrier {
        let _ = writeln!(out, "    clipboardSharingSize = {}", cfg.clipboard.max_bytes / 1024);
    }
    let _ = writeln!(
        out,
        "    switchDelay = {}\n    switchDoubleTap = {}\n    switchCorners = {}\n    switchCornerSize = {}",
        cfg.switch.delay_ms,
        cfg.switch.double_tap_ms,
        corner_string(&cfg.switch.dead_corners),
        cfg.switch.corner_size_px
    );
    if let Some(stroke) = &cfg.hotkeys.lock_to_screen {
        let trigger = if let Some(inner) = stroke.strip_prefix("mousebutton(").and_then(|s| s.strip_suffix(')')) {
            format!("mousebutton({})", core_stroke(inner))
        } else {
            format!("keystroke({})", core_stroke(stroke))
        };
        let _ = writeln!(out, "    {trigger} = lockCursorToScreen(toggle)");
    }
    out.push_str("end\n");
    Ok(out)
}
