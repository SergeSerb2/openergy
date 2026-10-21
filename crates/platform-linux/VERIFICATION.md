# WP8 shared-contract follow-up verification — 2026-10-21

This report supersedes the Step 0 report. Its original whole-workspace evidence
remains in commit `8fa0b9df967a2e62edba6e36d2363708d5ec13c0`.

## Integration changes

The single authorized merge of `t3/synergy-open-source-reimplementation` completed
without conflicts. Merge commit: `d537fe14dda763dde22ac0f1beb0ccde73b1e953`.
Integration parent: `9b56efbb2d136bd4e0b44481acb48ad2b377c308`.

- Both constructors return `openergy_platform::Backend` directly. The local
  construction shims were removed.
- Named wire KeyIDs and their family bounds now use `openergy_protocol::keys`.
  Native X11 keysyms, legacy Unicode facts and Win32 scan/VK values remain local.
  The protocol dependency is target-specific in each owned crate.
- The real X11 probe constructs the shared Backend and exercises the default
  `ScreenSystem::request_permission` for all three grants (`NotRequired`).
- V1 routes only clipboard id 0 (CLIPBOARD) through the shared Backend and screen
  clipboard handles. PRIMARY code remains compiled and locally exercised as a
  post-v1 extension. It is unrouted in v1. Per-selection XFixes stamps prevent a
  PRIMARY update from notifying the v1 CLIPBOARD handle. No shared API change is needed.
- Cargo.lock started from the integration branch's version, then Cargo added WP8
  dependency records. The updated lockfile is included in this follow-up.
- Linux helper build/check commands select the two platform packages. Per the
  task, the daemon's pending WP4 net-driver integration is outside this gate and
  was not changed.

## Commands and real output

Host: macOS ARM64, Rust 1.99.1. Compilation only; no native backend was started on
this Mac. These final commands used the updated lockfile without further changes:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --locked -p openergy-platform-linux -p openergy-platform-windows
cargo check --locked -p openergy-platform-windows --target x86_64-pc-windows-msvc
cargo clippy --locked -p openergy-platform-windows --target x86_64-pc-windows-msvc --all-targets -- -D warnings
cargo fmt -p openergy-platform-linux -p openergy-platform-windows -- --check
git diff --check
```

```text
Finished `release` profile [optimized] target(s) in 0.29s
Checking openergy-platform-windows v0.1.0 (.../crates/platform-windows)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.52s
Checking openergy-platform-windows v0.1.0 (.../crates/platform-windows)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
```

All commands passed with zero warnings. Formatting and diff checks produced no output.
Windows remains compile-only, **untested on real Windows**.

Linux: isolated ARM64 Debian Bookworm container, Rust 1.99.1, Xvfb at DISPLAY=:99,
1280x800x24. Source was mounted read-only; target and Cargo caches used WP8 named volumes.

```sh
tools/linux-dev/linux-dev.sh run check
tools/linux-dev/linux-dev.sh run all
```

`run check` invokes:

```sh
cargo build --release -p openergy-platform-linux -p openergy-platform-windows
cargo clippy -p openergy-platform-linux --all-targets -- -D warnings
```

Both commands passed with zero warnings:

```text
Compiling openergy-platform-linux v0.1.0 (/work/crates/platform-linux)
Finished `release` profile [optimized] target(s) in 4.29s
Checking openergy-platform-linux v0.1.0 (/work/crates/platform-linux)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.25s
```

`run all` invokes the real release `x11_probe` and the independent Python pixel
comparison. Final real output, with routine Cargo headers omitted:

```text
construction: shared Backend and default permission requests passed
inject pointer: X=117 Y=133
inject/capture Num Lock: keypad digit and shifted keypad navigation passed
inject release_all: Virtual core XTEST keyboard: all up
inject release_all: Virtual core XTEST pointer: all up
inject keymap restored; shared modifiers and repeats passed
inject xev: ASCII, Shift, eacute, U20AC fallback, Return, clicks, drag, wheel received
clipboard backend -> xclip: text, HTML, PNG and headerless little-endian DIB/BMP passed
clipboard xclip -> backend INCR: PNG=10498508 DIB=10496108
clipboard backend -> xclip INCR: 10498508 bytes equal; oversize read rejected
clipboard v1 routing: PRIMARY does not change shared CLIPBOARD data or stamp
clipboard PRIMARY, XFixes change stamp, empty selection, invalid PNG: passed
capture events=6 relative_moves=1 xev KeyPress counts: pass=1 swallowed=1 resumed=2
capture: XI2 observation, active keyboard/pointer grabs, CaptureDecision transitions passed
PASS all
independent pixels: PNG -> DIB -> PNG preserves RGBA; 10 MB INCR DIB pixels equal source PNG
```

## Coverage and remaining acceptance

The real X server path covers native construction, default permissions, absolute
and relative pointer motion, clicks/drag, shifted and Unicode keys, keypad Num Lock,
shared modifier ownership, repeat, spare-keymap restoration, and idempotent release-all.
xev and xinput independently observe injected input. XTEST device queries show all
keys and buttons up after forced release. Active keyboard/pointer grabs stop local
xev delivery; callback PassToOs restores it.

Independent xclip clients exercise UTF8_STRING, text/plain;charset=utf-8, text/html,
image/png and image/bmp. A 10,498,508-byte PNG transfers through INCR in both directions
and decodes to a 10,496,108-byte wire DIB. Python's standard zlib independently compares
all RGBA pixels. The probe also checks PRIMARY support locally, isolation of v1
CLIPBOARD data/stamps, empty data, invalid PNG, and too-small read limits.

No unit tests, mocks or test doubles were added or run. No host input/clipboard,
port 24800, LAN peer or Synergy binary was used. Cleanup uses
`tools/linux-dev/linux-dev.sh down`; only WP8-prefixed containers are removed.

Windows runtime, physical Linux devices, real display hotplug/mixed DPI,
non-US/dead-key/AltGr layout matrices, games that reject X SendEvent, abrupt process
death, desktop power managers, and network interoperability remain untested here.
Interlaced PNG and compressed/RLE DIB return typed errors. Several supported image
depths, palettes and bitfield variants are not represented by the RGBA probe.
Idle RSS/CPU, latency and final packaged app size were not measured.

Raw observation cannot undo an event already delivered before a remote grab. The
caller must capture at the screen transition before forwarding remote keys. See the
Linux README for PassToOs semantics. Neither shared platform APIs nor daemon code
were edited by the follow-up.
