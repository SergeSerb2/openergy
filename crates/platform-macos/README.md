# macOS backend

Original Rust port of `spikes/macos-input`, using public CoreGraphics,
Carbon, CoreFoundation, AppKit, ImageIO, IOKit and Objective-C runtime APIs.
No new Cargo dependency, async runtime, private framework, or copied upstream
implementation is used.

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

- This checkout has Step 0's `ClickNumberer`, but no shared `Backend`.
  `contract_shim::Backend` has the exact binding construction fields. Switch its
  export/import to `openergy_platform::Backend` after WP3 merges. Mouse number
  identity already calls the shared Step 0 `ClickNumberer`; time, 4-point distance,
  held state and drag invalidation are in `inject::MouseState`. Consolidate that
  history with WP3's full model after it is available. Native down numbers saturate
  and log once on i64 exhaustion; they never wrap.
- The binding `ScreenSystem::request_permission` addition is not in this checkout.
  `MacScreens` has the exact inherent method. Move its body into the trait impl
  when WP3's method arrives. It calls the requested public TCC request API only
  after an explicit call. No safe probe calls it.
- The frozen traits have no sleep/wake, lock/unlock or capture-error channel.
  `create_with_notices` is an additive macOS entry point that returns the same
  backend plus a bounded `Receiver<MacNotice>`. Retain and drain it on the io
  thread. Call `screens.on_display_changed` once to start lifecycle observation.
  That callback runs on the notification thread: queue work to io, do not operate
  io-owned backend parts from it. Treat Sleeping, ScreenLocked, capture failures,
  observer failure and CaptureStopped as leave/release-all triggers. Woke and
  ScreenUnlocked require permission/geometry/secure-input refresh before resume.
  The consumer must drain promptly; queue overflow logs a metadata-only error.
  A common lifecycle callback/channel in `openergy-platform` would let the daemon
  retain its binding `create` call across every OS. No other crate was changed here.
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

Read caps cover encoded input, decoded raster, output DIB and total wire formats.
ImageIO metadata is checked before pixel decode. AppKit/codec scratch allocations
and the initial NSPasteboard NSData fetch are not a hard RSS bound. Writes use
64 MiB, matching the default shared configuration. The port accepts positive,
bottom-up 32-bpp BI_RGB/BI_BITFIELDS DIB with 40/52/56/108/124-byte headers and
validated contiguous masks. Other bit depths, top-down images, palettes,
compression and embedded V5 profiles return typed errors. Additional real Windows
DIB variants remain a compatibility gate.

## Safe verification

See [VERIFICATION.md](VERIFICATION.md) for exact commands and retained output.
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
