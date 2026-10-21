//! Frozen session state-machine shapes (architecture-r2 §2.4). Bodies land in WP6.
#![allow(dead_code)]
use openergy_config::Config;
use openergy_ipc::Event as IpcEvent;
use openergy_platform::{ClipboardData, InputEvent};
use openergy_protocol::{Chunk, ClipboardId, Message};
use std::{fmt, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnId(pub u64);
pub type ScreenId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    ClosedByPeer,
    HeartbeatTimeout,
    TlsFailure,
    BadFrame,
    SlowPeer,
    Flood,
    Replaced,
    Shutdown,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseReason {
    Normal,
    BadFrame,
    SlowPeer,
    Flood,
    Replaced,
    Shutdown,
    UntrustedRejected,
}

#[derive(Debug)]
pub enum ServerEvent {
    ClientConnected { conn: ConnId },
    ClientMessage { conn: ConnId, msg: Message },
    ClientDisconnected { conn: ConnId, reason: DisconnectReason },
    LocalInput(InputEvent),
    LocalClipboardChanged { id: ClipboardId },
    ClipboardPayload { id: ClipboardId, data: ClipboardData },
    TimerTick { now_ms: u64 },
    ConfigChanged(Arc<Config>),
    DisplayChanged,
}
#[derive(Debug)]
pub enum ClientEvent {
    ServerConnected { conn: ConnId },
    ServerMessage { conn: ConnId, msg: Message },
    ServerDisconnected { conn: ConnId, reason: DisconnectReason },
    LocalClipboardChanged { id: ClipboardId },
    ClipboardPayload { id: ClipboardId, data: ClipboardData },
    TimerTick { now_ms: u64 },
    ConfigChanged(Arc<Config>),
    DisplayChanged,
}

#[derive(Debug)]
pub enum Action {
    Send {
        conn: ConnId,
        msg: Message,
    },
    SendClipboardChunk {
        conn: ConnId,
        id: ClipboardId,
        sequence: u32,
        chunk: Chunk,
    },
    Close {
        conn: ConnId,
        reason: CloseReason,
    },
    ReplaceConnection {
        old: ConnId,
        new: ConnId,
    },
    Inject(InputEvent),
    SetClipboard {
        id: ClipboardId,
        data: ClipboardData,
    },
    ReadClipboard {
        id: ClipboardId,
        max_bytes: usize,
    },
    HideLocalCursor(bool),
    ReleaseAllLocal,
    Notify(IpcEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    NotImplemented,
}
impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SessionError {}

pub mod layout {
    use openergy_config::{Config, Edge};
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub w: i32,
        pub h: i32,
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Link {
        pub from: usize,
        pub edge: Edge,
        pub from_range: (u8, u8),
        pub to: usize,
        pub to_range: (u8, u8),
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LayoutError {
        Validation(String),
        NotImplemented,
    }
    pub struct Layout {
        links: Vec<Link>,
    }
    impl Layout {
        pub fn from_config(_cfg: &Config) -> Result<Self, LayoutError> {
            Err(LayoutError::NotImplemented)
        }
        pub fn switch_target(
            &mut self,
            _current: usize,
            _x: i32,
            _y: i32,
            _dx: i32,
            _dy: i32,
            _now_ms: u64,
        ) -> Option<(usize, i32, i32)> {
            None
        }
        pub fn links(&self) -> &[Link] {
            &self.links
        }
    }
}

pub struct Server {
    config: Arc<Config>,
}
impl Server {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn handle(&mut self, _ev: ServerEvent, _out: &mut Vec<Action>) {}
}
pub struct Client {
    config: Arc<Config>,
    screen_name: String,
}
impl Client {
    pub fn new(config: Arc<Config>, screen_name: String) -> Self {
        Self { config, screen_name }
    }
    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn screen_name(&self) -> &str {
        &self.screen_name
    }
    pub fn handle(&mut self, _ev: ClientEvent, _out: &mut Vec<Action>) {}
}
