# Live pin revocation cutoff — 2026-10-22

## Reproduction

The follow-up audit found that successful live revocation of an explicitly
pinned TLS connection could still accept input during graceful close. A real
virtual TLS pair reproduced this on source
`f9e3f25ee1fe1647d7c26224c4eba3ab6c92cca0`: in five consecutive trials, one
key-down submitted after the `trust.revoke` reply reached the receiving sink.
A stored-trust control, without an explicit connection pin, rejected late
input in all five trials. All tests used synthetic keys and private state.

Changing the trust store marks it for reload. A connection authenticated with
an explicit pin still satisfies its own cached pin check, so that reload does
not immediately remove it. The revocation handler used graceful close, whose
read path could still deliver messages before closing. Clearing the runtime's
retry pin alone did not remove the live connection's pin.

The new [A13 external scenario](../../../tools/e2e/scenarios/A13-pinned-revoke-cutoff.toml)
failed on that baseline at the exact post-revocation ledger assertion. It
holds a key, revokes the explicit pin, sends another key after the reply, and
requires the receiving ledger to contain only the original key's release and
release-all. It also requires empty pressed state, a removed stored pin and
new trust approval before reconnecting.

## Fix and verification

Fix commit: `bcfe7217ba7374cfe81678e6404e6315bd920eb5`.
After successful trust persistence, the daemon immediately removes matching
live connections rather than starting graceful close. The existing disconnect
path releases held input before the IPC reply. Unrelated peers are not removed.
This adds no dependency, wire field or worker.

A13 passed with the fix. The separate pinned TLS probe accepted no late input
in five of five fixed trials and left no held keys. The existing real daemon
audit was expanded to check that an unrelated fingerprint and a real trust-file
persistence refusal preserve the valid connection, held input and exact trust
files. It then removes the failure fixture and verifies successful revocation,
released input and renewed approval. The expanded audit passed **15/15**.

The preceding incomplete-request regression also passed **19/19** against the
new binary. Locked release build, strict workspace/all-targets release Clippy
and formatting passed with Rust 1.99.1 and the macOS 15 deployment target.
The daemon executable remained 3,674,144 bytes. Exact hashes, original failure,
controls and the broader virtual-suite result are in the
[machine record](trust-revocation-cutoff.json).

The full virtual suite passed **89/89** with zero cleanup failures; **30 Docker
scenarios were explicitly excluded**, not counted as passes. S15 recorded
9,355,264-byte peak RSS for each daemon. Server/client idle CPU was
0.03331% / 0.01665% over separate 60-second windows. For 1,000 virtual moves,
external IPC-to-receiver-ledger latency was p50 0.675 ms, p95 0.839 ms and
p99 1.213 ms, with a 2.921 ms maximum. This is a plaintext loopback observation,
not physical latency or a comparative speedup. The p99 is higher than the
preceding finite run; this does not establish a change caused by revocation.

Reproduce through real binaries and sockets:

```sh
target/release/openergy-e2e run --scenario 'A13-*' \
  --openergy target/release/openergy --port-range 26300-26320 \
  --lab-port-range 26330-26350 --report /tmp/revoke-cutoff.json
python3 crates/daemon/scripts/verify_daemon_audit.py \
  --openergy target/release/openergy --port-range 26400-26419 \
  --report /tmp/daemon-audit.json
```

## Limits

These checks used macOS 27.2, the virtual backend, private config/TLS/control
paths and assigned loopback ports. No installed process, live input connection,
native clipboard, permission setting, service, task, login item or port 24800
was touched. No unit tests or mocks were added. The real persistence refusal
used a nonempty lock directory inside the disposable node's own TLS directory;
the fixture and original lock were restored before cleanup.

A fresh Linux follow-up could not run: Docker's configured Unix socket was
absent. Docker was not restarted. This does not establish new Linux or Windows
runtime acceptance, native input behavior, independent security clearance or
trusted distribution. Native acceptance stays a release gate. GitHub private
reporting setup stays deferred at the maintainer's request.
