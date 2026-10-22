# Virtual product backend (WP3)

This backend is headless product code for `openergy daemon --platform virtual`.
It uses no native input APIs, pasteboard, TCC, files, sockets, or Docker. The
constructor creates state and allocates buffers. `InputSource::start` starts one
parked capture worker. It consumes no CPU while idle.

## Binding daemon factory and ownership

```rust
use openergy_platform::virtual_backend::{create, VirtualOptions};

let (backend, control) = create(VirtualOptions {
    screen_w: 1920,
    screen_h: 1080,
    hid_seed: 1000,
    double_click_ms: 500,
});
// backend.name == "virtual"
// Move backend.source to the capture owner.
// Move backend.sink and backend.screens to the I/O owner.
// Move backend.clipboard to the clipboard worker.
// Clone control into the IPC owner.
```

The exact public factory is:

```rust
pub fn create(options: VirtualOptions) -> (openergy_platform::Backend, VirtualControl);
```

`VirtualOptions` has the four fields above, derives serde serialization and
deserialization, and implements `Default` with those values except `hid_seed=0`.
Unknown option fields are errors. Omitted fields use their defaults. Since the
binding constructor cannot return an error, nonpositive initial widths/heights
become one point. Later screen updates use strict validation.

`Backend` has `name: &'static str`, `source: Box<dyn InputSource>`,
`sink: Box<dyn InputSink>`, `screens: Box<dyn ScreenSystem>`, and
`clipboard: Box<dyn ClipboardBackend>`. The parts share state through `Arc` and
short mutex sections. The frozen `ScreenSystem::clipboard()` accessor shares
the same clipboard with the independent clipboard worker.

`VirtualPlatform::new()`, `VirtualPlatform::with_config(config)`,
`VirtualPlatform::control()`, and `Clone` are also available for callers that
need one object implementing all four frozen traits. A clone has no capture worker.
Only the instance that starts capture can stop that worker. Dropping that
instance stops and joins it. Dropping a control handle does not stop capture.

For custom limits:

```rust
use openergy_platform::{VirtualConfig, virtual_backend::{create_with_config, VirtualOptions}};
let (backend, control) = create_with_config(VirtualOptions::default(), VirtualConfig {
    ledger_capacity: 128,
    pressed_capacity: 8,
    double_click_ms: 500,
    max_clipboard_bytes: 64 * 1024 * 1024,
})?;
```

`VirtualConfig` derives serde with defaults and rejects unknown fields. Defaults
are ledger capacity 4096, pressed capacity 512, interval 500 ms, clipboard byte
limit 64 MiB. Ledger capacity must be `0..=65536`; pressed capacity must be
`1..=4096`. `create_with_config` uses the interval in `VirtualOptions`.
`VirtualPlatform::with_config` uses the interval in `VirtualConfig`.

## VirtualControl methods

`VirtualControl` is `Clone + Send + Sync`. Methods need only `&self`.

```rust
pub fn inject(&self, events: Vec<InputEvent>) -> Result<CaptureReport, PlatformError>;
pub fn recorded(&self, since_seq: u64) -> Vec<RecordedEvent>;
pub fn ledger(&self, since_seq: u64) -> LedgerSnapshot;
pub fn stats(&self) -> VirtualStats;
pub fn click_assignment(&self, button: u8) -> Option<ClickAssignment>;
pub fn clipboard_get(&self) -> ClipboardSnapshot;
pub fn clipboard_set(&self, data: ClipboardData) -> Result<u64, PlatformError>;
pub fn set_screens(&self, screens: Vec<ScreenRect>) -> Result<(), PlatformError>;
pub fn set_permissions(&self, permissions: Permissions);
pub fn configure_permissions(&self, request: PermissionsRequest);
pub fn set_permission_request_result(&self, kind: PermissionKind, grant: Option<Grant>);
pub fn set_secure_input(&self, app: Option<String>);
pub fn set_hid_seed(&self, seed: i64);
```

The control method `inject` feeds local capture. `backend.sink.inject(&event)`
records remote input. They are separate directions. Neither direction loops
back into the other.

Capture injection accepts at most 4096 events per call. It rejects a malformed
batch before delivery. Capture must be started, even for an empty batch. An
empty running batch succeeds with zero counts. The capture queue holds at most
16 batches. Injection waits for queue capacity and completion; on return its
callbacks and capture audit records exist. A batch keeps its event order;
concurrent callers are ordered by queue admission. A permission change or stop
can interrupt an accepted batch after partial delivery and return an error.

The callback runs outside backend locks. It may query control state or inject
into the independent sink. It must not block. Calling `control.inject` inside
the callback returns an error to prevent deadlock. A second `start` fails.
`stop` rejects/cancels pending capture calls, wakes the worker, and joins it.
Repeated stop is harmless. A new start can then resume capture.

`PassToOs` records a passed event and applies its movement to the virtual local
cursor. `Swallow` records a swallowed event and leaves that cursor unchanged.
While cursor capture is active, passed movement cannot move the pinned cursor.
`warp_cursor` changes its position explicitly. Capture hides the virtual cursor;
uncapture and stop restore it. Uncapture works even after permission revocation.
Capture does not add presses to the sink's remote pressed ledger.

Denied/unknown Accessibility or Input Monitoring blocks capture. Denied/unknown
Accessibility or Post Events blocks sink injection. Granted/not_required allow
the operation. `release_all` works with grants revoked. Permission setters change
state only; WP6 owns the safe-leave policy. Grant restoration permits the same
running capture worker to resume. Secure input is a scripted owner/status field;
it does not filter scripted events or call an OS secure-input API.

## JSON: virtual.inject

The daemon can decode the entire `params` object with the provided adapter:

```rust
let request: openergy_platform::InjectRequest = serde_json::from_value(params)?;
let result = control.inject(request.into_events())?;
let json = serde_json::to_value(result)?;
```

`InjectRequest` has only `events`; unknown top-level fields are errors.
`ScriptEvent(InputEvent)` preserves the frozen type and accepts either compact
scenario input or the frozen tagged input. Both forms can occur in one batch.
Serialization always emits the original tagged form.

All compact objects have exactly one operation:

```json
{"events":[
  {"move_abs":[100,200]},
  {"move_rel":[-4,2]},
  {"wheel":[0,-120]},
  {"mouse_down":1},
  {"mouse_up":1},
  {"key_down":"a"},
  {"key_repeat":{"key":"a","count":2}},
  {"key_up":"a"},
  {"key_down":{"key":"Shift_L","modifiers":["shift"],"button":57}},
  {"key_up":{"key":61409,"modifiers":0,"button":57}}
]}
```

- Moves and wheel are arrays of exactly two signed `i32` numbers. Wheel is
  horizontal first, vertical second; 120 units is one notch.
- Mouse IDs are `1..=7`: 1 left, 2 middle, 3 right, 4–7 additional. IDs 6–7
  extend the documented portable 1–5 set for the shared eight-slot model.
  Zero and values above seven are errors. A move while a button is held is a
  drag; there is no distinct drag operation in the frozen enum.
- A key is a JSON integer `0..=65535`, one BMP character, an exact KeyID name
  from `docs/re/03-core-protocol.md` Appendix A, or a string such as `"0xEF0D"`.
  Names are case-sensitive. Examples: `a`, `λ`, `Return`, `Shift_L`, `F35`.
  Supplementary characters and unknown names are errors.
- Keys also accept `{ "key": <key>, "modifiers": <mask-or-names>,
  "button": <u16>, "count": <positive-i16> }`. Unknown fields are errors.
  `key` is required. `modifiers` defaults to zero. `button` defaults to the
  numeric KeyID, providing a stable synthetic source-button identifier.
  Explicit button zero is supported and uses the logical key as identity.
  `count` is only allowed for repeats and defaults to one (`1..=32767`).
- Modifiers accept a `u16` mask or an array containing any of `shift`, `control`,
  `alt`, `meta`, `super`, `altgr`, `caps_lock`, `num_lock`, `scroll_lock`.
  Their bits are `1,2,4,8,16,32,4096,8192,16384`. These are protocol bits.

The equivalent frozen tagged events are:

```json
{"events":[
  {"type":"key_down","key":97,"modifiers":1,"button":1},
  {"type":"key_repeat","key":97,"modifiers":1,"button":1,"count":2},
  {"type":"key_up","key":97,"modifiers":0,"button":1},
  {"type":"mouse_down","button":1},
  {"type":"mouse_up","button":1},
  {"type":"mouse_move_abs","x":100,"y":200},
  {"type":"mouse_move_rel","dx":2,"dy":-1},
  {"type":"mouse_wheel","dx":0,"dy":-120}
]}
```

All tagged fields are required. Tagged serde retains Step 0's behavior for
unknown fields; capture/sink validate button and repeat ranges on entry.

The serialized capture result is:

```json
{"delivered":3,"passed":2,"swallowed":1}
```

## JSON: virtual.recorded and the full audit

Decode params as `virtual_backend::RecordedRequest`: `{"since":0}`.
`since_seq` is an alias; omitting both uses zero. The cursor is exclusive.

`serde_json::to_value(control.recorded(since))` emits an array of the unchanged
Step 0 records:

```json
[
  {"sequence":1,"event":{"type":"mouse_down","button":1},"release_all":false,"click_number":1001,"click_state":1},
  {"sequence":2,"event":{"type":"mouse_move_rel","dx":2,"dy":0},"release_all":false,"click_number":1001,"click_state":1},
  {"sequence":3,"event":{"type":"mouse_up","button":1},"release_all":false,"click_number":1001,"click_state":1},
  {"sequence":4,"event":null,"release_all":true,"click_number":null,"click_state":null}
]
```

Only sink input and release markers occur in this array. Other audit entries can
create sequence gaps. `since` at or beyond the current tail returns `[]`.

For timestamps and overflow detection, serialize `control.ledger(since)`:

```json
{
  "events":[{
    "sequence":1,"event":{"type":"mouse_down","button":1},
    "release_all":false,"click_number":1001,"click_state":1,
    "timestamp_us":123,"detail":{"kind":"sink"}
  }],
  "stats":{
    "last_sequence":1,"oldest_sequence":1,"dropped":0,"sequence_exhausted":false,
    "ledger_len":1,"ledger_capacity":4096,"ledger_reserved_bytes":360448,
    "pressed_keys":0,"pressed_buttons":1,"cursor":[0,0],
    "cursor_captured":false,"cursor_visible":true,"prevent_sleep":false
  }
}
```

`timestamp_us` is monotonic microseconds since backend creation. Equal timestamps
are allowed; `sequence` defines order. Byte reservation depends on target layout;
the shown value is from the verified arm64 build. Sequences start at one and
never reset. `oldest_sequence` is null for an empty ring. `dropped` is a lifetime
count of overwritten/disabled records. Sink records and all audit markers share
one ring. Reads clone a snapshot and do not remove records. At sequence-number
exhaustion new records are counted as dropped and `sequence_exhausted=true`;
input/release state remains usable.

Every `detail` shape is listed below. Cursor, clipboard, and permission markers have
`event=null`, `release_all=false`, and null click metadata. Capture entries have
the captured tagged event and null click metadata. Only release markers set
`release_all=true`.

```json
{"kind":"sink"}
{"kind":"capture","decision":"pass_to_os"}
{"kind":"capture","decision":"swallow"}
{"kind":"release_all"}
{"kind":"cursor_captured","captured":true}
{"kind":"cursor_warp","x":100,"y":200}
{"kind":"cursor_visible","visible":false}
{"kind":"clipboard_write","stamp":2,"own_write":true,"bytes":16}
{"kind":"permission_request","permission":"accessibility","previous":"denied","grant":"granted"}
```

The pressed ledger is independent of ring retention. Each distinct key/source
button and mouse button has one entry. Duplicate key downs are recorded without
adding a second press. Duplicate mouse downs fail without mutation. Key ups match
nonzero source buttons, or logical keys when both source buttons are zero.
Repeats do not create a new press. Unmatched ups are recorded with no invented
mouse assignment. Full pressed capacity returns a typed `PlatformError::Failed`.
`release_all` emits ups in reverse press order followed by one release marker.
Repeated release calls emit nothing until another sink event occurs. It clears
click history but preserves the global numbering counter.

## JSON: virtual.clipboard

Suggested daemon params: `{"get":true}` or `{"set":<ClipboardData>}`. Dispatch
get to `clipboard_get()` and set to `clipboard_set(data)`, then return the
snapshot. The helper does not implement an IPC dispatcher.

```json
{"set":{"formats":[
  {"kind":"text_utf8","bytes":[65,10,206,187]},
  {"kind":"html_utf8","bytes":[60,98,62,65,60,47,98,62]},
  {"kind":"dib","bytes":[40,0,0,0]}
]}}
```

`ClipboardData` is `{"formats":[{"kind":<kind>,"bytes":[<u8>,...]}]}`. Kinds are
`text_utf8`, `html_utf8`, `dib`, `png`; at most one of each. Empty data is
`{"formats":[]}`. DIB is header/palette/pixels without the BMP file header;
this backend stores bytes as provided and does not validate image encoding.
PNG is backend-local; session must not send it as a Synergy wire format.
No base64 encoding or text normalization occurs here.

The result of `clipboard_get` is:

```json
{"data":{"formats":[]},"stamp":4,"own_write":false,"last_own_write_stamp":3}
```

Initial stamp is zero, last own-write stamp is null. Each successful external
set and each backend write increments the stamp, including identical contents
and clearing the clipboard. External sets are local changes (`own_write=false`).
`ClipboardBackend::write` marks its resulting stamp as an own write. That flag
remains true until an external set. The last own stamp remains available for
loop assertions. Writes above the configured byte cap, duplicate formats, and
stamp exhaustion fail without mutation. `read(max_bytes)` fails whole above its
caller's limit. Control snapshots intentionally include bytes for IPC assertions;
the backend never logs those bytes. Clipboard audit markers store sizes only.

## JSON: screens, permissions, secure input, and HID seed

`virtual.set_screens` decodes `virtual_backend::ScreensRequest` and passes
`rects` to `control.set_screens`:

```json
{"rects":[{"x":-1280,"y":0,"w":1280,"h":720,"scale":2.0}]}
```

At most 64 rectangles; positive widths/heights and finite positive scales;
right/bottom edges must fit in `i32`. `[]` is valid for headless mode. Invalid
updates leave state unchanged. Changed geometry invokes the registered display
callback outside state locks; identical geometry invokes nothing. The callback
runs on the setter's thread and may query/change the backend. Concurrent or
reentrant updates during a callback coalesce into a subsequent notification.
Registration replaces the prior callback. Dropping the ScreenSystem that registered
it removes that registration, including a callback that holds a control handle.
This prevents an Arc cycle. The daemon owns any display debounce.

`virtual.set_permissions` can decode the provided `virtual_backend::PermissionsRequest`
and call `control.configure_permissions(request)`. Without `on_request`, its flat
shape is exactly the frozen `Permissions` shape:

```json
{"accessibility":"not_required","input_monitoring":"denied","post_events":"granted"}
```

All three fields are required. Each accepts `granted`, `denied`, `unknown`, or
`not_required`. No OS permissions are requested.

To script a grant change when the daemon handles `permissions.request`, add
`on_request` with all three target grants:

```json
{
  "accessibility":"denied", "input_monitoring":"unknown", "post_events":"denied",
  "on_request":{"accessibility":"granted","input_monitoring":"granted","post_events":"granted"}
}
```

The flat fields apply at once. Each `on_request` field is a one-shot result for
that permission's next request. A virtual `screens.request_permission(kind)`
consumes that result, changes the current grant, records `permission_request`
with previous/resulting grants, and returns `Ok(grant)`. With no pending result,
it records the request and returns the current grant. A denial is state
(`Ok(Grant::Denied)`), not an error or an OS prompt. The setter can script future
request outcomes for each kind independently; `None` clears one pending outcome.
`set_permissions(Permissions)` and a `configure_permissions` call without
`on_request` clear all pending request outcomes. Scripted results for other kinds
remain pending until their own requests.

The additive trait method is:

```rust
fn request_permission(&mut self, kind: PermissionKind) -> Result<Grant, PlatformError>;
```

Its default implementation returns the corresponding current grant, so existing
backends keep compiling. The virtual override records requests and applies
scripted outcomes. macOS may override it to show a real OS prompt; that backend
is outside this work package.

If the daemon adds a script operation for secure input, decode an `Option<String>`:
`null` disables it, `"scripted-owner"` enables it. Pass the value to
`set_secure_input`; `ScreenSystem::secure_input_app()` returns that owner.
For HID seed, decode an `i64`, for example `1000`, and pass to `set_hid_seed`.
These setters do not add IPC method names to the frozen closed method table.

## Shared ClickNumberer for macOS

```rust
let mut model = openergy_platform::ClickNumberer::with_double_click_interval(500);
let down = model.on_down_at(button, fresh_hid_seed, monotonic_ms, x_points, y_points)?;
// Apply down.number and down.state to the native down event.
let drag = model.on_drag_at(button, x_points, y_points);
let up = model.assignment(button); // Apply it BEFORE clearing.
model.on_up_clear(button);
```

The added types are `ClickAssignment { number: i64, state: i64 }` and
`ClickError::{InvalidButton, InvalidPosition, AlreadyPressed, NumberExhausted}`.
`on_down_at` returns `Result<ClickAssignment, ClickError>`. Positions are `f64`
screen points; times are `u64` monotonic milliseconds. Nonfinite positions and
buttons outside `1..=7` fail without mutation. Each down uses
`max(last_number, fresh_hid_seed)+1`. A typed error refuses an exhausted counter;
it logs the exhaustion once and never wraps. The frozen `on_down(button, seed)`
method remains as a number-only compatibility method with saturating behavior.
Use `on_down_at` for complete numbering and click-state behavior.

Each button keeps its prior down time, location, and count. An elapsed interval
at most the configured interval and Euclidean distance at most four points
increments the count. Other downs start at one. Backward time resets the count.
A drag farther than four points from the down breaks the next click chain even
if it returns to the origin. It preserves the current press's assignment.
Drag/up reuse the same number and count. Up clears only that button's pressed
assignment. `reset_click_history()` ends click chains without resetting numbers
or pressed assignments; call after releasing all presses on session leave.

A frozen movement event has no explicit drag button. With several buttons held,
the virtual record exposes the most recently pressed button's assignment. All
held buttons' histories still receive the drag position, and `click_assignment`
can read each held button independently. Each up carries its own assignment.

The model uses fixed arrays and scalar state, no allocation or OS APIs. Its
state lasts for the injector's lifetime. macOS must read the real HID counter
sum on every down and supply its real double-click interval.

## Bounded memory and allocation scope

The ring reserves all its slots at creation and removes the oldest entry before
pushing into a full ring. `InputEvent`, recorded metadata, and markers contain
no heap-owned values. The pressed vector reserves its configured cap at creation;
lookups, removal, release, and click numbering cannot grow it. Successful sink
injection therefore allocates nothing per event, even after ring overflow.
The real demo measures 100,000 mixed key, repeat, move, wheel, and click events
with a counting System allocator. Instrumentation `unsafe` exists only in that
example executable; the production backend and model contain no unsafe code.

Script decoding, caller-owned batches, per-batch completion channels, thread
creation, result snapshots, clipboard contents, and screen lists may allocate.
They are outside the sink event hot path. Queue/ring/pressed limits bound retained
input state; clipboard bytes have their own cap. No full-daemon RSS or native
activation result is claimed by this crate-level demonstration.

## Scripted click policy (daemon E2E controls)

`VirtualControl::set_numbering_controls(now_ms: Option<u64>,
click_interval_ms: Option<u64>, click_distance: Option<f64>) -> Result<(), PlatformError>`
updates only supplied values. A scripted `now_ms` remains fixed until the next
update; only the click model uses it. Audit timestamps keep the real monotonic
clock. The interval must be at most 60,000 ms. Distance is finite, nonnegative,
and at most 1,000,000 points. Invalid controls leave all values unchanged.
The default distance remains four points for every backend. Native backends
continue to use the real clock and system interval.

`VirtualControl::clipboard_summary()` returns kinds, sizes, and stamps without
cloning clipboard payloads. `clipboard_get()` retains its full snapshot API.
A clipboard read over the caller's limit returns
`PlatformError::ClipboardTooLarge { bytes, limit }` so the daemon can publish a
visible block event without reading the oversized data.

`virtual.clipboard` also accepts `read_available: bool`. False makes the worker's
backend read return an empty, unavailable payload without changing its stored
clipboard, stamp, or owner. `get` still inspects stored bytes. This scripts the
same unavailable read that native own-write suppression can produce.

Clipboard worker writes share an `Arc<ClipboardData>` with the session's current
remote clipboard. The virtual backend retains that same payload. Serialized
transfer blobs are dropped after their final sender completes. The additive
`ClipboardBackend::write_shared` default delegates to `write`, so native
backends need no trait implementation change.
