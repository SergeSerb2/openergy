//! Deterministic session machines. All time, I/O, and platform effects enter as events.

use openergy_config::Config;
use openergy_ipc::Event as IpcEvent;
use openergy_platform::{ClipboardData, InputEvent};
use openergy_protocol::{Chunk, ClipboardId, FileDataChunk, Message};
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
    ClientConnected {
        conn: ConnId,
    },
    ClientMessage {
        conn: ConnId,
        msg: Message,
    },
    ClientDisconnected {
        conn: ConnId,
        reason: DisconnectReason,
    },
    LocalInput(InputEvent),
    LocalLanguages(openergy_platform::KeyboardLanguages),
    LocalClipboardChanged {
        id: ClipboardId,
    },
    ClipboardPayload {
        id: ClipboardId,
        data: ClipboardData,
    },
    // A requested local read failed, was blocked, or could not be queued.
    ClipboardReadFailed {
        id: ClipboardId,
    },
    TimerTick {
        now_ms: u64,
    },
    ConfigChanged(Arc<Config>),
    DisplayChanged,
    LocalScreen {
        rect: layout::Rect,
        cursor: (i32, i32),
        displays: Vec<layout::Rect>,
    },
    ConnectionMetadata {
        conn: ConnId,
        fingerprint: Option<String>,
        certificate_verified: bool,
    },
    FileSendRequested {
        xfer: u64,
        conn: ConnId,
        transfer: u32,
        peer_name: String,
        fingerprint: Option<String>,
        kind: u8,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
    },
    FileChunkReady {
        xfer: u64,
        bytes: Vec<u8>,
    },
    FileChunkFailed {
        xfer: u64,
        reason: file::Reason,
    },
    FileCredit {
        xfer: u64,
        chunks: u32,
    },
    FileOfferDecided {
        xfer: u64,
        accept: bool,
        reason: file::Reason,
        resume_offset: u64,
    },
    FileStageResult {
        xfer: u64,
        ok: bool,
        reason: file::Reason,
    },
    FileFinishResult {
        xfer: u64,
        ok: bool,
        reason: file::Reason,
    },
    FileCancelRequested {
        xfer: u64,
    },
    FileBulkAborted {
        xfer: u64,
    },
    PermissionLost,
    CaptureOverflow,
    LocalSecureInput {
        app: Option<String>,
    },
    LocalScreenSaver {
        on: bool,
    },
    Shutdown,
}
#[derive(Debug)]
pub enum ClientEvent {
    LocalLanguages(openergy_platform::KeyboardLanguages),
    ServerConnected {
        conn: ConnId,
    },
    ServerMessage {
        conn: ConnId,
        msg: Message,
    },
    ServerDisconnected {
        conn: ConnId,
        reason: DisconnectReason,
    },
    LocalClipboardChanged {
        id: ClipboardId,
    },
    ClipboardPayload {
        id: ClipboardId,
        data: ClipboardData,
    },
    // A requested local read failed, was blocked, or could not be queued.
    ClipboardReadFailed {
        id: ClipboardId,
    },
    TimerTick {
        now_ms: u64,
    },
    ConfigChanged(Arc<Config>),
    DisplayChanged,
    LocalScreen {
        rect: layout::Rect,
        cursor: (i32, i32),
        displays: Vec<layout::Rect>,
    },
    PermissionLost,
    ReconnectTimer,
    Shutdown,
    ConnectionMetadata {
        conn: ConnId,
        fingerprint: Option<String>,
        certificate_verified: bool,
    },
    FileSendRequested {
        xfer: u64,
        conn: ConnId,
        transfer: u32,
        peer_name: String,
        fingerprint: Option<String>,
        kind: u8,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
    },
    FileChunkReady {
        xfer: u64,
        bytes: Vec<u8>,
    },
    FileChunkFailed {
        xfer: u64,
        reason: file::Reason,
    },
    FileCredit {
        xfer: u64,
        chunks: u32,
    },
    FileOfferDecided {
        xfer: u64,
        accept: bool,
        reason: file::Reason,
        resume_offset: u64,
    },
    FileStageResult {
        xfer: u64,
        ok: bool,
        reason: file::Reason,
    },
    FileFinishResult {
        xfer: u64,
        ok: bool,
        reason: file::Reason,
    },
    FileCancelRequested {
        xfer: u64,
    },
    FileBulkAborted {
        xfer: u64,
    },
}

#[derive(Debug)]
pub enum Action {
    /// Cleanup must bypass input backpressure and precede close.
    SendControl {
        conn: ConnId,
        msg: Message,
    },
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
    SendFileChunk {
        conn: ConnId,
        transfer: u32,
        chunk: FileDataChunk,
    },
    ReadFileChunk {
        xfer: u64,
        offset: u64,
        len: u32,
    },
    StageFileChunk {
        xfer: u64,
        offset: u64,
        bytes: Vec<u8>,
    },
    FinishFile {
        xfer: u64,
    },
    DropFile {
        xfer: u64,
        keep_part: bool,
    },
    PurgeFileBulk {
        conn: ConnId,
        xfer: u64,
    },
    FileOffered {
        xfer: u64,
        conn: ConnId,
        kind: u8,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
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
    SetKeyboardLanguage([u8; 2]),
    SetClipboardShared {
        id: ClipboardId,
        data: Arc<ClipboardData>,
    },
    SetClipboard {
        id: ClipboardId,
        data: ClipboardData,
    },
    ReadClipboard {
        id: ClipboardId,
        max_bytes: usize,
    },
    HideLocalCursor(bool),
    WarpLocalCursor {
        x: i32,
        y: i32,
    },
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

mod client;
pub mod clipboard;
mod common;
pub mod file;
mod hotkey;
pub mod layout;
mod server;
pub use client::Client;
pub use common::{PeerSnapshot, SessionSnapshot};
pub use server::Server;

pub(crate) fn notify(event: &str, data: serde_json::Value, out: &mut Vec<Action>) {
    out.push(Action::Notify(IpcEvent {
        event: event.into(),
        data,
    }));
}
