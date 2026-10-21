use crate::{keymap, native::error};
use openergy_platform::{InputEvent, InputSink, Modifiers, PlatformError};
use std::collections::{BTreeMap, BTreeSet};
use windows_sys::Win32::UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*};
pub const TAG: usize = 0x4f454e47;
#[derive(Clone, Copy, PartialEq, Eq)]
struct Stroke {
    vk: u16,
    scan: u16,
    flags: u32,
}
impl Stroke {
    fn vk(key: u16, vk: u16) -> Self {
        // Safety: MapVirtualKeyW only translates the scalar virtual key.
        let scan = if vk == VK_PAUSE {
            0
        } else {
            // Safety: the scalar virtual key is valid for MapVirtualKeyW.
            unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC_EX) }
        };
        Self {
            vk: if scan == 0 { vk } else { 0 },
            scan: (scan & 0xff) as u16,
            flags: if scan == 0 { 0 } else { KEYEVENTF_SCANCODE }
                | if scan & 0xff00 == 0xe000 || keymap::extended(key, vk) {
                    KEYEVENTF_EXTENDEDKEY
                } else {
                    0
                },
        }
    }
    fn input(self, down: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: self.vk,
                    wScan: self.scan,
                    dwFlags: self.flags | if down { 0 } else { KEYEVENTF_KEYUP },
                    time: 0,
                    dwExtraInfo: TAG,
                },
            },
        }
    }
}
struct Held {
    stroke: Stroke,
    modifiers: Vec<(u16, Stroke)>,
}
#[derive(Default)]
pub struct Sink {
    keys: BTreeMap<u16, Held>,
    buttons: BTreeSet<u8>,
    owned_modifiers: BTreeMap<u16, Stroke>,
    pending_toggles: BTreeMap<u16, Stroke>,
    wheel: [i64; 2],
    hidden: i32,
}
fn send(input: INPUT) -> Result<(), PlatformError> {
    // Safety: input is an initialized INPUT with its matching union arm; SendInput copies it.
    if unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) } != 1 {
        Err(error("SendInput failed (possibly UIPI integrity restriction)"))
    } else {
        Ok(())
    }
}
fn mouse(flags: u32, data: u32, x: i32, y: i32) -> Result<(), PlatformError> {
    send(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: TAG,
            },
        },
    })
}
impl Sink {
    fn down(&mut self, key: u16, mods: Modifiers, button: u16) -> Result<(), PlatformError> {
        if self.keys.contains_key(&button) {
            return Ok(());
        }
        for (mask, vk, lock_key) in [
            (Modifiers::CAPS_LOCK, VK_CAPITAL, 0xefe5),
            (Modifiers::NUM_LOCK, VK_NUMLOCK, 0xef7f),
            (Modifiers::SCROLL_LOCK, VK_SCROLL, 0xef14),
        ] {
            // Safety: GetKeyState reads the toggle state for this scalar virtual key.
            let current = unsafe { GetKeyState(i32::from(vk)) } & 1 != 0;
            if current != (mods.0 & mask != 0) && key != lock_key {
                let stroke = Stroke::vk(lock_key, vk);
                self.pending_toggles.insert(vk, stroke);
                send(stroke.input(true))?;
                send(stroke.input(false))?;
                self.pending_toggles.remove(&vk);
            }
        }
        let printable = !(0xe000..=0xefff).contains(&key) && key >= 0x20;
        let mut required = if printable { mods.0 & !Modifiers::SHIFT } else { mods.0 };
        let stroke = if let Some(vk) = keymap::virtual_key(key) {
            Stroke::vk(key, vk)
        } else {
            if (0xe000..=0xefff).contains(&key) || char::from_u32(u32::from(key)).is_none() {
                return Err(error("Unsupported Windows KeyID"));
            }
            // Safety: current keyboard layout is a borrowed OS handle; VkKeyScanExW reads scalars.
            let mapping = unsafe { VkKeyScanExW(key, GetKeyboardLayout(0)) };
            if mapping == -1 {
                Stroke {
                    vk: 0,
                    scan: key,
                    flags: KEYEVENTF_UNICODE,
                }
            } else {
                let bits = ((mapping as u16) >> 8) & 7;
                let caps = mods.0 & Modifiers::CAPS_LOCK != 0
                    && char::from_u32(u32::from(key)).is_some_and(char::is_alphabetic);
                if (bits & 1 != 0) ^ caps {
                    required |= Modifiers::SHIFT;
                }
                if bits & 6 == 6 {
                    required |= Modifiers::ALTGR;
                } else {
                    if bits & 2 != 0 {
                        required |= Modifiers::CONTROL;
                    }
                    if bits & 4 != 0 {
                        required |= Modifiers::ALT;
                    }
                }
                Stroke::vk(key, mapping as u16 & 0xff)
            }
        };
        let held = Held {
            stroke,
            modifiers: Vec::new(),
        };
        self.keys.insert(button, held);
        if key == 0xee20 {
            required |= Modifiers::SHIFT;
        }
        for (mask, vk) in [
            (Modifiers::SHIFT, VK_LSHIFT),
            (Modifiers::CONTROL, VK_LCONTROL),
            (Modifiers::ALT, VK_LMENU),
            (Modifiers::SUPER, VK_LWIN),
            (Modifiers::META, VK_LWIN),
            (Modifiers::ALTGR, VK_RMENU),
        ] {
            if required & mask == 0 {
                continue;
            }
            let s = Stroke::vk(0, vk);
            if s == stroke {
                continue;
            }
            let ours = self.owned_modifiers.contains_key(&vk)
                || self.keys.iter().any(|(id, h)| *id != button && h.stroke == s);
            // Safety: GetAsyncKeyState reads only the scalar virtual-key state.
            if ours || unsafe { GetAsyncKeyState(i32::from(vk)) } >= 0 {
                if let Some(held) = self.keys.get_mut(&button) {
                    if !held.modifiers.iter().any(|(v, _)| *v == vk) {
                        held.modifiers.push((vk, s));
                    }
                }
                if !ours {
                    self.owned_modifiers.insert(vk, s);
                    send(s.input(true))?;
                }
            }
        }
        send(stroke.input(true))
    }
    fn up(&mut self, button: u16) -> Result<(), PlatformError> {
        if let Some(held) = self.keys.get(&button) {
            if !self.keys.iter().any(|(id, h)| {
                *id != button && (h.stroke == held.stroke || h.modifiers.iter().any(|(_, s)| *s == held.stroke))
            }) {
                send(held.stroke.input(false))?;
            }
            for (_, s) in held.modifiers.iter().rev() {
                if !self
                    .keys
                    .iter()
                    .any(|(id, h)| *id != button && (h.stroke == *s || h.modifiers.iter().any(|(_, m)| m == s)))
                {
                    send(s.input(false))?;
                }
            }
            self.keys.remove(&button);
            self.owned_modifiers.retain(|_, s| {
                self.keys
                    .values()
                    .any(|h| h.stroke == *s || h.modifiers.iter().any(|(_, m)| m == s))
            });
        }
        Ok(())
    }
}
impl InputSink for Sink {
    fn inject(&mut self, ev: &InputEvent) -> Result<(), PlatformError> {
        match *ev {
            InputEvent::KeyDown { key, modifiers, button } => self.down(key, modifiers, button),
            InputEvent::KeyUp { button, .. } => self.up(button),
            InputEvent::KeyRepeat { button, count, .. } => {
                if let Some(h) = self.keys.get(&button) {
                    for _ in 0..count.max(0) {
                        send(h.stroke.input(true))?;
                    }
                }
                Ok(())
            }
            InputEvent::MouseMoveAbs { x, y } => {
                // Safety: GetSystemMetrics reads scalar virtual-desktop metrics.
                let (left, top, width, height) = unsafe {
                    (
                        GetSystemMetrics(SM_XVIRTUALSCREEN),
                        GetSystemMetrics(SM_YVIRTUALSCREEN),
                        GetSystemMetrics(SM_CXVIRTUALSCREEN),
                        GetSystemMetrics(SM_CYVIRTUALSCREEN),
                    )
                };
                let normalize = |p: i32, start: i32, size: i32| -> i32 {
                    ((i64::from(p) - i64::from(start)).clamp(0, i64::from(size.saturating_sub(1).max(0))) * 65535
                        / i64::from(size.saturating_sub(1).max(1))) as i32
                };
                mouse(
                    MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    0,
                    normalize(x, left, width),
                    normalize(y, top, height),
                )
            }
            InputEvent::MouseMoveRel { dx, dy } => mouse(MOUSEEVENTF_MOVE, 0, dx, dy),
            InputEvent::MouseDown { button } | InputEvent::MouseUp { button } => {
                let down = matches!(ev, InputEvent::MouseDown { .. });
                let (flags, data) = match (button, down) {
                    (1, true) => (MOUSEEVENTF_LEFTDOWN, 0),
                    (1, false) => (MOUSEEVENTF_LEFTUP, 0),
                    (2, true) => (MOUSEEVENTF_MIDDLEDOWN, 0),
                    (2, false) => (MOUSEEVENTF_MIDDLEUP, 0),
                    (3, true) => (MOUSEEVENTF_RIGHTDOWN, 0),
                    (3, false) => (MOUSEEVENTF_RIGHTUP, 0),
                    (4 | 5, true) => (MOUSEEVENTF_XDOWN, u32::from(button - 3)),
                    (4 | 5, false) => (MOUSEEVENTF_XUP, u32::from(button - 3)),
                    _ => return Err(error("Mouse button must be 1..=5")),
                };
                if down {
                    self.buttons.insert(button);
                }
                mouse(flags, data, 0, 0)?;
                if !down {
                    self.buttons.remove(&button);
                }
                Ok(())
            }
            InputEvent::MouseWheel { dx, dy } => {
                for (axis, delta) in [dx, dy].into_iter().enumerate() {
                    self.wheel[axis] += i64::from(delta);
                    let notches = self.wheel[axis] / 120;
                    if notches != 0 {
                        let units = (notches * 120).clamp(i32::MIN.into(), i32::MAX.into()) as i32;
                        mouse(
                            if axis == 0 {
                                MOUSEEVENTF_HWHEEL
                            } else {
                                MOUSEEVENTF_WHEEL
                            },
                            units as u32,
                            0,
                            0,
                        )?;
                        self.wheel[axis] -= i64::from(units);
                    }
                }
                Ok(())
            }
        }
    }
    fn release_all(&mut self) {
        for key in self.keys.keys().copied().collect::<Vec<_>>() {
            if let Err(e) = self.up(key) {
                log::error!("Windows key release failed: {e}");
            }
        }
        for button in self.buttons.clone() {
            if let Err(e) = self.inject(&InputEvent::MouseUp { button }) {
                log::error!("Windows button release failed: {e}");
            }
        }
        for (vk, stroke) in self.pending_toggles.clone() {
            if send(stroke.input(false)).is_ok() {
                self.pending_toggles.remove(&vk);
            }
        }
        self.wheel = [0; 2];
    }
    fn set_cursor_visible(&mut self, visible: bool) {
        // Safety: ShowCursor adjusts this thread's display counter, balanced by `hidden`.
        unsafe {
            if visible {
                while self.hidden > 0 {
                    ShowCursor(1);
                    self.hidden -= 1;
                }
            } else if self.hidden == 0 {
                loop {
                    let count = ShowCursor(0);
                    self.hidden += 1;
                    if count < 0 {
                        break;
                    }
                }
            }
        }
    }
}
impl Drop for Sink {
    fn drop(&mut self) {
        self.release_all();
        self.set_cursor_visible(true);
    }
}
