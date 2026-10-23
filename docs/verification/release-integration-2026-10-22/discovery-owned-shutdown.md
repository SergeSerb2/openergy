# Owned discovery shutdown — 2026-10-22

The integrated shutdown fix passes **4/4 real child-process cases** at source
`f0f2ee5b7701add5a6fcdf040b563a072a905865`. The parent reran the probe after
integrating the independently reviewed candidate
`330d83f2f168644035c4a99ef3484675214d0e65`.

Previously, a temporary command-wake socket could fail before `Exit` was
submitted. The cleanup sequence returned early, and the mDNS worker retained
its own command sender. The original private-child probe reproduced two threads
and five UDP sockets still alive seven seconds after dropping discovery.

`Discovery::shutdown` now submits `Exit` directly. The dependency handles
advertisement goodbyes and browse cleanup when it processes that command.
Errors remain visible. Submission success is not a worker join, and a full
upstream command queue can still reject `Exit`.

## Integrated real-process check

Each child stayed alive while the parent used `ps` and `lsof` to inspect its
worker and sockets. The two exhaustion cases lowered only that private child's
soft descriptor limit to 128 and opened `/dev/null` until `EMFILE`. The parent
then requested cleanup, released those descriptors and observed the child.
Explicit shutdown cases first resolved their own ordinary advertisement over
loopback. No sentinel worker was used as proof of owned-worker shutdown.

All four cases returned from **two threads/five UDP sockets** to their original
**one thread/no UDP sockets**:

| Case | Observed cleanup time | Child alive during final inspection |
| --- | ---: | --- |
| Normal Drop | 50 ms | Yes |
| Normal explicit shutdown | 50 ms | Yes |
| Descriptor-exhausted Drop | 823 ms | Yes |
| Descriptor-exhausted explicit shutdown | 832 ms | Yes |

These times include the observer's sampling interval. They are cleanup
observations, not an input-latency benchmark or a hard shutdown deadline.

The pinned Rust 1.99.1 debug example build passed with `--locked --offline`,
two jobs, `nice -n 15` and macOS deployment target 15.0. The frozen example's
SHA-256 is
`58f75b4342fe85cdf0eba6963f6b7d2ef4b90b1515ddecd628bf1111e52bf48e`.
This example is not the release daemon. All **114 tracked Rust, Cargo and
toolchain inputs** matched the exact source commit after the run. The final
audit found no remaining probe children or assigned UDP listeners.

The [machine record](discovery-owned-shutdown.json) contains commands, source
hashes, all four observations, raw-report hashes and the postflight audit.
Original local outputs remain private. The candidate's
[earlier evidence and known replay failure](../../../crates/net/DISCOVERY-REPLAY-20261022.md)
retain their own scope. Public copies identify path and whitespace normalization.

## Review and remaining work

The independent review checked the exact shutdown and DNS cancellation
candidates. It found no new concrete P1/P2 regression. It audited the earlier
runtime reports and locked upstream source; it did not rerun the processes.
The four checks above are the parent's separate integration run.

The retained 800-service namespace still fails useful-snapshot and selected-peer
freshness/removal acceptance. Both earlier full DNS probes still exit 1 on the
refused-localhost control. Those failures are separate active work.

The provider stopped the separate upstream cache-expiry reproduction for
possible cybersecurity risk. That verification remains open. Native
input/clipboard, positive Windows migration, original independent security
acceptance and trusted signed/notarized distribution also remain open. No live
sharing connection or installed state was changed.
