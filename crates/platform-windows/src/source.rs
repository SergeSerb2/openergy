use crate::{
    keymap,
    native::{self, error, MessageThread},
    sink::TAG,
};
use openergy_platform::{CaptureDecision, InputEvent, InputSource, PlatformError};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ptr::{null, null_mut},
    sync::mpsc::{self, SyncSender},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi,
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{
        Input::{self, KeyboardAndMouse::*},
        WindowsAndMessaging::*,
    },
};
type Callback = Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>;
const COMMAND: u32 = WM_APP + 8;
enum Command {
    Capture(bool, SyncSender<Result<(), PlatformError>>),
    Warp(i32, i32, SyncSender<Result<(), PlatformError>>),
}
#[derive(Default)]
pub struct Source {
    pub(crate) language_notifier: Option<SyncSender<crate::WinNotice>>,
    thread: Option<MessageThread>,
    tx: Option<SyncSender<Command>>,
    captured: bool,
}
thread_local! { static STATE: RefCell<Option<Capture>> = const { RefCell::new(None) }; }
struct Capture {
    language_notifier: Option<SyncSender<crate::WinNotice>>,
    callback: Callback,
    captured: bool,
    pressed: BTreeMap<u16, u16>,
    key_state: [u8; 256],
    hidden: i32,
    saved_clip: Option<RECT>,
    clipped: bool,
    clip_failures: u64,
    grab_failures: u64,
    saved_pos: POINT,
}
impl Capture {
    fn grab(&mut self, captured: bool) -> Result<(), PlatformError> {
        if captured == self.captured {
            return Ok(());
        }
        // Safety: stack POINT/RECT values are valid; all cursor changes occur on the hook thread.
        unsafe {
            if captured {
                if GetCursorPos(&mut self.saved_pos) == 0 {
                    return Err(error("Read cursor state"));
                }
                // Park on the center of the display under the cursor. The
                // virtual-screen center can fall in a gap between displays.
                let monitor = Gdi::MonitorFromPoint(self.saved_pos, Gdi::MONITOR_DEFAULTTONEAREST);
                let mut info = Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<Gdi::MONITORINFO>() as u32,
                    ..Default::default()
                };
                let (x, y) = if !monitor.is_null() && Gdi::GetMonitorInfoW(monitor, &mut info) != 0 {
                    let m = info.rcMonitor;
                    (m.left + (m.right - m.left) / 2, m.top + (m.bottom - m.top) / 2)
                } else {
                    (self.saved_pos.x, self.saved_pos.y)
                };
                let mut old = RECT::default();
                let had_clip = GetClipCursor(&mut old) != 0;
                let clip = RECT {
                    left: x,
                    top: y,
                    right: x + 1,
                    bottom: y + 1,
                };
                // The hook swallows local moves while captured, so the clip
                // is only a second guard. Windows refuses it on a secure
                // desktop or for some foreground apps; capture continues.
                if ClipCursor(&clip) != 0 {
                    self.saved_clip = had_clip.then_some(old);
                    self.clipped = true;
                } else {
                    self.clip_failures += 1;
                    if self.clip_failures.is_power_of_two() {
                        log::warn!(
                            "ClipCursor refused ({} times): {}; capture continues without it",
                            self.clip_failures,
                            std::io::Error::last_os_error()
                        );
                    }
                }
                loop {
                    let n = ShowCursor(0);
                    self.hidden += 1;
                    if n < 0 {
                        break;
                    }
                }
            } else {
                // Release always completes: a failed restore must not leave local input captured.
                if std::mem::take(&mut self.clipped) {
                    let restored = self.saved_clip.take().is_some_and(|clip| ClipCursor(&clip) != 0);
                    if !restored && ClipCursor(null()) == 0 {
                        log::warn!("Release cursor clip: {}", std::io::Error::last_os_error());
                    }
                }
                while self.hidden > 0 {
                    ShowCursor(1);
                    self.hidden -= 1;
                }
                SetCursorPos(self.saved_pos.x, self.saved_pos.y);
            }
        }
        self.captured = captured;
        Ok(())
    }
    fn emit(&mut self, ev: InputEvent) -> bool {
        let swallow = match (self.callback)(ev) {
            CaptureDecision::Swallow => true,
            CaptureDecision::PassToOs => false,
            CaptureDecision::PassOnce => return false,
        };
        if self.captured != swallow {
            if let Err(e) = self.grab(swallow) {
                // A secure desktop can refuse every attempt; log sparsely.
                self.grab_failures += 1;
                if self.grab_failures.is_power_of_two() {
                    log::error!("Windows capture transition failed ({} times): {e}", self.grab_failures);
                }
            }
        }
        swallow
    }
}
unsafe extern "system" fn keyboard(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        // Safety: WH_KEYBOARD_LL passes a KBDLLHOOKSTRUCT for nonnegative hook codes.
        let e = unsafe { &*(l as *const KBDLLHOOKSTRUCT) };
        if e.flags & LLKHF_INJECTED == 0 && e.dwExtraInfo != TAG {
            let swallow = STATE.with(|state| {
                let Ok(mut state) = state.try_borrow_mut() else {
                    return false;
                };
                let Some(s) = state.as_mut() else {
                    return false;
                };
                let down = w as u32 == WM_KEYDOWN || w as u32 == WM_SYSKEYDOWN;
                let extended = e.flags & LLKHF_EXTENDED != 0;
                let button = ((e.scanCode & 0xff) | if extended { 0x100 } else { 0 }) as u16 + 1;
                let vk = match e.vkCode as u16 {
                    VK_SHIFT => {
                        if e.scanCode == 0x36 {
                            VK_RSHIFT
                        } else {
                            VK_LSHIFT
                        }
                    }
                    VK_CONTROL => {
                        if extended {
                            VK_RCONTROL
                        } else {
                            VK_LCONTROL
                        }
                    }
                    VK_MENU => {
                        if extended {
                            VK_RMENU
                        } else {
                            VK_LMENU
                        }
                    }
                    vk => vk,
                };
                let index = usize::from(vk).min(255);
                if down && !s.pressed.contains_key(&button) && matches!(vk, VK_CAPITAL | VK_NUMLOCK | VK_SCROLL) {
                    s.key_state[index] ^= 1;
                }
                s.key_state[index] = (s.key_state[index] & 1) | if down { 0x80 } else { 0 };
                for (generic, left, right) in [
                    (VK_SHIFT, VK_LSHIFT, VK_RSHIFT),
                    (VK_CONTROL, VK_LCONTROL, VK_RCONTROL),
                    (VK_MENU, VK_LMENU, VK_RMENU),
                ] {
                    s.key_state[usize::from(generic)] =
                        (s.key_state[usize::from(left)] | s.key_state[usize::from(right)]) & 0x80;
                }
                let key = s
                    .pressed
                    .get(&button)
                    .copied()
                    .or_else(|| keymap::named_key(e.vkCode as u16, e.scanCode, extended))
                    .or_else(|| {
                        let mut state = s.key_state;
                        if keymap::modifiers(&state).0 & openergy_platform::Modifiers::ALTGR == 0 {
                            for vk in [VK_CONTROL, VK_LCONTROL, VK_RCONTROL, VK_MENU, VK_LMENU, VK_RMENU] {
                                state[usize::from(vk)] = 0;
                            }
                        }
                        // Safety: fixed buffers satisfy GetKeyboardState/ToUnicodeEx; flag 4 preserves dead-key state.
                        unsafe {
                            let mut text = [0u16; 8];
                            let count = ToUnicodeEx(
                                e.vkCode,
                                e.scanCode,
                                state.as_ptr(),
                                text.as_mut_ptr(),
                                8,
                                4,
                                native::keyboard_layout(),
                            );
                            if count != 0 && !(0xd800..=0xdfff).contains(&text[0]) {
                                Some(text[0])
                            } else {
                                None
                            }
                        }
                    });
                let Some(key) = key else {
                    return s.captured;
                };
                let modifiers = keymap::modifiers(&s.key_state);
                let ev = if down {
                    if s.pressed.insert(button, key).is_none() {
                        InputEvent::KeyDown { key, modifiers, button }
                    } else {
                        InputEvent::KeyRepeat {
                            key,
                            modifiers,
                            count: 1,
                            button,
                        }
                    }
                } else {
                    s.pressed.remove(&button);
                    InputEvent::KeyUp { key, modifiers, button }
                };
                s.emit(ev)
            });
            if swallow {
                return 1;
            }
        }
    }
    // Safety: unchanged hook arguments are forwarded to the next hook.
    unsafe { CallNextHookEx(null_mut(), code, w, l) }
}
unsafe extern "system" fn mouse(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        // Safety: WH_MOUSE_LL supplies a MSLLHOOKSTRUCT for nonnegative hook codes.
        let e = unsafe { &*(l as *const MSLLHOOKSTRUCT) };
        if e.flags & LLMHF_INJECTED == 0 && e.dwExtraInfo != TAG {
            let swallow = STATE.with(|state| {
                let Ok(mut state) = state.try_borrow_mut() else {
                    return false;
                };
                let Some(s) = state.as_mut() else {
                    return false;
                };
                let msg = w as u32;
                let button = match msg {
                    WM_LBUTTONDOWN | WM_LBUTTONUP => 1,
                    WM_MBUTTONDOWN | WM_MBUTTONUP => 2,
                    WM_RBUTTONDOWN | WM_RBUTTONUP => 3,
                    WM_XBUTTONDOWN | WM_XBUTTONUP => {
                        if e.mouseData >> 16 == 1 {
                            4
                        } else {
                            5
                        }
                    }
                    _ => 0,
                };
                let ev = match msg {
                    WM_MOUSEMOVE if s.captured => return true,
                    WM_MOUSEMOVE => Some(InputEvent::MouseMoveAbs { x: e.pt.x, y: e.pt.y }),
                    WM_MOUSEWHEEL => Some(InputEvent::MouseWheel {
                        dx: 0,
                        dy: (e.mouseData >> 16) as i16 as i32,
                    }),
                    WM_MOUSEHWHEEL => Some(InputEvent::MouseWheel {
                        dx: (e.mouseData >> 16) as i16 as i32,
                        dy: 0,
                    }),
                    WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN => {
                        Some(InputEvent::MouseDown { button })
                    }
                    WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP | WM_XBUTTONUP => Some(InputEvent::MouseUp { button }),
                    _ => None,
                };
                ev.map(|e| s.emit(e)).unwrap_or(s.captured)
            });
            if swallow {
                return 1;
            }
        }
    }
    // Safety: unchanged hook arguments are forwarded to the next hook.
    unsafe { CallNextHookEx(null_mut(), code, w, l) }
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_INPUTLANGCHANGE {
        STATE.with(|s| {
            if let Ok(state) = s.try_borrow() {
                if let Some(tx) = state.as_ref().and_then(|s| s.language_notifier.as_ref()) {
                    let _ = tx.try_send(crate::WinNotice::KeyboardLanguagesChanged);
                }
            }
        });
    }
    if msg == WM_INPUT {
        let mut input = Input::RAWINPUT::default();
        let mut size = std::mem::size_of::<Input::RAWINPUT>() as u32;
        // Safety: RAWINPUT is aligned writable storage; handle is supplied with WM_INPUT.
        unsafe {
            if Input::GetRawInputData(
                l as Input::HRAWINPUT,
                Input::RID_INPUT,
                &mut input as *mut _ as *mut _,
                &mut size,
                std::mem::size_of::<Input::RAWINPUTHEADER>() as u32,
            ) != u32::MAX
                && input.header.dwType == Input::RIM_TYPEMOUSE
            {
                let mouse = input.data.mouse;
                if mouse.usFlags & 1 == 0 && mouse.ulExtraInformation != TAG as u32 {
                    STATE.with(|s| {
                        if let Ok(mut guard) = s.try_borrow_mut() {
                            if let Some(s) = guard.as_mut() {
                                if s.captured && (mouse.lLastX != 0 || mouse.lLastY != 0) {
                                    s.emit(InputEvent::MouseMoveRel {
                                        dx: mouse.lLastX,
                                        dy: mouse.lLastY,
                                    });
                                }
                            }
                        }
                    });
                }
            }
        }
    }
    // Safety: WM_INPUT must also reach DefWindowProc for raw-input cleanup.
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}
impl Source {
    pub(crate) fn with_language_notifier(language_notifier: Option<SyncSender<crate::WinNotice>>) -> Self {
        Self {
            language_notifier,
            ..Self::default()
        }
    }
    fn command(
        &self,
        build: impl FnOnce(SyncSender<Result<(), PlatformError>>) -> Command,
    ) -> Result<(), PlatformError> {
        let thread = self
            .thread
            .as_ref()
            .ok_or_else(|| error("Windows capture is not started"))?;
        let tx = self
            .tx
            .as_ref()
            .ok_or_else(|| error("Windows capture queue is closed"))?;
        let (reply, rx) = mpsc::sync_channel(1);
        tx.send(build(reply)).map_err(error)?;
        // Safety: the queue is ready and belongs to our owned hook thread.
        if unsafe { PostThreadMessageW(thread.id, COMMAND, 0, 0) } == 0 {
            return Err(error("PostThreadMessageW"));
        }
        rx.recv().map_err(error)?
    }
}
impl InputSource for Source {
    fn start(&mut self, callback: Callback) -> Result<(), PlatformError> {
        self.stop();
        let (ready, rx) = mpsc::sync_channel(1);
        let (tx, commands) = mpsc::sync_channel(32);
        let captured = self.captured;
        let language_notifier = self.language_notifier.clone();
        let h = std::thread::Builder::new()
            .name("oe-win-capture".into())
            .spawn(move || {
                native::dpi_awareness();
                STATE.with(|s| {
                    *s.borrow_mut() = Some(Capture {
                        callback,
                        language_notifier,
                        captured: false,
                        pressed: BTreeMap::new(),
                        key_state: keymap::initial_state(),
                        hidden: 0,
                        saved_clip: None,
                        clipped: false,
                        clip_failures: 0,
                        grab_failures: 0,
                        saved_pos: POINT::default(),
                    })
                });
                let result = (|| -> Result<(), PlatformError> {
                    let hwnd = native::window("OpenergyInput", Some(window_proc), true)?;
                    // Safety: static callbacks and this thread's hidden window stay valid through the loop.
                    unsafe {
                        let instance = GetModuleHandleW(null());
                        let kh = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), instance, 0);
                        let mh = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), instance, 0);
                        if kh.is_null() || mh.is_null() {
                            if !kh.is_null() {
                                UnhookWindowsHookEx(kh);
                            }
                            if !mh.is_null() {
                                UnhookWindowsHookEx(mh);
                            }
                            DestroyWindow(hwnd);
                            return Err(error("Install low-level input hooks"));
                        }
                        let device = Input::RAWINPUTDEVICE {
                            usUsagePage: 1,
                            usUsage: 2,
                            dwFlags: Input::RIDEV_INPUTSINK,
                            hwndTarget: hwnd,
                        };
                        if Input::RegisterRawInputDevices(
                            &device,
                            1,
                            std::mem::size_of::<Input::RAWINPUTDEVICE>() as u32,
                        ) == 0
                        {
                            UnhookWindowsHookEx(kh);
                            UnhookWindowsHookEx(mh);
                            DestroyWindow(hwnd);
                            return Err(error("Register raw mouse input"));
                        }
                        let mut msg = MSG::default();
                        PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
                        let initial = STATE
                            .with(|s| s.borrow_mut().as_mut().map(|s| s.grab(captured)))
                            .unwrap_or(Ok(()));
                        if let Err(e) = initial {
                            unregister_mouse();
                            UnhookWindowsHookEx(kh);
                            UnhookWindowsHookEx(mh);
                            DestroyWindow(hwnd);
                            return Err(e);
                        }
                        let _ = ready.send(Ok(GetCurrentThreadId()));
                        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                            if msg.message == COMMAND {
                                while let Ok(command) = commands.try_recv() {
                                    match command {
                                        Command::Capture(value, reply) => {
                                            let result = STATE
                                                .with(|s| s.borrow_mut().as_mut().map(|s| s.grab(value)))
                                                .unwrap_or_else(|| Err(error("Missing Windows capture state")));
                                            let _ = reply.send(result);
                                        }
                                        Command::Warp(x, y, reply) => {
                                            let _ = reply.send(if SetCursorPos(x, y) == 0 {
                                                Err(error("SetCursorPos"))
                                            } else {
                                                Ok(())
                                            });
                                        }
                                    }
                                }
                            } else {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }
                        STATE.with(|s| {
                            if let Some(s) = s.borrow_mut().as_mut() {
                                let _ = s.grab(false);
                            }
                        });
                        UnhookWindowsHookEx(kh);
                        UnhookWindowsHookEx(mh);
                        unregister_mouse();
                        DestroyWindow(hwnd);
                    }
                    Ok(())
                })();
                if let Err(e) = result {
                    let _ = ready.send(Err(e));
                }
                STATE.with(|s| *s.borrow_mut() = None);
            })
            .map_err(error)?;
        match rx.recv().map_err(error)? {
            Ok(id) => {
                self.thread = Some(MessageThread { id, handle: Some(h) });
                self.tx = Some(tx);
                Ok(())
            }
            Err(e) => {
                let _ = h.join();
                Err(e)
            }
        }
    }
    fn stop(&mut self) {
        self.thread = None;
        self.tx = None;
        self.captured = false;
    }
    fn set_cursor_captured(&mut self, captured: bool) -> Result<(), PlatformError> {
        if self.thread.is_some() {
            self.command(|r| Command::Capture(captured, r))?;
        }
        self.captured = captured;
        Ok(())
    }
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<(), PlatformError> {
        self.command(|r| Command::Warp(x, y, r))
    }
}
fn unregister_mouse() {
    let device = Input::RAWINPUTDEVICE {
        usUsagePage: 1,
        usUsage: 2,
        dwFlags: Input::RIDEV_REMOVE,
        hwndTarget: null_mut(),
    };
    // Safety: RIDEV_REMOVE requires a null target and unregisters this process's mouse sink.
    if unsafe { Input::RegisterRawInputDevices(&device, 1, std::mem::size_of::<Input::RAWINPUTDEVICE>() as u32) } == 0 {
        log::warn!("Unregister raw mouse input: {}", std::io::Error::last_os_error());
    }
}
