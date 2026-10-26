use crate::{
    keymap::{keysym, Keymap},
    native::{connect, error},
};
use openergy_platform::{InputEvent, InputSink, Modifiers, PlatformError};
use openergy_protocol::keys;
use std::collections::{BTreeMap, BTreeSet};
use x11rb::{
    connection::Connection,
    protocol::{
        xfixes::ConnectionExt as _,
        xinput::ConnectionExt as _,
        xproto::{
            ConnectionExt as _, BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
            MOTION_NOTIFY_EVENT,
        },
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
    CURRENT_TIME, NONE,
};

#[derive(Clone)]
struct Held {
    code: u8,
    owned: bool,
    modifiers: Vec<u8>,
    restore: Option<(Vec<u32>, Vec<u32>)>,
}
pub struct Sink {
    c: RustConnection,
    root: u32,
    map: Keymap,
    keys: BTreeMap<u16, Held>,
    buttons: BTreeMap<u8, bool>,
    owned_modifiers: BTreeSet<u8>,
    pending_toggles: BTreeSet<u8>,
    wheel: [i64; 2],
    hidden: bool,
}
impl Sink {
    pub fn new(display: Option<&str>) -> Result<Self, PlatformError> {
        let (c, root) = connect(display)?;
        c.xtest_get_version(2, 2).map_err(error)?.reply().map_err(error)?;
        c.xfixes_query_version(4, 0).map_err(error)?.reply().map_err(error)?;
        c.xinput_xi_query_version(2, 0).map_err(error)?.reply().map_err(error)?;
        let map = Keymap::load(&c)?;
        Ok(Self {
            c,
            root,
            map,
            keys: BTreeMap::new(),
            buttons: BTreeMap::new(),
            owned_modifiers: BTreeSet::new(),
            pending_toggles: BTreeSet::new(),
            wheel: [0; 2],
            hidden: false,
        })
    }
    fn fake(&self, kind: u8, detail: u8, root: u32, x: i16, y: i16) -> Result<(), PlatformError> {
        self.c
            .xtest_fake_input(kind, detail, CURRENT_TIME, root, x, y, 0)
            .map_err(error)?
            .check()
            .map_err(error)
    }
    fn key(&self, code: u8, down: bool) -> Result<(), PlatformError> {
        self.fake(if down { KEY_PRESS_EVENT } else { KEY_RELEASE_EVENT }, code, NONE, 0, 0)
    }
    fn down(&mut self, key: u16, mods: Modifiers, button: u16) -> Result<(), PlatformError> {
        if self.keys.contains_key(&button) {
            return Ok(());
        }
        if (0xd800..=0xdfff).contains(&key) {
            return Err(error("Surrogate is not a Unicode KeyID"));
        }
        self.map = Keymap::load(&self.c)?;
        let state = self
            .c
            .query_pointer(self.root)
            .map_err(error)?
            .reply()
            .map_err(error)?
            .mask;
        let actual = self.map.modifiers(state.into()).0;
        let physical = self.c.query_keymap().map_err(error)?.reply().map_err(error)?.keys;
        for (mask, lock_key) in [
            (Modifiers::CAPS_LOCK, keys::CAPS_LOCK),
            (Modifiers::NUM_LOCK, keys::NUM_LOCK),
            (Modifiers::SCROLL_LOCK, keys::SCROLL_LOCK),
        ] {
            if (actual ^ mods.0) & mask != 0 && key != lock_key {
                if let Some(code) = self.map.modifier_code(mask) {
                    // A toggle is a press/release pair. Never use it to release
                    // a lock key that another input path is still holding.
                    if physical[usize::from(code) / 8] & (1 << (code % 8)) != 0 {
                        return Err(error("Cannot synchronize X11 lock state while its key is held"));
                    }
                    self.pending_toggles.insert(code);
                    self.key(code, true)?;
                    self.key(code, false)?;
                    self.pending_toggles.remove(&code);
                }
            }
        }
        let sym = keysym(key);
        let (code, shift, altgr, restore) = if let Some((code, shift, altgr)) = self.map.find(sym) {
            (code, shift, altgr, None)
        } else {
            let pressed = self.c.query_keymap().map_err(error)?.reply().map_err(error)?.keys;
            let code = (self.map.first..=self.c.setup().max_keycode)
                .rev()
                .find(|code| {
                    !self.map.modifier_codes.contains(code)
                        && pressed[usize::from(*code) / 8] & (1 << (*code % 8)) == 0
                        && !self.keys.values().any(|k| k.code == *code)
                        && self.map.row(*code).iter().all(|s| *s == 0)
                })
                .ok_or_else(|| error("No unused X keycode for Unicode fallback"))?;
            let old = self.map.row(code).to_vec();
            let mut row = vec![0; self.map.width];
            row[0] = sym;
            self.c
                .change_keyboard_mapping(1, code, self.map.width as u8, &row)
                .map_err(error)?
                .check()
                .map_err(error)?;
            let installed = match self
                .c
                .get_keyboard_mapping(code, 1)
                .map_err(error)
                .and_then(|c| c.reply().map_err(error))
            {
                Ok(reply) => reply.keysyms,
                Err(e) => {
                    if let Ok(cookie) = self.c.change_keyboard_mapping(1, code, old.len() as u8, &old) {
                        let _ = cookie.check();
                    }
                    return Err(e);
                }
            };
            (code, false, false, Some((old, installed)))
        };
        let held = Held {
            code,
            owned: self.owned_modifiers.contains(&code)
                || self.keys.values().any(|h| h.code == code && h.owned)
                || physical[usize::from(code) / 8] & (1 << (code % 8)) == 0,
            modifiers: Vec::new(),
            restore,
        };
        let printable = !(0xe000..=keys::DELETE).contains(&key) && key >= keys::SPACE;
        let caps =
            mods.0 & Modifiers::CAPS_LOCK != 0 && char::from_u32(u32::from(key)).is_some_and(char::is_alphabetic);
        let row = self.map.row(code);
        let keypad_num = mods.0 & Modifiers::NUM_LOCK != 0
            && row.first().is_some_and(|a| (0xff95..=0xff9f).contains(a))
            && row
                .get(1)
                .is_some_and(|b| (0xffb0..=0xffb9).contains(b) || *b == 0xffae);
        let required = (if printable { mods.0 & !Modifiers::SHIFT } else { mods.0 })
            | if shift ^ caps ^ keypad_num { Modifiers::SHIFT } else { 0 }
            | if altgr { Modifiers::ALTGR } else { 0 };
        // Record before any post so release_all recovers partial injection failures.
        self.keys.insert(button, held);
        for mask in [
            Modifiers::SHIFT,
            Modifiers::CONTROL,
            Modifiers::ALT,
            Modifiers::META,
            Modifiers::SUPER,
            Modifiers::ALTGR,
        ] {
            if required & mask == 0 {
                continue;
            }
            if let Some(m) = self.map.modifier_code(mask) {
                if m == code {
                    continue;
                }
                let ours = self.owned_modifiers.contains(&m)
                    || self.keys.iter().any(|(id, h)| *id != button && h.code == m && h.owned);
                if ours || physical[usize::from(m) / 8] & (1 << (m % 8)) == 0 {
                    if let Some(held) = self.keys.get_mut(&button) {
                        held.modifiers.push(m);
                    }
                    if !ours {
                        self.owned_modifiers.insert(m);
                        self.key(m, true)?;
                    }
                }
            }
        }
        if self.keys[&button].owned {
            self.key(code, true)?;
        }
        Ok(())
    }
    fn up(&mut self, button: u16) -> Result<(), PlatformError> {
        if let Some(held) = self.keys.get(&button).cloned() {
            let needed = self
                .keys
                .iter()
                .any(|(id, h)| *id != button && (h.code == held.code || h.modifiers.contains(&held.code)));
            if held.owned && !needed {
                self.key(held.code, false)?;
            }
            for m in held.modifiers.iter().rev() {
                if !self
                    .keys
                    .iter()
                    .any(|(id, h)| *id != button && (h.code == *m || h.modifiers.contains(m)))
                {
                    self.key(*m, false)?;
                }
            }
            if let Some((row, installed)) = &held.restore {
                if !needed {
                    let current = self
                        .c
                        .get_keyboard_mapping(held.code, 1)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?
                        .keysyms;
                    // XKB may change the global row width while this key is held.
                    // Compare without trailing NoSymbol padding. Restore only our
                    // mapping; a newer user/layout mapping belongs to its writer.
                    let trimmed = |values: &[u32]| values.iter().rposition(|s| *s != 0).map_or(0, |i| i + 1);
                    if current[..trimmed(&current)] == installed[..trimmed(installed)] {
                        self.c
                            .change_keyboard_mapping(1, held.code, row.len() as u8, row)
                            .map_err(error)?
                            .check()
                            .map_err(error)?;
                    }
                }
            }
            self.keys.remove(&button);
            if needed {
                if let Some(row) = held.restore {
                    if let Some(next) = self.keys.values_mut().find(|h| h.code == held.code) {
                        next.restore = Some(row);
                    }
                }
            }
            self.owned_modifiers
                .retain(|m| self.keys.values().any(|h| h.code == *m || h.modifiers.contains(m)));
        }
        Ok(())
    }
}
impl InputSink for Sink {
    fn inject(&mut self, ev: &InputEvent) -> Result<(), PlatformError> {
        match *ev {
            InputEvent::KeyDown { key, modifiers, button } => self.down(key, modifiers, button)?,
            InputEvent::KeyUp { button, .. } => self.up(button)?,
            InputEvent::KeyRepeat { button, count, .. } => {
                if let Some(k) = self.keys.get(&button).filter(|k| k.owned) {
                    for _ in 0..count.max(0) {
                        self.key(k.code, false)?;
                        self.key(k.code, true)?;
                    }
                }
            }
            InputEvent::MouseDown { button } | InputEvent::MouseUp { button } => {
                let code = match button {
                    1..=3 => button,
                    4 => 8,
                    5 => 9,
                    _ => return Err(error("Mouse button must be 1..=5")),
                };
                let down = matches!(ev, InputEvent::MouseDown { .. });
                if down {
                    if self.buttons.contains_key(&code) {
                        return Ok(());
                    }
                    let pointer = self
                        .c
                        .xinput_xi_get_client_pointer(NONE)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?;
                    if !pointer.set {
                        return Err(error("X11 client has no master pointer"));
                    }
                    let state = self
                        .c
                        .xinput_xi_query_pointer(self.root, pointer.deviceid)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?
                        .buttons;
                    let owned = state
                        .get(usize::from(code) / 32)
                        .is_none_or(|bits| bits & (1 << (code % 32)) == 0);
                    self.buttons.insert(code, owned);
                }
                if self.buttons.get(&code) == Some(&true) {
                    self.fake(
                        if down { BUTTON_PRESS_EVENT } else { BUTTON_RELEASE_EVENT },
                        code,
                        NONE,
                        0,
                        0,
                    )?;
                }
                if !down {
                    self.buttons.remove(&code);
                }
            }
            InputEvent::MouseMoveAbs { x, y } => self.fake(MOTION_NOTIFY_EVENT, 0, self.root, clamp(x), clamp(y))?,
            InputEvent::MouseMoveRel { dx, dy } => self.fake(MOTION_NOTIFY_EVENT, 1, NONE, clamp(dx), clamp(dy))?,
            InputEvent::MouseWheel { dx, dy } => {
                for (axis, delta) in [dx, dy].into_iter().enumerate() {
                    self.wheel[axis] += i64::from(delta);
                    while self.wheel[axis].abs() >= 120 {
                        let positive = self.wheel[axis] > 0;
                        let code = match (axis, positive) {
                            (0, true) => 7,
                            (0, false) => 6,
                            (_, true) => 4,
                            _ => 5,
                        };
                        self.fake(BUTTON_PRESS_EVENT, code, NONE, 0, 0)?;
                        self.fake(BUTTON_RELEASE_EVENT, code, NONE, 0, 0)?;
                        self.wheel[axis] += if positive { -120 } else { 120 };
                    }
                }
            }
        }
        self.c.flush().map_err(error)
    }
    fn release_all(&mut self) {
        for button in self.keys.keys().copied().collect::<Vec<_>>() {
            if let Err(e) = self.up(button) {
                log::error!("X11 key release failed: {e}");
            }
        }
        for (button, owned) in self.buttons.clone() {
            if !owned || self.fake(BUTTON_RELEASE_EVENT, button, NONE, 0, 0).is_ok() {
                self.buttons.remove(&button);
            }
        }
        for code in self.pending_toggles.clone() {
            if self.key(code, false).is_ok() {
                self.pending_toggles.remove(&code);
            }
        }
        self.wheel = [0; 2];
        let _ = self.c.flush();
    }
    fn set_cursor_visible(&mut self, visible: bool) {
        if self.hidden == !visible {
            return;
        }
        let result = if visible {
            self.c.xfixes_show_cursor(self.root)
        } else {
            self.c.xfixes_hide_cursor(self.root)
        };
        match result.map_err(error).and_then(|c| c.check().map_err(error)) {
            Ok(()) => self.hidden = !visible,
            Err(e) => log::warn!("X11 cursor visibility failed: {e}"),
        }
    }
}
impl Drop for Sink {
    fn drop(&mut self) {
        self.release_all();
        self.set_cursor_visible(true);
    }
}
fn clamp(x: i32) -> i16 {
    x.clamp(i16::MIN.into(), i16::MAX.into()) as i16
}
