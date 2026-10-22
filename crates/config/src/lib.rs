//! Config model, layout derivation, and Synergy migration (architecture-r2 §2.8).
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod cli;
mod layout;
mod model;
mod synergy;
mod synergy3;
pub use layout::derive_links;
pub use model::{apply_merge_patch, merge_patch};
pub use synergy::{export_synergy_conf, import_synergy_conf};
pub use synergy3::{import_synergy3_db, import_synergy_dir};

/// Maximum serialized daemon TOML size, shared by startup and config writers.
pub const MAX_CONFIG_FILE_LEN: usize = 16 * 1024 * 1024;

/// Maximum clipboard payload supported by the session's transfer implementation.
pub const MAX_CLIPBOARD_BYTES: usize = 64 * 1024 * 1024;

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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct DerivedLink {
    pub from: String,
    pub edge: Edge,
    pub from_range: (u8, u8),
    pub to: String,
    pub to_range: (u8, u8),
}
pub type DerivedLinks = Vec<DerivedLink>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LinksCfg {
    /// Ranged overrides. A direction in this list overrides geometric derivation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranges: Vec<RangedLink>,
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
#[serde(default, deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ScreenOptions>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
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
            max_bytes: MAX_CLIPBOARD_BYTES,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SwitchCfg {
    pub delay_ms: u64,
    pub double_tap_ms: u64,
    pub dead_corners: Vec<Corner>,
    pub corner_size_px: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
pub struct MacosCfg {
    pub click_numbering: ClickNumbering,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "SynergyOptions::is_default")]
    pub options: SynergyOptions,
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
            options: SynergyOptions::default(),
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
        match self {
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::Validation(message) => write!(f, "invalid config: {message}"),
            Self::Io(message) => write!(f, "I/O error: {message}"),
            Self::NotImplemented => f.write_str("config operation is not implemented"),
        }
    }
}
impl std::error::Error for ConfigError {}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportReport {
    pub warnings: Vec<String>,
    /// Candidate pins only. Importing never modifies a trust store.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_servers: Vec<String>,
}

/// An explicit half-open percentage mapping on one screen edge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RangedLink {
    pub edge: Edge,
    pub from_range: (u8, u8),
    pub to: String,
    pub to_range: (u8, u8),
}

/// Optional overrides retain global fallback semantics for each screen.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScreenOptions {
    pub half_duplex_caps_lock: Option<bool>,
    pub half_duplex_num_lock: Option<bool>,
    pub half_duplex_scroll_lock: Option<bool>,
    pub dead_corners: Option<Vec<Corner>>,
    pub corner_size_px: Option<u32>,
    pub xtest_is_xinerama_unaware: Option<bool>,
    pub preserve_focus: Option<bool>,
}

/// Synergy controls not represented by the initial UI fields.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SynergyOptions {
    pub barrier: bool,
    pub win32_keep_foreground: bool,
    pub disable_lock_to_screen: bool,
    pub default_lock_to_screen: bool,
    pub switch_needs_shift: bool,
    pub switch_needs_control: bool,
    pub switch_needs_alt: bool,
}
impl SynergyOptions {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}
