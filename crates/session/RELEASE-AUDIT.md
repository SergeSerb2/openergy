# Session release audit — 2026-10-21

Scope: `crates/session` and new `Q` external E2E scenarios. Implementation
commits: `e12c5d8`, `0b128a3`, and `c064338`. The original source baseline is
`06b61287afd7f31aa3002d252389f2c2b17f73f5`.

All runtime checks use real daemon processes, IPC, and TCP sockets with the
virtual platform. No unit tests, mocks, desktop input, native clipboard reads,
TCC calls, live config changes, or installed application changes were used.
Destination/listen ports were limited to `25040–25059`; the reserved lab range
was `25060–25079`. No Docker peers were needed for these focused checks.

## Fixed behavior and regressions

- **Q1:** Reject a remote claim to the primary screen name or an alias. Such a
  claim previously registered a peer as the local screen and could redirect
  locally routed input. Both claims now receive `EUNK` and close.
- **Q2/Q3:** Ignore clipboard ownership/data before client setup or server-side
  screen adoption. A hello alone no longer permits a clipboard write. Each
  scenario also completes setup and verifies a valid transfer.
- **Q4:** Release a physical modifier pressed after entry separately from the
  modifier synthesized by `CINN`. Its key-up was previously consumed, leaving
  the physical key held in the receiver ledger.
- **Q5:** Restore connected peers' learned `DINF` geometry after rebuilding a
  layout for a live config change. A 2560×1440 peer now retains its 720-pixel
  midpoint mapping instead of reverting to 540 pixels.
- **Q6:** Enforce `clipboard.enabled` and `send` when pushing cached data or
  admitting an asynchronous clipboard read result. A disabled sender no longer
  replays its old clipboard when entering a remote screen. Re-enabling and a
  fresh copy still work.
- **Q7:** Ignore ownership grabs when receive is disabled. A rejected remote
  update no longer invalidates the host payload needed by a later peer.
- **Q8:** Track zero-button keys by logical KeyID. Two such held keys no longer
  overwrite the same ledger entry; each gets a release before `COUT`.
- **Q9:** Reset clipboard ownership and the entry serial on a new server
  connection. A reconnect no longer sends a previous session's serial before
  the new server has sent `CINN`.
- **Q10:** Cancel partial incoming/outgoing clipboard transfers when clipboard
  policy changes. Disabling and re-enabling receive cannot publish the end of
  a transfer admitted under the old policy. A fresh transfer still works.
- **Q11:** Cancel a pending switch delay when moving onto an unmapped edge
  segment. Returning to a mapped segment now starts the full delay again.
- **Q12:** Ignore client-originated `CROP`/`DSOP`. Those messages are
  server-to-client options; a client can no longer disable server heartbeat
  generation or dead-peer expiry with `HART=0`.
- **Q13:** Add `ServerEvent::ClipboardReadFailed { id: ClipboardId }` and
  `ClientEvent::ClipboardReadFailed { id: ClipboardId }`. Server failures clear
  only the relevant readback latch; client failures clear unfulfilled local
  ownership. The daemon owner dispatches them on read failure, size block,
  command queue rejection, and lost result delivery. Write failures do not
  clear unrelated reads.
- **Q14/Q15:** Reject repeated known format IDs in received clipboard
  containers before backend dispatch, matching the protocol helper. Local
  marshalling also rejects duplicates instead of silently dropping one. Both
  receiving roles discard the malformed update without a transfer notice or
  backend error, then accept a valid transfer.
- **Q16:** Execute the lock hotkey grammar accepted by config: all 172 named
  keys from the documented KeyID table, literal/escaped UTF-16 keys, mouse
  triggers, and Meta/AltGr modifiers. Cache the parsed binding. Consume its
  repeated downs, repeats, and matching release; toggle once per press. With
  `disable_lock_to_screen`, forward the input normally. The real-pair scenario
  checks `meta+altgr+F2`, `mousebutton(ctrl+3)`, and disable-lock passthrough.
- **Q17:** Keep synthesized entry modifiers until their aggregate mask bit
  clears. Releasing one of two Shift keys held before entry no longer releases
  Shift prematurely. A last physical key tracked after entry also gets its own
  release when the synthesized modifier clears.
- **Q18:** Forward a genuine empty clipboard as a clear in both directions.
  The server previously dropped every empty payload and left stale text on
  another computer.
- **Q19:** Keep unavailable reads distinct from clears. An unavailable readback
  must preserve the remote clipboard and produce no backend error.

## Required integration contracts

The clear change in `c064338` requires the typed clipboard contract. Apply it
together with platform `82efa10` and `ffdd07a`, daemon `0a30525`,
and macOS own-write/unavailable handling `422ff6757bcf563951b9167c52c6b113711ba827`.
`PlatformError::ClipboardUnavailable` must become `ClipboardReadFailed` without
a user-facing error. A readable `Ok(ClipboardData { formats: [] })` is a true
clear. Treating an unavailable/own-write read as `Ok(empty)` is incompatible
with clear propagation. `read_error` and `read_available` are distinct virtual
controls. The latter previously returned `Ok(empty)`; the typed platform
commit changes it to `ClipboardUnavailable`.

The CI owner's root MSRV uplift to Rust 1.99.1 is also required. This scope
includes the requested `as_chunks::<2>()` language parsing change. It does not
edit the root manifest or lockfile.

## Validation

The sanitized [machine evidence](RELEASE-AUDIT.json) records scenario results
without personal paths, addresses, hostnames, input values, or clipboard bytes.
Raw reports and retained failure artifacts remain local under
`/tmp/oe-session-audit`; they are not committed.

Before fixes, Q1–Q12 all failed against the original daemon binary. Q14/Q15,
Q16, Q17, and Q18 were also run red before their respective fixes. Q13 needs
the new error control to reach the original read-error path; the daemon owner
replayed its baseline runtime/workers with that control and reported failure
at the final retry assertion, followed by a passing fixed run.

Release validation passed 35 focused scenarios after `0b128a3`: Q1–Q12,
Q14–Q16, A1/A2/A3/A5/A6/A11, S1/S2/S3/S4/S5/S8/S9/S10/S11/S12, and the
three S17 replacement cases. Release Q17 and Q18 then passed after `c064338`.
Q13 and Q19 were then checked with the joint daemon/platform contracts above.

The final joint run used a copied candidate binary from the daemon owner's
private fixture with session through `c064338`, platform `ffdd07a`, config
`a8cd6d9`, and Rust 1.99.1. Its SHA-256 is
`2590399be4872347d032aa4cbf43ce7833289c5fb77c944908ade3d3b5fab436`.
This is a debug daemon driven by the release external runner. It is separate
from the release-only runs above and from a native platform runtime check.
All 39 focused scenarios passed, including all 19 new Q scenarios. This run
includes A5, Q13 read-error retry, Q18 both clear directions, and Q19 unavailable
read privacy. The daemon owner's committed audit independently records its
29-scenario joint pass and the corrected Q13 baseline failure.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked -p openergy-daemon -p openergy-e2e
CLIPPY_CONF_DIR=/tmp/oe-session-audit/clippy-msrv CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --locked --no-deps -p openergy-session -- -D warnings
cargo fmt -p openergy-session -- --check
target/release/openergy-e2e run --scenario-dir /tmp/oe-session-audit/focused --port-range 25040-25059 --lab-port-range 25060-25079 --report /tmp/oe-session-audit/release-focused-final.json
target/release/openergy-e2e run --scenario 'Q17-*' --port-range 25040-25059 --lab-port-range 25060-25079 --report /tmp/oe-session-audit/q17-release.json
target/release/openergy-e2e run --scenario 'Q18-*' --port-range 25040-25059 --lab-port-range 25060-25079 --report /tmp/oe-session-audit/q18-release.json
target/release/openergy-e2e run --openergy /tmp/oe-session-audit/openergy-joint --scenario-dir /tmp/oe-session-audit/focused --port-range 25040-25059 --lab-port-range 25060-25079 --report /tmp/oe-session-audit/joint-focused.json
```

The temporary Clippy config sets `msrv = "1.99.1"` while this isolated
worktree retains the original root `rust-version = "1.85"`. Scoped strict
Clippy passed, with Clippy's expected MSRV mismatch warning. A dependency-wide
1.99.1 pass initially found `manual_is_multiple_of` in config; that finding
was handed to its owner. No other owner's source was edited here.

To recreate the focused catalogue after integrating the required commits:

```sh
python3 - <<'PY'
import json
from pathlib import Path
root = Path.cwd()
dest = Path('/tmp/oe-session-audit/focused')
dest.mkdir(parents=True, exist_ok=True)
evidence = json.loads((root / 'crates/session/RELEASE-AUDIT.json').read_text())
for item in evidence['joint_run']['scenarios']:
    name = item['scenario'] + '.toml'
    target = dest / name
    if not target.exists():
        target.symlink_to(root / 'tools/e2e/scenarios' / name)
PY
```

Native keyboard layouts, native clipboard formats, actual device behavior,
Docker interoperability, and the complete integrated suite are outside this
focused virtual run. These results do not assert signed-package or public
release readiness.
