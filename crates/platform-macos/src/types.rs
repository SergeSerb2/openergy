//! Wire-facing types. Coordinates use global display points, not backing pixels.
pub type KeyId = u16;
pub type ModifierMask = u16;
pub type KeyButton = u16;

pub const SHIFT: ModifierMask = 0x0001;
pub const CONTROL: ModifierMask = 0x0002;
pub const ALT: ModifierMask = 0x0004;
pub const META: ModifierMask = 0x0008;
pub const SUPER: ModifierMask = 0x0010;
pub const ALT_GR: ModifierMask = 0x0020;
pub const CAPS_LOCK: ModifierMask = 0x1000;
pub const NUM_LOCK: ModifierMask = 0x2000;
pub const SCROLL_LOCK: ModifierMask = 0x4000;
