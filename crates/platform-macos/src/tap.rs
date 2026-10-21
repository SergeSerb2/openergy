//! Active session tap. Only InputSource::start creates/enables it.
use crate::ffi::OwnedCf;
use crate::inject::{cg_to_wire_button, INJECTION_TAG};
use crate::keymap::{self, CapturedKey, Keymap, CG_CAPS};
use crate::{error, ffi, permissions, MacNotice, Notices, Result};
use openergy_platform::{CaptureDecision, InputEvent, InputSource, Modifiers, PermissionKind, PlatformError};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Instant;

type Consumer = Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>;

#[derive(Default)]
struct CursorCapture {
    hidden: bool,
}
struct Control {
    swallow: AtomicBool,
    captured: AtomicBool,
    stopping: AtomicBool,
    cursor: Mutex<CursorCapture>,
}
impl Control {
    fn leave(&self) {
        self.swallow.store(false, Ordering::Release);
        self.captured.store(false, Ordering::Release);
        if let Ok(mut cursor) = self.cursor.lock() {
            if cursor.hidden {
                // Safety: balance only this source's hide/disassociation on explicit leave.
                unsafe {
                    if ffi::CGAssociateMouseAndMouseCursorPosition(true) != 0 {
                        log::error!("macOS capture leave could not restore cursor association");
                    }
                    if ffi::CGDisplayShowCursor(ffi::CGMainDisplayID()) == 0 {
                        cursor.hidden = false;
                    } else {
                        log::error!("macOS capture leave could not show cursor; retry on stop");
                    }
                }
            }
        }
    }
}

pub struct MacSource {
    tap: Option<EventTap>,
    control: Arc<Control>,
    lines_per_notch: f64,
    notices: Notices,
    enter_position: crate::EnterPosition,
}
impl MacSource {
    pub(crate) fn new(lines_per_notch: f64, notices: Notices, enter_position: crate::EnterPosition) -> Self {
        Self {
            tap: None,
            lines_per_notch,
            notices,
            enter_position,
            control: Arc::new(Control {
                swallow: AtomicBool::new(false),
                captured: AtomicBool::new(false),
                stopping: AtomicBool::new(false),
                cursor: Mutex::new(CursorCapture::default()),
            }),
        }
    }
}
impl InputSource for MacSource {
    fn start(&mut self, sink: Consumer) -> Result<()> {
        if self.tap.is_some() {
            return Err(error("capture already started; stop before restarting"));
        }
        let grants = permissions::preflight();
        if !grants.accessibility {
            return Err(PlatformError::PermissionDenied(PermissionKind::Accessibility));
        }
        if !grants.listen {
            return Err(PlatformError::PermissionDenied(PermissionKind::InputMonitoring));
        }
        self.control.stopping.store(false, Ordering::Release);
        self.tap = Some(EventTap::start(
            sink,
            self.control.clone(),
            self.notices.clone(),
            self.lines_per_notch,
        )?);
        Ok(())
    }
    fn stop(&mut self) {
        if let Some(mut tap) = self.tap.take() {
            tap.shutdown();
        }
        self.control.leave();
    }
    fn set_cursor_captured(&mut self, captured: bool) -> Result<()> {
        if !captured {
            self.control.leave();
            return Ok(());
        }
        if self.tap.is_none() || self.control.stopping.load(Ordering::Acquire) {
            return Err(error("capture must be running before locking cursor"));
        }
        let mut cursor = self.control.cursor.lock().map_err(|_| error("cursor state poisoned"))?;
        if cursor.hidden {
            return Ok(());
        }
        let displays = crate::displays::active()?;
        // Safety: CGEventCreate queries metadata only; the owned event outlives the read.
        let point = unsafe {
            let event = OwnedCf::new(ffi::CGEventCreate(ffi::null()).cast_const(), "cursor query")?;
            ffi::CGEventGetLocation(event.0)
        };
        let display = displays
            .active
            .iter()
            .find(|d| {
                point.x >= d.bounds.x
                    && point.y >= d.bounds.y
                    && point.x < d.bounds.x + d.bounds.width
                    && point.y < d.bounds.y + d.bounds.height
            })
            .ok_or_else(|| error("cursor is outside active displays"))?;
        let b = display.bounds;
        let edges = [
            (point.x - b.x, ffi::Point { x: b.x, y: point.y }),
            (
                b.x + b.width - 1.0 - point.x,
                ffi::Point {
                    x: b.x + b.width - 1.0,
                    y: point.y,
                },
            ),
            (point.y - b.y, ffi::Point { x: point.x, y: b.y }),
            (
                b.y + b.height - 1.0 - point.y,
                ffi::Point {
                    x: point.x,
                    y: b.y + b.height - 1.0,
                },
            ),
        ];
        let pin = edges
            .into_iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|e| e.1)
            .ok_or_else(|| error("no cursor edge"))?;
        // Safety: explicit captured-mode request, never called by safe probes.
        unsafe {
            check(ffi::CGWarpMouseCursorPosition(pin), "pin cursor")?;
            check(ffi::CGDisplayHideCursor(ffi::CGMainDisplayID()), "hide captured cursor")?;
            if let Err(err) = check(ffi::CGAssociateMouseAndMouseCursorPosition(false), "lock cursor") {
                ffi::CGDisplayShowCursor(ffi::CGMainDisplayID());
                return Err(err);
            }
        }
        cursor.hidden = true;
        self.control.captured.store(true, Ordering::Release);
        self.control.swallow.store(true, Ordering::Release);
        Ok(())
    }
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<()> {
        // CINN only. Ordinary InputSink moves post mouse events.
        // Safety: explicit caller request; no probe calls this method.
        unsafe {
            check(
                ffi::CGWarpMouseCursorPosition(ffi::Point {
                    x: f64::from(x),
                    y: f64::from(y),
                }),
                "enter warp",
            )?;
            check(
                ffi::CGAssociateMouseAndMouseCursorPosition(true),
                "enter cursor association",
            )?;
        }
        // The sink belongs to another thread. Deliver the CINN point without
        // a synthetic move or a second warp before the next injected input.
        *self
            .enter_position
            .lock()
            .map_err(|_| error("enter position poisoned"))? = Some((x, y));
        Ok(())
    }
}
impl Drop for MacSource {
    fn drop(&mut self) {
        self.stop();
    }
}
fn check(status: i32, operation: &str) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(error(format!("{operation} failed: {status}")))
    }
}

struct EventTap {
    wake: ffi::RunLoopWake,
    control: Arc<Control>,
    thread: Option<JoinHandle<()>>,
}
impl EventTap {
    fn start(sink: Consumer, control: Arc<Control>, notices: Notices, lines: f64) -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel(1);
        let thread_control = control.clone();
        let handle = thread::Builder::new()
            .name("openergy-capture".into())
            .spawn(move || {
                let setup = (|| -> Result<_> {
                    let _pool = ffi::Pool::new();
                    // Safety: metadata-only event snapshot initializes Caps state without posting.
                    let caps = unsafe {
                        let event =
                            OwnedCf::new(ffi::CGEventCreate(ffi::null()).cast_const(), "initial capture flags")?;
                        ffi::CGEventGetFlags(event.0) & CG_CAPS != 0
                    };
                    let mut state = Box::new(CallbackState {
                        tap: ffi::null(),
                        control: thread_control.clone(),
                        notices: notices.clone(),
                        consumer: sink,
                        keymap: Keymap::new()?,
                        held: [None; 256],
                        caps,
                        lines,
                        wheel: [0.0; 2],
                        epoch: Instant::now(),
                        disabled: [0.0; 6],
                        disable_count: 0,
                        layout_dirty: false,
                    });
                    // Safety: callback state has a stable heap address through tap invalidation.
                    let tap = OwnedCf::new(
                        unsafe {
                            ffi::CGEventTapCreate(
                                1,
                                0,
                                0,
                                u64::MAX,
                                callback,
                                (&mut *state as *mut CallbackState).cast(),
                            )
                        },
                        "active session event tap",
                    )?;
                    state.tap = tap.0;
                    // Safety: live tap; owned source released after removal from its run loop.
                    let source = OwnedCf::new(
                        unsafe { ffi::CFMachPortCreateRunLoopSource(ffi::null(), tap.0, 0) },
                        "tap run-loop source",
                    )?;
                    let (wake, control_source) = ffi::RunLoopWake::new()?;
                    // Safety: notification observer lives on this run loop until explicit removal.
                    unsafe {
                        ffi::CFRunLoopAddSource(wake.loop_ref as ffi::Ref, source.0, ffi::kCFRunLoopCommonModes);
                        ffi::CFNotificationCenterAddObserver(
                            ffi::CFNotificationCenterGetDistributedCenter(),
                            (&mut *state as *mut CallbackState).cast(),
                            layout_changed,
                            ffi::kTISNotifySelectedKeyboardInputSourceChanged,
                            ffi::null(),
                            4,
                        );
                    }
                    Ok((tap, source, state, wake, control_source))
                })();
                match setup {
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                    Ok((tap, source, mut state, wake, control_source)) => {
                        let loop_ref = wake.loop_ref as ffi::Ref;
                        if tx.send(Ok(wake)).is_ok() {
                            while !thread_control.stopping.load(Ordering::Acquire) {
                                let _pool = ffi::Pool::new();
                                if state.layout_dirty {
                                    state.layout_dirty = false;
                                    if state.keymap.refresh().is_err() {
                                        notices.send(MacNotice::CaptureTranslationProblem);
                                        break;
                                    }
                                }
                                // Return after one source, then handle layout changes outside callback.
                                // Safety: current run loop owns all registered sources and state.
                                if unsafe { ffi::CFRunLoopRunInMode(ffi::kCFRunLoopDefaultMode, 86400.0, true) } == 1 {
                                    break;
                                }
                            }
                        }
                        // Safety: stop callbacks before freeing their state; all refs remain live.
                        unsafe {
                            ffi::CFNotificationCenterRemoveEveryObserver(
                                ffi::CFNotificationCenterGetDistributedCenter(),
                                (&mut *state as *mut CallbackState).cast(),
                            );
                            ffi::CGEventTapEnable(tap.0, false);
                            ffi::CFMachPortInvalidate(tap.0);
                            ffi::CFRunLoopRemoveSource(loop_ref, source.0, ffi::kCFRunLoopCommonModes);
                            ffi::CFRunLoopSourceInvalidate(control_source.0);
                        }
                    }
                }
                thread_control.leave();
                thread_control.stopping.store(true, Ordering::Release);
                notices.send(MacNotice::CaptureStopped);
            })
            .map_err(|e| error(format!("capture thread: {e}")))?;
        match rx.recv() {
            Ok(Ok(wake)) => Ok(Self {
                wake,
                control,
                thread: Some(handle),
            }),
            Ok(Err(e)) => {
                let _ = handle.join();
                Err(e)
            }
            Err(_) => {
                let _ = handle.join();
                Err(error("capture setup channel closed"))
            }
        }
    }
    fn shutdown(&mut self) {
        self.control.swallow.store(false, Ordering::Release);
        self.control.stopping.store(true, Ordering::Release);
        self.wake.wake();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}
impl Drop for EventTap {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct CallbackState {
    tap: ffi::Ref,
    control: Arc<Control>,
    notices: Notices,
    consumer: Consumer,
    keymap: Keymap,
    held: [Option<CapturedKey>; 256],
    caps: bool,
    lines: f64,
    wheel: [f64; 2],
    epoch: Instant,
    disabled: [f64; 6],
    disable_count: usize,
    layout_dirty: bool,
}
unsafe extern "C" fn layout_changed(_: ffi::Ref, observer: ffi::MutRef, _: ffi::Ref, _: ffi::Ref, _: ffi::Ref) {
    if !observer.is_null() {
        // Safety: registered stable state is owned by this same CFRunLoop thread.
        unsafe {
            (*observer.cast::<CallbackState>()).layout_dirty = true;
        }
    }
}
unsafe extern "C" fn callback(_: ffi::MutRef, kind: u32, event: ffi::MutRef, user: ffi::MutRef) -> ffi::MutRef {
    if user.is_null() {
        return event;
    }
    // Safety: user points to CallbackState until the tap is disabled and invalidated.
    let state = unsafe { &mut *user.cast::<CallbackState>() };
    if kind >= u32::MAX - 1 {
        let now = state.epoch.elapsed().as_secs_f64();
        state.disabled[state.disable_count % 6] = now;
        state.disable_count = state.disable_count.saturating_add(1);
        if state.disable_count >= 6 && now - state.disabled[state.disable_count % 6] <= 10.0 {
            state.control.swallow.store(false, Ordering::Release);
            state.control.stopping.store(true, Ordering::Release);
            state.notices.send(MacNotice::CapturePermissionProblem);
        } else {
            // Safety: this is our own live tap; guarded against re-enable storms.
            unsafe {
                ffi::CGEventTapEnable(state.tap, true);
            }
        }
        return event;
    }
    if event.is_null() || state.control.stopping.load(Ordering::Acquire) {
        return event;
    }
    // Safety: CoreGraphics supplies a live event throughout the callback.
    if unsafe { ffi::CGEventGetIntegerValueField(event.cast_const(), 42) } == INJECTION_TAG {
        return event;
    }
    // A layout notification must be processed before looking up the next key.
    // Pass it through until the outer loop rebuilds the table.
    if state.layout_dirty && matches!(kind, 10..=12) {
        return event;
    }
    let events = state.translate(kind, event.cast_const());
    let mut swallow = state.control.swallow.load(Ordering::Acquire);
    let mut any = false;
    for input in events.into_iter().flatten() {
        any = true;
        swallow = (state.consumer)(input) == CaptureDecision::Swallow;
        state.control.swallow.store(swallow, Ordering::Release);
    }
    // A fractional wheel sample can produce no wire event yet. Suppress that
    // original sample too while remote-active, otherwise local apps scroll.
    let input_kind = matches!(kind, 1..=7 | 10..=12 | 22 | 25..=27);
    if (any || input_kind) && swallow {
        std::ptr::null_mut()
    } else {
        event
    }
}
impl CallbackState {
    fn translate(&mut self, kind: u32, event: ffi::Ref) -> [Option<InputEvent>; 2] {
        let mut out = [None, None];
        // Safety: event is borrowed from the tap callback, and all reads are typed CGEvent fields.
        unsafe {
            let flags = ffi::CGEventGetFlags(event);
            let modifiers = Modifiers(keymap::flags_to_mask(flags));
            let input = match kind {
                10..=12 => {
                    let raw = ffi::CGEventGetIntegerValueField(event, 9);
                    let Ok(code) = u8::try_from(raw) else {
                        return out;
                    };
                    let index = usize::from(code);
                    let code = u16::from(code);
                    if kind == 12 && code == 0x39 {
                        let caps = flags & CG_CAPS != 0;
                        if caps != self.caps {
                            self.caps = caps;
                            if let Some(k) = self.keymap.capture_cached(code, flags) {
                                out = [
                                    Some(InputEvent::KeyDown {
                                        key: k.key,
                                        modifiers,
                                        button: k.button,
                                    }),
                                    Some(InputEvent::KeyUp {
                                        key: k.key,
                                        modifiers,
                                        button: k.button,
                                    }),
                                ];
                            }
                        }
                        return out;
                    }
                    let down = if kind == 12 {
                        let Some((aggregate, device)) = keymap::modifier_for_code(code) else {
                            return out;
                        };
                        if flags & 0x0000_207F != 0 && device != 0 {
                            flags & device != 0
                        } else {
                            flags & aggregate != 0 && self.held[index].is_none()
                        }
                    } else {
                        kind == 10
                    };
                    if down {
                        let Some(k) = self.held[index].or_else(|| self.keymap.capture_cached(code, flags)) else {
                            return out;
                        };
                        self.held[index] = Some(k);
                        if kind == 10 && ffi::CGEventGetIntegerValueField(event, 8) != 0 {
                            InputEvent::KeyRepeat {
                                key: k.key,
                                modifiers,
                                count: 1,
                                button: k.button,
                            }
                        } else {
                            InputEvent::KeyDown {
                                key: k.key,
                                modifiers,
                                button: k.button,
                            }
                        }
                    } else if let Some(k) = self.held[index].take() {
                        InputEvent::KeyUp {
                            key: k.key,
                            modifiers,
                            button: k.button,
                        }
                    } else {
                        return out;
                    }
                }
                1 | 2 | 3 | 4 | 25 | 26 => {
                    let cg = match kind {
                        1 | 2 => 0,
                        3 | 4 => 1,
                        _ => ffi::CGEventGetIntegerValueField(event, 3) as u32,
                    };
                    let Some(button) = cg_to_wire_button(cg) else {
                        return out;
                    };
                    if matches!(kind, 1 | 3 | 25) {
                        InputEvent::MouseDown { button }
                    } else {
                        InputEvent::MouseUp { button }
                    }
                }
                5 | 6 | 7 | 27 => {
                    if self.control.captured.load(Ordering::Acquire) {
                        InputEvent::MouseMoveRel {
                            dx: ffi::CGEventGetDoubleValueField(event, 4).round() as i32,
                            dy: ffi::CGEventGetDoubleValueField(event, 5).round() as i32,
                        }
                    } else {
                        let p = ffi::CGEventGetLocation(event);
                        InputEvent::MouseMoveAbs {
                            x: p.x.round() as i32,
                            y: p.y.round() as i32,
                        }
                    }
                }
                22 => {
                    let continuous = ffi::CGEventGetIntegerValueField(event, 88) != 0;
                    let mut delta = [0; 2];
                    for (i, (line, pixel)) in [(94, 97), (93, 96)].into_iter().enumerate() {
                        let lines = if continuous {
                            ffi::CGEventGetDoubleValueField(event, pixel) / 10.0
                        } else {
                            ffi::CGEventGetDoubleValueField(event, line)
                        };
                        let units = self.wheel[i] + lines * 120.0 / self.lines;
                        if !units.is_finite() {
                            return out;
                        }
                        delta[i] = units.trunc().clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32;
                        self.wheel[i] = units - f64::from(delta[i]);
                    }
                    if delta == [0, 0] {
                        return out;
                    }
                    InputEvent::MouseWheel {
                        dx: delta[0],
                        dy: delta[1],
                    }
                }
                _ => return out,
            };
            out[0] = Some(input);
        }
        out
    }
}
