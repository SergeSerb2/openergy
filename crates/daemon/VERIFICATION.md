# Daemon and IPC release audit — 2026-10-21

This audit covers `crates/daemon` except `windows_pipe.rs`, and `crates/ipc`.
It adds real-process regressions and keeps the older WP6 results below as a
dated historical record. It makes no new native input or release-performance
claim.

## Fixed behavior

- Unix socket startup refuses regular files, symlinks, non-owned sockets, and unsafe
  lock files. Shutdown removes only the socket inode created by this daemon.
  The accept thread polls the stop flag, so replacing the socket path no longer
  hangs shutdown. Pending IPC replies also observe shutdown.
- Relative config/socket names work. Config saves sync the file and, on Unix,
  its parent directory. A failure before rename rolls back live changes. A directory-sync
  failure after rename keeps live state aligned with the saved config and logs
  `config_durability_uncertain`; crash durability is then uncertain.
- On Unix, logs reject symlinks and unsafe file types and enforce owner-only mode `0600`.
  Rotation uses the same guarded open path.
- IPC reads and writes share a 16 MiB frame limit. An oversized reply returns
  `response_too_large`, and the caller can continue using that connection.
- CLI parsing rejects extra arguments, duplicate/empty options, wrong flags,
  and flags used as values. `connect HOST` forwards and saves the destination.
  Startup validates role/remote overrides. Linux honors `XDG_CONFIG_HOME`.
  Unix CLI writes have a timeout, and replies must match the request ID.
- Pinned connects save config before replacing the current session. Save
  failure preserves the exact config and TLS peer. Revocation clears a matching
  temporary pin, so retry requires trust approval again. Connect is idempotent
  during retry handshakes. Restart after disconnect enables capture and
  clipboard workers again.
- Clipboard read errors and size blocks retain their clipboard ID. Rejected
  read commands and dropped result events release the same pending-read latch.
  Write errors do not clear a read latch. `ClipboardUnavailable` silently clears
  a pending read; a readable empty clipboard remains a real payload/clear.
- On wake, the input sink receives the current permissions before reconnect.
  Capture start failures and transient capture notices schedule a retry during
  health checks even when the permission snapshot has not changed.
- Changing `macos.click_numbering` through live config on the macOS backend is
  rejected before save. Stop the daemon, edit that startup-only setting, and
  start the daemon again. This avoids a successful config reply for an injector
  setting that has not changed.
- A resolved native backend opts into the normal production port only when
  `OPENERGY_ALLOW_PORT_24800` is unset, before any worker starts. Explicit `0`
  retains denial. The virtual backend never sets this opt-in.

## Reproduction and validation

The committed [audit evidence](verification/2026-10-21-daemon-ipc-audit.json)
records the original failing checks and the final real-process results.
`scripts/verify_daemon_audit.py` runs the actual daemon and CLI, with private
configs, sockets, TLS directories, virtual input, and numeric loopback peers.
The new `D2-restart-disconnected-workers` scenario checks real capture,
forwarding, and clipboard polling after disconnect/restart.

Corrected `Q13-clipboard-readback-recovery` was replayed against the baseline
daemon runtime/workers with the same real backend error control. It failed at
the final retry assertion (step 14). The fixed daemon passed. `read_available`
and `read_error` remain distinct controls.

Joint validation uses a private source copy with session commits through
`c064338`, platform `ffdd07a`, config `a8cd6d9`, and the CI owner's Rust 1.99.1
MSRV. No files in those other owners' worktrees were changed. The audit commit
requires the session `ClipboardReadFailed` events and the platform
`ClipboardUnavailable`/`set_clipboard_read_error` APIs from those commits.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --locked -p openergy-daemon -p openergy-e2e
CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --no-deps --locked -p openergy-daemon -p openergy-ipc -- -D warnings
python3 crates/daemon/scripts/verify_daemon_audit.py --openergy target/debug/openergy --port-range 25130-25139
target/debug/openergy-e2e run --scenario-dir "$OE_AUDIT_SCENARIOS" --port-range 25120-25129 --lab-port-range 25140-25159
```

The actual build/Clippy calls selected the private source copy with
`--manifest-path "$OE_AUDIT_JOINT_DIR/Cargo.toml"` and reused `target` through
`CARGO_TARGET_DIR`. The root manifest and lockfile in this worktree stayed
unchanged. `OE_AUDIT_SCENARIOS` selects the focused catalogue listed in the JSON.

Native startup/port opt-in, macOS wake/capture retry, and the macOS setting rejection
have source/compile review only here. Linux default-path behavior has source review.
Native
runtime calls, desktop input, clipboard access, TCC, the installed app, and live
port 24800 were forbidden and were not used. A directory-fsync failure after a
successful rename and clipboard result-channel saturation were not forced.
Parent integration must run the full suite and platform release checks.

---

# WP6 adversarial verification — 2026-10-21

The final engine passes the complete 73-scenario catalogue, all standalone
real-process drivers, the eight Docker interop combinations, and all 14 app E2E
steps. All checks use real binaries, IPC, TCP/TLS, or isolated X11. No unit tests,
mocks, test doubles, or native input/pasteboard operations on this Mac were used.

## Build gates

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --workspace
cargo clippy --release --workspace --all-targets -- -D warnings
```

Both exit 0 with zero warnings. Exact logs:
`/tmp/oe-wp6-adversarial-build.log` and
`/tmp/oe-wp6-adversarial-clippy.log`.

## Full catalogue

```sh
export LAB_NAME=oe-lab-wp6 LAB_PORT_RANGE=24950-24959
export OPENERGY_ALLOW_PORT_24800=0
target/release/openergy-e2e run --port-range 24980-24983 \
  --lab-port-range 24950-24958 --keep \
  --report /tmp/oe-wp6-e2e-adversarial-final.json
```

```text
73/73 passed; JSON report: /tmp/oe-wp6-e2e-adversarial-final.json
```

| Scenario | Result | Time (ms) |
|---|---|---:|
| A1-modifier-and-receiver-wheel | PASS | 696 |
| A10-unsupported-hello-close | PASS | 370 |
| A11-input-backpressure-release | PASS | 2492 |
| A12-restart-connected-roles | PASS | 907 |
| A2-client-direction-version-heartbeat-sequence | PASS | 3705 |
| A3-heartbeat-close-flush | PASS | 1808 |
| A4-config-live-rollback | PASS | 595 |
| A5-unavailable-clipboard-readback | PASS | 1880 |
| A6-clipboard-backpressure | PASS | 520 |
| A7-trust-lock-crash | PASS | 574 |
| A8-config-save-rollback | PASS | 608 |
| A9-tls-trusted-status | PASS | 1095 |
| I1-barrier-plain | PASS | 32153 |
| I1-barrier-tls | PASS | 32810 |
| I1-deskflow-plain | PASS | 31947 |
| I1-deskflow-tls | PASS | 32599 |
| I2-barrier-plain | PASS | 2523 |
| I2-barrier-tls | PASS | 3233 |
| I2-barrier-wrong-pin | PASS | 1758 |
| I2-deskflow-plain | PASS | 2474 |
| I2-deskflow-tls | PASS | 3196 |
| I2-deskflow-wrong-pin | PASS | 1755 |
| I3-barrier-plain | PASS | 3793 |
| I3-barrier-tls | PASS | 4479 |
| I3-deskflow-plain | PASS | 3925 |
| I3-deskflow-tls | PASS | 4800 |
| I4-barrier-plain | PASS | 3722 |
| I4-barrier-tls | PASS | 3592 |
| I4-deskflow-plain | PASS | 3915 |
| I4-deskflow-tls | PASS | 4826 |
| I5-barrier-export | PASS | 2387 |
| I5-deskflow-export | PASS | 2342 |
| L1-barrier-hello | PASS | 1376 |
| L1-deskflow-hello | PASS | 1325 |
| L2-barrier-clipboard | PASS | 2911 |
| L2-deskflow-clipboard | PASS | 3207 |
| L3-barrier-pair | PASS | 4316 |
| L3-deskflow-pair | PASS | 4528 |
| L4-barrier-raw-client | PASS | 1312 |
| L4-deskflow-raw-client | PASS | 1341 |
| R1-control-files | PASS | 2115 |
| R2-lifecycle | PASS | 1114 |
| S0-smoke | PASS | 375 |
| S1-connect | PASS | 1680 |
| S10-clipboard-cap | PASS | 1650 |
| S11-heartbeat | PASS | 2364 |
| S12-display-change | PASS | 599 |
| S13-flood | PASS | 9320 |
| S13-garbage | PASS | 586 |
| S13-oversized | PASS | 617 |
| S13-truncated-hello | PASS | 296 |
| S13-truncated | PASS | 608 |
| S13-unknown | PASS | 1092 |
| S14-import-roundtrip | PASS | 682 |
| S15-resources-latency | PASS | 121436 |
| S16-permissions | PASS | 610 |
| S17-healthy | PASS | 892 |
| S17-same-fingerprint | PASS | 1137 |
| S17-stale | PASS | 1570 |
| S18-click-numbering | PASS | 600 |
| S19-config-patch | PASS | 675 |
| S2-edge-mapping | PASS | 599 |
| S2-wire-order | PASS | 302 |
| S20-parent-exit | PASS | 756 |
| S20-parent-off | PASS | 388 |
| S21-fingerprint-import | PASS | 1172 |
| S3-input-order | PASS | 596 |
| S4-disconnect-release | PASS | 863 |
| S5-disconnect-release | PASS | 867 |
| S6-trust-approval | PASS | 1573 |
| S7-wrong-pin | PASS | 706 |
| S8-text-clipboard | PASS | 1172 |
| S9-large-dib | PASS | 2084 |

A1–A12 cover exactly one held-modifier release, receiver wheel settings,
client-only message direction, negotiated hello versions, absent spontaneous
heartbeats, sequence-zero clipboard pushes, close flushing, live/save rollback,
unavailable readback, real queue pressure, SIGKILL during a held trust lock,
actual TLS trust status, unsupported-major close, and connected-role restarts.
A11 fills the actual input queue, keeps both peers after lost repeats, then
loses a key-up and verifies DKUP/DMUP/COUT before FIN for only that peer.

Hello-back uses `min(server minor, 8)`, with major 1. It echoes the server's
product spelling and accepts product names case-insensitively. Barrier 2.4 gets
1.6; Synergy/Deskflow 1.8 gets 1.8; a future 1.9+ gets 1.8 with one log entry.
The client never sends EICV, CROP, DSOP, or an unsolicited CALV.

The S10 receiver-cap phase checks the receiver's block event. A client's local
cap cannot be advertised by client DSOP, because DSOP is server-to-client only.
Advertised server limits still constrain outbound client clipboard transfers.

## Idle resources, retained clipboard, and latency

The catalogue measures each idle daemon separately for 60 seconds, without a
retained large clipboard. Exact observations:

```json
[
  {
    "cpu_percent": 0.03331751794735495,
    "kind": "resources",
    "rss_peak_bytes": 9338880,
    "window_ms": 60028
  },
  {
    "cpu_percent": 0.03331460060281979,
    "kind": "resources",
    "rss_peak_bytes": 9306112,
    "window_ms": 60033
  },
  {
    "kind": "capture_to_recorded_upper_bound",
    "max_ms": 1.393708,
    "p50_ms": 0.696666,
    "p95_ms": 0.79925,
    "p99_ms": 0.887084,
    "samples": 1000
  }
]
```

The independent 1000-move capture-to-sink measurement uses timestamps from the
server capture audit and client sink audit on this host. It does not include
external IPC observation overhead. The IPC observer is reported separately.

```json
{
  "samples": 1000,
  "p50_ms": 0.20425,
  "p95_ms": 0.364375,
  "p99_ms": 0.473,
  "max_ms": 0.748958,
  "kind": "IPC capture-to-recorded upper bound",
  "capture_to_sink_p99_ms": 0.18,
  "capture_to_sink_p50_ms": 0.07,
  "server_rss_bytes": 10387456,
  "client_rss_bytes": 10240000,
  "process_sample": "77906   0.5  10160\n77908   0.2  10000"
}
```

The retained-memory check measures both 8 MiB transfer directions after a
one-second settling period. Each result is below baseline plus 16,777,216 bytes.
The current remote payload is shared with the virtual clipboard. Reassembly,
serialized transfer blobs, and queued chunks are released after completion.

| Daemon | Idle baseline RSS | After server → client | After client → server | Bound |
|---|---:|---:|---:|---:|
| server | 10,108,928 | 18,612,224 | 19,365,888 | 26,886,144 |
| client | 9,928,704 | 19,218,432 | 19,316,736 | 26,705,920 |

## Standalone real-process checks

```sh
python3 crates/daemon/scripts/verify_virtual.py --port-base 24986
python3 crates/daemon/scripts/verify_security_lifetime.py
python3 crates/daemon/scripts/verify_errors.py
python3 crates/daemon/scripts/verify_memory.py
python3 crates/daemon/scripts/verify_latency.py
python3 crates/daemon/scripts/verify_interop.py
swift build --package-path apps/macos
python3 crates/daemon/scripts/verify_app.py
```

All exit 0. Trimmed output:

```text
PASS S1 connect
PASS S4 kill mid-keypress
PASS S5 kill mid-drag
PASS S9 8 MiB DIB with typing: typing_before_complete=True
PASS S10 peer cap visible block: bytes=5242892 limit=4194304
PASS S11 heartbeat death
PASS S16 permission revocation
PASS S17 same TLS fingerprint replacement
PASS S17 healthy incumbent EBSY
PASS S17 stale incumbent replacement
PASS S20 parent exit watcher
PASS connect refused: connect_refused
PASS DNS failed: dns_failed
PASS silent handshake timeout: connect_timeout
PASS idle and retained memory
E2E PASSED (14 steps)
```

Real artifact reports:

- virtual: `<temporary-directory>/oe-wp6/report.json`
- security: `<temporary-directory>/oe-wp6-security/report.json`
- errors: `<temporary-directory>/oe-wp6-errors/report.json`
- memory: `<temporary-directory>/oe-wp6-memory/report.json`
- latency: `<temporary-directory>/oe-wp6-latency/report.json`
- interop: `<temporary-directory>/oe-wp6-interop/report.json`
- app: `<temporary-directory>/oe-wp6-app/output/report.json`

## Docker interoperability and Linux

The standalone interop driver copies the lab scripts to its private artifact
directory, uses `LAB_NAME=oe-lab-wp6`, publishes only `24959`, and calls `lab.sh
down` in its cleanup. Its server uses host loopback port `24984`. The catalogue
uses distinct assigned containers and ports `24950–24958`.

| Peer | Our role | Plain | TLS |
|---|---|---|---|
| Deskflow 1.27 | Client | PASS | PASS |
| Deskflow 1.27 | Server | PASS | PASS |
| Barrier 2.4 | Client | PASS | PASS |
| Barrier 2.4 | Server | PASS | PASS |

Both directions show real keyboard, mouse, and wheel events; server direction
also verifies the Docker X11 pointer and native key/button recorder.

```sh
docker run --rm --name oe-linux-dev-wp6-adversarial --network bridge \
  -e OPENERGY_DISABLE_DISCOVERY=1 -v "$PWD:/src:ro" \
  -v oe-linux-dev-wp8-target:/target -v oe-linux-dev-wp8-cargo:/cargo \
  --tmpfs /work:exec,size=1536m oe-linux-dev-wp8:rust-1.99 bash -c \
  'cargo build --release --workspace && cargo clippy --release --workspace --all-targets -- -D warnings && ln -s /target /work/target && OPENERGY_X11_PROBE_ISOLATED=1 python3 crates/daemon/scripts/verify_linux.py'
```

```text
Finished `release` profile [optimized]
Finished `release` profile [optimized]
PASS Linux native clipboard cap: visible clipboard.blocked status
PASS Linux daemon: X11 backend, DSOP, native edge capture, key/click forwarding, clipboard both directions
```

Full Linux log: `/tmp/oe-wp6-adversarial-linux.log`. The initial 512 MiB source
copy tmpfs was too small after app build artifacts grew; the successful run
uses 1536 MiB. No host display, input device, or pasteboard was mounted.

## Safe native DIB probe

```sh
cargo build --release -p openergy-platform-macos --example mac_probe
target/release/examples/mac_probe dib /tmp/oe-wp6-dib-adversarial-final
```

```text
opaque: 64x48 ... decoded RGBA exactly equal (12288 bytes)
alpha: 64x48 ... decoded RGBA exactly equal (12288 bytes)
PASS 24-bottom-up: 3x2, padded rows, exact RGBA
PASS 24-top-down: 3x2, padded rows, exact RGBA
PASS: malformed image, empty DIB, and small cap rejected
```

The probe generates fixtures and uses native image decoding only. It does not
read/write the pasteboard, post input, or request permissions.

## Fix commits

| Fixes | Commits |
|---|---|
| H1/H2/H4, M2/M3, L1/L2: input ledger, message direction, receiver wheel, language/version/heartbeat rules | `1a39109`, `7a3bb49`, `1cfdadd` |
| H3 and retained memory: unavailable readback, retained origin after disconnect, shared current payload | `1613172`, `b2d903d` |
| H5 and M7: Windows DIB variants, crash-released trust lock | `137d332` |
| M1/M4: typed pressure errors, peer-specific release, reserved cleanup capacity and ordering | `eccbc52`, `452ed13`, `1613172`, `902864e` |
| M5/L7: cached native preflight/layout state, change-driven status and idle cadence | `383ca58`, `eccbc52`, `7a3bb49` |
| M6: transport transaction and session rollback on save failure | `eccbc52`, `c4d7c5b` |
| L3/L5: listening-only production-port guard, connected state, verified trust status | `1a39109`, `eccbc52` |
| M8/L4/L6: documented Windows IPC and v1 option limits | `2238335` |
| Connected-role restart found during verification | `b4379b9` |
| Real-process regressions and current clippy | `2751cd3`, `20c993a`, `3a6a110`, `5274523`, plus scenarios in the related fixes |

## Limits and safety

The Windows limitation from this original engine audit is superseded by the
[Windows daemon verification](../../docs/verification/windows-daemon-2026-10-21/README.md).
Desktop input acceptance remains separate from session-0 and virtual checks.

Native macOS input capture/injection, live permission prompts, and hardware
layout-change behavior remain untested on this live Synergy workstation. The
native crate builds and passes all-target clippy; the input regressions run on
virtual backends. No requested engine fix is left incomplete.

`input.sync_language`, half-duplex options, and modifier key-ID remapping limits
are documented in `crates/config/V1-LIMITS.md`. `connect HOST` intentionally
rewrites and saves client config. No IPC shape or cross-crate contract change
request remains open; new clipboard sharing uses a default additive trait method.

All test TCP ports stay within the assigned ranges. Port 24800 and LAN hosts
were never contacted. Only processes started by these drivers were stopped.
All assigned lab containers are stopped; Linux's disposable container is removed.

Final release binaries (SHA-256):

```text
29014824af54e8131303954bc2ec290d589eea0efdebb338f21a3303287a825c  openergy
4cfa0369bcc6140bf2e14d543aea882e54c1f968c85a2614abf978bbfc40057c  openergy-e2e
```

Engine source commit: `b2d903d5c83a72d09b8e1eb82c631defb0b4cd29` on
`t3/synergy-open-source-reimplementation`. The evidence commit changes this
report only. Final build and clippy checks also exit 0 after the complete run.
`docker ps` lists no running containers from these checks; `oe-lab-wp6` is exited.
