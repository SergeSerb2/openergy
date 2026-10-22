//! Original, standalone macOS platform modules for Openergy.
//! Side-effecting APIs are explicit; the verification binary never calls them.
#![cfg(target_os = "macos")]

pub mod displays;
mod ffi;
pub mod inject;
pub mod keymap;
pub mod pasteboard;
pub mod permissions;
pub mod tap;
pub mod types;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

pub(crate) fn error(message: impl Into<String>) -> Error {
    Error(message.into())
}
