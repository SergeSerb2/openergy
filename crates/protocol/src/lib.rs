//! Bounded, incremental Synergy/Barrier wire codec (architecture-r2 §2.3).
//!
//! `encode` appends one complete frame to its output. Reuse and clear that
//! buffer between writes, or append multiple frames for one socket write.
//! `Decoder::feed` checks frame lengths before copying payload bytes; drain
//! `next_message` after each feed and close the connection on any error.
//! Each connection starts with Hello or HelloBack. The decoder classifies it
//! from the first frame; a later hello is an error. The hello version enables
//! decoding legacy key repeats without a language suffix for wire 1.1–1.7.
//! Version/role acceptance policy and clipboard transfer reassembly belong
//! to session, outside this codec.
#![forbid(unsafe_code)]

mod clipboard;
mod codec;
pub mod keys;
pub use clipboard::{marshal_clipboard, unmarshal_clipboard};
pub use codec::{encode, Decoder};
use std::fmt;

pub const DEFAULT_PORT: u16 = 24800;
pub const MAX_FRAME_LEN: u32 = 4 * 1024 * 1024;
pub const MAX_STRING_LEN: u32 = 1024 * 1024;
pub const MAX_HELLO_BACK_LEN: usize = 1024;
pub const MAX_VECTOR_LEN: u32 = 1_048_576;
pub const MAX_LANGUAGE_LEN: usize = 64;
pub const MAX_SECURE_INPUT_LEN: usize = 255;
pub const CLIPBOARD_CHUNK_LEN: usize = 32 * 1024;
pub const WIRE_MAJOR: u16 = 1;
pub const WIRE_MINOR: u16 = 8;
pub const PRODUCT_SYNERGY: &str = "Synergy";
pub const PRODUCT_BARRIER: &str = "Barrier";

pub mod options {
    pub const HART: u32 = HEARTBEAT;
    pub const SSCM: u32 = SWITCH_CORNERS;
    pub const SSCS: u32 = SWITCH_CORNER_SIZE;
    pub const SSWT: u32 = SWITCH_DELAY;
    pub const SSTT: u32 = SWITCH_DOUBLE_TAP;
    pub const SSNS: u32 = SWITCH_NEEDS_SHIFT;
    pub const SSNC: u32 = SWITCH_NEEDS_CONTROL;
    pub const SSNA: u32 = SWITCH_NEEDS_ALT;
    pub const MDLT: u32 = RELATIVE_MOUSE_MOVES;
    pub const _KFW: u32 = WIN32_KEEP_FOREGROUND;
    pub const LTSS: u32 = DEFAULT_LOCK_TO_SCREEN_STATE;
    pub const DLTS: u32 = DISABLE_LOCK_TO_SCREEN;
    pub const CLPS: u32 = CLIPBOARD_SHARING;
    pub const CLSZ: u32 = CLIPBOARD_SHARING_SIZE;
    pub const HDCL: u32 = HALF_DUPLEX_CAPS_LOCK;
    pub const HDNL: u32 = HALF_DUPLEX_NUM_LOCK;
    pub const HDSL: u32 = HALF_DUPLEX_SCROLL_LOCK;
    pub const MMFS: u32 = MOD_SHIFT;
    pub const MMFC: u32 = MOD_CTRL;
    pub const MMFA: u32 = MOD_ALT;
    pub const MMFG: u32 = MOD_ALTGR;
    pub const MMFM: u32 = MOD_META;
    pub const MMFR: u32 = MOD_SUPER;
    pub const XTXU: u32 = XTEST_XINERAMA_UNAWARE;
    pub const SFOC: u32 = PRESERVE_FOCUS;

    pub const HEARTBEAT: u32 = 0x4841_5254;
    pub const SWITCH_CORNERS: u32 = 0x5353_434D;
    pub const SWITCH_CORNER_SIZE: u32 = 0x5353_4353;
    pub const SWITCH_DELAY: u32 = 0x5353_5754;
    pub const SWITCH_DOUBLE_TAP: u32 = 0x5353_5454;
    pub const SWITCH_NEEDS_SHIFT: u32 = 0x5353_4E53;
    pub const SWITCH_NEEDS_CONTROL: u32 = 0x5353_4E43;
    pub const SWITCH_NEEDS_ALT: u32 = 0x5353_4E41;
    pub const RELATIVE_MOUSE_MOVES: u32 = 0x4D44_4C54;
    pub const WIN32_KEEP_FOREGROUND: u32 = 0x5F4B_4657;
    pub const DEFAULT_LOCK_TO_SCREEN_STATE: u32 = 0x4C54_5353;
    pub const DISABLE_LOCK_TO_SCREEN: u32 = 0x444C_5453;
    pub const CLIPBOARD_SHARING: u32 = 0x434C_5053;
    pub const CLIPBOARD_SHARING_SIZE: u32 = 0x434C_535A;
    pub const HALF_DUPLEX_CAPS_LOCK: u32 = 0x4844_434C;
    pub const HALF_DUPLEX_NUM_LOCK: u32 = 0x4844_4E4C;
    pub const HALF_DUPLEX_SCROLL_LOCK: u32 = 0x4844_534C;
    pub const MOD_SHIFT: u32 = 0x4D4D_4653;
    pub const MOD_CTRL: u32 = 0x4D4D_4643;
    pub const MOD_ALT: u32 = 0x4D4D_4641;
    pub const MOD_ALTGR: u32 = 0x4D4D_4647;
    pub const MOD_META: u32 = 0x4D4D_464D;
    pub const MOD_SUPER: u32 = 0x4D4D_4652;
    pub const XTEST_XINERAMA_UNAWARE: u32 = 0x5854_5855;
    pub const PRESERVE_FOCUS: u32 = 0x5346_4F43;
    pub const CORNER_TOP_LEFT: i32 = 1;
    pub const CORNER_TOP_RIGHT: i32 = 2;
    pub const CORNER_BOTTOM_LEFT: i32 = 4;
    pub const CORNER_BOTTOM_RIGHT: i32 = 8;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}
impl ProtocolVersion {
    pub const V1_8: Self = Self { major: 1, minor: 8 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClipboardId(pub u8);
impl ClipboardId {
    pub const CLIPBOARD: Self = Self(0);
    pub const SELECTION: Self = Self(1);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chunk {
    Start { total_len: u32 },
    Data(Vec<u8>),
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Hello {
        product: String,
        version: ProtocolVersion,
    },
    HelloBack {
        product: String,
        version: ProtocolVersion,
        name: String,
    },
    QueryInfo,
    Noop,
    Bye,
    InfoAck,
    KeepAlive,
    ResetOptions,
    Leave,
    Enter {
        x: i16,
        y: i16,
        sequence: u32,
        modifiers: u16,
    },
    ScreenSaver {
        on: bool,
    },
    ClipboardGrab {
        id: ClipboardId,
        sequence: u32,
    },
    SetOptions {
        options: Vec<(u32, i32)>,
    },
    LanguageSync {
        languages: String,
    },
    SecureInputNotification {
        name: String,
    },
    KeyDown {
        key: u16,
        modifiers: u16,
        button: u16,
        lang: Option<String>,
    },
    KeyRepeat {
        key: u16,
        modifiers: u16,
        count: u16,
        button: u16,
        lang: String,
    },
    KeyUp {
        key: u16,
        modifiers: u16,
        button: u16,
    },
    MouseDown {
        button: u8,
    },
    MouseUp {
        button: u8,
    },
    MouseMoveAbs {
        x: i16,
        y: i16,
    },
    MouseMoveRel {
        dx: i16,
        dy: i16,
    },
    MouseWheel {
        dx: i16,
        dy: i16,
    },
    ScreenInfo {
        x: i16,
        y: i16,
        w: i16,
        h: i16,
        warp: i16,
        mx: i16,
        my: i16,
    },
    ClipboardData {
        id: ClipboardId,
        sequence: u32,
        chunk: Chunk,
    },
    FileTransfer {
        mark: u8,
        payload: Vec<u8>,
    },
    DragInfo {
        count: i16,
        payload: String,
    },
    ErrorIncompatible {
        major: u16,
        minor: u16,
    },
    ErrorBusy,
    ErrorUnknown,
    ErrorBad,
    Unknown {
        code: [u8; 4],
        /// Bytes after the code, excluding the outer length prefix.
        payload_len: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    OversizedFrame(u32),
    BadHello,
    Truncated,
    BadUtf8,
    BadValue(&'static str),
    NotImplemented,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    BadValue(&'static str),
    NotImplemented,
}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DecodeError {}
impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EncodeError {}
