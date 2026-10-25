# Independent full M1 COPY parent source/data review

2026-10-25, America/Vancouver. **HOLD_SOURCE. Two P2 findings; no P1 finding.**
execution=false; runtime_acceptance=false. This is a full review of the exact
candidate, including unchanged functions, module blocks, failure paths and data
contracts. It creates no executable receipt, execution authority or new human
approval requirement. Preparation, COPY and controller have not run in this task.

**Exact candidate and scope.** Candidate
`85c668f7e63c07e1f9d3cf9e76af05f21898f80d` has exact parent
`9379e2b50b566e3346ec934b322ef142db2ade39`. The review checkout started clean at
import `cd4fae4155e50e99d033fc78553ad50bbfa7d164`. Its 25 capsule files equal
the original candidate, imported Git tree and working bytes, with Git100644 and
working0644 modes. Candidate adds only that folder. Its retained 4,291-entry
parent tree inventory equals raw `git ls-tree -r` output exactly.

All candidate source locations below mean
[copy-parent.py.txt](../m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f/copy-parent.py.txt).
The full source has 1,241 lines and 120,613 bytes, SHA-256
`cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47`.
The full 1,630-line, 148,792-byte
[patch](../m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f/from-cleared-source-launcher.patch)
has SHA-256 `a9608f188227ff7a8ae593dec9258e7046cb24469fe6a9f96de98bc485df2aee`.
Independent processing of every hunk, deletion, context and addition reproduces
every final raw byte from cleared SOURCE launcher
`f78f2be0fe14e5e4cf81990156cb18a1abb5ff62`: 37,775 bytes, SHA-256
`8d7844a20ce347b7be52c07a516b8d9136bdb5c067e450bea24eb3453441afa6`.
No patch was applied to a supplied file.

**COPY-P2-1 — Popen.kill adds an OS poll after the eligibility sample.**
Source **822–834**, especially **833**, and
[PARENT-CONTRACT.md:157](../m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f/PARENT-CONTRACT.md#L157).
The explicit final poll returns None, observe_exit immediately samples the shared
clock, and the branch checks that sample against both original ends. However,
the subsequent `proc.kill()` is not a direct signal operation in CPython3.9.6.
Its POSIX implementation calls send_signal, which calls self.poll again before
os.kill. That poll can call waitpid(WNOHANG). It therefore inserts an OS call
between the retained clock sample and the actual signal, contrary to the fixed
contract and source comment at833.

The trigger is a cap/deadline failure with an unreaped live child, an eligible
first poll/sample, and a second internal poll that returns None after consuming
the remaining interval. There is no new shared-clock gate after that internal
poll. A later postcheck rejects success but cannot undo a late signal. This is
an extra OS operation in the stated ordering, separate from the acknowledged
scheduling pause after a final sample. Also, if the internal poll observes the
child's exit, Popen.kill returns without signalling; **834** still records
kill_delivered=true. A returned method call does not establish that delivery.

The source evidence is retained in
[public-stdlib-source-evidence.json](public-stdlib-source-evidence.json): local
public subprocess.py **1158–1161,1837–1869,2040–2080**. The local public header
declares3.9.6. Files were read as text only. Their hashes and exact excerpts are
retained; no local or M1 interpreter was started to qualify them. This is library
source evidence, not proof of the remote installed library identity or timing.
That identity remains an external bootstrap obligation.

Minimal repair: use one audited signal operation for the exact retained live
child that does not insert another poll or OS query. Keep the existing fresh
None poll, immediate shared sample, both strict end checks, cap/deadline gate,
one attempt, retained handle and original wait remainder. If a direct signal
syscall is used, take its PID only from that retained, still-unreaped handle;
add no PID lookup, group target or general signal authority. Record successful
signal syscall return separately from a method that can silently skip it. Do
not repair this by granting extra time or asserting signal delivery. This
finding is against the new COPY collect implementation. It does not silently
rewrite the frozen narrower dependency review verdicts.

**COPY-P2-2 — the full failed-receipt schema cannot retain signal exit codes.**
[prepared-copy-qualification.schema.json:620](../m1-copy-parent-capsule-2026-10-25-t3-f65c3d2f/prepared-copy-qualification.schema.json#L620),
**700–710,1153–1163,1233–1243**, with source **200–206,822–837,1184–1213**.
The helper_collection and caller_collection definitions allow only null or
nonnegative integers for actual_exit and kill_eligibility_exit. POSIX Popen
returns a negative signal number for a child terminated by a signal. The
candidate correctly stores that real code without coercion.

The trigger is an allowed overflow/deadline KILL that is actually reaped, or a
child that independently dies from a signal while enough phase time remains
to publish its failed receipt. A SIGKILL result is -9. The parent can publish
the truthful raw status and FAILED_UNVERIFIED full receipt, but that required
receipt violates its own closed schema. An eligibility poll that first sees a
signal exit also puts the negative code in kill_eligibility_exit. The defect
affects failure evidence, not just an impossible success state. Do not convert
the real code to zero, null or a positive number to satisfy the schema.

Minimal repair: give actual_exit and kill_eligibility_exit signed integer/null
domains in both failure-capable child records, including their repeated schema
definitions. Keep clocks, counts, PIDs and byte lengths nonnegative. Keep
success actual-exit requirements at zero. No synthetic receipt, validator,
mock, child or signal was used here: the contradiction follows from the
retained schema domains and the actual-code assignments.

**Complete source and packet coverage.** I read every final source body and
module/entry block. The long literal declarations were fully decoded as data
and compared with their complete public bindings, including all212 SOURCE raw
records. I processed all25 packet files, complete patch, full schemas, null
template, parent contract, function inventory and full before/after document.
[packet-manifest.json](packet-manifest.json) binds each file, Git entry, mode,
length and raw SHA-256. All24 SHA256SUMS entries match.

[function-bindings.json](function-bindings.json) independently records every
physical before/after range, including decorators, final LF, nested functions
and the class. All48 current functions and one class are covered. Across the
55-name union, there are16 UNCHANGED declarations,7 CHANGED,26 ADDED and6
REMOVED. These state counts include the class. Every retained before/after code
block equals its exact corresponding source slice. Unchanged declaration bytes
and hashes agree. [function-review.json](function-review.json) gives a separate
assessment for every current declaration. Coverage is not limited to changed
functions or substring checks.

The source assessment found no other concrete P1/P2 within this commission:

- **Entry, configuration and route:** module1–78 and entry1236–1241 keep the
  first CLOCK_MONOTONIC6 sample before priority, later imports and admission.
  COPY_END is assigned once as start+120s. Real/effective ownership and ownNI15
  are checked. The only configurable source token is CONFIG_TEXT. The embedded
  canonical JSON equals the full null template and schema's inert branch.
  execution remains false; all14 parent duties, all review slots, template
  hash and all9 prepared slots remain null. bind_config rejects these before
  opening ROOT. Full token boundaries and prefix/suffix hashes agree. No actual
  future render was created or accepted. The fixed ROOT and private/source
  paths stay literal declarations. The external29-token route and final ONE
  `/usr/bin/python3 -I -S -B -` token remain parent-owned; no selector or route
  override is added by this capsule.
- **Fixed input closure:** inspect_fixed_inputs reauthenticates the20 original
  inputs, all17 pins and ten source-check module records, then the exact caller.
  Independent raw public hashes agree with pins, module materialization and
  source-checks for every module. All50 ordered vectors independently expand
  from the full metadata recipe: seven objects by seven verbs, then deep
  verify. Direct Autoupdate and daemon objects and all flags remain exact.
  Prepared product environment is data only. Neither child starts with it.
- **Saved facts and new preparation:** inspect_saved_source binds the exact
  327,813-byte result and all212 raw manifest rows/213,334 bytes, six exact
  source boundary files and actual caller status interface. It repeats no
  SOURCE or native command. inspect_published_artifact binds all8 identities,
  total136,360 bytes, full9,374-byte publisher receipt and1,398-byte guard,
  operation hash, ten gates and support128. inspect_new_prepared binds the full
  new prepared bytes/tuple, prepare receipt/guard/raw/status, original shared
  clock enclosure, complete source/artifact/limits/environment data and canonical
  owner0700 same-filesystem directories. Its external local proof uses local
  clocks only. Full saved semantics and actual external prepare0/SSH0/collector0/
  EOF/D4 remain parent duties; booleans and public hashes cannot perform them.
- **SYSTEM common admission:** source610–665 checks SYSTEM3.9.6 -I-S-B,
  fixed Xcode prefix/base-prefix, standard-library-only search paths and module
  origins, no redirected pycache, two fixed ENV bindings, pinned files and
  preloaded imports. The only new module after the supplied import must be
  common. Post-import source paths, time object and every reserve constant are
  checked. The exact common top level has its known standard-library imports,
  constants and definitions, with no top-level process call. Its process helpers
  are never called by COPY. This authorized narrow SYSTEM import occurs after
  ROOT/artifact/prepared/SOURCE/pins/ledger verification and before prefix
  authentication. It is not a private-Python entry or a prefix-before-PRIVATE
  violation. Actual SYSTEM bootstrap/import compatibility remains UNVERIFIED.
- **Cache and launch controls:** reject_entry_caches runs before common and
  again before private entry. It rejects ROOT/__pycache__, all10 legacy.pyc/
  .pyo/package alternatives, and bounded flat-root unpinned.py/.pyc/.pyo/.so,
  directories and symlinks. It deletes nothing. SYSTEM sys.pycache_prefix must
  be None. The local public loader source confirms that -B alone does not
  disable cache reads; these separate controls matter. The private vector adds
  no -I/-S/-X or site exception, and its exact ten-key environment supplies no
  cache-prefix override. Its complete private prefix is authenticated, but
  normal private startup and possible startup effects stay UNMEASURED.
  collect samples launch_started_ns at754 and uses that fresh value at755,
  immediately before Popen757. The old begun sample does not gate the750ms
  launch floor. No analogous stale-sample/cache gap was found in these COPY
  controls. Later scheduling/OS stalls remain possible.
- **POSTLEDGER writer and one attempt:** existing_ledger requires the existing
  owned one-link regular0600 ledger and empty lock with closed integer values.
  No INITIAL writer, ledger initialization, reset, refund, rebuild, missing-lock
  recreation or alternate writer exists. Every parent output goes through the
  single common.write_bytes site707, critical=False. Unchanged common uses
  locked regular precharge, double temporary/final charge and exclusive final
  link. Actual encoded size gates precede growth. File8MiB/total32MiB and
  product16MiB/cleanup4MiB/accounting64KiB stay exact. Parent fsyncs, rereads and
  checks time after publication. It blocks writes while a retained direct child
  is unreaped. The durable exclusive parent guard precedes both launches. Old
  once/result/auth/raw/comparator leaves must be absent. No retry or second
  entry follows a failed helper, private spawn or publication.
- **Authentication and private entry:** collect retains both direct handles,
  stores real exit/reap before later clock reads, keeps EOF distinct, and bounds
  combined reads to1MiB plus one detection byte. No drain is added. Partial
  stdin writes feed only the same caller. First collection failure is latched
  before writers; publication errors remain separate. The sole SYSTEM entry
  helper must complete0/reap/both EOF/no errors and close within its10s slice of
  the original120. inspect_entry checks the full2006/46,504,040 prefix facts,
  digest, module10/root700/runtime facts and helper's own elapsed duration.
  The pinned helper also checks executable digest/mode700. parent_inspected_ns
  is a fresh sample after the checks. COPY stdin has exactly7 caller.main keys
  and its authentication exactly19 caller357–381 keys; source_file_qualification
  stays null. Canonical bytes and SHA bind the sole unchanged PRIVATE,-B,caller,
  copy,ROOT,original-start,stdin-hash vector, cwdROOT and exact ten-key startup
  with HOME=ROOT and TMPDIR=ROOT+'/'.
- **Original time limits and collection:** helper work/reap, caller52 records,
  comparisons, all writes, close and console share COPY_START+120. Caller
  work ends at original end-2s, reap at end-1.5s. The unchanged selected local
  metadata-copy collector remains130+2+3=135 from its original local start.
  No cross-host origin/offset/rate/latency assumption, renewed allowance,
  product/watchdog/group signal, extra native command, watcher or completion
  pass is introduced. Common's conversion samples local first/shared after,
  discards the gap, deducts1ms and rounds down; original shared alarms and
  postchecks still apply. None of this proves a clock-rate or OS-stall bound.
  COPY-P2-1 remains open despite the explicit source-level eligibility checks.
- **Complete new COPY result:** inspect_copy_result reads the canonical <=512
  caller success tuple, full canonical result and all208 actual raw/status/
  comparator references. It checks52 ordered records: copy-files-before,
  all50 vectors, copy-files-after; actual0/reap/EOF/no loss/error/kill, typed
  clocks, serial order and pid/argv/collected/direct-child agreement. It checks
  both full file facts and both native comparisons. Each actual native raw
  stream must equal both saved basis and fresh SOURCE after exact declared
  bundle substitution. Only -verify sorts complete strictUTF8 splitlines;
  duplicates and empty lines remain significant. No boolean substitutes for
  these reads. The caller's once/auth/raw data must bind the exact parent stdin.
- **Receipt and final boundary:** the required reserved full receipt remains
  <=262,144 actual encoded bytes, including its own common conversion. It binds
  prepared/artifact/SOURCE/helper/caller/full52 proof, all208 refs, first errors,
  publication states and unchanged caps. Failure-path signed exits need the
  COPY-P2-2 schema repair. COPY_DATA_COMPLETE_EXTERNAL_PENDING is data gates
  only. All external own SSH/collector/EOF/D4/retention/operator assessment fields
  remain null; controller_operator_gate is BLOCKED and controller_started is
  false. Before any target write, FD1 must be a nonblocking FIFO with integer
  PC_PIPE_BUF>=512. After the one receipt attempt and root close, there is one
  <=512 raw console write, exact-count check and os._exit. EAGAIN, EPIPE, short
  write, late close/exit or missing EOF invalidates apparent file success.
  There is no retry, buffered shutdown or fallback receipt writer.

**Frozen dependency boundaries.**
[dependency-bindings.json](dependency-bindings.json) verifies all43 declared
public Git inputs at their exact revisions, blobs/modes, lengths and SHA-256.
All current public working bytes equal those frozen inputs. Additional local
public library text is bound separately. These are identity checks, not new
whole-dependency reviews. In particular:

- Caller candidate `9b50dd7e3a672eafe319be33141cb875d54a471e`,46,908 bytes,
  source SHA `01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`;
  R10 `1c0e4bcdc2dddcb64a3277db34d5ccd5e872fab2`, report SHA
  `ad868577561145dcba9e74db7138e5118544e721ca3629f5c0c4f1808203c496`.
- SOURCE launcher f78f2be above; review
  `98a7ecab421ca64b67ac7f57db153be54b70f989`, report SHA
  `3b04ce2543a07325bf5ae114002806064d0b01cd63206837976d04bc1e079dfd`.
- SYSTEM helper7,381 bytes, SHA
  `bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428`;
  review `20187fe1b286851f1416437c40cc27108269b542`, report SHA
  `c7de1f4dcec5f045282e9a5c656aac9d798ba71bf72e969132716bfef4736efb`.
- Phase collector candidate `c591ea5941020e556d878ffef0349625ec42be9f`,28,060
  bytes, SHA `695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6`;
  full review `98b403d1902ac4d0651c230fb7ec28c4cc154035`, report SHA
  `0c3b311ebff23145ecebd1b64ed38f0e6feaaff46c80c0b1d7a8ecc92d769a3c`.

The full operator-plan, receipt-contract and receipt.schema under
m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44 and the full next-operation,
README, startup and metadata recipe were read. The relevant untouched common
writer, helper, caller/main/Writer/fresh_source/record contracts and prepare
output interface were read directly. None was executed. The separate local
observer41bf91d/review91f7c8a/importd3c58c5 remains prepare-only; it supplies no
COPY observer or D4 evidence.

Original prepare06634b5 remains HOLD_SOURCE under
`f6e15d0803d52905636f9b916b50e0c0a23ef05a`. Its source SHA
`3c7c6e639ee64ca2e8ec49b27c993921ee4e7abff90928fd2f498135f2b20135` cannot
be labelled CLEAR by this candidate. Repair
`88e4444b8e5604fe9bfde7e768ea31d138ab64be`, import
`161ffce69c9dd588ef44b348c9fa0efe8ac8f0b2`, remains UNREVIEWED for this
commission; its separate R4 receives no verdict here. Future repair source/review
and all nine actual prepared slots stay null. The fixed prepare.py pin remains
`16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`.
Any later interface change needs its own bounded integration assessment.

**Independent observations, author claims and proof limits.**
[static-observations.json](static-observations.json) records819 successful
independent public identity/structural data comparisons. The count includes
individual patch rows and is not819 behavior qualifications. The supplied
142-check report, earlier141/142 report, all4 author reader errors, final
integrity claims and byte-bound claims were read and retained unchanged. No
supplied checker ran. The independent reader's one corrected AST shape
assumption is retained in [reader-notes.json](reader-notes.json); no supplied
input or acceptance predicate was changed.

The full candidate Git whitespace check is **exit2**, solely for the42 literal
single-space blank context rows in the exact unified patch. The check excluding
that patch is **exit0**. [whitespace-observations.json](whitespace-observations.json)
retains both raw transcripts. Required patch bytes were not stripped. The full
check is not reported as zero or confused with the other-files pass.

Only public Git/file/text/strictJSON/hash/AST-data readers and this new review
folder were used. ASTs were parsed and inspected as data; supplied source was
not evaluated, imported, executed or compiled to executable code. No supplied
checker, capsule, caller, helper, prepare or collector ran. No private/state
read, remote/network/SSH/native/runtime action, process/port query, test/mock,
build, signing, child, PR/push or monitoring occurred. No TypeScript changed.
The explicit source/data commission excludes end-to-end execution.

The opening inventory binds all4,367 prior tracked entries, index blobs/modes
and actual working bytes/modes. Only this NEW review folder is owned. The
preservation record and final commit checks compare every old entry again.
Hooks and commit signing are disabled for the folder-only commit.

This review proves no actual render, startup, while-read/capture behavior,
signal/reap/EOF success, clock rate/equivalence, OS preemption, crash/power
durability, hostile same-user immutability, connected transport, D4, continuous
NI15, native metadata/signature/ABI, UI/performance, product or release acceptance.
All supporting full result/raw/status/comparators/helper/caller receipts must
remain retained in any future authorized flow. A later source CLEAR could not
substitute for those acts or accept product runtime.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431 EADDRINUSE
cause UNKNOWN; whole-flow NI15/runtime UNVERIFIED; missing original second
post-bind, bundle-after, outcome and final archive; stripping warning and unknown
complete outer build duration/OS exit; Windows SSH255/noRoute and remote state
UNKNOWN; all failed files and guards; V-READER-STARTUP UNVERIFIED; and updater
CLOSED. No failed or missing evidence is promoted by this HOLD_SOURCE review.
