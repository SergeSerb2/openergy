use crate::clipboard::Clipboard;
use openergy_platform::{ClipboardBackend, Permissions, PlatformError, ScreenRect, ScreenSystem};
use std::{
    ptr::{null, null_mut},
    sync::mpsc,
    thread::JoinHandle,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{LibraryLoader::GetModuleHandleW, Power::*},
    UI::{HiDpi::*, WindowsAndMessaging::*},
};
pub fn error(e: impl std::fmt::Display) -> PlatformError {
    PlatformError::Failed(e.to_string())
}
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
pub fn dpi_awareness() {
    // Safety: this changes only the calling thread's coordinate interpretation.
    unsafe {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}
pub fn keyboard_layout() -> windows_sys::Win32::UI::Input::KeyboardAndMouse::HKL {
    // Keyboard layouts belong to the focused application's thread, not to
    // our capture/injection worker (which has no focused window).
    // Safety: the foreground HWND is borrowed and only queried for its thread id.
    unsafe {
        let hwnd = GetForegroundWindow();
        let thread = if hwnd.is_null() {
            0
        } else {
            GetWindowThreadProcessId(hwnd, null_mut())
        };
        windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout(thread)
    }
}
pub fn window(class: &str, proc: WNDPROC, message_only: bool) -> Result<HWND, PlatformError> {
    let class = wide(class);
    // Safety: class is NUL-terminated; Win32 copies its name and stores a static callback.
    unsafe {
        let instance = GetModuleHandleW(null());
        let wc = WNDCLASSW {
            lpfnWndProc: proc,
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..Default::default()
        };
        RegisterClassW(&wc); // ERROR_CLASS_ALREADY_EXISTS is harmless for repeated creation.
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            if message_only { HWND_MESSAGE } else { null_mut() },
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            Err(error("CreateWindowExW"))
        } else {
            Ok(hwnd)
        }
    }
}
pub struct MessageThread {
    pub id: u32,
    pub handle: Option<JoinHandle<()>>,
}
impl Drop for MessageThread {
    fn drop(&mut self) {
        // Safety: id names a live thread whose queue was initialized before construction.
        unsafe {
            PostThreadMessageW(self.id, WM_QUIT, 0, 0);
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
pub struct Screens {
    clipboard: Clipboard,
    monitor: Option<MessageThread>,
}
impl Screens {
    pub fn new(clipboard: Clipboard) -> Self {
        Self {
            clipboard,
            monitor: None,
        }
    }
}
unsafe extern "system" fn enumerate(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> windows_sys::core::BOOL {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // Safety: EnumDisplayMonitors supplies a valid monitor and the callback data points to our Vec.
    unsafe {
        if GetMonitorInfoW(monitor, &mut info) != 0 {
            let rect = info.rcMonitor;
            let values = &mut *(data as *mut Vec<ScreenRect>);
            values.push(ScreenRect {
                x: rect.left,
                y: rect.top,
                w: rect.right - rect.left,
                h: rect.bottom - rect.top,
                scale: {
                    let (mut x, mut y) = (96, 96);
                    if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y) >= 0 {
                        f64::from(x) / 96.0
                    } else {
                        1.0
                    }
                },
            });
        }
    }
    1
}
thread_local! { static DISPLAY_CALLBACK: std::cell::RefCell<Option<Box<dyn FnMut() + Send>>> = std::cell::RefCell::new(None); }
unsafe extern "system" fn display_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_DISPLAYCHANGE || msg == WM_DPICHANGED {
        DISPLAY_CALLBACK.with(|c| {
            if let Some(cb) = c.borrow_mut().as_mut() {
                cb();
            }
        });
    }
    // Safety: all unhandled messages are forwarded unchanged to the system window procedure.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}
impl ScreenSystem for Screens {
    fn screens(&self) -> Vec<ScreenRect> {
        dpi_awareness();
        let mut values = Vec::new();
        // Safety: enumerate receives a mutable Vec valid until this synchronous call returns.
        unsafe {
            EnumDisplayMonitors(null_mut(), null(), Some(enumerate), &mut values as *mut _ as isize);
            if values.is_empty() {
                values.push(ScreenRect {
                    x: GetSystemMetrics(SM_XVIRTUALSCREEN),
                    y: GetSystemMetrics(SM_YVIRTUALSCREEN),
                    w: GetSystemMetrics(SM_CXVIRTUALSCREEN),
                    h: GetSystemMetrics(SM_CYVIRTUALSCREEN),
                    scale: 1.0,
                });
            }
        }
        values
    }
    fn cursor_pos(&self) -> (i32, i32) {
        dpi_awareness();
        let mut p = POINT::default();
        // Safety: p is writable and lives through the call.
        unsafe {
            GetCursorPos(&mut p);
        }
        (p.x, p.y)
    }
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend {
        &mut self.clipboard
    }
    fn permissions(&self) -> Permissions {
        Permissions::not_required()
    }
    fn on_display_changed(&mut self, cb: Box<dyn FnMut() + Send>) {
        self.monitor = None;
        let (ready, rx) = mpsc::sync_channel(1);
        let handle = std::thread::Builder::new()
            .name("oe-win-displays".into())
            .spawn(move || {
                dpi_awareness();
                DISPLAY_CALLBACK.with(|c| *c.borrow_mut() = Some(cb));
                let result = window("OpenergyDisplay", Some(display_proc), false);
                let hwnd = match result {
                    Ok(h) => h,
                    Err(e) => {
                        let _ = ready.send(Err(e));
                        return;
                    }
                };
                // Safety: this thread owns hwnd; GetMessage initializes and drains its message queue.
                unsafe {
                    let mut msg = MSG::default();
                    PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
                    let _ = ready.send(Ok(windows_sys::Win32::System::Threading::GetCurrentThreadId()));
                    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    DestroyWindow(hwnd);
                }
                DISPLAY_CALLBACK.with(|c| *c.borrow_mut() = None);
            });
        match handle {
            Ok(h) => match rx.recv() {
                Ok(Ok(id)) => self.monitor = Some(MessageThread { id, handle: Some(h) }),
                _ => {
                    let _ = h.join();
                    log::warn!("Windows display watcher could not start");
                }
            },
            Err(e) => log::warn!("Windows display watcher failed: {e}"),
        }
    }
    fn prevent_sleep(&mut self, on: bool) {
        // Safety: SetThreadExecutionState takes flags and changes only this thread's power request.
        unsafe {
            SetThreadExecutionState(
                ES_CONTINUOUS
                    | if on {
                        ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
                    } else {
                        0
                    },
            );
        }
    }
}
