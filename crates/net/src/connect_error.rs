//! Stable classifications for connector failures, independent of OS error wording.
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectErrorKind {
    Refused,
    Timeout,
    Unreachable,
    Dns,
    PinMismatch,
    Tls,
    Other,
}
impl ConnectErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::Refused => "connect_refused",
            Self::Timeout => "connect_timeout",
            Self::Unreachable => "host_unreachable",
            Self::Dns => "dns_failed",
            Self::PinMismatch => "tls_pin_mismatch",
            Self::Tls => "tls_error",
            Self::Other => "connect_failed",
        }
    }
    pub fn from_io(error: &io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::ConnectionRefused => Self::Refused,
            io::ErrorKind::TimedOut => Self::Timeout,
            io::ErrorKind::HostUnreachable | io::ErrorKind::NetworkUnreachable => Self::Unreachable,
            _ => Self::Other,
        }
    }
    pub fn from_message(message: &str) -> Self {
        let message = message.to_ascii_lowercase();
        if message.contains("tls_pin_mismatch") || message.contains("fingerprint mismatch") {
            Self::PinMismatch
        } else if message.contains("connect_refused") || message.contains("connection refused") {
            Self::Refused
        } else if message.contains("connect_timeout") || message.contains("connect timeout") {
            Self::Timeout
        } else if message.contains("host_unreachable")
            || message.contains("no route to host")
            || message.contains("network is unreachable")
        {
            Self::Unreachable
        } else if message.contains("dns_failed") {
            Self::Dns
        } else if message.contains("tls_error") {
            Self::Tls
        } else {
            Self::Other
        }
    }
}
pub(crate) fn io_failure(error: io::Error) -> crate::NetError {
    crate::NetError::Failed(format!("{}: {error}", ConnectErrorKind::from_io(&error).code()))
}
