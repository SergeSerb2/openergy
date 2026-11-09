use crate::{clipboard::Clipboard, WinNotice};
use openergy_platform::{ClipboardBackend, Permissions, PlatformError, ScreenRect, ScreenSystem};
use std::{
    ptr::{null, null_mut},
    sync::mpsc,
    thread::JoinHandle,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{
        LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
        Power::*,
    },
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
    notices: Option<MessageThread>,
    pub(crate) language_notifier: Option<mpsc::SyncSender<WinNotice>>,
}
impl Screens {
    pub fn new(clipboard: Clipboard) -> Self {
        Self {
            clipboard,
            monitor: None,
            notices: None,
            language_notifier: None,
        }
    }
    pub fn new_with_notices(clipboard: Clipboard) -> (Self, mpsc::Receiver<WinNotice>) {
        let mut screens = Self::new(clipboard);
        let (tx, rx) = mpsc::sync_channel(64);
        screens.language_notifier = Some(tx.clone());
        let failure = tx.clone();
        let (ready, ready_rx) = mpsc::sync_channel(1);
        let handle = std::thread::Builder::new()
            .name("oe-win-notices".into())
            .spawn(move || {
                NOTICE_SENDER.with(|s| *s.borrow_mut() = Some(tx));
                let hwnd = match window("OpenergyNotices", Some(notice_proc), true) {
                    Ok(h) => h,
                    Err(e) => {
                        send_notice(WinNotice::NotificationSetupFailed);
                        let _ = ready.send(Err(e));
                        NOTICE_SENDER.with(|s| *s.borrow_mut() = None);
                        return;
                    }
                };
                // Message-only windows do not receive system broadcasts, so
                // power delivery needs the directed suspend/resume
                // registration; session changes are likewise directed to
                // the registered HWND. This thread owns hwnd throughout.
                // Safety: registrations are released below before DestroyWindow.
                let power = unsafe { RegisterSuspendResumeNotification(hwnd, DEVICE_NOTIFY_WINDOW_HANDLE) };
                if power == 0 {
                    log::warn!("Windows suspend/resume notices unavailable");
                }
                let wts = load_wts().filter(|wts| {
                    // Safety: wts module stays loaded until wts is dropped after unregistering.
                    unsafe { (wts.register)(hwnd, NOTIFY_FOR_THIS_SESSION) != 0 }
                });
                if wts.is_none() {
                    log::warn!("Windows session lock notices unavailable");
                }
                if power == 0 && wts.is_none() {
                    send_notice(WinNotice::NotificationSetupFailed);
                    // Safety: this thread created hwnd and nothing registered against it.
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    let _ = ready.send(Err(error("lifecycle notification registration")));
                    NOTICE_SENDER.with(|s| *s.borrow_mut() = None);
                    return;
                }
                // Safety: this thread owns hwnd; GetMessage initializes and drains its message queue.
                unsafe {
                    let mut msg = MSG::default();
                    PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
                    let _ = ready.send(Ok(windows_sys::Win32::System::Threading::GetCurrentThreadId()));
                    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    if let Some(wts) = &wts {
                        (wts.unregister)(hwnd);
                    }
                    if power != 0 {
                        UnregisterSuspendResumeNotification(power);
                    }
                    DestroyWindow(hwnd);
                }
                NOTICE_SENDER.with(|s| *s.borrow_mut() = None);
            });
        match handle {
            Ok(h) => match ready_rx.recv() {
                Ok(Ok(id)) => screens.notices = Some(MessageThread { id, handle: Some(h) }),
                Ok(Err(_)) => {
                    let _ = h.join(); // The thread already queued the failure notice.
                }
                Err(_) => {
                    let _ = failure.send(WinNotice::NotificationSetupFailed);
                    let _ = h.join();
                }
            },
            Err(e) => {
                log::warn!("Windows lifecycle watcher failed: {e}");
                let _ = failure.send(WinNotice::NotificationSetupFailed);
            }
        }
        (screens, rx)
    }
}
// WTS session notifications sit behind the Win32_System_RemoteDesktop
// feature, which is not enabled; resolve them from wtsapi32.dll at runtime.
const NOTIFY_FOR_THIS_SESSION: u32 = 0;
type WtsRegister = unsafe extern "system" fn(HWND, u32) -> windows_sys::core::BOOL;
type WtsUnregister = unsafe extern "system" fn(HWND) -> windows_sys::core::BOOL;
struct Wts {
    module: HMODULE,
    register: WtsRegister,
    unregister: WtsUnregister,
}
impl Drop for Wts {
    fn drop(&mut self) {
        // Safety: the owning thread unregisters before this value is dropped.
        unsafe {
            FreeLibrary(self.module);
        }
    }
}
fn load_wts() -> Option<Wts> {
    // Safety: names are NUL-terminated literals and the resolved functions
    // keep the documented WTS signatures for the life of the module handle.
    unsafe {
        let module = LoadLibraryW(wide("wtsapi32.dll").as_ptr());
        if module.is_null() {
            return None;
        }
        let register = GetProcAddress(module, c"WTSRegisterSessionNotification".as_ptr().cast::<u8>());
        let unregister = GetProcAddress(module, c"WTSUnRegisterSessionNotification".as_ptr().cast::<u8>());
        match (register, unregister) {
            (Some(register), Some(unregister)) => Some(Wts {
                module,
                register: std::mem::transmute::<unsafe extern "system" fn() -> isize, WtsRegister>(register),
                unregister: std::mem::transmute::<unsafe extern "system" fn() -> isize, WtsUnregister>(unregister),
            }),
            _ => {
                FreeLibrary(module);
                None
            }
        }
    }
}
thread_local! { static NOTICE_SENDER: std::cell::RefCell<Option<mpsc::SyncSender<WinNotice>>> = const { std::cell::RefCell::new(None) }; }
fn send_notice(notice: WinNotice) {
    NOTICE_SENDER.with(|s| {
        if let Some(tx) = s.borrow().as_ref() {
            // The window procedure must never block a system power/session
            // broadcast. Lifecycle events are rare, so a full queue means
            // the consumer is stuck and dropping is the safe choice.
            match tx.try_send(notice) {
                Ok(()) | Err(mpsc::TrySendError::Disconnected(_)) => {}
                Err(mpsc::TrySendError::Full(_)) => {
                    log::error!("Windows lifecycle notice queue full; notice dropped")
                }
            }
        }
    });
}
unsafe extern "system" fn notice_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_INPUTLANGCHANGE => {
            send_notice(WinNotice::KeyboardLanguagesChanged);
            // Safety: preserve normal input-language notification handling.
            unsafe { DefWindowProcW(hwnd, msg, w, l) }
        }
        WM_POWERBROADCAST => {
            match w as u32 {
                PBT_APMSUSPEND => send_notice(WinNotice::Sleeping),
                // RESUMEAUTOMATIC fires on every resume; RESUMESUSPEND only
                // ever follows it (user present), so mapping both would
                // emit a duplicate Wake per resume.
                PBT_APMRESUMEAUTOMATIC => send_notice(WinNotice::Woke),
                _ => {}
            }
            1 // TRUE: allow the power transition.
        }
        WM_WTSSESSION_CHANGE => {
            match w as u32 {
                WTS_SESSION_LOCK => send_notice(WinNotice::SessionLocked),
                WTS_SESSION_UNLOCK => send_notice(WinNotice::SessionUnlocked),
                _ => {}
            }
            0
        }
        // Safety: all unhandled messages are forwarded unchanged to the system window procedure.
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
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
    fn keyboard_languages(&self) -> openergy_platform::KeyboardLanguages {
        let mut available = Vec::new();
        for code in keyboard_layouts().into_iter().filter_map(layout_language) {
            if !available.contains(&code) {
                available.push(code);
            }
        }
        openergy_platform::KeyboardLanguages {
            available,
            current: layout_language(keyboard_layout()),
        }
    }
    fn set_keyboard_language(&mut self, lang: &str) -> Result<(), PlatformError> {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::ActivateKeyboardLayout;
        let code = openergy_platform::keyboard_language(lang).ok_or(PlatformError::NotImplemented)?;
        let layout = keyboard_layouts()
            .into_iter()
            .find(|l| layout_language(*l) == Some(code))
            .ok_or(PlatformError::NotImplemented)?;
        // Safety: select an installed HKL on our thread and request it on the
        // foreground window. Posting succeeds without confirming application acceptance.
        unsafe {
            ActivateKeyboardLayout(layout, 0);
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() || PostMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, 0, layout as isize) == 0 {
                return Err(error("keyboard language request could not be posted"));
            }
        }
        log::info!("Windows keyboard language switch attempted; foreground application may ignore it");
        Ok(())
    }
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

fn layout_language(layout: windows_sys::Win32::UI::Input::KeyboardAndMouse::HKL) -> Option<[u8; 2]> {
    use windows_sys::Win32::Globalization::{GetLocaleInfoEx, LCIDToLocaleName, LOCALE_SISO639LANGNAME};
    // The low word is the LANGID, including its sublanguage. ISO639 removes
    // the regional distinction without guessing from a layout's variant id.
    let mut locale = [0u16; 85];
    let mut code = [0u16; 9];
    // Safety: both APIs receive writable, bounded UTF-16 buffers.
    unsafe {
        if LCIDToLocaleName(
            (layout as usize & 0xffff) as u32,
            locale.as_mut_ptr(),
            locale.len() as i32,
            0,
        ) == 0
        {
            return None;
        }
        let n = GetLocaleInfoEx(
            locale.as_ptr(),
            LOCALE_SISO639LANGNAME,
            code.as_mut_ptr(),
            code.len() as i32,
        );
        if n <= 1 {
            return None;
        }
        openergy_platform::keyboard_language(&String::from_utf16_lossy(&code[..n as usize - 1]))
    }
}
fn keyboard_layouts() -> Vec<windows_sys::Win32::UI::Input::KeyboardAndMouse::HKL> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayoutList;
    // Safety: first query is sizing only; second fills at most count handles.
    unsafe {
        let count = GetKeyboardLayoutList(0, null_mut());
        if count <= 0 {
            return Vec::new();
        }
        let mut layouts = vec![null_mut(); count as usize];
        let n = GetKeyboardLayoutList(count, layouts.as_mut_ptr());
        layouts.truncate(n.max(0) as usize);
        layouts
    }
}
