//! Native X11 backend. No connection is opened on non-Linux targets.
use openergy_platform::{Backend, PlatformError};

#[derive(Clone, Debug, Default)]
pub struct X11Options {
    pub display: Option<String>,
}

#[cfg(target_os = "linux")]
mod clipboard;
#[cfg(target_os = "linux")]
mod image;
#[cfg(target_os = "linux")]
mod keymap;
#[cfg(target_os = "linux")]
mod native;
#[cfg(target_os = "linux")]
mod sink;
#[cfg(target_os = "linux")]
mod source;
#[cfg(target_os = "linux")]
mod unicode_keysyms;
#[cfg(target_os = "linux")]
pub use clipboard::{Selection, X11Clipboard};

pub fn create(options: X11Options) -> Result<Backend, PlatformError> {
    #[cfg(target_os = "linux")]
    {
        let display = options.display.as_deref();
        let clipboard = X11Clipboard::connect(display, Selection::Clipboard)?;
        Ok(Backend {
            name: "x11",
            source: Box::new(source::Source::new(display)?),
            sink: Box::new(sink::Sink::new(display)?),
            screens: Box::new(native::Screens::new(display, clipboard.clone())?),
            clipboard: Box::new(clipboard),
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = options;
        Err(PlatformError::Failed("X11 backend requires Linux".into()))
    }
}
