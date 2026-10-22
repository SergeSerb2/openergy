//! Frozen daemon<->app IPC types (architecture-r2 §2.10, ui-contract).
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

pub mod methods {
    pub const STATUS_GET: &str = "status.get";
    pub const CONFIG_GET: &str = "config.get";
    pub const CONFIG_PATCH: &str = "config.patch";
    pub const CONFIG_SET: &str = "config.set";
    pub const CONFIG_IMPORT_SYNERGY: &str = "config.import_synergy";
    pub const CONNECTION_CONNECT: &str = "connection.connect";
    pub const CONNECTION_DISCONNECT: &str = "connection.disconnect";
    pub const CONNECTION_RESTART: &str = "connection.restart";
    pub const TRUST_LIST: &str = "trust.list";
    pub const TRUST_APPROVE: &str = "trust.approve";
    pub const TRUST_REJECT: &str = "trust.reject";
    pub const TRUST_REVOKE: &str = "trust.revoke";
    pub const DISCOVERY_LIST: &str = "discovery.list";
    pub const PERMISSIONS_REQUEST: &str = "permissions.request";
    pub const PERMISSIONS_OPEN_SETTINGS: &str = "permissions.open_settings";
    pub const LOGS_TAIL: &str = "logs.tail";
    pub const LOGS_SUBSCRIBE: &str = "logs.subscribe";
    pub const LOGS_UNSUBSCRIBE: &str = "logs.unsubscribe";
    pub const EVENTS_SUBSCRIBE: &str = "events.subscribe";
    pub const DAEMON_STOP: &str = "daemon.stop";
    pub const VIRTUAL_INJECT: &str = "virtual.inject";
    pub const VIRTUAL_RECORDED: &str = "virtual.recorded";
    pub const VIRTUAL_CLIPBOARD: &str = "virtual.clipboard";
    pub const VIRTUAL_SET_SCREENS: &str = "virtual.set_screens";
    pub const VIRTUAL_SET_PERMISSIONS: &str = "virtual.set_permissions";
}
pub mod events {
    pub const STATUS_CHANGED: &str = "status.changed";
    pub const CONFIG_CHANGED: &str = "config.changed";
    pub const DISCOVERY_CHANGED: &str = "discovery.changed";
    pub const PEER_CONNECTED: &str = "peer.connected";
    pub const PEER_DISCONNECTED: &str = "peer.disconnected";
    pub const PEER_UNTRUSTED: &str = "peer.untrusted";
    pub const CLIPBOARD_BLOCKED: &str = "clipboard.blocked";
    pub const PERMISSIONS_CHANGED: &str = "permissions.changed";
    pub const LOG_LINE: &str = "log.line";
    pub const ERROR: &str = "error";
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
}
impl IpcError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}
impl Response {
    pub fn ok(id: u64, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }
    pub fn err(id: u64, error: IpcError) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DaemonInfo {
    pub version: String,
    pub pid: u32,
    pub uptime_s: u64,
    pub platform: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CursorInfo {
    pub screen: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scale: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalScreenInfo {
    pub w: Option<i32>,
    pub h: Option<i32>,
    pub displays: Vec<DisplayInfo>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub listen: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    pub remote: Option<String>,
    pub next_retry_ms: Option<u64>,
    pub attempt: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerProtocol {
    pub product: Option<String>,
    pub major: Option<u16>,
    pub minor: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerScreen {
    pub w: Option<i32>,
    pub h: Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerTls {
    pub enabled: bool,
    pub version: Option<String>,
    pub fingerprint: Option<String>,
    pub trusted: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    pub name: String,
    pub address: Option<String>,
    pub role: Option<String>,
    pub connected: bool,
    pub since_unix_ms: Option<u64>,
    pub rtt_ms: Option<f64>,
    pub protocol: PeerProtocol,
    pub screen: PeerScreen,
    pub tls: PeerTls,
    pub secure_input_app: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TlsInfo {
    pub enabled: bool,
    pub fingerprint: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionsInfo {
    pub accessibility: String,
    pub input_monitoring: String,
    pub post_events: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecureInputInfo {
    pub local_app: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardTransfer {
    pub direction: Option<String>,
    pub bytes: Option<usize>,
    pub formats: Vec<String>,
    pub unix_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardBlocked {
    pub bytes: Option<usize>,
    pub limit: Option<usize>,
    pub peer: Option<String>,
    pub unix_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardInfo {
    pub enabled: bool,
    pub last_transfer: Option<ClipboardTransfer>,
    pub last_blocked: Option<ClipboardBlocked>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingTrust {
    pub fingerprint: String,
    pub address: Option<String>,
    pub name: Option<String>,
    pub expires_unix_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LastError {
    pub code: String,
    pub message: String,
    pub unix_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatsInfo {
    pub rss_bytes: Option<u64>,
    pub events_in: u64,
    pub events_out: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub daemon: DaemonInfo,
    pub role: String,
    pub state: String,
    pub name: Option<String>,
    pub active_screen: Option<String>,
    pub cursor: CursorInfo,
    pub local_screen: LocalScreenInfo,
    pub server: ServerInfo,
    pub client: ClientInfo,
    pub peers: Vec<PeerInfo>,
    pub tls: TlsInfo,
    pub permissions: PermissionsInfo,
    pub secure_input: SecureInputInfo,
    pub clipboard: ClipboardInfo,
    pub pending_trust: Vec<PendingTrust>,
    pub last_error: Option<LastError>,
    pub stats: StatsInfo,
}
impl Status {
    pub fn idle(version: &str, pid: u32, platform: &str) -> Self {
        Self {
            daemon: DaemonInfo {
                version: version.into(),
                pid,
                uptime_s: 0,
                platform: platform.into(),
            },
            role: "off".into(),
            state: "idle".into(),
            name: None,
            active_screen: None,
            cursor: CursorInfo {
                screen: None,
                x: None,
                y: None,
            },
            local_screen: LocalScreenInfo {
                w: None,
                h: None,
                displays: Vec::new(),
            },
            server: ServerInfo { listen: None },
            client: ClientInfo {
                remote: None,
                next_retry_ms: None,
                attempt: None,
            },
            peers: Vec::new(),
            tls: TlsInfo {
                enabled: false,
                fingerprint: None,
            },
            permissions: PermissionsInfo {
                accessibility: "unknown".into(),
                input_monitoring: "unknown".into(),
                post_events: "unknown".into(),
            },
            secure_input: SecureInputInfo { local_app: None },
            clipboard: ClipboardInfo {
                enabled: true,
                last_transfer: None,
                last_blocked: None,
            },
            pending_trust: Vec::new(),
            last_error: None,
            stats: StatsInfo {
                rss_bytes: None,
                events_in: 0,
                events_out: 0,
            },
        }
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

use std::io::{Read, Write};
pub const MAX_FRAME_LEN: usize = 16 * 1024 * 1024;
pub fn write_frame<W: Write>(w: &mut W, value: &serde_json::Value) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(std::io::Error::other)?;
    if bytes.len() > MAX_FRAME_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "ipc frame too large",
        ));
    }
    let len = u32::try_from(bytes.len())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"))?;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&bytes)?;
    w.flush()
}
pub fn read_frame<R: Read>(r: &mut R) -> std::io::Result<serde_json::Value> {
    let mut hdr = [0u8; 4];
    r.read_exact(&mut hdr)?;
    let len = u32::from_be_bytes(hdr) as usize;
    if len > MAX_FRAME_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "ipc frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    serde_json::from_slice(&buf).map_err(std::io::Error::other)
}
