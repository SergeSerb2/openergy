//! Translate the shared protocol::keys KeyIDs into native Win32 virtual keys.
use openergy_platform::Modifiers;
use openergy_protocol::keys;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
const NAMED: &[(u16, u16)] = &[
    (keys::BACK_SPACE, VK_BACK),
    (keys::TAB, VK_TAB),
    (keys::LEFT_TAB, VK_TAB),
    (keys::LINEFEED, VK_RETURN),
    (keys::CLEAR, VK_CLEAR),
    (keys::RETURN, VK_RETURN),
    (keys::PAUSE, VK_PAUSE),
    (keys::SCROLL_LOCK, VK_SCROLL),
    (keys::SYS_REQ, VK_SNAPSHOT),
    (keys::ESCAPE, VK_ESCAPE),
    (keys::HENKAN, VK_CONVERT),
    (keys::ZENKAKU, VK_DBE_DBCSCHAR),
    (keys::HOME, VK_HOME),
    (keys::LEFT, VK_LEFT),
    (keys::UP, VK_UP),
    (keys::RIGHT, VK_RIGHT),
    (keys::DOWN, VK_DOWN),
    (keys::PAGE_UP, VK_PRIOR),
    (keys::PAGE_DOWN, VK_NEXT),
    (keys::END, VK_END),
    (keys::BEGIN, VK_CLEAR),
    (keys::SELECT, VK_SELECT),
    (keys::PRINT, VK_SNAPSHOT),
    (keys::EXECUTE, VK_EXECUTE),
    (keys::INSERT, VK_INSERT),
    (keys::MENU, VK_APPS),
    (keys::CANCEL, VK_CANCEL),
    (keys::HELP, VK_HELP),
    (keys::BREAK, VK_CANCEL),
    (keys::DELETE, VK_DELETE),
    (keys::ALT_GR, VK_RMENU),
    (keys::NUM_LOCK, VK_NUMLOCK),
    (keys::KP_SPACE, VK_SPACE),
    (keys::KP_TAB, VK_TAB),
    (keys::KP_ENTER, VK_RETURN),
    (keys::KP_HOME, VK_HOME),
    (keys::KP_LEFT, VK_LEFT),
    (keys::KP_UP, VK_UP),
    (keys::KP_RIGHT, VK_RIGHT),
    (keys::KP_DOWN, VK_DOWN),
    (keys::KP_PAGE_UP, VK_PRIOR),
    (keys::KP_PAGE_DOWN, VK_NEXT),
    (keys::KP_END, VK_END),
    (keys::KP_BEGIN, VK_CLEAR),
    (keys::KP_INSERT, VK_INSERT),
    (keys::KP_DELETE, VK_DELETE),
    (keys::KP_MULTIPLY, VK_MULTIPLY),
    (keys::KP_ADD, VK_ADD),
    (keys::KP_SEPARATOR, VK_SEPARATOR),
    (keys::KP_SUBTRACT, VK_SUBTRACT),
    (keys::KP_DECIMAL, VK_DECIMAL),
    (keys::KP_DIVIDE, VK_DIVIDE),
    (keys::KP_EQUAL, VK_OEM_NEC_EQUAL),
    (keys::SHIFT_L, VK_LSHIFT),
    (keys::SHIFT_R, VK_RSHIFT),
    (keys::CONTROL_L, VK_LCONTROL),
    (keys::CONTROL_R, VK_RCONTROL),
    (keys::CAPS_LOCK, VK_CAPITAL),
    (keys::META_L, VK_LWIN),
    (keys::META_R, VK_RWIN),
    (keys::ALT_L, VK_LMENU),
    (keys::ALT_R, VK_RMENU),
    (keys::SUPER_L, VK_LWIN),
    (keys::SUPER_R, VK_RWIN),
    (keys::SLEEP, VK_SLEEP),
    (keys::WWW_BACK, VK_BROWSER_BACK),
    (keys::WWW_FORWARD, VK_BROWSER_FORWARD),
    (keys::WWW_REFRESH, VK_BROWSER_REFRESH),
    (keys::WWW_STOP, VK_BROWSER_STOP),
    (keys::WWW_SEARCH, VK_BROWSER_SEARCH),
    (keys::WWW_FAVORITES, VK_BROWSER_FAVORITES),
    (keys::WWW_HOME, VK_BROWSER_HOME),
    (keys::AUDIO_MUTE, VK_VOLUME_MUTE),
    (keys::AUDIO_DOWN, VK_VOLUME_DOWN),
    (keys::AUDIO_UP, VK_VOLUME_UP),
    (keys::AUDIO_NEXT, VK_MEDIA_NEXT_TRACK),
    (keys::AUDIO_PREV, VK_MEDIA_PREV_TRACK),
    (keys::AUDIO_STOP, VK_MEDIA_STOP),
    (keys::AUDIO_PLAY, VK_MEDIA_PLAY_PAUSE),
    (keys::APP_MAIL, VK_LAUNCH_MAIL),
    (keys::APP_MEDIA, VK_LAUNCH_MEDIA_SELECT),
    (keys::APP_USER1, VK_LAUNCH_APP1),
    (keys::APP_USER2, VK_LAUNCH_APP2),
];
pub fn virtual_key(key: u16) -> Option<u16> {
    match key {
        keys::F1..=keys::F24 => Some(VK_F1 + key - keys::F1),
        keys::KP_0..=keys::KP_9 => Some(VK_NUMPAD0 + key - keys::KP_0),
        _ => NAMED.iter().find(|(k, _)| *k == key).map(|(_, v)| *v),
    }
}
pub fn named_key(vk: u16, scan: u32, extended: bool) -> Option<u16> {
    match vk {
        VK_HOME | VK_LEFT | VK_UP | VK_RIGHT | VK_DOWN | VK_PRIOR | VK_NEXT | VK_END | VK_INSERT | VK_DELETE
            if !extended =>
        {
            Some(match vk {
                VK_HOME => keys::KP_HOME,
                VK_LEFT => keys::KP_LEFT,
                VK_UP => keys::KP_UP,
                VK_RIGHT => keys::KP_RIGHT,
                VK_DOWN => keys::KP_DOWN,
                VK_PRIOR => keys::KP_PAGE_UP,
                VK_NEXT => keys::KP_PAGE_DOWN,
                VK_END => keys::KP_END,
                VK_INSERT => keys::KP_INSERT,
                _ => keys::KP_DELETE,
            })
        }
        VK_SPACE => Some(keys::SPACE),
        VK_RETURN if !extended => Some(keys::RETURN),
        VK_SNAPSHOT => Some(keys::PRINT),
        VK_CANCEL => Some(keys::BREAK),
        VK_F1..=VK_F24 => Some(keys::F1 + vk - VK_F1),
        VK_NUMPAD0..=VK_NUMPAD9 => Some(keys::KP_0 + vk - VK_NUMPAD0),
        VK_SHIFT => Some(if scan == 0x36 { keys::SHIFT_R } else { keys::SHIFT_L }),
        VK_CONTROL => Some(if extended { keys::CONTROL_R } else { keys::CONTROL_L }),
        VK_MENU => Some(if extended { keys::ALT_GR } else { keys::ALT_L }),
        VK_RMENU => Some(keys::ALT_GR),
        VK_RETURN if extended => Some(keys::KP_ENTER),
        VK_LWIN => Some(keys::SUPER_L),
        VK_RWIN => Some(keys::SUPER_R),
        _ => NAMED.iter().find(|(_, v)| *v == vk).map(|(k, _)| *k),
    }
}
pub fn extended(key: u16, vk: u16) -> bool {
    matches!(
        vk,
        VK_RCONTROL | VK_RMENU | VK_LWIN | VK_RWIN | VK_APPS | VK_DIVIDE | VK_NUMLOCK | VK_SNAPSHOT
    ) || key == keys::KP_ENTER
        || ((keys::HOME..=keys::END).contains(&key) || key == keys::INSERT || key == keys::DELETE)
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
