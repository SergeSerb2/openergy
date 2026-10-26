# macOS backend

Original Rust port of `spikes/macos-input`, using public CoreGraphics,
Carbon, CoreFoundation, AppKit, ImageIO, IOKit and Objective-C runtime APIs.
No new external crate, async runtime, private framework, or copied upstream
implementation is used. The backend depends on the shared platform and protocol crates.

```rust,ignore
use openergy_platform_macos::{create, ClickNumbering, MacOptions};
let backend = create(MacOptions {
    click_numbering: ClickNumbering::Native,
    lines_per_notch: 3.0,
})?;
```

Construction queries the current keyboard layout and cursor flags, but does not
start capture, register observers, post events, change the cursor, fetch clipboard
payloads, prompt for permissions, select an input source, or assert power state.
Move `source` to the capture owner, `sink` and `screens` to the io owner, and
`clipboard` to the clipboard worker. Each part implements the frozen platform trait.

## Integration points

- `create` returns `openergy_platform::Backend` directly. There is no local
  backend shim. Mouse press identity, click count, time, four-point Euclidean
  distance, drag invalidation and exhaustion behavior use the shared
  `openergy_platform::ClickNumberer`. Each down supplies a fresh HID counter seed;
  `NSEvent.doubleClickInterval` and monotonic times convert to the model's u64
  milliseconds. Both modes use the same model. Legacy leaves the CGEvent number
  unset. Number exhaustion returns an error before posting; it never wraps.
  `release_all` resets click history without resetting the rising number.
- `MacScreens` overrides `ScreenSystem::request_permission` through the shared
  trait. It calls the selected public TCC request API only after an explicit
  request. Construction and all safe probes use preflight checks only.
- Named wire KeyIDs, modifier masks, button IDs and wheel units come from
  `openergy_protocol::keys`. The Appendix B native keycodes remain Mac-specific,
  including the intentional left/right aliases. Four unnamed Appendix B KeyIDs
  have no shared named constant and remain documented numeric entries.
- The new protocol path dependency adds one root Cargo.lock dependency entry.
  That file is outside WP5's owned paths. The generated update is supplied as
  [Cargo.lock.patch](verification/shared-contract/Cargo.lock.patch) for the
  integration owner; normal Cargo commands also regenerate that entry.
- The frozen traits have no sleep/wake, lock/unlock or capture-error channel.
  `create_with_notices` is an additive macOS entry point that returns the same
  backend plus a bounded `Receiver<MacNotice>`. Retain and drain it on the io
  thread. Call `screens.on_display_changed` once to start lifecycle observation.
  That callback runs on the notification thread: queue work to io, do not operate
  io-owned backend parts from it. Treat Sleeping, ScreenLocked, capture failures,
  observer failure and CaptureStopped as leave/release-all triggers. Woke and
  ScreenUnlocked require permission/geometry/secure-input refresh before resume.
  The consumer must drain promptly. Queue overflow logs a metadata-only error
  and retains a notification failure for delivery when capacity returns.
  Permission polling also retries this failure when native events are quiet.
  This macOS API stays public and additive; the binding `create` entry point
  remains available for callers that do not need a notice receiver.
- `source.warp_cursor` is the CINN path: warp, then associate. A shared metadata
  handoff updates the sink's cached position before its next input. Ordinary
  sink moves synthesize move/drag events with location and deltas. Remote capture
  pins to the nearest current-display edge, hides once, and disassociates for
  relative deltas; leaving balances its own hide and association.
- The daemon must apply configured scroll inversion before injection, and map
  sensitivity into `lines_per_notch`. The binding options contain no inversion,
  language-group or remote-modifier-on-enter fields. Current-layout translation
  works; LSYN/DKDL group selection and CINN modifier reconciliation need a shared
  trait extension or explicit daemon composition. Do not silently claim those
  wire features are implemented by this crate.
- Call `sink.release_all`, `source.set_cursor_captured(false)`, and
  `screens.prevent_sleep(false)` on leave, disconnect, lock, sleep, grant loss
  and orderly shutdown. The sink retains failed releases for retry and logs no
  input content. `prevent_sleep(true)` means remote-active only, never idle.
  The sink has no implicit posting Drop; the daemon owns cleanup order.

## Daemon lifecycle API

Use the additive constructor when wiring macOS lifecycle events:

```rust,ignore
use openergy_platform_macos::{create_with_notices, MacNotice, MacOptions};
let (mut backend, notices): (
    openergy_platform::Backend,
    std::sync::mpsc::Receiver<MacNotice>,
) = create_with_notices(MacOptions::default())?;
backend.screens.on_display_changed(Box::new(|| {
    // Queue a geometry refresh to the daemon io owner.
}));
// Move each backend part to its thread owner; drain notices on the io thread.
```

Observation starts only when `on_display_changed` is registered. The receiver
carries `Sleeping`, `Woke`, `ScreenLocked`, `ScreenUnlocked` and `DisplaysChanged`
plus capture/observer failure notices. Sleep/wake comes from IOKit; lock/unlock
uses distributed notifications; display changes debounce for 250 ms. No power
assertion is created by observation. The io owner must leave/release and update
status on failure, sleep or lock. It must recheck state before resuming after
wake/unlock. Do not call permission requests from polling or observation callbacks.

## Native behavior

Capture uses a head-insert active session tap and a dedicated CFRunLoop.
A signallable control source handles stop without an idle polling loop. Layout
notifications rebuild fixed capture tables outside the event callback. Translation
uses two stack slots, a fixed pressed-key array, and no backend heap allocation in
the hot callback. The supplied consumer must also avoid blocking and allocation.
An AtomicBool holds the synchronous consumer decision. Openergy-tagged events pass
through without capture. Sub-unit wheel events remain suppressed while captured,
even when the fractional accumulator has no whole wire unit yet. Six disable
notices within ten seconds stop the tap and report a permission problem.

Injection owns one persistent HIDSystemState source and posts globally at the HID
tap. Legacy mouse events use a nil source and leave numbers unset. Native downs
reseed from the three HID down counters; drag/up retain that down's identity and
click state. A drag beyond four Euclidean points breaks the next multi-click
chain. Line-wheel accumulators retain fractional parts and set matching line,
fixed-point and ten-pixel-per-line fields. Layout-aware key plans keep their
original source-button associations through repeat/up and layout changes.
Unicode payload fallback is used for unmapped printable BMP characters. Unsupported
named keys return errors. No input or clipboard content is logged by the backend.

Display geometry is in global logical points. Display callbacks debounce for
250 ms. IOKit sleep requests are acknowledged immediately before the notice is
queued. Lock/unlock uses distributed notifications. Secure-input state comes from
public Carbon; there is no supported public owner-name query, so enabled with an
unresolved owner is `Some("unknown")`. No undocumented IORegistry key is used.

Clipboard access uses per-call NSPasteboard objects and autorelease pools. Text
and HTML export as UTF-8 with LF endings; PNG/TIFF export as wire DIB. A write
prepares all representations before replacement, emits both PNG and TIFF for DIB,
and records the own-write change count. Both backend clipboard handles share only
this metadata. Own writes keep the last visible local generation so a polling
worker does not announce them back. NSPasteboard has no atomic replace/rollback;
a final failed write can leave a cleared pasteboard. The own-write count covers
that partial replacement too.

An own-write read returns `PlatformError::ClipboardUnavailable`, as do a missing
pasteboard, missing promised data, a changing read snapshot, or a pasteboard with
only unsupported types. A stable empty pasteboard remains a successful empty
`ClipboardData`, so the session can forward a real clear without treating an
unavailable read as a clear.

Read caps cover encoded input, decoded raster, output DIB and total wire formats.
ImageIO metadata is checked before pixel decode. AppKit/codec scratch allocations
and the initial NSPasteboard NSData fetch are not a hard RSS bound. Writes use
64 MiB, matching the default shared configuration. The port accepts
32-bpp BI_RGB/BI_BITFIELDS DIB with 40/52/56/108/124-byte headers and
validated contiguous masks. The parser also accepts top-down images and 24-bpp
BI_RGB with padded rows. It reads validated DIB pixels directly, without a second
BMP copy or native BMP decode. Other bit depths, palettes, compression and embedded
V5 profiles return errors. Additional real Windows DIB variants remain a
compatibility gate.

## Safe verification

See [VERIFICATION.md](VERIFICATION.md) for exact commands and retained output.
The current constrained audit is in [RELEASE-AUDIT.md](RELEASE-AUDIT.md); it ran
builds, source checks, and virtual daemon scenarios only.
`mac_probe` has only safe commands. `events-dry` builds the same real CGEvents as
live injection, reads them back, and never calls CGEventPost. Inspection mode also
refuses warps and cursor visibility changes. It is not a replacement event mock.
Keymap output describes generated sample keys only; no event tap captures user input.

Live work belongs only on the separate test machine. Follow
[TEST-MAC.md](TEST-MAC.md), including a fresh persistent Focus Lab service for each
mode. The command adapter uses a private Unix socket, validates its peer, requires
an explicit pointer acknowledgement to serve, and returns after the real sink
accepts each primitive. It uses integer global points because the frozen
InputEvent uses i32; decimal lab coordinates round once, by at most 0.5 point.

Public ABI references: installed SDK headers, Apple's
[CGEventTapCreate](https://developer.apple.com/documentation/coregraphics/cgevent/tapcreate(tap:place:options:eventsofinterest:callback:userinfo:)),
[display reconfiguration callback](https://developer.apple.com/documentation/coregraphics/cgdisplayreconfigurationcallback),
[IORegisterForSystemPower](https://developer.apple.com/documentation/iokit/1557114-ioregisterforsystempower),
and [IODeregisterForSystemPower](https://developer.apple.com/documentation/iokit/1557132-ioderegisterforsystempower).
