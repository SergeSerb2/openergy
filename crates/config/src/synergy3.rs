use crate::{
    synergy::{blank_screen, disconnected, overlay_conf},
    *,
};
use serde_json::Value;
use std::{collections::BTreeMap, fs, io::Read, path::Path};

// Preserve the service's JavaScript object insertion order for its last-corner-size rule.
// serde_json::Value maps sort keys, so this small second view captures order without new deps.
#[derive(Default)]
struct OrderedCorners(Vec<(String, Value)>);
impl<'de> serde::Deserialize<'de> for OrderedCorners {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = OrderedCorners;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("corner object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    values.push(entry);
                }
                Ok(OrderedCorners(values))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
#[derive(serde::Deserialize)]
struct CornerComputer {
    #[serde(default)]
    id: String,
    #[serde(default)]
    dead_corners: OrderedCorners,
}
#[derive(serde::Deserialize)]
struct CornerData {
    #[serde(default)]
    computers: Vec<CornerComputer>,
}
#[derive(serde::Deserialize)]
struct CornerDb {
    data: CornerData,
}

fn bad(path: &str, reason: &str) -> ConfigError {
    ConfigError::Parse(format!("{path}: {reason}"))
}
fn field<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |value, key| value.get(key))
}
fn bool_at(root: &Value, path: &str) -> Result<Option<bool>, ConfigError> {
    field(root, path)
        .map(|v| v.as_bool().ok_or_else(|| bad(path, "expected boolean")))
        .transpose()
}
fn number_at(root: &Value, path: &str) -> Result<Option<f64>, ConfigError> {
    field(root, path)
        .map(|v| {
            v.as_f64()
                .filter(|n| n.is_finite())
                .ok_or_else(|| bad(path, "expected finite number"))
        })
        .transpose()
}
fn uint_at(root: &Value, path: &str) -> Result<Option<u64>, ConfigError> {
    field(root, path)
        .map(|v| v.as_u64().ok_or_else(|| bad(path, "expected nonnegative integer")))
        .transpose()
}
fn str_at<'a>(root: &'a Value, path: &str) -> Result<Option<&'a str>, ConfigError> {
    field(root, path)
        .map(|v| v.as_str().ok_or_else(|| bad(path, "expected string")))
        .transpose()
}
fn set_bool(root: &Value, path: &str, slot: &mut bool) -> Result<(), ConfigError> {
    if let Some(v) = bool_at(root, path)? {
        *slot = v;
    }
    Ok(())
}
fn set_port(cfg: &mut Config, port: u64) -> Result<(), ConfigError> {
    cfg.server.port = u16::try_from(port)
        .ok()
        .filter(|v| *v != 0)
        .ok_or_else(|| bad("core port", "expected 1..=65535"))?;
    cfg.server.listen = format!("0.0.0.0:{}", cfg.server.port);
    Ok(())
}

fn active_computers(db: &Value) -> Result<Vec<&Value>, ConfigError> {
    let computers = field(db, "data.computers")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("data.computers", "expected array"))?;
    computers
        .iter()
        .filter_map(|c| match bool_at(c, "isRemoved") {
            Ok(Some(true)) => None,
            Ok(_) => Some(Ok(c)),
            Err(e) => Some(Err(e)),
        })
        .collect()
}
fn generated_name(computer: &Value) -> Result<String, ConfigError> {
    let display = str_at(computer, "name")?
        .filter(|n| !n.is_empty())
        .ok_or_else(|| bad("computer.name", "missing or empty"))?;
    let base: String = display
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let id =
        str_at(computer, "id")?.ok_or_else(|| bad("computer.id", "missing identity needed for wire-name mapping"))?;
    let suffix: String = id.chars().rev().take(8).collect::<String>().chars().rev().collect();
    let name = if base.is_empty() {
        suffix
    } else {
        format!("{base}-{suffix}")
    };
    if !crate::model::valid_name(&name) {
        return Err(bad("computer.name", "cannot derive a valid wire name"));
    }
    Ok(name)
}

pub fn import_synergy3_db(json: &str, core_ini: Option<&str>) -> Result<(Config, ImportReport), ConfigError> {
    let db: Value = serde_json::from_str(json).map_err(|e| ConfigError::Parse(format!("db.json: {e}")))?;
    let order: CornerDb =
        serde_json::from_str(json).map_err(|e| ConfigError::Parse(format!("db.json corners: {e}")))?;
    let order: BTreeMap<_, _> = order
        .data
        .computers
        .into_iter()
        .map(|c| (c.id, c.dead_corners.0))
        .collect();
    import_db(&db, core_ini, &order)
}
fn import_db(
    db: &Value,
    core_ini: Option<&str>,
    order: &BTreeMap<String, Vec<(String, Value)>>,
) -> Result<(Config, ImportReport), ConfigError> {
    if uint_at(db, "version.schemaVersion")? != Some(17) {
        return Err(bad("db.json version.schemaVersion", "only schema 17 is supported"));
    }
    let data = db
        .get("data")
        .filter(|v| v.is_object())
        .ok_or_else(|| bad("db.json data", "expected object"))?;
    let mut cfg = Config::default();
    let mut report = ImportReport::default();
    for c in active_computers(db)? {
        let mut screen = blank_screen(generated_name(c)?);
        let rect_values = [
            number_at(c, "misc.left")?,
            number_at(c, "misc.top")?,
            number_at(c, "misc.width")?,
            number_at(c, "misc.height")?,
        ];
        if let [Some(x), Some(y), Some(w), Some(h)] = rect_values {
            screen.rect = Some(LayoutRect { x, y, w, h });
        } else if rect_values.iter().any(Option::is_some) {
            return Err(bad("computer.misc", "incomplete layout rect"));
        } else {
            report
                .warnings
                .push("data.computers[].misc: missing layout; no links derived for that screen".into());
        }
        screen.os = str_at(c, "misc.os")?.map(str::to_string);
        if let Some(modifier) = c.get("modifier") {
            if !modifier.is_object() {
                return Err(bad("computer.modifier", "expected object"));
            }
            let mut m = ModifierRemap::default();
            for (key, slot) in [
                ("Shift", &mut m.shift),
                ("Ctrl", &mut m.ctrl),
                ("Alt", &mut m.alt),
                ("AltGr", &mut m.altgr),
                ("Meta", &mut m.meta),
                ("Super", &mut m.super_),
            ] {
                *slot = str_at(modifier, key)?.map(str::to_ascii_lowercase);
            }
            screen.modifiers = Some(m);
        }
        if bool_at(data, "keyboard.altgr")? == Some(true) {
            screen.modifiers.get_or_insert_with(ModifierRemap::default).altgr = Some("alt".into());
        }
        let mut o = ScreenOptions {
            half_duplex_caps_lock: bool_at(c, "fix.capslock")?,
            half_duplex_num_lock: bool_at(c, "fix.num_lock")?,
            half_duplex_scroll_lock: bool_at(c, "fix.scr_lock")?,
            xtest_is_xinerama_unaware: bool_at(c, "fix.xtest")?,
            ..ScreenOptions::default()
        };
        if let Some(corners) = c.get("dead_corners") {
            let obj = corners
                .as_object()
                .ok_or_else(|| bad("computer.dead_corners", "expected object"))?;
            let mut selected = Vec::new();
            let mut last = None;
            let empty = Vec::new();
            let ordered = order.get(str_at(c, "id")?.unwrap_or("")).unwrap_or(&empty);
            if ordered.len() != obj.len() {
                return Err(bad("computer.dead_corners", "duplicate corner names"));
            }
            for (key, value) in ordered {
                let corner = match key.as_str() {
                    "top-left" => Corner::TopLeft,
                    "top-right" => Corner::TopRight,
                    "bottom-left" => Corner::BottomLeft,
                    "bottom-right" => Corner::BottomRight,
                    _ => {
                        report
                            .warnings
                            .push("data.computers[].dead_corners: unknown corner ignored".into());
                        continue;
                    }
                };
                if value.as_str() == Some("") {
                    continue;
                }
                let size = value
                    .as_u64()
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or_else(|| bad("computer.dead_corners", "expected empty string or nonnegative integer"))?;
                if size > 0 {
                    if last.is_some_and(|old| old != size) {
                        report.warnings.push("data.computers[].dead_corners: different sizes collapse to one scalar; confirm corner_size_px".into());
                    }
                    selected.push(corner);
                    last = Some(size);
                }
            }
            o.dead_corners = Some(selected);
            o.corner_size_px = last;
        }
        if o != ScreenOptions::default() {
            screen.options = Some(o);
        }
        warn_unmapped(
            c,
            "data.computers[]",
            &[
                "id",
                "name",
                "isRemoved",
                "scroll_sensitivity",
                "misc",
                "modifier",
                "fix",
                "dead_corners",
            ],
            &mut report,
        );
        if let Some(fix) = c.get("fix") {
            warn_unmapped(
                fix,
                "data.computers[].fix",
                &[
                    "capslock",
                    "num_lock",
                    "scr_lock",
                    "xtest",
                    "sync_language",
                    "invert_scroll",
                    "prevent_sleep",
                ],
                &mut report,
            );
        }
        if let Some(misc) = c.get("misc") {
            warn_unmapped(
                misc,
                "data.computers[].misc",
                &["left", "top", "width", "height", "os"],
                &mut report,
            );
        }
        if let Some(m) = c.get("modifier") {
            warn_unmapped(
                m,
                "data.computers[].modifier",
                &["Shift", "Ctrl", "Alt", "AltGr", "Meta", "Super"],
                &mut report,
            );
        }
        cfg.screens.push(screen);
    }
    // Keep the editor coordinate system. It is already a uniform layout-unit system.
    // Freeze service-converter ranges before any later UI move or coordinate scaling.
    let derived = derive_links(&cfg.screens);
    for screen in &mut cfg.screens {
        let mut links = disconnected();
        links.ranges = derived
            .iter()
            .filter(|l| l.from == screen.name)
            .map(|l| RangedLink {
                edge: l.edge,
                from_range: l.from_range,
                to: l.to.clone(),
                to_range: l.to_range,
            })
            .collect();
        screen.links = Some(links);
    }
    report.warnings.push("db.json rects retain 622x440 editor layout units; physical display sizes are unavailable; imported links are explicit service ranges".into());
    set_bool(data, "security.encryption", &mut cfg.tls.enabled)?;
    set_bool(data, "mouse.relative", &mut cfg.input.relative_mouse)?;
    set_bool(data, "performance.allow_copy", &mut cfg.clipboard.enabled)?;
    if let Some(limit) = uint_at(data, "performance.limit")? {
        cfg.clipboard.max_bytes = usize::try_from(limit)
            .ok()
            .and_then(|v| v.checked_mul(1024))
            .ok_or_else(|| bad("performance.limit", "KiB size overflow"))?;
    }
    // performance.sync is persisted by Synergy but has no service/core effect.
    if bool_at(data, "performance.sync")?.is_some() {
        report
            .warnings
            .push("data.performance.sync: no Synergy core effect; clipboard send and receive remain enabled".into());
    }
    if let Some(port) = uint_at(data, "developers.corePort")? {
        set_port(&mut cfg, port)?;
    }
    if let Some(delay) = uint_at(data, "developers.switchDelay")? {
        cfg.switch.delay_ms = delay;
    }
    if bool_at(data, "developers.shouldSwitchDoubleTap")? == Some(true) {
        cfg.switch.double_tap_ms = uint_at(data, "developers.switchDoubleTap")?.unwrap_or(0);
    }
    cfg.options.disable_lock_to_screen = bool_at(data, "keyboard.lock_to_screen")?.is_some_and(|v| !v);
    if let Some(enabled) = bool_at(data, "local-network.keep_alive")? {
        let heartbeat = if enabled {
            uint_at(data, "local-network.interval")?.unwrap_or(3000)
        } else {
            report
                .warnings
                .push("data.local-network.keep_alive=false: Synergy omits heartbeat; default 3000 ms retained".into());
            3000
        };
        cfg.server.heartbeat_ms = heartbeat;
        cfg.client.heartbeat_ms = heartbeat;
    }
    import_hotkeys(data, &mut cfg, &mut report)?;
    warn_unmapped(
        data,
        "data",
        &[
            "computers",
            "mainID",
            "hotkeys",
            "keyboard",
            "mouse",
            "performance",
            "security",
            "developers",
            "local-network",
        ],
        &mut report,
    );
    for (section, keys) in [
        ("keyboard", vec!["altgr", "lock_to_screen"]),
        ("mouse", vec!["relative"]),
        ("performance", vec!["allow_copy", "limit", "sync"]),
        ("security", vec!["encryption"]),
        (
            "developers",
            vec!["corePort", "switchDelay", "shouldSwitchDoubleTap", "switchDoubleTap"],
        ),
        ("local-network", vec!["keep_alive", "interval"]),
    ] {
        if let Some(value) = data.get(section) {
            warn_unmapped(value, &format!("data.{section}"), &keys, &mut report);
        }
    }
    if let Some(ini) = core_ini {
        apply_ini(&mut cfg, ini, &mut report)?;
    } else {
        report.warnings.push(
            "db.json alone has no local identity; role remains off until local.json or core INI is supplied".into(),
        );
    }
    cfg.validate()?;
    if active_computers(db)?
        .iter()
        .any(|c| field(c, "fix.sync_language").is_some() || field(c, "scroll_sensitivity").is_some())
    {
        report.warnings.push("data.computers[].fix.{sync_language,invert_scroll,prevent_sleep} and scroll_sensitivity: local values map to global input fields when identity is available; remote per-screen values have no global mapping".into());
    }
    report.warnings.extend(cfg.warnings());
    Ok((cfg, report))
}
fn private_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "serial",
        "license",
        "telemetry",
        "sentry",
        "account",
        "fingerprint",
        "token",
        "password",
        "secret",
    ]
    .iter()
    .any(|word| key.contains(word))
        || key == "id"
        || key == "myid"
        || key == "mainid"
        || key == "ips"
        || key == "local_computers"
}
fn warn_unmapped(value: &Value, path: &str, known: &[&str], report: &mut ImportReport) {
    if let Some(object) = value.as_object() {
        for key in object.keys() {
            if !known.contains(&key.as_str()) && !private_key(key) {
                // Never echo arbitrary database keys: keys themselves can contain IDs.
                let safe = key.chars().all(|c| c.is_ascii_alphabetic() || "_-".contains(c));
                report.warnings.push(format!(
                    "{path}.{}: no Openergy mapping",
                    if safe { key.as_str() } else { "<unknown-field>" }
                ));
            }
        }
    }
}
fn import_hotkeys(data: &Value, cfg: &mut Config, report: &mut ImportReport) -> Result<(), ConfigError> {
    let Some(hotkeys) = data.get("hotkeys") else {
        return Ok(());
    };
    let hotkeys = hotkeys
        .as_object()
        .ok_or_else(|| bad("data.hotkeys", "expected object"))?;
    for hotkey in hotkeys.values() {
        if bool_at(hotkey, "enabled")? != Some(true) {
            continue;
        }
        let stroke = hotkey
            .get("keystroke")
            .ok_or_else(|| bad("hotkey.keystroke", "missing stroke"))?;
        let mut parts = Vec::new();
        for (key, name) in [
            ("control", "ctrl"),
            ("alt", "alt"),
            ("meta", "meta"),
            ("shift", "shift"),
        ] {
            if bool_at(stroke, key)? == Some(true) {
                parts.push(name.to_string());
            }
        }
        let kind = str_at(stroke, "type")?.unwrap_or("");
        let trigger = match kind {
            "keystroke" => {
                let key = str_at(stroke, "key")?.ok_or_else(|| bad("hotkey.key", "missing key"))?;
                parts.push(dom_key(key));
                parts.join("+")
            }
            "mousebutton" => {
                parts.push(
                    str_at(stroke, "button")?
                        .ok_or_else(|| bad("hotkey.button", "missing button"))?
                        .into(),
                );
                format!("mousebutton({})", parts.join("+"))
            }
            _ => {
                report
                    .warnings
                    .push("data.hotkeys[].keystroke.type: unsupported trigger".into());
                continue;
            }
        };
        let actions = hotkey
            .get("actions")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("hotkey.actions", "expected array"))?;
        for action in actions {
            if bool_at(action, "enabled")? == Some(false) {
                report.warnings.push(
                    "data.hotkeys[].actions[].enabled: Synergy ignores this flag; imported active behavior".into(),
                );
            }
            if str_at(action, "type")? == Some("LOCK_CURSOR")
                && str_at(action, "lockType")? == Some("toggle")
                && uint_at(action, "trigger")?.unwrap_or(0) == 0
            {
                crate::model::validate_stroke(&trigger)?;
                if cfg.hotkeys.lock_to_screen.is_some() {
                    report
                        .warnings
                        .push("data.hotkeys: only one lock binding is supported; later binding wins".into());
                }
                cfg.hotkeys.lock_to_screen = Some(trigger.clone());
            } else {
                report
                    .warnings
                    .push("data.hotkeys[].actions[]: action has no Openergy mapping".into());
            }
        }
    }
    Ok(())
}
fn dom_key(key: &str) -> String {
    if let Some(k) = key.strip_prefix("Key").filter(|s| s.len() == 1) {
        return k.to_ascii_lowercase();
    }
    if let Some(k) = key.strip_prefix("Digit").filter(|s| s.len() == 1) {
        return k.into();
    }
    if let Some(k) = key
        .strip_prefix("Numpad")
        .filter(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_digit())
    {
        return format!("KP_{k}");
    }
    match key {
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        "ArrowUp" => "Up",
        "ArrowDown" => "Down",
        "Enter" => "Linefeed",
        "Backspace" => "BackSpace",
        "Backquote" => "Grave",
        "BracketLeft" => "BracketL",
        "BracketRight" => "BracketR",
        "Quote" => "DoubleQuote",
        "ContextMenu" => "Menu",
        "IntlBackslash" => "Backslash",
        "IntlRo" => "\\u30ED",
        "IntlYen" => "\\u00A5",
        "Convert" => "\\uEF23",
        "KanaMode" => "\\uEF7e",
        "Lang1" => "\\uEF31",
        "Lang2" => "\\uEF34",
        "Lang3" => "\\uFF26",
        "Lang4" => "\\uFF25",
        "Lang5" => "\\uEF2A",
        "PrintScreen" => "\\uFD1D",
        "BrowserBack" => "WWWBack",
        "BrowserForward" => "WWWForward",
        "BrowserRefresh" => "WWWRefresh",
        "BrowserStop" => "WWWStop",
        "BrowserSearch" => "WWWSearch",
        "BrowserFavorites" => "WWWFavorites",
        "BrowserHome" => "WWWHome",
        "AudioVolumeDown" | "VolumeDown" => "AudioDown",
        "AudioVolumeUp" | "VolumeUp" => "AudioUp",
        "AudioVolumeMute" | "VolumeMute" => "AudioMute",
        "LaunchApp1" => "AppUser1",
        "LaunchApp2" => "AppUser2",
        "LaunchMail" => "AppMail",
        "MediaPlayPause" => "AudioPlay",
        "MediaStop" => "AudioStop",
        "MediaTrackNext" => "AudioNext",
        "MediaTrackPrevious" => "AudioPrev",
        "NumpadAdd" => "KP_Add",
        "NumpadComma" => "KP_Separator",
        "NumpadDecimal" => "KP_Decimal",
        "NumpadDivide" => "KP_Divide",
        "NumpadEnter" => "KP_Enter",
        "NumpadEqual" => "KP_Equal",
        "NumpadMultiply" => "KP_Multiply",
        "NumpadSubtract" => "KP_Subtract",
        _ => key,
    }
    .into()
}

pub(crate) fn read_text(path: &Path) -> Result<String, ConfigError> {
    let file = fs::File::open(path).map_err(|e| ConfigError::Io(format!("read {}: {e}", path.display())))?;
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ConfigError::Io(e.to_string()))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(ConfigError::Io("import file exceeds 16 MiB".into()));
    }
    String::from_utf8(bytes).map_err(|_| bad("import file", "not UTF-8"))
}
fn optional_text(path: &Path) -> Result<Option<String>, ConfigError> {
    match fs::symlink_metadata(path) {
        Ok(_) => read_text(path).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Io(e.to_string())),
    }
}
pub fn import_synergy_dir(path: &Path) -> Result<(Config, ImportReport), ConfigError> {
    if path.is_file() {
        return import_synergy_conf(&read_text(path)?);
    }
    let db = optional_text(&path.join("db.json"))?;
    let conf = optional_text(&path.join("synergy.conf"))?;
    let ini = optional_text(&path.join("synergy-core.conf"))?;
    let local = optional_text(&path.join("local.json"))?;
    let mut parsed_db = None;
    let (mut cfg, mut report) = if let Some(db) = db {
        let value: Value = serde_json::from_str(&db).map_err(|e| ConfigError::Parse(format!("db.json: {e}")))?;
        let result = import_synergy3_db(&db, None)?;
        parsed_db = Some(value);
        result
    } else if let Some(conf) = &conf {
        import_synergy_conf(conf)?
    } else {
        return Err(ConfigError::Io(
            "no db.json or synergy.conf found in import directory".into(),
        ));
    };
    if let (Some(db), Some(local)) = (&parsed_db, local) {
        let local: Value = serde_json::from_str(&local).map_err(|e| ConfigError::Parse(format!("local.json: {e}")))?;
        apply_local(&mut cfg, db, &local, &mut report)?;
    }
    if parsed_db.is_some() {
        if let Some(conf) = conf {
            let (next, other) = overlay_conf(&conf, Some(cfg))?;
            cfg = next;
            report.warnings.extend(other.warnings);
            report.warnings.push(
                "synergy.conf supplies authoritative screens and links; matching db layout metadata was retained"
                    .into(),
            );
        }
    }
    if let Some(ini) = ini {
        apply_ini(&mut cfg, &ini, &mut report)?;
    }
    if !cfg.name.is_empty() {
        report
            .warnings
            .retain(|w| !w.starts_with("db.json alone has no local identity"));
    }
    if let Some(trust) = optional_text(&path.join("tls/trusted-servers"))? {
        for (index, line) in trust.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(hex) = line
                .strip_prefix("v2:sha256:")
                .filter(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
            {
                let hex = hex.to_ascii_lowercase();
                if !report.trusted_servers.contains(&hex) {
                    report.trusted_servers.push(hex);
                }
            } else {
                report.warnings.push(format!(
                    "tls/trusted-servers line {}: unsupported or malformed pin record ignored",
                    index + 1
                ));
            }
        }
        if !report.trusted_servers.is_empty() {
            report.warnings.push("Trust candidates returned in ImportReport.trusted_servers; approve them through the trust store before connecting".into());
        }
    }
    cfg.validate()?;
    report.warnings.extend(cfg.warnings());
    report.warnings.sort();
    report.warnings.dedup();
    Ok((cfg, report))
}
fn apply_local(cfg: &mut Config, db: &Value, local: &Value, report: &mut ImportReport) -> Result<(), ConfigError> {
    if uint_at(local, "version")? != Some(7) {
        report
            .warnings
            .push("local.json version differs from observed schema 7; recognized fields only were imported".into());
    }
    // IDs are used transiently only to join records. They are never returned or logged.
    let self_id = str_at(local, "myId")?.unwrap_or("");
    let main = str_at(db, "data.mainID.id")?.unwrap_or("");
    let computers = active_computers(db)?;
    if let Some(computer) = computers
        .iter()
        .find(|c| c.get("id").and_then(Value::as_str) == Some(self_id))
    {
        cfg.name = generated_name(computer)?;
        cfg.role = if main.is_empty() {
            Role::Off
        } else if main == self_id {
            Role::Server
        } else {
            Role::Client
        };
        set_bool(computer, "fix.sync_language", &mut cfg.input.sync_language)?;
        set_bool(computer, "fix.invert_scroll", &mut cfg.input.invert_scroll_y)?;
        set_bool(computer, "fix.prevent_sleep", &mut cfg.input.prevent_sleep)?;
        if let Some(sensitivity) = number_at(computer, "scroll_sensitivity")? {
            cfg.input.scroll_sensitivity = sensitivity;
        }
        if cfg.role == Role::Client {
            if let Some(remote) = local
                .get("local_computers")
                .and_then(|v| v.get(main))
                .and_then(Value::as_str)
            {
                cfg.client.remote = remote.into();
            }
        }
    } else {
        report
            .warnings
            .push("local.json self record is unavailable; local role and name remain unset".into());
    }
    set_bool(
        local,
        "local_developers.keepForeground",
        &mut cfg.options.win32_keep_foreground,
    )?;
    if bool_at(local, "local_system.core_enabled")? == Some(false) {
        cfg.role = Role::Off;
        report
            .warnings
            .push("local.json core_enabled=false: role set to off".into());
    }
    warn_unmapped(
        local,
        "local",
        &["version", "myId", "local_computers", "local_system", "local_developers"],
        report,
    );
    if let Some(v) = local.get("local_system") {
        warn_unmapped(v, "local.local_system", &["core_enabled"], report);
    }
    if let Some(v) = local.get("local_developers") {
        warn_unmapped(v, "local.local_developers", &["keepForeground"], report);
    }
    Ok(())
}
fn ini_value(value: &str, line: usize) -> Result<String, ConfigError> {
    if let Some(value) = value.strip_prefix('"') {
        let value = value
            .strip_suffix('"')
            .ok_or_else(|| bad(&format!("core INI line {line}"), "unclosed quoted string"))?;
        let mut out = String::new();
        let mut chars = value.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                out.push(match chars.next() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\\') => '\\',
                    Some('"') => '"',
                    _ => return Err(bad("core INI", "unsupported escape")),
                });
            } else {
                out.push(c);
            }
        }
        Ok(out)
    } else {
        Ok(value.into())
    }
}
fn apply_ini(cfg: &mut Config, text: &str, report: &mut ImportReport) -> Result<(), ConfigError> {
    let mut section = String::new();
    let mut entries = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = name.into();
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| bad(&format!("core INI line {}", index + 1), "expected assignment"))?;
        entries.insert(
            format!("{section}/{}", key.trim()),
            (index + 1, ini_value(value.trim(), index + 1)?),
        );
    }
    for (key, (line, value)) in entries {
        let bool_value = || match value.to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(bad(&format!("core INI {key} line {line}"), "invalid boolean")),
        };
        match key.as_str() {
            "core/coreMode" => {
                let role = match value.as_str() {
                    "0" | "None" | "none" => Role::Off,
                    "1" | "Client" | "client" => Role::Client,
                    "2" | "Server" | "server" => Role::Server,
                    _ => return Err(bad("core/coreMode", "unknown enum spelling")),
                };
                if cfg.role != Role::Off && cfg.role != role {
                    report.warnings.push("core INI role overrides local.json role".into());
                }
                cfg.role = role;
            }
            "core/computerName" => {
                if !cfg.name.is_empty() && cfg.name != value {
                    report
                        .warnings
                        .push("core INI computerName overrides local.json name".into());
                }
                cfg.name = value;
            }
            "core/port" => set_port(cfg, value.parse().map_err(|_| bad("core/port", "invalid integer"))?)?,
            "core/preventSleep" => cfg.input.prevent_sleep = bool_value()?,
            "security/tlsEnabled" => cfg.tls.enabled = bool_value()?,
            "client/remoteHost" => cfg.client.remote = value,
            "client/languageSync" => cfg.input.sync_language = bool_value()?,
            "client/invertYScroll" => cfg.input.invert_scroll_y = bool_value()?,
            "security/checkPeerFingerprints" => {
                if !bool_value()? {
                    report.warnings.push(
                        "core INI disables peer fingerprint checking; Openergy retains require_client_trust=true"
                            .into(),
                    );
                }
            }
            _ => {
                if !private_key(&key) {
                    let safe = key.chars().all(|c| c.is_ascii_alphabetic() || "/_-".contains(c));
                    report.warnings.push(format!(
                        "core INI {}: no Openergy mapping",
                        if safe { key.as_str() } else { "<unknown-field>" }
                    ));
                }
            }
        }
    }
    Ok(())
}
