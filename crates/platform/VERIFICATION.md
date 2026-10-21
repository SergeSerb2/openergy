# WP3 verification

Verified on 2026-10-21 on the live arm64 macOS workstation, with `rustc 1.99.1`
and `cargo 1.99.1`. All execution stayed inside the virtual backend. No network,
Docker, Synergy process, event tap, native input, cursor, pasteboard, or TCC
operation ran. The only permission requests exercised were virtual ledger calls.

## Commands and build gates

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
cargo clippy --release -p openergy-platform --all-targets -- -D warnings
cargo fmt -p openergy-platform -- --check
git diff --check
cargo run --release --example virtual_demo
```

All commands exited zero. Release compilation covered the whole workspace with
zero warnings. Clippy covered the platform library and its real example with
warnings denied. Formatting and diff checks printed no output. No unit tests,
`#[cfg(test)]`, mocks, or test doubles were added or run.

Final workspace build output:

```text
Compiling openergy-platform v0.1.0 (<repo>/crates/platform)
   Compiling openergy-session v0.1.0 (<repo>/crates/session)
   Compiling openergy-platform-macos v0.1.0 (<repo>/crates/platform-macos)
   Compiling openergy-platform-windows v0.1.0 (<repo>/crates/platform-windows)
   Compiling openergy-platform-linux v0.1.0 (<repo>/crates/platform-linux)
   Compiling openergy-net v0.1.0 (<repo>/crates/net)
   Compiling openergy-daemon v0.1.0 (<repo>/crates/daemon)
    Finished `release` profile [optimized] target(s) in 5.03s
```

Final Clippy output:

```text
Checking openergy-platform v0.1.0 (<repo>/crates/platform)
    Finished `release` profile [optimized] target(s) in 0.19s
```

## Real program results

`virtual_demo` uses exactly `virtual_backend::create(VirtualOptions)` and the
binding `Backend` split. Capture ownership, sink injection, clipboard work,
and screen changes cross real thread boundaries. The capture callback runs on
the backend's parked worker. It receives the scripted sequence in exact order.
All checks use the caller-facing traits and control handle.

There were 57 PASS lines and no FAIL lines. Trimmed real stdout (ledger
and release JSON appear below):

```text
PASS split backend name and default screen
PASS empty recorded ledger
PASS capture refuses injection before start
PASS capture callback received 10 scripted events in order on a real thread
PASS swallow and captured cursor preserve local position
PASS capture decision, cursor capture, and warp markers recorded
PASS capture events stay out of the sink ledger
PASS S18 strictly increasing down numbers, first above seed, drag/up reuse, right-button re-seed
PASS S18 click states 1,2,3; distance/time/drag reset; independent right button
PASS recorded(since) is exclusive and supports an empty future cursor
PASS sink permission revocation returns typed error
PASS release_all emits reverse press order and marker despite revoked permission
PASS release_all is idempotent with an empty pressed ledger
PASS cursor visibility and sleep state are shared
PASS external clipboard set increments stamp without own-write flag
PASS clipboard worker sees shared formats and every own write advances stamp
PASS clipboard read byte cap leaves data and stamp unchanged
PASS external empty clipboard breaks own-write loop marker
PASS legacy screen clipboard accessor shares independent worker state
CLIPBOARD stamp=5 formats=3 bytes=16 own_write=true
PASS display callback fires once for a changed geometry on a real thread
PASS invalid screen geometry is rejected atomically
PASS empty display list supports a headless machine
PASS secure input owner is shared
PASS permission state is shared
PASS capture revocation returns typed error
PASS capture start is refused while permission is denied
PASS second capture owner is refused
PASS running capture refuses events after revocation
PASS cursor can always be uncaptured after revocation
PASS running capture resumes after grant without restart
PASS callback reentry returns an error without deadlock
PASS capture stop restores a pinned hidden cursor
PASS scripted grants stay unchanged until a permission request
PASS virtual permission requests apply scripted outcomes
PASS permission requests are recorded with previous and resulting grants
PASS a scripted denial is returned as grant state
PASS unscripted requests return current grant
PASS permission setters clear pending prompt outcomes
PASS malformed script returns an error
PASS malformed script returns an error
PASS malformed script returns an error
PASS malformed script returns an error
PASS malformed script returns an error
PASS malformed script returns an error
PASS frozen tagged events remain accepted
PASS invalid native event returns an error
PASS number exhaustion rejects down without ledger or pressed-state mutation
PASS dropping screen owner releases callback and its captured control
MEASURE events=100000 elapsed_us=3741 ns_per_event=37 allocations=0 ring_len=128 ring_cap=128 reserved_bytes=11264 dropped=99872
PASS 100k real sink events allocate zero times in the hot path
PASS ring stays bounded and reports exact dropped count
PASS retained records have strict sequences and monotonic timestamps
PASS pressed capacity failure preserves existing state
PASS bounded pressed ledger still releases all keys
PASS clipboard write size cap rejects both external and own writes atomically
PASS duplicate clipboard formats return an error
PASS zero-cap ring reports dropped records and still clears pressed state
PASS virtual_demo completed; no native input, clipboard, permissions, network, or Docker used
```

First click and drag, from the printed ledger (other records omitted):

```json
[
  {
    "sequence": 15,
    "event": {
      "type": "mouse_move_abs",
      "x": 100,
      "y": 100
    },
    "release_all": false,
    "click_number": null,
    "click_state": null,
    "timestamp_us": 468,
    "detail": {
      "kind": "sink"
    }
  },
  {
    "sequence": 16,
    "event": {
      "type": "mouse_down",
      "button": 1
    },
    "release_all": false,
    "click_number": 1001,
    "click_state": 1,
    "timestamp_us": 468,
    "detail": {
      "kind": "sink"
    }
  },
  {
    "sequence": 17,
    "event": {
      "type": "mouse_move_rel",
      "dx": 2,
      "dy": 0
    },
    "release_all": false,
    "click_number": 1001,
    "click_state": 1,
    "timestamp_us": 468,
    "detail": {
      "kind": "sink"
    }
  },
  {
    "sequence": 18,
    "event": {
      "type": "mouse_up",
      "button": 1
    },
    "release_all": false,
    "click_number": 1001,
    "click_state": 1,
    "timestamp_us": 468,
    "detail": {
      "kind": "sink"
    }
  }
]
```

The complete printed sequence had down numbers 1001, 1002, 1003, 1004,
2001, 2002, 2003, 2004. Down states were 1, 2, 3, 1, 1, 1, 2, 1.
The first down exceeded seed 1000. The right down exceeded changed seed 2000.
Every drag and up retained its down's number and count. A 550 ms pause reset
the count with the default 500 ms interval. A far drag that returned to its
origin still reset the next click. Distance changes and the right button's
independent click chain also passed.

Printed forced-release tail after pressing Shift, A twice, and left mouse:

```json
[
  {
    "sequence": 40,
    "event": {
      "type": "mouse_up",
      "button": 1
    },
    "release_all": false,
    "click_number": 2005,
    "click_state": 2
  },
  {
    "sequence": 41,
    "event": {
      "type": "key_up",
      "key": 97,
      "modifiers": 1,
      "button": 1
    },
    "release_all": false,
    "click_number": null,
    "click_state": null
  },
  {
    "sequence": 42,
    "event": {
      "type": "key_up",
      "key": 61409,
      "modifiers": 1,
      "button": 57
    },
    "release_all": false,
    "click_number": null,
    "click_state": null
  },
  {
    "sequence": 43,
    "event": null,
    "release_all": true,
    "click_number": null,
    "click_state": null
  }
]
```

The duplicate key down did not create another outstanding press. The tail
released mouse, A, Shift, then emitted the marker. Post Events was denied
before release. A repeated `release_all` added no further records.

Additional flows include permission revocation/restoration without restarting
capture; safe uncapture after revocation; permission-request scripts changing
all three grant states; recorded previous/result grants; denial as state;
callback reentry rejection; capture restart/stop; callback owner cleanup;
clipboard own-write stamps; unchanged stamps after size failures; display
notifications; malformed script rejection; and an empty headless display list.

## Allocation and bounded-memory evidence

```text
MEASURE events=100000 elapsed_us=3741 ns_per_event=37 allocations=0 ring_len=128 ring_cap=128 reserved_bytes=11264 dropped=99872
```

This measurement injects 100,000 mixed key down/repeat/up, mouse down/drag/up,
wheel, and absolute movement events through the real `InputSink` trait object.
The example's counting allocator delegates every operation to the real System
allocator. It enables counting only during the loop, after construction, and
before result snapshots or printing. It observed zero allocations/reallocations.
Each explicit unsafe block in that example has a one-line Safety comment.
The production library contains no unsafe code.

The ring reserved 128 slots (11,264 bytes on this target), retained 128 records,
and reported exactly 99,872 overwrites. Its reservation did not grow. Every
retained sequence and timestamp was monotonic. The independent pressed ledger
was empty at the end, still worked at its configured limit, and released all
keys after rejecting an overflow. A zero-capacity ring dropped all three
records of down/release-up/marker but still released the press.

The design supports the measurement: `InputEvent`, click assignments, and
`VirtualRecord` contain no heap values. The ring removes an old entry before
pushing into a full preallocated `VecDeque`. The pressed vector has a fixed
reserved capacity, and the click model uses fixed arrays. Control JSON,
batches/completion channels, clipboard data, thread creation, and snapshots may
allocate outside this sink event loop. This timing is a local injection cost,
not whole-daemon IPC/network latency or an idle RSS measurement.

## Artifact sizes and remaining integration

Before WP3, `target/release/openergy` was 882,256 bytes; after WP3 it is
882,528 bytes (+272 bytes). `openergy-e2e` remains 460,816 bytes. The real demo
is 646,304 bytes. The platform rlib grew from 657,288 to 2,101,088 bytes
(+1,443,800 bytes); the rlib is not a shipped app artifact. These are stripped
release binary sizes and an unlinked archive size, respectively.

The current daemon still uses the Step 0 integration stubs. Its binary delta
does not measure the fully wired virtual backend. The demo exercises the
binding factory and public API, as explicitly requested for WP3. S18 over two
real daemon IPC endpoints remains a WP6/WP7 integration gate. Native macOS
WindowServer activation, full-daemon idle RSS/CPU, and network latency remain
separate gates; this report makes no claim for them.

All edits stay under `crates/platform/`. No dependencies or root manifest changed.
The authorized additive contracts `Backend`, the exact virtual factory/options,
and default `ScreenSystem::request_permission` are present. The original frozen
InputEvent/RecordedEvent shapes and existing trait signatures remain intact.
No unresolved cross-crate signature request remains. See [VIRTUAL.md](VIRTUAL.md)
for the exact adapters, JSON shapes, permission scripts, and macOS numbering API.
