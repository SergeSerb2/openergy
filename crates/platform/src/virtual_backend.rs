//! Product headless backend. No platform APIs, network, files, or native input.
use crate::{
    CaptureDecision, ClickNumberer, ClipboardBackend, ClipboardData, Grant, InputEvent, InputSink, InputSource,
    PermissionKind, Permissions, PlatformError, RecordedEvent, ScreenRect, ScreenSystem,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard, Weak};
use std::thread::{self, JoinHandle, ThreadId};
use std::time::Instant;

type CaptureCallback = Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>;
type DisplayCallback = Box<dyn FnMut() + Send>;

/// Initial virtual screen geometry and click policy supplied by the daemon.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VirtualOptions {
    pub screen_w: i32,
    pub screen_h: i32,
    pub hid_seed: i64,
    pub double_click_ms: u64,
}
impl Default for VirtualOptions {
    fn default() -> Self {
        Self {
            screen_w: 1920,
            screen_h: 1080,
            hid_seed: 0,
            double_click_ms: 500,
        }
    }
}

/// Construct the independently owned backend parts. Nonpositive initial screen
/// sizes become one point; use set_screens for strict geometry validation.
pub fn create(options: VirtualOptions) -> (crate::Backend, VirtualControl) {
    let config = VirtualConfig {
        double_click_ms: options.double_click_ms,
        ..VirtualConfig::default()
    };
    split(VirtualPlatform::create(config), options)
}

/// Fallible constructor when a caller needs custom ring and pressed limits.
/// The options' click interval takes precedence over config.double_click_ms.
pub fn create_with_config(
    options: VirtualOptions,
    mut config: VirtualConfig,
) -> Result<(crate::Backend, VirtualControl), PlatformError> {
    config.double_click_ms = options.double_click_ms;
    Ok(split(VirtualPlatform::with_config(config)?, options))
}
fn split(platform: VirtualPlatform, options: VirtualOptions) -> (crate::Backend, VirtualControl) {
    {
        let mut state = lock(&platform.shared.state);
        state.screens[0].w = options.screen_w.max(1);
        state.screens[0].h = options.screen_h.max(1);
        state.hid_seed = options.hid_seed;
    }
    let control = platform.control();
    let backend = crate::Backend {
        name: "virtual",
        sink: Box::new(platform.clone()),
        screens: Box::new(platform.clone()),
        clipboard: Box::new(platform.clipboard.clone()),
        source: Box::new(platform),
    };
    (backend, control)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VirtualConfig {
    pub ledger_capacity: usize,
    pub pressed_capacity: usize,
    pub double_click_ms: u64,
    pub max_clipboard_bytes: usize,
}
impl Default for VirtualConfig {
    fn default() -> Self {
        Self {
            ledger_capacity: 4096,
            pressed_capacity: 512,
            double_click_ms: 500,
            max_clipboard_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedDecision {
    PassToOs,
    Swallow,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordKind {
    Sink,
    Capture {
        decision: RecordedDecision,
    },
    ReleaseAll,
    CursorCaptured {
        captured: bool,
    },
    CursorWarp {
        x: i32,
        y: i32,
    },
    CursorVisible {
        visible: bool,
    },
    ClipboardWrite {
        stamp: u64,
        own_write: bool,
        bytes: usize,
    },
    PermissionRequest {
        permission: PermissionKind,
        previous: Grant,
        grant: Grant,
    },
}
/// Additive wrapper; the Step 0 RecordedEvent type remains unchanged.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VirtualRecord {
    #[serde(flatten)]
    pub recorded: RecordedEvent,
    pub timestamp_us: u64,
    pub detail: RecordKind,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerSnapshot {
    pub events: Vec<VirtualRecord>,
    pub stats: VirtualStats,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualStats {
    pub last_sequence: u64,
    pub oldest_sequence: Option<u64>,
    pub dropped: u64,
    pub sequence_exhausted: bool,
    pub ledger_len: usize,
    pub ledger_capacity: usize,
    pub ledger_reserved_bytes: usize,
    pub pressed_keys: usize,
    pub pressed_buttons: usize,
    pub cursor: [i32; 2],
    pub cursor_captured: bool,
    pub cursor_visible: bool,
    pub prevent_sleep: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardSnapshot {
    pub data: ClipboardData,
    pub stamp: u64,
    pub own_write: bool,
    pub last_own_write_stamp: Option<u64>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CaptureReport {
    pub delivered: usize,
    pub passed: usize,
    pub swallowed: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRequest {
    #[serde(default, alias = "since_seq")]
    pub since: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreensRequest {
    pub rects: Vec<ScreenRect>,
}

/// Flat virtual.set_permissions params, with optional one-shot prompt results.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsRequest {
    pub accessibility: Grant,
    pub input_monitoring: Grant,
    pub post_events: Grant,
    #[serde(default)]
    pub on_request: Option<Permissions>,
}

fn permission_index(kind: PermissionKind) -> usize {
    match kind {
        PermissionKind::Accessibility => 0,
        PermissionKind::InputMonitoring => 1,
        PermissionKind::PostEvents => 2,
    }
}
fn permission(permissions: &Permissions, kind: PermissionKind) -> Grant {
    match kind {
        PermissionKind::Accessibility => permissions.accessibility,
        PermissionKind::InputMonitoring => permissions.input_monitoring,
        PermissionKind::PostEvents => permissions.post_events,
    }
}
fn assign_permission(permissions: &mut Permissions, kind: PermissionKind, grant: Grant) {
    match kind {
        PermissionKind::Accessibility => permissions.accessibility = grant,
        PermissionKind::InputMonitoring => permissions.input_monitoring = grant,
        PermissionKind::PostEvents => permissions.post_events = grant,
    }
}

/// Clone to give capture, sink, and clipboard workers independent trait objects.
/// Only the instance that calls start owns its capture worker; dropping that
/// instance stops and joins it. Clones and VirtualControl share all OS state.
pub struct VirtualPlatform {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    clipboard: VirtualClipboard,
    display_registration: Option<u64>,
}
impl Clone for VirtualPlatform {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            worker: None,
            clipboard: self.clipboard.clone(),
            display_registration: None,
        }
    }
}
#[derive(Clone)]
pub struct VirtualControl {
    shared: Arc<Shared>,
}
#[derive(Clone)]
pub struct VirtualClipboard {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,
    capture: Mutex<Option<CaptureEndpoint>>,
    display_callback: Mutex<DisplaySlot>,
}
#[derive(Default)]
struct DisplaySlot {
    next_id: u64,
    owner_id: Option<u64>,
    callback: Option<DisplayCallback>,
}
#[derive(Clone)]
struct CaptureEndpoint {
    sender: mpsc::SyncSender<CaptureCommand>,
    thread_id: ThreadId,
    running: Arc<AtomicBool>,
}
enum CaptureCommand {
    Events(Vec<InputEvent>, mpsc::SyncSender<Result<CaptureReport, PlatformError>>),
    Stop,
}
#[derive(Clone, Copy)]
enum Press {
    Key {
        key: u16,
        modifiers: crate::Modifiers,
        button: u16,
    },
    Mouse {
        button: u8,
    },
}
struct State {
    start: Instant,
    ledger: VecDeque<VirtualRecord>,
    capacity: usize,
    sequence: u64,
    dropped: u64,
    sequence_exhausted: bool,
    pressed: Vec<Press>,
    pressed_capacity: usize,
    released: bool,
    click: ClickNumberer,
    hid_seed: i64,
    click_now_ms: Option<u64>,
    screens: Vec<ScreenRect>,
    display_generation: u64,
    permissions: Permissions,
    permission_request_results: [Option<Grant>; 3],
    secure_input: Option<String>,
    cursor: [i32; 2],
    cursor_captured: bool,
    cursor_visible: bool,
    prevent_sleep: bool,
    clipboard: ClipboardData,
    clipboard_stamp: u64,
    own_write_stamp: Option<u64>,
    max_clipboard_bytes: usize,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn failed(message: &str) -> PlatformError {
    PlatformError::Failed(message.to_owned())
}
fn validate_event(event: &InputEvent) -> Result<(), PlatformError> {
    match event {
        InputEvent::MouseDown { button } | InputEvent::MouseUp { button } if !(1..=7).contains(button) => {
            Err(failed("mouse button must be in 1..=7"))
        }
        InputEvent::KeyRepeat { count, .. } if *count <= 0 => Err(failed("repeat count must be positive")),
        _ => Ok(()),
    }
}
fn require(grant: Grant, kind: PermissionKind) -> Result<(), PlatformError> {
    if matches!(grant, Grant::Denied | Grant::Unknown) {
        Err(PlatformError::PermissionDenied(kind))
    } else {
        Ok(())
    }
}
impl State {
    fn capture_permission(&self) -> Result<(), PlatformError> {
        require(self.permissions.accessibility, PermissionKind::Accessibility)?;
        require(self.permissions.input_monitoring, PermissionKind::InputMonitoring)
    }
    fn sink_permission(&self) -> Result<(), PlatformError> {
        require(self.permissions.accessibility, PermissionKind::Accessibility)?;
        require(self.permissions.post_events, PermissionKind::PostEvents)
    }
    fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
    }
    fn record(&mut self, event: Option<InputEvent>, detail: RecordKind, click: Option<crate::ClickAssignment>) {
        let Some(sequence) = self.sequence.checked_add(1) else {
            self.sequence_exhausted = true;
            self.dropped = self.dropped.saturating_add(1);
            return;
        };
        self.sequence = sequence;
        if self.capacity == 0 {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        if self.ledger.len() == self.capacity {
            self.ledger.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.ledger.push_back(VirtualRecord {
            recorded: RecordedEvent {
                sequence,
                event,
                release_all: matches!(detail, RecordKind::ReleaseAll),
                click_number: click.map(|value| value.number),
                click_state: click.map(|value| value.state),
            },
            timestamp_us: self.start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64,
            detail,
        });
    }
    fn move_cursor(&mut self, event: &InputEvent) {
        match *event {
            InputEvent::MouseMoveAbs { x, y } => self.cursor = [x, y],
            InputEvent::MouseMoveRel { dx, dy } => {
                self.cursor[0] = self.cursor[0].saturating_add(dx);
                self.cursor[1] = self.cursor[1].saturating_add(dy);
            }
            _ => {}
        }
    }
    fn stats(&self) -> VirtualStats {
        VirtualStats {
            last_sequence: self.sequence,
            oldest_sequence: self.ledger.front().map(|entry| entry.recorded.sequence),
            dropped: self.dropped,
            sequence_exhausted: self.sequence_exhausted,
            ledger_len: self.ledger.len(),
            ledger_capacity: self.capacity,
            ledger_reserved_bytes: self.ledger.capacity() * std::mem::size_of::<VirtualRecord>(),
            pressed_keys: self
                .pressed
                .iter()
                .filter(|press| matches!(press, Press::Key { .. }))
                .count(),
            pressed_buttons: self
                .pressed
                .iter()
                .filter(|press| matches!(press, Press::Mouse { .. }))
                .count(),
            cursor: self.cursor,
            cursor_captured: self.cursor_captured,
            cursor_visible: self.cursor_visible,
            prevent_sleep: self.prevent_sleep,
        }
    }
    fn press_index(&self, event: &InputEvent) -> Option<usize> {
        self.pressed.iter().position(|press| match (*press, event) {
            (
                Press::Key { key, button, .. },
                InputEvent::KeyDown {
                    key: other_key,
                    button: other_button,
                    ..
                }
                | InputEvent::KeyUp {
                    key: other_key,
                    button: other_button,
                    ..
                },
            ) => {
                if button != 0 || *other_button != 0 {
                    button == *other_button
                } else {
                    key == *other_key
                }
            }
            (
                Press::Mouse { button },
                InputEvent::MouseDown { button: other } | InputEvent::MouseUp { button: other },
            ) => button == *other,
            _ => false,
        })
    }
    fn inject_sink(&mut self, event: &InputEvent) -> Result<(), PlatformError> {
        validate_event(event)?;
        self.sink_permission()?;
        let index = self.press_index(event);
        let down = matches!(event, InputEvent::KeyDown { .. } | InputEvent::MouseDown { .. });
        if down && index.is_none() && self.pressed.len() == self.pressed_capacity {
            return Err(failed("pressed ledger capacity exceeded"));
        }
        let click = match *event {
            InputEvent::MouseDown { button } => {
                let now = self.click_now_ms.unwrap_or_else(|| self.elapsed_ms());
                Some(
                    self.click
                        .on_down_at(
                            button,
                            self.hid_seed,
                            now,
                            f64::from(self.cursor[0]),
                            f64::from(self.cursor[1]),
                        )
                        .map_err(|error| PlatformError::Failed(error.to_string()))?,
                )
            }
            InputEvent::MouseUp { button } => {
                let assignment = self.click.assignment(button);
                self.click.on_up_clear(button);
                assignment
            }
            InputEvent::MouseMoveAbs { .. } | InputEvent::MouseMoveRel { .. } => {
                self.move_cursor(event);
                let mut active = None;
                for button in 1..=7 {
                    if let Some(assignment) =
                        self.click
                            .on_drag_at(button, f64::from(self.cursor[0]), f64::from(self.cursor[1]))
                    {
                        // One protocol move represents all held buttons. Expose the
                        // most recent press; per-button assignments remain queryable.
                        if active.is_none_or(|old: crate::ClickAssignment| old.number < assignment.number) {
                            active = Some(assignment);
                        }
                    }
                }
                active
            }
            _ => None,
        };
        if down && index.is_none() {
            self.pressed.push(match *event {
                InputEvent::KeyDown { key, modifiers, button } => Press::Key { key, modifiers, button },
                InputEvent::MouseDown { button } => Press::Mouse { button },
                _ => return Err(failed("invalid press event")),
            });
        } else if matches!(event, InputEvent::KeyUp { .. } | InputEvent::MouseUp { .. }) {
            if let Some(index) = index {
                self.pressed.remove(index);
            }
        }
        self.released = false;
        self.record(Some(event.clone()), RecordKind::Sink, click);
        Ok(())
    }
    fn validate_clipboard(&self, data: &ClipboardData) -> Result<(), PlatformError> {
        if data.formats.len() > 4 {
            return Err(failed("clipboard allows at most four distinct format kinds"));
        }
        let mut bytes = 0_usize;
        for (index, format) in data.formats.iter().enumerate() {
            if data.formats[..index]
                .iter()
                .any(|previous| previous.kind == format.kind)
            {
                return Err(failed("duplicate clipboard format kind"));
            }
            bytes = bytes
                .checked_add(format.bytes.len())
                .ok_or_else(|| failed("clipboard size overflow"))?;
        }
        if bytes > self.max_clipboard_bytes {
            return Err(failed("clipboard exceeds virtual backend byte limit"));
        }
        Ok(())
    }
    fn clipboard_set(&mut self, data: ClipboardData, own_write: bool) -> Result<u64, PlatformError> {
        self.validate_clipboard(&data)?;
        let stamp = self
            .clipboard_stamp
            .checked_add(1)
            .ok_or_else(|| failed("clipboard stamp exhausted"))?;
        let bytes = data.total_bytes();
        self.clipboard = data;
        self.clipboard_stamp = stamp;
        if own_write {
            self.own_write_stamp = Some(stamp);
        }
        self.record(
            None,
            RecordKind::ClipboardWrite {
                stamp,
                own_write,
                bytes,
            },
            None,
        );
        Ok(stamp)
    }
}

impl Default for VirtualPlatform {
    fn default() -> Self {
        Self::new()
    }
}
impl VirtualPlatform {
    pub fn new() -> Self {
        Self::create(VirtualConfig::default())
    }
    pub fn with_config(config: VirtualConfig) -> Result<Self, PlatformError> {
        if config.ledger_capacity > 65_536 || config.pressed_capacity == 0 || config.pressed_capacity > 4096 {
            return Err(failed(
                "ledger capacity must be 0..=65536; pressed capacity must be 1..=4096",
            ));
        }
        Ok(Self::create(config))
    }
    fn create(config: VirtualConfig) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                start: Instant::now(),
                ledger: VecDeque::with_capacity(config.ledger_capacity),
                capacity: config.ledger_capacity,
                sequence: 0,
                dropped: 0,
                sequence_exhausted: false,
                pressed: Vec::with_capacity(config.pressed_capacity),
                pressed_capacity: config.pressed_capacity,
                released: false,
                click: ClickNumberer::with_double_click_interval(config.double_click_ms),
                hid_seed: 0,
                click_now_ms: None,
                screens: vec![ScreenRect {
                    x: 0,
                    y: 0,
                    w: 1920,
                    h: 1080,
                    scale: 1.0,
                }],
                display_generation: 0,
                permissions: Permissions::not_required(),
                permission_request_results: [None; 3],
                secure_input: None,
                cursor: [0, 0],
                cursor_captured: false,
                cursor_visible: true,
                prevent_sleep: false,
                clipboard: ClipboardData::default(),
                clipboard_stamp: 0,
                own_write_stamp: None,
                max_clipboard_bytes: config.max_clipboard_bytes,
            }),
            capture: Mutex::new(None),
            display_callback: Mutex::new(DisplaySlot::default()),
        });
        Self {
            clipboard: VirtualClipboard {
                shared: Arc::clone(&shared),
            },
            shared,
            worker: None,
            display_registration: None,
        }
    }
    pub fn control(&self) -> VirtualControl {
        VirtualControl {
            shared: Arc::clone(&self.shared),
        }
    }
}
impl Drop for VirtualPlatform {
    fn drop(&mut self) {
        self.stop();
        if let Some(owner_id) = self.display_registration.take() {
            let callback = {
                let mut slot = lock(&self.shared.display_callback);
                if slot.owner_id == Some(owner_id) {
                    slot.owner_id = None;
                    slot.callback.take()
                } else {
                    None
                }
            };
            // A callback can own a control handle. Drop it outside locks to break
            // that reference cycle when its ScreenSystem owner goes away.
            drop(callback);
        }
    }
}

impl VirtualControl {
    /// Synchronous completion barrier: all callbacks and capture audit records
    /// for this batch exist when this returns. Never call from a capture callback.
    pub fn inject(&self, events: Vec<InputEvent>) -> Result<CaptureReport, PlatformError> {
        if events.len() > 4096 {
            return Err(failed("capture batch exceeds 4096 events"));
        }
        for event in &events {
            validate_event(event)?;
        }
        lock(&self.shared.state).capture_permission()?;
        let endpoint = lock(&self.shared.capture)
            .clone()
            .ok_or_else(|| failed("capture is not running"))?;
        if !endpoint.running.load(Ordering::Acquire) {
            return Err(failed("capture is stopping"));
        }
        if endpoint.thread_id == thread::current().id() {
            return Err(failed("capture injection cannot re-enter its callback"));
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        endpoint
            .sender
            .send(CaptureCommand::Events(events, sender))
            .map_err(|_| failed("capture stopped"))?;
        receiver
            .recv()
            .map_err(|_| failed("capture stopped before completing batch"))?
    }
    /// Frozen sink records only, strictly after since_seq (exclusive).
    pub fn recorded(&self, since_seq: u64) -> Vec<RecordedEvent> {
        lock(&self.shared.state)
            .ledger
            .iter()
            .filter(|entry| {
                entry.recorded.sequence > since_seq && matches!(entry.detail, RecordKind::Sink | RecordKind::ReleaseAll)
            })
            .map(|entry| entry.recorded.clone())
            .collect()
    }
    /// Full audit with monotonic timestamps, markers, and overflow information.
    pub fn ledger(&self, since_seq: u64) -> LedgerSnapshot {
        let state = lock(&self.shared.state);
        LedgerSnapshot {
            events: state
                .ledger
                .iter()
                .filter(|entry| entry.recorded.sequence > since_seq)
                .cloned()
                .collect(),
            stats: state.stats(),
        }
    }
    pub fn stats(&self) -> VirtualStats {
        lock(&self.shared.state).stats()
    }
    pub fn click_assignment(&self, button: u8) -> Option<crate::ClickAssignment> {
        lock(&self.shared.state).click.assignment(button)
    }
    pub fn clipboard_get(&self) -> ClipboardSnapshot {
        let state = lock(&self.shared.state);
        ClipboardSnapshot {
            data: state.clipboard.clone(),
            stamp: state.clipboard_stamp,
            own_write: state.own_write_stamp == Some(state.clipboard_stamp),
            last_own_write_stamp: state.own_write_stamp,
        }
    }
    /// Script a local/external clipboard change; this is NOT an own write.
    pub fn clipboard_set(&self, data: ClipboardData) -> Result<u64, PlatformError> {
        lock(&self.shared.state).clipboard_set(data, false)
    }
    pub fn set_screens(&self, screens: Vec<ScreenRect>) -> Result<(), PlatformError> {
        if screens.len() > 64
            || screens.iter().any(|rect| {
                rect.w <= 0
                    || rect.h <= 0
                    || !rect.scale.is_finite()
                    || rect.scale <= 0.0
                    || rect.x.checked_add(rect.w).is_none()
                    || rect.y.checked_add(rect.h).is_none()
            })
        {
            return Err(failed(
                "invalid screen geometry (maximum 64 finite, positive rectangles)",
            ));
        }
        {
            let mut state = lock(&self.shared.state);
            if state.screens == screens {
                return Ok(());
            }
            state.screens = screens;
            state.display_generation = state.display_generation.saturating_add(1);
        }
        // Take the callback out before calling user code. Reentrant and concurrent
        // updates coalesce by generation; callbacks may query or change this backend.
        let (callback, owner_id) = {
            let mut slot = lock(&self.shared.display_callback);
            (slot.callback.take(), slot.owner_id)
        };
        let Some(mut callback) = callback else {
            return Ok(());
        };
        loop {
            let generation = lock(&self.shared.state).display_generation;
            callback();
            let state = lock(&self.shared.state);
            let mut slot = lock(&self.shared.display_callback);
            if slot.owner_id != owner_id || slot.callback.is_some() {
                break; // Registration changed, or its owning ScreenSystem was dropped.
            }
            if state.display_generation == generation {
                slot.callback = Some(callback);
                break;
            }
        }
        Ok(())
    }
    pub fn set_permissions(&self, permissions: Permissions) {
        let mut state = lock(&self.shared.state);
        state.permissions = permissions;
        state.permission_request_results = [None; 3];
    }
    /// Apply flat virtual.set_permissions params and optional one-shot prompt results.
    pub fn configure_permissions(&self, request: PermissionsRequest) {
        let mut state = lock(&self.shared.state);
        state.permissions = Permissions {
            accessibility: request.accessibility,
            input_monitoring: request.input_monitoring,
            post_events: request.post_events,
        };
        state.permission_request_results = request.on_request.map_or([None; 3], |grants| {
            [
                Some(grants.accessibility),
                Some(grants.input_monitoring),
                Some(grants.post_events),
            ]
        });
    }
    /// Script one permission's next request. None restores current-state behavior.
    pub fn set_permission_request_result(&self, kind: PermissionKind, grant: Option<Grant>) {
        lock(&self.shared.state).permission_request_results[permission_index(kind)] = grant;
    }
    /// None disables secure input; Some(name) reports its scripted owner.
    pub fn set_secure_input(&self, app: Option<String>) {
        lock(&self.shared.state).secure_input = app;
    }
    /// Set virtual numbering time and policy. None keeps the current value.
    /// All values are validated before any state changes.
    pub fn set_numbering_controls(
        &self,
        now_ms: Option<u64>,
        click_interval_ms: Option<u64>,
        click_distance: Option<f64>,
    ) -> Result<(), PlatformError> {
        if click_interval_ms.is_some_and(|n| n > 60_000)
            || click_distance.is_some_and(|n| !n.is_finite() || !(0.0..=1_000_000.0).contains(&n))
        {
            return Err(failed("invalid click interval or distance"));
        }
        let mut state = lock(&self.shared.state);
        if let Some(now) = now_ms {
            state.click_now_ms = Some(now);
        }
        if let Some(interval) = click_interval_ms {
            state.click.double_click_ms = interval;
        }
        if let Some(distance) = click_distance {
            state.click.click_distance = distance;
        }
        Ok(())
    }
    /// The simulated sum of HID counters, re-read on every sink down.
    pub fn set_hid_seed(&self, seed: i64) {
        lock(&self.shared.state).hid_seed = seed;
    }
}

fn capture_loop(
    shared: Weak<Shared>,
    receiver: mpsc::Receiver<CaptureCommand>,
    running: Arc<AtomicBool>,
    mut callback: CaptureCallback,
) {
    while let Ok(command) = receiver.recv() {
        if !running.load(Ordering::Acquire) {
            break;
        }
        let CaptureCommand::Events(events, reply) = command else {
            break;
        };
        let Some(shared) = shared.upgrade() else {
            break;
        };
        let mut report = CaptureReport::default();
        let mut outcome = Ok(());
        for event in events {
            if !running.load(Ordering::Acquire) {
                outcome = Err(failed("capture stopped during batch"));
                break;
            }
            if let Err(error) = lock(&shared.state).capture_permission() {
                outcome = Err(error);
                break;
            }
            let decision = callback(event.clone());
            let mut state = lock(&shared.state);
            let decision = match decision {
                CaptureDecision::PassToOs => {
                    report.passed += 1;
                    if !state.cursor_captured {
                        state.move_cursor(&event);
                    }
                    RecordedDecision::PassToOs
                }
                CaptureDecision::Swallow => {
                    report.swallowed += 1;
                    RecordedDecision::Swallow
                }
            };
            state.record(Some(event), RecordKind::Capture { decision }, None);
            report.delivered += 1;
        }
        let _ = reply.send(outcome.map(|()| report));
    }
}

impl InputSource for VirtualPlatform {
    fn start(&mut self, sink: CaptureCallback) -> Result<(), PlatformError> {
        lock(&self.shared.state).capture_permission()?;
        let mut endpoint = lock(&self.shared.capture);
        if endpoint.is_some() {
            return Err(failed("capture is already running"));
        }
        let (sender, receiver) = mpsc::sync_channel(16);
        let shared = Arc::downgrade(&self.shared);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let worker = thread::Builder::new()
            .name("openergy-virtual-capture".into())
            .spawn(move || capture_loop(shared, receiver, worker_running, sink))
            .map_err(|error| PlatformError::Failed(error.to_string()))?;
        *endpoint = Some(CaptureEndpoint {
            sender,
            thread_id: worker.thread().id(),
            running,
        });
        self.worker = Some(worker);
        Ok(())
    }
    fn stop(&mut self) {
        let Some(worker) = self.worker.take() else {
            return;
        };
        let endpoint = lock(&self.shared.capture).clone();
        if let Some(endpoint) = endpoint.filter(|value| value.thread_id == worker.thread().id()) {
            endpoint.running.store(false, Ordering::Release);
            // If full, the next queued command wakes the worker and sees running=false.
            let _ = endpoint.sender.try_send(CaptureCommand::Stop);
        }
        let thread_id = worker.thread().id();
        if thread_id != thread::current().id() {
            let _ = worker.join();
        }
        let mut endpoint = lock(&self.shared.capture);
        if endpoint.as_ref().is_some_and(|value| value.thread_id == thread_id) {
            *endpoint = None;
        }
        let mut state = lock(&self.shared.state);
        if state.cursor_captured {
            state.cursor_captured = false;
            state.cursor_visible = true;
            state.record(None, RecordKind::CursorCaptured { captured: false }, None);
        }
    }
    fn set_cursor_captured(&mut self, captured: bool) -> Result<(), PlatformError> {
        let mut state = lock(&self.shared.state);
        if captured {
            state.capture_permission()?;
        }
        state.cursor_captured = captured;
        state.cursor_visible = !captured;
        state.record(None, RecordKind::CursorCaptured { captured }, None);
        Ok(())
    }
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<(), PlatformError> {
        let mut state = lock(&self.shared.state);
        state.capture_permission()?;
        state.cursor = [x, y];
        state.record(None, RecordKind::CursorWarp { x, y }, None);
        Ok(())
    }
}
impl InputSink for VirtualPlatform {
    fn inject(&mut self, event: &InputEvent) -> Result<(), PlatformError> {
        lock(&self.shared.state).inject_sink(event)
    }
    fn release_all(&mut self) {
        let mut state = lock(&self.shared.state);
        if state.released {
            return;
        }
        while let Some(press) = state.pressed.pop() {
            let (event, click) = match press {
                Press::Key { key, modifiers, button } => (InputEvent::KeyUp { key, modifiers, button }, None),
                Press::Mouse { button } => {
                    let assignment = state.click.assignment(button);
                    state.click.on_up_clear(button);
                    (InputEvent::MouseUp { button }, assignment)
                }
            };
            state.record(Some(event), RecordKind::Sink, click);
        }
        state.click.reset_click_history();
        state.record(None, RecordKind::ReleaseAll, None);
        state.released = true;
    }
    fn set_cursor_visible(&mut self, visible: bool) {
        let mut state = lock(&self.shared.state);
        state.cursor_visible = visible;
        state.record(None, RecordKind::CursorVisible { visible }, None);
    }
}
impl ClipboardBackend for VirtualClipboard {
    fn change_stamp(&self) -> u64 {
        lock(&self.shared.state).clipboard_stamp
    }
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        let state = lock(&self.shared.state);
        if state.clipboard.total_bytes() > max_bytes {
            return Err(failed("clipboard exceeds caller byte limit"));
        }
        Ok(state.clipboard.clone())
    }
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError> {
        let mut state = lock(&self.shared.state);
        state.validate_clipboard(data)?;
        state.clipboard_set(data.clone(), true)?;
        Ok(())
    }
}
impl ClipboardBackend for VirtualPlatform {
    fn change_stamp(&self) -> u64 {
        self.clipboard.change_stamp()
    }
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError> {
        self.clipboard.read(max_bytes)
    }
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError> {
        self.clipboard.write(data)
    }
}
impl ScreenSystem for VirtualPlatform {
    fn screens(&self) -> Vec<ScreenRect> {
        lock(&self.shared.state).screens.clone()
    }
    fn cursor_pos(&self) -> (i32, i32) {
        let [x, y] = lock(&self.shared.state).cursor;
        (x, y)
    }
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend {
        &mut self.clipboard
    }
    fn permissions(&self) -> Permissions {
        lock(&self.shared.state).permissions.clone()
    }
    fn request_permission(&mut self, kind: PermissionKind) -> Result<Grant, PlatformError> {
        let mut state = lock(&self.shared.state);
        let previous = permission(&state.permissions, kind);
        let grant = state.permission_request_results[permission_index(kind)]
            .take()
            .unwrap_or(previous);
        assign_permission(&mut state.permissions, kind, grant);
        state.record(
            None,
            RecordKind::PermissionRequest {
                permission: kind,
                previous,
                grant,
            },
            None,
        );
        Ok(grant)
    }
    fn on_display_changed(&mut self, cb: DisplayCallback) {
        let previous = {
            let mut slot = lock(&self.shared.display_callback);
            slot.next_id = slot.next_id.saturating_add(1);
            slot.owner_id = Some(slot.next_id);
            self.display_registration = slot.owner_id;
            slot.callback.replace(cb)
        };
        drop(previous);
    }
    fn prevent_sleep(&mut self, on: bool) {
        lock(&self.shared.state).prevent_sleep = on;
    }
    fn secure_input_app(&self) -> Option<String> {
        lock(&self.shared.state).secure_input.clone()
    }
}
