# WP5 shared-contract follow-up — 2026-10-21

The authorized merge of `t3/synergy-open-source-reimplementation` completed without
conflicts. Merge commit: `0c17956c19f2ce67456617d26a24db65af7c9ff6`; merged integration
head: `c36f877`. All follow-up source and evidence changes are in
`crates/platform-macos/`. No tests, mocks or external dependencies were added.

## Shared contract changes

- Deleted `contract_shim`. Both constructors return the real
  `openergy_platform::Backend`. The create probe explicitly assigns the public
  return type and constructs the public notice-enabled API too.
- Removed the local Click struct/history, distance calculation, time window and
  exhaustion logic. The small MouseState event adapter now owns only the shared
  ClickNumberer plus Native/Legacy mode. Shared `on_down_at`, `on_drag_at`,
  `on_up_clear` and `reset_click_history` supply every assignment and count.
  Every down still reads the real HID left/right/other-down seed. System double
  click interval and monotonic seconds convert to the shared u64 milliseconds.
  Legacy uses the same shared history and omits number setters on its nil-source
  CGEvents. Model failures leave press/history state unchanged; the model's
  one-time exhaustion log flag is retained. Exhaustion now follows the shared
  policy: a typed error before posting, never wrap or duplicate at i64::MAX.
- Moved request_permission into the ScreenSystem override. Accessibility uses
  AXIsProcessTrustedWithOptions and the prompt option; Input Monitoring and Post
  Events use their respective request APIs. These paths were compiled only.
- Reused protocol::keys for named logical keys, modifier masks, buttons and wheel
  units. The four unnamed Appendix B IDs have no named shared constants and are
  documented numeric entries. Native virtual keycodes remain Mac-specific.
- Kept create_with_notices/MacNotice public and additive. README documents the
  sleep/wake/lock/unlock/display stream, explicit observer startup, thread
  ownership and safe leave/resume duties. No daemon code was changed.

## Follow-up verification

The user's follow-up gate replaces the previous whole-workspace build gate. No
whole-workspace build or daemon smoke was attempted; the integration daemon's
pending WP4 dependency is outside this work package.

Exact commands from the repository root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt -p openergy-platform-macos
cargo build --release -p openergy-platform-macos --examples
cargo clippy -p openergy-platform-macos --all-targets --release -- -D warnings
cargo fmt -p openergy-platform-macos --check
./target/release/examples/mac_probe keymap
./target/release/examples/mac_probe events-dry
./target/release/examples/mac_probe perms
./target/release/examples/mac_probe displays
./target/release/examples/mac_probe pasteboard-info
./target/release/examples/mac_probe dib crates/platform-macos/verification/shared-contract/fixtures
./target/release/examples/mac_probe create
./target/release/examples/focuslab_inject --help
```

All returned 0. Release examples and Clippy produced zero warnings. Formatting
produced no output. No CGEvent was posted, tap created/enabled, cursor changed,
pasteboard payload read/written, permission requested or power assertion created.
No input source changed, LAN host contacted, port 24800 used or synergy-core run.
Only read-only metadata and real in-memory CGEvent/image paths ran.

Trimmed real output:

```text
$ cargo build --release -p openergy-platform-macos --examples
    Finished `release` profile [optimized] target(s) in 6.18s
$ cargo clippy -p openergy-platform-macos --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 0.62s
$ ./target/release/examples/mac_probe events-dry
mode=Native double_click_interval=0.5s; real CGEvents built, ZERO posts
01 type=1 ... number=1097 click=1 ... hid_seed=1096
02 type=6 ... number=1097 click=1 ... hid_seed=1096
PASS: rising native downs above HID seed, retained drag/up identity, click states [1, 1, 2, 3, 1, 1, 1], fractional wheel [3, -3, 1, 2, -1, -2], Unicode fallback, tag; release resets click history; ledger empty
mode=Legacy double_click_interval=0.5s; real CGEvents built, ZERO posts
(same click/wheel checks pass; all event numbers remain zero)
$ ./target/release/examples/mac_probe create
backend=macos permissions=Permissions { accessibility: Granted, input_monitoring: Granted, post_events: Granted }; capture not started
invalid lines_per_notch rejected before construction
create_with_notices backend=macos; shared Backend type; notice queue empty; observers not started
```

The extra seventh click occurs inside the double-click interval immediately after
release_all. Its count returns to one while its number remains above the previous
down. Both modes pass through the shared model. Existing keymap, permission,
display and pasteboard metadata results are unchanged. The image paths again
preserve 12,288 canonical RGBA bytes exactly for both synthetic fixtures; malformed
input, empty DIB and small caps are rejected.

Full [build](verification/shared-contract/build.txt),
[Clippy](verification/shared-contract/clippy.txt),
[CGEvents](verification/shared-contract/events-dry.txt),
[create](verification/shared-contract/create.txt),
[keymap](verification/shared-contract/keymap.txt),
[permissions](verification/shared-contract/perms.txt),
[displays](verification/shared-contract/displays.txt),
[pasteboard metadata](verification/shared-contract/pasteboard-info.txt),
[images](verification/shared-contract/dib.txt), and
[environment/binary hashes](verification/shared-contract/environment.txt) are retained.

Adding the protocol workspace path dependency makes Cargo generate one root
Cargo.lock entry. To keep WP5 changes inside the owned crate, the generated
[lockfile patch](verification/shared-contract/Cargo.lock.patch) is supplied for the
integration owner. The root lockfile is restored to the merged version after
verification. No versions or external dependencies change; an ordinary Cargo build
also regenerates that one entry.

The shared Backend/click-model/permission-override merge requests from the initial
report below are resolved. Native lifecycle wiring still uses the documented
additive notice API. Existing language-group/CINN modifier surface questions and
all real posting/capture/clipboard/lifecycle/power/performance acceptance remain
for the integration owner and separate [test Mac](TEST-MAC.md).

---

# Initial WP5 verification — before the shared-contract merge

Branch: `t3/platform-macos-backend`. Changes and evidence are confined to
`crates/platform-macos/`. No unit tests, mocks, or `#[cfg(test)]` were added.
No new Cargo dependency or lockfile edit was needed.

Environment: arm64, macOS 27.2 (26B5112a), Xcode 27.1 (27A9275), SDK 27.0,
Rust 1.99.1. [Exact environment and example SHA-256 hashes](verification/environment.txt).

## Safety boundary

No CGEvent/IOHID event was posted. No event tap was created or enabled. No cursor
was warped, hidden, shown, associated or disassociated. No clipboard payload was
read or written. `pasteboard-info` read type names and change count only. No TCC
permission was requested, input source selected, or power assertion created.
No port 24800, LAN host or synergy-core process was used. No desktop/computer-use
automation was used. The only native events created were owned in-memory CGEvents
in the inspection poster and read-only cursor/flags snapshots.

`create` does not start capture or observers. Its source Drop sees no running tap
and no owned cursor hide, so it performs no cursor API call. Its screens Drop has
no observer or assertion to remove. Permission requests exist as compiled code
only. Focus Lab service live mode was never started.

## Exact commands

From the repository root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt -p openergy-platform-macos
cargo build --release
cargo build --release -p openergy-platform-macos --examples
cargo clippy -p openergy-platform-macos --all-targets --release -- -D warnings
cargo fmt -p openergy-platform-macos --check

./target/release/examples/mac_probe keymap
./target/release/examples/mac_probe events-dry
./target/release/examples/mac_probe perms
./target/release/examples/mac_probe displays
./target/release/examples/mac_probe pasteboard-info
./target/release/examples/mac_probe dib
./target/release/examples/mac_probe create
./target/release/examples/focuslab_inject --help
./target/release/examples/focuslab_inject serve --socket /tmp/openergy-no-run.sock
```

All commands returned 0 except the final intentional refusal, which returned 1
before creating a socket or backend. Full stdout/stderr is retained under
`verification/`. Clippy's `--all-targets` compiles its normal Cargo target set;
it does not run tests. No tests were authored or run.

Final trimmed build output:

```text
$ cargo build --release
   Compiling openergy-platform-macos v0.1.0 (.../crates/platform-macos)
    Finished `release` profile [optimized] target(s) in 2.37s
$ cargo build --release -p openergy-platform-macos --examples
    Finished `release` profile [optimized] target(s) in 6.20s
$ cargo clippy -p openergy-platform-macos --all-targets --release -- -D warnings
    Finished `release` profile [optimized] target(s) in 0.67s
$ cargo fmt -p openergy-platform-macos --check
(no output; exit 0)
```

The whole-workspace release build and both example binaries produced zero warnings.
See [build](verification/build.txt), [examples](verification/examples-build.txt),
[Clippy](verification/clippy.txt), and [formatting](verification/fmt.txt).

## Safe real-binary results

[Keymap](verification/keymap.txt): current `com.apple.keylayout.Canadian`, complete
128-key none/Shift capture table, plus all requested inject plans and arrows.
Command+C captures logical 99 with wire Super mask 16; physical button is 9.
é and ü use Unicode fallback on this layout, ß uses Option+S, € uses
Shift+Option+2. Return/Tab/F5/arrows/KP_5 resolve to native virtual codes.
F35 returns a typed unsupported-key error. Current-layout refresh returns false.
The printed keys are generated sample requests, never captured user keystrokes.

[CGEvent readback](verification/events-dry.txt): both modes build the exact native
objects through the sink's real event builder and inspect their fields. Native
first down was 1074 above HID seed 1073; its two drags and up all read back 1074.
Later Native downs were 1075–1079. Legacy down/drag/up numbers all read back zero.
Click states are 1,1,2,3 for the left-button drag then click/double/triple sequence,
and 1 for right and middle. A held F5 and middle button are released through the
actual reverse ledger by `release_all`; the resulting ledger count is zero.
All events retain the `0x4f50454e45524759` user-data tag. Native move/drag delta
fields, keyboard types/codes/flags, and Unicode fallback are recorded.

```text
mode=Native double_click_interval=0.5s; real CGEvents built, ZERO posts
01 type=1 xy=(110.0, 110.0) button=0 number=1074 click=1 ... hid_seed=1073
02 type=6 xy=(120.0, 120.0) button=0 number=1074 click=1 ...
03 type=6 xy=(130.0, 120.0) button=0 number=1074 click=1 ...
04 type=2 xy=(130.0, 120.0) button=0 number=1074 click=1 ...
PASS: rising native downs above HID seed, retained drag/up identity,
click states [1, 1, 2, 3, 1, 1], fractional wheel [3, -3, 1, 2, -1, -2],
Unicode fallback, tag; ledger empty
```

Wheel ±120 yields ±3 lines. Two +60 samples yield +1 then +2 lines; two -60 yield
-1 then -2. Horizontal and vertical line/fixed-point values agree, with point
fields equal to lines×10. The sample logical lowercase 'a' with a remote Shift
mask resolves to the lowercase-producing layer; uppercase 'A' additionally proves
physical Shift flags (`0x20000`). Super+C uses code 8 and `0x100000`; fallback
U+2603 reads Unicode 9731. Native keycode events' default Unicode may reflect
physical source state at construction. Actual editor text/shortcut behavior is
still a test-Mac acceptance gate; the Unicode override fallback itself is exact.

[Permissions](verification/perms.txt) and [create](verification/create.txt):

```text
Permissions { accessibility: Granted, input_monitoring: Granted, post_events: Granted } (preflight only)
backend=macos permissions=Permissions { accessibility: Granted, input_monitoring: Granted, post_events: Granted }; capture not started
invalid lines_per_notch rejected before construction
```

This proves the current process's preflight state, not which minimal TCC grant set
a macOS 27 active tap requires. The fully granted host cannot resolve that matrix.

[Displays](verification/displays.txt): one main display, ID 4, bounds/union
(0,0,2560,1440) global points, scale 1×1. Secure input was false; no public owner
query is used. Negative-origin, Retina, hot-plug and lifecycle callbacks were not
exercised.

[Pasteboard metadata](verification/pasteboard-info.txt): change count 161, five
text type names. No dataForType/stringForType call or clipboard payload access.

[DIB](verification/dib.txt): synthetic 64×48 opaque and alpha fixtures, generated
by the real example and encoded/decoded by native ImageIO/AppKit. PNG→DIB→PNG and
DIB→TIFF preserve canonical decoded sRGB RGBA exactly. The alpha fixture includes
0,1,2,17,64,127,128,200,254,255. Generated files are under
[verification/fixtures](verification/fixtures). Input/encoded PNG bytes need not
match because native encoding and alpha quantization change representation.

```text
opaque: 64x48 PNG=980 DIB=12396 PNG=980 TIFF=15654; decoded RGBA exactly equal (12288 bytes)
alpha: 64x48 PNG=1389 DIB=12396 PNG=5726 TIFF=15654; decoded RGBA exactly equal (12288 bytes)
PASS: malformed image, empty DIB, and small cap rejected
```

[Adapter help](verification/injector-help.txt) and
[missing acknowledgement](verification/injector-refusal.txt):

```text
error: serve requires --i-understand-this-moves-the-pointer on a separate test Mac
exit=1
```

Only help and refusal paths ran. Persistent service IPC, signal cleanup, and real
sink posting are compile-checked, not runtime-verified on this Mac.

## Acceptance still open

Follow [TEST-MAC.md](TEST-MAC.md) for exact Legacy/Native cmd-injector runs,
expected S1–S7 outcomes, the TCC matrix, native keyboard/clipboard/cursor cases,
lifecycle/power checks and daemon smoke. The focus lab reports, actual activation,
posting/capture behavior, permission prompts, clipboard exchange, lifecycle
callbacks, power assertions, idle RSS/CPU and latency remain unverified here.
The DIB port rejects non-32-bpp/top-down/paletted/compressed/profile variants
explicitly; real Windows clipboard variants need compatibility coverage.

Integration requests, with no edits to other crates:

1. Switch `contract_shim::Backend` to WP3's shared Backend. Align MouseState's
   time/distance history with WP3's full shared ClickNumberer when it arrives.
   Current number identity already uses the actual Step 0 shared model.
2. Move the exact inherent `MacScreens::request_permission` body to the new
   ScreenSystem trait method after WP3 merges the binding addendum.
3. WP6 must consume `create_with_notices` and start observation, or the shared
   platform trait must gain a lifecycle/error channel. Its existing display-only
   callback cannot carry lock/sleep/capture errors. The additive macOS channel is
   implemented; no shared-trait change was made here.
4. The frozen input surface has no LSYN/DKDL language group or CINN modifier-state
   operation. Current-layout translation is implemented. Full language selection
   and enter-time modifier reconciliation need a shared extension or agreed
   daemon composition. Those missing cross-crate surfaces were not invented.
   Apply scroll inversion in the daemon before injection and fold sensitivity
   into the binding lines_per_notch option.
