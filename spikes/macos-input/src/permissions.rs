//! Permission checks never prompt. Requests are explicit, separate entry points.
use crate::Result;
use crate::ffi::{self, OwnedCf};

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}

pub fn preflight() -> Permissions {
    unsafe {
        Permissions {
            accessibility: ffi::AXIsProcessTrusted(),
            listen: ffi::CGPreflightListenEventAccess(),
            post: ffi::CGPreflightPostEventAccess(),
        }
    }
}

/// May show a macOS privacy prompt. Never called by the spike CLI.
pub fn request_accessibility() -> Result<bool> {
    unsafe {
        let key = ffi::kAXTrustedCheckOptionPrompt;
        let value = ffi::kCFBooleanTrue;
        // Static singleton key/value references outlive this dictionary, so
        // NULL retain/release callbacks are intentional and valid.
        let options = OwnedCf::new(
            ffi::CFDictionaryCreate(ffi::null(), &key, &value, 1, ffi::null(), ffi::null()),
            "AX options dictionary",
        )?;
        Ok(ffi::AXIsProcessTrustedWithOptions(options.0))
    }
}

/// May show a privacy prompt. Never called by the spike CLI.
pub fn request_listen() -> bool {
    unsafe { ffi::CGRequestListenEventAccess() }
}
/// May show a privacy prompt. Never called by the spike CLI.
pub fn request_post() -> bool {
    unsafe { ffi::CGRequestPostEventAccess() }
}
