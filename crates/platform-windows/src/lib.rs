//! Win32 backend. Compile-checked on MSVC; real Windows acceptance is still required.
use openergy_platform::{Backend, PlatformError};
#[derive(Clone, Debug, Default)]
pub struct WindowsOptions {}
#[cfg(target_os = "windows")]
mod clipboard;
#[cfg(target_os = "windows")]
mod keymap;
#[cfg(target_os = "windows")]
mod native;
#[cfg(target_os = "windows")]
mod sink;
#[cfg(target_os = "windows")]
mod source;
/// Metadata only: never includes key identities or clipboard data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinNotice {
    Sleeping,
    Woke,
    SessionLocked,
    SessionUnlocked,
    NotificationSetupFailed,
    KeyboardLanguagesChanged,
}
pub fn create(options: WindowsOptions) -> Result<Backend, PlatformError> {
    create_with_notices(options).map(|(backend, _)| backend)
}
/// Additive Windows extension because frozen ScreenSystem has no sleep/lock
/// callback. A dedicated thread owns a message-only window receiving power
/// and session-change messages; drain notices on the daemon io thread.
pub fn create_with_notices(
    _options: WindowsOptions,
) -> Result<(Backend, std::sync::mpsc::Receiver<WinNotice>), PlatformError> {
    #[cfg(target_os = "windows")]
    {
        // Query coordinates in physical pixels. The executable should also declare PerMonitorV2.
        native::dpi_awareness();
        let clipboard = clipboard::Clipboard::new()?;
        let (screens, notices) = native::Screens::new_with_notices(clipboard.clone());
        let source = source::Source::with_language_notifier(screens.language_notifier.clone());
        Ok((
            Backend {
                name: "windows",
                source: Box::new(source),
                sink: Box::new(sink::Sink::default()),
                screens: Box::new(screens),
                clipboard: Box::new(clipboard),
            },
            notices,
        ))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(PlatformError::Failed("Win32 backend requires Windows".into()))
    }
}
