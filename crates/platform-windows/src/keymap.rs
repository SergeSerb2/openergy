//! Public KeyID constants from 03-core-protocol Appendix A, mapped to Win32 VKs.
use openergy_platform::Modifiers;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
const NAMED: &[(u16, u16)] = &[
    (0xef08, VK_BACK),
    (0xef09, VK_TAB),
    (0xee20, VK_TAB),
    (0xef0a, VK_RETURN),
    (0xef0b, VK_CLEAR),
    (0xef0d, VK_RETURN),
    (0xef13, VK_PAUSE),
    (0xef14, VK_SCROLL),
    (0xef15, VK_SNAPSHOT),
    (0xef1b, VK_ESCAPE),
    (0xef23, VK_CONVERT),
    (0xef2a, VK_DBE_DBCSCHAR),
    (0xef50, VK_HOME),
    (0xef51, VK_LEFT),
    (0xef52, VK_UP),
    (0xef53, VK_RIGHT),
    (0xef54, VK_DOWN),
    (0xef55, VK_PRIOR),
    (0xef56, VK_NEXT),
    (0xef57, VK_END),
    (0xef58, VK_CLEAR),
    (0xef60, VK_SELECT),
    (0xef61, VK_SNAPSHOT),
    (0xef62, VK_EXECUTE),
    (0xef63, VK_INSERT),
    (0xef67, VK_APPS),
    (0xef69, VK_CANCEL),
    (0xef6a, VK_HELP),
    (0xef6b, VK_CANCEL),
    (0xefff, VK_DELETE),
    (0xef7e, VK_RMENU),
    (0xef7f, VK_NUMLOCK),
    (0xef80, VK_SPACE),
    (0xef89, VK_TAB),
    (0xef8d, VK_RETURN),
    (0xef95, VK_HOME),
    (0xef96, VK_LEFT),
    (0xef97, VK_UP),
    (0xef98, VK_RIGHT),
    (0xef99, VK_DOWN),
    (0xef9a, VK_PRIOR),
    (0xef9b, VK_NEXT),
    (0xef9c, VK_END),
    (0xef9d, VK_CLEAR),
    (0xef9e, VK_INSERT),
    (0xef9f, VK_DELETE),
    (0xefaa, VK_MULTIPLY),
    (0xefab, VK_ADD),
    (0xefac, VK_SEPARATOR),
    (0xefad, VK_SUBTRACT),
    (0xefae, VK_DECIMAL),
    (0xefaf, VK_DIVIDE),
    (0xefbd, VK_OEM_NEC_EQUAL),
    (0xefe1, VK_LSHIFT),
    (0xefe2, VK_RSHIFT),
    (0xefe3, VK_LCONTROL),
    (0xefe4, VK_RCONTROL),
    (0xefe5, VK_CAPITAL),
    (0xefe7, VK_LWIN),
    (0xefe8, VK_RWIN),
    (0xefe9, VK_LMENU),
    (0xefea, VK_RMENU),
    (0xefeb, VK_LWIN),
    (0xefec, VK_RWIN),
    (0xe05f, VK_SLEEP),
    (0xe0a6, VK_BROWSER_BACK),
    (0xe0a7, VK_BROWSER_FORWARD),
    (0xe0a8, VK_BROWSER_REFRESH),
    (0xe0a9, VK_BROWSER_STOP),
    (0xe0aa, VK_BROWSER_SEARCH),
    (0xe0ab, VK_BROWSER_FAVORITES),
    (0xe0ac, VK_BROWSER_HOME),
    (0xe0ad, VK_VOLUME_MUTE),
    (0xe0ae, VK_VOLUME_DOWN),
    (0xe0af, VK_VOLUME_UP),
    (0xe0b0, VK_MEDIA_NEXT_TRACK),
    (0xe0b1, VK_MEDIA_PREV_TRACK),
    (0xe0b2, VK_MEDIA_STOP),
    (0xe0b3, VK_MEDIA_PLAY_PAUSE),
    (0xe0b4, VK_LAUNCH_MAIL),
    (0xe0b5, VK_LAUNCH_MEDIA_SELECT),
    (0xe0b6, VK_LAUNCH_APP1),
    (0xe0b7, VK_LAUNCH_APP2),
];
pub fn virtual_key(key: u16) -> Option<u16> {
    match key {
        0xefbe..=0xefd5 => Some(VK_F1 + key - 0xefbe),
        0xefb0..=0xefb9 => Some(VK_NUMPAD0 + key - 0xefb0),
        _ => NAMED.iter().find(|(k, _)| *k == key).map(|(_, v)| *v),
    }
}
pub fn named_key(vk: u16, scan: u32, extended: bool) -> Option<u16> {
    match vk {
        VK_HOME | VK_LEFT | VK_UP | VK_RIGHT | VK_DOWN | VK_PRIOR | VK_NEXT | VK_END | VK_INSERT | VK_DELETE
            if !extended =>
        {
            Some(match vk {
                VK_HOME => 0xef95,
                VK_LEFT => 0xef96,
                VK_UP => 0xef97,
                VK_RIGHT => 0xef98,
                VK_DOWN => 0xef99,
                VK_PRIOR => 0xef9a,
                VK_NEXT => 0xef9b,
                VK_END => 0xef9c,
                VK_INSERT => 0xef9e,
                _ => 0xef9f,
            })
        }
        VK_SPACE => Some(0x20),
        VK_RETURN if !extended => Some(0xef0d),
        VK_SNAPSHOT => Some(0xef61),
        VK_CANCEL => Some(0xef6b),
        VK_F1..=VK_F24 => Some(0xefbe + vk - VK_F1),
        VK_NUMPAD0..=VK_NUMPAD9 => Some(0xefb0 + vk - VK_NUMPAD0),
        VK_SHIFT => Some(if scan == 0x36 { 0xefe2 } else { 0xefe1 }),
        VK_CONTROL => Some(if extended { 0xefe4 } else { 0xefe3 }),
        VK_MENU => Some(if extended { 0xef7e } else { 0xefe9 }),
        VK_RMENU => Some(0xef7e),
        VK_RETURN if extended => Some(0xef8d),
        VK_LWIN => Some(0xefeb),
        VK_RWIN => Some(0xefec),
        _ => NAMED.iter().find(|(_, v)| *v == vk).map(|(k, _)| *k),
    }
}
pub fn extended(key: u16, vk: u16) -> bool {
    matches!(
        vk,
        VK_RCONTROL | VK_RMENU | VK_LWIN | VK_RWIN | VK_APPS | VK_DIVIDE | VK_NUMLOCK | VK_SNAPSHOT
    ) || key == 0xef8d
        || ((0xef50..=0xef57).contains(&key) || key == 0xef63 || key == 0xefff)
}
pub fn initial_state() -> [u8; 256] {
    let mut state = [0; 256];
    // Safety: each GetKeyState argument is one scalar VK in the documented 0..256 range.
    unsafe {
        for (vk, value) in state.iter_mut().enumerate() {
            let key = GetKeyState(vk as i32);
            *value = (if key < 0 { 0x80 } else { 0 }) | (key as u8 & 1);
        }
    }
    state
}
pub fn modifiers(state: &[u8; 256]) -> Modifiers {
    let mut mask = 0;
    for (vk, bit) in [
        (VK_SHIFT, Modifiers::SHIFT),
        (VK_CONTROL, Modifiers::CONTROL),
        (VK_MENU, Modifiers::ALT),
        (VK_LWIN, Modifiers::SUPER),
        (VK_RWIN, Modifiers::SUPER),
    ] {
        if state[usize::from(vk)] & 0x80 != 0 {
            mask |= bit;
        }
    }
    if state[usize::from(VK_RMENU)] & 0x80 != 0 && mask & Modifiers::CONTROL != 0 {
        mask = (mask & !(Modifiers::CONTROL | Modifiers::ALT)) | Modifiers::ALTGR;
    }
    for (vk, bit) in [
        (VK_CAPITAL, Modifiers::CAPS_LOCK),
        (VK_NUMLOCK, Modifiers::NUM_LOCK),
        (VK_SCROLL, Modifiers::SCROLL_LOCK),
    ] {
        if state[usize::from(vk)] & 1 != 0 {
            mask |= bit;
        }
    }
    Modifiers(mask)
}
