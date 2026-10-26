//! Logical wire keys translated through the current TIS Unicode layout.
use openergy_protocol::keys;
use std::collections::HashMap;

use crate::ffi::{self, OwnedCf};
use crate::types::*;
use crate::{error, Result};

pub const CG_CAPS: u64 = 0x0001_0000;
pub const CG_SHIFT: u64 = 0x0002_0000;
pub const CG_CONTROL: u64 = 0x0004_0000;
pub const CG_OPTION: u64 = 0x0008_0000;
pub const CG_COMMAND: u64 = 0x0010_0000;
pub const CG_LAYOUT_FLAGS: u64 = CG_SHIFT | CG_OPTION | CG_CAPS;

pub fn flags_to_mask(flags: u64) -> ModifierMask {
    let mut mask = 0;
    for (cg, wire) in [
        (CG_SHIFT, SHIFT),
        (CG_CONTROL, CONTROL),
        (CG_OPTION, ALT),
        (CG_COMMAND, SUPER),
        (CG_CAPS, CAPS_LOCK),
    ] {
        if flags & cg != 0 {
            mask |= wire;
        }
    }
    // NumericPad marks a key's location, not the NumLock toggle state.
    mask
}

pub fn mask_to_flags(mask: ModifierMask) -> u64 {
    let mut flags = 0;
    for (wire, cg) in [
        (SHIFT, CG_SHIFT),
        (CONTROL, CG_CONTROL),
        (ALT | ALT_GR, CG_OPTION),
        (META | SUPER, CG_COMMAND),
        (CAPS_LOCK, CG_CAPS),
    ] {
        if mask & wire != 0 {
            flags |= cg;
        }
    }
    flags
}

/// docs/re/03-core-protocol.md Appendix B, including intentional left/right modifier aliases.
pub const SPECIAL_KEYS: &[(KeyId, u16)] = &[
    (keys::LEFT, 0x7B),
    (keys::RIGHT, 0x7C),
    (keys::UP, 0x7E),
    (keys::DOWN, 0x7D),
    (keys::HOME, 0x73),
    (keys::END, 0x77),
    (keys::PAGE_UP, 0x74),
    (keys::PAGE_DOWN, 0x79),
    (keys::INSERT, 0x72),
    (keys::F1, 0x7A),
    (keys::F2, 0x78),
    (keys::F3, 0x63),
    (keys::F4, 0x76),
    (keys::F5, 0x60),
    (keys::F6, 0x61),
    (keys::F7, 0x62),
    (keys::F8, 0x64),
    (keys::F9, 0x65),
    (keys::F10, 0x6D),
    (keys::F11, 0x67),
    (keys::F12, 0x6F),
    (keys::F13, 0x69),
    (keys::F14, 0x6B),
    (keys::F15, 0x71),
    (keys::F16, 0x6A),
    (keys::KP_0, 0x52),
    (keys::KP_1, 0x53),
    (keys::KP_2, 0x54),
    (keys::KP_3, 0x55),
    (keys::KP_4, 0x56),
    (keys::KP_5, 0x57),
    (keys::KP_6, 0x58),
    (keys::KP_7, 0x59),
    (keys::KP_8, 0x5B),
    (keys::KP_9, 0x5C),
    (keys::KP_DECIMAL, 0x41),
    (keys::KP_EQUAL, 0x51),
    (keys::KP_MULTIPLY, 0x43),
    (keys::KP_ADD, 0x45),
    (keys::KP_DIVIDE, 0x4B),
    (keys::KP_SUBTRACT, 0x4E),
    (keys::KP_ENTER, 0x4C),
    (keys::SHIFT_L, 0x38),
    (keys::SHIFT_R, 0x38),
    (keys::CONTROL_L, 0x3B),
    (keys::CONTROL_R, 0x3B),
    (keys::ALT_L, 0x3A),
    (keys::ALT_R, 0x3A),
    (keys::SUPER_L, 0x37),
    (keys::SUPER_R, 0x37),
    (keys::META_L, 0x37),
    (keys::META_R, 0x37),
    (keys::NUM_LOCK, 0x47),
    (keys::CAPS_LOCK, 0x39),
    (keys::HENKAN, 0x68),
    (keys::ZENKAKU, 0x66),
    // Unnamed Appendix B IDs have no matching public protocol::keys constant.
    (0xE0C0, 0xA0),
    (0xE0C1, 0x83),
    (0xE0B9, 0x90),
    (0xE0B8, 0x91),
];

const EXTRA_KEYS: &[(KeyId, u16)] = &[
    (keys::RETURN, 0x24),
    (keys::TAB, 0x30),
    (keys::BACK_SPACE, 0x33),
    (keys::ESCAPE, 0x35),
    (keys::DELETE, 0x75),
    // Public Events.h virtual keycodes missing from the report's static table.
    (keys::F17, 0x40),
    (keys::F18, 0x4F),
    (keys::F19, 0x50),
    (keys::F20, 0x5A),
    (keys::AUDIO_UP, 0x48),
    (keys::AUDIO_DOWN, 0x49),
    (keys::AUDIO_MUTE, 0x4A),
];

pub fn special_to_code(key: KeyId) -> Option<u16> {
    SPECIAL_KEYS
        .iter()
        .chain(EXTRA_KEYS)
        .find(|entry| entry.0 == key)
        .map(|entry| entry.1)
}

pub fn code_to_special(code: u16) -> Option<KeyId> {
    // Capture physical right modifiers faithfully; injection preserves the
    // report's canonical aliases above. Command always captures as Super.
    match code {
        0x3C => Some(keys::SHIFT_R),
        0x3E => Some(keys::CONTROL_R),
        0x3D => Some(keys::ALT_R),
        0x36 => Some(keys::SUPER_R),
        _ => SPECIAL_KEYS
            .iter()
            .chain(EXTRA_KEYS)
            .find(|entry| entry.1 == code)
            .map(|entry| entry.0),
    }
}

/// Physical modifier's aggregate CG flag and the public IOLLEvent device bit.
pub fn modifier_for_code(code: u16) -> Option<(u64, u64)> {
    match code {
        0x38 => Some((CG_SHIFT, 0x02)),
        0x3C => Some((CG_SHIFT, 0x04)),
        0x3B => Some((CG_CONTROL, 0x01)),
        0x3E => Some((CG_CONTROL, 0x2000)),
        0x3A => Some((CG_OPTION, 0x20)),
        0x3D => Some((CG_OPTION, 0x40)),
        0x37 => Some((CG_COMMAND, 0x08)),
        0x36 => Some((CG_COMMAND, 0x10)),
        0x39 => Some((CG_CAPS, 0)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapturedKey {
    pub key: KeyId,
    pub mask: ModifierMask,
    pub button: KeyButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyStroke {
    pub keycode: u16,
    pub flags: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionPlan {
    /// Earlier strokes are complete down/up dead-key prefixes. The final
    /// stroke remains down until the matching source button is released.
    Keycodes(Vec<KeyStroke>),
    Unicode {
        utf16: Vec<u16>,
        flags: u64,
    },
}

struct Layout {
    id: String,
    data: OwnedCf,
    keyboard_type: u32,
}
impl Layout {
    fn current() -> Result<Self> {
        // Safety: retained CF layout/string references and checked buffers outlive the public Carbon/CF calls.
        unsafe {
            let source = OwnedCf::new(ffi::TISCopyCurrentKeyboardLayoutInputSource(), "TIS current layout")?;
            let id = ffi::cf_string(ffi::TISGetInputSourceProperty(source.0, ffi::kTISPropertyInputSourceID))?;
            let data = ffi::TISGetInputSourceProperty(source.0, ffi::kTISPropertyUnicodeKeyLayoutData);
            if data.is_null() || ffi::CFDataGetLength(data) == 0 {
                return Err(error(format!("layout {id} has no Unicode keyboard-layout data")));
            }
            Ok(Self {
                id,
                data: OwnedCf(ffi::CFRetain(data)),
                // Unlike legacy LMGetKbdType (documented not thread-safe),
                // this public Quartz getter is suitable on the tap thread.
                keyboard_type: ffi::CGEventSourceGetKeyboardType(ffi::null()),
            })
        }
    }

    fn translate(&self, code: u16, flags: u64, options: u32, state: &mut u32) -> Result<Vec<u16>> {
        // Carbon modifiers >> 8: shift=2, caps=4, option=8. Neither Command
        // nor Control is ever passed to UCKeyTranslate, including shortcuts.
        let carbon = (((flags & CG_SHIFT != 0) as u32) * 2)
            | (((flags & CG_CAPS != 0) as u32) * 4)
            | (((flags & CG_OPTION != 0) as u32) * 8);
        let mut output = [0u16; 255];
        let mut length = 0usize; // UniCharCount is unsigned long on LP64.
                                 // Safety: retained layout data and fixed output buffer remain live; lengths match UCKeyTranslate ABI.
        let status = unsafe {
            ffi::UCKeyTranslate(
                ffi::CFDataGetBytePtr(self.data.0).cast(),
                code,
                0,
                carbon,
                self.keyboard_type,
                options,
                state,
                output.len(),
                &mut length,
                output.as_mut_ptr(),
            )
        };
        if status != 0 {
            return Err(error(format!("UCKeyTranslate failed: {status}")));
        }
        Ok(output[..length].to_vec())
    }
}

pub struct Keymap {
    layout: Layout,
    reverse: HashMap<KeyId, Vec<Vec<KeyStroke>>>,
    capture_table: [[Option<KeyId>; 256]; 8],
}

impl Keymap {
    pub fn new() -> Result<Self> {
        let layout = Layout::current()?;
        let mut value = Self {
            layout,
            reverse: HashMap::new(),
            capture_table: [[None; 256]; 8],
        };
        value.rebuild()?;
        Ok(value)
    }

    pub fn layout_id(&self) -> &str {
        &self.layout.id
    }

    /// Refresh outside the event path after a keyboard-layout notification.
    pub fn refresh(&mut self) -> Result<bool> {
        let current = Layout::current()?;
        if current.id == self.layout.id && current.keyboard_type == self.layout.keyboard_type {
            return Ok(false);
        }
        // Build separately so a bad replacement never leaves a partial cache.
        let mut replacement = Self {
            layout: current,
            reverse: HashMap::new(),
            capture_table: [[None; 256]; 8],
        };
        replacement.rebuild()?;
        *self = replacement;
        Ok(true)
    }

    pub fn capture(&mut self, code: u16, flags: u64) -> Result<Option<CapturedKey>> {
        self.refresh()?;
        Ok(self.capture_cached(code, flags))
    }

    /// Fixed-size lookup only. No TIS, CF, allocation, lock, or layout rebuild in
    /// the event-tap callback. Refresh runs after layout notifications outside it.
    pub fn capture_cached(&self, code: u16, flags: u64) -> Option<CapturedKey> {
        let layer = usize::from(flags & CG_SHIFT != 0)
            | (usize::from(flags & CG_OPTION != 0) << 1)
            | (usize::from(flags & CG_CAPS != 0) << 2);
        self.capture_table[layer]
            .get(code as usize)
            .copied()
            .flatten()
            .map(|key| CapturedKey {
                key,
                mask: flags_to_mask(flags),
                button: code + 1,
            })
    }

    pub fn plan(&mut self, key: KeyId, mask: ModifierMask) -> Result<InjectionPlan> {
        let requested = mask_to_flags(mask);
        if let Some(keycode) = special_to_code(key) {
            return Ok(InjectionPlan::Keycodes(vec![KeyStroke {
                keycode,
                flags: requested,
            }]));
        }
        if (0xE000..=0xEFFF).contains(&key) {
            return Err(error("named key has no public macOS virtual-key mapping"));
        }
        if char::from_u32(u32::from(key)).is_none_or(char::is_control) {
            return Err(error("invalid printable BMP key"));
        }
        if let Some(candidates) = self.reverse.get(&key) {
            let best = candidates
                .iter()
                .min_by_key(|strokes| {
                    let difference: u32 = strokes
                        .iter()
                        .map(|stroke| ((stroke.flags ^ requested) & CG_LAYOUT_FLAGS).count_ones())
                        .sum();
                    (
                        strokes.len(),
                        difference,
                        strokes.iter().map(|stroke| stroke.flags.count_ones()).sum::<u32>(),
                    )
                })
                .ok_or_else(|| error("empty reverse keymap entry"))?;
            return Ok(InjectionPlan::Keycodes(
                best.iter()
                    .map(|stroke| KeyStroke {
                        keycode: stroke.keycode,
                        // Character-producing layers take precedence over remote
                        // Shift/Option/Caps. Control/Command remain shortcut flags.
                        flags: stroke.flags | (requested & !CG_LAYOUT_FLAGS),
                    })
                    .collect(),
            ));
        }
        // Unicode payload already carries the glyph; do not apply a second
        // character-producing Shift/Option/Caps transformation.
        Ok(InjectionPlan::Unicode {
            utf16: vec![key],
            flags: requested & !CG_LAYOUT_FLAGS,
        })
    }

    fn rebuild(&mut self) -> Result<()> {
        for layer in 0..8 {
            let flags = if layer & 1 != 0 { CG_SHIFT } else { 0 }
                | if layer & 2 != 0 { CG_OPTION } else { 0 }
                | if layer & 4 != 0 { CG_CAPS } else { 0 };
            for code in 0..256 {
                let key = if let Some(key) = code_to_special(code) {
                    Some(key)
                } else if code < 128 {
                    let text = self.layout.translate(code, flags, 1, &mut 0)?;
                    printable(&text).or_else(|| {
                        self.layout
                            .translate(code, flags & !CG_OPTION, 1, &mut 0)
                            .ok()
                            .and_then(|text| printable(&text))
                    })
                } else {
                    None
                };
                self.capture_table[layer][code as usize] = key;
            }
        }
        let combos = [0, CG_SHIFT, CG_OPTION, CG_SHIFT | CG_OPTION, CG_CAPS];
        let mut dead = Vec::new();
        for flags in combos {
            for code in 0..128 {
                if code_to_special(code).is_some() {
                    continue;
                }
                let mut state = 0;
                let text = self.layout.translate(code, flags, 0, &mut state)?;
                let stroke = KeyStroke { keycode: code, flags };
                if state != 0 {
                    dead.push((state, stroke));
                } else if let Some(key) = printable(&text) {
                    self.reverse.entry(key).or_default().push(vec![stroke]);
                }
            }
        }
        // A bounded, two-stroke dead-key search supports é/ü on layouts where
        // no single key yields them. Longer compositions use Unicode fallback.
        for (state, prefix) in dead {
            for flags in combos {
                for code in 0..128 {
                    if code_to_special(code).is_some() {
                        continue;
                    }
                    let mut state = state;
                    let text = self.layout.translate(code, flags, 0, &mut state)?;
                    if let (0, Some(key)) = (state, printable(&text)) {
                        // Keep only genuinely composed characters, not the
                        // dead key's default terminator + another character.
                        self.reverse
                            .entry(key)
                            .or_default()
                            .push(vec![prefix, KeyStroke { keycode: code, flags }]);
                    }
                }
            }
        }
        Ok(())
    }
}

fn printable(text: &[u16]) -> Option<KeyId> {
    let key = if text.len() == 1 {
        text[0]
    } else {
        if text.is_empty() {
            return None;
        }
        // Some Unicode layout tables return decomposed dead-key results.
        // NFC can make them one wire-representable BMP character. Longer
        // strings and surrogate pairs still have no 16-bit KeyID.
        // Safety: retained CF layout/string references and checked buffers outlive the public Carbon/CF calls.
        unsafe {
            let string = OwnedCf::new(
                ffi::CFStringCreateMutable(ffi::null(), 0).cast_const(),
                "layout normalization",
            )
            .ok()?;
            ffi::CFStringAppendCharacters(string.0.cast_mut(), text.as_ptr(), text.len() as isize);
            ffi::CFStringNormalize(string.0.cast_mut(), 2); // kCFStringNormalizationFormC
            if ffi::CFStringGetLength(string.0) != 1 {
                return None;
            }
            ffi::CFStringGetCharacterAtIndex(string.0, 0)
        }
    };
    let ch = char::from_u32(u32::from(key))?;
    (!ch.is_control() && !(0xE000..=0xF8FF).contains(&key)).then_some(key)
}
