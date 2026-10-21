//! Win32 backend. Compile-checked on MSVC; real Windows acceptance is still required.
use openergy_platform::PlatformError;
/// Temporary copy of integration-contract §2 until WP3 lands the shared type.
/// Replace `use contract_shim::Backend` with the shared import at integration.
pub mod contract_shim {
    use openergy_platform::{ClipboardBackend, InputSink, InputSource, ScreenSystem};
    pub struct Backend {
        pub name: &'static str,
        pub source: Box<dyn InputSource>,
        pub sink: Box<dyn InputSink>,
        pub screens: Box<dyn ScreenSystem>,
        pub clipboard: Box<dyn ClipboardBackend>,
    }
}
use contract_shim::Backend;
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
