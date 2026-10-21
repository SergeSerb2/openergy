# WP7 verification — 2026-10-21

**12/61 PASS; 49 expected product failures; 0 runner bugs; 0 environment failures.**

The complete run selected 59 scenarios. Two wrong-pin interop cases were then added and run. S12 and I5 received stronger declarative assertions and were rerun. Final source review also allowed a valid zero-based ledger sequence; R1/R2 and S18 were rerun after that correction. The table and JSON use the latest result for each of the 61 current scenarios. All were exercised against the same real Step 0 daemon from this worktree. No failed expectation was skipped or converted to PASS.

The 8 lab-only cases passed with real Deskflow 1.27.0 and Barrier 2.4.0 processes, including both raw handshake directions and real client/server edge/click transfer between separate containers. S0, R1, R2, and the independent S20 parent-watch case also passed. Product failures stop before unimplemented behavior; input forwarding, large clipboard transfer, numbering, TLS trust, and S15 performance budgets remain unverified.

## Commands and trimmed real output

```sh
export PATH="$HOME/.cargo/bin:$PATH"
rustc --version
# rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo fmt -p openergy-e2e -- --check
# exit 0
cargo clippy --release -p openergy-e2e -- -D warnings
# Finished `release` profile [optimized] target(s) in 0.84s
cargo build --release
# Finished `release` profile [optimized] target(s) in 13.28s
# zero warnings

target/release/openergy-e2e run --keep --report /tmp/oe7-catalogue-final.json
# 12/59 passed; exit 1

target/release/openergy-e2e run --scenario 'I2-*-wrong-pin' --port-range 24998-24999 --lab-port-range 24938-24939 --keep --report /tmp/oe7-extra-pins.json
# 0/2 passed; exit 1

target/release/openergy-e2e run --scenario 'S12-*' --port-range 24998-24999 --lab-port-range 24938-24939 --keep --report /tmp/oe7-s12-final.json
# 0/1 passed; exit 1

target/release/openergy-e2e run --scenario 'I5-*' --port-range 24996-24999 --lab-port-range 24938-24939 --keep --report /tmp/oe7-i5-final.json
# 0/2 passed; exit 1

target/release/openergy-e2e run --scenario 'R*' --keep --report /tmp/oe7-runner-final.json
# 2/2 passed; exit 0

target/release/openergy-e2e run --scenario 'S18-*' --keep --report /tmp/oe7-s18-final.json
# 0/1 passed; exit 1

python3 tools/e2e/verify_teardown.py
# Real daemon and Docker lab ready; sending SIGKILL to owned runner 81782
# PASS: daemon gone; owned lab stopped and removed after runner SIGKILL

docker ps -a --filter 'name=oe-lab-wp7' --format '{{.Names}} {{.Status}}'
# empty: all owned lab containers stopped and removed
```

The SIGKILL check also passed earlier with a separate disposable runner. `verify_teardown.py` is a standalone end-to-end driver for that destructive-to-its-own-runner check. It accepts configurable port ranges and uses real IPC readiness.

Negative CLI checks returned exit 2 for an empty scenario selection, a range containing 24800, and `--allow-desktop`. No native peer, native event tap, desktop event injection, host pasteboard write, TCC prompt, or LAN connection was used.

R1 generated a real DIB fixture file. An independent Python `struct`/`hashlib` read checked its 40-byte header and sampled pixel values:

```text
DIB fixture: PASS; header=2048x1024 BI_RGB 32-bit; bytes=8388648
sha256=62981d77d3688e3e298b6fa36b3494fc90515e2a790ff8c60b1664182cef009d
```

## Scenario results

| Scenario | Result | Blocker / evidence |
| --- | --- | --- |
| I1-barrier-plain | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| I1-barrier-tls | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| I1-deskflow-plain | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| I1-deskflow-tls | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| I2-barrier-plain | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| I2-barrier-tls | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I2-barrier-wrong-pin | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| I2-deskflow-plain | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| I2-deskflow-tls | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I2-deskflow-wrong-pin | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| I3-barrier-plain | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I3-barrier-tls | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I3-deskflow-plain | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I3-deskflow-tls | FAIL (expected) | WP3, WP4, WP6; bounded wait failed: predicate did not match |
| I4-barrier-plain | FAIL (expected) | WP6, wire dialect selection (no frozen daemon control); bounded wait failed: predicate did not match |
| I4-barrier-tls | FAIL (expected) | WP4, WP6, wire dialect selection (no frozen daemon control); openergy fingerprint returned exit code 1 |
| I4-deskflow-plain | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| I4-deskflow-tls | FAIL (expected) | WP4, WP6; openergy fingerprint returned exit code 1 |
| I5-barrier-export | FAIL (expected) | WP2, WP6; openergy export-synergy-conf returned exit code 1 |
| I5-deskflow-export | FAIL (expected) | WP2, WP6; openergy export-synergy-conf returned exit code 1 |
| L1-barrier-hello | PASS | Pinned barrier server emits documented hello bytes on assigned loopback port. |
| L1-deskflow-hello | PASS | Pinned deskflow server emits documented hello bytes on assigned loopback port. |
| L2-barrier-clipboard | PASS | Container-only Xvfb clipboard, pointer and recorder commands work without Openergy networking. |
| L2-deskflow-clipboard | PASS | Container-only Xvfb clipboard, pointer and recorder commands work without Openergy networking. |
| L3-barrier-pair | PASS | Two isolated real barrier peers connect across Docker loopback publication; edge transfer delivers a click. |
| L3-deskflow-pair | PASS | Two isolated real deskflow peers connect across Docker loopback publication; edge transfer delivers a click. |
| L4-barrier-raw-client | PASS | Real barrier client exchanges hello-back/info/CBYE with an original scripted TCP probe. |
| L4-deskflow-raw-client | PASS | Real deskflow client exchanges hello-back/info/CBYE with an original scripted TCP probe. |
| R1-control-files | PASS | Real daemon control, typed errors, TOML/file hashes/modes, restart and CLI paths exercise the runner. |
| R2-lifecycle | PASS | Owned process STOP/CONT/KILL, forced stop, start, restart and exit status operate on real virtual daemons. |
| S0-smoke | PASS | Isolated virtual IPC status has all frozen fields; daemon.stop exits zero. |
| S1-connect | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S2-edge-mapping | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S2-wire-order | FAIL (expected) | WP6; raw TCP connect failed: Connection refused (os error 61) |
| S3-input-order | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S4-disconnect-release | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S5-disconnect-release | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S6-trust-approval | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| S7-wrong-pin | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| S8-text-clipboard | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S9-large-dib | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| S10-clipboard-cap | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| S11-heartbeat | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S12-display-change | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S13-flood | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S13-garbage | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S13-oversized | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S13-truncated | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S13-truncated-hello | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S13-unknown | FAIL (expected) | WP1, WP6; raw TCP connect failed: Connection refused (os error 61) |
| S14-import-roundtrip | FAIL (expected) | WP2, WP6; openergy import-synergy returned exit code 1 |
| S15-resources-latency | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S16-permissions | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| S17-healthy | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S17-same-fingerprint | FAIL (expected) | WP4, WP6; bounded wait failed: predicate did not match |
| S17-stale | FAIL (expected) | WP6; bounded wait failed: predicate did not match |
| S18-click-numbering | FAIL (expected) | WP3, WP6; bounded wait failed: predicate did not match |
| S19-config-patch | FAIL (expected) | WP2, WP6; bounded wait failed: predicate did not match |
| S20-parent-exit | FAIL (expected) | WP6; raw TCP connect failed: Connection refused (os error 61) |
| S20-parent-off | PASS | A real daemon watches a gated parent, then exits zero; observer records its actual exit code. |
| S21-fingerprint-import | FAIL (expected) | WP4, WP6; openergy fingerprint returned exit code 1 |

## Retained evidence and requests

`verification.json` contains every result, failing step number, redacted child stderr tails, elapsed time, measurements, run command, and retained artifact directory. The principal run is `/tmp/oe7-69645-1792620752793`; each failure retained its private `sN` directory. Failed CLI examples include actual WP2 “not implemented” output and WP4 “no TLS identity exists yet” output. Raw S13 probes currently receive connection refusal because the daemon has no network listener; no codec success is claimed.

The README documents the full language and exact integration requests. No other crate was changed. These remain for the orchestrator:

- Accept `bytes_base64: string` alongside `bytes: number[]` in `virtual.clipboard` format JSON, and use base64 for large reads. An 8 MiB integer array exceeds the frozen 16 MiB IPC frame cap. No Rust signature change is needed.
- Apply `virtual.inject {events: [], seed: u64, now_ms: u64, click_interval_ms?: u64, click_distance?: f64}` controls at the receiving virtual sink. S18 controls the client clock/seed separately from server capture, so no timing/seed fields are added to the wire protocol.
- Add an explicit outgoing Barrier 1.6 dialect selector if WP6 supports Barrier 2.4 clients. Proposed additive config fields: `ServerCfg { protocol_product: String, protocol_minor: u16, .. }`, defaulting to `"Synergy"`, `8`. WP7 does not edit config/session/daemon or insert unknown config keys.
- Confirm explicit identity/trust CLI isolation through `--config PATH` or the supplied config/TLS environment selectors; the runner never uses the user’s default identity.
- Regenerate the root Cargo.lock when integrating the crate dependency-edge changes. It remains outside WP7 ownership and is excluded from these commits.

Tested source revision: `62cc6079673ad3ecaa2a94edc76217f05b3d3e00`. The final documentation commit adds these reports without changing the verified Rust code.
