No concrete P1 or P2 finding in the reviewed change. The source removes the
normal-operation poll opportunity after the runtime sends a stop response to
the IPC reply channel. It preserves the response construction and the complete
existing shutdown tail. This is an independent source and saved-evidence
review. It is not an independent runtime reproduction or general performance
qualification.

Reviewed candidate: `2485c1425e4d2f414eb22742dfd73f355d47390c`.
Its exact parent is `d2ab17d4e053437d9f7568b0f6dbf96727bfd6c9`.
The complete candidate document read was
`docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/README.md`.
Below, archive paths are relative to that directory and source line numbers
refer to this exact candidate. Source and committed evidence were read through
Git, without checking out or changing the candidate.

This review owns only this document in T3 worktree
`/Users/serge/.t3/worktrees/openergy/t3-d6fd8854`, branch
`t3/review-daemon-stop-latency`, starting at
`ad280644a96905d9831825bb4f87d1388dbdca73`.

The source checks support the narrow result:

- The entire production delta is six added lines and one removed line in
  `crates/daemon/src/runtime.rs`. The patch artifact `runtime-fix.patch` matches
  the exact parent-to-candidate Git diff. The change labels the outer loop at
  line 1508 and checks the shared stop flag at lines 1543-1545. It does not
  change dispatch, IPC budgets, queues, cancellation, discovery, import,
  native backends, timeouts, or rebind behavior.
- `daemon.stop` still stores `true` and returns `{"stopping":true}` at
  `runtime.rs:1239-1242`. Lines 1534-1540 still construct `Response::ok(req.id,
  value)` or the existing error response, apply actions, process network
  events, and call `reply.send(response)`. The new break is after that send.
  It leaves the outer loop, rather than only the eight-request drain. Failed
  reply delivery does not prevent shutdown once the flag is set.
- The send is the internal channel handoff. The IPC client still receives
  and writes the framed response at `ipc_server.rs:330-363`, including when
  stop is already true and `recv_timeout` returns `Ok(response)`. The new
  check does not close that client stream or drop its thread. The saved stop
  envelopes and CLI output confirm delivery in the two recorded flows. This
  does not create a new unconditional wire-delivery guarantee under arbitrary
  scheduling, client failure, or the existing pre-send work.
- Requests behind stop no longer receive the remaining normal-operation
  dispatch turns. There is no complete queue-drain promise. The runtime
  receiver can still hold queued reply senders while `accept.join()` runs.
  This does not make client completion depend on dropping that receiver:
  the existing receive loop checks stop after each 50 ms timeout and also
  retains its ten-second elapsed-time check (`ipc_server.rs:331-343`). A
  buffered pipelined successor can still enqueue before the client's next
  outer guard; it uses this same stop-aware wait. A receive timeout returns
  from the client when stop is observed. These checks do not establish a
  fixed completion bound under arbitrary scheduling or socket writes.
- On Unix, `ServerThread::join` drops its private shutdown peer before joining
  (`ipc_server.rs:54-58`). This wakes the accept poll without a new public
  connection. The accept thread closes the listener and joins its clients
  at lines 224-227. Fatal listener/accept errors already set this same stop
  flag and wake the reactor at lines 188-208. If such a store is observed
  after any IPC response, the new check also takes the shutdown path. It does
  not claim that only `daemon.stop` can trigger the break, or that every
  concurrent fatal store must be observed at that precise point.
- The public-stop store and new load are on the same runtime thread and use
  the same atomic allocation. The runtime retains a clone of the shared
  `Arc` at line 1485. No intervening code clears the flag. Relaxed ordering
  suffices for this thread to observe its own stop store; this check does
  not publish other state or require a stronger ordering. Existing cross-thread
  stop stores and wake behavior remain in place.
- The signal and parent-watch predicates stay in the original outer guard.
  The new load does not read `SIGNALLED` or call `parent.exited()`.
  `lifetime.rs`, its handlers, parent-watch implementation, and Windows
  completion store are unchanged. This review supplies source identity for
  those paths, without new signal, parent-exit, or Windows runtime evidence.

The shutdown suffix from `log::info!("shutdown: leave and goodbye")` through
the end of `run_daemon` is byte-identical to the parent: 888 bytes. Candidate
`runtime.rs:1646-1668` still disables the runtime, calls machine shutdown,
applies its actions, runs the full flush loop while elapsed time is below
250 ms, releases the sleep assertion and all input, restores the cursor,
executes network shutdown, stores stop, sends both worker Stop commands,
and performs the accept, capture, and clipboard joins. Server leave/Bye
generation (`crates/session/src/server.rs:368-374`) and client leave/Bye
generation (`crates/session/src/client.rs:182-188`) are unchanged. Worker
Stop handling is unchanged. The 250 ms threshold is not a hard process-exit
bound; poll work, scheduling, sends, joins, and destruction remain separate.
The skipped ordinary clipboard, timer, health, and status work is not a
replacement for, or removal of, this shutdown sequence.

I independently checked committed archive bytes and their bindings with
read-only Git and in-memory hash/JSON comparisons. All 193 entries in
`evidence-manifest.json` match their byte lengths and SHA-256 values. The
candidate tree contains exactly those 193 files plus the manifest in the
owned evidence directory. All 397 root inventory records match the stated
parent blobs. All three complete per-build inventories match their expected
source bytes. Reconstructing `cli-trace-overlay.patch` in memory matches the
instrumented `main.rs` hash in both measured builds. Between those two
inventories, only `runtime.rs` differs. The final-source inventory matches
the committed candidate, with `main.rs` restored byte-for-byte.

The final runtime SHA-256 is
`afaee53963705065e5bd6ca10927b905ac4a36380cf647f5544587e79be4b2d9`.
The measured baseline binary's recorded SHA-256 is
`1e4ea648bcada9fafc2a4ac0a59d723d04dbb6c9ee4cb71898d9b7b04a1e2f02`.
The measured candidate binary's recorded SHA-256 is
`4ac294f7be9a1512c10537944f2ce1ffc257a62c9c34fb5c69c9cdf75b74a2ab`.
The final-source build's separate recorded SHA-256 is
`23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb`.
Prelaunch binary hashes agree with the corresponding build receipts. The
collector/helper hashes and overlay hash agree across both prelaunch records
and the committed artifacts. The tested candidate is final runtime source
plus the declared CLI-only overlay. The uninstrumented final binary was
built but not launched; it is not a second candidate runtime observation.

I read `collect.py`, `exec_owned.py`, `rustc_owned.py`, `validate.py`, the CLI
overlay, raw flow receipts, source inventories, build receipts and samples,
clock preflight, warning-redaction records, cleanup records, and the saved
validation result as evidence. I did not execute those scripts. The collector
uses exclusive output directories, one stop request per flow, and an
independent blocking daemon wait thread. Its parent process remains alive
during the observed stop. TLS is generated before daemon launch. The saved
configuration, command arguments, and environment select private paths,
virtual role Off, literal loopback addresses, disabled discovery, and disabled
autoconnect. Source also prevents role Off from starting a connection and
disables mDNS for the virtual backend.

Both flows retain version, fingerprint, status, config, empty discovery,
invalid-method, and stop command receipts. I checked exit 1 with
`unknown_method` for the intended failure case, and exit 0 with exact output
`{"stopping":true}\n` and matching envelope ID for stop. The post-exec daemon
snapshots agree with the same-PID exec receipts on PID, parent, group, UID,
NI 15, start time, and executable path. Baseline daemon PID/group is 52629;
candidate PID/group is 60363. Their starts are respectively
`Fri Oct 23 18:47:39 2026` and `Fri Oct 23 18:48:50 2026`.

The saved timing arithmetic is consistent. Baseline request-to-reply is
0.199 ms and request-to-observed-exit is 1254.038 ms. Candidate intervals are
0.120 ms and 252.767 ms. Reply-to-observed-exit is respectively 1253.839 ms
and 252.647 ms. The recorded request, decoded reply, CLI end, and observed
exit increase in order and finish before the five-second deadline. The
request and reply traces use macOS `CLOCK_MONOTONIC`; the Python helpers use
`clock_gettime_ns(CLOCK_MONOTONIC)`. The preflight explicitly exposes the
different process-relative Python monotonic origin. Earlier preparation and
baseline-build timing fields are not mixed into this cross-process comparison.
The request sample precedes diagnostic emission and frame write; the reply
sample follows decoded frame read. The exit sample follows blocking wait
return and can include unbounded observer scheduling delay. These are audited
saved observations, not kernel event timestamps or newly measured intervals.
One baseline and one candidate sample support only this narrow role-Off flow.
They do not establish a distribution, general speedup, exit deadline, exact
dynamic poll attribution, or connected-server/native/Windows acceptance.

The raw cleanup receipts retain code 0, no signals, PID/group/socket absence,
TLS directory removal, and seven CLI calls in each flow. The four first
exclusive bind records pass at the fixed private ports before launch and
after cleanup. The collector does not set `SO_REUSEADDR` or retry a bind.
Empty `lsof` output is qualified by the retained incomplete-visibility warning.
`port-warning-redactions.json` retains original private paths and hashes;
all four archived hashes match the committed redacted records. The saved
validator reports that it checked the private originals against their hashes.
Those ignored parent-worktree originals, binaries, and registry copies were
not reopened or rehashed by this review. Their live existence and contents
remain outside this Git-only review; the recorded validation is audited.

All three saved release build receipts report exit 0 with Rust 1.99.1,
`--locked --offline`, jobs 2, and deployment target 15.0. Cargo and compiler
exec receipts record absolute NI 15. The archived post-exec rustc samples
also show NI 15: 571 baseline, 165 candidate, and 151 final-source samples.
The complete saved stderr retains the `rust-objcopy` SIGABRT and missing
`libLLVM.dylib` warning for each build. Successful stripping is not claimed.
The copied registry inventory and saved validator result are retained as
records, without reopening the source registry or changing toolchain state.

I also read the complete prior diagnosis README at
`a64b33dd53a9ba11f2a887086b2615ef949c80fc`, checked its 39 manifest entries,
and matched all 15 copied origins to their named Git blobs. Its 387 source
records match the candidate parent. The candidate's recorded diagnosis
README hash agrees with that exact prior commit. The declared 397-file source
scope also has no changes from `1fd5144` through the candidate parent. The
diagnosis is on a separate branch and is read at its own commit, not assumed
to be present in the candidate tree. Candidate-to-parent changes do not alter
prior evidence. The raw historical timeout, strict-cleanup failure, first
65202 bind failure, and peer logs remain available at the diagnosis commit.
The historical shutdown entry precedes SIGTERM by 70.532 ms and precedes the
post-join log by 256 ms in the saved wall-clock fields. Those inherited
measurements remain audited, not freshly reproduced. They do not establish a
product P2, a sole historical cause, normal unsignalled exit latency, or
resolution of the old timeout/rebind failure. TIME_WAIT does not prove the
cause of that bind failure.

Review-only exploratory errors were confined to reads: one nonexistent
`crates/daemon/src/ipc.rs` path, an initial assumption that the prior manifest
was an object rather than a list, and an incorrect cross-branch archive
identity assumption. Corrected Git reads established `ipc_server.rs`, the
manifest schema, and the separate diagnosis commit. No collector, product,
or compiler was run or retried, and no raw artifact was changed.

This turn ran no build, Clippy, test, collector, project CLI, daemon, port or
process probe, signal, key operation, network fixture, desktop interaction,
or nested delegation. It changed no source, prior evidence, installed app,
production configuration, TLS, default IPC, service, account, toolchain,
firewall, global DNS, or system setting. It did not resume or reframe any of
the three provider-stopped tasks. Only this review document is committed;
there is no push or publication.
