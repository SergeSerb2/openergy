# Win32 backend — untested on real Windows

This is a native implementation, compile-checked for `x86_64-pc-windows-msvc`.
It has **not been run on a real Windows computer**. No input, clipboard or display
operation from this crate was executed on the live macOS workstation.

`create(WindowsOptions {})` returns the shared `openergy_platform::Backend` with
input capture/injection, screen enumeration and a real clipboard worker. On other targets it returns a typed platform error.

- A dedicated message thread installs WH_KEYBOARD_LL and WH_MOUSE_LL hooks.
  Capture decisions return nonzero to swallow remote input. A hidden window
  receives raw relative mouse input. Injected flags and an extra-info tag prevent
  capture of synthetic input. Source buttons retain their logical key identity.
- Remote capture clips/parks the cursor and balances its ShowCursor counter.
  Stop/drop restores the clip, cursor position and visibility and removes hooks.
- SendInput uses scan codes and extended flags, VK fallback and Unicode packets.
  It normalizes absolute positions against the entire virtual desktop, supports
  buttons 1–5 and fractional wheel/hwheel accumulation, and retains a pressed
  ledger with shared modifier ownership for release-all. Keys and modifier masks
  use the shared `openergy_protocol::keys` KeyID constants. AltGr uses right Alt and the layout's Ctrl+Alt
  requirements; the capture mask identifies AltGr separately.
- A message-only clipboard window uses AddClipboardFormatListener. CF_UNICODETEXT
  converts UTF-16/CRLF to wire UTF-8/LF. `HTML Format` uses byte-correct CF_HTML
  offsets around the UTF-8 fragment. CF_DIB/CF_DIBV5 carry headerless wire DIBs.
  Unsupported PNG input returns an error; protocol bitmap data must use DIB.
- EnumDisplayMonitors supplies physical rectangles and monitor scale, with virtual
  desktop metrics as fallback. The calling thread uses PerMonitorV2 coordinates.
  The final executable should declare PerMonitorV2 in its manifest. Display
  notifications use a hidden top-level window. Sleep prevention uses
  SetThreadExecutionState. Permissions are `NotRequired`.

Real acceptance must cover hardware hooks, hook timeout/recovery, raw mouse input,
AltGr/dead keys and multiple layouts, cursor recovery, multi-monitor mixed DPI,
clipboard ownership/contention and rich formats, elevated/UIPI-restricted apps,
and forced disconnect during held keys/buttons. The OS secure desktop cannot be
captured by these ordinary desktop hooks. F25–F35 and named keys without a Win32
mapping return a typed error rather than fabricated key packets.

## Verification

```sh
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add x86_64-pc-windows-msvc
cargo check -p openergy-platform-windows --target x86_64-pc-windows-msvc
cargo clippy -p openergy-platform-windows --target x86_64-pc-windows-msvc --all-targets -- -D warnings
```

All commands passed without warnings on 2026-10-21 with Rust 1.99.1.
See the Linux crate's VERIFICATION.md for the package release gates after integration.
The shared default permission request returns `NotRequired`; native Windows execution
remains untested.

## Integration

The construction shim was removed after merging `t3/synergy-open-source-reimplementation`.
`create` returns `openergy_platform::Backend` directly. V1 shares only clipboard id 0.
No shared signature or daemon implementation change is required for this follow-up.
