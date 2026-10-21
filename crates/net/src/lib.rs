//! Frozen net surface (architecture-r2 §2.7/§2.9). Transport bodies land in WP4.
#![allow(dead_code)]
use openergy_protocol::Message;
use openergy_session::{ConnId, DisconnectReason};
use std::{
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
};

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
        let clean: String = hex.chars().filter(|c| *c != ':').collect();
        if clean.len() != 64 || !clean.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(clean.get(i * 2..i * 2 + 2)?, 16).ok()?;
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
    Shutdown,
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
    Discovered(Vec<DiscoveredPeer>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    NotImplemented,
    Failed(String),
}
impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
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
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TrustError {}

pub struct TrustStore {
    dir: PathBuf,
    servers: Vec<Fingerprint>,
    clients: Vec<Fingerprint>,
}
impl TrustStore {
    pub fn load(dir: &Path) -> Result<Self, TrustError> {
        Ok(Self {
            dir: dir.to_path_buf(),
            servers: Vec::new(),
            clients: Vec::new(),
        })
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn is_trusted_server(&self, fp: &Fingerprint) -> bool {
        self.servers.contains(fp)
    }
    pub fn is_trusted_client(&self, fp: &Fingerprint) -> bool {
        self.clients.contains(fp)
    }
    pub fn trust_server(&mut self, _fp: Fingerprint) -> Result<(), TrustError> {
        Err(TrustError::NotImplemented)
    }
    pub fn trust_client(&mut self, _fp: Fingerprint) -> Result<(), TrustError> {
        Err(TrustError::NotImplemented)
    }
    pub fn revoke(&mut self, _fp: &Fingerprint) -> Result<(), TrustError> {
        Err(TrustError::NotImplemented)
    }
    pub fn servers(&self) -> &[Fingerprint] {
        &self.servers
    }
    pub fn clients(&self) -> &[Fingerprint] {
        &self.clients
    }
}

pub struct NetHandle {
    _private: (),
}
impl NetHandle {
    pub fn send(&self, _cmd: NetCommand) -> Result<(), NetError> {
        Err(NetError::NotImplemented)
    }
}
