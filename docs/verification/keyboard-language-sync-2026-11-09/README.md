# SER-215 keyboard language sync — 2026-11-09

Working-tree implementation of §3 of `docs/plan/selection-and-sync-options.md`.
No commit, push, PR, unit test, mock, real desktop session, real desktop layout setting,
or real HOME config was used. The host is aarch64 Linux with Rust 1.99.1.

## Behavior and design adherence

`input.sync_language` remains **true by default**. The release note in
`CHANGELOG.md` and `crates/config/V1-LIMITS.md` calls out that an upgrade can
make compatible existing clients start switching layouts. Both local opt-outs
are enforced. No new DSOP option, wire format, hello field, IPC method
constant, or config field was added. Existing LSYN/DKDL/DKRP fields carry
languages. LSYN stays before DSOP; DKDL/LSYN require wire >= 1.8; DKRP's suffix
works on older versions. Invalid language units keep input moving.

The receiving session emits a language action before the triggering key and
suppresses requests for the same language. Status `active_language` is the
last valid request, not confirmation of OS state. `language.sync` reports
applied/no_match/unsupported. Virtual audit records and a readable current
language snapshot expose the result through the real daemon IPC.

| Platform | Semantics built |
| --- | --- |
| X11 | XKB group names and state, alias mapping, and first matching configured group via the lock-group LatchLockState request (XkbLockGroup). No layout installation. Parenthesized variants are ignored. Human-readable names such as English (US) are accepted as well as layout ids. Names outside the small alias table are not advertised. |
| macOS | First enabled TIS keyboard-layout source whose languages contain the code. Input methods are excluded. Metadata is cached by layout id and refreshed on selected/enabled-source notices; the sink's dirty flag remains independent. TIS can reject a switch. |
| Windows | Installed HKLs in OS list order; language from foreground-thread HKL's LANGID via locale ISO639 data. Activate on our thread and post WM_INPUTLANGCHANGEREQUEST to the foreground app. Success means attempted/post succeeded, never confirmed; the event adds attempted=true. |
| Virtual | Script installed/current languages through virtual.inject; record each platform request in the real audit ledger and update current only for an installed code. |

Metadata is sampled at backend/session start, native layout notices, and the
existing health tick. Only changed snapshots are forwarded to sessions. The
regular tick also detects foreground-app layout changes that Windows does not
deliver to our hook window.

No production design deviation. Verification harness adjustments:

- S30/S31 follow main's existing S29, rather than reusing the design's old S29
  filename. P11–P14 cover modern wire bytes, the sender opt-out, legacy wire,
  malformed units, case normalization, unsupported capability, and legacy
  receive-only DKRP semantics.
- A live server config patch deliberately returns capture to the server. S30
  now waits for that return and re-enters the client before testing the opt-out.
  Its first run was 1/2 pair scenarios passing; the corrected S30 repeat passed.
- This host has no Docker command. The native mode uses the same existing
  OPENERGY_X11_PROBE_ISOLATED=1 and DISPLAY=:99 guards in a private Xvfb.
  `isolated-x11.sh` refuses an existing :99, creates private Xauthority state,
  disables TCP, and kills only its owned Xvfb. HOME is not changed.
- The initial native probe failed on the external group-toggle check.
  xdotool 3.20160805 restores the prior group after every key; see
  [its XTEST implementation](https://github.com/jordansissel/xdotool/blob/master/xdo.c).
  The final probe still runs xdotool ISO_Next_Group, then sends the configured
  physical toggle key directly through real XTEST without group restoration.
  This is a real X-server input path, not a mock. Production code is unchanged.

## Verification

Commands run from the workspace root. The explicit local range 27400–27499
and lab range 27500–27599 were checked unused before scenario execution.
The real runner assigns and checks each endpoint. Port 24800 was never used.

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --release --locked
cargo clippy --locked --target aarch64-apple-darwin -p openergy-platform-macos --all-targets -- -D warnings
cargo clippy --locked --target x86_64-pc-windows-gnu -p openergy-platform-windows -p openergy-daemon --all-targets -- -D warnings
cargo build --release --locked -p openergy-platform-linux --example x11_probe
target/release/openergy-e2e run --no-docker --scenario '*language*' --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-language-complete.json
docs/verification/keyboard-language-sync-2026-11-09/isolated-x11.sh
target/release/openergy-e2e run --no-docker --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-full-complete.json
```

Format, strict workspace Clippy, release build, both cross-target strict Clippy
checks, and the release probe build passed. The six language scenarios passed
6/6 with zero failures and zero exclusions. The final native probe passed 1/1,
including reporting, external change, first-match/deduplication, notifications,
QWERTZ delivery, missing-layout no-op, and exact core-keymap restoration.
The single-layout pair control keeps current=en and delivers the exact expected
key down/repeat/up stream under the default-true setting.

The first catalogue run was stopped after **68 passed / 6 failed** to fix a
production regression and avoid mixing binaries during a rebuild. Refreshing
all health state after ordinary virtual.inject batches had also refreshed
cursor geometry. That changed queued edge/reentry and dead-corner behavior.
The fix refreshes only requests that explicitly script keyboard languages;
ordinary injection keeps its prior path. Other failures in that incomplete
run included one IPC readiness timeout and a flood expectation. The N2 queued reentry check then passed 1/1 and all seven R5 edge/config
checks passed 7/7. A subsequent language run was 5/6: the very short raw P12
handshake raced asynchronous capture startup. P11–P13 now observe a bounded
100 ms post-DSOP wire window, also proving no late LSYN or unsolicited DKDL.
The final language run passed 6/6. The completed corrected-tree strict run and CI-budget repeat below are
reported separately.

The completed strict catalogue was **96/104 passed, 8 failed, 30 Docker
scenarios excluded**. The eight failures were deadline/readiness failures:
Q15, Q16, S11, S13-flood, S13-unknown, S15, S16, and S17-healthy. S15 reported
an external child-command timeout; others reported IPC readiness/read or raw
send deadline expiry. This does not establish that they are baseline flakes.
No scenario budgets or runner implementation were changed. A repeat uses the
repository's hosted-CI setting, --slowdown 3, with its adjusted deadlines and
latency/CPU budgets. (Machine-readable run reports were kept with the
run, not committed; the counts and failure names above are the record.)

The CI-budget repeat was **102/104 passed, 2 failed, 30 Docker scenarios
excluded**:

- S11-heartbeat: step 16 did not re-enter the client after reconnect within
  the bounded wait. A strict isolated repeat then passed 1/1 in 2712 ms,
  without a source change. The full-run failure is retained.
- S15-resources-latency: p99 **20.177 ms** exceeded the CI-adjusted **15 ms**
  budget (original budget 5 ms); p50 was 2.300 ms, max 132.586 ms. Both idle
  samples were 0.0% CPU, with RSS peaks 4,726,784 and 4,734,976 bytes.

These are not claimed as baseline failures without an unmodified-build
comparison. The full catalogue is **not clean**, despite the new language
checks and targeted edge regression checks passing. No timing budgets in
scenario files were changed, and no further slowdown was used to conceal the
remaining failures. Native runtime acceptance and strict latency acceptance
remain outstanding.

```sh
target/release/openergy-e2e run --no-docker --slowdown 3 --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-full-ci.json
```

Preliminary edit-time compiler/lint errors were corrected before these final
gates. A build was interrupted by the turn interruption and was rerun to
completion. No unit tests were run or added; existing tests remain in place.

## Limits still unverified

Windows CI in `.github/workflows/verify.yml` uses a native Windows runner and
`tools/release/release.py verify`, which selects the MSVC target and the daemon
all-targets Clippy gate. This host's installed GNU cross-target compiled the
Windows platform, daemon, and their targets with strict warnings. It did not
link an MSVC release or exercise an app accepting/ignoring the request.

The installed aarch64-apple-darwin target compiled all macOS platform targets
with strict warnings. Framework linking, x86_64 macOS compilation, actual TIS
selection, secure-input rejection, and real device behavior were not verified.
No macOS or Windows desktop was accessed.

Deskflow/Barrier Docker interop and the design's real-X11 interop lab flow
were not run because Docker is unavailable. Raw modern/legacy wire coverage
passed, but it is not a claim of native third-party layout acceptance.

## Changed files

- `CHANGELOG.md`; `crates/config/V1-LIMITS.md`.
- `crates/daemon/VIRTUAL-IPC.md`; `crates/daemon/src/{adapter,runtime}.rs`.
- `crates/ipc/src/lib.rs`.
- `crates/platform/src/{lib,virtual_backend}.rs`.
- `crates/session/src/{lib,common,server,client}.rs`.
- `crates/platform-linux/README.md`; `crates/platform-linux/src/{keymap,native}.rs`;
  `crates/platform-linux/examples/x11_probe.rs`.
- `crates/platform-macos/src/{ffi,keymap,lib,screens}.rs`.
- `crates/platform-windows/Cargo.toml`; `crates/platform-windows/src/{lib,native,source}.rs`.
- `tools/e2e/scenarios/{S30-language-sync,S31-language-no-match,P11-language-wire,
  P12-language-off-wire,P13-language-legacy-wire,P14-language-receive}.toml`.
- This verification directory: README, guarded isolated-Xvfb script, runner
  JSON reports and build/lint/probe logs. Initial failures are kept separately
  from final evidence.

Targeted regression command forms (same explicit port ranges as above):

```sh
target/release/openergy-e2e run --no-docker --scenario 'N2*' --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-n2.json
target/release/openergy-e2e run --no-docker --scenario 'R5*' --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-r5.json
```

Additional isolated re-check commands (same tree and release binary):

```sh
target/release/openergy-e2e run --no-docker --scenario 'S11*' --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-heartbeat.json
target/release/openergy-e2e run --no-docker --scenario 'S15*' --port-range 27400-27499 --lab-port-range 27500-27599 --report /tmp/ser215-resource-isolated.json
```

The strict isolated S15 repeat failed 0/1: p99 **57.181 ms** versus the
original **5 ms** budget, p50 0.447 ms, max 405.627 ms. Idle CPU remained
0.0% for both nodes and RSS peaks were 4,702,208 / 4,628,480 bytes. The
1000-event latency upper-bound acceptance remains **failed** on this host.
No source change was made in response to these timing results.

The final guarded native probe was repeated after adding an Xvfb lock-PID
ownership check to its wrapper and passed again (1/1).

## Final release binary hashes

- `openergy`: `d91daecca86d212586ee52a7fd8958c3cd275dee84d5690838d6f8f601189d2b`
- `openergy-e2e`: `fdafebc479d50d6c200cd59a9a84225ab378332f485fef58e795bf98c82b6740`
- `examples/x11_probe`: `6b7ca096a8903477a80f02dcc5620213c6e112fe692d5955080d6a60e5ceb513`

## Cleanup

The strict catalogue also recorded one cleanup failure:

- `S13-unknown`: `{'kind': 'environment', 'message': 'owned child did not exit during cleanup'}`.

The CI-budget catalogue and isolated re-checks recorded zero cleanup failures.
Final bind checks found no listeners on 27400–27599 (using SO_REUSEADDR to
ignore TIME_WAIT), and the owned Xvfb :99 lock was absent.
