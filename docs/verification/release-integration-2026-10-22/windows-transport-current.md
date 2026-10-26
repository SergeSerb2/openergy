# Current Windows transport checks — 2026-10-22

## Result and source scope

The current runtime passed a fresh native x86-64 MSVC build, strict Clippy and
the scoped real pipe/TCP/TLS/virtual checks below. Every daemon used
`--platform virtual` in SSH session 0. This is not Windows desktop acceptance.

Runtime source archive: `c2d14a695ef9b4e4bcd9bd70c29ee393fe9286e2`.
It includes the identity/trust, IPC and receive-fairness changes. All **563**
files, sizes, hashes and the complete inventory were checked on Windows before
and after compilation and runtime checks. Source files were read-only and the
build target was outside the source tree. No source overlay was applied.

The first guardrail run exposed a test portability limit. The follow-up source
`db0d0bdc7bc9ebfe7c6d7d80629cf329d587acdb` differs in exactly one file:
`crates/net/examples/guardrails.rs`. A second full archive was independently
verified, made read-only and used to compile that updated probe. All **563**
files in each snapshot stayed unchanged. The daemon's compiler inputs are
identical between the snapshots; only the standalone probe changed.

| Artifact | SHA-256 |
| --- | --- |
| Runtime source archive | `321263293e3bdf9e3b7d35c04d066f6eeff655db7f011689d1d7f09735c8bd9b` |
| Runtime full-file manifest | `46c51d68371577804d29c31019703c4d4ab64c79fd01dbbb8ed6e5bc31e785eb` |
| Probe follow-up source archive | `3d40cbb92b2dbd556857d9954448c3860044bd9e5765ef6c5b6af1b1278baf8e` |
| Windows release daemon | `dbfb1d619a9ed04a90a17bc6509d1419fc2402aad5ebb05464fe2bc3297437fb` |

The daemon is **4,101,120 bytes**. This is its native Windows size, not a
comparison with another platform or a packaged release. The frozen executable
and all test binaries kept their expected hashes throughout the checks.

## Pinned build

The existing Rust **1.99.1** toolchain ran natively on
`x86_64-pc-windows-msvc`. No compiler fallback, bootstrap override, dependency
change or toolchain installation was used. The global default stayed
`stable-x86_64-pc-windows-msvc`.

The build used two jobs and BelowNormal priority. Strict **release**
all-targets Clippy passed for daemon, Windows platform, net, session, platform,
protocol, config and IPC. The locked native release build and all five net
example builds passed. The updated guardrail then passed its own locked
release build and strict net/all-targets release Clippy from the second archive.

```powershell
$env:RUSTUP_TOOLCHAIN = '1.99.1'
$env:CARGO_BUILD_JOBS = '2'
$env:CARGO_TARGET_DIR = Join-Path $Lab 'target'
$env:OPENERGY_ALLOW_PORT_24800 = '0'
$env:OPENERGY_DISABLE_DISCOVERY = '1'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
cargo +1.99.1 clippy --locked --release -p openergy-daemon -p openergy-platform-windows -p openergy-net -p openergy-session -p openergy-platform -p openergy-protocol -p openergy-config -p openergy-ipc --all-targets -- -D warnings
cargo +1.99.1 build --locked --release -p openergy-daemon -p openergy-platform-windows
cargo +1.99.1 build --locked --release -p openergy-net --example tls_peer --example priority_peer --example guardrails --example lifecycle --example trust_directories
```

The original build steps took 29.858 / 64.847 / 90.738 seconds for Clippy,
daemon and examples. The follow-up probe's Clippy/build took 6.653 / 22.116
seconds. These are build durations, not runtime performance measurements.

## Real-interface checks

- **Pipe identification and control:** identification level 2; protected
  current-user-only kernel DACL; actual CLI aliases; config save/apply;
  duplicate-daemon rejection; event/log subscriptions; fragmented requests;
  unknown-method error; 64 completed idle clients, 65th-client rejection and
  slot recovery; stop with a partial request; absent-daemon failure; and
  watched-parent shutdown all passed.
- **Identity and trust ACLs:** the actual offline CLI verified protected TLS
  directories, certificate/key/lock files and both trust roles. Repair of an
  intentionally broadened disposable key preserved its identity. Approval
  and revocation preserved private kernel ACLs.
- **Transport guardrails:** exactly one typed refused-connect event without
  a Connected event; input-budget refusal with peer preservation; reserved
  Control slow-peer closure; 120 ms handshake deadline; original cross-thread
  waker reuse; and the unchanged five-second flood watchdog passed. The flood
  check processed **3,753,984** actual framed messages before closure.
- **Lifecycle:** TLS and plaintext cycles passed, including frame delivery
  after stopping the listener, shutdown closure, and reuse of the original
  driver, handle and waker.
- **Retained trust directories:** **4/4** real mutual-TLS scenarios passed:
  retained-directory revoke/admission, current and retained read failures,
  reverse recovery and rejected reconfiguration preparation. Surviving peers
  continued bidirectional frame delivery and all child/listener cleanup passed.
- **TLS priority pair:** **10,485,760 bytes** and all **325** diagnostic probes
  crossed real TLS. At most one bulk frame appeared between input-priority
  frames; both processes exited zero. The reported echo p99 was 1,685 us for
  this diagnostic frame boundary, not physical input latency.
- **Virtual TLS and bounds:** two actual daemons generated distinct identities,
  saved mutual pins and reported trusted TLS. Synthetic virtual clipboard text
  crossed in both directions, followed by a true empty clear. An IPC request
  within 16 MiB whose serialized TOML exceeded 16 MiB was refused, preserving
  exact config bytes, live input settings and the TLS peer. A normal patch
  saved/applied and survived restart. Compact trust revocation preserved every
  other pin; oversized approval/import and absent revocation preserved the
  expected file hashes. Malformed fingerprints, invalid daemon options and
  absent-daemon commands returned the required failure exits.

The pipe/control/ACL drivers are the existing real-interface scripts. Only
fallback cleanup identity/hash guards were added to private copies of the
first two. The TLS/bounds driver is byte-identical to the retained round-5
driver, SHA-256
`ea28319f76c2c4aa4f44abb9890595be8b2b8ba49826b2a2d5dfdb93c6a3476c`.
The [machine record](windows-transport-current.json) retains every driver,
harness, source-manifest and result hash.

## Failed observations and probe correction

The original guard stopped waiting after two seconds. A separate run of the
unchanged real transport peer reported the correct `connect_refused` with no
delivered frame after **2,177 ms**. Windows' own TCP API reported
`ConnectionRefused` after **2,052 ms**. This reproduced an observation-window
limit, not a falsely successful transport connection.

The updated probe configures a five-second connector deadline, observes it for
at most six seconds, and now requires the typed **Refused** classification,
exactly once, with no Connected event. A generic timeout cannot pass. It
observed the Windows refusal after **2,027 ms**. On macOS the same updated
probe observed refusal immediately and passed all remaining guardrails.
The production connector defaults and runtime code were not changed.

A later wrapper attempt passed the guardrails, lifecycle and directory checks,
then failed because its own log markers entered a PowerShell return-value
pipeline used to capture fixture fingerprints. The wrapper logging was
corrected, and only the remaining priority/TLS/bounds checks ran in another
fresh fixture. The original failed attempt records remain failed; the machine
record lists the successful evidence for each verified group separately.

## Integrity, cleanup and limits

The final Windows audit found **zero owned processes** and **zero listeners**
in the assigned integration ports. Both complete source inventories and
archives, and every frozen binary, remained unchanged. The updated probe used
only the task's first build target as a compiler cache; the original source,
archive, frozen binaries and failed evidence were preserved. Earlier Windows
test fixtures were not changed.

Assigned loopback ports were 24973–24978, 25090–25093, 25241 and 26525–26527.
All config, TLS, pipe, app-data and temporary paths belonged to fresh private
fixtures. The checks did not touch port 24800, installed Openergy/Synergy,
native input/clipboard, services, tasks, startup entries, accounts, credentials
or desktop/windowstation settings. No unit tests or mocks were added.

The Unix-only full E2E runner and the flood-response latency matrix were not
run on Windows. This record does not establish native focus/input/clipboard,
sleep/wake, installation or permissions behavior. Positive Windows migration
and service/UI recovery, the two original security review clearances and
trusted signed/notarized distribution remain open. GitHub private reporting
setup remains deferred.
