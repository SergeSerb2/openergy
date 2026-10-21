# X11 backend

`create(X11Options { display: None })` opens the selected X display and returns
independent capture, injection, screen and clipboard handles. This crate has no
native Xlib dependency. Other targets return a typed error without opening a display.

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

`X11Clipboard::connect(display, Selection::Clipboard)` owns a hidden window and a
selection event worker. `for_selection`, `read_selection`, and `write_selection`
provide PRIMARY (wire id 1) in addition to CLIPBOARD (id 0). Selection requests and
change stamps use XFixes. Both INCR directions have byte limits and timeouts.
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

Run the real probes through `tools/linux-dev/linux-dev.sh`; see [VERIFICATION.md](VERIFICATION.md).
These never use the host's display, mouse, clipboard, Synergy service or LAN peers.

## Integration

The only temporary construction adaptation is `contract_shim::Backend`, which is
identical to integration-contract §2. Replace its import with
`use openergy_platform::Backend;` when WP3 is merged, then remove the shim module.

The frozen `ClipboardBackend` has no selection-id argument. Its trait handle uses
CLIPBOARD. The additive concrete API supports PRIMARY, but the shared daemon needs
an agreed selection-id routing API to expose both wire clipboard slots.
