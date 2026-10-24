# Independent saved-record review of caller cancellation

Review ID: `oe-macos-caller-cancellation-review-20261023-24ad7903`.

**Result: no concrete P1/P2 finding in the reviewed change and saved flow.**
This is a source and saved-record audit. It adds no runtime execution or
acceptance to the inherited result.

## Immutable scope

- Root source parent: `8c9f0abbdf4016e22738b5b5a51e7c35185e3596`.
- Candidate parent: `a121376fe6c1715fa99d72559336ca3b3a73b439`.
- Source: `511b0959e62aca46dc9d436470c1056d66f4fb29`.
- Evidence: `caafdadb4d5d0851c23ae299296ca8000b495e85`.
- Saved evidence ID: `oe-macos-caller-cancellation-a121376-20261023-pbgh33w0`.
- Saved evidence directory:
  `docs/verification/macos-caller-cancellation-2026-10-23/` at the evidence commit.

The reviewer read Git blobs and parsed their text/JSON locally. No build,
probe, daemon, network request, desktop action, unit test, mock, installation,
publication, or further delegation ran. No private fixture, cache, signing
key, token, production configuration, or live process state was inspected.
The provider-stopped work was outside this review.

## Source and public-flow checks

`511b095` adds 140 lines to `AppEndToEnd.swift` only. Removing its three added
ranges reproduces the complete parent file byte for byte. The old default
routine and production classes are unchanged. This does not rerun the prior
25/25 routine, stalled-write/crash-recovery flow, or event measurement.

`OpenergyApp.swift:27-35` enters this code through explicit `--e2e`.
`AppEndToEnd.swift:29-75` requires `--caller-cancellation`, no rendering and
discovery disabled. It retains the private-path, virtual-platform and fresh
Off-config checks before `AppModel.start()`. The new branch returns before
the default routine can Join or Host. `adapter.rs:16-22` disables mDNS for the
virtual platform and for the discovery-off override. These are source checks,
not new observations of network activity.

`AppEndToEnd.swift:271-310` gets the model's supervised child PID and matches
it to daemon status. Before each cancelled call, `sysctl` confirms parent identity
and stopped state. The signal targets that captured child PID.
At `:314-330`, the MainActor task sets `entered` and enters the real
`DaemonClient.call`; it remains unfinished after the 125 ms hold, while the
child remains stopped. `DaemonClient.swift:55-85` installs the real pending
continuation and enqueues the request before that call suspends. This is a
pending public caller precondition, not a measurement of private waiter or
writer-queue depth or proof that the daemon read the request before resume.

At `AppEndToEnd.swift:331-344`, each cancellation must return one
`CancellationError` within 500 ms, with the child still stopped and the client
still connected, before SIGCONT. The saved observations are 106.906250,
106.115708 and 106.092500 ms. The shared 100 ms polling helper limits their
precision; they are completion observations, not exact cancellation latency.
The saved transcript and report agree for all three cycles.

One local `let client = DaemonClient()` at `:277` has one `connect` at `:298`.
There is no reset, recreation or reconnect in the flow. Its `disconnect` is
in the final defer, after the observations. The same captured daemon PID is
checked during the cycles and final status. AppModel's own client is a
separate connection; this flow cancels callers on the added real client.
It does not cancel an AppModel action task.

The outgoing trace has exactly 24 requests: AppModel startup IDs 1-5, then
observer IDs 1-19 without gaps. Cancelled `config.patch` IDs are 2, 7 and 12;
healthy patch IDs are 4, 9 and 14. Each cancelled marker is subsequently read
from the real daemon and seen by the model and observer. Each healthy patch,
config read and status read must match its cycle marker. Status must retain
the original PID, virtual platform and Off role.

The observer compares the complete collected name-marker array to exactly
six ordered `config.changed` markers, including the final check. The marker
sequence is `caller-cancelled-1`, `caller-healthy-1`, through cycle 3 in that
order. Collection filters to `config.changed` events with a string config
name. This is exact cardinality for that stream, not every event type or
malformed callback. No inbound frame transcript was saved; response identity
rests on the real parser's request-ID routing and the checked result fields.

After cycle 3, a 2200 ms sleep precedes fresh config/status/discovery calls
(observer IDs 17-19). With the saved 3.761074708-second flow duration, all three
configured call-start-plus-two-second deadlines precede the final observation.
The first cannot precede flow time 2 seconds. Even the last is no later than
3.561074708 seconds, using the final sleep alone as the ordering bound.
Exact per-request timer start timestamps were not saved. All three outcomes
remain `CancellationError`, completion counters remain one, the six-marker
array is unchanged, disconnect count is zero, and discovery is empty.
There is no observation after the final check and disconnect. The completion
counters describe caller-task completion, not private continuation counts.

## Record and artifact bindings

All 17 committed raw artifact byte hashes match `results.json`. Sixteen are
labelled untransformed. Cleanup is the one declared derivative. All 19 report
entries match the saved stdout PASS lines exactly; the transcript has no FAIL
line and runtime stderr is empty.

All 421 recorded app/crate/Cargo input hashes in `raw/source.json` match the
source commit. Against the referenced cached-build manifest, the harness is
the only changed recorded input; the other 420 match. The recorded source
parent is `a121376`, consistent with recording the modified harness before
its source commit. The harness hash binds those bytes to `511b095`.

- Harness SHA-256: `cf3bc66eec68fbf0f508e487773f280759529f0a49782525c92522a10ed4f357`.
- Archived `run.py` SHA-256: `1014b2865b64475a1d198b8e9722b956a3c7fdb09662c4ad108710fda8be4112`.
- Recorded app SHA-256: `224f8e9f38d3d99ac5300b6930843323d123f93d22b51df8625c5bfe80b0f0a0`.
- Recorded daemon SHA-256: `350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.

The app/daemon digests agree between `results.json` and `raw/ready.json`.
The daemon digest also agrees with `raw/source.json`'s cached daemon record.
The archived controller digest matches its declared digest. The controller
source copies the app source, builds the whole executable in private scratch,
copies the cached bundle, replaces the executable and applies an ad-hoc outer
signature. These agree with the saved commands. This audit did not hash live
binaries or the private controller copy. The archived records do not establish
independent release provenance or authenticate the actual executed controller.

## First attempts, deviations and cleanup limits

The saved first build exits 0 in 30.569371875 seconds. Its only stderr text is
the `--skip-update` deprecation warning. Private ad-hoc signing exits 0 and
reports replacement of the private bundle signature. The saved app run exits
0, passes 19/19 checks and takes 4.485389249999997 seconds including shutdown.
Both first-failure fields are null, and the manifest reports one build and
one runtime attempt. The controller has no retry loop. There is no actual
build/runtime failure in this archive to quote. This audit cannot exclude
unarchived attempts and does not import failures from other flows.

The saved commands use two build jobs, macOS deployment target 15.0, disabled
automatic Swift package resolution, skipped package update and private caches.
The compiler version record reports Rust 1.99.1, but no Rust rebuild ran.
The real daemon is the inherited cached binary. Cargo offline mode is recorded;
it is not proof of a fresh locked/offline Rust build. The exact 768-file cache
hash list remains private; the archive records its digest and successful
before/after equality, which this audit cannot independently repeat.

The controller invocation and its children both use `nice -n 15`. These are
nested increments. Effective niceness was not sampled; exact nice 15 is not
verified. The records disclose this deviation and retain the run without a
retry. The 15-second outer bound is the app subprocess wait. Build and
postflight cleanup have separate time limits; 15 seconds does not bound the
entire controller including build and cleanup.

The saved cleanup record reports no process-group members before or after
cleanup, no cleanup signal, and no IPC socket after exit. It reports successful
fresh exclusive loopback binds on ports 59698/59699. The controller selects
process inventory rows by the app's process-group ID; it does not archive a
separate final daemon-PID lookup or the ps exit status. Thus this is the saved
group/socket/bind evidence, not a new independent per-PID cleanup check.
App source calls `model.shutdown()` before report/exit; the supervisor stops
its child. `lsof` returns 1 with empty stdout, but its unrelated OS mount
warning was redacted in the committed derivative. The declared original
cleanup digest is `73ea732d9bc69c73185f2e9406cffd0e5dd0fd034e86d42417df60f0b0e34b12`;
the derivative hashes correctly to `66fcd8b266598c86625529dd4614e69c3800c8800ef68d4219d68b17a7849858`.
The private original was not read. The separate bind observations support the
assigned-port result without treating the lsof warning as a clean inventory.

The bounded result supports the saved pending-caller cancellation/reuse case.
It does not establish private waiter/timeout/queue counts, a memory bound,
burst-load behavior, absence of all leaks, rendered UI, native hardware,
Windows, live sharing, distribution/signing acceptance, or provider-stopped
security/provenance acceptance. No production defect or required source
change was identified within this review scope.
