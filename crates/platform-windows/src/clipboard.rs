//! Real Win32 clipboard, CF_HTML byte offsets, UTF-16 text and opaque wire DIBs.
use crate::native::{self, error, MessageThread};
use openergy_platform::{ClipboardBackend, ClipboardData, ClipboardFormat, FormatKind, PlatformError};
use std::{
    cell::RefCell,
    ptr::null_mut,
    sync::{
        atomic::{AtomicU64, Ordering},
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
    stamp: Arc<AtomicU64>,
}
enum Command {
    Read(usize, SyncSender<Result<ClipboardData, PlatformError>>),
    Write(ClipboardData, SyncSender<Result<(), PlatformError>>),
}
thread_local! { static STAMP: RefCell<Option<Arc<AtomicU64>>> = const { RefCell::new(None) }; }
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        STAMP.with(|s| {
            if let Some(s) = s.borrow().as_ref() {
                s.fetch_add(1, Ordering::AcqRel);
            }
        });
    }
    // Safety: unknown messages are forwarded unchanged to DefWindowProcW.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}
impl Clipboard {
    pub fn new() -> Result<Self, PlatformError> {
        let (ready, rx) = mpsc::sync_channel(1);
        let (tx, commands) = mpsc::sync_channel(16);
        let stamp = Arc::new(AtomicU64::new(0));
        let shared = stamp.clone();
        let h = std::thread::Builder::new()
            .name("oe-win-clipboard".into())
            .spawn(move || {
                STAMP.with(|s| *s.borrow_mut() = Some(shared));
                let result = (|| -> Result<(), PlatformError> {
                    let hwnd = native::window("OpenergyClipboard", Some(proc), true)?;
                    // Safety: hwnd belongs to this message thread and the registered format name is terminated.
                    unsafe {
                        if AddClipboardFormatListener(hwnd) == 0 {
                            DestroyWindow(hwnd);
                            return Err(error("AddClipboardFormatListener"));
                        }
                        let html = RegisterClipboardFormatW(native::wide("HTML Format").as_ptr());
                        if html == 0 {
                            RemoveClipboardFormatListener(hwnd);
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
                        RemoveClipboardFormatListener(hwnd);
                        DestroyWindow(hwnd);
                    }
                    Ok(())
                })();
                if let Err(e) = result {
                    let _ = ready.send(Err(e));
                }
                STAMP.with(|s| *s.borrow_mut() = None);
            })
            .map_err(error)?;
        match rx.recv().map_err(error)? {
            Ok(id) => Ok(Self {
                inner: Arc::new(Inner {
                    tx,
                    thread: MessageThread { id, handle: Some(h) },
                    stamp,
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
        self.inner.stamp.load(Ordering::Acquire)
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
        if IsClipboardFormatAvailable(format) == 0 {
            return Ok(None);
        }
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return Err(error("GetClipboardData"));
        }
        let size = GlobalSize(handle);
        if size > max || size > LIMIT {
            return Err(error("Native clipboard exceeds caller limit"));
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
    let mut data = ClipboardData::default();
    if let Some(bytes) = get(CF_UNICODETEXT, max.saturating_mul(2).saturating_add(2))? {
        if bytes.len() % 2 != 0 {
            return Err(error("CF_UNICODETEXT has odd byte length"));
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .take_while(|v| *v != 0)
            .collect();
        let text = String::from_utf16(&units)
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
        return Err(error("Clipboard exceeds caller limit"));
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
    let head = bytes.get(..4).ok_or_else(|| error("Truncated DIB"))?;
    let size = u32::from_le_bytes(head.try_into().map_err(error)?) as usize;
    if ![12, 40, 52, 56, 108, 124].contains(&size) || size > bytes.len() {
        return Err(error("Invalid DIB header"));
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
