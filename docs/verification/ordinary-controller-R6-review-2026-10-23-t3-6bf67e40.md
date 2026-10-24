# R6 independent static review of ordinary controller repair R5

Date: 2026-10-23, America/Vancouver.
Candidate: `ec723d8fde7c65e955c015653f89d24f7cc7f091`.
Reviewer: T3-owned isolated worktree `t3-6bf67e40`, branch `t3/review-controller-repair-r5`,
starting clean at `23a8206a38999c118921a2156036b82172dc42e2`.

**Verdict: two actionable P2 findings; no P1 finding. This exact revision is
not clear for the next ordinary flow.** Resolve these source gaps and review
the resulting exact revision first. This is a source review, not general
release clearance, an executable receipt, runtime acceptance, or a new user
permission gate. The ordinary flow remains already authorized.

All candidate source locations below refer to this immutable folder at the
candidate commit:
`docs/verification/ordinary-app-e2e-controller-repair-r5-2026-10-23-t3-e9c64d62/`.

**R6-P2-1 — Keep an unresolved private candidate visible after its parent exits.**

Locations: `macproc.py:129-139`; `ownership.py:104-109,125-143,161-169,311-320`;
`watchdog.py:221-248`.

`snapshot` can positively read an executable path inside the fresh private
root at lines 133-134, then fail the staged detail read. R5 correctly discards
the staged executable and argv. But the returned metadata row retains no
private-candidate marker. `Inventory._observe` selects a partial row only if
its PID, parent PID, or group is already known. Its next candidate predicate
also cannot recover the private-path classification without `executable`.

The reachable trigger is an as-yet unregistered supervised daemon with its
own group, followed by app exit and a partial detail read. This can also start
with a partial child row while the app is live: that row enters `pending`,
then loses its known-parent relationship when the app exits. On a subsequent
snapshot, BSD metadata still shows the child at PPID 1, and the preliminary
path read can still show the private executable. If detail remains uncertain,
the filters drop the row. Lines 167-169 then clear its earlier pending entry
because it was not selected. The daemon has no owned record or direct wrapper
receipt, and its separate group is not yet in `groups`.

As a result, `absent(..., app_role)` can return true from the registered app
and child records while this unresolved private daemon remains in the full
snapshot. Recovery can advance through `children-stop` and `peer-request`,
publish peer-stop permission, and allow the controller to stop the peer.
Those phases use known-record absence, rather than checking the discarded
partial private candidate. This violates the required app/children-before-peer
order. It also bypasses the stated eight-observation/two-second uncertainty
allowance. A later complete detail read can fail the flow, but cannot retract
the already published permission or undo peer teardown.

Retain a metadata-only private-candidate indication through selection and
validation. Retain unresolved PID/start metadata across reparenting and
query it for resolution or explicit ESRCH. Keep it separate from ownership:
it must block absence and peer permission, but must not authorize adoption,
argv publication from a reused identity, or any signal. This finding concerns
a candidate the probe already detected; it does not ask sampling to discover
every possible process or provide an atomic kernel identity lease.

**R6-P2-2 — Detect watchdog loss independently of terminal-evidence reads.**

Location: `controller.py:324-344`, especially `328-339`; related retained
watchdog-exit state is at `controller.py:93-100`.

`collect` now polls the watchdog before publication and can retain its real
exit in `control_exit`. However, `cleanup` checks and loads `recovery-done.json`
before it checks watchdog loss and constructs `Recovery(..., 'fallback')`.
All of those operations share one `try` block. An exception from either
`exists(done, limit)` or `load(done, limit)` jumps directly to lines 340-341.
The retained exit does not start fallback elsewhere.

The reachable trigger is unexpected watchdog exit while a terminal-evidence
lookup/read repeatedly fails, for example a filesystem I/O error on that
path, with the other identity inputs still readable. A deadline error on that
lookup/read has the same branch. Each cleanup iteration can keep collecting
direct exits, yet skip fallback construction. Line 343 then has no recovery
object to advance. Verified products that needed the lost watchdog's cleanup
can remain until the fixed deadline, with no parent fallback attempt. The
later failed-bounded-recovery record reports failure but does not repair this
lost action path. This is a control dependency on evidence I/O, not a request
to signal a process without fresh verification.

Move watchdog-loss detection and fallback initialization into an independent
action boundary, using the retained handle/exit state regardless of terminal
receipt lookup or parsing. Keep terminal reads independent, preserve the same
cleanup deadline, and retain fresh ownership checks before fallback signals.
The comment at line 342 moves an existing fallback's step outside the read
boundary; it does not protect creation of that fallback.

**Separate assessment of all seven R4 findings and R5 responses.**

1. **Actual interpreter versus selected Homebrew launcher: addressed in
   source.** `ownership.py:49-55,210-214` binds wrapper argv0 to the observed
   actual interpreter and keeps the remaining tokens exact. The product argv
   checks at `ownership.py:215-226,242-245` remain exact. The selected launch
   path, resolved launcher and hash are separate from the actual executable
   hash in `controller.py:410-416`; `_launch` checks both file bindings at
   `controller.py:178-182`. Watchdog argv0 is checked at `controller.py:432-437`.
   This does not freshly establish the actual interpreter or kernel argv on a
   running process. The old Homebrew/Python.app observation remains saved
   evidence, not a new observation by this reviewer.

2. **Mixed exec reads and reused-process argv: the staged-read repair is
   present, with the adjacent unresolved-candidate gap in R6-P2-1.**
   `macproc.py:85-105` stages exe/argv/exe/argv, checks pair equality and
   canonical argv0, then checks BSD identity before returning details. The
   input row is not mutated. Error text exposes only an error type, and a
   changed identity cannot leave staged argv in the row. Known direct wrappers
   remain selected by PID, so their mixed stage reaches the finite pending
   handling at `ownership.py:134-143`. That protection must also retain the
   metadata-only private candidates identified in R6-P2-1.

3. **Evidence failures versus abort, exit collection and recovery: substantial
   repair, with the remaining fallback dependency in R6-P2-2.**
   `controller.py:78-113` polls all retained handles before independent receipt
   attempts and keeps real exits in memory. Abort state precedes I/O at
   `controller.py:115-122`. The bounded nonblocking inherited request pipe is
   set up at `controller.py:422-426`, written at `135-144`, and read at
   `watchdog.py:302-322`. Inventory stores fresh rows and identities before
   publication at `ownership.py:110,151-154`. Signal receipts cannot directly
   veto a verified signal at `ownership.py:289-309`. Failed streams switch to
   drain/discard at `controller.py:45-76`. `common.py:158-167` returns an existing
   first receipt before ledger access, reservation or temporary payload writes.
   The bounded error state and failed-acceptance checks are present. These
   changes do not remove the fallback-creation dependency described above.

4. **Missing post-spawn receipt and vacuous recovered:true: addressed in
   source.** `controller.py:183-201` retains an attempt and publishes a command
   intent before Popen; it retains the handle before post-spawn evidence.
   `controller.py:107-111,348-355` seals launches and permits a collector terminal
   only after all retained handles have real exits. `watchdog.py:169-192`
   requires agreement across intents, spawns, sealed handles, exits, labels,
   PIDs and collector identity. `ownership.py:311-316` rejects an unresolved
   launch intent as absence. The parent also checks its retained sets at
   `controller.py:330-333`. Missing records or controller loss still prevent
   acceptance; these paths grant neither an unverified signal nor a second
   launch. This does not resolve the separate unknown-child gap in R6-P2-1.

5. **Known gated wrapper-to-product exec after controller loss: addressed in
   source.** `ownership.py:198-232` binds the direct receipt, same start and
   group, exact wrapper registration, role, UIDs, NI15, parent identity and
   exact product executable/argv. The new product needs a live matched
   controller or PPID 1 plus explicit controller ESRCH. Existing orphan
   transitions also require ESRCH at `ownership.py:185-196`. Targeted recovery
   uses that policy at `watchdog.py:107-138`. Group membership, an unknown
   orphan, enumeration omission or a reused controller PID cannot grant this
   transition. Direct OS exits remain mandatory. This assessment concerns
   verified direct wrappers, not adoption of an unregistered daemon.

6. **TERM grace measured from delivery: addressed in source.**
   `ownership.py:296-306` masks SIGALRM around the single kill syscall and
   in-memory successful-delivery record. `watchdog.py:62-103` recovers those
   records and tracks TERM/KILL by PID/start. KILL requires five seconds after
   successful TERM and a later advance number. Normal recovery, targeted
   recovery and CLI deadlines use this same stop routine. Late delivery or
   inadequate post-KILL time records failure inside the original deadline;
   phase-entry timestamps no longer authorize immediate escalation. No signal
   or timer behavior was exercised here. Separate read/signal calls still do
   not provide an atomic kernel identity guarantee.

7. **Concurrent evidence/log/archive growth: the reported accounting repair
   is present in source for the stated evidence allocations.**
   `common.py:85-182` initializes the ledger, serializes reservations and writes
   with one cross-process lock, charges before payload growth, retains failed
   charges, and checks per-file limits. Exclusive records conservatively
   charge twice their payload. The fixed infrastructure allowance covers the
   bounded accounting temporaries and counters. Regular stream/observation
   reservations leave cleanup capacity; nonterminal reservations cannot consume
   the terminal-only share. `controller.py:45-76,190-195,421-426` routes captured output through
   draining pipes and the common writer. `exec_owned.py:10-11,34-42` applies
   the subordinate inherited 2 MiB regular-file limit before product exec.
   The full eight-slot 16 MiB allocation is reserved before startup. The slots
   cover app daemon stderr, both three-file daemon log families, and the raw
   report. Adjacent logger source confirms the two rotations. The 4 MiB
   cleanup allocation includes the 1 MiB terminal share; the 64 KiB accounting
   allocation also stays inside 32 MiB. Preparation checks the filesystem at
   `prepare.py:65-66`; archival uses rename, not a second payload copy, at
   `common.py:216-228` and `controller.py:474-485`. These are coordinated
   reservations plus subordinate limits, not merely sampled sums or per-file
   caps offered as an aggregate proof. Exceeding a subordinate product limit
   may fail the future flow. No product source change or general product
   resource bound is established. Sink failure still has R6-P2-2's recovery
   control gap even though the byte reservation mechanism is present.

**Independent static integrity checks.**

I read the complete R5 README, responses, design, operation plan, all eight
Python source texts, all 1269 patch lines, input bindings, source-checks,
original index, manifest, and static reader notes. I also read the complete
R4 review at `cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d`. Its prior verdict did
not substitute for this review.

All 18 R5 package files were byte-equal to the candidate Git objects. All 17
manifest entries matched their byte counts and SHA-256 values. All package
JSON parsed. A new in-memory reader parsed the eight Python files as ASTs only
and checked their line counts, byte counts and hashes against source-checks.
No task AST was evaluated, imported or executed.

The same independent reader checked all 14 bound R3 input files and the R4
review against their immutable Git blobs, disk bytes, sizes and hashes. It
reconstructed all 108 zero-context R5 patch hunks in memory from the exact R3
Git texts, checking removed bytes, positions and counts. All seven changed
outputs matched R5 exactly. `probe.py` remained byte-unchanged. No patch was
applied and no temporary source copy was written.

All 87 original-index entries matched commit
`498190feb00f51039f500ae618ac7de5fb113d86` by Git blob ID, size and SHA-256.
Their bytes also matched both this review worktree and the authoritative
`t3-52422617` copy. No original evidence was changed. I did not repeat the old
421-input or signed-bundle audits. I checked only six adjacent product files
against the immutable original input bindings to support the child lifecycle,
file-slot and rotation reads: AppEndToEnd, DaemonSupervisor, daemon logger,
IPC server, daemon runtime and config CLI. That is a narrow source check,
not a fresh artifact, build, signing or runtime audit.

Candidate hashes independently confirmed:

- Controller: `2b7e788dcf50ccb709a16d701d6c415efbd7929a672754ced37fca3b3ddbbc23`
- Full patch: `0201bd3bf39e69f4a452c97bb09d9b5acb46ae093444bf06ac3ea94f46640d47`
- Manifest: `1e11df56ec8f304465f156d22a5c70b9877cad2d5c29efe5eaf398e9deb714f9`

**Saved evidence, finite scope and remaining proof.**

I read the original full README, first controller and cleanup tracebacks,
saved outcome, and full 25-step recovery report. They retain the shared-PGID
assertion failure, early peer stop, 23/25 result, missing app OS exit and
incomplete late priority coverage. The original flow remains **FAILED / NOT
ACCEPTED**. These are audited saved records, not fresh reproduction. They
establish no product defect. R3 and R5 have source/static records only; no
retry or fresh runtime proof was generated in this review.

The reviewed source/plan retains one launch per role and one ordinary
no-render `--e2e --require-peer` default25 real AppModel/daemon/peer IPC path.
The peer must remain until app/children are absent. Flow/cleanup remain
120/60 seconds, bounded by the predeclared flow start + 180 seconds. The
6000-round/24000-row total retains 1024/4096 for cleanup, with main shares
4976/19904. The 8 MiB evidence-file and 32 MiB aggregate ceilings, exact frame,
request and stalled-write limits, and ordinary routine waits remain bound.
Actual future app/daemon/peer NI15, complete identities, stop/restart coverage,
all direct OS exits, 25 unchanged passing checks, full cleanup and unchanged
artifact bytes still need the later real flow. Rust 1.99.1, jobs2 and
deployment15 are inherited build lineage, not a new build result.

No further concrete P1/P2 finding was identified within these reviewed paths.
The two findings above block this exact revision for the next reviewed
ordinary flow. They do not change its existing authorization or add a human
approval step. Running/relaunching Sparkle remains separate and pending. The
crafted mDNS experiment, original provenance/security review and IPC/Synergy3
import-budget tasks remain stopped. The isolated Swift build gate remains
stopped pending approved isolated instrumentation.

Only this review document was written. No supplied program, task module,
validator, helper, controller, preflight, product or build was run or imported.
No fixture, unit test, mock, fake process, callback/interleaving simulation,
signing, networking, process/port/live-OS query, delegation, desktop driving,
push or publication occurred. All review checks used file, Git, hash, JSON,
AST or patch reads. Live sharing, installed apps, production config/TLS/default
IPC, TCC, login/services/tasks/accounts/firewall/global DNS/toolchain settings,
credentials and existing private keys were untouched. Candidate, prior source,
raw evidence and all indexed originals remain unmodified.
