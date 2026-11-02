//! Permission checks never prompt. Requests are explicit, separate entry points.
use crate::ffi;

#[derive(Debug, Clone, Copy)]
pub struct Permissions {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}

/// Trust as this process sees it. macOS can keep a denial cached in a running
/// process after the user grants access, so `preflight` also asks a fresh
/// child process while anything is denied.
pub fn preflight_local() -> Permissions {
    // Safety: public preflight APIs only read trust state and do not prompt.
    unsafe {
        Permissions {
            accessibility: ffi::AXIsProcessTrusted(),
            listen: ffi::CGPreflightListenEventAccess(),
            post: ffi::CGPreflightPostEventAccess(),
        }
    }
}

/// Hidden CLI argument for the fresh-process probe.
pub const PROBE_ARG: &str = "permissions-probe";

/// Entry point of the probe child: prints three 0/1 digits.
pub fn probe_main() -> i32 {
    let p = preflight_local();
    println!("{}{}{}", u8::from(p.accessibility), u8::from(p.listen), u8::from(p.post));
    0
}

pub fn preflight() -> Permissions {
    let local = preflight_local();
    if local.accessibility && local.listen && local.post {
        return local;
    }
    match fresh() {
        Some(fresh) => Permissions {
            accessibility: local.accessibility || fresh.accessibility,
            listen: local.listen || fresh.listen,
            post: local.post || fresh.post,
        },
        None => local,
    }
}

/// The child inherits this process's TCC responsibility (the app that
/// started the daemon), so its answer is the current grant for us.
fn fresh() -> Option<Permissions> {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static CACHE: Mutex<Option<(Instant, Option<Permissions>)>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, value)) = *cache {
        if at.elapsed() < Duration::from_secs(1) {
            return value;
        }
    }
    let value = (|| {
        let exe = std::env::current_exe().ok()?;
        let mut child = std::process::Command::new(exe)
            .arg(PROBE_ARG)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
            }
        }
        let mut out = String::new();
        std::io::Read::read_to_string(&mut child.stdout.take()?, &mut out).ok()?;
        let digits = out.trim().as_bytes();
        (digits.len() == 3 && digits.iter().all(|d| matches!(d, b'0' | b'1'))).then(|| Permissions {
            accessibility: digits[0] == b'1',
            listen: digits[1] == b'1',
            post: digits[2] == b'1',
        })
    })();
    *cache = Some((Instant::now(), value));
    value
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
