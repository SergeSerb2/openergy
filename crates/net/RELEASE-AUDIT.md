# Net release audit, 2026-10-21

Scope: `crates/net` only. Rust 1.99.1 on macOS. The crate's existing MSRV
is unchanged. No unit tests or mocks were added. All runtime checks used
disposable files and real loopback sockets. No installed application,
production state, native input, clipboard, TCC, LAN discovery, SSH, or port
24800 was used.

## Fixes

- Explicit `ConnectPinned` could connect in plaintext when TLS was disabled.
  It now fails before opening TCP. The same check runs after DNS resolution,
  because configuration can change while resolution is pending. Explicit pins
  still authorize their matching certificate independently of the trust store;
  daemon retry/revocation policy remains the daemon owner's responsibility.
- Established Control traffic took priority over queued Input. The scheduler
  now gives Input priority after setup, while Control retains priority during
  setup and slow-peer draining. Partly transmitted frames stay intact.
- Control-only queues bypassed the 4 MiB slow-peer check. All non-Bulk queue
  pressure now starts the fixed watchdog, and further Control messages cannot
  extend it. Bulk admission and the global mailbox budget remain bounded.
- Repeated `Close` requests restarted the 250 ms close deadline. The first
  request now sets that deadline once.
- Identity readers could observe `key.pem` before the creator renamed
  `cert.pem`. All readers now take the same kernel lock as creators before
  checking the pair. Partial or corrupt identities still fail without rotation.
  Dangling identity links count as existing entries and are never regenerated.
- The TLS directory previously used default Unix directory permissions, and
  Windows key permissions were a no-op. The directory is now 0700 on Unix;
  identity files are 0600. Active trust files are also restricted on load.
  Identity, trust and lock links/special files are refused. Windows uses
  protected DACLs for the current process user, including at temporary-file
  creation before any private bytes are written. Existing ACLs are repaired.
  Filesystems without persistent Windows ACLs fail closed. Imported Synergy
  files remain read-only inputs and keep their original permissions.
- Failed exclusive temporary-file creation previously deleted the pre-existing
  colliding file during error cleanup. Cleanup now removes only a file this
  writer created. Atomic rename, file fsync, Unix directory fsync, and kernel
  writer locks remain. Trust mutations publish their in-memory result only
  after the required saves succeed, and do not rewrite unchanged role files.
- PEM parsing now uses `rustls::pki_types::pem::PemObject`. The direct
  `rustls-pemfile` dependency was removed in response to unmaintained advisory
  RUSTSEC-2025-0134. No advisory ignore was added.

The previous verification document's machine-specific temporary path was
replaced with a placeholder.

The Windows implementation follows Microsoft's
[protected DACL API contract](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setnamedsecurityinfow)
and checks the volume's
[persistent ACL flag](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationbyhandlew).

## Build and static checks

The parent owns the root manifest and lockfile. To avoid changing either in
this worktree, validation used an isolated source copy at
`/tmp/oe-net-validation-current`: `git archive HEAD` plus the updated
`crates/net` directory. Only that copy's lockfile was refreshed, offline.
The target directory below is this worktree's absolute `target` path.

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="<worktree>/target"
nice -n 15 cargo build --offline \
  --manifest-path /tmp/oe-net-validation-current/Cargo.toml \
  -p openergy-net --examples -p openergy-daemon --bin openergy \
  -p openergy-e2e --bin openergy-e2e
nice -n 15 cargo clippy --offline \
  --manifest-path /tmp/oe-net-validation-current/Cargo.toml \
  -p openergy-net --all-targets --no-deps -- -D warnings
cargo fmt -p openergy-net -- --check
git diff --check
```

All pass. No TypeScript is in this scope.

The exact `windows_permissions.rs` source also passed Rust type checking in
a temporary crate with its declared windows-sys features, targeting
`x86_64-pc-windows-msvc`. This checks API signatures, not Windows behavior.
The full net cross-check was attempted:

```sh
nice -n 15 cargo check --offline \
  --manifest-path /tmp/oe-net-validation-current/Cargo.toml \
  -p openergy-net --target x86_64-pc-windows-msvc
```

It stops in ring's C build because this Mac lacks Windows C headers:
`fatal error: 'assert.h' file not found`. Native Windows build and ACL checks
remain an integration gate. The Windows owner can run the retained real CLI
flow in a fresh PowerShell process:

```powershell
powershell -NoProfile -File crates/net/examples/verify_permissions.ps1 -Openergy <isolated-build>/openergy.exe
```

This checks initial directory/certificate/key/lock ACLs, repairs a deliberately
broadened disposable key ACL without changing its fingerprint, and checks
trust approval/revocation ACLs. It uses a new temporary config/TLS/socket
namespace and invokes only offline commands. It was not run on this Mac.

## Real socket regressions

Each probe uses the public NetDriver/NetCommand interface and a real TCP
peer. The first four cases failed on the original parent implementation:
plaintext with a pin was accepted, Control overtook Input, Control pressure
did not close, and repeated Close requests exceeded 650 ms.

```sh
target/debug/examples/audit_transport pinned-plain /tmp/oe-net/pin 25095
target/debug/examples/audit_transport pinned-reconfigure /tmp/oe-net/dns 25096
target/debug/examples/audit_transport priority /tmp/oe-net/priority 25097
target/debug/examples/audit_transport control-pressure /tmp/oe-net/pressure 25098
target/debug/examples/audit_transport close-deadline /tmp/oe-net/close 25099
```

All five pass after the fixes. Representative pressure close: 117 ms with a
120 ms heartbeat. Repeated-close case: 250 ms. These are watchdog checks,
not latency or performance claims. The reconfigure case queues a real
localhost lookup while TLS is enabled, disables TLS, then checks that the
pending pin fails without a plaintext connection.

## Real CLI identity and trust

```sh
python3 crates/net/examples/verify_release.py --openergy target/debug/openergy
```

All checks pass:

- Stable fingerprint and private permissions, including permission repair.
- 24 simultaneous real fingerprint CLI processes share one identity.
- A real fingerprint reader waits on the kernel lock while a writer is paused
  between key and certificate publication, then reads the complete pair.
  Before the fix, this reader exited early with `No such file or directory`.
- Partial identities, dangling key links and corrupt PEM fail without rotation.
- 24 concurrent real approval commands retain all pins in both role files;
  revocation removes the selected pin from both.
- Import deduplicates SHA-256 records, ignores legacy SHA-1, and does not
  change source bytes or permissions.
- A deliberately colliding temporary file survives a failed real CLI save;
  no destination trust file is published on that failure.

## TLS, framing and held trust

```sh
python3 crates/net/examples/verify_transport.py \
  --openergy target/debug/openergy --peer target/debug/examples/tls_peer \
  --port-base 25088
```

All pass: trusted bidirectional TLS frames; unknown server hold and API
approval; unknown client hold and correct-role approval; external trust-file
reload; wrong pin; held deadline; explicitly configured plaintext; oversized
prefix rejection; import/dedup/invalid CLI cases. A real TCP relay captured
899 encrypted TLS bytes for the wrong-pin case and no plaintext Synergy
hello. Held peers delivered no application frame before approval or expiry.

## Existing daemon E2E scenarios

The external runner used unchanged copies of the following scenarios in
`/tmp/oe-net-targeted-scenarios`:

```sh
target/debug/openergy-e2e run \
  --scenario-dir /tmp/oe-net-targeted-scenarios \
  --port-range 25080-25099 --lab-port-range 25100-25119 \
  --report /tmp/oe-net-targeted-final.json --keep
target/debug/openergy-e2e run --scenario 'S6-*' \
  --port-range 25080-25099 --lab-port-range 25100-25119 \
  --report /tmp/oe-net-s6.json --keep
```

**13/13 plus S6 pass:** A11 input backpressure release; A3 heartbeat close
flush; A6 clipboard backpressure; A7 killed trust-writer lock recovery; A9
trusted TLS status; S13 flood, garbage, oversized, truncated hello, truncated
frame and unknown frame; S21 fingerprint/import; S7 wrong pin; S6 held
approval and reconnect.

An exploratory new pin-with-plaintext daemon scenario was discarded: the
existing IPC API already rejects it with `invalid_params`, and the external
runner deliberately refuses expected-error connection probes. No runner
safety check was weakened. The retained socket probe verifies the transport
boundary directly, including absence of a TCP connection.

## Discovery and production-port policy

Two real discovery processes advertised and browsed on loopback UDP 25099,
with service port 25098:

```sh
target/debug/examples/discovery browse /tmp/oe-net/mdns 25099 25098
target/debug/examples/discovery advertise /tmp/oe-net/mdns 25099 25098
```

Both pass and resolve the expected name, host, port and 16-hex display hint.
The wrapper caps snapshots at 64 peers and once per second, validates the
fingerprint hint, and does not connect or approve trust. LAN behavior was
not exercised. No concrete discovery vulnerability was found in this audit.

The net layer requires `OPENERGY_ALLOW_PORT_24800=1` for protected-port
listeners, advertisements, and (after the follow-up below) outbound sockets.
Explicit `0` always denies access. The parent reports that the daemon owner
now enables native startup only when the variable is unset; virtual startup
never opts in automatically. The external E2E runner's unconditional
production-port refusal is unchanged.

## Follow-up: outbound guards and causal input ordering

The outbound request entry point now calls `check_port` before numeric
connects or DNS submission. `add_outbound` checks again after resolution,
immediately before creating TCP. `retry_address` checks every alternate
address before its TCP call and propagates a denial instead of replacing it
with the previous address's error. All `TcpStream::connect` sites are guarded.

No guard check was tested against port 24800. The real-socket proof used a
disposable source copy at `/tmp/oe-net-guard-redirect`. Only the protected
port literal inside `check_port` was changed from 24800 to **25094**. The
environment-variable name and decision logic stayed unchanged. The retained
example was built from that copy:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="<worktree>/target" nice -n 15 \
  cargo build --offline --manifest-path /tmp/oe-net-guard-redirect/Cargo.toml \
  -p openergy-net --example audit_transport
OPENERGY_ALLOW_PORT_24800=0 target/debug/examples/audit_transport guard-denied /tmp/oe-net-guard 25094
env -u OPENERGY_ALLOW_PORT_24800 target/debug/examples/audit_transport guard-denied /tmp/oe-net-guard 25094
OPENERGY_ALLOW_PORT_24800=1 target/debug/examples/audit_transport guard-allowed /tmp/oe-net-guard 25094
```

All three pass. Denial covers both numeric and hostname requests and proves
that the owned listener accepts no TCP connection. Explicit opt-in connects
through real localhost resolution and sends a framed payload to that listener.
As a separate fallback check, the initial request guard was removed **only in
that disposable redirected copy**. With explicit `0`, numeric and resolved
requests still fail before TCP in `add_outbound`. This check also passes.
No environment variable was changed while a resolver thread was running.

The ordering review found a concrete counterexample to unconditional Input
priority. In one real `virtual.inject` batch, a held key can leave and reenter
the same peer, then press the same key again. `SendControl` carries the old
safety KeyUp, while Leave, Enter and the new KeyDown use Input. The latter
messages could overtake the old KeyUp. The new
`../../tools/e2e/scenarios/N2-queued-release-reentry.toml` fails its strict DKUP expectation
against commit `a57954c`; report: `/tmp/oe-net-causal-a579-final.json`.

Typed Control KeyUp, MouseUp, Enter and Leave now record their queue order.
Older Input drains before that Control message, and newer Input waits until
it passes. Thus the real sequence is old KeyDown, safety KeyUp, Leave,
reentry, new KeyDown. Ordinary Control, such as heartbeats, can still be
overtaken by Input. Handshake Control priority, slow-peer Control draining,
Input coalescing and partly transmitted frame handling remain unchanged.
This is an ordering fix for the reproduced case, not a scheduler redesign.

N2 passed five consecutive runs. Final verification used a fresh target
directory to keep the baseline comparison artifact separate:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/oe-net-followup-target nice -n 15 \
  cargo build --offline --manifest-path /tmp/oe-net-validation-current/Cargo.toml \
  -p openergy-daemon --bin openergy
target/debug/openergy-e2e run \
  --openergy /tmp/oe-net-followup-target/debug/openergy \
  --scenario-dir /tmp/oe-net-followup-scenarios \
  --port-range 25080-25099 --lab-port-range 25100-25119 \
  --report /tmp/oe-net-followup-verified.json --keep
```

**8/8 pass:** N2, both S2 scenarios, A3 heartbeat close, A11 pressure release,
A12 restart, S6 trust approval and S7 wrong pin. The five transport probes
also pass using examples built in this fresh target. A real two-process TLS
priority transfer sends all 10 MiB, receives 321 Input frames, and observes
at most one Bulk frame between Input frames. These counts verify scheduling;
they are not a new performance claim. Formatting, diff checks and net
Clippy with `--all-targets --no-deps -- -D warnings` pass.

## Parent integration

The parent reports that `a57954c`, the root rustls-pemfile removal, and the
lockfile refresh are integrated. This follow-up changes no dependency or
MSRV. Keep the daemon owner's explicit-pin revoke/retry and native opt-in
policies. The parent moved N2 into `tools/e2e/scenarios/N2-queued-release-reentry.toml`
so standard integration and CI runs include it. Run native Windows ACL checks and the
parent's full suite after integration. No release-readiness claim is made
for those outstanding checks.
