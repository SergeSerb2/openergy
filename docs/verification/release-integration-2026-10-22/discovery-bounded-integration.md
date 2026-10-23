# Bounded discovery integration — 2026-10-22

The integrated discovery source passed the real custom-port loopback flows:
**14/14 cancelled-startup checks, 19/19 retained-discovery checks, and 4/4 owned
shutdown cases**. Release example builds, strict workspace/all-target Clippy,
strict vendor-library Clippy, workspace formatting, source verification and
notice freshness passed. No unit tests or mocks were added or run.

The exact tested source is `dddf9fef012f9d56ea6435be36bd5c74a08087d7`.
Candidate `884ff876` was integrated as `efca42e`; its cancelled-startup fix
`da668c3` was integrated as `dddf9fe`. The source/evidence review of the final
candidate found no concrete P1/P2 issue and closed the prior startup finding.
That review did not run binaries. This parent run supplies separate runtime
integration evidence. [Exact record and raw flow output](discovery-bounded-integration.json).

## Resulting behavior

The existing mDNS worker maintains a coalesced view of at most 64 selected
resolved services. A paused caller receives a complete replacement view, so
updates and removals do not accumulate in the subscription event channel.
Retained replay skips incomplete services and avoids an alias backlog.
Cancelled startup preserves the previous live subscription and leaves no
cancelled worker-map state. Shared-host goodbye handling preserves addresses
still used by another local registration. Direct Exit shutdown ownership stays
intact.

The ordinary public-API probe retains 800 advertisements. Standard browse still
hits its unchanged 1,000-event overflow contract. Limited browse stays open at
64 entries; the selected peer's changed fields remain fresh, removal remains
visible, and later new services can use a freed slot. The separate startup flow
queues 32 cancelled distinct service types and checks live limited/standard
replacement identity, later updates, stop cleanup and shutdown acknowledgement.

All four shutdown children stayed alive while their owned worker thread and UDP
sockets disappeared. This observes actual cleanup instead of treating command
submission or an acknowledgement alone as proof. The check includes the existing
child-only descriptor exhaustion cases. No global resource limit was changed.

## Frozen artifacts and execution

| Release example | Bytes | SHA-256 |
| --- | ---: | --- |
| discovery_recovery | 1,025,456 | `37d9f3971965722837e80ab0795b5792c3fcfd6a22c07c181708886f239ec475` |
| discovery_owned_shutdown | 923,392 | `bb28bbc34798807b542b2ec73da6fc6c57224eca496dde07c80a40fa9bc2fad7` |

Builds and checks used Rust 1.99.1, deployment target 15.0, nice 15 and two jobs.
The frozen release examples ran sequentially. All 129 tracked Rust/Cargo/toolchain
inputs stayed unchanged. Startup took 13.386 s, retained flow 40.946 s and the
four observed shutdown cases 5.188 s in total. These are probe durations, not
network latency or a product performance benchmark.

Only assigned loopback UDP ports 25090–25097 were used, plus the library's
existing OS-assigned command-wake sockets. Postflight found no assigned UDP
listener or owned fixture executable. No live daemon, input/clipboard sharing,
production 24800/5353 endpoint, installed config/state/TLS identity, service,
account, TCC, firewall, Docker or publication was used or changed.

The [vendored dependency-policy record](vendor-dependency-policy.md) separately
checks the exact candidate's combined graph and five platform graphs. The
integrated policy/vendor inputs match that snapshot.

## Remaining limits

This bounds delivery state. It does not bound the whole DNS cache, timers,
query work or variable record bytes. Replay can still scan the cache. Selection
has no automatic refill guarantee from unselected cached services. Existing
baseline and intermediate failures remain preserved in the candidate evidence.

No full combined daemon E2E or fresh native Windows discovery run was completed
on this integrated source before the requested pause. The IPC candidate remains
separate because independent review found an import-budget gap. Slow DNS,
provider-stopped expiry testing, native input/clipboard, positive Windows
migration, original independent security/provenance acceptance and trusted
signed/notarized distribution remain open. These checks do not clear those gates.
