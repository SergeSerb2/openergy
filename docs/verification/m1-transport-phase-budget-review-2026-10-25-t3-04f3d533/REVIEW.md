# Independent full M1 transport phase-budget review

**Bounded SOURCE CLEAR. No open P1/P2 finding in this exact transport
source/data slice. execution=false; runtime_acceptance=false. The two prior
transport P2s remain CLOSED_SOURCE_ONLY.**

Reviewed independently on 2026-10-25, America/Vancouver. This report owns only
`docs/verification/m1-transport-phase-budget-review-2026-10-25-t3-04f3d533/`.
It creates no execution authority or human approval gate. Candidate records
that say IMPLEMENTED_UNREVIEWED remain immutable historical implementation
records; this separate review supplies the bounded disposition.

## Frozen identities and review coverage

The candidate is `c591ea5941020e556d878ffef0349625ec42be9f`, imported as
`684400fc45f1ecaf032e79b350abb49a7c6df696`. Both commits have exact parent
`04444ec2dce5dd96334d807f9c2caff86aefae64` and whole-tree identity
`3f4aa2fc93b472328fce37b939ae0480f7ad89c8`. The review checkout started clean
at that import. The candidate adds exactly 14 files in its assigned folder.
No pre-existing tracked blob or mode changes.

The full [candidate collector](../m1-transport-phase-budgets-2026-10-25-t3-b31ab18a/transport-collector.py.txt)
is 28,060 bytes and 626 lines, SHA-256
`695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6`.
All 626 lines, module statements, 25 full functions, four patch hunks, six
complete before/after function blocks, and the new phase-table block were
reviewed. This review covers inherited behavior as well as the diff.

The full [baseline collector](../m1-metadata-transport-r4-2026-10-25-t3-53ab1856/transport-collector.py.txt)
at the exact parent is 27,834 bytes and 620 lines, SHA-256
`d312814e74c6f42e2a86d56648450dc145531df6292ea8c79ccf1768f34b14e4`.
The complete baseline was parsed as inert AST data and compared. The full
shared functions were read in the candidate; the complete changed baseline
functions were read in the retained function document. The exact 2,000-byte
patch has SHA-256
`a20c3afc4caa940d49b7e29ccf54f27cca0bfeba18ecb078fddff5abf487e82c`.
An independent byte diff reproduces that whole patch exactly. No patch was
applied to a candidate or baseline file.

The full baseline SOURCE CLEAR review is
`a7e22725cbbf07b5e6ac7e7aa7e7fb0d3dff93b4`, imported at
`81252214f7c391c465510193acf48115ec3d95db`. Its review folder has the same
tree at original, import and exact parent. Its original finding dispositions
were checked against actual candidate source. They were not inherited as a
verdict for the extension.

The full phase contract, observer contract, current false/null plan, bindings,
caller size contract, author observations, static-check records and retained
reader error were reviewed. The full next-operation JSON and README, full
operator plan and full receipt contract were read. The current parent04444ec
metadata README and full public validation/bindings data were inspected.
[bindings.json](bindings.json) pins all 14 candidate files and all 18 declared
immutable public inputs by commit, Git blob/mode, length and SHA-256.
[function-bindings.json](function-bindings.json) covers all 25 complete ranges.

## Closed phases and original-start deadlines

Candidate source lines 20-23 contain the sole added module statement:
`PHASE_WORK_SECONDS`. It admits exactly metadata-source130, metadata-copy130,
prepare130 and controller335. `arguments:78-97` still requires exactly nine
distinct flags with one nonempty value each. Extra, duplicate, missing and
unknown flags fail. The four hash shapes and original-start shape remain
unchanged. Exact phase-map membership supplies no duration option, alias,
profile override or environment override.

`Collector.__init__:221-254` selects only that fixed work value. It derives
work from the supplied original parent start S, settle from work+2 seconds,
and absolute from settle+3 seconds. Both metadata phases retain exactly
S+130/S+132/S+135. Prepare has the same 130+2+3=135 allowance. Controller has
S+335/S+337/S+340, exactly 335+2+3=340. No stage selects a fresh absolute end.
`Collector.deadlines:325-331` records these same selected values.

`ENTRY_NS:8`, `run:455-459` and `check:261-265` retain the original-start and
future-start checks. Admission checks, bounded input reads, guard/snapshot
writes, spawn return, capture/read/write checks all use selected work.
`settle:403` uses min(selected settle end, entry+2s). `run:540` uses
min(selected absolute, entry+3s). A late spawn return fails before capture;
the outer finally closes undrained pipes and enters the existing settle path.
The old comment at line 525 says work130; it is historical metadata wording.
Operative controller checks use the selected fields, not that comment.

The new local 340s allowance matches the existing next-operation controller
step. That allowance includes 120s file admission, 120s flow, at most60s cleanup
and evidence. It does not grant product life after `flow_start+180`.
Prepare/controller/common/watchdog/caller and opaque payload logic do not
change. The full operator plan still forbids detached-watchdog cancellation.
A phase name selects a local transport budget; it cannot authenticate the
payload or prove that the remote internal clocks obey their own limits.

## Inherited admission, route and I/O behavior

Exactly three functions change: `arguments`, `Collector.__init__` and
`Collector.deadlines`. All other 22 full function ranges and all bytes outside
those functions and the single phase table match the baseline. The independent
hashes also match the candidate function inventory and the prior R5 inventory.
Source reasoning was applied to these unchanged paths:

- `canonical:49-51`, `digest:54-55`, `clock_descriptor:58-75` and
  `main:615-626` retain canonical finite ASCII JSON with a final newline,
  SHA-256, trusted local macOS CPython>=3.10, Mach monotonic implementation,
  `-I -S -B`, no optimization and quiet CLI failure. The descriptor includes
  actual resolution and all clock fields. Its hash is not a host/boot identity.
- `directory:100-118`, `leaf:121-125`, `identity:128-130` and
  `read_bound:133-164` retain canonical absolute paths, no-follow component
  walks, final directory ownership, owned regular 0400/0600 one-link inputs,
  size caps, bounded reads and stable FD/name identity, exact size and hash.
  Profile128KiB, payload1MiB and self128KiB remain unchanged. A read or binding
  error reaches failure before spawn; a late OS return cannot establish timely
  success. These are cooperative owner controls, without a hostile same-user
  immutability claim.
- `unique_object:167-173` and `profile_vector:176-217` retain duplicate-key
  rejection, the exact closed profile shape, fixed guard hash and 29-token
  SYSTEM stdin route. The exact executable, 20-token public prefix, three
  ordered private option pairs, selector grammar and final single token
  `/usr/bin/python3 -I -S -B -` are preserved. The parent must authenticate
  the saved guard and compare all 29 original tokens before binding the whole
  profile. Public grammar and a hash shape cannot supply private provenance.
  This review opens no private guard, argv, profile, known-hosts or SSH config.
- `Collector.fail:256-259`, `check:261-265`, `admission_clock:267-268` and
  `publication_failure:270-276` retain the first observed failure, timeout
  latch, separate total publication-failure count and at most16 detail rows.
  A later error does not replace the first failure.
- `Collector.create:278-280`, `write:282-293`, `put:295-307` and
  `bindings:309-323` retain exclusive no-follow files, bounded checked writes,
  actual prefix accounting, file/root fsyncs and exact input references.
  Exclusive0400 O_RDWR `payload.stdin` retains its original snapshot FD after
  fsync and rewind at `run:497-504`; SSH receives that same FD. Later pathname
  edits cannot redirect stdin to a different file.
- `Collector.observe_exit:333-341` and `capture:343-400` retain real handle
  poll status and immediate local time. SSH exit does not substitute for EOF.
  Remaining-capacity-plus-one reads enforce stdout128KiB, stderr64KiB and
  combined192KiB during capture. Overflow latches failure before prefix write.
  The one probe byte is marked dropped; it does not enlarge the saved prefix.
  Hashes, observed/written/dropped counts, true EOF and unknown unread bytes
  remain distinct. Read/write failure or an unfinished stream cannot satisfy
  completeness. The finally closes pipes without a later drain.
- `Collector.settle:402-449` and `run:451-612` retain their full admission,
  retained-handle, bounded settle, final publication and failure paths. The
  only spawn is `/usr/bin/ssh` at lines 511-515, with shell=False, close_fds,
  cwd=/ and the exact controlled environment. There is no retry, alternate
  transport, resumed root, guard reset, overwrite, group signal, remote/product
  signal, process discovery, watcher or added drain.

The no-follow root creation and existing owner/mode checks still precede all
input qualification and SSH. Parent-directory fsync uses the retained parent
FD at `run:460-472`, between work checks at lines 468 and 470. Guard publication
at lines 494-496 follows that barrier and completes its file/root barriers
before `guard_durable=True`. A parent fsync error or late return latches failure
and cannot reach guard/spawn. Prior P2-01 remains **CLOSED_SOURCE_ONLY**.

In `settle:412-433`, only the exact retained live local SSH handle is eligible
for KILL on cap/deadline. The eligibility poll at line 416 is followed
immediately by the retained clock sample at line 417. A real status is recorded
with that sample. None reaches the sole KILL only through line 423, requiring
sample strictly before both selected settle stop and selected absolute. A late
None sets timeout/failure and skips KILL. The KILL record reuses that sample.
An OSError from KILL creates no exit status. Later poll/wait can record only
actual results and uses the remaining selected settle allowance. Prior P2-02
remains **CLOSED_SOURCE_ONLY**.

This closes the source branch, without qualifying interpreter subprocess
internals or promising timely signal delivery. Scheduling, poll/clock/kill/
wait/close/spawn/fsync calls can stall. No source check provides hard OS
preemption, universal cleanup, guaranteed reap or power-loss durability.

## Record layout, publication and independent D4

The collector keeps exactly 10 guard fields, 33 receipt fields and nine
deadline fields, in their original source order. Schema identifiers, input
binding fields, Python fields, stream fields, failure records, output names,
caps and statuses remain unchanged. Both metadata phases keep their original
deadline values as well as their layout. Prepare uses those values; controller
uses work335/total340 in the same fields. New source/hash bindings are required
for this new collector even where a phase keeps its old arithmetic.

`run:554-561` requires no first failure, actual SSH0 observed before selected
work, and both EOF/no drop/no overflow/fsynced streams. Failed fsyncs and writes
cannot qualify this predicate. `receipt.json` is exclusive and capped32KiB;
guard is capped16KiB. A final failure may leave a partial or stale receipt.
The one bounded exclusive side record cannot replace it with success. The
actual returned collector status and external D4 remain necessary, including
when the serialized receipt computed COMPLETE before a later write failure.
The all-local-file sum is 1,310,720 bytes from unchanged per-file caps, not a
new aggregate counter or R9 reservation. Remote child status stays null,
remote state UNVERIFIED, artifact acceptance false and own-exit observation
false. No PID, file, KILL return or EOF is used to infer an actual exit.

The new [observer contract](../m1-transport-phase-budgets-2026-10-25-t3-b31ab18a/parent-observer.contract.json)
lines 21-27 binds selected phase, exact hashes, S and original A before spawn.
It requires a retained actual collector handle and real OS-exit result, then
an immediate same-running-local-Mac clock T strictly before selected A,
BEFORE any receipt read. Late or missing D4 rejects even TRANSPORT_COMPLETE.
Receipt assessment checks the exact phase and every selected deadline field,
real SSH0, both EOF, no loss/error and durable bindings. Its full actual clock
descriptor and same host/boot requirements remain. The later-process variant
still requires the earlier real retained OS-exit record; a COMPLETE file is
not such a record. No observer implementation or new actual D4 is supplied.

Observer record compatibility is narrower than collector layout compatibility.
Lines 29-54 preserve the previous 19 fields and add five null fields:
phase, selected_work_seconds, selected_settle_seconds, selected_final_seconds
and selected_total_seconds. The record keeps its existing independent-exit.v1
schema; the outer inert contract has a new phase-specific schema. The cap stays
16KiB. A strict old 19-field consumer or a consumer fixed to135s is not proven
compatible with this proposal. Future phase-aware consumption and exact payload
binding remain parent integration work. No such executable consumer is added
here, so there is no demonstrated bounded source defect to report from that
unverified integration. This review does not claim drop-in runtime compatibility.

The local raw adapter is **not the R9 reserved publisher**. The full operator
and receipt contracts still require full caller/raw evidence, encoding and
reserved pre-write accounting: 32MiB aggregate, 8MiB/file, 4MiB cleanup including
1MiB terminal, 64KiB accounting and the existing product reservations. The
copied caller size contract remains byte-exact: its incomplete209,116-byte
shape exceeds transport stdout131,072 by78,044 before omitted mandatory facts.
Full1MiB caller capture and a separately bounded outer envelope remain separate
parent payload duties. This extension supplies no compression, raw dropping,
extra writer, cap relaxation, private authentication or reserved-writer bypass.
Missing private/copy/controller raw proof cannot be qualified by this review.

## Current evidence and retained non-acceptance

Parent04444ec public data records one DATA materialization and one SOURCE pass,
53 direct commands actual0, all50 native saved-basis comparisons, and212 retained
raw/status/comparison file references totaling213,334 bytes. The public53-row
command table and212-row file inventory were fully parsed and checked for
counts, unique labels/leaves, internal clocks, declared hashes and total bytes.
The canonical inventory hash matches its public binding. The17 recorded local
transports retain collector0/SSH0/both EOF and D4 inside their own original135s
ends, including15 read-only retention operations. No private raw file or private
receipt was reopened; these are checked historical public records, not new
runtime observations by this review.

Those records do not prove this new collector ran. They supply no new prepare,
metadata-copy or controller acceptance, no fresh complete artifact receipt,
and no ordinary-flow acceptance. The new finite plan and all unresolved bindings
remain false/null. Future materialized source, exact phase/payload, full private
route provenance, same boot/runtime/clock, retained exit/D4 and R9 integration
remain parent duties under the existing authorization. No human gate is added.

Preserve every older failure and limit: earlier metadata FAILED_UNVERIFIED and
first comparator failure/completion; Windows SSH255/noRoute FAILED_UNVERIFIED
with unknown remote state; M1 FAILED_NOT_ACCEPTED25/25, app0/controller1 and
earlier23/25 FAILED; first post65431 EADDRINUSE cause UNKNOWN; original second
post-bind, bundle-after, outcome and archival missing; whole-flow NI15/ordinary
runtime UNVERIFIED; stripping warning and unknown complete outer build duration/
exit; updater operation CLOSED. The first historical local reader AssertionError
and unknown exact differing stat field remain, as does the separate failed
source-leaf lookup. The author's37-vs33 receipt-field reader error is retained.
The exact patch also retains its two valid one-space context lines; the author's
full raw whitespace check was exit2 and is not relabeled clear.

## Actual static observations and final scope

[static-checks.json](static-checks.json) records156 independent data checks,
all passed. They include every candidate blob/mode and all13 candidate manifest
entries, all18 immutable public bindings, original/import/parent identities,
19 unique-key finite JSON documents, two complete inert ASTs, all25 physical
function ranges/hashes, unchanged outside bytes, the entire four-hunk patch,
all seven full Markdown code blocks, literal phase-map data, record layouts,
closed budgets, current public evidence consistency and preserved false/null
states. These are static data observations, not unit tests or runtime passes.
The author's157 checks remain a separate historical claim. No reviewer data
reader error occurred. Truncated display of the compact public JSON was replaced
with full data parsing and bounded output; no target operation was repeated.

No supplied source, AST or new helper was executed or imported. No bytecode
compilation, unit test, mock, runtime test, native/remote/network operation,
private/state access, process/port query, GUI, credentials, build, signing,
child agent, PR, push or monitoring occurred. No TypeScript changed. Only the
new review folder is committed, with hooks and signing disabled.

This verdict clears the exact bounded transport source/data extension only.
Actual timing/caps, signal/reap/OS-stall behavior, filesystem faults/power loss,
future private/payload/observer/R9 integration, prepare/copy/controller runtime,
continuous NI15, native/product/artifact/package/signing/activation/performance/
updater and release acceptance remain UNVERIFIED or retain their existing
FAILED/UNKNOWN/CLOSED dispositions. Full product and ordinary runtime are not
cleared.
