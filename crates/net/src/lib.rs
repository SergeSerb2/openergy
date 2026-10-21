//! Synchronous transport, identity, pin trust, and discovery.
//!
//! The daemon io thread owns the driver and its single `mio::Poll`. Cross-thread commands use
//! [`NetHandle`] and its mio waker. No transport thread or async runtime is created.
pub mod cli;
mod connect_error;
pub use connect_error::ConnectErrorKind;
pub mod discovery;
mod identity;
mod reactor;
mod tls;
mod trust;
pub use identity::{default_config_dir, Identity};
use openergy_protocol::Message;
use openergy_session::{ConnId, DisconnectReason};
pub use reactor::{ConnInfo, MdnsConfig, NetConfig, NetDriver, TransportMode, BULK_SLICE};
use std::{fmt, net::SocketAddr, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub [u8; 32]);
impl Fingerprint {
    pub fn from_der(der: &[u8]) -> Self {
        use sha2::Digest;
        let d = sha2::Sha256::digest(der);
        let mut out = [0u8; 32];
        out.copy_from_slice(&d);
        Self(out)
    }
    pub fn to_colon_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":")
    }
    pub fn to_synergy_line(&self) -> String {
        format!(
            "v2:sha256:{}",
            self.0.iter().map(|b| format!("{b:02x}")).collect::<String>()
        )
    }
    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim();
        let hex = t.strip_prefix("v2:sha256:").unwrap_or(t);
        let clean: String = if hex.len() == 64 {
            hex.to_owned()
        } else if hex.len() == 95 && hex.split(':').all(|part| part.len() == 2) {
            hex.replace(':', "")
        } else {
            return None;
        };
        if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(clean.get(i * 2..i * 2 + 2)?, 16).ok()?;
        }
        Some(Self(out))
    }
}
impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_colon_hex())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredPeer {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub fingerprint_hint: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Input,
    Control,
    Bulk,
}

#[derive(Debug)]
pub enum NetCommand {
    Listen {
        addr: SocketAddr,
    },
    Connect {
        host: String,
        port: u16,
    },
    Send {
        conn: ConnId,
        msg: Message,
        priority: Priority,
    },
    Close {
        conn: ConnId,
    },
    /// Pin a specific outbound identity. A mismatch fails the TLS handshake.
    ConnectPinned {
        host: String,
        port: u16,
        fingerprint: Fingerprint,
    },
    /// Diagnostic frame mode only: payload excludes its u32 length prefix.
    SendFrame {
        conn: ConnId,
        payload: Vec<u8>,
        priority: Priority,
    },
    /// Persist approval in the correct role's file, then resume the held peer.
    Approve {
        conn: ConnId,
    },
    Reject {
        conn: ConnId,
    },
    ReloadTrust,
    /// Stop accepting new peers without touching established connections.
    StopListening,
    /// Reset transport state. The driver, handle, and waker remain reusable.
    Shutdown,
    /// Close connections while keeping the current listener and driver reusable.
    DisconnectAll,
}
#[derive(Debug)]
pub enum NetEvent {
    Listening {
        addr: SocketAddr,
    },
    Connected {
        conn: ConnId,
        peer: SocketAddr,
        fingerprint: Option<Fingerprint>,
        tls: bool,
        tls_version: Option<String>,
    },
    Message {
        conn: ConnId,
        msg: Message,
    },
    Disconnected {
        conn: ConnId,
        reason: DisconnectReason,
    },
    UntrustedPeer {
        conn: ConnId,
        fingerprint: Fingerprint,
        addr: SocketAddr,
    },
    ListenFailed {
        addr: SocketAddr,
        error: String,
    },
    ConnectFailed {
        host: String,
        port: u16,
        error: String,
    },
    Discovered(Vec<DiscoveredPeer>),
    /// Available only when explicitly configured with TransportMode::Frames.
    Frame {
        conn: ConnId,
        payload: Vec<u8>,
    },
    /// Command/connect errors with no established peer also reach the daemon.
    Error {
        conn: Option<ConnId>,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    NotImplemented,
    Failed(String),
    QueueFull,
    InvalidFrame,
    ProductionPortDenied,
    Shutdown,
}
impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented => f.write_str("This transport is unavailable"),
            Self::Failed(message) => f.write_str(message),
            Self::QueueFull => f.write_str("Network queue is full"),
            Self::InvalidFrame => f.write_str("Invalid network frame"),
            Self::ProductionPortDenied => f.write_str("Production port requires explicit authorization"),
            Self::Shutdown => f.write_str("Transport is stopped"),
        }
    }
}
impl std::error::Error for NetError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustError {
    NotImplemented,
    Failed(String),
}
impl fmt::Display for TrustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented => f.write_str("This trust operation is unavailable."),
            Self::Failed(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for TrustError {}

/// A cloneable bounded command mailbox. Successful submission wakes the io loop.
#[derive(Clone)]
pub struct NetHandle {
    inner: std::sync::Arc<reactor::Mailbox>,
}
impl NetHandle {
    pub fn send(&self, cmd: NetCommand) -> Result<(), NetError> {
        self.inner.submit(cmd)
    }
}

/// TrustStore paths refer to the TLS directory, not its parent config directory.
pub struct TrustStore {
    dir: PathBuf,
    servers: Vec<Fingerprint>,
    clients: Vec<Fingerprint>,
}
