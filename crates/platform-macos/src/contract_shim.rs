//! Temporary integration-contract §2 shim. WP3 owns these items in
//! openergy-platform; switch Backend to its shared import at integration.
//! ClickNumberer mirrors the Step 0 model; time/distance history stays in
//! inject::MouseState until WP3's richer shared model is available.
use openergy_platform::{ClipboardBackend, InputSink, InputSource, ScreenSystem};

pub struct Backend {
    pub name: &'static str,
    pub source: Box<dyn InputSource>,
    pub sink: Box<dyn InputSink>,
    pub screens: Box<dyn ScreenSystem>,
    pub clipboard: Box<dyn ClipboardBackend>,
}

// Number identity uses the actual shared Step 0 type, not a second algorithm.
pub use openergy_platform::ClickNumberer;
