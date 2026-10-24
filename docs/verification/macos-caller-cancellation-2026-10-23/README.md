# Real app caller cancellation

Evidence ID: `oe-macos-caller-cancellation-a121376-20261023-pbgh33w0`.
Source parent: `a121376`; harness source: `511b0959e62aca46dc9d436470c1056d66f4fb29`.
The first whole-app build passed in **30.569 seconds**. The only app run passed
**19/19 checks**, exit **0**, in **4.485 seconds**, including model shutdown.
The caller flow itself took **3.761 seconds**. Its outer controller had a
**15-second wall limit**. There was no build or app retry and no production
problem was found.

`AppEndToEnd.swift` adds the explicit `--caller-cancellation` branch after the
existing safe-start checks. It requires no rendering and discovery disabled.
The real `AppModel` starts and owns the real virtual daemon. A separate real
`DaemonClient` connects once and uses the ordinary request/event interface.
The original default 25-step routine remains present and unchanged. It did not
run here; its prior 25/25 result remains inherited evidence.

Each of three cycles sends SIGSTOP only to the model's own daemon child.
`sysctl` confirms both its parent identity and stopped state before the caller
starts a small, ordinary `config.patch` request. The caller enters the request
and remains unfinished for 125 ms while the child stays stopped. Cancelling
that caller Task returns `CancellationError` once, before SIGCONT. The
100 ms polling loop observed completion at **106.91, 106.12 and 106.09 ms**,
within the 500 ms bound. These are polling observations, not exact completion
latencies. The request's ordinary timeout was two seconds.

After each resume, the cancelled mutation actually reaches the daemon. Its
exact name marker appears in a later `config.get`, the live AppModel and a real
`config.changed` callback. This also shows that cancelling a caller does not
undo a mutation already sent. The same client then makes a healthy mutation
and receives exact config/status replies for the new marker, original PID and
virtual Off role. The observer sees exactly six ordered config markers, with
no duplicate. There is no reconnect, disconnect callback or daemon replacement.

After a 2.2-second settling wait, beyond all cancelled requests' original
deadlines, new config/status/discovery calls still return the exact final name,
original daemon PID and empty discovery list. Each cancelled Task still has
one completion. The trace contains **24 real IPC requests**: five AppModel
startup requests and nineteen requests on the single cancellation observer.
No frames, callbacks, server responses or private counters were injected.

This finite public flow closes the pending-caller cancellation/reuse scenario.
It found no stale public event, second caller completion or later timeout that
closed the session. Exact private waiter counts, timeout-task counts and GCD
callback queue depth remain **unverified**. Three cycles do not establish a
memory bound, sustained-load result or absence of all leaks.

## Isolation and build limits

All runtime paths were fresh and private under `/private/tmp/oe-caller-pbgh33w0`.
The original harness wrote its private Off config before AppModel startup.
The virtual platform, discovery-off flag and private config/TLS/IPC overrides
were set before launch. The controller exclusively reserved literal loopback
ports **59698/59699**, then released them. This Off-role branch never calls
Join or starts a TCP server; the required `--join` value is unused.
No 24800/5353 endpoint, native platform, rendered UI, installed app, live sharing,
desktop automation, TCC, login, service, account, firewall or global setting was
used or changed. No unit test or mock was added. No import or protocol-budget
source changed and no provider-stopped experiment was resumed.

The whole app compiled from a private source copy with two jobs, deployment
target 15.0 and automatic dependency resolution disabled. Sparkle 2.10.0's exact
cached checkout, repository and framework artifacts were copied from the
authorized cache into private scratch. The copied bundle received only the new
app executable and a private ad-hoc outer signature. All **768** original
dependency cache file hashes still match. Of the 421 recorded app/crate/Cargo
inputs, only the owned harness differs from the cached build manifest.
The real universal daemon is the unchanged cached binary, SHA-256
`350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
No Rust rebuild occurred. The pinned local compiler reported Rust 1.99.1 and
the controller set offline Cargo mode. Source matching and the cached daemon's
prior build are separate from this fresh caller runtime evidence.

**Scheduling limit:** the actual controller invocation used `nice -n 15`, and
its child commands also used `nice -n 15`. These are nested increments. Effective
niceness was not sampled, so this run does **not** verify exact nice 15. There is
no scheduling or speed comparison claim. The first run is retained without a
retry to change this result.

Postflight needed no cleanup signal: the app and child were already gone and
the IPC socket was absent. Both assigned ports allowed fresh exclusive loopback
binds. `lsof` found no assigned-port entry but emitted an unrelated OS mount
inventory warning. Its private original is retained; the committed cleanup
derivative omits only that warning's private mount details. The separate bind
checks remain in the record.

## Exact records

The executed controller is [run.py](run.py). Its command was:

```sh
nice -n 15 python3 docs/verification/macos-caller-cancellation-2026-10-23/run.py
```

[results.json](results.json) binds the exact source, app/daemon hashes, commands,
timings, limits and 17 archived records. Sixteen records are byte-exact copies
of the private originals. The cleanup derivative has an explicit transform and
original SHA-256. The original dependency hash list and exact controller copy
remain in the private fixture. Build stderr's deprecation warning, empty runtime
stderr and the complete first-run request/check transcript are preserved.

The [prior source review](../macos-app-current-flow-2026-10-23/background-review.md)
at `043757a`, [current 25/25 flow](../macos-app-current-flow-2026-10-23/README.md)
and [event measurement](../macos-event-delivery-2026-10-23/README.md) remain
separate evidence. This run adds no rendered UI, Windows, native input,
clipboard, signing/distribution, security, provenance or sustained-load proof.
