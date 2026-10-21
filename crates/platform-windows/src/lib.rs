//! Win32 backend stub (WP8). Frozen stub: returns typed NotImplemented errors.
#![allow(dead_code)]
use openergy_platform::{ClipboardBackend, ClipboardData, PlatformError};
pub struct Backend;
impl Backend {
    pub fn new() -> Self {
        Self
    }
}
impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}
pub struct MemoryClipboard {
    stamp: u64,
    data: ClipboardData,
}
impl Default for MemoryClipboard {
    fn default() -> Self {
        Self {
            stamp: 0,
            data: ClipboardData::default(),
        }
    }
}
impl ClipboardBackend for MemoryClipboard {
    fn change_stamp(&self) -> u64 {
        self.stamp
    }
    fn read(&mut self, _max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        Err(PlatformError::NotImplemented)
    }
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError> {
        self.data = data.clone();
        self.stamp += 1;
        Err(PlatformError::NotImplemented)
    }
}
