# Discovery recovery: first iteration — 2026-10-22

## Intermediate result and open finding

Source `ac5c45081a9e060392cb850eddd625f07b591bfe` fixes one reproduced permanent
discovery outage, but independent review found that a large retained cache can
still overflow replacement subscriptions. This is an intermediate record. It
does not approve the discovery change as complete or clear any release gate.

mdns-sd 0.21.6 closes a browse subscription when its 1000-event channel fills.
The previous wrapper treated a disconnected receiver like an empty receiver,
so it retained stale peers and stopped observing advertisements. The change
distinguishes these states, clears the unreliable snapshot, and retries on
the existing worker after a one-second delay. Shutdown cancels pending retries.
The 64-peer cap, 256-event drain budget, one-second snapshot throttle and
display-only fingerprint hints remain in place. No dependency or worker was added.

## Real multicast checks

The [new command](../../../crates/net/examples/discovery_recovery.rs), source
`9f68fd6237bfc1b4fb14242ec80c7f2248535eda`, uses a separate publisher process,
800 actual services and private-port IPv4 loopback multicast. A second raw
receiver confirms that the upstream 1000-event channel closes. No message
injection, native backend, unit test or mock is used.

The same probe passed **5/8** checks on the baseline and **8/8** on the first
fix. The baseline missed later advertisements, TXT/port updates and goodbye
removals. Initial resolution, the full-channel workload, peer cap and publisher
exit controls passed on both. Ordinary advertise/browse commands and five
argument/existing-fixture refusal commands also passed on the first fix.

The initial probe removes the 800 burst advertisements before checking fresh
discovery. It therefore does not cover a retained large namespace. Also, its
`worker-shutdown=true` marker includes an acknowledged raw sentinel worker and
a submitted Discovery shutdown request. It does not acknowledge the Discovery
worker itself. The successful process exits and absence of owned Mac test
executables do not replace the missing worker-lifetime assertion. Both gaps
were raised in independent source review and assigned follow-up work.

The printed zero-millisecond recovery observation is not an end-to-end recovery
latency measurement. The parent starts observing after the publisher's own
two-second announcement wait. The run durations also include failure observation
timeouts, so they are not performance comparisons.

## Broader verification of this exact intermediate artifact

The locked Mac release build, strict workspace/all-targets release Clippy and
format check passed with Rust 1.99.1, two build jobs, reduced priority and minimum
macOS 15.0. A fresh clone matched all **113** tracked Rust/Cargo/toolchain inputs.
Only `crates/net/src/discovery.rs` differs between the baseline-with-probe
commit and the fixed runtime commit. The probe source is identical in both runs.

The new Mac daemon remains **3,707,520 bytes**. SHA-256:
`e95f7c7476358e3801892a6fd4f76adee10fa015edc819f75da0fdc724bd67dd`.
The full virtual suite passed **89/89**, with **30 Docker exclusions** and zero
cleanup failures. The Docker socket was absent; the engine was not started.

S15 passed its existing 60-second CPU/RSS gates for both virtual daemons:
0.050% / 9,338,880 bytes and 0.033% / 9,388,032 bytes. Its 1000 plaintext
virtual-loopback moves had p99 **0.889542 ms**. These observations do not measure
native discovery, physical input latency, sustained memory retention or Synergy.

The [machine record](discovery-recovery-initial.json) retains exact hashes,
checks, source scope, scenario outcomes, measurements and open review findings.
Raw private reports are retained outside the repository. No installed sharing,
native input/clipboard, services/tasks, account/desktop settings, production
configuration, default IPC, Bonjour port 5353 or production port 24800 was used.

## Parallel follow-up and remaining gates

Independent source review identified repeated cached replay, missing upstream
cache expiry, cancelled DNS work and early-return shutdown risks. These are
source findings. Real reproductions and fixes are in progress; this record does
not present the proposed checks as executed or the suspected resource growth as
a measured leak or denial of service. Native Windows verification of this
intermediate source has a separate owner and record.

Native input/clipboard acceptance, positive Windows migration/recovery, the
original independent security clearances and trusted signed/notarized distribution
remain open. GitHub private reporting setup remains deferred.
