Public `daemon.stop` has an avoidable post-dispatch poll opportunity. Source
establishes this control-flow cause without another daemon flow. It does not
establish the exact duration of that poll in the saved r3 run, or prove that it
was the sole cause of the saved one-second timeout. No product P2, production
repair, or fresh runtime reproduction is claimed.

This research used only the T3-owned worktree
`/Users/serge/.t3/worktrees/openergy/t3-d1ff2556`. Its source base is
`1fd5144628c0494f1ebf561581e0d84fd7231523`. The input runtime evidence is
`b75dfa1b543e4bf6b47858581d4d2cf19aa777e6`, with source parent
`71a40b7c2a5cf22cd0bf18a8b602f9ead51de681`. The supplied review is
`ce22c1f37618563dea522c4a04b2248b7fc56e11`; its committed bytes match the
document read in `t3-4021ec78`.

The declared source set contains 387 tracked files under the daemon, net,
session, IPC, config, platform, macOS platform, and vendored mDNS paths, plus
the workspace manifest, lockfile, and toolchain file. All match the current
source base, saved r3 commit, and r3 source parent exactly. This is source
identity evidence, not a new check of complete executable build provenance.
[source-bindings.json](source-bindings.json) records every path, byte length,
SHA-256, and the exact Git comparisons. In particular:

- `crates/daemon/src/runtime.rs`: SHA-256
  `187c2301efa9873791597289a45247abc6221c9cf8538db492d20e4b0d202af9`.
- `crates/daemon/src/ipc_server.rs`: SHA-256
  `4ceb1dc908211fd0b6493c55787f32582b8b248bedff20c99d800fed41a2301e`.
- `crates/net/src/reactor.rs`: SHA-256
  `2fdfd8783e0d249c2441b1994ad7b67d8926662f6fee8244bbfd9558451e851d`.

The causal source chain is more specific than the timeout clamp alone:

1. `ipc_server.rs:318-329` puts the real request on the runtime IPC queue and
   wakes the network poller. A normal ordering is: the current poll consumes
   that wake, returns, and the next runtime iteration reads the queued request.
   The wake precedes dispatch; it is not a new wake after the stop flag is set.
2. `runtime.rs:1508` checks stop only at the outer loop guard. Inside that loop,
   `runtime.rs:1239-1242` dispatches `daemon.stop`, stores `true` in the shared
   stop flag, and returns `{"stopping":true}`. This is the runtime thread's own
   store, not evidence of a stale cross-thread atomic read.
3. `runtime.rs:1530-1541` builds the public response, applies actions, processes
   network events, and sends the response. It neither breaks the outer loop
   nor checks stop after sending. `ipc_server.rs:330-374` receives and writes
   that response; it has no new runtime wake after a successful reply.
4. The current runtime iteration continues through clipboard work, session
   timers, health checks, status publication, and deadline calculation. There
   is no intervening stop check. `runtime.rs:1634-1639` then calls
   `driver.poll(Some(timeout), ...)` and processes its results before reaching
   the outer loop guard again.
5. `reactor.rs:1761-1777` can make a real blocking poll. Its maintenance and
   wait calculation do not receive the runtime stop flag. It can shorten the
   wait for other network work, but cannot skip this wait because public stop
   was dispatched. Only after it returns does the outer guard observe stop
   and enter the shutdown sequence at `runtime.rs:1641`.

Thus an already-dispatched stop can wait for an unrelated normal-operation
deadline when its request wake was already consumed and no other event is
ready. A still-pending wake, network event, new event from a worker,
maintenance event, or poll error can shorten or remove that wait. Source does
not say every stop sleeps, nor that every poll uses its full requested timeout.

The runtime caps its requested poll timeout at 2,000 ms. That is not a process
exit bound: prior work, OS scheduling, maintenance, blocking sends, joins, and
destructors are separate. For the allowed hypothetical role-Off virtual case,
`Machine::Off` selects `now + 1000` at `runtime.rs:1622-1625`, so the nominal
requested wait is at most 1,000 ms before other deadlines shorten it. Do not
use a role-Off result to claim a measured 2,000 ms connected-server wait.

The shutdown path at `runtime.rs:1641-1663` is distinct: leave/goodbye, a flush
loop with a 250 ms elapsed-time threshold, release input state, network close,
worker Stop commands, and joins. `ipc_server.rs:54-58` closes its private
shutdown peer before joining the accept thread; that join does not require a
new public IPC connection. No grace period, worker join, cancellation path,
IPC policy, discovery code, or Synergy import budget was changed.

The exact saved public command was:

```sh
/private/tmp/oe-idle-r3-f553ae64-nnze4977/openergy-daemon ctl daemon.stop --socket /private/tmp/oe-idle-r3-f553ae64-nnze4977/run/peer/ipc.sock --raw
```

It ran from `/private/tmp/oe-idle-r3-f553ae64-nnze4977/run`, exited 0, wrote
`{"stopping":true}\n`, and had empty stderr. Its recorded elapsed time was
`0.01127445799999549` seconds. The controller then ran
`peer.wait(timeout=1)` and timed out. These are audited saved receipts, not
commands executed by this research. The response is acknowledgement of stop
dispatch, not a one-second exit guarantee. The archived command helper records
elapsed duration only; it omits absolute command-start and command-end times.
It also does not retain the wire request ID or framed response envelope.

The saved peer identity was PID/group 51089, parent 51066, UID 501, effective
nice 15 by both `ps` and `getpriority`, executable under the private root above,
and `lstart` `Fri Oct 23 18:04:15 2026`. The copied exec-barrier records also
retain the app and supervised daemon identities. The saved daemon hash is
`350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
This research neither reopened nor ran that cached binary.

The saved wall-clock sequence adds useful evidence beyond the one-second
failure:

- `1792803892455`: peer log `shutdown: leave and goodbye`, at shutdown entry.
- `1792803892525.532`: controller receipt for SIGTERM to owned peer group 51089.
- `1792803892711`: peer log `daemon stopped`, after all three explicit joins.
- Later cleanup records peer exit 0 and PID/group absence, without an exact
  process-exit timestamp. The supervised app daemon's exit code is uncollected.

The log places shutdown entry 70.532 ms before the cleanup SIGTERM receipt,
and places the post-join log 256 ms after shutdown entry. Thus the saved wall
records do not support "SIGTERM was needed to begin shutdown" or a long delay
between shutdown entry and the post-join log. The latter interval includes
the flush and joins; it is not an exact measurement of either alone. SIGTERM
was sent during that interval and could affect its poll calls, so this is not
an unsignalled normal-exit measurement.

The first dispatch store, first outer-guard stop observation, poll entry/return,
exact flush start/end, exact reply receipt, and exact process exit were not
instrumented. The daemon logger uses integer `SystemTime` Unix milliseconds
(`logger.rs:12-17,120-128`); the signal receipt uses controller wall time. No
clock-adjustment or logging-overhead bound was established. The 256 ms and
70.532 ms values are differences of saved wall-clock fields, not a new
monotonic trace. They cannot identify the exact old pre-shutdown poll duration.
[analysis.json](analysis.json) keeps all missing timestamps explicitly null.

The remaining explanations have different support. The avoidable poll is
established by source and is compatible with the failure. A long shutdown
flush/join interval is not supported by the saved 256 ms interval. The saved
ordering already places shutdown entry before cleanup's signal receipt. OS
scheduling and other pre-shutdown work remain unmeasured. These conclusions
do not convert the saved timeout into a P2 or make the old flow pass.

The first exclusive bind of port 65202 remains FAILED with Errno 48. The saved
`lsof` ran before that bind and returned exit 1 with empty output. The saved
filtered `netstat` ran after the failed bind and contains two TIME_WAIT rows.
Those are not listener rows and do not prove why bind failed. There was no
pre-bind `netstat` state snapshot. Port 65203 passed its single first bind.
No new socket state capture, bind, or retry ran here. The old strict cleanup
result remains FAILED, with its raw traceback retained byte for byte.

The execution bounds for this research were zero new daemon flows, zero new
stop requests, zero compiler/product launches, zero builds, zero sockets or
port probes, zero signals, and zero trace overlays. No temporary runtime,
config, TLS identity, or IPC endpoint was created, so runtime exit deadlines,
product NI 15 checks, and fixture cleanup are not applicable. The evidence
collector sets only its own absolute nice value to 15 and records its own
start identity. [scope.json](scope.json) records these limits. There are no
compiler/product PIDs to clean up and no daemon cleanup claim for this turn.
The source collector finished in less than one second. Collection plus final
archive/invariance validation are recorded in their separate receipts; no
60-second product cleanup interval was started.

The diagnosing-bugs and macOS build-run-debug skills were read. The explicit
source-first brief permits a static result and asks to avoid unnecessary
runtime. For that reason this research stops before the skill's runtime
feedback/fix loop. No unit test, mock, or callback replacement was written.
No build bootstrap files were needed. No desktop, installed app, live sharing,
native input, production config/TLS/default IPC, TCC, service, login, task,
account, firewall, global DNS, toolchain default, Docker, signing key, push,
publication, external network, TCP 24800, or UDP 5353 action ran. The three
provider-stopped tasks were not resumed. All writes are in this unique folder.

For a later authorized implementation, the narrow source direction is to
observe stop after response send and before the next normal-operation poll,
while preserving the reply and the complete existing shutdown path. This is
an implementation direction only. No production patch or acceptance result
is supplied. If exact dynamic attribution becomes necessary, one private
trace would need monotonic events at dispatch, reply send, post-dispatch poll
entry/return, guard observation, flush boundaries, joins, runtime return, and
an external process-exit observation. That trace was not built or run here.

The new evidence consists of byte-exact copies of 15 selected saved blobs,
their origin hashes, source excerpts with exact line numbers, source hashes,
an evidence reader, and receipts. `raw/saved-r3/measure.py` is archived text;
the reader never imports or executes it. Existing evidence and failures are
unchanged. The old README and full supplied review remain in the raw archive;
their claims retain their original scope.

The evidence collector ran once:

```sh
python3 docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/collect.py > docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/collection.stdout 2> docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/collection.stderr
```

It exited 0 with empty stderr. [collection.json](collection.json) retains exact
read-only command vectors, exits, and outputs or the byte-exact output paths.
[reader-errors.txt](reader-errors.txt) preserves corrected exploratory read
errors. None was a product execution or fixture retry. The collector refuses
to reuse its raw output directory; no rerun was needed.

After staging only this folder, the archive/invariance command is:

```sh
python3 docs/verification/daemon-stop-latency-source-2026-10-23-t3-d1ff2556/validate.py
```

It checks the copied-origin hashes, the original r3 manifest entries, all 387
declared source hashes, the diff outside this folder, staged whitespace, and
exclusive ownership of staged and untracked paths. It writes
`validation.json` and the new `evidence-manifest.json`, then verifies every
listed byte length and SHA-256. The manifest excludes only itself. These are
archive checks, not product tests. No TypeScript package or compiler is in scope.

The first archive check failed on a trailing space in the derived architecture
excerpt's numbered empty line `588:`. The exact failed command and traceback
remain in `archive-validation-first-failure.txt`. One presentation-only edit
removed that space; `excerpt-presentation-correction.json` records the exact
before/after hashes and replacement. The collector and its original command
outputs were not changed or rerun. No source or copied raw blob was altered.
The same archive check then ran once more after staging that correction.

A later precommit completeness check found 39 files on disk but only 37 in
the index: repository ignore rules omitted the two copied peer `.log` files.
That check failed; the shell still ran the following commit command because
the heredoc and commit were separated by a newline without failure gating.
The initial evidence commit was `f1d0aac` and was not pushed. The failure is
retained in `commit-completeness-first-failure.txt`. Both byte-exact log files
were then added explicitly with `git add -f`, followed by complete index-byte
and manifest checks before amending only this owned evidence commit. This
correction did not execute the collector, a daemon, or a port probe again.
