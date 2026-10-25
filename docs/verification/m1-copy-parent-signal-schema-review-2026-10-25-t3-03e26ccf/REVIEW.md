# Independent fresh COPY R4 signal/schema repair review

2026-10-25, America/Vancouver. **CLEAR_SOURCE_ONLY, bounded to the exact COPY
repair below. No new P1/P2 finding. COPY-P2-1 and COPY-P2-2 are
CLOSED_SOURCE_ONLY.** execution=false; runtime_acceptance=false. This review
does not accept a complete dependency composition, prepare, COPY execution,
controller, product, or release. It supplies no execution authority and adds
no human approval gate.

The review started in a clean checkout at
`6ef1663caee7ec8e421daaaf9effca6796ecedf0`. Only this NEW review folder is
owned. No candidate or other old file was changed. No supplied code, checker,
helper, caller, prepare, collector or controller was executed, imported,
evaluated or compiled into executable code. ASTs were read as data only.

## Exact inputs and complete coverage

The candidate is `c82cc12b8a2d709d8ce1e384545475159b24d2a9`, exact parent
`7e5de706e7457de5f4880cdef180b27536dc0d5e`, imported at the opening checkout.
Its only changes are 53 added files in
`docs/verification/m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7/`.
All 53 candidate Git entries equal the import and actual working bytes, with
Git100644 and working0644 modes. [packet-bindings.json](packet-bindings.json)
binds every file, including all supplied data-reader sources, as public data.
The complete 51-row packet manifest and all 52 unique SHA256SUMS entries match;
the manifest excludes itself and SHA256SUMS, and SHA256SUMS excludes itself.
Those exclusions are explicit, not missing payload coverage.

All candidate source locations below mean
[copy-parent.py.txt](../m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7/copy-parent.py.txt).
The complete source is 1,242 lines / 120,718 bytes, SHA256
`94efc9b4de615485827e471c3e3f31250b28ea64f35228955b91154e4ba2a22a`.
The exact source patch is 1,634 bytes, SHA256
`73442f657c57762239f7d2dbabea0f8a533d83593a258d609bc2bf5bf6f1a12d`.
The schema patch is 12,398 bytes, SHA256
`f3e7bd9d56c3f3e7ae3e4574de4edc1999108e32ea70f3d7ebbe6949728bb29f`.
The combined patch is their exact concatenation: 14,032 bytes, SHA256
`f901f0faf35b0fe2541137c0c2979b34b589cc487423a698b55adf74c18fc059`.
The new full receipt schema is 114,228 bytes, SHA256
`8a4ae3e83a744682ac895d62aeca2228da10f22dbe8779653e6dbe54e9cbf665`.

The basis is `85c668f7e63c07e1f9d3cf9e76af05f21898f80d`, import
`cd4fae4155e50e99d033fc78553ad50bbfa7d164`: 120,613 source bytes, SHA256
`cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47`.
I read the complete 311-line / 21,224-byte full review at
`f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5`, import
`931d2c3364e177abd6615d5bd26811d9d854f8a8`; report SHA256
`33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f`.
Its historical HOLD_SOURCE / two P2 verdict remains attached to its old bytes.
All 25 basis files and 21 full-review files were rebound to their frozen Git
objects and current bytes.

I read every current function body, failure and receipt path, module/entry
block, complete parent contract, full new/baseline schemas, template, plan,
inventory, patches, reader errors, bindings and manifests. The long literal
declarations were fully parsed and compared as data. The full upstream operator
plan, receipt contract/schema, next-operation/README, metadata recipe and
startup contract were also read. Relevant common writer, caller authentication,
stdin/main and prepare receipt/collection paths were read directly.

[function-bindings.json](function-bindings.json) independently binds raw physical
slices, decorators, final LF, nested functions and the class. **Only collect
changes. All other 47 functions and the class are raw byte exact.** All 49
declarations match the prior full review's complete baseline ranges/hashes.
[function-coverage.json](function-coverage.json) retains each prior assessment,
adds the current assessment and range, and omits no current declaration.
The old collect/main finding statements are labelled historical; they are not
silently carried forward as current findings. [global-bindings.json](global-bindings.json)
binds all 46 complete non-declaration segments, all byte exact. The complete
before/after Markdown code blocks equal the actual source slices. All 31 simple
literal bindings and six tuple-bound cap names, 37 names in total, are exact.
This coverage uses full raw slices and AST structure, not author substring checks.

Every source/schema repair hunk was independently reconstructed in memory,
including counts, positions, old bytes, context and additions. No patch was
applied to a supplied file. The old 148,792-byte SOURCE-to-COPY patch, SHA256
`a9608f188227ff7a8ae593dec9258e7046cb24469fe6a9f96de98bc485df2aee`, is
retained byte exact and also reconstructs the complete baseline source from the
fixed 37,775-byte SOURCE launcher. See [patch-bindings.json](patch-bindings.json).

## COPY-P2-1: CLOSED_SOURCE_ONLY

The old trigger was a cap/deadline failure with a final live-child poll and
eligible clock sample, followed by Popen.kill's hidden poll. That extra poll
could cross the old end or observe exit and return without signalling while
the parent recorded delivery. The new branch removes that wrapper.

Current source **725-872**, especially **820-842**, has this exact sequence:

1. The same direct Popen handle is retained in HANDLES at **750-762**. The
   cleanup branch requires a non-null handle and no recorded actual reap.
2. `proc.poll()` at **822** returns the actual code. `observe_exit` at **823**
   immediately samples the shared clock. For a real code, **200-207** stores
   actual_exit and reaped before its next clock read, including negative codes.
   No zero, inferred signal code or reap is manufactured.
3. A returned None enters **826**. The same sample must be strictly before
   both unchanged reap_end and COPY_END at **827**. Late None takes the failure
   path without a signal or new wait. Only overflow or timed_out authorizes
   signalling at **831**.
4. At **834** there is one `os.kill(proc.pid, signal.SIGKILL)`. Its PID comes
   only from that retained, still-unreaped direct child. Between the sample
   and this call are local field assignments and Boolean gates. There is no
   second poll, OS query, timer setup, PID lookup, group target or wrapper.
5. `kill_syscall_returned` starts false at **745** and becomes true only after
   that call returns without exception at **835**. It claims syscall return,
   not delivery, termination, EOF or reap. ESRCH/ProcessLookupError and other
   exceptions remain kill-or-reap errors at **839-842**. Unknown actual exit
   remains null unless an actual poll/wait returned a code. No fallback poll,
   retry or later cleanup pass follows the exception.
6. The only wait at **837** uses `remaining(reap_end)` inside the original
   absolute end. No new time, renewed drain or phase allowance is added.

The complete capsule has two poll sites and one wait site, all in collect; no
second waiter, thread, waitpid scan, handle borrower or product/watchdog signal
path is introduced. HANDLES retains each handle until the final os._exit.
Under the trusted POSIX child lifecycle and absence of another or automatic
reaper, natural exit leaves this child unreaped and prevents PID reuse before
the signal. The source does not claim to prove the target's signal disposition,
audit hooks, kernel behavior or bootstrap at runtime. Those premises and any
scheduling pause after the last sample remain unverified. They do not become
new signal or PID-search authority.

The public local 3.9 library files and every retained excerpt were rechecked as
text. Popen.kill -> send_signal -> poll -> _internal_poll can call waitpid and
can return without os.kill. os.py exports the POSIX primitive; the new branch
does not traverse the Popen wrapper. [public-library-bindings.json](public-library-bindings.json)
binds this evidence. No C implementation body, remote installed identity,
actual signal, poll/wait experiment or runtime receipt was qualified here.
No further repair is required for this original bounded COPY source finding.

## COPY-P2-2: CLOSED_SOURCE_ONLY

The old trigger was a truthful negative POSIX exit from a signalled child,
including a negative final eligibility code, in failure-capable full evidence.
The old unsigned schema rejected that actual status. The source assignments
remain unchanged and retain the signed actual result.

The complete new/baseline schema comparison is exact after only these intended
changes: four parent flag renames/descriptions/required-key updates; eight
signed exit domains; and success-only actual0/both-flags-false constraints.
No other schema property, key set, integer bound or branch changed.
[schema-bindings.json](schema-bindings.json) records every complete JSON pointer.

There are four closed 37-key parent collection definitions: base helper,
base caller, and their two success repetitions. Their actual_exit domains are
at schema **620,1150,3029,3445**; kill_eligibility_exit domains are at
**699,1229,3108,3524**. All eight are exactly integer-or-null, without an
unsigned minimum or maximum. Thus a truthful negative signal code belongs to
the failure domain. No synthetic receipt, validator, test or mock was used.
This follows from the complete domains and unchanged actual-code assignments.

The success branch at **3892-3920** intersects helper and caller actual_exit
with const0, kill_attempted with constfalse, and kill_syscall_returned with
constfalse. Its complete repeated object definitions and required keys still
apply. Success cannot use a negative code, missing status, signal attempt or
signal return. PIDs, clocks, counts, bytes, lengths and all unrelated const0
authentication/record predicates retain their exact old domains.

The parent field rename is scoped. Frozen R9 **46908 bytes**, SHA256
`01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`, is
unchanged. Current COPY **913-935** still requires the exact R9 key set with
`kill_delivered`, strict false; it does not accept a renamed key or loosen
the record. inspect_copy_result is byte exact and still reads all 52 actual
record interfaces and all 208 raw/status/comparator references. No global
rename or historical receipt rewrite occurred. No further repair is required
for this original bounded COPY schema finding.

## Unchanged gates and static facts

[dependency-bindings.json](dependency-bindings.json) rechecks all 43 declared
frozen public dependencies plus all 57 candidate/prior-review bindings against
actual Git blobs/modes, lengths, SHA256 and working bytes. These are identities,
not fresh whole-dependency clearance. [upstream-static-data.json](upstream-static-data.json)
records independent complete comparisons of all 17 pins, ten module
materialization/source-check entries and all 50 recipe-expanded ordered
vectors, including direct Autoupdate and daemon objects. All 212 SOURCE rows
equal the public validation's complete record_files list, total213334 bytes;
canonical manifest SHA256 is
`2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee`.
The saved 327813-byte SOURCE result binding, allzero/no-KILL observation and
six boundary bindings remain unchanged. This reads public saved data; it does
not read private raw files or rerun SOURCE.

The unchanged source gates remain in force:

- COPY120 begins at the same first shared sample; helper10 is inside that
  absolute end. COPY work end is old end-2s, reap end old end-1.5s. The selected
  local metadata-copy envelope remains original130+2+3=135. No cross-host clock
  bridge or renewed allowance appears. Real code precedes clock, signed exits
  stay real, EOF is independent, and no drain or signal retry is added.
- Exact SYSTEM helper then PRIVATE,-B,caller,copy,ROOT,original-start,stdin-hash;
  seven stdin keys, nineteen authentication keys and ten startup keys remain
  compatible with the actual caller. Failed helper authentication blocks entry.
  The 2006/46504040 full prefix facts, executable/helper hashes, ten modules,
  ROOT ownership/modes and strict same-start clocks remain required.
- Fixed source/artifact gates, artifact8/136360 bytes, full publisher receipt
  9374/guard1398, operation hash, ten gates and support128 remain exact. The
  SYSTEM3.9.6 -I-S-B common bootstrap, origins, two ENV bindings, no redirected
  pycache and both flat-cache admissions remain exact. Current target bootstrap
  and normal private startup remain UNVERIFIED/UNMEASURED.
- POSTLEDGER still requires the existing ledger and empty lock. The sole
  common.write_bytes call is **707**, critical=False. The unchanged common
  writer uses locked regular precharge, double temporary/final charge and
  exclusive publication; no INITIAL writer, ledger initialization, reset,
  refund or second writer is added. Actual reap blocks every postchild write.
  First collection failure remains sticky; publication errors stay separate.
  Original8MiB/file,32MiB total,16MiB product,4MiB cleanup and64KiB accounting
  reservations remain exact. Maximum streams are not promised to fit.
- The full result still requires copy-files-before, all50 vectors, and
  copy-files-after in order. It reads full52 records and208 refs, actual0/reap/
  EOF/no error/loss/kill, typed serial clocks, exact pid/argv/direct/collected
  equality, both complete file facts and both saved-basis/fresh-SOURCE raw
  comparisons. Only -verify sorts complete strictUTF8 splitlines; duplicates
  and empty lines remain significant. No native command or generic validator
  is added by this review.
- CONFIG/template/stdin/console schema bytes are exact. execution is false;
  all14 duties, both review objects' slots, template hash and all9 prepared
  slots remain null. The sole CONFIG token, prefix/suffix, canonical JSON,
  parse-only normalized AST and exact current-token restoration were rebound.
  No future render was made. The independently recomputed finite CONFIG bound
  is9977 bytes, token19956, complete render138912<=1MiB. See
  [template-bindings.json](template-bindings.json),
  [byte-bound-observations.json](byte-bound-observations.json) and
  [plan-and-config-observations.json](plan-and-config-observations.json).
- The required full receipt remains <=262144 actual encoded bytes, including
  its own common conversion. State COPY_DATA_COMPLETE_EXTERNAL_PENDING is a
  data gate only. External SSH/collector/EOF/D4/retention/operator fields remain
  null, controller_operator_gate stays BLOCKED and controller_started false.
  FIFO/nonblocking/PIPE_BUF>=512 admission precedes target growth. Full receipt
  barriers and root close precede one exact-count compact<=512 raw console write
  and os._exit. Late/short/EAGAIN/EPIPE/close/exit/EOF failures remain failures;
  no fallback receipt or buffered finalization is added.

## Frozen dependencies and separate prepare work

[frozen-signal-boundaries.json](frozen-signal-boundaries.json) checks the complete
frozen source identities, enclosing declarations and exact contexts. R9 caller
**474-485** still calls proc.kill at484; SOURCE launcher **504-516** still calls
proc.kill at515; phase collector **412-430** still calls process.kill at427.
The matching hidden-poll failure patterns are not repaired or newly cleared
by this COPY packet or this report. The root's current local CPython3.15
collector audit is reported in the commission; it is not independent current
local-composition evidence from this review. Whole direct-signal composition
remains PENDING. Historical allzero/no-KILL SOURCE facts remain untouched.

Prepare R4 `c85725bdd7dc15ab69a065f7bc2b9cd3592b5e6e` / import `a302d4b`
remains HOLD_SOURCE for PREPARE-P2-3 on old `88e4444`. Its original fresh-launch
and named-cache cases remain CLOSED_SOURCE_ONLY for their stated scope.
Separate prepare R5 `02f94afbe3e1001b8d3e8fc4d79976475485f661`, import95181be,
has actual source109444 bytes, SHA256
`b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9`.
The closed260/264 root/direct-signal repair has its separate independent R6
review pending per the commission. No whole-prepare verdict is assigned here.

I read actual COPY inspect_new_prepared **538-607**, generic
collection_complete **211-220**, and the actual R5 collection/receipt paths.
COPY has no closed prepare-primary key set and neither path reads
kill_delivered. Full receipt-primary equality, exact source/review identity,
actual0/reap/EOF/noerrors, raw publications, clock enclosure and tuple/source/
artifact checks are unchanged. The prepare parent field rename alone needs
no COPY relaxation. This is only field-interface compatibility; it does not
accept prepare or invent any of its nine future bindings. The fixed prepare.py
pin remains `16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`.
See [prepare-field-compatibility.json](prepare-field-compatibility.json).

## Evidence integrity and limits

The three reviewer-owned readers produced4408,4633 and148 successful identity,
patch-row, inventory, schema and AST-data observations respectively. These
counts include thousands of raw inventory/patch comparisons, not behavior
qualifications. No supplied checker ran. All supplied original four author
reader errors, one prior reviewer error and two repair-author notes remain
bound to unchanged files; the copied records are byte exact.
[reader-notes.json](reader-notes.json) retains six corrected reviewer mistakes,
including path/manifest/header assumptions and a clock-call classification.
No candidate or acceptance predicate changed to fix the reviewer tools.

The actual complete candidate whitespace check remains **exit2**, only for
the42 historical literal single-space blank patch context rows. The
patch-excluded check is **exit0** and all three new repair patches are
**exit0**. [whitespace-observations.json](whitespace-observations.json) retains
the raw commands and reversible JSON transcripts. Historical patch bytes were
not stripped; the narrower zero results are not reported as global zero.

The opening inventory binds all4515 prior tracked entries. The final
preservation record compares old Git tree/index blobs and modes plus actual
working bytes/modes again. The candidate's original4422-row parent inventory
also matches that exact historical tree and the current opening bytes.
Only this new folder is committed, with hooks and signing disabled. Its final
commit identity and clean checkout status are reported outside this packet,
to avoid a self-referential report hash. [SHA256SUMS](SHA256SUMS) binds this
report and review payload; the review manifest excludes its own self-hash.

No private/state input, network, remote/SSH, native/runtime/GUI, process/port
query, test/mock, build, signing, child agent, PR/push or monitoring action
occurred. Public local Git/text/JSON/hash/AST-data readers were the only
verification tools. No TypeScript changed. The explicit execution=false
commission excludes end-to-end execution.

Actual runtime, signal delivery, reap, EOF, current target bootstrap, continuous
NI15, clock-rate/enclosure, OS-stall/preemption, hostile same-user immutability,
crash/power durability, pipe delivery, D4, operator signature/entitlement/copy
assessment, native ABI/UI/performance and product/release acceptance remain
UNVERIFIED. No prepare, COPY or controller actually ran.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431 EADDRINUSE
cause UNKNOWN; whole-flow NI15/runtime UNVERIFIED; missing original second
post-bind/bundle-after/outcome/final archive; stripping warning and unknown
complete outer build duration/OS exit; Windows SSH255/noRoute and remote state
UNKNOWN; V-READER-STARTUP UNVERIFIED; all failed files/guards; and updater CLOSED.
No failed, unknown or missing evidence is promoted by this bounded source clear.
