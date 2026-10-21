//! Native macOS backend. Construction and the `mac_probe` binary never start capture.
//! Side effects occur only through explicit trait methods; see TEST-MAC.md.
#![cfg(target_os = "macos")]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod contract_shim;
pub mod displays;
mod ffi;
pub mod inject;
pub mod keymap;
pub mod pasteboard;
pub mod permissions;
pub mod screens;
pub mod tap;
pub mod types;

use openergy_platform::{ClipboardBackend, ClipboardData, ClipboardFormat, FormatKind, PlatformError};
use std::sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc, Mutex,
};

pub use contract_shim::Backend;
pub use inject::ClickNumbering;

pub(crate) type EnterPosition = Arc<Mutex<Option<(i32, i32)>>>;

pub type Result<T> = std::result::Result<T, PlatformError>;
pub(crate) fn error(message: impl Into<String>) -> PlatformError {
    PlatformError::Failed(message.into())
}

#[derive(Debug, Clone, Copy)]
pub struct MacOptions {
    pub click_numbering: ClickNumbering,
    pub lines_per_notch: f64,
}
impl Default for MacOptions {
    fn default() -> Self {
        Self {
            click_numbering: ClickNumbering::Native,
            lines_per_notch: 3.0,
        }
    }
}

/// Metadata only: never includes key identities or clipboard data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacNotice {
    CapturePermissionProblem,
    CaptureTranslationProblem,
    CaptureStopped,
    DisplaysChanged,
    Sleeping,
    Woke,
    ScreenLocked,
    ScreenUnlocked,
    NotificationSetupFailed,
}

#[derive(Clone)]
pub(crate) struct Notices(SyncSender<MacNotice>);
impl Notices {
    pub(crate) fn send(&self, notice: MacNotice) {
        // Bounded, nonblocking delivery. A full queue fails safe: daemon must
        // treat any lifecycle notice as a reason to resample state and leave.
        if let Err(mpsc::TrySendError::Full(_)) = self.0.try_send(notice) {
            log::error!("macOS lifecycle notice queue full; resample platform state");
        }
    }
}

/// Creates four independently movable parts. No tap, cursor operation, pasteboard
/// payload access, permission prompt, observer, or power assertion starts here.
pub fn create(options: MacOptions) -> Result<Backend> {
    create_with_notices(options).map(|(backend, _)| backend)
}

/// Additive macOS extension because frozen ScreenSystem has no sleep/lock/error
/// callback. Call screens.on_display_changed to start the notification run loop,
/// then drain notices on the daemon io thread. Capture errors use the same queue.
pub fn create_with_notices(options: MacOptions) -> Result<(Backend, Receiver<MacNotice>)> {
    if !options.lines_per_notch.is_finite() || options.lines_per_notch <= 0.0 || options.lines_per_notch > 1000.0 {
        return Err(error("lines_per_notch must be in (0, 1000]"));
    }
    let (sender, receiver) = mpsc::sync_channel(64);
    let notices = Notices(sender);
    let enter_position = EnterPosition::default();
    let mut sink = inject::Injector::new(options.click_numbering)?;
    sink.set_enter_position(enter_position.clone());
    sink.set_lines_per_notch(options.lines_per_notch)?;
    let clipboard = MacClipboard::default();
    let backend = Backend {
        name: "macos",
        source: Box::new(tap::MacSource::new(
            options.lines_per_notch,
            notices.clone(),
            enter_position,
        )),
        sink: Box::new(sink),
        screens: Box::new(screens::MacScreens::new(notices, clipboard.clone())),
        clipboard: Box::new(clipboard),
    };
    Ok((backend, receiver))
}

/// Each thread obtains its own NSPasteboard object. Only loop-prevention
/// metadata is shared between the worker backend and legacy screens.clipboard.
#[derive(Clone, Default)]
pub struct MacClipboard {
    stamp: Arc<Mutex<ClipboardStamp>>,
}
#[derive(Default)]
struct ClipboardStamp {
    own_write: Option<u64>,
    visible: u64,
}
impl MacClipboard {
    pub fn own_write_stamp(&self) -> Option<u64> {
        self.stamp.lock().ok().and_then(|s| s.own_write)
    }
    pub fn is_own_write(&self) -> bool {
        self.own_write_stamp() == Some(pasteboard::change_count() as u64)
    }
}
impl ClipboardBackend for MacClipboard {
    fn change_stamp(&self) -> u64 {
        let current = pasteboard::change_count() as u64;
        match self.stamp.lock() {
            Ok(mut stamp) => {
                // A successful remote write does not look like a new local
                // clipboard generation, so the worker does not announce it back.
                if stamp.own_write != Some(current) {
                    stamp.visible = current;
                }
                stamp.visible
            }
            Err(_) => current,
        }
    }
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData> {
        if self.is_own_write() {
            return Ok(ClipboardData::default());
        }
        let clipboard = pasteboard::read(max_bytes)?;
        let mut formats = Vec::with_capacity(3);
        if let Some(text) = clipboard.text {
            formats.push(ClipboardFormat {
                kind: FormatKind::TextUtf8,
                bytes: text.into_bytes(),
            });
        }
        if let Some(html) = clipboard.html {
            formats.push(ClipboardFormat {
                kind: FormatKind::HtmlUtf8,
                bytes: html.into_bytes(),
            });
        }
        if let Some(bytes) = clipboard.dib {
            formats.push(ClipboardFormat {
                kind: FormatKind::Dib,
                bytes,
            });
        }
        Ok(ClipboardData { formats })
    }
    fn write(&mut self, data: &ClipboardData) -> Result<()> {
        const CAP: usize = 64 * 1024 * 1024;
        let mut clipboard = pasteboard::Clipboard::default();
        let mut total = 0usize;
        for format in &data.formats {
            total = total
                .checked_add(format.bytes.len())
                .filter(|n| *n <= CAP)
                .ok_or_else(|| error("clipboard write exceeds 64 MiB"))?;
            match format.kind {
                FormatKind::TextUtf8 | FormatKind::HtmlUtf8 => {
                    let text =
                        String::from_utf8(format.bytes.clone()).map_err(|_| error("clipboard text must be UTF-8"))?;
                    let slot = if format.kind == FormatKind::TextUtf8 {
                        &mut clipboard.text
                    } else {
                        &mut clipboard.html
                    };
                    if slot.replace(text).is_some() {
                        return Err(error("duplicate clipboard text format"));
                    }
                }
                FormatKind::Dib => {
                    if clipboard.dib.replace(format.bytes.clone()).is_some() {
                        return Err(error("duplicate clipboard image format"));
                    }
                }
                FormatKind::Png => {
                    if clipboard
                        .dib
                        .replace(pasteboard::image_to_dib(&format.bytes, CAP)?)
                        .is_some()
                    {
                        return Err(error("duplicate clipboard image format"));
                    }
                }
            }
        }
        // Hold the metadata lock across replacement so another backend part
        // cannot observe the own-write generation as a local change.
        let mut stamp = self.stamp.lock().map_err(|_| error("clipboard stamp poisoned"))?;
        let before = pasteboard::change_count() as u64;
        if stamp.own_write != Some(before) {
            stamp.visible = before;
        }
        let result = pasteboard::write(&clipboard, CAP);
        let after = pasteboard::change_count() as u64;
        // Also suppress a partial local replacement if the final write fails.
        if result.is_ok() || before != after {
            stamp.own_write = Some(after);
        }
        result
    }
}
