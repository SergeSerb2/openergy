//! Wire-facing types. Coordinates use global display points, not backing pixels.
pub type KeyId = u16;
pub type ModifierMask = u16;
pub type KeyButton = u16;

// Compatibility names from the spike, backed by the shared wire definitions.
pub use openergy_protocol::keys::modifiers::{
    ALT, ALTGR as ALT_GR, CAPS as CAPS_LOCK, CONTROL, META, NUM as NUM_LOCK, SCROLL as SCROLL_LOCK, SHIFT, SUPER,
};
