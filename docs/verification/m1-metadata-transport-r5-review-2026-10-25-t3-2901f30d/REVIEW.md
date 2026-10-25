# Independent source/data review of M1 transport R4

**SOURCE CLEAR for the exact transport source below. No open bounded P1/P2
finding. Both prior transport P2s are closed at source level. execution=false;
runtime_acceptance=false; future_execution_approval_created=false.**

Reviewed on 2026-10-25, America/Vancouver. This review owns only this new folder.
The full 620-line collector, all module statements and 25 functions, exact
two-hunk patch, full changed-function blocks, R4 README, contracts, schema,
templates, bindings and retained static-check data were reviewed. The prior
full review and both original briefs were read. [BRIEF.md](BRIEF.md) retains
those briefs and this review assignment. [bindings.json](bindings.json),
[function-bindings.json](function-bindings.json) and
[static-checks.json](static-checks.json) retain exact identities and checks.
The verdict applies to this transport source/data slice. It supplies no
execution approval, runtime result or clearance of another component.

## Exact source and import bindings

The original repair is `1c46c16b225924d6664d33c06382a3cd05581272`, imported as
`afb68e54dc5474844ea46252f897c32fc6120e9b`. The complete R4 folder has Git tree
`a6f672ea83a1ce01dafef947f152060986e0bac3` in both commits. All 15 working-copy
files also equal both immutable versions. The whole commit trees differ;
this is folder-scoped identity.

The reviewed [collector](../m1-metadata-transport-r4-2026-10-25-t3-53ab1856/transport-collector.py.txt)
is 27,834 bytes and 620 lines, SHA-256
`d312814e74c6f42e2a86d56648450dc145531df6292ea8c79ccf1768f34b14e4`.
The exact [patch](../m1-metadata-transport-r4-2026-10-25-t3-53ab1856/collector-from-88db0b5.patch)
is 2,237 bytes, SHA-256
`abb64be363ea1fffb7090cdd0cbd6c1802829e31479625b58fc2e19c4637c5c4`.
Its two hunks reconstruct the full reviewed collector from the immutable R2
blob at `88db0b54cf4d07092ea464392fe9b16ac7da2940` without applying a patch to
any candidate file. R2 is imported at
`87dcd6aec728c7ef01c1bf87c617c31afcb2ff6d`. Its folder tree is
`76c7f2a2fc99519fa6ab9440f8edbf2279f20c51`, and its 607-line collector is
27,181 bytes, SHA-256
`0aa1032bfbeb6eb13642c5c7cd4805c10a3b2c9ea897f1810ff67ec2f8a0b389`.

The prior full review is `fc7a7bb02842f79936bfa989717e0a3facf71d8b`, imported
as `cd97ba975867a571ce1cd2070083f07865d05eec`. Both copies of its folder have
tree `4cb2af94a90fd31e10f088beda4e5041b99ed215`. Its original HOLD and two P2s
remain historical records. This report assesses their R4 responses.

Exactly `Collector.settle` and `Collector.run` change. All other 23 full
function ranges and all bytes outside the two changed functions match R2.
All four full before/after Markdown blocks equal the source bytes. The
function bindings cover complete physical lines, including indentation and
the final newline. Thus their lengths/hashes can differ from an AST segment
convention that omits indentation or that newline.

## Prior P2-01: closed at source level

The prior finding was a missing fsync of the directory that contains the new
run directory. Fsync of the guard file and the new directory did not supply
that containing-directory namespace barrier.

R4 `Collector.run:454-466` keeps the exact parent FD returned by `leaf` open
through mkdir, the no-follow root open, and root owner/mode qualification.
It calls `admission_clock`, `os.fsync(parent)`, then `admission_clock` at
lines 462-464. The existing `finally` closes that parent FD at line 466.
No pathname reopen or replacement parent is introduced.

An fsync error or a deadline check failure leaves this work block through the
existing exception handler at lines 513-517. The handler latches failure
before final publication. There is no SSH handle yet. The path cannot reach
the guard write or SSH spawn. A late successful fsync return also fails the
postcheck. A parent-close failure likewise prevents forward admission.
If mkdir/open/qualification fails earlier, the same parent `finally` still
runs. Any available root FD remains for the existing bounded failure path;
there is no reset, retry, unlink or alternate run path.

The guard write remains exclusive at lines 488-490. `put:290-302` retains its
file fsync and root-directory fsync, with deadline checks before returning.
`guard_durable=True` follows that return. It is therefore reachable only after
the new parent barrier and the existing guard barriers succeed in source.
Snapshot creation and the sole SSH spawn at lines 493-509 come later.
The parent FD is needed through the namespace barrier, not through SSH.
The retained root and snapshot FDs preserve their existing lifetimes.

This closes the missing-barrier finding. It does not prove filesystem fault
behavior or power-loss durability on macOS. `guard_durable` remains the
collector's claim about completed barriers, subject to all parent checks.

## Prior P2-02: closed at source level

The prior finding allowed a new KILL call after an eligibility poll returned
late with `None`, because the only gate preceded that poll.

R4 `Collector.settle:409-427` stores the actual poll result at line 410 and
immediately samples the same local clock at line 411. A real code is stored
as `actual_exit`, with that exact sample as `exit_observed_ns`. Nonzero uses
the existing first-failure latch. It does not replace an earlier cap/deadline
failure. The later `observe_exit` skips a new poll when this actual code is
already retained, so it cannot replace this observation with a later sample.

For `None`, line 417 requires the retained sample to be strictly before both
the unchanged `settle_stop_ns` and `absolute`. Lines 418-419 reuse that sample
for the KILL record. No write, new clock sample or new budget intervenes in
this source branch. A late `None` takes lines 425-427, sets timeout, retains
the first failure, and skips the only KILL call. The later observation can
still record a real exit; it does not issue another signal.

The initial settle observation is followed by a deadline check at lines
398-401. If that observation itself returns late, the subsequent gate cannot
start the KILL branch. A late real eligibility result is still retained as an
actual result, then the final settle check marks timeout. A poll/clock error
escapes to the existing settle handler and cannot produce transport success.
The KILL call catches `OSError` without inventing an exit. Actual status stays
null until a real retained-handle poll/wait returns a code. An unexpected
settle error also latches failure before publication.

The only signal call remains `self.process.kill()` at line 421, on the same
direct local SSH Popen handle. Eligibility remains cap/deadline only. There
is no PID lookup, process group, remote/product signal, second signal call,
watcher or retry. The wait uses remaining time to the same stop at lines
428-443. Settle remains `min(start+132s, entry+2s)`; publication remains
`min(start+135s, entry+3s)`. There is no hidden fresh source allowance.

This closes the explicit late-return branch defect. It does not prove timely
OS signal delivery or preempt a stalled `poll`, clock, `kill`, `wait`, close,
filesystem operation or scheduler interval. The post-poll sample is an
eligibility observation, not an independently measured signal-delivery time.
This review does not qualify the eventual interpreter's subprocess internals.
The original brief and prior P2 already preserve those cooperative OS limits.

## Preserved source/data controls

The prior 16-token Windows route mismatch remains resolved at public source
level. The fixed vector has 29 tokens: executable `/usr/bin/ssh`, the exact
20-token public prefix, three ordered `-o` private option pairs, the selector,
and one final token `/usr/bin/python3 -I -S -B -`. The schema and template have
all 29 positions. The nine required flags, duplicate/missing-flag rejection,
phase choices, private-slot grammar and fixed guard hash remain unchanged.
The private route values and guard path stay null in the template.

The guard's supplied SHA-256 is
`9e0b9a5afb47b7755d1ebaf5f50edf45cf0a7372baaa2f83b83d9255011aed4c`;
its 1,563-byte length is declared by the brief. Neither was measured against
the private guard here. Before binding the whole profile path/hash, the
parent must qualify that saved guard and compare the full original argv,
including executable and final command. Public syntax and a shaped hash do
not establish provenance. No guard, private profile, known-hosts file, SSH
configuration or credential was read. No `ssh -G`, new global configuration
audit, arbitrary argv, shell, configuration edit or discovery is introduced.
Inherited effective SSH configuration is not newly verified.

Input caps remain profile 128KiB, payload 1MiB and collector 128KiB. The
no-follow directory walk, final owner check, regular owned 0400/0600 one-link
inputs, stable before/after metadata and bounded reads remain. The snapshot
is the same exclusive 0400 O_RDWR FD, fsynced, rewound and passed as stdin.
It does not reopen the source payload by pathname. These are cooperative
owner controls, without a hostile same-user immutability claim.

Output caps remain stdout 128KiB, stderr 64KiB, combined 192KiB, enforced
during reads with one overflow-detection byte. The capture function is exact
R2 source. A cap latches failure before the prefix write. Failure closes pipes
without a later drain. Actual written prefixes/hashes, observed and dropped
bytes, true EOF and unknown unread counts remain distinct. Process exit does
not imply EOF. Partial files remain. `fail` preserves the first failure;
publication failures have separate records/counts. A failed final write can
leave a partial or stale receipt, with nonzero/missing facts for the parent.
It does not create a replacement success receipt.

The local absolute envelope still starts in the parent before admission and
spawn. Work ends at start+130s, settle by start+132s and final publication by
start+135s. There is no extra local 120-second caller phase. The remote phase
remains separately bounded at 120 seconds. No cross-host clock comparison,
retry, reset, overwrite, unlink, drain retry or console output is added.

## Parent, prerequisite and integration limits

The full parent-observer contract remains byte-identical. D4 still requires
the direct parent to retain the actual collector handle, get the actual OS
exit, and immediately sample the same local clock before reading any receipt.
That sample must be strictly before the original start+135s. A late successful
wait/poll return rejects acceptance. Receipt flags, self elapsed time, a PID,
substitute files or a new watcher/budget do not discharge D4. This folder
supplies no executable observer and no actual collector-exit record.

Retained prerequisite `190e11f2477c0a68b8980a154dcf934ea0363882` reports local
CPython 3.15.0, `-I -S -B`, optimization 0, the Mach monotonic clock API, own
NI15 and actual probe exit 0. Its retained raw bytes and canonical descriptor
hash match the receipt. Those public records were read as data; the probe was
not rerun. They close only that recorded interpreter/version/API prerequisite.
They do not prove this collector ran, continuous NI15, D4, same future parent
and collector boot/runtime, timing, signal delivery, reap or durability.
The collector's floor remains trusted local macOS CPython >=3.10; the parent
intends actual 3.15. The exact future launcher and full shared Mach descriptor
still need binding. Remote SYSTEM Python is not qualified here.

Parent profile, materialized collector, SOURCE/COPY payload and R9 reserved
publication/accounting bindings remain pending. Local adapter files cannot
become R9 receipts by bypassing reservations. The separately reviewed payload
must preserve full caller evidence and emit a small outer envelope. The
retained incomplete caller JSON was independently parsed and canonicalized
as data: 209,116 bytes, SHA-256
`b0b8fc94cf7b1c44fc31128aee173e192798cb552726d8909c422c1170b50c13`.
It exceeds 131,072 bytes by 78,044 before omitted mandatory facts. Full caller
stdout still cannot pass directly through this transport cap. No cap change,
compression, raw drop, extra publisher or COPY implementation is supplied.

Caller R7 `c9a3...` awaits its own independent review per the supplied current
brief. SOURCE payload `fa72a0d` remains held on its own late-poll gate, with a
separate repair in progress. Older R4 plan text that says these reviews are
running is historical task context. This verdict does not clear either
component or substitute transport clearance for their pending parent bindings.

Preserve metadata FAILED_UNVERIFIED, Windows SSH255/no route and unknown
remote state. Preserve M1 FAILED_NOT_ACCEPTED25/25, controller1, port cause
UNKNOWN, whole-flow NI15/runtime UNVERIFIED and the closed updater operation.
No native, package, signing, activation, controller, performance, updater,
artifact, whole-product or release acceptance follows from this source review.

## Actual static checks and scope

Reviewer-authored standard-library processing completed 79 static checks.
They verify original/import folder identity, every R4 blob, declared input
bindings, all 14 R4 manifest hashes, 12 R2 manifest hashes, four prior-review
manifest hashes and four prerequisite manifest hashes. All 22 distinct JSON
files parse with unique keys and finite numbers. They also verify both full
ASTs, all 25 function ranges, the two changed functions and 23 byte-identical
functions, unchanged outside bytes, all four full function blocks, exact patch
reconstruction, route/template/flag/cap constants, new branch order and sample
reuse, parent-FD fsync ordering, retained prerequisite hashes and caller size.
The JSON schema was read as data; no schema engine or private example ran.

The first reviewer tool submission failed JavaScript parsing because a
Markdown fence occurred inside a template literal. It failed before the shell
ran or any file was written. Only the reviewer command delimiter was corrected.
That setup failure is retained in static-checks.json. All 79 resulting static
checks passed. No candidate was changed or retried. The data parser was local
CPython 3.9.6; it did not run this collector or qualify a future runtime.

No supplied code, AST, helper or repository module was compiled, executed or
imported. No tests, unit tests, mocks, private access, remote/native operation,
process/port query, child agent, build, signing or activation ran. Only the new
review report/bindings folder is committed, with hooks and signing disabled.
Static identity and source reasoning do not prove actual while-read behavior,
timing, OS exit, D4, signal/reap behavior, filesystem faults, power-loss
durability, private provenance, payload/R9 integration or execution acceptance.
