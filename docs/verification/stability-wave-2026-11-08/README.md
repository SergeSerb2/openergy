# Stability wave — 2026-11-08

Candidate work on branch `t3/stability-wave-2`, based on published-alpha merge
commit `fdd1d694bc40b5b0eda5327e29c276123bf1702f` plus an uncommitted diff at
record time (27 files, 1771 insertions, 334 deletions). This record covers the
dirty candidate tree, not a published artifact. It grants no native desktop
acceptance.

## Environment

| Field | Value |
| --- | --- |
| Date | 2026-11-08, America/Vancouver |
| Host | Linux aarch64 VM (OrbStack guest), kernel `7.2.12-orbstack`, 18 CPUs |
| Toolchain | Pinned Rust 1.99.1 (`rust-toolchain.toml`), installed locally for this run |
| Daemon SHA-256 | `8597f3e86db1fccd5ac30611f1dc7ec143f9b6cb429d5d9a6e5773250d6dc40f` (`target/release/openergy`) |
| E2E runner SHA-256 | `11a63b72d49a878981df3be57cba39dd3ce81cb5fd40d40fc70d4f9eedd04cb0` |
| Constraints | Virtual backend and isolated Xvfb only; owned processes, private state, no port 24800, no live sharing or desktop control |

`tools/release/release.py verify` refuses this host because release targets
are x86_64-only. The same steps ran directly for the host target: locked
fetch, notices `--check`, `cargo fmt --all --check`, strict workspace Clippy
(`--all-targets -D warnings`), locked release build, the real
`release.smoke()` CLI/IPC check imported from `release.py`, then the full
virtual e2e suite. Hosted CI must still repeat the gate on x86_64.

## Results

| Check | Result |
| --- | --- |
| Notices, format, strict Clippy, locked release build, CLI smoke | Pass |
| Virtual e2e (`openergy-e2e run --no-docker`) | **95/95 passed; 30 excluded** (Docker interop) |
| Isolated X11 probe (`x11_probe all`, Xvfb :99, incl. new `reconnect` mode) | Pass, run twice |
| Pixel independence (`tools/linux-dev/verify-pixels.py`) | Pass: PNG→DIB→PNG preserves RGBA; 10 MB INCR DIB pixels equal source |
| Config CLI checks (`crates/config/verify.py`) | Pass, 83 real CLI invocations |
| Cross-target type/lint checks | Pass: `x86_64-pc-windows-gnu` (platform-windows, net, daemon), `aarch64-apple-darwin` (platform-macos). Type-check only; no native run |
| Release download check (`release.py download-check --tag v0.1.0 --skip-smoke`) | Pass: published checksums and embedded `BUILD.json` verified. The x86_64 binary smoke cannot execute on this aarch64 host |
| Update feed check (`deploy_updates.py verify`) | Pass after the user-agent fix below: HTTP 200, signed feed, **0 published updates** |

First full-suite attempt was 90/95 under parallel build load (S15 latency
budget, S9/R5/S20/S21 timing). Each failing case then passed alone, and R5 was
shown to fail the same way on the unmodified base build, so those are recorded
as load flakes, not candidate regressions. The strict repeat and the final
gate run were 95/95.

## What changed in this candidate

Robustness fixes, each covered by an existing or new real-process check:

- A connecting peer whose first resolved address blackholes now falls through
  to alternate addresses at the connect deadline (was: never connects).
  Verified with a real blackholed first address and loopback fallback.
- DNS lookups use a small worker pool and expired jobs release their admission
  slot; mDNS browse failures back off exponentially instead of 1 s warns.
- Approving a held (untrusted) TLS peer no longer delivers input it sent
  before the decision: the held connection closes and the trusted reconnect
  starts from a clean stream.
- Large clipboard transfers no longer abort when the Bulk queue fills on a
  slow link; chunks wait in a bounded per-connection backlog and retry as the
  queue drains. A stopped reader still aborts visibly (A6 semantics kept).
- An invalid config file no longer bricks daemon startup: the daemon starts
  with `role=off`, reports `invalid_config` in status, and accepts a live
  repair through `config.set` without a restart. CLI config commands remain
  strict.
- Linux X11: transient X errors no longer kill the capture or clipboard
  workers, and all five X connections now reconnect with bounded backoff after
  server death (re-grab, re-own selections), proven by killing and restarting
  Xvfb in the new probe mode. While the server is down, calls fail honestly;
  the screens API serves last-known values instead of a fake empty display.
- Windows: key/button/modifier/toggle release is best-effort when SendInput
  fails (UIPI, secure desktop), so releases cannot strand stuck keys in the
  sink ledger. Sleep/wake and session lock/unlock notices now reach the
  runtime (hidden-window power + WTS channel, cross-compiled only).
- Sleep and lock are tracked separately in the daemon: waking to a still
  locked screen no longer resumes the session.
- macOS: permission revocation can now surface — the fresh-process probe is
  authoritative per grant instead of an OR-merge that only ever reported
  grants (cross-target check only; no Mac run).
- Release: validated draft archives get build-provenance attestation
  (Sigstore) in the trusted publisher job, and `release.py download-check`
  re-downloads a published release, verifies checksums/manifests, and smokes
  the Linux binary. `deploy_updates.py verify` sends a user agent, fixing its
  HTTP 403 against the live feed.
- Dependencies adopt the five open Dependabot bumps with commit-pinned
  actions: checkout v7.0.1, upload-artifact v7.0.2, download-artifact v8.0.2,
  `toml` 1.1.8, `x11rb` 0.14.0; notices regenerated. The GitHub PRs remain
  open and are superseded by this branch.
- E2E coverage: new `S6b-trust-reject` and `S26-sigterm-release` scenarios.
  `A7-trust-lock-crash` and `R5-incoming-clipboard-cancellation` flakes were
  root-caused to runner/scenario timing races (lock-observation sampling;
  dead-peer heartbeat window blocked by the measure step) and fixed without
  weakening assertions.

## Not established

- No native macOS, Windows, or physical X11 desktop acceptance. All
  [matrix](../native-acceptance.md) native rows stay **Blocked**.
- No Docker interop (30 excluded; no Docker on this host), no Synergy 1.21.1
  peer retest, no native latency/CPU benchmark (S15 budgets are
  virtual-backend only).
- Windows notices and macOS permission changes are compile-verified only.
- The x86_64/macOS/Windows candidate builds, signing, notarization, and the
  attestation step itself need hosted CI on the exact candidate; no workflow
  was triggered by this work.
- The Sparkle feed remains empty; a signed update install/relaunch
  (SER-197) still needs a Mac and release signing keys.
