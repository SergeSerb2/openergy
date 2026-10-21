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
    thread: Option<MessageThread>,
    tx: Option<SyncSender<Command>>,
    captured: bool,
}
thread_local! { static STATE: RefCell<Option<Capture>> = const { RefCell::new(None) }; }
struct Capture {
    callback: Callback,
    captured: bool,
    pressed: BTreeMap<u16, u16>,
    key_state: [u8; 256],
    hidden: i32,
    saved_clip: Option<RECT>,
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
                let mut old = RECT::default();
                if GetClipCursor(&mut old) == 0 || GetCursorPos(&mut self.saved_pos) == 0 {
                    return Err(error("Read cursor state"));
                }
                let x = GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN) / 2;
                let y = GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN) / 2;
                let clip = RECT {
                    left: x,
                    top: y,
                    right: x + 1,
                    bottom: y + 1,
                };
                if ClipCursor(&clip) == 0 {
                    return Err(error("ClipCursor"));
                }
                self.saved_clip = Some(old);
                loop {
                    let n = ShowCursor(0);
                    self.hidden += 1;
                    if n < 0 {
                        break;
                    }
                }
            } else {
                if let Some(clip) = self.saved_clip {
                    if ClipCursor(&clip) == 0 {
                        return Err(error("Restore cursor clip"));
                    }
                    self.saved_clip = None;
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
        let swallow = (self.callback)(ev) == CaptureDecision::Swallow;
        if self.captured != swallow {
            if let Err(e) = self.grab(swallow) {
                log::error!("Windows capture transition failed: {e}");
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
                                GetKeyboardLayout(0),
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
        let h = std::thread::Builder::new()
            .name("oe-win-capture".into())
            .spawn(move || {
                native::dpi_awareness();
                STATE.with(|s| {
                    *s.borrow_mut() = Some(Capture {
                        callback,
                        captured: false,
                        pressed: BTreeMap::new(),
                        key_state: keymap::initial_state(),
                        hidden: 0,
                        saved_clip: None,
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
