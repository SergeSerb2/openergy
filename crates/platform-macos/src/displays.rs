//! Read-only display geometry and public secure-input status.
use crate::ffi::{self, OwnedCf};
use crate::{error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub id: u32,
    pub bounds: Bounds,
    pub scale_x: f64,
    pub scale_y: f64,
    pub main: bool,
}

#[derive(Debug, Clone)]
pub struct Displays {
    pub active: Vec<Display>,
    pub union: Option<Bounds>,
    pub main_id: u32,
}

pub fn active() -> Result<Displays> {
    // Safety: display IDs and owned modes remain live; list output buffers match declared capacities.
    unsafe {
        // Retry a bounded number of times if a hot-plug grows the list.
        for _ in 0..3 {
            let mut count = 0;
            check(ffi::CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut count))?;
            let mut ids = vec![0; count as usize];
            let mut actual = 0;
            check(ffi::CGGetActiveDisplayList(count, ids.as_mut_ptr(), &mut actual))?;
            if actual > count {
                continue;
            }
            ids.truncate(actual as usize);
            let main_id = ffi::CGMainDisplayID();
            let mut active = Vec::with_capacity(ids.len());
            let mut union: Option<Bounds> = None;
            for id in ids {
                let rect = ffi::CGDisplayBounds(id);
                let bounds = Bounds {
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                };
                let mode = OwnedCf::new(ffi::CGDisplayCopyDisplayMode(id), "CG display mode")?;
                let width = ffi::CGDisplayModeGetWidth(mode.0);
                let height = ffi::CGDisplayModeGetHeight(mode.0);
                let scale_x = ffi::CGDisplayModeGetPixelWidth(mode.0) as f64 / width.max(1) as f64;
                let scale_y = ffi::CGDisplayModeGetPixelHeight(mode.0) as f64 / height.max(1) as f64;
                union = Some(match union {
                    None => bounds,
                    Some(old) => {
                        let x = old.x.min(bounds.x);
                        let y = old.y.min(bounds.y);
                        Bounds {
                            x,
                            y,
                            width: (old.x + old.width).max(bounds.x + bounds.width) - x,
                            height: (old.y + old.height).max(bounds.y + bounds.height) - y,
                        }
                    }
                });
                active.push(Display {
                    id,
                    bounds,
                    scale_x,
                    scale_y,
                    main: id == main_id,
                });
            }
            return Ok(Displays { active, union, main_id });
        }
        Err(error("display list kept changing during enumeration"))
    }
}

fn check(status: i32) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(error(format!("CoreGraphics display error {status}")))
    }
}

#[derive(Debug, Clone)]
pub struct SecureInput {
    pub enabled: bool,
    pub owner_app: Option<String>,
}

pub fn secure_input() -> SecureInput {
    // Public Carbon can report the state, but exposes no owning PID. Avoid
    // undocumented CGSSession/IORegistry property keys and never guess.
    SecureInput {
        // Safety: public Carbon query reads secure-input state without changing it.
        enabled: unsafe { ffi::IsSecureEventInputEnabled() },
        owner_app: None,
    }
}
