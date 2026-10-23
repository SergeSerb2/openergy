# Identity-file contention — 2026-10-22

## Reproduction and cause

A process holding `.identity-lock` could stop the daemon's single session owner
during a live TLS change. A real virtual TLS pair and a real Unix advisory lock
reproduced the failure on source `e499eec1b097c996a39164d9773e9b903ee9d431`.
The external owner held the lock for 1.2 seconds. Server `config.patch` and
client explicitly pinned `connection.connect` each waited about 1.217 seconds.
Concurrent status requests waited about 1.112 seconds; the input probe failed.
The operation then changed the configuration and disconnected the old peer.
Holding the same lock without a mutation left status responsive.

Startup with an existing or missing identity also waited beyond the declared
300 ms budget. It left its IPC socket present before the IPC service was ready.
The baseline eventually recovered after lock release. All four regression cases
failed on that baseline, and every owned process/socket was cleaned up.

## Fix

Code commit: `9edf45d80503f5963ca7b553a3ff3120cf6296db`.
Network preparation now uses nonblocking identity-lock acquisition. Startup
returns a busy error and unwinds its owned socket. A live TLS change returns
`config_apply_failed` or `connect_failed` before committing configuration or
connection changes. Existing peers and held input remain intact until a
successful retry. Offline identity commands retain their blocking creator lock.

Readers still hold the creator lock. The key is installed before the
certificate, so reading around a held lock could expose an incomplete pair.
The fix refuses contention instead. A genuinely incomplete identity remains an
error and does not cause silent key replacement.

This removes a wait for a contended advisory lock. File reads, writes and sync
operations remain synchronous. No worker or dependency was added.

## Real-interface verification

The committed [contention command](../../../crates/daemon/scripts/verify_identity_contention.py)
uses actual daemons, public IPC, loopback TLS peers and a separate lock holder:

```sh
python3 crates/daemon/scripts/verify_identity_contention.py \
  --openergy target/release/openergy --port-range 27200-27219 \
  --report /tmp/identity-contention.json
```

The same script passed **4/4 cases** on the fixed binary. With the lock held,
live mutations returned errors in 0.222–0.286 ms; concurrent status requests
took 0.343–0.351 ms and established input probes took 0.506–0.513 ms. Exact
config/key/certificate bytes, peer name/connection time and the original held
key stayed unchanged. Both retries succeeded after lock release, kept the
identity, and released old held input on reconnect.

Both startup cases exited nonzero in about 10.7 ms and removed their owned
socket. Neither wrote a partial identity. After lock release, retries loaded the
original identity or created a complete new one. A separate unlocked partial-pair
check refused startup without changing the existing key. These are finite local
samples against a 300 ms budget, not a general latency guarantee.

The existing trust-contention script now clears its pass flag if a later retry
raises an exception. Earlier recorded runs completed their retries, but future
exceptions must fail the case. The corrected script passed **6/6 cases** on this
binary, including 24 concurrent identity creators with one stable fingerprint.
The expanded daemon/CLI audit passed **15/15** and IPC deadline checks passed
**19/19**. The locked release build, strict workspace/all-targets release Clippy
and formatting passed. The daemon decreased from 3,674,208 to 3,674,160 bytes.

The full virtual suite passed **89/89**, with zero cleanup failures. **30 Docker
scenarios were explicitly excluded**, not counted as passes; the Docker socket
was unavailable. The S15 plaintext virtual-loopback check measured server/client
idle CPU at 0.050%/0.017% over separate 60-second windows, with peak resident
memory of 9,437,184/9,371,648 bytes. The 1,000-move observation had p50 0.795 ms,
p95 0.872 ms and p99 0.968 ms. This is an external IPC-to-receiver upper bound,
not physical input-to-screen latency or a Synergy speedup comparison.

Exact measurements, binary hashes, script hashes and 112 tracked Rust/Cargo/
toolchain file hashes are recorded in the [machine record](identity-lock-contention.json).
No Rust source changed between the build and these checks.

## Scope

Checks used private temporary state, the virtual backend, assigned loopback
ports and production-port denial. No installed process, live input connection,
native clipboard, permission, login item, service or task was changed. Builds
used two Cargo jobs and reduced priority. No unit tests or mocks were added.
This is not new Windows or Linux runtime acceptance. Native and distribution
gates remain open. GitHub private reporting setup remains deferred.
