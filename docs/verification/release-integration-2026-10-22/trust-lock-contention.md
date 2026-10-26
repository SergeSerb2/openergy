# Live trust-file contention — 2026-10-22

## Reproduction and cause

A real process holding `.trust-lock` could stop the daemon's single session
owner inside a live trust update. The baseline source was
`2b930eeb7adf8c371c683b953b5b356eecdb168a`. A private virtual pair and a real
Unix advisory lock reproduced the stall through public IPC. No mock or product
function was substituted.

The external owner held the lock for 1.2 seconds. Live server approval, client
approval, revocation, outbound held-peer approval and inbound mutual-TLS
held-peer approval all waited for release. Status calls took about 1.1 seconds;
established input transfers also took about 1.1 seconds. Holding the lock
without issuing a trust mutation left status and input responsive. This
distinguished a synchronous lock wait from general filesystem/IPC interference.

## Fix

Code commit: `4b5f762799f0deb98e77e1e8f2af523721133807`.
Live trust updates now use nonblocking lock acquisition. A busy store returns
`trust_failed` with a retry message before changing pins or live connections.
The same operation can succeed after release. Both held-peer approval directions
use this path. Offline CLI updates keep their blocking lock and read/merge/write
transaction, preserving concurrent writers.

The implementation uses Rust's
[nonblocking file lock](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock),
available in the pinned toolchain. Unix lock opens also reject symlink traversal
and do not block on a FIFO substituted during open; opened handles must refer
to regular files. The existing empty-directory lock migration remains. The
path-swap race was reviewed in source, not forced in this runtime run.

This removes waiting for a contended advisory lock. Ordinary filesystem reads,
writes and sync operations remain synchronous; this is not a claim that all
storage operations are nonblocking. No new dependency or worker was added.

## Real-interface verification

The committed [contention command](../../../crates/daemon/scripts/verify_trust_contention.py)
runs real daemons, public IPC, actual TCP/TLS virtual peers and a real OS lock:

```sh
python3 crates/daemon/scripts/verify_trust_contention.py \
  --openergy target/release/openergy --port-range 26800-26819 \
  --report /tmp/trust-contention.json
```

The final script failed all five live cases on the baseline; its offline control
passed. The fixed binary passed **6/6 cases**. With the lock still held, live
mutations returned errors in 0.128–0.440 ms, status calls took 0.169–0.301 ms,
and the three established input transfers took 0.312–0.414 ms. These finite
loopback samples passed the declared 300 ms response budget; they are not a
general latency guarantee. Pins stayed byte-for-byte unchanged on busy, retries
succeeded after release, and every owned daemon/socket was cleaned up.

The offline case ran 24 concurrent approvals, then 12 revocations mixed with
12 new approvals. Both role lists retained exactly the expected 24 pins. It
also ran 24 concurrent identity creators; all returned the same fingerprint,
with private-key mode `0600`. The same control passed against the baseline.

The locked release build, strict workspace/all-targets release Clippy and
formatting passed with Rust 1.99.1 and the macOS 15 deployment target. Exact
hashes, original failures and affected-suite results are in the
[machine record](trust-lock-contention.json).

The expanded daemon/CLI audit passed **15/15**, the incomplete-request regression
passed **19/19**, and the full virtual suite passed **89/89** with zero cleanup
failures. **30 Docker scenarios were explicitly excluded**, not counted as
passes. The daemon grew from 3,674,144 to 3,674,208 bytes: 64 bytes in this build.

## Scope

Runtime checks used macOS 27.2, the virtual backend, private temporary state,
assigned loopback ports and production-port denial. No installed process, live
input connection, native clipboard, native permissions, login item, service or task was
changed. Builds used two Cargo jobs and reduced priority. No unit tests or mocks
were added. This is not new Linux or Windows runtime acceptance. Native and
distribution gates remain open; GitHub private reporting setup remains deferred.
