# Independent static review of the ordinary AppEndToEnd controller repair

Reviewed repair: `8324371b67b056c7ac8667d3f67830f1115c3c31`.
Reviewer worktree: `/Users/serge/.t3/worktrees/openergy/t3-f92a997d`.
Date: 2026-10-23, America/Vancouver. This is the first independent review of
this repair. Verdict: **seven actionable P2 findings; resolve before the one
ordinary execution task.** No P1 finding. No product regression is established.

All repair line numbers below refer to that exact commit, under
`docs/verification/ordinary-app-e2e-controller-repair-2026-10-23-t3-432ee8ea/`.
The eight reviewed sources are `common.py`, `controller.py`, `exec_owned.py`,
`macproc.py`, `ownership.py`, `prepare.py`, `probe.py`, and `watchdog.py`.
I read all 14 owned files, including the entire patch, README, design,
operation plan, original index, and static-check record. The source files in
the repair worktree were byte-equal to the reviewed Git objects.

**P2-1 — Match the actual framework-Python argv0 at the wrapper gate.**
`ownership.py:47-53,192-194`; `controller.py:128-133`.

The executable check uses `spec['python_executable']`, but the next predicate
still compares the wrapper's kernel argv0 to `/opt/homebrew/bin/python3` through
`exact_argv`. The original saved `raw/runtime/app-exec.json` and
`peer-exec.json` show the running wrapper command with the framework
`Resources/Python.app/Contents/MacOS/Python` as argv0. Static symlink reads
resolve the selected launcher to `Frameworks/Python.framework/Versions/3.15/bin/python3.15`,
which is a different file from that Python.app executable. The launcher file
also contains the Python.app target path. Thus the recorded framework-Python
form fails the wrapper predicate even when PID/start/parent, executable,
script, runtime path, and label all match. The controller cannot publish its
registration gate and the peer cannot exec. Bind wrapper argv0 to the verified
actual interpreter identity, with exact matching of the remaining arguments;
keep the launch-path binding separately. Do not weaken product argv checks.
This conclusion uses saved process evidence and current file reads, not a new
kernel-argv observation.

**P2-2 — Detect mixed executable/argv reads across the expected same-PID exec.**
`macproc.py:78-89`; `ownership.py:119-145,192-200`.

`detail` reads the executable, then argv, then checks only BSD identity fields.
The intended wrapper-to-product exec preserves those fields, including PID,
microsecond start, parent, and group. If exec falls between the executable and
argv reads, the row can contain the Python executable and product argv while
passing the final BSD check. `identify` then matches neither stage. This is
raised immediately as an ownership error, rather than retained as a read race
subject to the finite eight-observation/two-second allowance. Each released
wrapper creates this real transition. Stage details in temporary values,
check executable/argv coherence around exec, and classify a mixed stage as
pending within the existing finite limit before publishing a complete row.
Also discard unverified detail fields when the final PID/start check changes:
the present in-place mutation leaves any newly read argv in the partial row
that `snapshot` publishes after a reuse race. Metadata-only race evidence can
remain without retaining the replacement process's command arguments.

**P2-3 — A failed evidence write must not stop later exit collection or all recovery actions.**
`controller.py:35-47,237-266`; `watchdog.py:143-157,94-108`;
`common.py:77-113`.

`collect` polls and writes one child at a time. If an earlier child's receipt
cannot be written, the exception prevents polling every later child. The next
cleanup iteration starts at the same earlier child and repeats the write.
This also happens to an already published receipt at the byte cap, because
`first` runs the capacity check and temporary write before discovering that
the destination already exists. A sink or cap failure can therefore leave
the app's OS status uncollected even though its Popen handle remains available.
Poll all retained handles independently of publication, preserve their exit
results in memory, and treat each failed publication as a failed-evidence
condition without skipping the remaining handles.

Recovery has the same control dependency: when observation cannot append,
`inventory.last` remains unset. The targeted route then calls `first` before
identity handling or signals. A persistent sink failure throws out of that
route on every step. The normal-to-abort transition also writes `abort.json`
before setting its in-memory abort state. Catch those evidence failures at
the action boundary, retain failure state, and continue the bounded recovery
decisions using fresh ownership verification. A failed sink must still
prevent PASS; it must not silently disable the collector and recovery loop.

**P2-4 — Do not certify absence from an incomplete post-spawn ledger.**
`controller.py:107-111,240-243,347`; `ownership.py:88-98,267-288`;
`watchdog.py:236-248`.

The child starts and its Popen handle is retained before `*-spawn.json` is
published. If that publication fails or times out before the wrapper publishes
its exec receipt, recovery knows neither its PID nor its group. Its final
launch set comes only from existing spawn files. With no other launches, the
app/peer absence predicates and collected-exit check are all vacuously true.
Recovery can publish `recovered=true` while this direct wrapper is still
alive. The parent accepts that result and only polls once more; it does not
wait for that wrapper's exit receipt before leaving collection. The overall
flow is failed, but the clean-recovery claim and complete exit collection are
wrong. Make unclosed launch intents and all retained direct-child handles
part of the terminal protocol. A missing post-spawn record must prevent a
clean recovery result. Continue bounded collection, without signalling an
unverified wrapper or authorizing a second product launch.

**P2-5 — Recover a verified wrapper's product stage after controller loss.**
`ownership.py:162-200,226-255`; `watchdog.py:99-118,282-287`.

If the controller exits after publishing the wrapper identity and registration
gate, but before observing the product stage, the product retains that known
PID/start and is reparented to PID 1. Its executable no longer matches the
saved wrapper. The new-stage branch insists that its current parent is the
live controller, so it refuses the product stage. Both normal and targeted
recovery retain only the wrapper record. Every signal then fails the executable
check against that stale stage. The watchdog can leave the exact owned app
and peer alive until its failed recovery deadline. This is a previously
verified direct child with a durable same-start exec gate, not an unseen
orphan discovered by PGID. Permit a narrowly bound stage transition using
that gate, the known start, exact product executable/argv, UID, group, and an
explicit controller-absence check. Preserve refusal of PID reuse and unknown
orphans. Missing direct-child exits after controller death must still fail
acceptance, as the design already states.

**P2-6 — Start each TERM grace after that identity's successful TERM.**
`watchdog.py:20-21,43-44,63-70,113-132,170-176,184-205,223-230`.

The five-second timers start when a phase begins, before observation and
signal verification. `signals` catches a failed verification, but the caller
still advances when the phase timer expires. After delayed or repeatedly
failed reads, the first successful TERM can occur at or after that timer; the
same `advance` call can then enter the KILL phase and send KILL immediately.
The CLI path similarly escalates at `absolute_deadline + 5` even if its TERM
could only be delivered later. This does not provide the declared five-second
TERM grace. Return a successful-signal receipt to the state machine and track
the TERM time per identity. Escalate after its grace, within the unchanged
absolute cleanup deadline. If the remaining budget cannot supply that grace,
record the explicit bounded failure rather than claiming the planned grace.

**P2-7 — Coordinate byte accounting across writers and reserve terminal evidence.**
`common.py:77-113`; `controller.py:68-76,370-385`;
`ownership.py:7-31`; `watchdog.py:250-262`.

The advisory lock protects observation rounds/rows only. Each evidence writer
separately sums HERE and then writes. The controller and watchdog can both
pass that check against the same remaining space and exceed 32 MiB. Direct
product log writes are not in that transaction. The app's two daemon logs
live outside HERE, then archival copies them through raw `open('xb')/write`
without either the stream or aggregate capacity reservation. Archival can
therefore cross the cap before `outcome.json` notices it. No space is reserved
for abort, child exits, or the terminal recovery record; at exhaustion these
records themselves fail and feed P2-3. Use coordinated bounded byte
reservations for all evidence phases and account for product-log/archival
growth. Retain room for finite cleanup and terminal failure receipts within
the existing 32 MiB total and 8 MiB per-file limits. Do not describe periodic
size checks or per-file RLIMIT_FSIZE as a strict aggregate bound.

**Static integrity and original failure audit.**

I independently parsed the eight sources as ASTs, without imports or
evaluation. All recorded byte counts, line counts, and source SHA-256 values
matched `source-checks.json`. A separate file/Git-data reader reconstructed
every patch hunk in memory from the four exact original bases and four empty
new-module bases. It checked each removed byte range and both hunk positions
and counts. All eight results were byte-equal to the repair sources. No patch
was applied to either worktree or to the original archive.

The reviewed controller SHA-256 is
`4a5f4d49f140b9635618a0e2134ceba93921fca52ffa868cd81ed910605199c0`.
The complete patch SHA-256 is
`bb3c18243ebcf403069fc4e1ffa3a0cf2db215e935060b58374af6d58efd5f2e`.
All 87 original-index paths, Git blob IDs, byte sizes, and SHA-256 values
matched immutable commit `498190feb00f51039f500ae618ac7de5fb113d86`.
The original archive is on a separate history; an initial read at the repair
commit correctly returned “path does not exist.” The completed check used
the explicitly named original commit for every archive entry.

I read the full original README and controller; both first runtime/cleanup
tracebacks; first controller and archive-validator failures; frozen-controller
record; complete saved priority samples, recovery report, final app log,
relevant daemon/peer logs, identity/exec/readiness/config records, recovery and
final cleanup inventories, outcome, preparation and limit records. The saved
samples show app PID/PGID 89176 and daemon PID/PGID 89180 with PPID 89176,
UID 501, nice 15. The assertion preceded identity publication. The old
`finally` cancelled its timer; cleanup stopped the peer, then repeated the
assertion. The later report has 23/25 passing checks and failed connection/
rejoin checks. The app's OS exit and recovered-daemon priority/start/group
were not collected. That run remains **FAILED / NOT ACCEPTED**. These are
audited saved observations, not independent runtime reproduction or evidence
of a product connection regression. The archive-count correction is separate
from the runtime failure.

All 29 app records matched their recorded compiled-input hashes at
`7287e9aa4caf6570fc4116a89908d548a258f302`. All current repair-worktree app
records matched the original actual hashes. The exact 22-target Swift set
matched. The original documented bundle.sh difference remains a non-compile
input exception. All 397 daemon records matched their expected hashes at
`2485c1425e4d2f414eb22742dfd73f355d47390c`, and the current exact declared
input set matched by path, size, and hash. The saved 124-record Sparkle
expected/actual inventories agree. I read the build/result/resolution records
as inherited evidence; no build, signing, or architecture tool ran.

File-only inspection of the retained ordinary signed bundle matched all 167
saved tree records, including modes and symlinks. Its Info.plist bytes matched
the saved ordinary plist: `dev.openergy.app`, executable `Openergy`, no private
fixture key or launcher. Executed signed hashes remain app
`53ba6f431811eb1178c39b8b7ad7cdb425b0088e7c893b8dcab8e601ad338c23`
and daemon/peer
`1c558733a56ada70661f8a8493eae7deaf3813ce1bb572d0592fe86900b6b971`.
This is file identity, not fresh signature validation or runtime proof. The
two local SDK header hashes matched the static-check records; declaration
review does not validate the process ABI on a running process.

**Scope and remaining proof.**

The sources replace the shared timer-thread sampler with separate controller
and watchdog processes. I traced their launch/gate, observer-failure, cleanup
request, fallback, targeted-read, app-first stop, peer-permission, CLI deadline,
exit-collection, and terminal transitions, plus the nonblocking budget lock.
Separate actual PGIDs are recorded. PGID membership alone does not authorize
adoption or signals. Fresh single-PID signal checks, explicit ESRCH absence,
initial/recovered identities, retained partial observations, and refusal to
unlink stale sockets are present. The findings above concern gaps in those
mechanisms. Sampling and the final read-to-signal interval do not establish
universal process coverage or an atomic kernel identity lease.

The preparation text creates a fresh exclusive random root and fresh private
directories. It copies only the unchanged ordinary signed bundle. It does not
reuse the failed runtime config or guard, re-sign, compile, select an update
launcher, or change the default product routine. The finite plan retains one
ordinary no-render `--e2e --require-peer` real-model/IPC flow, private virtual
daemons and peer, NI15, 120 s flow/60 s cleanup, 16 MiB frames, the unchanged
1,000,000-byte/300 ms stalled-write check, 6000 rounds/24000 rows and cleanup
reserve, and 8 MiB/32 MiB file/evidence limits. Ports 65431/65432 remain
text-only and unqueried by this review.

After these source findings are resolved and the resulting exact revision is
reviewed, the separate one-flow task can follow the corrected finite plan.
The ordinary flow is already authorized; this review adds no human approval
gate. That later task must supply actual ABI/process identities and NI15,
SIGSTOP and crash/restart observations, all direct-child OS exits, 25 unchanged
passing checks, unchanged bundle bytes, and complete PID/group/socket/port
cleanup with the watchdog's terminal status. None of that real-process proof
was generated here. Running-update/relaunch and all three provider-stopped
tasks remain outside this review.

Only this review document was written. No supplied validator, helper,
controller, prepare script, probe, or watchdog was imported, executed, or
evaluated. No synthetic process/interleaving, unit test, mock, build, signing,
product/CLI launch, process/port/live-OS query, network, desktop action,
delegation, push, or publication occurred. All source/evidence/artifact checks
used static AST, hash, filesystem, or Git-object reads. Prior raw evidence,
first failures, guards, product sources, installed apps, and live settings
were preserved.
