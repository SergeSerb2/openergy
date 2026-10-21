//! Dormant until `EventTap::start` is explicitly called. No CLI path starts it.
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};

use crate::ffi::{self, OwnedCf};
use crate::inject::{INJECTION_TAG, cg_to_wire_button};
use crate::keymap::{self, CG_CAPS, CapturedKey, Keymap};
use crate::types::InputEvent;
use crate::{Result, error};

#[derive(Debug, Clone, Copy)]
pub struct TapConfig {
    pub relative_moves: bool,
    pub lines_per_notch: f64,
    pub pixels_per_line: f64,
}
impl Default for TapConfig {
    fn default() -> Self {
        Self {
            relative_moves: false,
            lines_per_notch: 3.0,
            pixels_per_line: 10.0,
        }
    }
}

struct CallbackState {
    tap: ffi::Ref,
    swallow: Arc<AtomicBool>,
    sender: mpsc::Sender<InputEvent>,
    keymap: Keymap,
    held: HashMap<u16, CapturedKey>,
    caps: bool,
    config: TapConfig,
    wheel_remainder: [f64; 2],
}

pub struct EventTap {
    swallow: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    // An owned CFRunLoop reference retained on its originating thread, stored
    // as an address for Send. CFRunLoopStop/WakeUp are explicitly thread-safe.
    run_loop: usize,
    thread: Option<JoinHandle<()>>,
}

impl EventTap {
    /// This creates and enables a global session event tap. Requires prior
    /// authorization and Input Monitoring/Accessibility grants. Not a check.
    pub fn start(config: TapConfig, sender: mpsc::Sender<InputEvent>, swallow: Arc<AtomicBool>) -> Result<Self> {
        if !config.lines_per_notch.is_finite()
            || config.lines_per_notch <= 0.0
            || !config.pixels_per_line.is_finite()
            || config.pixels_per_line <= 0.0
        {
            return Err(error("invalid capture scroll configuration"));
        }
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread_swallow = swallow.clone();
        let stopping = Arc::new(AtomicBool::new(false));
        let thread_stopping = stopping.clone();
        let handle = thread::Builder::new()
            .name("openergy-event-tap".into())
            .spawn(move || {
                let _pool = ffi::Pool::new();
                let setup = (|| -> Result<(OwnedCf, OwnedCf, Box<CallbackState>, usize)> {
                    let mut state = Box::new(CallbackState {
                        tap: ffi::null(),
                        swallow: thread_swallow,
                        sender,
                        keymap: Keymap::new()?,
                        held: HashMap::new(),
                        caps: false,
                        config,
                        wheel_remainder: [0.0; 2],
                    });
                    let kinds = [1u32, 2, 3, 4, 5, 6, 7, 10, 11, 12, 22, 25, 26, 27];
                    let mask = kinds.into_iter().fold(0u64, |mask, kind| mask | (1 << kind));
                    let tap = OwnedCf::new(
                        unsafe {
                            ffi::CGEventTapCreate(1, 0, 0, mask, callback, (&mut *state as *mut CallbackState).cast())
                        },
                        "CG session event tap (check permissions)",
                    )?;
                    state.tap = tap.0;
                    let source = OwnedCf::new(
                        unsafe { ffi::CFMachPortCreateRunLoopSource(ffi::null(), tap.0, 0) },
                        "event-tap run-loop source",
                    )?;
                    let run_loop = unsafe { ffi::CFRetain(ffi::CFRunLoopGetCurrent()) } as usize;
                    Ok((tap, source, state, run_loop))
                })();
                match setup {
                    Err(err) => {
                        let _ = ready_tx.send(Err(err));
                    }
                    Ok((tap, source, state, run_loop)) => {
                        let loop_ref = run_loop as ffi::Ref;
                        unsafe { ffi::CFRunLoopAddSource(loop_ref, source.0, ffi::kCFRunLoopCommonModes) };
                        // Keep a separate reference for the controller even
                        // if this run loop exits before EventTap is dropped.
                        unsafe { ffi::CFRetain(loop_ref) };
                        if ready_tx.send(Ok(run_loop)).is_ok() {
                            // Bounded iterations close the stop-before-run
                            // race without a block callback or a polling spin.
                            while !thread_stopping.load(Ordering::Acquire) {
                                let _iteration_pool = ffi::Pool::new();
                                let result =
                                    unsafe { ffi::CFRunLoopRunInMode(ffi::kCFRunLoopDefaultMode, 0.25, false) };
                                if result == 1 {
                                    break;
                                } // kCFRunLoopRunFinished: source invalidated.
                            }
                        } else {
                            unsafe { ffi::CFRelease(loop_ref) };
                        }
                        unsafe {
                            ffi::CGEventTapEnable(tap.0, false);
                            ffi::CFMachPortInvalidate(tap.0);
                            ffi::CFRunLoopRemoveSource(loop_ref, source.0, ffi::kCFRunLoopCommonModes);
                            ffi::CFRelease(loop_ref);
                        }
                        // The callback's user pointer outlives tap invalidation.
                        drop(state);
                    }
                }
            })
            .map_err(|err| error(format!("cannot start capture thread: {err}")))?;
        match ready_rx.recv() {
            Ok(Ok(run_loop)) => Ok(Self {
                swallow,
                stopping,
                run_loop,
                thread: Some(handle),
            }),
            Ok(Err(err)) => {
                let _ = handle.join();
                Err(err)
            }
            Err(err) => {
                let _ = handle.join();
                Err(error(format!("capture setup failed: {err}")))
            }
        }
    }

    pub fn set_swallow(&self, swallow: bool) {
        self.swallow.store(swallow, Ordering::Release);
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.swallow.store(false, Ordering::Release);
        self.stopping.store(true, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            unsafe {
                ffi::CFRunLoopStop(self.run_loop as ffi::Ref);
                ffi::CFRunLoopWakeUp(self.run_loop as ffi::Ref);
            }
            let _ = handle.join();
            unsafe { ffi::CFRelease(self.run_loop as ffi::Ref) };
        }
    }
}
impl Drop for EventTap {
    fn drop(&mut self) {
        self.shutdown();
    }
}

unsafe extern "C" fn callback(_proxy: ffi::MutRef, kind: u32, event: ffi::MutRef, user: ffi::MutRef) -> ffi::MutRef {
    // Rust panics must never unwind through CoreGraphics' C callback.
    let decision = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if user.is_null() {
            return false;
        }
        let state = unsafe { &mut *user.cast::<CallbackState>() };
        if kind == u32::MAX - 1 || kind == u32::MAX {
            // Re-enable both timeout and user-input notices. Trust revocation
            // can make this ineffective; the owner must monitor permissions.
            unsafe { ffi::CGEventTapEnable(state.tap, true) };
            return false;
        }
        if event.is_null() {
            return false;
        }
        if unsafe { ffi::CGEventGetIntegerValueField(event.cast_const(), 42) } == INJECTION_TAG {
            return false;
        }
        match state.translate(kind, event.cast_const()) {
            Ok(events) => {
                if events.is_empty() {
                    return false;
                }
                // std::mpsc Sender never waits for a consumer. The owner must
                // drain promptly; dropping the receiver makes capture pass.
                for input in events {
                    if state.sender.send(input).is_err() {
                        state.swallow.store(false, Ordering::Release);
                        return false;
                    }
                }
                state.swallow.load(Ordering::Acquire)
            }
            Err(_) => {
                state.swallow.store(false, Ordering::Release);
                false
            }
        }
    }));
    if decision.unwrap_or(false) {
        std::ptr::null_mut()
    } else {
        event
    }
}

impl CallbackState {
    fn translate(&mut self, kind: u32, event: ffi::Ref) -> Result<Vec<InputEvent>> {
        let flags = unsafe { ffi::CGEventGetFlags(event) };
        let mask = keymap::flags_to_mask(flags);
        let mut output = Vec::with_capacity(2);
        match kind {
            10..=12 => {
                let code = unsafe { ffi::CGEventGetIntegerValueField(event, 9) };
                let Ok(code) = u16::try_from(code) else {
                    return Ok(output);
                };
                if kind == 12 && code == 0x39 {
                    let caps = flags & CG_CAPS != 0;
                    if caps != self.caps {
                        self.caps = caps;
                        if let Some(captured) = self.keymap.capture(code, flags)? {
                            output.push(InputEvent::KeyDown {
                                key: captured.key,
                                mask,
                                button: captured.button,
                            });
                            output.push(InputEvent::KeyUp {
                                key: captured.key,
                                mask,
                                button: captured.button,
                            });
                        }
                    }
                    return Ok(output);
                }
                let down = if kind == 12 {
                    let Some((aggregate, device)) = keymap::modifier_for_code(code) else {
                        return Ok(output);
                    };
                    // Device bits distinguish releasing left Shift while right
                    // Shift remains held. Use aggregate only when unavailable.
                    if flags & 0x0000_207F != 0 && device != 0 {
                        flags & device != 0
                    } else {
                        flags & aggregate != 0 && !self.held.contains_key(&code)
                    }
                } else {
                    kind == 10
                };
                if down {
                    let captured = match self.held.get(&code).copied() {
                        Some(captured) => Some(captured),
                        None => self.keymap.capture(code, flags)?,
                    };
                    if let Some(captured) = captured {
                        let repeat = kind == 10 && unsafe { ffi::CGEventGetIntegerValueField(event, 8) } != 0;
                        self.held.insert(code, captured);
                        output.push(if repeat {
                            InputEvent::KeyRepeat {
                                key: captured.key,
                                mask,
                                count: 1,
                                button: captured.button,
                            }
                        } else {
                            InputEvent::KeyDown {
                                key: captured.key,
                                mask,
                                button: captured.button,
                            }
                        });
                    }
                } else if let Some(captured) = self.held.remove(&code) {
                    output.push(InputEvent::KeyUp {
                        key: captured.key,
                        mask,
                        button: captured.button,
                    });
                }
            }
            1 | 2 | 3 | 4 | 25 | 26 => {
                let cg = match kind {
                    1 | 2 => 0,
                    3 | 4 => 1,
                    _ => (unsafe { ffi::CGEventGetIntegerValueField(event, 3) }) as u32,
                };
                if let Some(button) = cg_to_wire_button(cg) {
                    output.push(if matches!(kind, 1 | 3 | 25) {
                        InputEvent::MouseDown { button }
                    } else {
                        InputEvent::MouseUp { button }
                    });
                }
            }
            5 | 6 | 7 | 27 => {
                if self.config.relative_moves {
                    output.push(InputEvent::MouseMoveRel {
                        dx: unsafe { ffi::CGEventGetDoubleValueField(event, 4) },
                        dy: unsafe { ffi::CGEventGetDoubleValueField(event, 5) },
                    });
                } else {
                    let point = unsafe { ffi::CGEventGetLocation(event) };
                    output.push(InputEvent::MouseMoveAbs { x: point.x, y: point.y });
                }
            }
            22 => {
                let continuous = unsafe { ffi::CGEventGetIntegerValueField(event, 88) } != 0;
                let mut deltas = [0i32; 2];
                for (axis, (line_field, point_field)) in [(94, 97), (93, 96)].into_iter().enumerate() {
                    let lines = if continuous {
                        (unsafe { ffi::CGEventGetDoubleValueField(event, point_field) }) / self.config.pixels_per_line
                    } else {
                        unsafe { ffi::CGEventGetDoubleValueField(event, line_field) }
                    };
                    let units = self.wheel_remainder[axis] + lines * 120.0 / self.config.lines_per_notch;
                    deltas[axis] = units.trunc() as i32;
                    self.wheel_remainder[axis] = units - f64::from(deltas[axis]);
                }
                if deltas != [0, 0] {
                    output.push(InputEvent::MouseWheel {
                        dx: deltas[0],
                        dy: deltas[1],
                    });
                }
            }
            _ => {}
        }
        Ok(output)
    }
}
