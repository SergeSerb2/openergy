//! Original translation of the public KeyID constants in 03-core-protocol §7/App A.
use crate::native::error;
use openergy_platform::{Modifiers, PlatformError};
use x11rb::{connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection};

const MEDIA: &[(u16, u32)] = &[
    (0xe001, 0x1008ff2c),
    (0xe05f, 0x1008ff2f),
    (0xe0a6, 0x1008ff26),
    (0xe0a7, 0x1008ff27),
    (0xe0a8, 0x1008ff29),
    (0xe0a9, 0x1008ff28),
    (0xe0aa, 0x1008ff1b),
    (0xe0ab, 0x1008ff30),
    (0xe0ac, 0x1008ff18),
    (0xe0ad, 0x1008ff12),
    (0xe0ae, 0x1008ff11),
    (0xe0af, 0x1008ff13),
    (0xe0b0, 0x1008ff17),
    (0xe0b1, 0x1008ff16),
    (0xe0b2, 0x1008ff15),
    (0xe0b3, 0x1008ff14),
    (0xe0b4, 0x1008ff19),
    (0xe0b5, 0x1008ff32),
    (0xe0b6, 0x1008ff40),
    (0xe0b7, 0x1008ff41),
];
pub fn keysym(key: u16) -> u32 {
    match key {
        0xef00..=0xefff => u32::from(key) + 0x1000,
        0xee20 => 0xfe20,
        _ => MEDIA
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, s)| *s)
            .unwrap_or_else(|| {
                if key <= 0xff {
                    u32::from(key)
                } else {
                    0x01000000 | u32::from(key)
                }
            }),
    }
}
pub fn keyid(sym: u32) -> Option<u16> {
    match sym {
        0xff00..=0xffff => Some((sym - 0x1000) as u16),
        0xfe20 => Some(0xee20),
        0xfe03 => Some(0xef7e),
        0x20..=0xff => Some(sym as u16),
        0x01000100..=0x0100ffff => Some(sym as u16),
        _ => MEDIA.iter().find(|(_, s)| *s == sym).map(|(k, _)| *k).or_else(|| {
            crate::unicode_keysyms::LEGACY
                .binary_search_by_key(&sym, |(s, _)| *s)
                .ok()
                .map(|i| crate::unicode_keysyms::LEGACY[i].1)
        }),
    }
}
#[derive(Clone)]
pub struct Keymap {
    pub first: u8,
    pub width: usize,
    pub symbols: Vec<u32>,
    pub masks: [u16; 8],
    pub modifier_codes: Vec<u8>,
}
impl Keymap {
    pub fn load(c: &RustConnection) -> Result<Self, PlatformError> {
        let setup = c.setup();
        let reply = c
            .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)
            .map_err(error)?
            .reply()
            .map_err(error)?;
        let mods = c.get_modifier_mapping().map_err(error)?.reply().map_err(error)?;
        let width = usize::from(reply.keysyms_per_keycode);
        if width == 0 {
            return Err(error("X server returned an empty keymap"));
        }
        let mut map = Self {
            first: setup.min_keycode,
            width,
            symbols: reply.keysyms,
            masks: [0; 8],
            modifier_codes: mods.keycodes.clone(),
        };
        for (index, codes) in mods
            .keycodes
            .chunks(usize::from(mods.keycodes_per_modifier()).max(1))
            .enumerate()
        {
            for code in codes.iter().filter(|code| **code != 0) {
                for sym in map.row(*code).to_vec() {
                    map.masks[index] |= match sym {
                        0xffe1 | 0xffe2 | 0xffe6 => Modifiers::SHIFT,
                        0xffe3 | 0xffe4 => Modifiers::CONTROL,
                        0xffe9 | 0xffea => Modifiers::ALT,
                        0xffe7 | 0xffe8 => Modifiers::META,
                        0xffeb..=0xffee => Modifiers::SUPER,
                        0xff7e | 0xfe03 => Modifiers::ALTGR,
                        0xffe5 => Modifiers::CAPS_LOCK,
                        0xff7f => Modifiers::NUM_LOCK,
                        0xff14 => Modifiers::SCROLL_LOCK,
                        _ => 0,
                    };
                }
            }
        }
        Ok(map)
    }
    pub fn row(&self, code: u8) -> &[u32] {
        let start = usize::from(code.saturating_sub(self.first)) * self.width;
        if code < self.first {
            return &[];
        }
        self.symbols.get(start..start + self.width).unwrap_or(&[])
    }
    pub fn modifiers(&self, state: u16) -> Modifiers {
        Modifiers(
            self.masks
                .iter()
                .enumerate()
                .filter(|(i, _)| state & (1 << i) != 0)
                .fold(0, |m, (_, value)| m | value),
        )
    }
    pub fn logical(&self, code: u8, state: u16) -> Option<u16> {
        let row = self.row(code);
        let mask = self.modifiers(state).0;
        let shift = mask & Modifiers::SHIFT != 0;
        let group = if mask & Modifiers::ALTGR != 0 {
            if row.get(4).is_some_and(|s| *s != 0) {
                4
            } else if row.get(2).is_some_and(|s| *s != 0) {
                2
            } else {
                0
            }
        } else {
            0
        };
        let a = *row.get(group)?;
        let b = row.get(group + 1).copied().filter(|s| *s != 0).unwrap_or_else(|| {
            if (0x61..=0x7a).contains(&a) {
                a - 32
            } else {
                a
            }
        });
        let caps = mask & Modifiers::CAPS_LOCK != 0
            && keyid(a)
                .and_then(|key| char::from_u32(u32::from(key)))
                .is_some_and(char::is_alphabetic);
        let keypad_num = mask & Modifiers::NUM_LOCK != 0
            && (0xff95..=0xff9f).contains(&a)
            && ((0xffb0..=0xffb9).contains(&b) || b == 0xffae);
        keyid(if shift ^ caps ^ keypad_num { b } else { a })
    }
    pub fn find(&self, sym: u32) -> Option<(u8, bool, bool)> {
        self.symbols.chunks(self.width).enumerate().find_map(|(i, row)| {
            // Core XKB compatibility mapping: levels 3/4 of group 1 occupy columns 4/5.
            [0usize, 1, 4, 5]
                .into_iter()
                .find(|column| {
                    row.get(*column)
                        .is_some_and(|s| *s == sym || (keyid(*s).is_some() && keyid(*s) == keyid(sym)))
                })
                .map(|column| (self.first + i as u8, column & 1 != 0, column >= 4))
        })
    }
    pub fn modifier_code(&self, mask: u16) -> Option<u8> {
        let sym = match mask {
            Modifiers::SHIFT => 0xffe1,
            Modifiers::CONTROL => 0xffe3,
            Modifiers::ALT => 0xffe9,
            Modifiers::SUPER => 0xffeb,
            Modifiers::META => 0xffe7,
            Modifiers::ALTGR => 0xfe03,
            Modifiers::CAPS_LOCK => 0xffe5,
            Modifiers::NUM_LOCK => 0xff7f,
            Modifiers::SCROLL_LOCK => 0xff14,
            _ => return None,
        };
        self.find(sym).map(|(code, _, _)| code)
    }
}
