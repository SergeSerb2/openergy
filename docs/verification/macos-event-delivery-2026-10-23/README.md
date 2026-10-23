# Bounded macOS event delivery measurement

No actionable runtime issue reproduced for the two observations from
`oe-macos-background-ipc-lifetime-review-r1-043757a-20261023`.
This is a finite result, not proof that either queue is bounded.

On 2026-10-23, base `d83807d542a86a99ad83978cc460028d972373e1`, the real
headless app passed **12/12 measurement checks**. The successful launch took
**9.623 seconds**, including owned-process cleanup. Its live model measurement
lasted 7.551 seconds. The controller enforced a 60-second wall limit; the
measurement branch had its own 45-second deadline.

The 12 paced, ordinary `connection.restart` calls produced **12 genuine
`peer.untrusted` events**, **12 distinct warning log lines visible in
`AppModel.logLines`**, and **12 new pending-trust observations**. There was one
additional real initial-Join trust event. The app's existing IPC trace shows
exactly **12 model `status.get` requests** during the 12-restart phase. No other
source of `status.get` ran in that phase. This witnesses refresh issuance, not
the exact number of unfinished refresh tasks.

The daemon timestamp to model-log polling delay was **15.33–68.72 ms**. The
pending-trust polling delay was **16.35–68.74 ms**. These include the harness's
25 ms polling interval and daemon IPC delivery. The separate real
`DaemonClient` event observer saw daemon-event timestamp to callback delays of
**0.46–51.22 ms**. Each restart reply took **0.29–1.42 ms**; request to genuine
trust-event callback took **101.94–152.82 ms**. Event emission times use the
daemon's millisecond wall clock; heartbeat and request timing use
`ContinuousClock`.

During delivery, the 25 ms main-actor heartbeat had intervals of
**25.26–31.17 ms**. Across all phases its maximum delay beyond 25 ms was
**7.59 ms**. This measures the real model's main actor, not rendered UI.

The OS supplied 27 app samples during delivery. App RSS was
**12,880–13,136 KiB**. Displayed `ps %CPU` was **0.4–1.0%**; displayed cumulative
CPU time moved from **0.03 to 0.07 seconds** at 0.01-second display resolution.
The final settled sample was **13,552 KiB**, **0.09 CPU seconds**. Measurement
code and heartbeat logging are included. RSS did not return to its first value;
this short sample does not establish a leak or its absence.

A separate `swift-build` appeared in three startup samples, through controller
time 0.310 seconds. It was absent from all event, recovery and settled samples.
The private app build had already ended. There is no comparable-idle or
performance-improvement claim.

Actual approval restored the TLS peer in **209.71 ms** at the harness's 100 ms
condition-check granularity. Disconnect cleared the peer. A trusted rejoin
connected again. Returning to Off cleared peers and pending trust, and a real
`refreshAll()` completed. After a two-second settling period, app numeric file
descriptors were **6 → 6 → 6** at setup, delivery and settled snapshots. The
app daemon was **13 → 14 → 13** and the peer **12 → 13 → 12**. All three owned
PIDs then disappeared, with no remaining descriptors. Both IPC paths were
removed. Assigned TCP ports **55819/55820** had no `lsof` entries and allowed
fresh exclusive loopback binds.

The complete work used **73 genuine IPC requests**, including the failed-loader
attempt's peer readiness query. The largest traced request was **157 bytes**.
A conservative allowance that counts both connection ends keeps the combined
request/connection count at **at most 117**, below 128. The derivation is in
`results.json`. Fourteen actual TLS peer establishments also appear in the
peer log. No frames or callbacks were injected.

## Exact source and isolation

First inspection found `AppEndToEnd` already uses `AppModel.subscribeLogs` and
the daemon's `Logger` publishes real `log.line` events. `Runtime::process_net`
publishes `peer.untrusted` for an unknown TLS identity. Thus both observations
had genuine runtime witnesses in this run; neither was replaced with a mock.

[harness-overlay.patch](harness-overlay.patch) is the exact temporary overlay.
It adds one opt-in branch after the existing safe-start check and one measurement
method. The **whole original app harness remains present** in the private source
copy. Its remaining 25-step flow was not executed. In particular, no stalled
daemon, oversized request, crash test or config import ran here. This does not
repeat the parent's 25/25 result.

Every other copied app source file, including `AppModel`, `DaemonClient`,
`IPCSocket` and `DaemonSupervisor`, matches production byte for byte. All 456
inputs in the recorded Sparkle integration manifest match this checkout. The
copied real daemon SHA-256 is
`350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
No production behavior changed.

The controller copied the pinned Sparkle **2.10.0** checkout, repository and
artifacts to a private scratch path. It copied the existing daemon and bundle,
replaced only the private app executable, and applied an ad-hoc signature to
that private bundle. The original cached files remained unchanged. The build
used `nice 15`, two jobs, deployment target **15.0**, and disabled automatic
dependency resolution. It performed no dependency download or Rust rebuild.

Both daemons used fresh private config/TLS/IPC paths, the virtual platform,
discovery disabled, and literal `127.0.0.1` endpoints. The peer's private TLS
config accepted client certificates so one app approval could complete the real
connection. This was only an event witness, not security acceptance. The app
started from the original harness's fresh Off config before enabling private TLS.
No 24800 or 5353 endpoint, installed app, native input, TCC, login item, service,
global setting, desktop automation, unit test or mock was used.

## Commands and artifacts

[measure.py](measure.py) prepares and builds the private whole app, drives the
ordinary caller interfaces, samples the OS, and checks cleanup. The commands
actually used were:

```sh
nice -n 15 python3 docs/verification/macos-event-delivery-2026-10-23/measure.py
nice -n 15 python3 docs/verification/macos-event-delivery-2026-10-23/measure.py --finalize /private/tmp/oe-delivery-qxsu9z10
nice -n 15 python3 docs/verification/macos-event-delivery-2026-10-23/measure.py --measure /private/tmp/oe-delivery-qxsu9z10
```

The first build exited **0 in 37.027 seconds**. The controller then failed to
find the executable because Xcode SwiftPM uses `out/Products/Release`. Its
metadata finalization was corrected without rebuilding. The first measurement
launch exited **-6 before model startup**, because the standalone binary could
not load Sparkle despite `DYLD_FRAMEWORK_PATH`. Its peer stopped and its groups,
sockets and assigned listeners were removed in **1.458 seconds**. That failed
launch is archived under `failed-launch`. The corrected finalization copied and
signed the private bundle. The same `--finalize` and `--measure` commands above
then produced the successful result. The final controller contains both fixes.

The full Swift build argv/environment, process launch argv/environment and
OS inspection argv are stored in [raw](raw). [results.json](results.json)
contains derived numbers, exact source pointers, artifact paths and SHA-256
values. Each listed raw record is a byte-exact copy with `transform: none`.
The full private fixture is `/private/tmp/oe-delivery-qxsu9z10`. The failed
launch's exact cleanup record remains there; its unrelated private OS mount
warning is not copied into Git. No TLS private key is published.

## Unverified scope

Exact internal GCD callback queue depth, unfinished refresh-task count and each
model refresh completion latency remain **UNVERIFIED**. The unchanged production
classes do not expose them. This one-client, 12-event paced sample does not test
high-rate or multi-peer buildup, long-term growth, native UI rendering, daemon
or config-import budgets, provider-stopped IPC, Synergy 3, security or mDNS
acceptance. No unboundedness, memory cap, speedup or production fix is inferred.
