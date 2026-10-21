//! IPC adapters. Frozen InputEvent keeps its original tagged serde shape.
use crate::{InputEvent, Modifiers};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Accepts both frozen tagged events and compact scenario objects.
/// Serializes to the frozen tagged shape, so recorded events stay compatible.
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptEvent(pub InputEvent);
impl Serialize for ScriptEvent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ScriptEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.get("type").is_some() {
            return serde_json::from_value(value).map(Self).map_err(de::Error::custom);
        }
        let object = value
            .as_object()
            .filter(|object| object.len() == 1)
            .cloned()
            .ok_or_else(|| de::Error::custom("event must be a tagged event or a single compact operation"))?;
        let (kind, value) = object
            .into_iter()
            .next()
            .ok_or_else(|| de::Error::custom("empty event"))?;
        let parsed = match kind.as_str() {
            "move_abs" | "move_rel" | "wheel" => {
                let [a, b]: [i32; 2] = serde_json::from_value(value).map_err(de::Error::custom)?;
                match kind.as_str() {
                    "move_abs" => InputEvent::MouseMoveAbs { x: a, y: b },
                    "move_rel" => InputEvent::MouseMoveRel { dx: a, dy: b },
                    _ => InputEvent::MouseWheel { dx: a, dy: b },
                }
            }
            "mouse_down" | "mouse_up" => {
                let button = serde_json::from_value(value).map_err(de::Error::custom)?;
                if !(1..=7).contains(&button) {
                    return Err(de::Error::custom("mouse button must be in 1..=7"));
                }
                if kind == "mouse_down" {
                    InputEvent::MouseDown { button }
                } else {
                    InputEvent::MouseUp { button }
                }
            }
            "key_down" | "key_up" | "key_repeat" => {
                let spec: KeySpec = serde_json::from_value(value).map_err(de::Error::custom)?;
                let (key, modifiers, button, count) = spec.parts().map_err(de::Error::custom)?;
                if kind != "key_repeat" && count.is_some() {
                    return Err(de::Error::custom("count is only valid on key_repeat"));
                }
                match kind.as_str() {
                    "key_down" => InputEvent::KeyDown { key, modifiers, button },
                    "key_up" => InputEvent::KeyUp { key, modifiers, button },
                    _ => InputEvent::KeyRepeat {
                        key,
                        modifiers,
                        button,
                        count: count.unwrap_or(1),
                    },
                }
            }
            _ => return Err(de::Error::custom("unknown compact input operation")),
        };
        Ok(Self(parsed))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InjectRequest {
    pub events: Vec<ScriptEvent>,
}
impl InjectRequest {
    pub fn into_events(self) -> Vec<InputEvent> {
        self.events.into_iter().map(|event| event.0).collect()
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum KeyId {
    Number(u16),
    Name(String),
}
impl KeyId {
    fn resolve(self) -> Result<u16, &'static str> {
        match self {
            Self::Number(number) => Ok(number),
            Self::Name(name) => {
                if let Some(hex) = name.strip_prefix("0x") {
                    return u16::from_str_radix(hex, 16).map_err(|_| "invalid hexadecimal KeyID");
                }
                let mut characters = name.chars();
                if let (Some(character), None) = (characters.next(), characters.next()) {
                    return u16::try_from(u32::from(character)).map_err(|_| "KeyID must fit in 16 bits");
                }
                KEY_NAMES
                    .iter()
                    .find_map(|(key_name, id)| (*key_name == name).then_some(*id))
                    .ok_or("unknown KeyID name; use a name from protocol Appendix A or a u16")
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum KeySpec {
    Simple(KeyId),
    Detailed(KeyDetails),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyDetails {
    key: KeyId,
    #[serde(default)]
    modifiers: ModifierSpec,
    button: Option<u16>,
    count: Option<i16>,
}
impl KeySpec {
    fn parts(self) -> Result<(u16, Modifiers, u16, Option<i16>), &'static str> {
        match self {
            Self::Simple(key) => {
                let key = key.resolve()?;
                Ok((key, Modifiers::NONE, key, None))
            }
            Self::Detailed(details) => {
                let key = details.key.resolve()?;
                if details.count.is_some_and(|count| count <= 0) {
                    return Err("repeat count must be positive");
                }
                Ok((
                    key,
                    details.modifiers.resolve()?,
                    details.button.unwrap_or(key),
                    details.count,
                ))
            }
        }
    }
}
#[derive(Deserialize)]
#[serde(untagged)]
enum ModifierSpec {
    Mask(u16),
    Names(Vec<String>),
}
impl Default for ModifierSpec {
    fn default() -> Self {
        Self::Mask(0)
    }
}
impl ModifierSpec {
    fn resolve(self) -> Result<Modifiers, &'static str> {
        match self {
            Self::Mask(mask) => Ok(Modifiers(mask)),
            Self::Names(names) => {
                let mut mask = 0;
                for name in names {
                    mask |= match name.as_str() {
                        "shift" => Modifiers::SHIFT,
                        "control" => Modifiers::CONTROL,
                        "alt" => Modifiers::ALT,
                        "meta" => Modifiers::META,
                        "super" => Modifiers::SUPER,
                        "altgr" => Modifiers::ALTGR,
                        "caps_lock" => Modifiers::CAPS_LOCK,
                        "num_lock" => Modifiers::NUM_LOCK,
                        "scroll_lock" => Modifiers::SCROLL_LOCK,
                        _ => return Err("unknown modifier name"),
                    };
                }
                Ok(Modifiers(mask))
            }
        }
    }
}

// Interface facts from docs/re/03-core-protocol.md Appendix A, not upstream code.
const KEY_NAMES: &[(&str, u16)] = &[
    ("AltGr", 0xEF7E),
    ("Alt_L", 0xEFE9),
    ("Alt_R", 0xEFEA),
    ("AppMail", 0xE0B4),
    ("AppMedia", 0xE0B5),
    ("AppUser1", 0xE0B6),
    ("AppUser2", 0xE0B7),
    ("AudioDown", 0xE0AE),
    ("AudioMute", 0xE0AD),
    ("AudioNext", 0xE0B0),
    ("AudioPlay", 0xE0B3),
    ("AudioPrev", 0xE0B1),
    ("AudioStop", 0xE0B2),
    ("AudioUp", 0xE0AF),
    ("BackSpace", 0xEF08),
    ("Begin", 0xEF58),
    ("Break", 0xEF6B),
    ("Cancel", 0xEF69),
    ("CapsLock", 0xEFE5),
    ("Clear", 0xEF0B),
    ("Control_L", 0xEFE3),
    ("Control_R", 0xEFE4),
    ("Delete", 0xEFFF),
    ("Down", 0xEF54),
    ("Eject", 0xE001),
    ("End", 0xEF57),
    ("Escape", 0xEF1B),
    ("Execute", 0xEF62),
    ("F1", 0xEFBE),
    ("F2", 0xEFBF),
    ("F3", 0xEFC0),
    ("F4", 0xEFC1),
    ("F5", 0xEFC2),
    ("F6", 0xEFC3),
    ("F7", 0xEFC4),
    ("F8", 0xEFC5),
    ("F9", 0xEFC6),
    ("F10", 0xEFC7),
    ("F11", 0xEFC8),
    ("F12", 0xEFC9),
    ("F13", 0xEFCA),
    ("F14", 0xEFCB),
    ("F15", 0xEFCC),
    ("F16", 0xEFCD),
    ("F17", 0xEFCE),
    ("F18", 0xEFCF),
    ("F19", 0xEFD0),
    ("F20", 0xEFD1),
    ("F21", 0xEFD2),
    ("F22", 0xEFD3),
    ("F23", 0xEFD4),
    ("F24", 0xEFD5),
    ("F25", 0xEFD6),
    ("F26", 0xEFD7),
    ("F27", 0xEFD8),
    ("F28", 0xEFD9),
    ("F29", 0xEFDA),
    ("F30", 0xEFDB),
    ("F31", 0xEFDC),
    ("F32", 0xEFDD),
    ("F33", 0xEFDE),
    ("F34", 0xEFDF),
    ("F35", 0xEFE0),
    ("Find", 0xEF68),
    ("Help", 0xEF6A),
    ("Henkan", 0xEF23),
    ("Home", 0xEF50),
    ("Hyper_L", 0xEFED),
    ("Hyper_R", 0xEFEE),
    ("Insert", 0xEF63),
    ("KP_0", 0xEFB0),
    ("KP_1", 0xEFB1),
    ("KP_2", 0xEFB2),
    ("KP_3", 0xEFB3),
    ("KP_4", 0xEFB4),
    ("KP_5", 0xEFB5),
    ("KP_6", 0xEFB6),
    ("KP_7", 0xEFB7),
    ("KP_8", 0xEFB8),
    ("KP_9", 0xEFB9),
    ("KP_Add", 0xEFAB),
    ("KP_Begin", 0xEF9D),
    ("KP_Decimal", 0xEFAE),
    ("KP_Delete", 0xEF9F),
    ("KP_Divide", 0xEFAF),
    ("KP_Down", 0xEF99),
    ("KP_End", 0xEF9C),
    ("KP_Enter", 0xEF8D),
    ("KP_Equal", 0xEFBD),
    ("KP_F1", 0xEF91),
    ("KP_F2", 0xEF92),
    ("KP_F3", 0xEF93),
    ("KP_F4", 0xEF94),
    ("KP_Home", 0xEF95),
    ("KP_Insert", 0xEF9E),
    ("KP_Left", 0xEF96),
    ("KP_Multiply", 0xEFAA),
    ("KP_PageDown", 0xEF9B),
    ("KP_PageUp", 0xEF9A),
    ("KP_Right", 0xEF98),
    ("KP_Separator", 0xEFAC),
    ("KP_Space", 0xEF80),
    ("KP_Subtract", 0xEFAD),
    ("KP_Tab", 0xEF89),
    ("KP_Up", 0xEF97),
    ("Left", 0xEF51),
    ("LeftTab", 0xEE20),
    ("Linefeed", 0xEF0A),
    ("Menu", 0xEF67),
    ("Meta_L", 0xEFE7),
    ("Meta_R", 0xEFE8),
    ("NumLock", 0xEF7F),
    ("PageDown", 0xEF56),
    ("PageUp", 0xEF55),
    ("Pause", 0xEF13),
    ("Print", 0xEF61),
    ("Redo", 0xEF66),
    ("Return", 0xEF0D),
    ("Right", 0xEF53),
    ("ScrollLock", 0xEF14),
    ("Select", 0xEF60),
    ("ShiftLock", 0xEFE6),
    ("Shift_L", 0xEFE1),
    ("Shift_R", 0xEFE2),
    ("Sleep", 0xE05F),
    ("Super_L", 0xEFEB),
    ("Super_R", 0xEFEC),
    ("SysReq", 0xEF15),
    ("Tab", 0xEF09),
    ("Undo", 0xEF65),
    ("Up", 0xEF52),
    ("WWWBack", 0xE0A6),
    ("WWWFavorites", 0xE0AB),
    ("WWWForward", 0xE0A7),
    ("WWWHome", 0xE0AC),
    ("WWWRefresh", 0xE0A8),
    ("WWWSearch", 0xE0AA),
    ("WWWStop", 0xE0A9),
    ("Zenkaku", 0xEF2A),
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
    ("Asterisk", 0x002A),
    ("Plus", 0x002B),
    ("Comma", 0x002C),
    ("Minus", 0x002D),
    ("Period", 0x002E),
    ("Slash", 0x002F),
    ("Colon", 0x003A),
    ("Semicolon", 0x003B),
    ("Less", 0x003C),
    ("Equal", 0x003D),
    ("Greater", 0x003E),
    ("Question", 0x003F),
    ("At", 0x0040),
    ("BracketL", 0x005B),
    ("Backslash", 0x005C),
    ("BracketR", 0x005D),
    ("Circumflex", 0x005E),
    ("Underscore", 0x005F),
    ("Grave", 0x0060),
    ("BraceL", 0x007B),
    ("Bar", 0x007C),
    ("BraceR", 0x007D),
    ("Tilde", 0x007E),
];
