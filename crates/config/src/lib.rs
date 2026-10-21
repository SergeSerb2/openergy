//! Frozen config model (architecture-r2 §2.8). Importer/derivation bodies land in WP2.
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::{fmt, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Off,
    Server,
    Client,
}
impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Server => "server",
            Self::Client => "client",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedLink {
    pub from: String,
    pub edge: Edge,
    pub from_range: (u8, u8),
    pub to: String,
    pub to_range: (u8, u8),
}
pub type DerivedLinks = Vec<DerivedLink>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinksCfg {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifierRemap {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shift: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctrl: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altgr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub super_: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScreenCfg {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<LayoutRect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<LinksCfg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<ModifierRemap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerCfg {
    pub listen: String,
    pub port: u16,
    pub heartbeat_ms: u64,
}
impl Default for ServerCfg {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:24800".into(),
            port: 24800,
            heartbeat_ms: 3000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientCfg {
    pub remote: String,
    pub autoconnect: bool,
    pub heartbeat_ms: u64,
}
impl Default for ClientCfg {
    fn default() -> Self {
        Self {
            remote: String::new(),
            autoconnect: true,
            heartbeat_ms: 3000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TlsCfg {
    pub enabled: bool,
    pub require_client_trust: bool,
}
impl Default for TlsCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            require_client_trust: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardCfg {
    pub enabled: bool,
    pub send: bool,
    pub receive: bool,
    pub max_bytes: usize,
}
impl Default for ClipboardCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            send: true,
            receive: true,
            max_bytes: 64 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputCfg {
    pub relative_mouse: bool,
    pub invert_scroll_y: bool,
    pub scroll_sensitivity: f64,
    pub sync_language: bool,
    pub prevent_sleep: bool,
}
impl Default for InputCfg {
    fn default() -> Self {
        Self {
            relative_mouse: false,
            invert_scroll_y: false,
            scroll_sensitivity: 1.0,
            sync_language: true,
            prevent_sleep: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwitchCfg {
    pub delay_ms: u64,
    pub double_tap_ms: u64,
    pub dead_corners: Vec<Corner>,
    pub corner_size_px: u32,
}
impl Default for SwitchCfg {
    fn default() -> Self {
        Self {
            delay_ms: 0,
            double_tap_ms: 0,
            dead_corners: Vec::new(),
            corner_size_px: 0,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HotkeysCfg {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_to_screen: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickNumbering {
    #[default]
    Native,
    Legacy,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MacosCfg {
    pub click_numbering: ClickNumbering,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub role: Role,
    pub name: String,
    #[serde(default)]
    pub server: ServerCfg,
    #[serde(default)]
    pub client: ClientCfg,
    #[serde(default)]
    pub tls: TlsCfg,
    #[serde(default)]
    pub clipboard: ClipboardCfg,
    #[serde(default)]
    pub input: InputCfg,
    #[serde(default, rename = "switch")]
    pub switch: SwitchCfg,
    #[serde(default)]
    pub hotkeys: HotkeysCfg,
    #[serde(default)]
    pub macos: MacosCfg,
    #[serde(default, rename = "screen")]
    pub screens: Vec<ScreenCfg>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            role: Role::Off,
            name: String::new(),
            server: ServerCfg::default(),
            client: ClientCfg::default(),
            tls: TlsCfg::default(),
            clipboard: ClipboardCfg::default(),
            input: InputCfg::default(),
            switch: SwitchCfg::default(),
            hotkeys: HotkeysCfg::default(),
            macos: MacosCfg::default(),
            screens: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Parse(String),
    Validation(String),
    NotImplemented,
    Io(String),
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ConfigError {}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    pub warnings: Vec<String>,
}

impl Config {
    pub fn load_toml(text: &str) -> Result<Self, ConfigError> {
        let cfg: Self = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }
    pub fn to_toml(&self) -> String {
        toml::to_string(self).unwrap_or_default()
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(ConfigError::Validation(format!(
                "unsupported config version {}",
                self.version
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for s in &self.screens {
            if s.name.is_empty() || !seen.insert(s.name.clone()) {
                return Err(ConfigError::Validation(format!(
                    "duplicate or empty screen name {:?}",
                    s.name
                )));
            }
        }
        Ok(())
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
    pub fn from_json(v: &serde_json::Value) -> Result<Self, ConfigError> {
        let cfg: Self = serde_json::from_value(v.clone()).map_err(|e| ConfigError::Parse(e.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }
}

/// RFC 7386 JSON merge patch. Full validation lands in WP2/WP6.
pub fn merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    match (target, patch) {
        (serde_json::Value::Object(t), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                if v.is_null() {
                    t.remove(k);
                } else {
                    merge_patch(t.entry(k.clone()).or_insert(serde_json::Value::Null), v);
                }
            }
        }
        (t, p) => {
            *t = p.clone();
        }
    }
}
pub fn apply_merge_patch(cfg: &Config, patch: &serde_json::Value) -> Result<(Config, Vec<String>), ConfigError> {
    let mut v = cfg.to_json();
    merge_patch(&mut v, patch);
    Ok((Config::from_json(&v)?, Vec::new()))
}
pub fn derive_links(_screens: &[ScreenCfg]) -> DerivedLinks {
    Vec::new()
}
pub fn import_synergy_conf(_text: &str) -> Result<(Config, ImportReport), ConfigError> {
    Err(ConfigError::NotImplemented)
}
pub fn import_synergy3_db(_json: &str, _core_ini: Option<&str>) -> Result<(Config, ImportReport), ConfigError> {
    Err(ConfigError::NotImplemented)
}
pub fn import_synergy_dir(_path: &Path) -> Result<(Config, ImportReport), ConfigError> {
    Err(ConfigError::NotImplemented)
}
pub fn export_synergy_conf(_cfg: &Config) -> Result<String, ConfigError> {
    Err(ConfigError::NotImplemented)
}

/// Command-line handlers for config subcommands of the `openergy` binary.
/// The daemon crate only dispatches to these; WP2 owns their behaviour.
pub mod cli {
    /// `openergy import-synergy --from <Synergy prefs dir> [--out config.toml]`
    pub fn import_synergy(_args: &[String]) -> Result<i32, String> {
        Err("import-synergy: not implemented yet (WP2)".into())
    }

    /// `openergy export-synergy-conf [--config <path>] [--out synergy.conf]`
    pub fn export_synergy_conf(_args: &[String]) -> Result<i32, String> {
        Err("export-synergy-conf: not implemented yet (WP2)".into())
    }
}
