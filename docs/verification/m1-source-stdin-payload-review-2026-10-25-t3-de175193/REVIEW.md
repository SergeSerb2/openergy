# Independent full SOURCE stdin payload review

2026-10-25, America/Vancouver. **HOLD: one bounded P2, PAYLOAD-P2-1.
No P1 found. No SOURCE CLEAR. execution=false. Runtime UNVERIFIED.**
This review supplies no runtime, artifact, native or approval grant. It adds
no human approval gate. Only this new review folder is owned and changed.

The exact candidate is `fa72a0d865f12ae94b91f84400a31f51026c5b16`, imported at
review base `dde69c794ae6b547393c81a3de411b6c9af1d179`. Its original parent is
`a26871ad9fdae2d55ff53544aed26ae0cdf2620e`. Candidate and import have the same
payload-folder tree, `09dce8e36cc2a82a449ea2160c52931fec0e22b0`.
All 11 candidate files equal their candidate Git blobs, import blobs and
working-tree bytes. This is folder identity, not whole-commit tree equality.

All source locations below refer to
[`source-payload.py.txt`](../m1-source-stdin-payload-template-2026-10-25-t3-65b09134/source-payload.py.txt).
The source has 647 lines and 37,122 bytes, SHA-256
`dc208d02222c92418046139ecf622c797881af46b3edc1b8a891742b22f9fd9f`.
I read the complete source, all module statements and 28 functions, and all
contracts in that folder. The supplied `check-static.py` was read as text;
it was neither run nor imported. The add-file patch was reconstructed only
in memory and equals the complete source.

## P2 PAYLOAD-P2-1: Check the deadline after the KILL-eligibility poll

Primary location: **collect, lines 501-508**, especially **502-506**.
Supporting locations: **observe_exit, 428-433**; **until, 149-164**;
fixed work/reap limits **438-440**.

The settle path enters `until(reap_end)`, calls `proc.poll()`, then calls
`observe_exit`. If the actual poll returns `None`, `observe_exit` does not
read the clock. The next branch sets `kill_attempted` and calls `proc.kill()`.
There is no post-poll clock check before that new signal operation. The next
explicit `remaining(reap_end)` is at line 508, after KILL, to bound the wait.

Thus a poll or scheduling interval which returns late with `None` can lead
to a new KILL after the fixed `end-1.5s` settle ceiling. A sufficiently late
return can put that new operation after the original phase end as well.
The surrounding alarm can interrupt many such paths, but does not establish
the missing post-return predicate. In particular, `until` computes its
relative duration at line 151, before handler/timer setup, and later arms that
unchanged duration at line 159. A scheduling delay during that setup can arm
the alarm after its intended absolute ceiling. The late `None` branch has no
clock gate of its own. The context's normal-exit postcheck occurs too late
to prevent the signal already attempted.

This is a source ordering defect in a new action after a call returns. It is
distinct from the documented inability to preempt a call already in progress.
Later timeout/error flags, a failed envelope, or the parent's rejection do
not prevent the late signal. No signal, stall or scheduling race was tested.

Minimal repair: retain the eligibility poll's real return value and take an
immediate shared-clock reading after it, including when the value is `None`.
If an exit was returned, preserve the actual exit and its observation. If it
returned `None`, permit KILL only while that post-poll reading is strictly
before both the unchanged reap ceiling and phase end. A late reading must
record failure/uncertainty and skip KILL. Keep the original bounded wait,
single retained direct SYSTEM handle and unknown/null rules. Do not add a
PID lookup, process group, product signal, second caller, drain, retry or
new allowance. An unreaped caller must still block all later file writes.

The new transport full review reports the same class of gap in its separate
collector. This finding is bound independently to the payload's own lines
above. The transport's other finding, a missing fsync of the parent of a
new run directory, does not apply to this payload: it creates no directory
and requires the exact existing root. This review does not clear transport.

## Retained brief and unresolved dependencies

The requested slice is a SOURCE-only remote SYSTEM Python stdin template.
It connects an independently reviewed metadata caller to the unchanged local
transport: stdout 131,072 bytes, stderr 65,536 bytes, combined 196,608 bytes,
outer 135 seconds. It must retain caller stdout/stderr on M1 within a 1 MiB
combined while-reading cap, publish exclusive initial raw/status evidence
before ledger initialization, and emit at most 16 KiB within the original
shared 120 seconds. It supplies no COPY, prepare, controller, product,
artifact-receipt constructor, SSH route, upload or materializer.

The reduced 50-native-command success shape remains 209,116 compact JSON
bytes, SHA-256
`b0b8fc94cf7b1c44fc31128aee173e192798cb552726d8909c422c1170b50c13`.
That is 78,044 bytes above the transport stdout cap before the omitted
auxiliary commands and mandatory status/timing fields. Its retained
`collected_commands` status copies correctly omit later-added stream refs.
Direct passthrough of that full summary remains inadmissible. There is no
compression, raised cap, truncated-success rule or completion pass here.

Caller R3's complete 715-line source and full interface contract, the R4 full
review, and R5's complete interface/context were read. R5 source was parsed
as data; whole-function AST comparison against R3 confirms changes only to
`Writer._put`, `emit_summary`, `main`, plus added `normalize_startup`.
The original R1/R2 and caller-owned buffered-publication closures are retained
as prior bounded review dispositions, not newly measured behavior.

The user's later steering is incorporated from immutable reports:

- Caller R6 `df028a8d03ed3aa657aa18c8c170ba4521df3769`, import `75051a1`, is
  HOLD for the large successful summary's one nonblocking pipe write. Actual
  partial stdout plus caller exit 1 cannot become success merely because a
  result file or all command files exist. R7 compact success-receipt repair
  is reported running; no R7 bytes, interface or final caller are cleared here.
- Transport full review `fc7a7bb02842f79936bfa989717e0a3facf71d8b`, import
  `cd97ba9`, is HOLD for new-directory namespace persistence and post-poll
  KILL deadline ordering. Its separate repair is reported running. No route,
  transport runtime or D4 acceptance is supplied here.

The payload treats caller stdout as opaque bytes. It neither expects the full
R5 JSON shape nor parses a proposed compact receipt. Therefore a later compact
stdout interface does not itself change this collector's capture logic. The
parent must bind the final exact caller bytes and its independent review,
then inspect the full retained result and all 53 command records through that
reviewed interface. A changed CLI, parent-v2 contract, environment, root,
capture need or writer-lifetime contract requires explicit reconciliation
and review. No future interface compatibility is presumed.

All caller, caller-review, payload-review, template-hash and prerequisite
slots remain null in the candidate. The final rendered payload hash is also
unbound. This HOLD cannot fill the payload CLEAR prerequisite. There is no
prior independent payload or runtime clearance to inherit.

## Full source and contract assessment

**Closed admission and identity, lines 8-15, 28-30, 202-235, 565-581.** The
first entry action sets and observes this process's NI15 before any child.
The fixed stdin flags reject another argv, site/bytecode/optimization mode.
`bind_config` rejects the unresolved template hash and caller/review slots
before root opening or private-path I/O. Exact type checks and recursive
`same_data` avoid bool/int equality weakening fixed bindings. The JSON schema
closes the configurable objects; its null branches describe pending data,
while executable admission is stricter. No configurable argv/path/env/code
or general command field exists.

The parent must strictly parse the replacement JSON, reject duplicate keys
and non-finite numbers, replace only the marked string token, restore the
original token and prove exact reviewed template bytes. The payload's initial
`json.loads` is not an independent renderer/substitution proof. Identity is
the unrendered template hash plus sorted compact ASCII CONFIG hash without
LF. The transport separately binds the full rendered payload SHA-256; there
is no self-reference. The unchanged null CONFIG's canonical hash is
`77926c546299639ea96304b8d2602680cbaa295a4760a7f1e7b29e862c76b50e`.
Hashes and booleans cannot independently establish a review or causal act.

**Exact root and completed evidence, lines 243-325, 367-375, 575-581.** Root
traversal uses nofollow directory-relative handles and requires current-owner
0700 on the same filesystem as `/private/tmp`. It creates or selects no retry
root. The only root is
`/Users/m1-dev/.local/state/openergy-verification/m1-ordinary-reviewed-flow-20261025-4b20357c1503`.
Bound leaves require regular type, current owner, 0600, one link, exact
size/hash and stable opened/linked metadata. The 17 pins, pin/vector files,
saved basis and eventual caller are bytes only during this authentication.

Before a new `inspected_ns`, the code checks the exact five remote originals,
full known file facts, facts/stdout agreement, empty stderr, helper0, both EOFs,
null first error, no kill, original deadline, NI15/product-count facts and
guard/root/vector agreement. The seven-file manifest stays
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26`;
saved audit stays `f7f9b10fc99cc2196d0fd88a5b93a35a39a9adc5`.
The two local records stay with the parent. Their existence in the manifest
does not mean the payload opened or independently inspected them.

Parent report `492f067` says it inspected all seven originals, guards,
667,038-byte original payload/reviews, exact 29-token route, actual helper0/
SSH0, EOF/error/kill/retry/root facts and original deadlines. It records two
passes, 14 opens and 11,010 content bytes. This review read that tracked report,
not those originals. Its new M1 inspected timestamp remains null. The payload
must still check the five remote files before new inspected then phase-start
readings. External parent causal inspection is not proven by our hashes.

**Clock and phase, lines 134-164, 428-440, 579-591, 633-639.** New readings
use integer `time.clock_gettime_ns(CLOCK_MONOTONIC)`, darwin clock6, exact
same-host/same-boot descriptor, with no fallback. Old approximately 0.x
readings are checked only in their original within-process arithmetic.
No retagging, offset or comparison with the new or local-transport clock exists.
The sole phase starts before parent-envelope/once-guard publication and spawn.
Collection ends at phase-end minus 2 seconds; reap ends at minus 1.5 seconds;
publication/console consume the original remainder. PAYLOAD-P2-1 is the
specific missing deadline gate; this is not overall deadline-path clearance.

**Caller interface, lines 415-420, 442-465.** There is one Popen site and one
retained direct SYSTEM handle. The fixed argv is `/usr/bin/python3 -I -S -B`,
the existing root's `metadata-caller.py.txt`, `source`, exact root, canonical
shared start and parent-record hash. Cwd is the root; stdin is DEVNULL;
stdout/stderr are raw pipes; shell is false; fds close; bufsize is zero.
All ten supplied environment keys/values equal next-operation. The wrapper
does not normalize its own global environment or enter private Python.
The six measured SYSTEM bootstrap names and their R5 normalization remain
caller review scope. The caller must still perform SYSTEM entry-template
before private version, Python lipo and closed50. No COPY broadening occurs.

**Capture, actual exit and failure retention, lines 436-552.** Reads are
nonblocking and limited to remaining combined 1 MiB plus one detection byte.
Overflow records one known dropped byte and unknown unread suffix, then stops;
there is no later drain. Both streams retain their complete bounded prefixes
in memory. Successful actual poll/wait return codes are stamped before file
publication. Actual exit0, real reap, both EOFs, no loss/timeout/kill/errors,
and an integer exit observation strictly before the original phase end are
all required. A file's complete flag cannot supply that actual observation.
Null, late, failed, partial and nonzero cases cannot be collection success.

The first collection error is latched before later publication. Pipe/selector
close errors remain separate available facts. Publication errors do not erase
an earlier collection cause. `HANDLE` survives until `os._exit`; no summary
poll, PID search, process-group, product, watchdog or remote SSH signal exists.
An unreaped caller fails the explicit line-594 gate and prevents all subsequent
file publication. No later cleanup or inferred exit repairs missing facts.

**Initial accounting and publication, lines 329-412, 578, 584-608.** A bounded
root-only walk rejects links, special files, foreign ownership/filesystems,
non-0700 directories, non-0600 or multiply-linked regular files, and any
bytes.json/bytes.lock. Walk bounds are 4 levels, 4096 entries and 255 name
bytes. Initial count precedes the first write. Every later publication performs
a fresh complete count, including caller-created files after actual caller
reap. The wrapper makes no file write during caller lifetime and does not
reuse a stale initial sum.

The 8 MiB file and 32 MiB total preserve product 16 MiB, cleanup 4 MiB and
accounting 64 KiB, leaving INITIAL_CAP 12,517,376. Each of the six fixed output
names uses exclusive nofollow 0600 creation, one raw write with short-write
rejection, file fsync, one raw close and root-directory fsync. No overwrite,
reset, deletion, rename, ledger init or reserved COPY writer is added.
Attempted/partial paths remain distinct from durable length/hash references.
A publication failure exits the publication sequence and preserves partials.
Close has no deferred Python payload and is not retried.

Checks do not prove the external no-concurrent-mutation condition. Stable
directory metadata is not continuous observation of regular-file contents.
The parent must establish exclusive temporal ownership and the reviewed
caller's writer-lifetime boundary. Actual direct reap alone does not prove
that an unknown other writer cannot survive. This review performs no process
query and makes no such proof claim.

**Small receipt and acceptance boundary, lines 555-562, 601-647.** The guard
checks the actual encoded byte length against 16,384 before the one nonblocking
raw console write. No raw caller stream or full caller result is embedded.
Partial/EAGAIN/alarm/expired output fails; there is no second attempt, Python
pending output buffer or deferred finalizer flush. All entry paths use
`os._exit`. The source guard proves the maximum attempted byte length; it
does not prove complete delivery or that every failure envelope will fit.

`source_collection_complete` requires full actual collection and all six
durable launcher publications. It is explicitly not 53 native comparisons,
caller-result parsing or artifact acceptance. Parent must retain/check exact
referenced originals, complete caller result and all 53 raw/status/comparison
records, then actual transport exits, EOF, losses/errors and deadline facts,
including independent D4 strictly before the local 135-second end. Neither
caller0 nor transport0 nor a receipt alone can accept the operation.

## Actual independent static checks

The reviewer-authored readers used only Git, AST as data, strict JSON, hashes
and patch reconstruction. They did not invoke any supplied checker/function.
[bindings.json](bindings.json) binds 136 input ref/path/blob/length/hash records
and all 28 source function ranges. [static-checks.json](static-checks.json)
and [findings.json](findings.json) retain the checks and bounded finding.

- All 11 candidate files and mode100644 bindings match candidate/import/current
  bytes; all ten SHA256SUMS entries match. All six candidate JSON files parse
  with unique keys and no non-finite values. Python 3.9 AST parsing succeeds.
- The 650-line add-file patch reconstructs all 647 source lines and equals
  independent unified-diff generation. Every declared function range/hash
  matches. No patch was applied to source.
- All 44 public input declarations in candidate bindings match exact Git bytes.
  All 17 public pins match; all ten modules match materialization/source-checks
  and the exact controller candidate. This is identity checking, not a new
  review of unchanged controller/helper logic.
- Independent expansion gives all 50 ordered unique recipe vectors, including
  direct Autoupdate/daemon objects. Canonical SDK data differs from preparation
  only in the ten header paths; other values remain exact. All 20 fixed initial
  input bindings, the completed manifest, parent-v2 clock/manifest shape and
  all ten supplied startup values agree.
- Embedded CONFIG equals its template and fixed schema fields. All 22 pending
  binding fields remain null. Actual rendered bytes and final caller remain
  unbound. The retained lower-bound JSON canonicalizes to 209,116 bytes.
- AST inventory finds one Popen, one direct KILL, two raw-write sites, two
  explicit poll sites, one wait and the expected file/root fsync sites. It
  confirms no renderer, arbitrary command, retry/drain, fallback clock/source,
  deletion, directory creation, buffered console or normal-exit flush path.

The public f7 audit and ce7 basis bridge retain their separate data scopes.
The saved basis stays 142,587 bytes, raw SHA-256
`1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`.
The complete50 commands projection remains canonical
`b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`;
the full wrapper remains canonical
`7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159`.
No hash was substituted, private output reopened or audit rerun.

## Proof limits and preserved states

No supplied code/AST/checker was executed, imported or compiled to executable
code. No tests, unit tests, mocks, native command/probe, process/port query,
remote/network operation, private file access, private interpreter, child agent,
build, signing, activation, prepare, COPY, controller or product ran. No
TypeScript changed. Only report/binding files in this folder are committed,
with hooks and signing disabled. Source candidate bytes remain unchanged.

This review proves no runtime behavior, hard OS preemption, actual clock rate
or cross-process equivalence, timely actual exit, complete pipe delivery,
signal delivery, reap/containment, durability under faults, whole-flow NI15,
parent causal act, native/UI, performance, signature, ABI, artifact, release
or integration acceptance. The full preparation, operator and receipt contracts
remain applicable. PAYLOAD-P2-1 and the separate caller/transport HOLD states
remain open; no runtime gate follows from a source hash.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause
UNKNOWN; whole-flow NI15 UNVERIFIED; earlier23/25 FAILED; all original failed
files/guards; missing original second post-bind, bundle-after, outcome/archive;
stripping warning; unknown full outer build duration/exit; and the closed
updater operation. The saved completed file pass supplies no new metadata,
copy or ordinary-flow acceptance.
