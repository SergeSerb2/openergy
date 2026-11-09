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
Image writes also require decoded RGBA and each exported PNG/BMP representation
to fit 64 MiB. The backend checks DIB expansion before allocating image output.
A rejected conversion preserves the previous selection and its change stamp.
Near-limit valid conversions still use several temporary buffers. See the
[resource regression evidence](../net/examples/r5-evidence/README.md).

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

## Read-only Wayland discovery (M1)

X11 remains the only production Linux backend. `wayland_probe` is a Linux
example, not a backend, and has only a `discover` mode. It never creates a
surface, portal session or EIS connection, reads clipboard data, injects input,
or requests permission. Its dependencies are Linux dev-dependencies only.

Use the separate native Weston lab on a disposable Linux host. It needs
Weston **13.0.0** (tested), `dbus-daemon`, systemd's `busctl`, GNU `timeout`,
Python 3, `dpkg-query` for metadata, and the workspace Rust toolchain.
The native lab currently targets Debian/Ubuntu. On Ubuntu 24.04, install the tools
with `sudo apt-get install weston dbus-daemon systemd coreutils python3`.
It does not need Docker, XWayland, a GPU or input-device access.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR=target
cargo build --release --locked -p openergy-platform-linux --example wayland_probe
cargo clippy --release --locked -p openergy-platform-linux --example wayland_probe -- -D warnings
python3 tools/linux-dev/wayland-discover.py --probe "$CARGO_TARGET_DIR/release/examples/wayland_probe"
# Omitting --probe builds the example first.
```

The lab starts Weston with `-B headless --renderer=pixman --no-config`, one
1280×720 output, and no host sockets. It creates private HOME/XDG directories,
a private session bus with no service activation directories, a private Wayland
socket and owned process groups. It saves JSON, stderr, redacted compositor/bus
logs, commands, package versions and source SHA in the printed evidence directory.
This is a native host lane: image digest is explicitly null. It checks a logical
output, missing portal service, isolation refusals, a stopped compositor's
five-second timeout, and failure after compositor removal. It removes all owned
process groups and session files, including on interrupt. Evidence remains.

For a separately provisioned, **run-owned** compositor/portal lane, the exact
probe command is `wayland_probe discover`. The guard requires
`OPENERGY_WAYLAND_PROBE_ISOLATED=1`, an absolute canonical
`OPENERGY_WAYLAND_PROBE_ROOT`, and owned mode-0700 directories at that root:
`HOME=ROOT/home`, `XDG_RUNTIME_DIR=ROOT/runtime`, `XDG_CONFIG_HOME=ROOT/config`,
`XDG_DATA_HOME=ROOT/data`, `XDG_STATE_HOME=ROOT/state`, and
`XDG_CACHE_HOME=ROOT/cache`. It requires `WAYLAND_DISPLAY=wayland-probe-0` and
`DBUS_SESSION_BUS_ADDRESS=unix:path=ROOT/runtime/bus`, with owned sockets there.
`DISPLAY` and `WAYLAND_SOCKET` must be unset. These checks prevent accidental
host access; they are not a security boundary against a caller who forges the
environment. Do not point this probe at an active desktop.

The probe emits one JSON object on stdout. Errors also have a short stderr
message. Exit 0 means discovery completed, **not** that a role is supported.
Exit 1 means invalid usage/isolation, no Wayland session, connection failure,
or timeout. Missing `WAYLAND_DISPLAY` gives a clear `no Wayland session` error
before any connection. Portal absence or failure appears in JSON and does not
turn a valid Wayland discovery into a connection error.

Output fields:

- `compositor` contains environment hints. Wayland has no standard compositor
  name/version query. Inspect vendor globals and output make/model too; desktop
  names never determine capabilities.
- `wayland.globals` lists every advertised global and version. `relevant_globals`
  also makes absent virtual keyboard/pointer, screencopy, output, seat,
  relative-pointer, constraints, warp and data-control extensions explicit.
  Privileged extensions are not bound or exercised.
- `wayland.outputs` separates core pixel modes, physical geometry, integer scale
  and xdg-output logical position/size. Missing logical data and fractional
  scale stay null. These are supplemental coordinates, not EI regions or IC
  zones. There is no guessed origin, scale ratio or fallback output.
- `wayland.seats` contains seat capabilities and keyboard keymap format/size
  and repeat metadata if available. It closes the keymap FD without reading
  its contents. Layout and physical modifier/lock state remain unknown.
  It never records key events or claims global keyboard observation.
- `portals` reports `portal_service_missing` when the well-known service has no
  owner on the private bus. For a running service it uses D-Bus Introspect and
  property reads for RemoteDesktop, InputCapture, Clipboard and ScreenCast.
  It lists methods, interface versions and advertised keyboard/pointer masks.
  `busctl --auto-start=no --allow-interactive-authorization=no` prevents service
  activation or authorization prompts. Every call has a two-second bus timeout
  and a three-second process bound. Missing tools, read errors and timeouts stay
  distinct from missing interfaces. Private bus identifiers are not output.
- `ei` reports socket-environment presence and owned socket metadata only inside
  the private runtime directory. It does not inspect external socket paths.
  Negotiated EI versions, devices
  and absolute regions remain unknown: they cannot be proved by portal version
  or `ConnectToEIS` introspection. No EIS socket is opened.
- `capability_matrix` evaluates client/server independently. `unsupported` means
  a required advertised prerequisite is missing. `unknown` means discovery could
  not establish a prerequisite. `candidate_after_grant` means portal methods,
  version and device masks look sufficient for later acceptance, with unresolved
  device/region/barrier requirements listed. **Every role has `ready=false`.**
  The optional direct path is only a later explicit-opt-in candidate. Virtual
  input and screencopy never imply server capture. Client absolute Enter,
  capture-contract/held-transition gaps, and missing passive capture/PassOnce
  stay explicit. Clipboard scope and formats need separate acceptance.

The [Wayland design](../../docs/plan/wayland-backend.md) defines later milestones.
Weston discovery does not prove GNOME/KDE portal support, authorization, input,
clipboard or daemon readiness. M2 still needs independently provisioned Mutter
and KWin sessions and real granted EI devices/absolute regions.

To also check introspection against an installed real portal frontend, add
`--portal /usr/libexec/xdg-desktop-portal` to the lab command. This starts only
that run-owned service after the missing-service check, then removes it. A
frontend without a GNOME/KDE backend still cannot supply input capabilities.
The default lane does not need the portal package.
