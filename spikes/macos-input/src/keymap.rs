//! Logical wire keys translated through the current TIS Unicode layout.
use std::collections::HashMap;

use crate::ffi::{self, OwnedCf};
use crate::types::*;
use crate::{Result, error};

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

/// The report's Appendix B, including intentional left/right modifier aliases.
pub const SPECIAL_KEYS: &[(KeyId, u16)] = &[
    (0xEF51, 0x7B),
    (0xEF53, 0x7C),
    (0xEF52, 0x7E),
    (0xEF54, 0x7D),
    (0xEF50, 0x73),
    (0xEF57, 0x77),
    (0xEF55, 0x74),
    (0xEF56, 0x79),
    (0xEF63, 0x72),
    (0xEFBE, 0x7A),
    (0xEFBF, 0x78),
    (0xEFC0, 0x63),
    (0xEFC1, 0x76),
    (0xEFC2, 0x60),
    (0xEFC3, 0x61),
    (0xEFC4, 0x62),
    (0xEFC5, 0x64),
    (0xEFC6, 0x65),
    (0xEFC7, 0x6D),
    (0xEFC8, 0x67),
    (0xEFC9, 0x6F),
    (0xEFCA, 0x69),
    (0xEFCB, 0x6B),
    (0xEFCC, 0x71),
    (0xEFCD, 0x6A),
    (0xEFB0, 0x52),
    (0xEFB1, 0x53),
    (0xEFB2, 0x54),
    (0xEFB3, 0x55),
    (0xEFB4, 0x56),
    (0xEFB5, 0x57),
    (0xEFB6, 0x58),
    (0xEFB7, 0x59),
    (0xEFB8, 0x5B),
    (0xEFB9, 0x5C),
    (0xEFAE, 0x41),
    (0xEFBD, 0x51),
    (0xEFAA, 0x43),
    (0xEFAB, 0x45),
    (0xEFAF, 0x4B),
    (0xEFAD, 0x4E),
    (0xEF8D, 0x4C),
    (0xEFE1, 0x38),
    (0xEFE2, 0x38),
    (0xEFE3, 0x3B),
    (0xEFE4, 0x3B),
    (0xEFE9, 0x3A),
    (0xEFEA, 0x3A),
    (0xEFEB, 0x37),
    (0xEFEC, 0x37),
    (0xEFE7, 0x37),
    (0xEFE8, 0x37),
    (0xEF7F, 0x47),
    (0xEFE5, 0x39),
    (0xEF23, 0x68),
    (0xEF2A, 0x66),
    (0xE0C0, 0xA0),
    (0xE0C1, 0x83),
    (0xE0B9, 0x90),
    (0xE0B8, 0x91),
];

const EXTRA_KEYS: &[(KeyId, u16)] = &[
    (0xEF0D, 0x24),
    (0xEF09, 0x30),
    (0xEF08, 0x33),
    (0xEF1B, 0x35),
    (0xEFFF, 0x75),
    // Public Events.h virtual keycodes missing from the report's static table.
    (0xEFCE, 0x40),
    (0xEFCF, 0x4F),
    (0xEFD0, 0x50),
    (0xEFD1, 0x5A),
    (0xE0AF, 0x48),
    (0xE0AE, 0x49),
    (0xE0AD, 0x4A),
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
        0x3C => Some(0xEFE2),
        0x3E => Some(0xEFE4),
        0x3D => Some(0xEFEA),
        0x36 => Some(0xEFEC),
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
            return Err(error(format!("UCKeyTranslate({code:#x}) failed: {status}")));
        }
        Ok(output[..length].to_vec())
    }
}

pub struct Keymap {
    layout: Layout,
    reverse: HashMap<KeyId, Vec<Vec<KeyStroke>>>,
}

impl Keymap {
    pub fn new() -> Result<Self> {
        let layout = Layout::current()?;
        let mut value = Self {
            layout,
            reverse: HashMap::new(),
        };
        value.rebuild()?;
        Ok(value)
    }

    pub fn layout_id(&self) -> &str {
        &self.layout.id
    }

    /// Compare the current layout ID and keyboard type before each public
    /// translation. This avoids a notification/run-loop lifetime dependency.
    pub fn refresh(&mut self) -> Result<bool> {
        let current = Layout::current()?;
        if current.id == self.layout.id && current.keyboard_type == self.layout.keyboard_type {
            return Ok(false);
        }
        // Build separately so a bad replacement never leaves a partial cache.
        let mut replacement = Self {
            layout: current,
            reverse: HashMap::new(),
        };
        replacement.rebuild()?;
        *self = replacement;
        Ok(true)
    }

    pub fn capture(&mut self, code: u16, flags: u64) -> Result<Option<CapturedKey>> {
        self.refresh()?;
        let key = if let Some(key) = code_to_special(code) {
            Some(key)
        } else {
            // Stateless dead-key rule: request the layout's spacing accent
            // (NoDeadKeys *mask*, 1, not its bit index, 0). If absent, use
            // the same key's base layer. Never compose across capture events.
            let text = self.layout.translate(code, flags, 1, &mut 0)?;
            printable(&text).or_else(|| {
                self.layout
                    .translate(code, flags & !CG_OPTION, 1, &mut 0)
                    .ok()
                    .and_then(|text| printable(&text))
            })
        };
        Ok(key.map(|key| CapturedKey {
            key,
            mask: flags_to_mask(flags),
            button: code + 1,
        }))
    }

    pub fn plan(&mut self, key: KeyId, mask: ModifierMask) -> Result<InjectionPlan> {
        self.refresh()?;
        let requested = mask_to_flags(mask);
        if let Some(keycode) = special_to_code(key) {
            return Ok(InjectionPlan::Keycodes(vec![KeyStroke {
                keycode,
                flags: requested,
            }]));
        }
        if (0xE000..=0xEFFF).contains(&key) {
            return Err(error(format!(
                "named KeyID {key:#06x} has no public macOS virtual-key mapping"
            )));
        }
        if char::from_u32(u32::from(key)).is_none_or(char::is_control) {
            return Err(error(format!("invalid printable BMP KeyID {key:#06x}")));
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
                .expect("reverse entries are nonempty");
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
                } else if state == 0
                    && let Some(key) = printable(&text)
                {
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
                    if state == 0
                        && let Some(key) = printable(&text)
                    {
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
