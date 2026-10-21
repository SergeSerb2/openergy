//! Frozen platform traits (architecture-r2 §2.5) and the headless virtual backend.
mod click;
mod script;
pub mod virtual_backend;

pub use click::{ClickAssignment, ClickError};
pub use script::{InjectRequest, ScriptEvent};
use serde::{Deserialize, Serialize};
use std::fmt;
pub use virtual_backend::{VirtualConfig, VirtualControl, VirtualPlatform};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers(pub u16);
impl Modifiers {
    pub const SHIFT: u16 = 0x0001;
    pub const CONTROL: u16 = 0x0002;
    pub const ALT: u16 = 0x0004;
    pub const META: u16 = 0x0008;
    pub const SUPER: u16 = 0x0010;
    pub const ALTGR: u16 = 0x0020;
    pub const CAPS_LOCK: u16 = 0x1000;
    pub const NUM_LOCK: u16 = 0x2000;
    pub const SCROLL_LOCK: u16 = 0x4000;
    pub const NONE: Self = Self(0);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEvent {
    KeyDown {
        key: u16,
        modifiers: Modifiers,
        button: u16,
    },
    KeyRepeat {
        key: u16,
        modifiers: Modifiers,
        count: i16,
        button: u16,
    },
    KeyUp {
        key: u16,
        modifiers: Modifiers,
        button: u16,
    },
    MouseDown {
        button: u8,
    },
    MouseUp {
        button: u8,
    },
    MouseMoveAbs {
        x: i32,
        y: i32,
    },
    MouseMoveRel {
        dx: i32,
        dy: i32,
    },
    MouseWheel {
        dx: i32,
        dy: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureDecision {
    PassToOs,
    Swallow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grant {
    Granted,
    Denied,
    Unknown,
    NotRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub accessibility: Grant,
    pub input_monitoring: Grant,
    pub post_events: Grant,
}
impl Permissions {
    pub fn not_required() -> Self {
        Self {
            accessibility: Grant::NotRequired,
            input_monitoring: Grant::NotRequired,
            post_events: Grant::NotRequired,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    Accessibility,
    InputMonitoring,
    PostEvents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatKind {
    TextUtf8,
    HtmlUtf8,
    Dib,
    Png,
}
impl FormatKind {
    pub fn wire_id(self) -> Option<u32> {
        match self {
            Self::TextUtf8 => Some(0),
            Self::HtmlUtf8 => Some(1),
            Self::Dib => Some(2),
            Self::Png => None,
        }
    }
    pub fn from_wire_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::TextUtf8),
            1 => Some(Self::HtmlUtf8),
            2 => Some(Self::Dib),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextUtf8 => "text",
            Self::HtmlUtf8 => "html",
            Self::Dib => "dib",
            Self::Png => "png",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardFormat {
    pub kind: FormatKind,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClipboardData {
    pub formats: Vec<ClipboardFormat>,
}
impl ClipboardData {
    pub fn total_bytes(&self) -> usize {
        self.formats.iter().map(|f| f.bytes.len()).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub sequence: u64,
    pub event: Option<InputEvent>,
    pub release_all: bool,
    pub click_number: Option<i64>,
    pub click_state: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    PermissionDenied(PermissionKind),
    ClipboardTooLarge { bytes: usize, limit: usize },
    NotImplemented,
    Failed(String),
}
impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(message) => f.write_str(message),
            Self::NotImplemented => f.write_str("This platform feature is unavailable."),
            Self::ClipboardTooLarge { bytes, limit } => {
                write!(f, "Clipboard size {bytes} exceeds the {limit} byte limit.")
            }
            Self::PermissionDenied(kind) => f.write_str(match kind {
                PermissionKind::Accessibility => "Accessibility permission is required.",
                PermissionKind::InputMonitoring => "Input Monitoring permission is required.",
                PermissionKind::PostEvents => "Permission to send input is required.",
            }),
        }
    }
}
impl std::error::Error for PlatformError {}

pub trait InputSource: Send {
    fn start(&mut self, sink: Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>) -> Result<(), PlatformError>;
    fn stop(&mut self);
    fn set_cursor_captured(&mut self, captured: bool) -> Result<(), PlatformError>;
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<(), PlatformError>;
}
pub trait InputSink: Send {
    /// Update cached permission state outside the input event path.
    fn permissions_changed(&mut self, _permissions: &Permissions) {}
    fn inject(&mut self, ev: &InputEvent) -> Result<(), PlatformError>;
    fn release_all(&mut self);
    fn set_cursor_visible(&mut self, visible: bool);
}
pub trait ClipboardBackend: Send {
    fn change_stamp(&self) -> u64;
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError>;
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError>;
}
pub trait ScreenSystem: Send {
    fn screens(&self) -> Vec<ScreenRect>;
    fn cursor_pos(&self) -> (i32, i32);
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend;
    fn permissions(&self) -> Permissions;
    /// Backends may request an OS grant; the default returns current state.
    fn request_permission(&mut self, kind: PermissionKind) -> Result<Grant, PlatformError> {
        let permissions = self.permissions();
        Ok(match kind {
            PermissionKind::Accessibility => permissions.accessibility,
            PermissionKind::InputMonitoring => permissions.input_monitoring,
            PermissionKind::PostEvents => permissions.post_events,
        })
    }
    fn on_display_changed(&mut self, cb: Box<dyn FnMut() + Send>);
    fn prevent_sleep(&mut self, on: bool);
    fn secure_input_app(&self) -> Option<String> {
        None
    }
}

/// One backend split by thread ownership: capture owns source, I/O owns sink
/// and screens, and the clipboard worker owns clipboard.
pub struct Backend {
    pub name: &'static str,
    pub source: Box<dyn InputSource>,
    pub sink: Box<dyn InputSink>,
    pub screens: Box<dyn ScreenSystem>,
    pub clipboard: Box<dyn ClipboardBackend>,
}

/// Pure click-numbering model shared by the virtual backend (WP3) and macOS (WP5).
#[derive(Clone, Debug)]
pub struct ClickNumberer {
    pub last_number: i64,
    pub pressed_number: [Option<i64>; 8],
    pub pressed_click_state: [Option<i64>; 8],
    history: [click::ClickHistory; 8],
    double_click_ms: u64,
    click_distance: f64,
    exhaustion_logged: bool,
}
impl ClickNumberer {
    pub fn on_down(&mut self, button: u8, seed: i64) -> i64 {
        let base = self.last_number.max(seed);
        if base == i64::MAX {
            self.log_exhaustion();
        }
        let n = base.saturating_add(1);
        self.last_number = n;
        if let Some(slot) = self.pressed_number.get_mut(button as usize) {
            *slot = Some(n);
        }
        n
    }
    pub fn on_drag_or_up(&self, button: u8) -> Option<i64> {
        self.pressed_number.get(button as usize).copied().flatten()
    }
    pub fn on_up_clear(&mut self, button: u8) {
        if let Some(slot) = self.pressed_number.get_mut(button as usize) {
            *slot = None;
        }
        if let Some(slot) = self.pressed_click_state.get_mut(button as usize) {
            *slot = None;
        }
    }
}
