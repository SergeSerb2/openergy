//! Permission checks never prompt. Requests are explicit, separate entry points.
use crate::ffi;

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}

pub fn preflight() -> Permissions {
    // Safety: public preflight APIs only read trust state and do not prompt.
    unsafe {
        Permissions {
            accessibility: ffi::AXIsProcessTrusted(),
            listen: ffi::CGPreflightListenEventAccess(),
            post: ffi::CGPreflightPostEventAccess(),
        }
    }
}

pub fn platform() -> openergy_platform::Permissions {
    use openergy_platform::Grant;
    let state = preflight();
    let grant = |on| if on { Grant::Granted } else { Grant::Denied };
    openergy_platform::Permissions {
        accessibility: grant(state.accessibility),
        input_monitoring: grant(state.listen),
        post_events: grant(state.post),
    }
}

/// Explicit privacy prompt. Compile-only on the development workstation.
pub fn request(kind: openergy_platform::PermissionKind) -> crate::Result<bool> {
    use openergy_platform::PermissionKind;
    // Safety: public TCC request APIs; caller explicitly requested this permission.
    unsafe {
        Ok(match kind {
            PermissionKind::Accessibility => {
                let key = ffi::kAXTrustedCheckOptionPrompt;
                let value = ffi::kCFBooleanTrue;
                // Static CF singleton keys/values outlive this temporary dictionary.
                let options = ffi::OwnedCf::new(
                    ffi::CFDictionaryCreate(ffi::null(), &key, &value, 1, ffi::null(), ffi::null()),
                    "AX prompt options",
                )?;
                ffi::AXIsProcessTrustedWithOptions(options.0)
            }
            PermissionKind::InputMonitoring => ffi::CGRequestListenEventAccess(),
            PermissionKind::PostEvents => ffi::CGRequestPostEventAccess(),
        })
    }
}
