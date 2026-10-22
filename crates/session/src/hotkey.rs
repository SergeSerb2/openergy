//! Lock binding grammar shared with the configured Synergy hotkey syntax.
use openergy_platform::InputEvent;

#[derive(Clone, Copy)]
enum Trigger {
    Key(u16),
    Mouse(u8),
}
#[derive(Clone, Copy)]
pub(crate) struct Hotkey {
    modifiers: u16,
    trigger: Trigger,
}
impl Hotkey {
    pub fn parse(stroke: &str) -> Option<Self> {
        let (mouse, stroke) = if let Some(inner) = stroke.strip_prefix("mousebutton(") {
            (true, inner.strip_suffix(')')?)
        } else {
            (false, stroke)
        };
        let (modifiers, key) = stroke.rsplit_once('+').unwrap_or(("", stroke));
        let mut mask = 0;
        for part in modifiers.split('+').filter(|part| !part.is_empty()) {
            mask |= match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => 2,
                "shift" => 1,
                "alt" => 4,
                "meta" => 8,
                "super" | "command" | "cmd" => 16,
                "altgr" => 32,
                _ => return None,
            };
        }
        let trigger = if mouse {
            Trigger::Mouse(key.parse::<u8>().ok().filter(|button| *button != 0)?)
        } else {
            Trigger::Key(key_id(key)?)
        };
        Some(Self {
            modifiers: mask,
            trigger,
        })
    }
    pub fn matches(self, event: &InputEvent, local_modifiers: u16) -> bool {
        match (self.trigger, event) {
            (Trigger::Key(expected), InputEvent::KeyDown { key, modifiers, .. }) => {
                let own_modifier = match expected {
                    0xefe1 | 0xefe2 => 1,
                    0xefe3 | 0xefe4 => 2,
                    0xefe9 | 0xefea => 4,
                    0xefe7 | 0xefe8 => 8,
                    0xefeb | 0xefec => 16,
                    0xef7e => 32,
                    _ => 0,
                };
                normalized_key(expected) == normalized_key(*key)
                    && modifiers.0 & 0x3f & !own_modifier == self.modifiers & !own_modifier
            }
            (Trigger::Mouse(expected), InputEvent::MouseDown { button }) => {
                expected == *button && local_modifiers & 0x3f == self.modifiers
            }
            _ => false,
        }
    }
}
fn normalized_key(key: u16) -> u16 {
    if key <= u8::MAX as u16 {
        u16::from((key as u8).to_ascii_lowercase())
    } else {
        key
    }
}
fn key_id(name: &str) -> Option<u16> {
    let mut chars = name.chars();
    if let (Some(key), None) = (chars.next(), chars.next()) {
        return u16::try_from(key as u32).ok();
    }
    if let Some(hex) = name.strip_prefix("\\u").filter(|hex| hex.len() == 4) {
        return u16::from_str_radix(hex, 16).ok();
    }
    KEY_NAMES
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, id)| *id)
}

// Interface constants documented in docs/re/03-core-protocol.md, Appendix A.
// Keep every key name accepted by config::canonical_key mapped here.
const KEY_NAMES: &[(&str, u16)] = &[
    ("AltGr", 0xef7e),
    ("Alt_L", 0xefe9),
    ("Alt_R", 0xefea),
    ("AppMail", 0xe0b4),
    ("AppMedia", 0xe0b5),
    ("AppUser1", 0xe0b6),
    ("AppUser2", 0xe0b7),
    ("AudioDown", 0xe0ae),
    ("AudioMute", 0xe0ad),
    ("AudioNext", 0xe0b0),
    ("AudioPlay", 0xe0b3),
    ("AudioPrev", 0xe0b1),
    ("AudioStop", 0xe0b2),
    ("AudioUp", 0xe0af),
    ("BackSpace", 0xef08),
    ("Begin", 0xef58),
    ("Break", 0xef6b),
    ("Cancel", 0xef69),
    ("CapsLock", 0xefe5),
    ("Clear", 0xef0b),
    ("Control_L", 0xefe3),
    ("Control_R", 0xefe4),
    ("Delete", 0xefff),
    ("Down", 0xef54),
    ("Eject", 0xe001),
    ("End", 0xef57),
    ("Escape", 0xef1b),
    ("Execute", 0xef62),
    ("F1", 0xefbe),
    ("F2", 0xefbf),
    ("F3", 0xefc0),
    ("F4", 0xefc1),
    ("F5", 0xefc2),
    ("F6", 0xefc3),
    ("F7", 0xefc4),
    ("F8", 0xefc5),
    ("F9", 0xefc6),
    ("F10", 0xefc7),
    ("F11", 0xefc8),
    ("F12", 0xefc9),
    ("F13", 0xefca),
    ("F14", 0xefcb),
    ("F15", 0xefcc),
    ("F16", 0xefcd),
    ("F17", 0xefce),
    ("F18", 0xefcf),
    ("F19", 0xefd0),
    ("F20", 0xefd1),
    ("F21", 0xefd2),
    ("F22", 0xefd3),
    ("F23", 0xefd4),
    ("F24", 0xefd5),
    ("F25", 0xefd6),
    ("F26", 0xefd7),
    ("F27", 0xefd8),
    ("F28", 0xefd9),
    ("F29", 0xefda),
    ("F30", 0xefdb),
    ("F31", 0xefdc),
    ("F32", 0xefdd),
    ("F33", 0xefde),
    ("F34", 0xefdf),
    ("F35", 0xefe0),
    ("Find", 0xef68),
    ("Help", 0xef6a),
    ("Henkan", 0xef23),
    ("Home", 0xef50),
    ("Hyper_L", 0xefed),
    ("Hyper_R", 0xefee),
    ("Insert", 0xef63),
    ("KP_0", 0xefb0),
    ("KP_1", 0xefb1),
    ("KP_2", 0xefb2),
    ("KP_3", 0xefb3),
    ("KP_4", 0xefb4),
    ("KP_5", 0xefb5),
    ("KP_6", 0xefb6),
    ("KP_7", 0xefb7),
    ("KP_8", 0xefb8),
    ("KP_9", 0xefb9),
    ("KP_Add", 0xefab),
    ("KP_Begin", 0xef9d),
    ("KP_Decimal", 0xefae),
    ("KP_Delete", 0xef9f),
    ("KP_Divide", 0xefaf),
    ("KP_Down", 0xef99),
    ("KP_End", 0xef9c),
    ("KP_Enter", 0xef8d),
    ("KP_Equal", 0xefbd),
    ("KP_F1", 0xef91),
    ("KP_F2", 0xef92),
    ("KP_F3", 0xef93),
    ("KP_F4", 0xef94),
    ("KP_Home", 0xef95),
    ("KP_Insert", 0xef9e),
    ("KP_Left", 0xef96),
    ("KP_Multiply", 0xefaa),
    ("KP_PageDown", 0xef9b),
    ("KP_PageUp", 0xef9a),
    ("KP_Right", 0xef98),
    ("KP_Separator", 0xefac),
    ("KP_Space", 0xef80),
    ("KP_Subtract", 0xefad),
    ("KP_Tab", 0xef89),
    ("KP_Up", 0xef97),
    ("Left", 0xef51),
    ("LeftTab", 0xee20),
    ("Linefeed", 0xef0a),
    ("Menu", 0xef67),
    ("Meta_L", 0xefe7),
    ("Meta_R", 0xefe8),
    ("NumLock", 0xef7f),
    ("PageDown", 0xef56),
    ("PageUp", 0xef55),
    ("Pause", 0xef13),
    ("Print", 0xef61),
    ("Redo", 0xef66),
    ("Return", 0xef0d),
    ("Right", 0xef53),
    ("ScrollLock", 0xef14),
    ("Select", 0xef60),
    ("ShiftLock", 0xefe6),
    ("Shift_L", 0xefe1),
    ("Shift_R", 0xefe2),
    ("Sleep", 0xe05f),
    ("Super_L", 0xefeb),
    ("Super_R", 0xefec),
    ("SysReq", 0xef15),
    ("Tab", 0xef09),
    ("Undo", 0xef65),
    ("Up", 0xef52),
    ("WWWBack", 0xe0a6),
    ("WWWFavorites", 0xe0ab),
    ("WWWForward", 0xe0a7),
    ("WWWHome", 0xe0ac),
    ("WWWRefresh", 0xe0a8),
    ("WWWSearch", 0xe0aa),
    ("WWWStop", 0xe0a9),
    ("Zenkaku", 0xef2a),
    ("Space", 0x0020),
    ("Exclaim", 0x0021),
    ("DoubleQuote", 0x0022),
    ("Number", 0x0023),
    ("Dollar", 0x0024),
    ("Percent", 0x0025),
    ("Ampersand", 0x0026),
    ("Apostrophe", 0x0027),
    ("ParenthesisL", 0x0028),
    ("ParenthesisR", 0x0029),
    ("Asterisk", 0x002a),
    ("Plus", 0x002b),
    ("Comma", 0x002c),
    ("Minus", 0x002d),
    ("Period", 0x002e),
    ("Slash", 0x002f),
    ("Colon", 0x003a),
    ("Semicolon", 0x003b),
    ("Less", 0x003c),
    ("Equal", 0x003d),
    ("Greater", 0x003e),
    ("Question", 0x003f),
    ("At", 0x0040),
    ("BracketL", 0x005b),
    ("Backslash", 0x005c),
    ("BracketR", 0x005d),
    ("Circumflex", 0x005e),
    ("Underscore", 0x005f),
    ("Grave", 0x0060),
    ("BraceL", 0x007b),
    ("Bar", 0x007c),
    ("BraceR", 0x007d),
    ("Tilde", 0x007e),
];
