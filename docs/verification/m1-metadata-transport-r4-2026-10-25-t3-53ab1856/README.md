# Inert local M1 metadata transport R4 repair

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
Independent R5 review REQUIRED. Runtime UNVERIFIED.** This implementation does
not clear itself or create execution or runtime acceptance. It adds only this
new folder on parent `cd97ba975867a571ce1cd2070083f07865d05eec`.

The [full inert collector](transport-collector.py.txt) is 27,834 bytes and
620 lines. Its SHA-256 is
`d312814e74c6f42e2a86d56648450dc145531df6292ea8c79ccf1768f34b14e4`.
[bindings.json](bindings.json) binds the frozen inputs and final artifacts.
[BRIEF.md](BRIEF.md) retains the implementation assignment.

The source baseline is `88db0b54cf4d07092ea464392fe9b16ac7da2940`, imported
at `87dcd6aec728c7ef01c1bf87c617c31afcb2ff6d`. Its collector has 607 lines,
27,181 bytes and SHA-256
`0aa1032bfbeb6eb13642c5c7cd4805c10a3b2c9ea897f1810ff67ec2f8a0b389`.
The full independent review is `fc7a7bb02842f79936bfa989717e0a3facf71d8b`,
imported at the current parent. Its [report](../m1-metadata-transport-full-review-2026-10-25-t3-cb6cf25c/REVIEW.md)
remains HOLD with two P2 findings. Both original folders remain unchanged.
Folder tree identity was checked between each original and import; this is
not a claim that their whole commit trees match.

## Two bounded source repairs

P2-01 changes only `Collector.run`. After mkdir and no-follow open qualify the
new run directory, the code retains the existing containing-directory FD. It
checks the original work deadline, calls `os.fsync(parent)`, then checks that
deadline again before closing that FD. An error or late return reaches the
existing in-memory failure path before SSH spawn. The guard-file and run-root
fsyncs remain. `guard_durable` can become true only after this new barrier and
the existing guard publication barriers return in time. There is no reset,
retry, overwrite or alternate path. This is a namespace barrier in source;
fsync does not prove macOS power-loss durability.

P2-02 changes only `Collector.settle`. The KILL-eligibility poll now stores its
actual result and immediately samples the local monotonic clock. A real code
sets `actual_exit` and `exit_observed_ns`; a nonzero code uses the existing
first-failure latch. A None result permits KILL only if that same post-poll
sample is strictly before both the unchanged settle stop and absolute end.
A late None result records timeout/failure and skips KILL. The KILL record uses
that retained sample. The later actual poll and bounded wait remain unchanged.
Only the retained local SSH handle is eligible, and only on a cap or deadline.
Actual exit stays null until an actual poll/wait returns a code.

[FUNCTION-CHANGES.md](FUNCTION-CHANGES.md) retains all four complete before/after
blocks. Exactly two of 25 functions change. All other 23 function byte ranges,
module statements and class statements outside those two functions match the
baseline exactly. No capture, publication, clock, route, cap or input routine
is refactored.

## Exact reconstruction

[collector-from-88db0b5.patch](collector-from-88db0b5.patch) contains two hunks.
Its baseline is the full collector blob at the exact frozen 88db0b5 commit and
R2 path above. Its target is this folder's full collector, with patch labels
`a/transport-collector.py.txt` and `b/transport-collector.py.txt`. Static hunk
reconstruction and byte/hash equality establish the target without running it.
The patch has no dependence on the older f252 patch or the local working copy.

## Preserved interface and contracts

The profile schema/template, private bindings template, parent observer contract
and caller size contract are copied byte-for-byte from R2. Their R2 schema names
and historical observations are retained. They are data contracts, not fresh
runtime observations. [repair.contract.json](repair.contract.json) records only
the two R4 changes and their failure rules. [finite-plan.json](finite-plan.json)
keeps execution false and all future use bindings unresolved.

The exact prebound argv still has 29 tokens: `/usr/bin/ssh`, the 20-token fixed
public prefix, the three fixed private option pairs, the private selector and
the single final token `/usr/bin/python3 -I -S -B -`. The closed private grammar,
nine explicit collector flags and fixed guard SHA-256
`9e0b9a5afb47b7755d1ebaf5f50edf45cf0a7372baaa2f83b83d9255011aed4c`
are unchanged. The guard's declared length is 1,563 bytes. The parent must
qualify it and compare all 29 original tokens before binding the whole profile.
No private guard, SSH config, profile, route or credential was read here. No
`ssh -G` or new global config audit was added. Inherited effective SSH config
remains unverified. The private slots and future path/hash bindings remain null.

The future runtime remains trusted local macOS CPython >=3.10 with `-I -S -B`
and no optimization. The parent says it will actually select trusted 3.15;
this task does not qualify that launcher. The direct parent and collector need
the same running Mac, boot and full shared Mach clock descriptor. No local
clock is compared with a remote clock.

The unchanged absolute 135 seconds starts in the parent before admission and
spawn. Work ends at start+130s; settle is at most 2s and ends by start+132s;
final publication is at most 3s and ends by start+135s. Stage entry cannot renew
these ends. The remote phase remains 120 seconds. OS calls can stall after a
check. Neither this repair nor a post-poll sample gives hard OS preemption or
guaranteed timely signal delivery, wait/reap, publication or collector exit.

Input caps remain profile 128KiB, payload 1MiB and self 128KiB. Output caps
remain stdout 128KiB, stderr 64KiB and combined 192KiB. Reads enforce remaining
capacity plus one overflow byte. The no-follow owner/regular/one-link/stable
metadata checks, exclusive once guard and exclusive0400 O_RDWR retained and
rewound payload snapshot FD remain exact. No overwrite, reset, drain, retry,
watcher, PID search, group/product/remote signal or console output is added.
The first failure stays in memory before later writes. Publication errors stay
separate. Partial prefixes, EOF, dropped bytes, unknown unread bytes and null
actual exit retain their original rules.

## D4 and pending integration

[parent-observer.contract.json](parent-observer.contract.json) remains external
proof. The direct parent must retain the actual collector handle and its actual
OS exit, then sample the same local clock immediately before any receipt read.
That observation must be strictly before the original absolute end. Late or
missing observations reject acceptance. A receipt, prewrite flag, self elapsed
time, PID or substitute file cannot discharge D4. No observer is implemented
here and no actual D4 record exists for this candidate.

This local raw adapter is not the R9 reserved publisher/accounting writer.
Exact payload/R9 parent integration remains pending. The parent's caller R5
is held on the large one-shot pipe summary; its separate R7 repair does not
become cleared by this transport. Per the supplied brief, SOURCE payload
`fa72a0d`, imported at `dde69c7`, has a separate full review in progress. That
status is task context, not a result independently obtained here. A compact
outer receipt remains separate from full caller capture. The retained incomplete
caller shape is 209,116 bytes, 78,044 above the unchanged 131,072-byte stdout cap.
Direct full caller-summary passthrough cannot compose. No cap increase,
compression, raw dropping, extra writer, COPY publication or R9 bypass is added.

Old metadata FAILED_UNVERIFIED, Windows SSH255/noRoute and unknown remote state
remain. M1 FAILED_NOT_ACCEPTED25/25, controller1, port cause UNKNOWN,
whole-flow NI15 UNVERIFIED and the closed updater operation remain. This repair
does not clear the caller, payload, controller, native runtime or artifact flow.

## Actual static checks and limits

[static-checks.json](static-checks.json) records author-owned AST/JSON/hash/patch
checks. The parser was local CPython 3.9.6; it did not execute/import supplied
code or qualify any future runtime. Checks cover the exact baseline and review
bindings, all 25 full functions, unchanged closed controls, both new source
branches, copied contract identity, unique-key finite JSON, full function blocks,
exact patch reconstruction and final hashes. [SHA256SUMS](SHA256SUMS) covers all
other files in this folder.

No supplied code execution/import, tests/unit/mocks, private access, remote or
native operation, process/port query, credentials, build, signing or children
were used. Only the new folder is committed, with hooks and signing disabled.
Actual timing/caps, signals, reap, OS exit, D4, filesystem faults and durability
remain UNVERIFIED. Independent R5 is the next source review; it is not launched
by this task. No human gate or runtime acceptance is created.
