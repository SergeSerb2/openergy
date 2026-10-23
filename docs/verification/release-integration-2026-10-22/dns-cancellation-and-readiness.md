# DNS cancellation and connection readiness — 2026-10-22

The integrated real-interface probe exits **0** at source
`07adb0a3cdc4cff1aa77ce31dbe078a48db56bd5`. It verifies eight DNS queue-recovery
batches, ten typed refusal checks, healthy IPv4 fallback and IPv6 connections,
bidirectional frame exchange, and reuse of the original driver and handle.

## Two concrete fixes

Queued hostname jobs previously continued into OS lookup after cancellation or
deadline expiry. Jobs now share cancellation and deadline state with their
pending requests and check it before lookup. Skipped work still returns one
completion. The existing 16-job budget counts active OS work until completion;
an active OS lookup remains uncancellable. Late results cannot create stale
connections or duplicate request failures.

The first full probe exposed a second bug on both the baseline and cancellation
candidate. A refused `localhost` connection emitted `Connected`, then
`Disconnected`, instead of one typed refusal. Real tracing showed a Darwin
readiness event for a refused IPv6 socket arriving after address fallback had
replaced it with a pending IPv4 socket under the same connection token.
The replacement had no `SO_ERROR`, but `peer_addr` reported `NotConnected`.

TCP completion now requires both no socket error and successful `peer_addr`.
`NotConnected` and Unix `EINPROGRESS` retain the pending state and original
deadline. Other errors keep the existing fallback and typed failure handling.
The fix adds no worker, dependency, product sleep or resolver replacement.

The implementation's [failure trace and unchanged original-probe control](../../../crates/net/examples/dns_connection_c46e00d5-evidence/README.md)
retain their exact candidate scope. Both original failed reports remain
byte-identical. The unchanged original probe and expanded probe both passed
against the implementation candidate; the parent reran the expanded probe after
integration with the separate discovery shutdown fix.

## Integrated verification

The parent used pinned Rust 1.99.1, two low-priority jobs, macOS deployment
target 15.0 and a frozen debug example. It exercised the real `NetDriver` API,
standard OS resolver and private loopback sockets on TCP 25100–25119.
Discovery, TLS and native backends were disabled in this focused probe.

- Eight batches each admitted 16 obsolete jobs and rejected request 17.
  Each produced one failure per obsolete request, recovered admission, opened
  no stale TCP connection and exchanged real frames with a healthy listener.
- Ten requests across two unused ports tested numeric IPv4, numeric IPv6 and
  repeated `localhost`. Every request produced exactly one `connect_refused`
  event and no false `Connected` event. Exclusive binds and direct OS connects
  checked that both loopback addresses were unused and refused connections.
- Driver reset, reconfiguration, rejected configuration, original handle/waker
  reuse and handle rejection after Drop passed. Healthy IPv4 fallback and IPv6
  paths exchanged frames in both directions.

The full probe completed in 0.660 seconds. This is a local test duration, not
an input-latency or slow-DNS benchmark. Workspace formatting and strict
all-target release Clippy passed. All **115 tracked Rust, Cargo and toolchain
inputs** remained unchanged. The final audit found no assigned TCP listeners.
The [machine record](dns-cancellation-and-readiness.json) preserves the complete
output, commands, input/artifact hashes and exact exit codes.

An independent source/evidence review found no concrete P1/P2 issue in the
cancellation and readiness candidates. It checked completion accounting,
fallback state, TLS/pin preservation, deadlines, stale readiness and historical
report integrity. It did not independently run the binaries.

The pinned release daemon/E2E-runner build also passed. The daemon is
**3,707,680 bytes**, 160 bytes larger than the preceding shutdown-only build.
SHA-256: `aaa10f3e5f9e0a38ac9fba2737dcd13d486c177fc580a5924a18d4f10552f434`.

The full virtual-daemon run completed in 208.849 seconds with **88/89 passed**,
30 Docker exclusions and zero cleanup failures. `P07-hostile-field-bounds`
failed before executing its steps because no free port was available in the
configured ten-port range. That run's exit status remains **1**. A focused retry
of only that case on the separate 25900–25999 range passed **1/1**, exit **0**.
This is coverage across two runs, not a passing original full batch. The reason
for the earlier range's exhaustion was not established. All 235 source/scenario
inputs stayed unchanged; no fixture processes or assigned listeners remained.

## Remaining work

Native Windows verification is pending. Retained discovery replay remains
separate active work. Deterministic slow-DNS timing and cache-expiry runtime remain unverified.
The separate cache-expiry test was provider-stopped and was not rerouted.

Native input/clipboard, positive Windows migration, the original independent
security/provenance acceptance and trusted signed/notarized distribution remain
open. Live sharing and installed state were preserved. GitHub private reporting
remains deferred by the maintainer.
