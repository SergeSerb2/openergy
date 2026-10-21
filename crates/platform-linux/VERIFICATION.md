# WP8 verification — 2026-10-21

Environment: macOS ARM64 host, Rust 1.99.1; isolated ARM64 Debian Bookworm
container, Rust 1.99.1, Xvfb 21.1.7 at DISPLAY=:99, 1280x800x24. The source
mount was read-only. Target output and downloaded crates used WP8 named volumes.
No unit tests, mocks or test doubles were added or run.

## Final commands and real output

On the host, compilation only:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add x86_64-pc-windows-msvc
cargo build --release
cargo check -p openergy-platform-windows --target x86_64-pc-windows-msvc
cargo clippy -p openergy-platform-windows --target x86_64-pc-windows-msvc --all-targets -- -D warnings
```

```text
Finished `release` profile [optimized] target(s) in 0.16s
Checking openergy-platform-windows v0.1.0 (.../crates/platform-windows)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.67s
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s
```

The complete workspace release build had **zero warnings** on macOS. No native
backend was started there. The MSVC target check and clippy had **zero warnings**.
Windows verification is compile-only; see the Windows README acceptance list.

Using the checked-in helper, not a host X server:

```sh
tools/linux-dev/linux-dev.sh build
tools/linux-dev/linux-dev.sh run check
tools/linux-dev/linux-dev.sh run all
```

`build` built the independent Docker image and then the **whole Rust workspace**
for native ARM64 Linux:

```text
Compiling openergy-platform-linux v0.1.0 (/work/crates/platform-linux)
Compiling openergy-platform-windows v0.1.0 (/work/crates/platform-windows)
Compiling openergy-daemon v0.1.0 (/work/crates/daemon)
Compiling openergy-e2e v0.1.0 (/work/tools/e2e)
Finished `release` profile [optimized] target(s) in 13.61s
```

`run check` invokes `cargo build --release` and
`cargo clippy -p openergy-platform-linux --all-targets -- -D warnings` in Linux.
Both completed with **zero warnings**:

```text
Finished `release` profile [optimized] target(s) in 4.29s
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
```

`run all` invokes the actual release example and the independent Python pixel
comparison. This is the final real output, with Cargo's routine header omitted:

```text
fixtures: small dynamic-DEFLATE PNG; large PNG bytes=10498508
inject pointer: X=117 Y=133
inject/capture Num Lock: keypad digit and shifted keypad navigation passed
inject release_all: Virtual core XTEST keyboard: all up
inject release_all: Virtual core XTEST pointer: all up
inject keymap restored; shared modifiers and repeats passed
inject xev: ASCII, Shift, eacute, U20AC fallback, Return, clicks, drag, wheel received
clipboard backend -> xclip: text, HTML, PNG and headerless little-endian DIB/BMP passed
clipboard xclip -> backend INCR: PNG=10498508 DIB=10496108
clipboard backend -> xclip INCR: 10498508 bytes equal; oversize read rejected
clipboard PRIMARY, XFixes change stamp, empty selection, invalid PNG: passed
capture events=6 relative_moves=1 xev KeyPress counts: pass=1 swallowed=1 resumed=2
capture: XI2 observation, active keyboard/pointer grabs, CaptureDecision transitions passed
PASS all
independent pixels: PNG -> DIB -> PNG preserves RGBA; 10 MB INCR DIB pixels equal source PNG
```

## What those checks exercise

- Injection uses the real backend sink. xdotool independently reads the absolute
  and relative pointer position. xev receives the clicks, double-click sequence,
  drag, ASCII/shifted/Unicode key events and wheel input. xinput records XI2 input.
  X query-keymap and `xinput query-state` on both XTEST devices prove every key and
  mouse button is up after two `release_all` calls during a held key and button.
  The complete native keymap before/after proves spare mapping restoration.
  A two-key chord proves one release does not drop the other key's synthetic Shift.
  A real source/sink round trip checks keypad digits and shifted keypad navigation
  while Num Lock is set.
- Clipboard checks compare bytes read by independent xclip clients for UTF8_STRING,
  text/plain;charset=utf-8, text/html, image/png and image/bmp. The backend requests
  selections from real xclip owners. A 10,498,508-byte PNG transfers through INCR
  in both directions, and decodes to a 10,496,108-byte wire DIB. Python's standard
  zlib independently compares all RGBA pixels. PRIMARY is exercised both ways.
  Empty clipboard, invalid PNG and a too-small read limit are failure/empty cases.
- Capture observes xdotool input while xev also sees it, actively grabs keyboard
  and pointer while xev's key count stays unchanged, captures relative motion,
  and returns to local delivery. The final return uses the callback's PassToOs
  decision while still grabbed, so the current grabbed key is also forwarded.
  Only synthetic container input is used; the backend does not log key contents.

A separate native artifact inspection found:

```sh
wc -c /target/release/examples/x11_probe
ldd /target/release/examples/x11_probe
```

```text
856944 /target/release/examples/x11_probe
linux-vdso.so.1
libgcc_s.so.1 => /lib/aarch64-linux-gnu/libgcc_s.so.1
libc.so.6 => /lib/aarch64-linux-gnu/libc.so.6
/lib/ld-linux-aarch64.so.1
```

There is no Xlib/XTest/XInput native link dependency. This example size is not a
measurement of the final packaged app.

## Integration and remaining acceptance

- Both crates currently return the allowed identical `contract_shim::Backend`.
  Replace the import with `openergy_platform::Backend` once WP3 lands it.
- The frozen `ClipboardBackend` cannot route a selection id. The Linux concrete
  API implements both selections. Shared daemon wiring needs an agreed id-aware
  clipboard API to expose PRIMARY through the trait. No shared crate was edited.
- Cargo regenerated a local lockfile during checks. The shared root Cargo.lock
  is outside WP8 ownership and is restored, not committed. The orchestrator must
  regenerate it when integrating the new target-specific dependency declarations.
- Windows runtime, physical Linux devices, real display hotplug/mixed DPI,
  non-US/dead-key/AltGr layout matrices, games that reject X SendEvent, abrupt
  process death, real compositor/power-manager behavior, and Synergy/Deskflow
  network interoperability remain untested here. The daemon in this Step 0
  worktree does not yet construct a native backend; the real public construction
  API is driven directly by the example.
- Interlaced PNG and RLE/compressed DIB are explicit typed errors. Several supported
  image depths, palettes and bitfield variants are implemented but not represented
  by this RGBA probe. Performance goals (idle RSS/CPU, latency, final app size)
  were not measured and are not claimed.
- Raw local observation cannot retrospectively swallow an event already delivered
  before a remote grab. The caller must acquire capture at the screen transition
  before forwarding remote keys. See the Linux README for PassToOs semantics.

Cleanup command: `tools/linux-dev/linux-dev.sh down`. It removes only containers
whose name starts with `oe-linux-dev-wp8-`; named build caches remain. No container
or image owned by another agent was modified. No host input/clipboard, port 24800,
LAN peer or Synergy binary was used.
