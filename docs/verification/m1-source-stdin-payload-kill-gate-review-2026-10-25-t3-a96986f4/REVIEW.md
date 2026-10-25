# Independent SOURCE payload KILL-gate repair review

2026-10-25, America/Vancouver. **SOURCE CLEAR for the exact payload source/data
below. PAYLOAD-P2-1 is CLOSED_SOURCE_ONLY. No new bounded P1/P2 found.
execution=false. Runtime and whole-flow NI15 remain UNVERIFIED.** This verdict
supplies no execution approval, runtime result, artifact acceptance or human
permission gate. Only this new review folder is owned and changed.

[BRIEF.md](BRIEF.md) retains this assignment and the original requirements,
with links to the immutable prior full report. I read the full 656-line repaired
source, all module statements and28 functions, the baseline source, all repair
files and contracts, and the complete prior report/bindings/findings/checks.
Both supplied static readers were data only; neither was run or imported.
Unchanged caller/controller/helper behavior remains prior scoped source
support. It is not a new full review or execution of those components.

## Exact identity

Candidate: `f78f2be0fe14e5e4cf81990156cb18a1abb5ff62`.
Import: `920eac7deae597b18963b88ccce7c0ba97910ce9`.
Review base: `81252214f7c391c465510193acf48115ec3d95db`.
Candidate parent: `0dad1c00d0a27f8266ed8e138f780a9d94656cdc`.
Import parent: `51533bd0b265e5941faa967da02351d4b1146437`.
Both candidate/import payload folders have tree
`154b0823017737b464dd57b7f8853b78278be448`. All 13 files match their candidate,
import and working-copy bytes and mode100644. This is folder identity;
it does not assert whole-commit tree equality.

The reviewed [source](../m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d/source-payload.py.txt)
is 37,775 bytes, 656 lines, Git blob
`b5425f6d3c3856933053c13dc093199b4eadc8c2`, SHA-256
`8d7844a20ce347b7be52c07a516b8d9136bdb5c067e450bea24eb3453441afa6`.
The [patch](../m1-source-stdin-payload-kill-gate-repair-2026-10-25-t3-da89534d/payload-from-fa72.patch)
is 1,910 bytes, three hunks, Git blob
`936e792a04d2aa7bdf85b4b9802c54a50388db95`, SHA-256
`bf302cd44d99267ca632004fea0d86c286d38c4f75a5d9ffce17e20971b81808`.
It equals independent unified-diff generation and reconstructs the full source
forward and backward in memory. No patch was applied to any candidate file.

Baseline: `fa72a0d865f12ae94b91f84400a31f51026c5b16`, import
`dde69c794ae6b547393c81a3de411b6c9af1d179`, source 37,122 bytes / 647 lines,
SHA-256 `dc208d02222c92418046139ecf622c797881af46b3edc1b8a891742b22f9fd9f`.
Prior full review: `4b5d756769cdc540abd7f07c92fead31b9530101`, import
`cac1b01c3ea211d70574daca5df13fc9c82d0a50`. Its HOLD remains the historical
baseline disposition. This report closes its one finding on the new bytes.

Only observe_exit428-435 and collect438-549 differ. Their new full-physical-line
SHA-256 values are respectively
`94a6fafecbef10b6509533a0d792cce1fc61b4fcb99d9183118da1916b83610c` and
`1c5c67d0cbb061573921e5291c1cde20eaaa5bc2f484e5e8df1c8b7a57b16d21`.
The 26 other function bodies and every byte outside the two changed functions
are exact. [bindings.json](bindings.json) records all 28 old/new ranges, byte
lengths/hashes, input Git blobs, modes, full commit diff hash and fixed inputs.
The outside-byte hash uses named placeholders for those two whole functions;
decorators and every other original byte remain in that comparison.

## PAYLOAD-P2-1 response: closed at source level

The prior501-508 path could poll late with None, then attempt KILL without a
post-return clock predicate. The subsequent wait check and normal-exit alarm
postcheck could reject success but could not prevent that new late signal.

New collect505-508 retains the actual poll result in code, calls observe_exit
immediately, then stores that same code and returned sample in the two new
eligibility fields. Both fields start null at456. observe_exit428-435 calls
the unchanged shared clock for None as well as real exits. There is no second
poll, substituted status, earlier snapshot or new time allowance in this path.

For None,509-513 requires the retained sample to be strictly below BOTH
reap_end and end. Equality is late. A late sample first sets timed_out and
unread_or_dropped_unknown, then raises TimeoutError before kill_attempted or
proc.kill. The existing519-521 handler retains kill_or_reap_error and latches
the failure without replacing an earlier cause. None cannot create actual
exit, an exit timestamp or reap. The603 gate then prohibits subsequent file
publication while reap is false.

An eligible None reuses that same sample for the branch predicate. Only then
does514 set kill_attempted and515 call the same retained direct SYSTEM handle.
The original wait at517 remains byte-exact:
`proc.wait(timeout=remaining(reap_end))`. Its real return is observed at518.
There is no extra poll, alternate target, group/PID/product/watchdog signal,
second caller, retry, drain, settlement extension or replacement phase.

For a first real returned code, observe_exit430-434 stores the code and real
reap before sampling. It stores and returns the same exit observation sample.
Thus a successful observation does not replace that exit timestamp with a
later eligibility timestamp. An already retained actual exit is never replaced.
The None branch cannot signal when this real code was returned. A late real
code stays an actual code; the enclosing deadline postcheck can then fail,
and an observation at/after phase end cannot satisfy completion541-561.
Missing EOF or another earlier cause still fails collection.

If poll raises, control cannot reach KILL. If observe_exit/clock raises, control
also cannot reach KILL. A real code already stored before that clock failure
is not rewritten to null; its timestamp can remain null. The eligibility
snapshot fields may remain null when the observation did not return. They do
not invent a successful poll or clock sample. A signal/wait/alarm error takes
the existing failure latch. A failed wait does not infer exit or reap from
kill_delivered. Errors can prevent a complete returned or durable status;
neither this review nor the parent may fill that missing evidence.

The ordinary work poll475-476 now also samples None. It does not use the
eligibility fields or grant signal authority. Its existing work-clock gate
remains. There is still one Popen, two poll sites, one wait and one direct
KILL. collection_complete is byte-exact. Raw prefixes, close errors and first
collection failure remain latched before main can start later publication.

This closes the specific missing post-poll source predicate. Scheduling can
still pause after any clock sample, including before KILL. poll, clock, signal,
wait, close and filesystem operations can stall. A sample is an eligibility
observation, not signal-delivery evidence. No hard OS preemption, interpreter
internals, timely reap or actual failure-path delivery is proven here.

## Unchanged source controls

Admission8-30,202-235,579-587 remains exact. Own-process NI15 precedes children.
The unresolved null template fails bind_config before private-path I/O. The
closed schema, embedded CONFIG and separate template agree. All 22 pending
slots remain null; execution is false. Canonical CONFIG SHA-256 remains
`77926c546299639ea96304b8d2602680cbaa295a4760a7f1e7b29e862c76b50e`.
Null schema alternatives document pending data, not executable admission.
The parent must strictly parse the replacement token, reject duplicates and
nonfinite values, restore the token to prove exact template bytes, and bind
the independently reviewed final caller and rendered payload. Hashes or CLEAR
strings alone do not prove inspection or authorize execution.

The root remains the existing M1 root ending4b20357c1503. No root creation,
retry root or qualification replay exists. open_root/read_leaf243-281 retain
nofollow directory traversal, owner/mode/device and one-link exact raw binding
checks. inspect_remote284-318 still checks exactly five remote files, full
saved facts, raw agreement, empty stderr, helper0, EOF, errors/kills, original
deadline arithmetic and guard/root/vector before new clock readings588-589.
All seven declarations and the completed manifest remain exact:
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26`.
Saved audit remains f7f9b10; local originals stay the parent's duty. Old
relative clock values are not retagged or compared with new readings.

All 17 pins and ten module byte bindings match tracked public inputs. All 50
ordered unique native vectors equal an independent expansion of the metadata
recipe, including direct Autoupdate and daemon objects. The 20 fixed initial
input bindings are unchanged. The saved basis remains 142,587 bytes, raw SHA-256
`1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`.
These are byte/identity checks, not new native, controller or helper acceptance.

The single fixed launch444-466 remains `/usr/bin/python3 -I -S -B`, fixed
caller, source, exact root, original start and parent hash; cwd is root.
The exact ten environment values agree with next-operation after its documented
EVIDENCE substitution. No arbitrary CLI/environment or private interpreter
entry exists in this wrapper. SYSTEM entry-template and private/native sequence
ordering remain the final caller's independent source-review duties.

The clock134-164 and phase442,588-590 remain darwin CLOCK_MONOTONIC6 integer ns,
same-host/same-boot, no fallback or offset. Parent/guard publication, launch,
capture, kill/reap, close, accounting, files and console share start+120s.
Work ends2s before end; reap1.5s before end. The0.5s settlement and1.5s final
publication are deductions from that same phase, not added allowances.

Capture474-549 retains the combined1MiB cap during reads, one detection byte,
full bounded prefixes, actual exit/null, EOF/loss uncertainty and first error.
No drain follows overflow. The new status fields do not alter stdout parsing:
stdout remains opaque. Main603 prevents later file writes without actual reap.
publication378-412 re-counts all root files before each new file, including
caller-created files after reap. Each fixed output uses exclusive nofollow0600,
one raw write, short-count rejection, file fsync and root-directory fsync.
Per-file 8MiB and total 32MiB preserve 16MiB product, 4MiB cleanup and 64KiB
accounting, leaving INITIAL_CAP 12,517,376. Root count bounds remain depth 4,
4096 entries and 255 name bytes. No ledger, COPY, rewrite, rename or deletion
exists. Partial attempts remain distinct from durable references. Publication
errors remain separate from the first collection cause.

The parent must establish exclusive temporal ownership and the final caller's
writer lifetime. A count or direct reap does not independently establish that
no unrelated writer survives. This reviewer made no process query. The single
console path564-571 checks the actual encoded <=16KiB length and makes one
nonblocking raw write inside the original end. The module exits through
os._exit656. No shutdown-buffer retry exists. Delivery, every possible failure
envelope fitting, and filesystem fault durability remain unproven.

## Current dependencies and unresolved obligations

The current transport verdict supersedes the old report's transport HOLD:
repair1c46c16 is SOURCE CLEAR under
`a7e22725cbbf07b5e6ac7e7aa7e7fb0d3dff93b4`, import8125221. Its source-only
closure of the containing-directory fsync and late-poll issues supplies no
private route provenance, actual transport result or D4. This review read
that retained report; it does not independently re-review the collector.

Caller R7's compact receipt defect is CLOSED_SOURCE_ONLY under
`8326f49945824c71e1b58d421f107a3cfafff3da`. That same report is HOLD on
R8-P2-1 in the separate caller collector. R9 is reported running; no final R9
bytes or review are accepted here. Final caller hash/review and rendered payload
bindings stay null. This payload's opaque capture does not depend on the
compact schema, and its repair cannot close the caller's own late-poll path.
The local old-R7 packet remains disabled; SOURCE CLEAR here does not activate it.

Parent inspection492f067, profile75ef628 and coordinator190e11f remain bounded
saved-data/profile/local-API prerequisites only. Their tracked records were
read as data. No private originals, profile, guard, interpreter or endpoint
were opened, and none of those acts was repeated. Future remote five-file
checks and new inspected/start readings still occur in the required order.

Parent must later retain and verify complete exact caller stdout/stderr/OS
status, any compact receipt, full referenced result, all 53 SOURCE native/aux
raw/status/comparison records, all lengths/hashes and original success/failure
predicates. It must also check actual transport exits/EOF/loss/error/timing and
independent D4 strictly before the original local135s end. A receipt, file,
caller0, transport0 or50 matching rows alone cannot accept the operation.
There is no COPY, prepare, controller, product, artifact receipt constructor,
SSH route or general materializer implementation or grant in this slice.

## Actual static checks and limits

[static-checks.json](static-checks.json) records 312 successful final independent
static checks, 237 distinct Git input bindings, all 13 candidate/import files,
all 11 baseline files and all 5 prior-review files, their manifests, all 7 new
JSON documents, Python3.9 AST data, all 28 function bindings, the exact patch,
schema/null/config, pins, vectors, environment and source ordering checks.
All 136 prior input declarations, 26 repair declarations and 44 inherited
declarations match their Git byte/hash/blob bindings where declared.
Reading or parsing the supplied checker is not running its checks.

The complete candidate diff whitespace check returns2 for literal patch9/10,
the required single-space blank context prefixes. All other candidate files
return0. Both raw results are retained; the complete check is not marked0.
No behavioral issue is assigned to those necessary patch bytes.

Two reviewer setup errors remain recorded: the first static environment
comparison omitted the documented EVIDENCE substitution, then one tool
submission failed JavaScript interpolation before starting the shell. Neither
wrote a file or ran supplied code. Only the reviewer comparison/submission
was corrected. Candidate data and all unfavorable historical states remain.

No supplied source, AST or checker was executed, imported or compiled into
executable code. No tests/unit tests/mocks, private access, remote/native
operation, process/port query, children, build, signing or activation ran.
No TypeScript changed. Only this new report/bindings folder is committed,
with hooks and signing disabled. All candidate files remain unchanged.

This proves no runtime behavior, actual clock equivalence/rate, deadline,
exit/EOF, signal delivery, reap/containment, fault durability, private startup,
parent causal act, continuous NI15, native/UI, ABI, signature, artifact,
performance, integration, product or release acceptance. Preserve strict SDK
FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/controller1
FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause UNKNOWN;
whole-flow NI15/runtime UNVERIFIED; earlier23/25 FAILED; all old files/guards;
missing original second post-bind, bundle-after, outcome/archive; stripping
warning; unknown full outer build duration/exit; Windows SSH255/no route and
unknown remote state; and the closed updater operation. Source closure changes
none of those results.
