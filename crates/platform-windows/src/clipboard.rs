//! Real Win32 clipboard, CF_HTML byte offsets, UTF-16 text and opaque wire DIBs.
use crate::native::{self, error, MessageThread};
use openergy_platform::{ClipboardBackend, ClipboardData, ClipboardFormat, FormatKind, PlatformError};
use std::{
    ptr::null_mut,
    sync::{
        mpsc::{self, SyncSender},
        Arc,
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{DataExchange::*, Memory::*, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::*,
};
const CF_UNICODETEXT: u32 = 13;
const CF_DIB: u32 = 8;
const CF_DIBV5: u32 = 17;
const COMMAND: u32 = WM_APP + 9;
const LIMIT: usize = 256 * 1024 * 1024;
#[derive(Clone)]
pub struct Clipboard {
    inner: Arc<Inner>,
}
struct Inner {
    tx: SyncSender<Command>,
    thread: MessageThread,
}
enum Command {
    Read(usize, SyncSender<Result<ClipboardData, PlatformError>>),
    Write(ClipboardData, SyncSender<Result<(), PlatformError>>),
}
impl Clipboard {
    pub fn new() -> Result<Self, PlatformError> {
        let (ready, rx) = mpsc::sync_channel(1);
        let (tx, commands) = mpsc::sync_channel(16);
        let h = std::thread::Builder::new()
            .name("oe-win-clipboard".into())
            .spawn(move || {
                let result = (|| -> Result<(), PlatformError> {
                    let hwnd = native::window("OpenergyClipboard", Some(DefWindowProcW), true)?;
                    // Safety: hwnd belongs to this message thread and the registered format name is terminated.
                    unsafe {
                        let html = RegisterClipboardFormatW(native::wide("HTML Format").as_ptr());
                        if html == 0 {
                            DestroyWindow(hwnd);
                            return Err(error("Register HTML Format"));
                        }
                        let mut msg = MSG::default();
                        PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
                        let _ = ready.send(Ok(GetCurrentThreadId()));
                        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                            if msg.message == COMMAND {
                                while let Ok(command) = commands.try_recv() {
                                    match command {
                                        Command::Read(max, reply) => {
                                            let _ = reply.send(read(hwnd, html, max.min(LIMIT)));
                                        }
                                        Command::Write(data, reply) => {
                                            let _ = reply.send(write(hwnd, html, data));
                                        }
                                    }
                                }
                            } else {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }
                        DestroyWindow(hwnd);
                    }
                    Ok(())
                })();
                if let Err(e) = result {
                    let _ = ready.send(Err(e));
                }
            })
            .map_err(error)?;
        match rx.recv().map_err(error)? {
            Ok(id) => Ok(Self {
                inner: Arc::new(Inner {
                    tx,
                    thread: MessageThread { id, handle: Some(h) },
                }),
            }),
            Err(e) => {
                let _ = h.join();
                Err(e)
            }
        }
    }
    fn command<T>(
        &self,
        build: impl FnOnce(SyncSender<Result<T, PlatformError>>) -> Command,
    ) -> Result<T, PlatformError> {
        let (reply, rx) = mpsc::sync_channel(1);
        self.inner.tx.send(build(reply)).map_err(error)?;
        // Safety: the owned clipboard thread initialized its queue before publishing its id.
        if unsafe { PostThreadMessageW(self.inner.thread.id, COMMAND, 0, 0) } == 0 {
            return Err(error("Wake clipboard thread"));
        }
        rx.recv().map_err(error)?
    }
}
impl ClipboardBackend for Clipboard {
    fn change_stamp(&self) -> u64 {
        // Read the kernel sequence synchronously. WM_CLIPBOARDUPDATE is queued
        // after write() returns and can otherwise echo our own write as a grab.
        // Safety: this query has no pointer arguments and does not open the clipboard.
        u64::from(unsafe { GetClipboardSequenceNumber() })
    }
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        self.command(|r| Command::Read(max_bytes, r))
    }
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError> {
        if data.total_bytes() > LIMIT {
            return Err(error("Clipboard exceeds 256 MiB limit"));
        }
        self.command(|r| Command::Write(data.clone(), r))
    }
}
struct Open;
impl Open {
    fn acquire(hwnd: HWND) -> Result<Self, PlatformError> {
        for _ in 0..10 {
            // Safety: hwnd is owned by this clipboard worker and remains live during the transaction.
            if unsafe { OpenClipboard(hwnd) } != 0 {
                return Ok(Self);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err(error("Clipboard is busy"))
    }
}
impl Drop for Open {
    fn drop(&mut self) {
        // Safety: this guard exists only after a successful OpenClipboard on the same thread.
        unsafe {
            CloseClipboard();
        }
    }
}
fn get(format: u32, max: usize) -> Result<Option<Vec<u8>>, PlatformError> {
    // Safety: caller holds OpenClipboard; GlobalLock gives size bytes until GlobalUnlock.
    unsafe {
        SetLastError(ERROR_SUCCESS);
        if IsClipboardFormatAvailable(format) == 0 {
            let code = GetLastError();
            if code != ERROR_SUCCESS {
                return Err(error(std::io::Error::from_raw_os_error(code as i32)));
            }
            return Ok(None);
        }
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return Err(error("GetClipboardData"));
        }
        let size = GlobalSize(handle);
        if size > max || size > LIMIT {
            return Err(PlatformError::ClipboardTooLarge {
                bytes: size,
                limit: max.min(LIMIT),
            });
        }
        if size == 0 {
            return Ok(Some(Vec::new()));
        }
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            return Err(error("GlobalLock clipboard"));
        }
        let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
        GlobalUnlock(handle);
        Ok(Some(bytes))
    }
}
fn read(hwnd: HWND, html: u32, max: usize) -> Result<ClipboardData, PlatformError> {
    let _open = Open::acquire(hwnd)?;
    // OpenClipboard excludes other normal writers. Also check the sequence:
    // delayed rendering can change clipboard data during GetClipboardData.
    // Safety: this query does not open or mutate the clipboard.
    let sequence = unsafe { GetClipboardSequenceNumber() };
    let mut data = ClipboardData::default();
    if let Some(bytes) = get(CF_UNICODETEXT, max.saturating_mul(2).saturating_add(2))? {
        if bytes.len() % 2 != 0 {
            return Err(error("CF_UNICODETEXT has odd byte length"));
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect();
        let end = units
            .iter()
            .position(|v| *v == 0)
            .ok_or_else(|| error("CF_UNICODETEXT lacks NUL terminator"))?;
        let text = String::from_utf16(&units[..end])
            .map_err(error)?
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        data.formats.push(ClipboardFormat {
            kind: FormatKind::TextUtf8,
            bytes: text.into_bytes(),
        });
    }
    if let Some(bytes) = get(html, max.saturating_sub(data.total_bytes()).saturating_add(4096))? {
        data.formats.push(ClipboardFormat {
            kind: FormatKind::HtmlUtf8,
            bytes: decode_html(&bytes)?,
        });
    }
    let left = max.saturating_sub(data.total_bytes());
    let dib = if let Some(bytes) = get(CF_DIBV5, left)? {
        Some(bytes)
    } else {
        get(CF_DIB, left)?
    };
    if let Some(bytes) = dib {
        validate_dib(&bytes)?;
        data.formats.push(ClipboardFormat {
            kind: FormatKind::Dib,
            bytes,
        });
    }
    if data.total_bytes() > max {
        return Err(PlatformError::ClipboardTooLarge {
            bytes: data.total_bytes(),
            limit: max,
        });
    }
    let empty = if data.formats.is_empty() {
        // Unsupported-only contents must not become a remote clear. Zero can
        // also indicate an API failure, so do not mistake a stale/error code
        // for a successful empty snapshot.
        // Safety: the Open guard still holds the clipboard on this thread.
        unsafe {
            SetLastError(ERROR_SUCCESS);
            let count = CountClipboardFormats();
            if count == 0 {
                let code = GetLastError();
                if code != ERROR_SUCCESS {
                    return Err(error(std::io::Error::from_raw_os_error(code as i32)));
                }
            }
            count == 0
        }
    } else {
        false
    };
    // Safety: the guard is still live; reject a changed generation as a failed
    // read even when the final count is zero, rather than forwarding a clear.
    if unsafe { GetClipboardSequenceNumber() } != sequence {
        return Err(error("Clipboard changed during read"));
    }
    if data.formats.is_empty() && !empty {
        return Err(PlatformError::ClipboardUnavailable);
    }
    Ok(data)
}
struct Memory(HGLOBAL);
impl Memory {
    fn new(bytes: &[u8]) -> Result<Self, PlatformError> {
        // Safety: GlobalAlloc creates moveable memory; copy uses exactly the allocated byte count.
        unsafe {
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
            if h.is_null() {
                return Err(error("GlobalAlloc clipboard"));
            }
            let m = Self(h);
            let p = GlobalLock(h);
            if p.is_null() {
                return Err(error("GlobalLock allocated clipboard"));
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, bytes.len());
            GlobalUnlock(h);
            Ok(m)
        }
    }
}
impl Drop for Memory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // Safety: memory is freed only while it is still owned by this guard.
            unsafe {
                GlobalFree(self.0);
            }
        }
    }
}
fn write(hwnd: HWND, html: u32, data: ClipboardData) -> Result<(), PlatformError> {
    let mut values = Vec::new();
    for format in data.formats {
        let (id, bytes) = match format.kind {
            FormatKind::TextUtf8 => {
                let text = std::str::from_utf8(&format.bytes).map_err(error)?;
                if text.contains('\0') {
                    return Err(error("Win32 text cannot contain NUL"));
                }
                let text = text.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\r\n");
                (
                    CF_UNICODETEXT,
                    text.encode_utf16()
                        .chain(Some(0))
                        .flat_map(u16::to_le_bytes)
                        .collect::<Vec<_>>(),
                )
            }
            FormatKind::HtmlUtf8 => (html, encode_html(&format.bytes)?),
            FormatKind::Dib => {
                validate_dib(&format.bytes)?;
                (
                    if u32::from_le_bytes(format.bytes[0..4].try_into().map_err(error)?) >= 124 {
                        CF_DIBV5
                    } else {
                        CF_DIB
                    },
                    format.bytes,
                )
            }
            FormatKind::Png => return Err(error("Windows clipboard expects wire DIB, not PNG")),
        };
        values.push((id, Memory::new(&bytes)?));
    }
    let _open = Open::acquire(hwnd)?;
    // Safety: clipboard is open on this thread; ownership transfers only on SetClipboardData success.
    unsafe {
        if EmptyClipboard() == 0 {
            return Err(error("EmptyClipboard"));
        }
        for (id, mut memory) in values {
            if SetClipboardData(id, memory.0).is_null() {
                return Err(error("SetClipboardData"));
            }
            memory.0 = null_mut();
        }
    }
    Ok(())
}
fn validate_dib(bytes: &[u8]) -> Result<(), PlatformError> {
    let word = |at| -> Result<u16, PlatformError> {
        Ok(u16::from_le_bytes(
            bytes
                .get(at..at + 2)
                .ok_or_else(|| error("Truncated DIB"))?
                .try_into()
                .map_err(error)?,
        ))
    };
    let dword = |at| -> Result<u32, PlatformError> {
        Ok(u32::from_le_bytes(
            bytes
                .get(at..at + 4)
                .ok_or_else(|| error("Truncated DIB"))?
                .try_into()
                .map_err(error)?,
        ))
    };
    let size = dword(0)? as usize;
    if ![12, 40, 52, 56, 108, 124].contains(&size) || size > bytes.len() {
        return Err(error("Invalid DIB header"));
    }
    let (width, height, planes, bpp, compression, declared, colors) = if size == 12 {
        (i32::from(word(4)?), i32::from(word(6)?), word(8)?, word(10)?, 0, 0, 0)
    } else {
        (
            dword(4)? as i32,
            dword(8)? as i32,
            word(12)?,
            word(14)?,
            dword(16)?,
            dword(20)? as usize,
            dword(32)? as usize,
        )
    };
    if width <= 0 || height == 0 || height == i32::MIN || planes != 1 {
        return Err(error("Invalid DIB dimensions or planes"));
    }
    let valid = match compression {
        0 => [1, 4, 8, 16, 24, 32].contains(&bpp),
        1 => bpp == 8 && height > 0,
        2 => bpp == 4 && height > 0,
        3 | 6 => [16, 32].contains(&bpp),
        4 | 5 => bpp == 0 && height > 0,
        _ => false,
    };
    if !valid || (size == 12 && ![1, 4, 8, 24].contains(&bpp)) {
        return Err(error("Invalid DIB depth or compression"));
    }
    let palette = if bpp != 0 && bpp <= 8 {
        let maximum = 1usize << bpp;
        if colors > maximum {
            return Err(error("Invalid DIB palette size"));
        }
        if colors == 0 {
            maximum
        } else {
            colors
        }
    } else {
        colors
    };
    let masks = if size == 40 {
        match compression {
            3 => 12,
            6 => 16,
            _ => 0,
        }
    } else {
        0
    };
    let offset = size
        .checked_add(masks)
        .and_then(|n| {
            palette
                .checked_mul(if size == 12 { 3 } else { 4 })
                .and_then(|p| n.checked_add(p))
        })
        .ok_or_else(|| error("DIB offset overflow"))?;
    if compression == 3 || compression == 6 {
        let count = if compression == 6 { 4 } else { 3 };
        if size != 40 && size < 40 + count * 4 {
            return Err(error("Truncated DIB masks"));
        }
        let mut used = 0;
        for at in (40..40 + count * 4).step_by(4) {
            let mask = dword(at)?;
            let shifted = mask.checked_shr(mask.trailing_zeros()).unwrap_or(0);
            if mask == 0 || used & mask != 0 || shifted & shifted.wrapping_add(1) != 0 || (bpp == 16 && mask > 0xffff) {
                return Err(error("Invalid DIB masks"));
            }
            used |= mask;
        }
    }
    let length = if [0, 3, 6].contains(&compression) {
        let stride = (width as usize)
            .checked_mul(usize::from(bpp))
            .and_then(|n| n.checked_add(31))
            .map(|n| (n / 32) * 4)
            .ok_or_else(|| error("DIB row overflow"))?;
        let required = stride
            .checked_mul(height.unsigned_abs() as usize)
            .ok_or_else(|| error("DIB size overflow"))?;
        if declared != 0 && declared < required {
            return Err(error("DIB image size is too small"));
        }
        required.max(declared)
    } else {
        if declared == 0 {
            return Err(error("Compressed DIB lacks image size"));
        }
        declared
    };
    if offset.checked_add(length).is_none_or(|end| end > bytes.len()) {
        return Err(error("Truncated DIB palette or pixels"));
    }
    if size == 124 {
        let profile = dword(112)? as usize;
        let length = dword(116)? as usize;
        if length != 0 && (profile < size || profile.checked_add(length).is_none_or(|end| end > bytes.len())) {
            return Err(error("DIB profile is out of bounds"));
        }
    }
    Ok(())
}
fn encode_html(fragment: &[u8]) -> Result<Vec<u8>, PlatformError> {
    std::str::from_utf8(fragment).map_err(error)?;
    if fragment.contains(&0) {
        return Err(error("CF_HTML cannot contain NUL"));
    }
    let prefix = b"<html><body><!--StartFragment-->";
    let suffix = b"<!--EndFragment--></body></html>";
    let header = |start, end, from, to| {
        format!("Version:1.0\r\nStartHTML:{start:010}\r\nEndHTML:{end:010}\r\nStartFragment:{from:010}\r\nEndFragment:{to:010}\r\n")
    };
    let start = header(0, 0, 0, 0).len();
    let from = start + prefix.len();
    let to = from + fragment.len();
    let end = to + suffix.len();
    let mut bytes = header(start, end, from, to).into_bytes();
    bytes.extend(prefix);
    bytes.extend(fragment);
    bytes.extend(suffix);
    bytes.push(0);
    Ok(bytes)
}
fn decode_html(bytes: &[u8]) -> Result<Vec<u8>, PlatformError> {
    let bytes = &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())];
    let header = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    let offset = |field: &str| -> Option<usize> {
        header
            .lines()
            .find_map(|line| line.strip_prefix(field).and_then(|n| n.trim().parse().ok()))
    };
    let range = offset("StartFragment:")
        .zip(offset("EndFragment:"))
        .or_else(|| offset("StartHTML:").zip(offset("EndHTML:")))
        .ok_or_else(|| error("CF_HTML lacks valid offsets"))?;
    let value = bytes
        .get(range.0..range.1)
        .ok_or_else(|| error("CF_HTML offsets out of bounds"))?;
    std::str::from_utf8(value).map_err(error)?;
    Ok(value.to_vec())
}
