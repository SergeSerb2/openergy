# Independent source/data review of the combined ordinary recorder and diagnostic

2026-10-25, America/Vancouver. **SOURCE NOT CLEAR: one actionable P2; no P1.**
Candidate `1ecac53902893d9a4fc2281b7967679e7121dd49` cannot launch its first
pre-bind port query through the composed port method. No additional concrete P1/P2
was found in the full priority delta, diagnostic body, or adjacent recovery
paths reviewed below. Runtime remains **UNVERIFIED**.

Reviewer checkout: `/Users/serge/.t3/worktrees/openergy/t3-392ae27f`, initially
clean at that exact candidate. The authoritative parent checkout is
`/Users/serge/.t3/worktrees/openergy/t3-52422617`. Only this new review folder
is owned. The candidate was neither repaired nor integrated. This record grants
no executable receipt, approval, or runtime acceptance. It adds no human
approval gate to ordinary source work.

All source locations below are under
`docs/verification/ordinary-combined-recorder-diagnostic-2026-10-25/source/`
unless another package is named.

**P2-1: restore the missing session argument in the port helper's coverage tuple.**

Primary location: `controller.py.txt:260-261`. Contract and consumer:
`coverage.py.txt:34-41` and `common.py.txt:268-272`. First caller:
`controller.py.txt:343-345`.

The priority candidate passes `(self.runtime, actor, self.spec)` to `command`.
The composition changes this to `(self.runtime, actor)`, where actor is selected
from `main` and `postflight`. `command` unpacks that tuple into
`CommandCoverage(*coverage, argv, deadline)`. The constructor requires five
arguments after `self`: `runtime, actor, spec, argv, deadline`. The composed
call supplies only four. Python argument binding therefore raises `TypeError`
for the missing `deadline` before the constructor body or the helper's Popen.
This conclusion follows from static argument counts; no code was invoked.

Every flow that reaches the first `_flow` port call takes this branch. It
cannot obtain the original pre-bind snapshot, attempt the first strict bind,
reach helper admission, or launch the peer/app flow. The same defect also
affects the postflight calls if they are reached after failure handling. It
occurs before `Controller.port`'s bind exception handler, so the new bind-failure
diagnostic cannot repair or describe this argument failure. The surrounding
abort/recovery path still fails the attempt; that does not make the candidate
useful for the requested fresh ordinary flow.

Restore the priority candidate's exact three-item tuple:
`coverage=(self.runtime, 'main' if label == 'pre' else 'postflight', self.spec)`.
Regenerate the affected controller binding, materialization binding,
composition patch, checks and checksums. Review that exact new revision.
Do not remove helper recording or relax its constructor. No fix was made here.

The package README's claim that the original coverage argument is retained,
and `source-checks.json`'s instrumentation-preserved claim, miss this regression.
Their hashes correctly bind the defective bytes. The bindings record this
claim as false. Of five active `command` call sites, four have three-item
tuples; the composed port call alone has two. These are two phase uses of one
source defect, not separate findings.

**The complete priority repair was reviewed independently.**

Priority input is `43446949beac4dd445dfce980da9cd7f9a869ceb`, imported as
`c884126196f0215094896f29649af4b895befd72`, in
`docs/verification/ni15-controller-coverage-2026-10-25-t3-a6d1c0b1`.
The review covered its full design, responses, checks, disabled handoff,
input bindings, 36-hunk delta, all nine source texts and their error paths.
The eight carried modules were read in the combined package and independently
proved byte-exact to priority. The original priority port method was separately
reconstructed and compared. Its complete delta was not treated as already clear.

`Controller._launch` retains its original intent, handle, exact wrapper
registration, and guarded app-first peer-stop gate (`179-232`). Its new suffix
calls `observer.observe(..., focus=label)` immediately after that gate and
collects direct handles in `finally` (`233-244`). This is actual prompt source
acquisition, not only a terminal rejection rule. It precedes normal tick sleep
and the next full-snapshot helper.

`Inventory.observe` still restores and begins the serialized uncertainty
transaction before `_observe` (`ownership.py.txt:230-254`). Prompt acquisition
uses the existing targeted reader with registered child/controller PID-start
tokens inside that transaction (`256-299,318-319`). Each extra query reserves
one round and two rows before its read. Its raw rows, races, absence and times
are published before the next query. The unchanged `identify` predicate
governs wrapper/product ownership; complete identities are retained before
NI15 validation. Partial reads spend the existing pending allowance. Explicit
ESRCH ends acquisition without inventing a product sample. A verified wrapper
can be observed again with a bounded parent yield; no product delay or repeated
launch is added. All work remains under the existing gate/CLI/flow deadline.

The subsequent full snapshot remains mandatory (`ownership.py.txt:320-430`).
Prompt data cannot grant absence, peer permission, or signal authority. An
exception before complete full processing leaves the lost/dirty handoff and
blocks successor absence. This can yield failed bounded recovery with the
peer retained; it is not a clean-recovery promise. Existing metadata-only
uncertainty, original pending clocks/counts, registered-start loss checks,
sticky overflow and exact checkpoint revision guards remain intact.

The full snapshot now asks for the controller and armed-watchdog starts and
retains separate control rows. These do not enter owned-product records or
signal sets (`ownership.py.txt:320-350`; `coverage.py.txt:12-29`). Identity,
argv, UID, group and parent checks govern these rows. Explicit controller
ESRCH is required for the allowed watchdog reparenting. Main flow completion
sets its phase before observation (`controller.py.txt:370-373`); recovery sets
the current cleanup phase before its full observation
(`watchdog.py.txt:202-208`). Final coverage requires both controls in flow,
flow-complete and cleanup:all-exits, plus a real postflight controller sample.
An initial ready identity cannot fill those late samples.

`common.command` keeps one real helper invocation and observes its retained
child before the first communicate wait and after timeout slices
(`common.py.txt:263-305`). The recorder's intent, spawn, actual coherent
parent/child rows, PID-start, executable/argv, UID/group, NI15 and actual
complete OS-exit records are checked at `coverage.py.txt:32-104,150-177`.
The Python actual-executable binding comes from the in-memory session. No
additional observer helper or session-file reload is added to targeted recovery.
Budget/publication errors remain sticky acceptance failures. Sample errors
remain in the ledger and fail its audit. They do not suppress successfully
returned probe data or authorize a product signal. An unclosed intent,
spawn failure, missed short helper, attempted-read gap or unknown exit cannot
satisfy the coverage policy.

The original helper exception kill/reap body is AST-exact. Recorder work uses
the existing child-work share and enclosing absolute deadline; it cannot
borrow product cleanup time. This is a source bound, not measured proof about
uninterruptible OS operations or helper observability.

`audit` matches actual stage rows against retained identities and requires
separate wrapper and product samples for all four direct roles. It requires
both daemon roles, separated later product samples, app coverage with recovered
operation, and peer cleanup coverage (`coverage.py.txt:107-207`). Main launch
coverage is required after acquisition; missing peer-status coverage aborts
before app launch. Cleanup peer-stop gaps cannot stop cleanup but fail final
coverage. `coverage-before-postflight.json` retains the assessment before
strict binds; final require and sticky failures prevent acceptance when any
required evidence is missing (`controller.py.txt:564-596`). Samples remain
point observations. Ultra-short stages can still be missed and must remain
UNVERIFIED. Continuous NI15 is never established.

**The diagnostic body and integration retain their bounded failure scope.**

Port input is `2aefaf4056a8d35a005f38e7856ffad89d771a82`, imported as
`e4a2be2d2e92433d8b42c943bacc158b1687023d`, in
`docs/verification/ordinary-port-failure-diagnostic-2026-10-25-t3-619a42c8`.
Its full design, checks, bindings, delta and disabled plan were read. The full
independent report at `90a221bf01aa8359488913ff567ca11d7e77fa47` was also read.
That report left priority composition unreviewed; its verdict was not inherited.

Apart from P2-1, the composed port method equals the standalone method after
removing its recorder keyword. The new diagnostic module is byte-exact.
Only a failed postflight bind with EADDRINUSE and `bind_succeeded=False`
enters the diagnostic branch (`controller.py.txt:279-332`). Failure and the
original snapshot/exception metadata enter memory before best-effort receipts.
Separate query/publication handlers precede the bare rethrow. The outer
postflight loop cannot continue to the second port or archival after that
exception. No query result can make the bind pass.

The query uses one fixed `netstat -AanW -p tcp` helper. Its deadline is at most
0.5 seconds and a subset of the original port deadline, with 0.05 seconds held
for publication (`controller.py.txt:305-327`). The module keeps its 80/20
read-versus-direct-helper-reap partition, nonblocking pipes, 262144-byte
combined read cap, 64 selected rows, 1024-byte row prefixes, 4096-byte retained
stderr and 98304-byte encoded record cap. No reserve or deadline grows.

Only the retained unreaped direct diagnostic child can be killed/reaped
(`port_diagnostic.py.txt:110-117,143-166`). A failure to reap retains that
handle in the separate in-memory map and leaves INCOMPLETE. The earlier
product cleanup result cannot certify this later helper's absence. No product
intent/exit ledger, owned identity, group signal or product signal is added.
There is no separate NI15 sample for this failure-only helper. The branch
already has a FAILED bind and always throws, before any final coverage or
passing outcome. Thus this omission adds no acceptance path and does not alter
the priority helper ledger or process-row/round counters.

The parser keeps exact local/remote port matches distinct and applies no TCP
state allowlist. Decode/header/layout/read/row/exit/reap limits block complete
collection. Target-M1 flag support, output layout and visibility remain
UNVERIFIED. Later state rows and empty output cannot identify the bind-time
cause. Publication failures can leave only in-memory markers. Neither those
markers nor COLLECTED_CONTEXT_ONLY prove durable evidence or cleanup PASS.

**Protected R9 contracts remain present in source.**

The exact R9 and M1 admission reviews, actual carried source and reconstructed
admission were used to assess each earlier repair:

1. Actual Python executable/hash and wrapper argv0 remain separate from the
   selected launcher binding. `_launch`'s entire original prefix remains exact;
   `identify`, `wrapper_argv` and `exact_argv` remain exact.
2. Complete macproc/probe texts are byte-exact. Staged exe/argv coherence,
   final BSD identity and reused-start metadata-only privacy remain intact.
3. All-handle polling, abort-before-I/O, bounded pipe wakeup and drain/discard
   remain. `collect` adds only the sticky coverage-error operand. The prompt
   acquisition suffix still collects retained direct handles in `finally`.
4. Pre-spawn intents, retained handles, one launch per role and exact sealed
   intent/spawn/handle/PID/real-exit sets remain. The watchdog terminal checker
   is exact; missing direct evidence cannot yield vacuous recovered:true.
5. Same-start product adoption still needs the exact wrapper gate and matched
   live controller or explicit controller ESRCH. Unknown orphans, groups and
   metadata-only tokens gain no adoption or signal authority.
6. Fresh single-PID signal checks, actual successful TERM timestamps, full
   five-second grace, later-advance KILL and separate five-second settlement
   allowance remain. Signal/targeted methods differ only by recorder keywords;
   `Recovery.stop` is exact. No allowance expands.
7. Shared locked pre-write charges, immutable first receipts, failed-charge
   retention, 8 MiB/file, 32 MiB total, eight 2 MiB product slots, 4 MiB cleanup
   including 1 MiB terminal and 64 KiB accounting remain exact. New records
   spend those allocations. Reserved archival remains rename-only after absence.

R6's terminal-read versus actual-watchdog-loss boundaries remain exact after
removing only the new cleanup phase label. The one fallback is assigned before
its loss receipt and stepped outside receipt I/O (`controller.py.txt:394-458`).
Recorder failures cannot suppress returned targeted probe data. R8/R9's shared
uncertainty restoration and revision guards remain in main, watchdog, fallback
and cleanup-launch Inventories (`ownership.py.txt:59-254,566-596`). Prompt
acquisition is inside, rather than ahead of, that transaction. Interrupted
processing cannot manufacture an empty successor or premature peer permission.

The original 120-second flow, 60-second cleanup, start+180 product end,
6000 rounds/24000 rows, main4976/19904 and cleanup1024/4096 remain. Prompt and
helper reads reserve their own counts; full control rows add to the existing
full-packet charge. The one zero-row initialization round is explicitly retained.
Extra evidence can exhaust existing time/count/byte capacity sooner. That is
failed coverage, not permission to enlarge limits. Product source, predicates,
frame/request/stalled-write/wait bounds and launch vectors remain unchanged.

**Independent integrity checks bind the defective candidate exactly.**

New standard-library readers used only Git/file bytes, JSON, AST data, hashes
and strict in-memory patch reconstruction. No supplied AST was evaluated.

- All 27 priority, 12 diagnostic and 20 combined files match original commit,
  imported commit, candidate HEAD, this checkout and the authoritative parent
  checkout. Their 26/11/19 checksum entries match their full file sets.
  Priority manifest records and all package JSON data also match/parse.
- All 41 priority dependencies, 24 diagnostic dependencies and 39 combined
  input records match their declared lengths/hashes and relevant immutable
  Git blobs/modes. The bindings retain 254 commit/path records for 195 distinct
  repository paths, including the protected reviews and indexed older evidence.
- Admission's 7 hunks, priority's 36, diagnostic's 7 and composition's 3
  reconstruct with exact old/new positions, counts and removed bytes. All
  resulting module bytes match. No runtime module was materialized on disk.
- All ten final texts parse as AST data and match source-checks and
  materialization hashes. All eight other priority modules are byte-exact;
  the new diagnostic module is byte-exact. Limits, coverage policy and disabled
  operation template are exact. Both future plans retain execution=false.
- All 76 unchanged admission functions compare exactly. Nine narrowly
  normalized comparisons independently preserve the documented transaction,
  signal/targeted, original priority port, cleanup/advance, launch prefix,
  collect and original helper kill/reap structures. The composed controller
  differs from priority only in port and the declared imports. These checks
  do not validate the broken tuple; the separate argument audit finds it.
- Original/imported full-index binary diffs are identical for each base:
  priority241634 bytes, SHA-256
  `ca63688418e61ac5d5dcaeb73d66839efecd835dd9f2f25dca66e75b21c986a5`;
  diagnostic99666 bytes, SHA-256
  `6fdf282c8b347ab745a8396e0374a2af22a7b87ec5f3e35f17914d84ff4d2835`.
  Candidate composition diff204931 bytes, SHA-256
  `129a9c851319d7fe530713b0ed25170b15670cb5a040078841bc2a26f24cc82c`.
- All 87 older indexed evidence files match original Git blobs, hashes and
  checkout bytes: 410149 bytes. Their 23/25 FAILED_NOT_ACCEPTED result remains.
  The bases' retained first reader/whitespace failures remain unchanged.

Combined controller SHA-256:
`25348029134e10e7d587a5018d081cb3c7bfbe2025e257a0dd05beff6ba15e79`.
Combined package Git tree: `7fa97d16dd468c26d0667c83e77cbfea6b9f6d14`.
All ten hashes and exact inputs are in [bindings.json](bindings.json), SHA-256
`72f68086bc12221a270a17e613c53cde76f72b96864af9bd13e743f1e1153742`.
SHA256SUMS binds this report and bindings; the documentation commit binds all
three files.

The complete public saved-attempt review and bindings were rechecked. M1 stays
**FAILED_NOT_ACCEPTED**: app25/25 and actual app exit0, controller exit1, first
strict post-bind65431 EADDRINUSE. Both CLI product stages, late controls and
helper coverage cannot be filled by inherited claims. Whole-flow NI15 remains
UNVERIFIED and the cause remains UNKNOWN. The private archive was not reopened.
Successful known-identity recovery does not supply the missing second bind,
original archival, bundle-after or final outcome gates.

Only this review folder was written. No supplied program/module/checker/AST
was executed, evaluated or imported. No helper/native parser, live OS/process/
port, remote/network, private-path/payload, fixture, build, signing, product,
credential, test/mock, desktop or child-agent operation occurred. No TypeScript
changed; compiler/runtime checks were outside the explicit source/data scope.
Hooks and commit signing are disabled. Whole-app runtime, native UI/input,
Windows, updates, release, performance, universal containment and host
restoration remain UNVERIFIED. The expired Sparkle scope stays closed.
