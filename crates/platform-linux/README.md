# X11 backend

`create(X11Options { display: None })` opens the selected X display and returns
`openergy_platform::Backend` with independent capture, injection, screen and
clipboard handles. KeyIDs use the shared `openergy_protocol::keys` constants.
This crate has no native Xlib dependency. Other targets return a typed error without opening a display.

The capture worker uses XI2 raw observation while local, core keyboard/pointer
active grabs while remote, XKB detectable repeat, and XFixes cursor visibility.
Raw physical motion remains available at screen edges; core motion also supports
warps and synthetic X clients. The sink uses XTEST, a source-button ledger, shared
modifier ownership, fractional wheel accumulation, and saved spare-key mappings
for Unicode. Mapping restoration and repeated release are part of the real probe.

Call `source.set_cursor_captured(true)` at the screen transition **before** remote
keys arrive. Raw observation cannot undo an event already delivered locally.
Returning `PassToOs` releases the grab and forwards the current grabbed key/button
as an X SendEvent; applications that reject synthetic SendEvent require an explicit
local transition before their next input. Stop/drop removes grabs and restores
cursor visibility. Sink drop releases its injected keys/buttons and mappings.
Capture refusal rolls back both grabs before returning. The sink records which
keys and buttons it actually pressed, so release/repeat does not release input
that was already down in another X client. A Unicode mapping is restored only
if it still matches the installed fallback; a newer external mapping is kept.
Lock-state synchronization returns an error while its toggle key is held,
instead of releasing that key to force a new lock state.

`X11Clipboard::connect(display, Selection::Clipboard)` owns a hidden window and a
selection event worker. V1 routes only CLIPBOARD (wire id 0) through the shared
`Backend.clipboard` and `ScreenSystem::clipboard` handles. PRIMARY (id 1) remains
compiled in `for_selection`, `read_selection`, and `write_selection` as a local,
post-v1 extension. It is not routed or shared by v1. Each selection has its own
XFixes change stamp, so a PRIMARY update does not notify the v1 CLIPBOARD handle.
Both INCR directions have byte limits and timeouts.
Each incoming conversion uses a disposable requestor window. Oversized INCR
data is drained without retention within the ten-second/256 MiB bounds, so a
later read can still use the same owner. Destroyed outgoing requestors release
their transfer slots immediately; completed/expired transfers remove event
subscriptions. Outgoing transfers have a combined 256 MiB byte budget and at
most 16 slots. Own-write XFixes notifications do not advance the stamp a second
time after `write` returns.
Reads follow the shared unavailable/clear contract (`ffdd07a`). A stable
ownerless selection, or this backend's explicit empty write, returns successful
empty data. An owner with no supported targets, a refused/missing conversion,
or a selection that changes during the read returns `ClipboardUnavailable`.
Zero-byte text and HTML remain supported payloads. Owner identity and XFixes
stamps guard the complete snapshot, including same-owner content changes.
Text and HTML use UTF-8; image/png is converted to wire DIB and image/bmp adds or
removes its 14-byte file header. PNG supports non-interlaced grayscale, indexed,
RGB and RGBA images, 1/2/4/8/16-bit samples where valid, all PNG filters, and
stored/fixed/dynamic DEFLATE. Exported PNG uses valid stored DEFLATE. Unsupported
interlaced PNG, compressed/RLE DIB and invalid data return errors. Clipboard data
has a 256 MiB backend safety cap in addition to each caller's `max_bytes`.

RandR provides monitors and display notifications, with root geometry as fallback.
X11 coordinates use pixels and scale 1.0. Screensaver inhibition uses the X server
screensaver extension; desktop-specific power management is not implemented.
Permissions are `NotRequired`; secure-input owner is not applicable.

Run the real probes through `tools/linux-dev/linux-dev.sh`; see the current
[audit evidence](AUDIT-2026-10-21.md) and historical [VERIFICATION.md](VERIFICATION.md).
These never use the host's display, mouse, clipboard, Synergy service or LAN peers.

## Integration

Both `create` functions now return the shared `openergy_platform::Backend` directly;
there is no construction shim. X11 uses the shared default
`ScreenSystem::request_permission`, which returns `NotRequired` for all three grants.
No shared clipboard API change is needed for v1. PRIMARY routing is a post-v1 extension.
