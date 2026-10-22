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
pub fn create(_options: WindowsOptions) -> Result<Backend, PlatformError> {
    #[cfg(target_os = "windows")]
    {
        // Query coordinates in physical pixels. The executable should also declare PerMonitorV2.
        native::dpi_awareness();
        let clipboard = clipboard::Clipboard::new()?;
        Ok(Backend {
            name: "windows",
            source: Box::new(source::Source::default()),
            sink: Box::new(sink::Sink::default()),
            screens: Box::new(native::Screens::new(clipboard.clone())),
            clipboard: Box::new(clipboard),
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(PlatformError::Failed("Win32 backend requires Windows".into()))
    }
}
