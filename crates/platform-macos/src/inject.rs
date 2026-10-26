//! Global HID injection. Only `NativePoster::post` calls CGEventPost.
//! Inspection mode constructs and reads the same CGEvents without posting them.
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::ffi::{self, OwnedCf};
use crate::keymap::{self, InjectionPlan, Keymap, CG_CAPS, CG_COMMAND, CG_CONTROL, CG_OPTION, CG_SHIFT};
use crate::types::{KeyButton, KeyId, ModifierMask};
use crate::{error, Result};
use openergy_platform::{ClickNumberer, InputEvent, InputSink};
use openergy_protocol::keys::{self, buttons};

/// Public Openergy self-event tag. The tap passes these events through.
pub const INJECTION_TAG: i64 = 0x4F50_454E_4552_4759; // "OPENERGY"

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickNumbering {
    Native,
    Legacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Down,
    Up,
    Move,
    Drag,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SynthEvent {
    Mouse {
        kind: MouseKind,
        button: u8,
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        number: Option<i64>,
        click_state: i64,
        flags: u64,
    },
    Keyboard {
        keycode: u16,
        down: bool,
        repeat: bool,
        unicode: Vec<u16>,
        flags: u64,
    },
    Wheel {
        dx_lines: i32,
        dy_lines: i32,
        pixels_per_line: f64,
        flags: u64,
    },
}

pub trait EventPoster {
    fn post(&mut self, event: &SynthEvent) -> Result<()>;
    fn mouse_counter_seed(&self) -> i64;
    fn cursor_position(&self) -> Result<Option<(f64, f64)>> {
        Ok(None)
    }
    fn warp(&mut self, _x: f64, _y: f64) -> Result<()> {
        Err(error("poster cannot warp the cursor"))
    }
    fn cursor_visible(&mut self, _visible: bool) -> Result<()> {
        Err(error("poster cannot change cursor visibility"))
    }
}

pub struct NativePoster {
    source: OwnedCf,
    hidden_display: Option<u32>,
    mode: ClickNumbering,
    inspect: bool,
    post_allowed: bool,
    observations: Vec<NativeEventFields>,
    last_seed: std::cell::Cell<i64>,
}
impl NativePoster {
    pub fn new(mode: ClickNumbering) -> Result<Self> {
        // Source state HIDSystemState=1; posting location HIDEventTap=0.
        // Safety: HIDSystemState is a public enum value; NULL is checked by OwnedCf.
        let source = OwnedCf::new(unsafe { ffi::CGEventSourceCreate(1) }, "CG HID event source")?;
        // Safety: source is a live owned CGEventSource; these setters change its metadata only.
        unsafe {
            ffi::CGEventSourceSetPixelsPerLine(source.0, 10.0);
            ffi::CGEventSourceSetLocalEventsSuppressionInterval(source.0, 0.0);
        }
        Ok(Self {
            source,
            hidden_display: None,
            mode,
            inspect: false,
            post_allowed: crate::permissions::preflight().post,
            observations: Vec::new(),
            last_seed: std::cell::Cell::new(0),
        })
    }
}

impl NativePoster {
    /// Build and inspect genuine CGEvents; prohibit all global input mutations.
    pub fn inspection(mode: ClickNumbering) -> Result<Self> {
        let mut poster = Self::new(mode)?;
        poster.inspect = true;
        Ok(poster)
    }
    pub fn observations(&self) -> &[NativeEventFields] {
        &self.observations
    }
}

/// Values read from a real CGEvent after all setters used by the live sink.
#[derive(Debug, Clone)]
pub struct NativeEventFields {
    pub kind: u32,
    pub location: (f64, f64),
    pub button: i64,
    pub number: i64,
    pub click_state: i64,
    pub flags: u64,
    pub pointer_delta: (i64, i64),
    pub lines: (i64, i64),
    pub fixed_lines: (f64, f64),
    pub pixels: (i64, i64),
    pub keycode: i64,
    pub user_data: i64,
    pub unicode: Vec<u16>,
    pub seed: i64,
}
impl NativeEventFields {
    fn read(event: ffi::Ref, seed: i64) -> Self {
        // Safety: event is a live CGEvent held by the caller for this entire read.
        unsafe {
            let point = ffi::CGEventGetLocation(event);
            let mut text = [0u16; 16];
            let mut len = 0usize;
            if matches!(ffi::CGEventGetType(event), 10..=12) {
                ffi::CGEventKeyboardGetUnicodeString(event, text.len(), &mut len, text.as_mut_ptr());
            }
            Self {
                kind: ffi::CGEventGetType(event),
                location: (point.x, point.y),
                button: ffi::CGEventGetIntegerValueField(event, 3),
                number: ffi::CGEventGetIntegerValueField(event, 0),
                click_state: ffi::CGEventGetIntegerValueField(event, 1),
                flags: ffi::CGEventGetFlags(event),
                pointer_delta: (
                    ffi::CGEventGetIntegerValueField(event, 4),
                    ffi::CGEventGetIntegerValueField(event, 5),
                ),
                lines: (
                    ffi::CGEventGetIntegerValueField(event, 12),
                    ffi::CGEventGetIntegerValueField(event, 11),
                ),
                fixed_lines: (
                    ffi::CGEventGetDoubleValueField(event, 94),
                    ffi::CGEventGetDoubleValueField(event, 93),
                ),
                pixels: (
                    ffi::CGEventGetIntegerValueField(event, 97),
                    ffi::CGEventGetIntegerValueField(event, 96),
                ),
                keycode: ffi::CGEventGetIntegerValueField(event, 9),
                user_data: ffi::CGEventGetIntegerValueField(event, 42),
                unicode: text[..len.min(text.len())].to_vec(),
                seed,
            }
        }
    }
}

impl EventPoster for NativePoster {
    fn post(&mut self, event: &SynthEvent) -> Result<()> {
        if !self.inspect && !self.post_allowed {
            return Err(openergy_platform::PlatformError::PermissionDenied(
                openergy_platform::PermissionKind::PostEvents,
            ));
        }
        // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
        unsafe {
            let (native, flags) = match event {
                SynthEvent::Mouse {
                    kind,
                    button,
                    x,
                    y,
                    dx,
                    dy,
                    number,
                    click_state,
                    flags,
                } => {
                    let cg_button = wire_to_cg_button(*button).ok_or_else(|| error("invalid wire mouse button"))?;
                    let kind = mouse_event_type(*kind, cg_button);
                    let native = ffi::CGEventCreateMouseEvent(
                        if self.mode == ClickNumbering::Legacy {
                            ffi::null()
                        } else {
                            self.source.0
                        },
                        kind,
                        ffi::Point { x: *x, y: *y },
                        cg_button,
                    );
                    let owned = OwnedCf::new(native.cast_const(), "CG mouse event")?;
                    ffi::CGEventSetIntegerValueField(native, 1, *click_state);
                    ffi::CGEventSetIntegerValueField(native, 3, i64::from(cg_button));
                    ffi::CGEventSetIntegerValueField(native, 4, dx.round() as i64);
                    ffi::CGEventSetIntegerValueField(native, 5, dy.round() as i64);
                    if let Some(number) = number {
                        ffi::CGEventSetIntegerValueField(native, 0, *number);
                    }
                    (owned, *flags)
                }
                SynthEvent::Keyboard {
                    keycode,
                    down,
                    repeat,
                    unicode,
                    flags,
                } => {
                    let native = ffi::CGEventCreateKeyboardEvent(self.source.0, *keycode, *down);
                    let owned = OwnedCf::new(native.cast_const(), "CG keyboard event")?;
                    ffi::CGEventSetIntegerValueField(native, 8, i64::from(*repeat));
                    if !unicode.is_empty() {
                        ffi::CGEventKeyboardSetUnicodeString(native, unicode.len(), unicode.as_ptr());
                    }
                    (owned, *flags)
                }
                SynthEvent::Wheel {
                    dx_lines,
                    dy_lines,
                    pixels_per_line,
                    flags,
                } => {
                    let native = ffi::CGEventCreateScrollWheelEvent2(self.source.0, 1, 2, *dy_lines, *dx_lines, 0);
                    let owned = OwnedCf::new(native.cast_const(), "CG scroll event")?;
                    ffi::CGEventSetIntegerValueField(native, 88, 0); // discrete line scroll
                    ffi::CGEventSetDoubleValueField(native, 93, f64::from(*dy_lines));
                    ffi::CGEventSetDoubleValueField(native, 94, f64::from(*dx_lines));
                    ffi::CGEventSetIntegerValueField(
                        native,
                        96,
                        (f64::from(*dy_lines) * pixels_per_line).round() as i64,
                    );
                    ffi::CGEventSetIntegerValueField(
                        native,
                        97,
                        (f64::from(*dx_lines) * pixels_per_line).round() as i64,
                    );
                    (owned, *flags)
                }
            };
            ffi::CGEventSetFlags(native.0.cast_mut(), flags);
            ffi::CGEventSetIntegerValueField(native.0.cast_mut(), 42, INJECTION_TAG);
            if self.inspect {
                self.observations
                    .push(NativeEventFields::read(native.0, self.last_seed.get()));
            } else {
                ffi::CGEventPost(0, native.0);
            }
            Ok(())
        }
    }

    fn mouse_counter_seed(&self) -> i64 {
        let seed = [1, 3, 25]
            .into_iter()
            // Safety: reads public HID counters only; no event is posted.
            .map(|kind| i64::from(unsafe { ffi::CGEventSourceCounterForEventType(1, kind) }))
            .sum();
        self.last_seed.set(seed);
        seed
    }

    fn cursor_position(&self) -> Result<Option<(f64, f64)>> {
        let event = OwnedCf::new(
            // Safety: creates a metadata-only snapshot; ownership is checked before use.
            unsafe { ffi::CGEventCreate(ffi::null()).cast_const() },
            "current cursor event",
        )?;
        // Safety: event is an owned live CGEvent for this read.
        let point = unsafe { ffi::CGEventGetLocation(event.0) };
        Ok(Some((point.x, point.y)))
    }

    fn warp(&mut self, x: f64, y: f64) -> Result<()> {
        if self.inspect {
            return Err(error("inspection mode cannot warp"));
        }
        cg_status(
            // Safety: explicit live warp request has finite coordinates; inspection mode already rejected it.
            unsafe { ffi::CGWarpMouseCursorPosition(ffi::Point { x, y }) },
            "cursor warp",
        )?;
        // CINN: association follows the warp, never the normal move path.
        // Safety: explicit CINN warp succeeded; inspection mode cannot reach this call.
        cg_status(
            unsafe { ffi::CGAssociateMouseAndMouseCursorPosition(true) },
            "cursor association",
        )
    }

    fn cursor_visible(&mut self, visible: bool) -> Result<()> {
        if self.inspect {
            return Err(error("inspection mode cannot change cursor visibility"));
        }
        if visible == self.hidden_display.is_none() {
            return Ok(());
        }
        // Safety: public main-display query has no pointer arguments or mutation.
        let display = self.hidden_display.unwrap_or_else(|| unsafe { ffi::CGMainDisplayID() });
        // Safety: explicit live visibility request balances this poster's own cursor hide.
        let status = unsafe {
            if visible {
                ffi::CGDisplayShowCursor(display)
            } else {
                ffi::CGDisplayHideCursor(display)
            }
        };
        cg_status(status, "cursor visibility")?;
        self.hidden_display = if visible { None } else { Some(display) };
        Ok(())
    }
}

fn cg_status(status: i32, name: &str) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(error(format!("{name} failed: {status}")))
    }
}

pub fn wire_to_cg_button(button: u8) -> Option<u32> {
    match button {
        buttons::LEFT => Some(0),
        buttons::MIDDLE => Some(2),
        buttons::RIGHT => Some(1),
        buttons::EXTRA_1 => Some(3),
        buttons::EXTRA_2 => Some(4),
        _ => None,
    }
}

pub fn cg_to_wire_button(button: u32) -> Option<u8> {
    match button {
        0 => Some(buttons::LEFT),
        1 => Some(buttons::RIGHT),
        2 => Some(buttons::MIDDLE),
        3 => Some(buttons::EXTRA_1),
        4 => Some(buttons::EXTRA_2),
        _ => None,
    }
}

fn mouse_event_type(kind: MouseKind, button: u32) -> u32 {
    match kind {
        MouseKind::Move => 5,
        MouseKind::Down => match button {
            0 => 1,
            1 => 3,
            _ => 25,
        },
        MouseKind::Up => match button {
            0 => 2,
            1 => 4,
            _ => 26,
        },
        MouseKind::Drag => match button {
            0 => 6,
            1 => 7,
            _ => 27,
        },
    }
}

pub fn event_type_name(event: &SynthEvent) -> &'static str {
    match event {
        SynthEvent::Mouse { kind, button, .. } => match (kind, wire_to_cg_button(*button).unwrap_or(0)) {
            (MouseKind::Move, _) => "kCGEventMouseMoved",
            (MouseKind::Down, 0) => "kCGEventLeftMouseDown",
            (MouseKind::Down, 1) => "kCGEventRightMouseDown",
            (MouseKind::Down, _) => "kCGEventOtherMouseDown",
            (MouseKind::Up, 0) => "kCGEventLeftMouseUp",
            (MouseKind::Up, 1) => "kCGEventRightMouseUp",
            (MouseKind::Up, _) => "kCGEventOtherMouseUp",
            (MouseKind::Drag, 0) => "kCGEventLeftMouseDragged",
            (MouseKind::Drag, 1) => "kCGEventRightMouseDragged",
            (MouseKind::Drag, _) => "kCGEventOtherMouseDragged",
        },
        SynthEvent::Keyboard { down: true, .. } => "kCGEventKeyDown/FlagsChanged",
        SynthEvent::Keyboard { down: false, .. } => "kCGEventKeyUp/FlagsChanged",
        SynthEvent::Wheel { .. } => "kCGEventScrollWheel",
    }
}

/// Native event adapter over the shared click model. All time/distance,
/// per-button click history, press identity and exhaustion policy live in
/// openergy-platform. Legacy suppresses number assignment only on the CGEvent.
#[derive(Debug, Clone)]
pub struct MouseState {
    pub numbering: ClickNumberer,
    mode: ClickNumbering,
}

impl MouseState {
    pub fn new(mode: ClickNumbering, double_click_interval: f64) -> Result<Self> {
        let milliseconds = monotonic_millis(double_click_interval)?;
        if milliseconds == 0 {
            return Err(error("invalid double-click interval"));
        }
        Ok(Self {
            numbering: ClickNumberer::with_double_click_interval(milliseconds),
            mode,
        })
    }

    fn down(&mut self, button: u8, x: f64, y: f64, now: f64, seed: i64, flags: u64) -> Result<SynthEvent> {
        valid_button(button)?;
        let assignment = self
            .numbering
            .on_down_at(button, seed, monotonic_millis(now)?, x, y)
            .map_err(|err| error(format!("mouse down: {err}")))?;
        Ok(SynthEvent::Mouse {
            kind: MouseKind::Down,
            button,
            x,
            y,
            dx: 0.0,
            dy: 0.0,
            number: (self.mode == ClickNumbering::Native).then_some(assignment.number),
            click_state: assignment.state,
            flags,
        })
    }

    fn motion(&mut self, button: u8, x: f64, y: f64, dx: f64, dy: f64, flags: u64) -> Result<SynthEvent> {
        valid_button(button)?;
        let assignment = self
            .numbering
            .on_drag_at(button, x, y)
            .ok_or_else(|| error("drag without held mouse button"))?;
        Ok(SynthEvent::Mouse {
            kind: MouseKind::Drag,
            button,
            x,
            y,
            dx,
            dy,
            number: (self.mode == ClickNumbering::Native).then_some(assignment.number),
            click_state: assignment.state,
            flags,
        })
    }

    fn up(&mut self, button: u8, x: f64, y: f64, flags: u64) -> Result<SynthEvent> {
        valid_button(button)?;
        // Up can include a final position change. Let the shared model break
        // the next chain, but retain this down's assignment on the up event.
        let assignment = self
            .numbering
            .on_drag_at(button, x, y)
            .ok_or_else(|| error("mouse up without held mouse button"))?;
        self.numbering.on_up_clear(button);
        Ok(SynthEvent::Mouse {
            kind: MouseKind::Up,
            button,
            x,
            y,
            dx: 0.0,
            dy: 0.0,
            number: (self.mode == ClickNumbering::Native).then_some(assignment.number),
            click_state: assignment.state,
            flags,
        })
    }
}

fn valid_button(button: u8) -> Result<()> {
    wire_to_cg_button(button)
        .map(|_| ())
        .ok_or_else(|| error("wire mouse button must be 1..=5"))
}

fn monotonic_millis(seconds: f64) -> Result<u64> {
    let duration =
        Duration::try_from_secs_f64(seconds).map_err(|_| error("invalid mouse timestamp or click interval"))?;
    u64::try_from(duration.as_millis()).map_err(|_| error("mouse timestamp or click interval exceeds u64 milliseconds"))
}

pub fn system_double_click_interval() -> f64 {
    let _pool = ffi::Pool::new();
    // Safety: NSEvent class method returns NSTimeInterval (double) inside a pool.
    unsafe { ffi::msg0(ffi::class(c"NSEvent"), c"doubleClickInterval") }
}

#[derive(Clone, Copy)]
enum Pressed {
    Key(KeyButton),
    Mouse(u8),
    TemporaryKey(u16),
}

#[derive(Clone)]
struct HeldKey {
    key: KeyId,
    keycode: u16,
    unicode: Vec<u16>,
    repeat_unicode: Vec<u16>,
    flags: u64,
}

/// No implicit posting in Drop. Call release_all explicitly before disconnect.
pub struct Injector<P: EventPoster = NativePoster> {
    poster: P,
    keymap: Option<Keymap>,
    mouse: MouseState,
    keys: HashMap<KeyButton, HeldKey>,
    ledger: Vec<Pressed>,
    x: f64,
    y: f64,
    flags: u64,
    lock_flags: u64,
    hide_balance: u32,
    lines_per_notch: f64,
    wheel_remainder: [f64; 2],
    epoch: Instant,
    enter_position: Option<crate::EnterPosition>,
    layout_dirty: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl Injector<NativePoster> {
    pub fn new(mode: ClickNumbering) -> Result<Self> {
        let poster = NativePoster::new(mode)?;
        let keymap = Keymap::new()?;
        // Safety: live native references and checked buffers outlive these SDK-matched public API calls.
        let (point, initial_caps) = unsafe {
            let event = OwnedCf::new(ffi::CGEventCreate(ffi::null()).cast_const(), "current cursor event")?;
            (
                ffi::CGEventGetLocation(event.0),
                ffi::CGEventGetFlags(event.0) & CG_CAPS,
            )
        };
        let mut injector = Self::with_poster(
            poster,
            Some(keymap),
            mode,
            system_double_click_interval(),
            (point.x, point.y),
        )?;
        injector.flags = initial_caps;
        injector.lock_flags = initial_caps;
        Ok(injector)
    }
}

impl<P: EventPoster> Injector<P> {
    pub fn with_poster(
        poster: P,
        keymap: Option<Keymap>,
        mode: ClickNumbering,
        double_click_interval: f64,
        position: (f64, f64),
    ) -> Result<Self> {
        finite_point(position.0, position.1)?;
        Ok(Self {
            poster,
            keymap,
            mouse: MouseState::new(mode, double_click_interval)?,
            keys: HashMap::new(),
            ledger: Vec::new(),
            x: position.0,
            y: position.1,
            flags: 0,
            lock_flags: 0,
            hide_balance: 0,
            lines_per_notch: 3.0,
            wheel_remainder: [0.0; 2],
            epoch: Instant::now(),
            enter_position: None,
            layout_dirty: None,
        })
    }

    pub(crate) fn set_layout_dirty(&mut self, dirty: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        self.layout_dirty = Some(dirty);
    }

    pub(crate) fn set_enter_position(&mut self, position: crate::EnterPosition) {
        self.enter_position = Some(position);
    }

    pub fn poster(&self) -> &P {
        &self.poster
    }
    pub fn poster_mut(&mut self) -> &mut P {
        &mut self.poster
    }
    pub fn mouse_state(&self) -> &MouseState {
        &self.mouse
    }

    pub fn mouse_down(&mut self, button: u8) -> Result<()> {
        self.mouse_down_at(button, self.epoch.elapsed().as_secs_f64())
    }

    /// Explicit monotonic time permits deterministic dry runs without a clock.
    pub fn mouse_down_at(&mut self, button: u8, seconds: f64) -> Result<()> {
        if !seconds.is_finite() {
            return Err(error("invalid click timestamp"));
        }
        let mut next = self.mouse.clone();
        let event = match next.down(
            button,
            self.x,
            self.y,
            seconds,
            self.poster.mouse_counter_seed(),
            self.flags,
        ) {
            Ok(event) => event,
            Err(err) => {
                // The shared model leaves assignments/history unchanged on
                // failure, but retains its one-time exhaustion-log flag.
                self.mouse = next;
                return Err(err);
            }
        };
        self.poster.post(&event)?;
        self.mouse = next;
        self.ledger.push(Pressed::Mouse(button));
        Ok(())
    }

    pub fn mouse_up(&mut self, button: u8) -> Result<()> {
        let mut next = self.mouse.clone();
        let event = next.up(button, self.x, self.y, self.flags)?;
        self.poster.post(&event)?;
        self.mouse = next;
        self.ledger
            .retain(|pressed| !matches!(pressed, Pressed::Mouse(held) if *held == button));
        Ok(())
    }

    pub fn move_absolute(&mut self, x: f64, y: f64) -> Result<()> {
        finite_point(x, y)?;
        let (dx, dy) = (x - self.x, y - self.y);
        finite_point(dx, dy)?;
        let button = self.ledger.iter().rev().find_map(|pressed| match pressed {
            Pressed::Mouse(button) => Some(*button),
            _ => None,
        });
        let mut next = self.mouse.clone();
        let event = if let Some(button) = button {
            next.motion(button, x, y, dx, dy, self.flags)?
        } else {
            SynthEvent::Mouse {
                kind: MouseKind::Move,
                button: buttons::LEFT,
                x,
                y,
                dx,
                dy,
                number: None,
                click_state: 0,
                flags: self.flags,
            }
        };
        self.poster.post(&event)?;
        // Invalidate multi-click chains for ALL held buttons after a drag.
        for button in buttons::LEFT..=buttons::EXTRA_2 {
            next.numbering.on_drag_at(button, x, y);
        }
        self.mouse = next;
        (self.x, self.y) = (x, y);
        Ok(())
    }

    pub fn move_relative(&mut self, dx: f64, dy: f64) -> Result<()> {
        finite_point(dx, dy)?;
        if let Some((x, y)) = self.poster.cursor_position()? {
            finite_point(x, y)?;
            (self.x, self.y) = (x, y);
        }
        self.move_absolute(self.x + dx, self.y + dy)
    }

    pub fn set_lines_per_notch(&mut self, lines: f64) -> Result<()> {
        if !lines.is_finite() || lines <= 0.0 || lines > 1000.0 {
            return Err(error("lines per notch must be in (0, 1000]"));
        }
        self.lines_per_notch = lines;
        Ok(())
    }

    pub fn wheel(&mut self, dx: i32, dy: i32) -> Result<()> {
        let totals = [
            self.wheel_remainder[0] + f64::from(dx) * self.lines_per_notch / f64::from(keys::WHEEL_UNITS_PER_NOTCH),
            self.wheel_remainder[1] + f64::from(dy) * self.lines_per_notch / f64::from(keys::WHEEL_UNITS_PER_NOTCH),
        ];
        if totals
            .iter()
            .any(|value| !value.is_finite() || value.abs() > f64::from(i32::MAX))
        {
            return Err(error("wheel line delta exceeds i32"));
        }
        let lines = [totals[0].trunc() as i32, totals[1].trunc() as i32];
        if lines != [0, 0] {
            self.poster.post(&SynthEvent::Wheel {
                dx_lines: lines[0],
                dy_lines: lines[1],
                pixels_per_line: 10.0,
                flags: self.flags,
            })?;
        }
        self.wheel_remainder = [totals[0] - f64::from(lines[0]), totals[1] - f64::from(lines[1])];
        Ok(())
    }

    pub fn key_down(&mut self, key: KeyId, mask: ModifierMask, button: KeyButton) -> Result<()> {
        if button == 0 {
            return Err(error("source key button zero is reserved"));
        }
        if self.keys.contains_key(&button) {
            return self.key_repeat(button, 1);
        }
        if self
            .layout_dirty
            .as_ref()
            .is_some_and(|dirty| dirty.swap(false, std::sync::atomic::Ordering::AcqRel))
        {
            if let Some(keymap) = &mut self.keymap {
                if let Err(err) = keymap.refresh() {
                    // A transient TIS failure must not permanently consume the
                    // notification and leave later keys on the stale layout.
                    if let Some(dirty) = &self.layout_dirty {
                        dirty.store(true, std::sync::atomic::Ordering::Release);
                    }
                    return Err(err);
                }
            }
        }
        let plan = self
            .keymap
            .as_mut()
            .ok_or_else(|| error("injector has no keymap"))?
            .plan(key, mask)?;
        let baseline = self.flags;
        let result = (|| {
            let held = match plan {
                InjectionPlan::Keycodes(strokes) => {
                    let (last, prefixes) = strokes.split_last().ok_or_else(|| error("empty key plan"))?;
                    for prefix in prefixes {
                        self.set_modifiers(prefix.flags)?;
                        self.tap_key(prefix.keycode, self.flags)?;
                    }
                    if let Some((flag, _)) = keymap::modifier_for_code(last.keycode) {
                        // Momentary modifier keys remain held in the ledger.
                        // Caps is a toggle: key-up must not clear its state.
                        let flags = if flag == CG_CAPS {
                            baseline ^ CG_CAPS
                        } else {
                            baseline | flag
                        };
                        // Appendix B maps both sides to one native modifier.
                        // Reference-count those aliases so releasing one side
                        // cannot release the other's physical native key.
                        if flag == CG_CAPS
                            || !self
                                .keys
                                .values()
                                .any(|other| other.keycode == last.keycode && other.unicode.is_empty())
                        {
                            self.post_key(last.keycode, true, false, &[], flags)?;
                        }
                        if flag == CG_CAPS {
                            self.lock_flags = flags & CG_CAPS;
                        }
                        self.flags = flags;
                        HeldKey {
                            key,
                            keycode: last.keycode,
                            unicode: Vec::new(),
                            repeat_unicode: Vec::new(),
                            flags,
                        }
                    } else {
                        self.set_modifiers(last.flags)?;
                        self.post_key(last.keycode, true, false, &[], self.flags)?;
                        HeldKey {
                            key,
                            keycode: last.keycode,
                            unicode: Vec::new(),
                            // A native repeat of the base key would lose a
                            // completed dead-key composition. Carry the glyph
                            // explicitly on repeats, without replaying a prefix.
                            repeat_unicode: if prefixes.is_empty() { Vec::new() } else { vec![key] },
                            flags: last.flags,
                        }
                    }
                }
                InjectionPlan::Unicode { utf16, flags } => {
                    self.set_modifiers(flags)?;
                    self.post_key(0, true, false, &utf16, self.flags)?;
                    HeldKey {
                        key,
                        keycode: 0,
                        unicode: utf16.clone(),
                        repeat_unicode: utf16,
                        flags,
                    }
                }
            };
            let is_modifier = held.unicode.is_empty() && keymap::modifier_for_code(held.keycode).is_some();
            self.keys.insert(button, held);
            self.ledger.push(Pressed::Key(button));
            if !is_modifier {
                self.set_modifiers(baseline)?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = self.set_modifiers(self.held_modifier_flags());
        }
        result
    }

    /// Repeat uses the original down's mapping, even after a layout switch.
    /// Composed characters repeat with a Unicode payload. Prefixes are not
    /// replayed; key-up always matches the final native key of the down.
    pub fn key_repeat(&mut self, button: KeyButton, count: u16) -> Result<()> {
        let held = self
            .keys
            .get(&button)
            .cloned()
            .ok_or_else(|| error("repeat without key down"))?;
        if held.unicode.is_empty() && keymap::modifier_for_code(held.keycode).is_some() {
            return Ok(());
        }
        let baseline = self.flags;
        let result = (|| {
            let flags = if held.repeat_unicode.is_empty() {
                held.flags
            } else {
                held.flags & !keymap::CG_LAYOUT_FLAGS
            };
            self.set_modifiers(flags)?;
            for _ in 0..count {
                self.post_key(held.keycode, true, true, &held.repeat_unicode, self.flags)?;
            }
            Ok(())
        })();
        let restore = self.set_modifiers(baseline);
        result.and(restore)
    }

    pub fn key_up(&mut self, button: KeyButton) -> Result<()> {
        let Some(held) = self.keys.get(&button).cloned() else {
            return Ok(());
        };
        if held.unicode.is_empty() && keymap::modifier_for_code(held.keycode).is_some() {
            let mut desired = self.lock_flags;
            for (source, other) in &self.keys {
                if *source != button && other.unicode.is_empty() {
                    if let Some((flag, _)) = keymap::modifier_for_code(other.keycode) {
                        desired |= flag & !CG_CAPS;
                    }
                }
            }
            if !self
                .keys
                .iter()
                .any(|(source, other)| *source != button && other.keycode == held.keycode && other.unicode.is_empty())
            {
                self.post_key(held.keycode, false, false, &[], desired)?;
            }
            self.flags = desired;
        } else {
            let baseline = self.flags;
            self.set_modifiers(held.flags)?;
            let result = self.post_key(held.keycode, false, false, &held.unicode, self.flags);
            let restore = self.set_modifiers(baseline);
            result.and(restore)?;
        }
        self.keys.remove(&button);
        self.ledger
            .retain(|pressed| !matches!(pressed, Pressed::Key(held) if *held == button));
        Ok(())
    }

    /// Number of outstanding synthetic downs (keys, buttons, temporary taps).
    pub fn held_count(&self) -> usize {
        self.ledger.len()
    }

    pub fn held_key(&self, button: KeyButton) -> Option<KeyId> {
        self.keys.get(&button).map(|held| held.key)
    }

    fn held_modifier_flags(&self) -> u64 {
        self.keys
            .values()
            .filter(|held| held.unicode.is_empty())
            .filter_map(|held| keymap::modifier_for_code(held.keycode))
            .fold(self.lock_flags, |flags, (flag, _)| flags | (flag & !CG_CAPS))
    }

    fn post_key(&mut self, code: u16, down: bool, repeat: bool, unicode: &[u16], flags: u64) -> Result<()> {
        self.poster.post(&SynthEvent::Keyboard {
            keycode: code,
            down,
            repeat,
            unicode: unicode.to_vec(),
            flags,
        })
    }

    fn tap_key(&mut self, code: u16, flags: u64) -> Result<()> {
        self.post_key(code, true, false, &[], flags)?;
        self.flags = flags;
        self.ledger.push(Pressed::TemporaryKey(code));
        self.post_key(code, false, false, &[], flags)?;
        self.ledger.pop();
        Ok(())
    }

    fn set_modifiers(&mut self, target: u64) -> Result<()> {
        // Release incompatible layers before pressing required ones; restore
        // in reverse order. This state covers synthetic modifiers only.
        const MODIFIERS: [(u64, u16); 4] = [
            (CG_SHIFT, 0x38),
            (CG_CONTROL, 0x3B),
            (CG_OPTION, 0x3A),
            (CG_COMMAND, 0x37),
        ];
        for (flag, code) in MODIFIERS.into_iter().rev() {
            if self.flags & flag != 0 && target & flag == 0 {
                let flags = self.flags & !flag;
                self.post_key(code, false, false, &[], flags)?;
                self.flags = flags;
            }
        }
        if (self.flags ^ target) & CG_CAPS != 0 {
            let flags = (self.flags & !CG_CAPS) | (target & CG_CAPS);
            self.tap_key(0x39, flags)?;
        }
        for (flag, code) in MODIFIERS {
            if self.flags & flag == 0 && target & flag != 0 {
                let flags = self.flags | flag;
                self.post_key(code, true, false, &[], flags)?;
                self.flags = flags;
            }
        }
        Ok(())
    }

    /// Release successful downs in reverse arrival order. Continue after an
    /// error; failed entries remain available for a later recovery attempt.
    pub fn release_all(&mut self) -> Result<()> {
        let mut first_error = None;
        for pressed in self.ledger.clone().into_iter().rev() {
            let result = match pressed {
                Pressed::Key(button) => self.key_up(button),
                Pressed::Mouse(button) => self.mouse_up(button),
                Pressed::TemporaryKey(code) => {
                    let result = self.post_key(code, false, false, &[], self.flags);
                    if result.is_ok() {
                        if let Some(index) = self
                            .ledger
                            .iter()
                            .rposition(|pressed| matches!(pressed, Pressed::TemporaryKey(held) if *held == code))
                        {
                            self.ledger.remove(index);
                        }
                    }
                    result
                }
            };
            if let Err(err) = result {
                first_error.get_or_insert(err);
            }
        }
        if let Err(err) = self.set_modifiers(self.held_modifier_flags()) {
            first_error.get_or_insert(err);
        }
        while self.hide_balance > 0 {
            if let Err(err) = self.show_cursor() {
                first_error.get_or_insert(err);
                break;
            }
        }
        self.wheel_remainder = [0.0; 2];
        self.mouse.numbering.reset_click_history();
        first_error.map_or(Ok(()), Err)
    }

    pub fn warp_cursor(&mut self, x: f64, y: f64) -> Result<()> {
        finite_point(x, y)?;
        self.poster.warp(x, y)?;
        (self.x, self.y) = (x, y);
        Ok(())
    }

    /// Nesting is local: one native hide on 0→1, one show on 1→0.
    pub fn hide_cursor(&mut self) -> Result<()> {
        let next = self
            .hide_balance
            .checked_add(1)
            .ok_or_else(|| error("cursor hide balance overflow"))?;
        if self.hide_balance == 0 {
            self.poster.cursor_visible(false)?;
        }
        self.hide_balance = next;
        Ok(())
    }

    pub fn show_cursor(&mut self) -> Result<()> {
        if self.hide_balance == 0 {
            return Ok(());
        }
        if self.hide_balance == 1 {
            self.poster.cursor_visible(true)?;
        }
        self.hide_balance -= 1;
        Ok(())
    }
}

fn finite_point(x: f64, y: f64) -> Result<()> {
    if x.is_finite() && y.is_finite() {
        Ok(())
    } else {
        Err(error("mouse coordinates must be finite"))
    }
}

impl InputSink for Injector<NativePoster> {
    fn permissions_changed(&mut self, permissions: &openergy_platform::Permissions) {
        self.poster.post_allowed = permissions.post_events == openergy_platform::Grant::Granted;
    }
    fn inject(&mut self, ev: &InputEvent) -> Result<()> {
        if let Some(position) = &self.enter_position {
            if let Some((x, y)) = position.lock().map_err(|_| error("enter position poisoned"))?.take() {
                self.x = f64::from(x);
                self.y = f64::from(y);
            }
        }
        match *ev {
            InputEvent::KeyDown { key, modifiers, button } => self.key_down(key, modifiers.0, button),
            InputEvent::KeyRepeat { count, button, .. } => {
                if count < 0 {
                    return Err(error("negative key repeat count"));
                }
                self.key_repeat(button, count as u16)
            }
            InputEvent::KeyUp { button, .. } => self.key_up(button),
            InputEvent::MouseDown { button } if wire_to_cg_button(button).is_some() => self.mouse_down(button),
            InputEvent::MouseUp { button } if wire_to_cg_button(button).is_some() => {
                // Failed downs and duplicate cleanup ups are harmless.
                if self
                    .ledger
                    .iter()
                    .any(|p| matches!(p, Pressed::Mouse(b) if *b == button))
                {
                    self.mouse_up(button)
                } else {
                    Ok(())
                }
            }
            InputEvent::MouseDown { .. } | InputEvent::MouseUp { .. } => Ok(()),
            InputEvent::MouseMoveAbs { x, y } => self.move_absolute(f64::from(x), f64::from(y)),
            InputEvent::MouseMoveRel { dx, dy } => self.move_relative(f64::from(dx), f64::from(dy)),
            InputEvent::MouseWheel { dx, dy } => self.wheel(dx, dy),
        }
    }
    fn release_all(&mut self) {
        if self.poster.inspect {
            return;
        }
        if self.release_all().is_err() {
            log::error!("macOS input release failed; ledger retained for retry");
        }
    }
    fn set_cursor_visible(&mut self, visible: bool) {
        self.poster.post_allowed = crate::permissions::preflight().post;
        let result = if visible {
            // Leave force-shows all hides owned by this injector, without
            // unbalancing a cursor hide owned by another process.
            while self.hide_balance > 1 {
                self.hide_balance -= 1;
            }
            self.show_cursor()
        } else {
            self.hide_cursor()
        };
        if result.is_err() {
            log::error!("macOS cursor visibility failed");
        }
    }
}
