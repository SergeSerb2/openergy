# Independent full source/data review of M1 transport R2

**HOLD — two bounded P2 findings. No SOURCE CLEAR. execution=false. Runtime UNVERIFIED.**

Reviewed on 2026-10-25, America/Vancouver. This review owns only this new folder.
The [original brief](BRIEF.md), [candidate byte bindings](bindings.json), and
[actual static checks](static-checks.json) are retained. No candidate or other
existing file was edited. No implementation or runtime approval is supplied.

The review covers original candidate `88db0b54cf4d07092ea464392fe9b16ac7da2940`
as imported at `87dcd6aec728c7ef01c1bf87c617c31afcb2ff6d`. Its entire R2 folder
has the same Git tree object, `76c7f2a2fc99519fa6ab9440f8edbf2279f20c51`,
in both commits. The whole commit trees differ; this is folder-scoped identity.
The complete 607-line, 27,181-byte
[collector](../m1-metadata-transport-r2-2026-10-25-t3-5256c911/transport-collector.py.txt)
has SHA-256 `0aa1032bfbeb6eb13642c5c7cd4805c10a3b2c9ea897f1810ff67ec2f8a0b389`.

All final source was read, including all 25 functions and module statements.
The full 585-line baseline, all R2 README/contracts/schema/templates/bindings/
function blocks/static checks, and the exact six-hunk f252 patch were read.
The baseline is `f2520dfb9b07c6ab54c85f438f2ed0ecb5c05c37`, imported at
`22b81eb3438dcb4b56ea193ceb85db165a105b97`. Its collector is 26,005 bytes,
SHA-256 `5c27140ebe7c7ab575168187f08d4a28a4f52e0e53b789e75b4187abf5e2eb4e`.
The review is not limited to the R2 delta.

## P2-01 — Persist the new run-directory entry before declaring the once guard durable

Locations in the final collector: `Collector.run:444-453` and `:475-477`;
supporting `Collector.put:290-302`; SSH spawn at `:492-496`.

The collector creates the previously absent run directory with mkdir, opens it,
then closes its containing-directory FD without an fsync of that parent.
The guard path later fsyncs the guard file and the new run directory itself,
then sets `guard_durable=True` and permits SSH spawn. All five fsync call sites
in the complete source operate on `fd`, `payload_fd`, or `self.root`.
None syncs the directory that contains the newly created run directory.

This leaves the new directory's name without an explicit parent namespace
persistence barrier. After a crash or power loss, the code cannot support its
claimed durable, discoverable once guard at the assigned run path before SSH
was started. Syncing entries inside the new directory does not supply the
missing parent-directory sync. The parent cannot have persisted this particular
new entry before the collector creates it. The documented once-only path
assignment and no-retry rule remain necessary, but do not fill this source gap.

**Minimal fix:** while retaining the already opened containing-directory FD,
fsync that parent after the new run-directory entry is created and qualified,
with work-deadline checks before and after the call. Treat any failure or late
return as failure before SSH. Keep the existing guard-file and run-directory
fsyncs. Set the durable claim only after all required barriers succeed.
Do not add a reset, retry, alternate run directory, or overwrite.

This is a static missing-barrier finding. It does not claim that a crash was
observed, that data was lost, or that fsync alone proves power-loss durability
on the eventual macOS filesystem. Actual durability remains UNVERIFIED.
This issue is inherited from the baseline and remains in R2.

## P2-02 — Recheck the deadline after the KILL-eligibility poll returns

Location in the final collector: `Collector.settle:409-415`.
Supporting fixed bounds: `:219-222` and `:397`.

The KILL condition checks `time.monotonic_ns() < self.settle_stop_ns` before
calling `self.process.poll()`. If that actual OS poll or its scheduling returns
late with None, the code enters the body, takes a new timestamp for the kill
record, and calls kill without checking that timestamp against the stage end.
An early cap uses stage-entry+2 seconds; a slow poll can therefore cause a new
KILL call after that two-second limit. A sufficiently late return can also
cause that new call after the original start+135-second absolute bound.

The OS call already in progress cannot be preempted by this source. The
avoidable defect is the later signal operation after the poll returns.
Recording a late timestamp or rejecting acceptance through D4 does not prevent
that late operation. The existing prose says a stall that returns late fails
closed; this branch does not enforce that rule.

**Minimal fix:** retain the eligibility poll's actual returned code and take
the immediate local monotonic sample after it. If it returned a code, retain
that real exit and observation. If it returned None, issue KILL only when the
post-poll sample is still strictly before the unchanged settle deadline and
absolute deadline. A late sample must record timeout/failure and skip KILL.
Preserve the exact retained SSH handle, cap/deadline-only eligibility, actual
status/null rules, and bounded wait. Do not add PID lookup, process groups,
another signal target, a watcher, retry, or a fresh budget.

This does not claim hard OS preemption or a reproduced scheduling race.
A subsequent kill/wait can itself stall; those limits stay explicit.
This issue is in the unchanged baseline `Collector.settle` function.

## Response to the prior finding and the other R2 changes

The prior Windows16-token OPTIONS sequence could not represent the supplied
M1 route. The final collector fixes that source-level mismatch. Its exact
public 20-token prefix, three ordered private option pairs, selector, and final
command occupy all 29 positions. Executable token0 is exactly `/usr/bin/ssh`;
tokens21/23/25 are `-o`; option values22/24/26 must use the fixed names;
token27 has the closed selector grammar; token28 is exactly the single string
`/usr/bin/python3 -I -S -B -`. There is one direct Popen call with shell=False,
the fixed executable/environment, and no added argv options.

The public private-slot checks reject whitespace, shell metacharacters,
leading selector options, expansions, and noncanonical known-hosts paths.
Host/alias fields have the stated literal ASCII grammar. The JSON schema
and template match the fixed slots; decoded regex strings were inspected as
data. No schema engine or private example was run. Four route slots and the
guard path remain null. The null template cannot pass collector admission.
The source-guard hash is fixed to the brief's declared
`9e0b9a5afb47b7755d1ebaf5f50edf45cf0a7372baaa2f83b83d9255011aed4c`
and its 1,563-byte length is declared, not measured in this review.

That resolves the public source shape only. The parent must qualify the actual
saved guard and compare every original argv token with the whole profile before
binding the profile path and SHA-256. Neither the schema nor a hash-shaped field
proves that provenance. No private guard, profile, endpoint, known-hosts file,
SSH config, or credential was accessed. Inherited effective SSH configuration
is not newly verified. The removed broad global-config audit gate is not
reintroduced. No ssh -G or config inspection belongs to this review.

The version-floor change is consistent with this brief's trusted local
CPython>=3.10 clock contract. It broadens the baseline's exact3.15 admission
gate. The parent still says it will actually select trusted3.15. The full clock
descriptor, same running Mac and boot, exact launcher, and -I -S -B/no-optimize
startup remain parent bindings. The recorded Python version is a fact to assess,
not proof of toolchain trust. This review's data parser was local system
CPython3.9.6; it never ran the collector and cannot qualify its future runtime
or the remote SYSTEM Python.

The final containing-directory owner check at `directory:109-110` adds the
stated current-owner requirement. System ancestors are still traversed through
O_NOFOLLOW directory FDs. Leaf inputs still require current-owner regular,
one-link0400/0600 files and matching metadata through bounded reads.
This is a cooperative owner boundary. It does not protect against a hostile
same-user process, and the parent must protect the input/run paths.

The snapshot change keeps the same exclusive O_RDWR FD, writes the already
bound payload bytes, fsyncs, rewinds, and passes that FD as SSH stdin.
It removes the close/reopen pathname interval. File mode remains0400; the
retained creating descriptor is read/write. The review does not infer
immutability against concurrent same-user writes.

Exactly four functions change: clock_descriptor, directory, profile_vector,
and Collector.run. All other 21 full function byte ranges match the baseline.
All eight full before/after blocks in FUNCTION-CHANGES.md match their sources.
Unchanged code received full review; P2-02 is in one of those functions.

## Full source/data assessment and D4

The nine-flag parser is closed, rejects duplicates/missing values, limits phases
to metadata-source/metadata-copy, and accepts only shaped hashes and start text.
Admission binds self/profile/payload through bounded reads and before/after
metadata equality. Input read requests are at most64KiB and remaining+one byte;
caps are profile128KiB, payload1MiB, and self128KiB. The guard and payload
snapshot precede the only SSH spawn. Both findings above still block source
clearance of that full arrangement.

Capture uses nonblocking raw pipes and per-stream remaining+one-byte reads.
It keeps at most stdout128KiB, stderr64KiB, combined192KiB. An observed cap is
recorded before the prefix write. Failures close unread pipes without a drain.
The capture/write paths retain known observed/written/drop counts and prefix
hashes; unread counts remain null without real EOF. No EOF is inferred from
process exit. Partial files remain. Publication failures are separate from the
latched first failure. Actual SSH status comes from the retained handle;
poll/wait observations take local clock samples. KILL is only on that handle;
no group, remote, product, or discovery signal path exists. The P2-02 deadline
ordering remains a specific exception that needs repair.

Work, settle, and final limits derive from the original parent start:
130,132,135 seconds. Stage limits also use entry+2 and entry+3 where shorter.
There is no second local120-second caller budget and no comparison of local
and remote clocks. The source has no retry, drain retry, guard reset, unlink,
host discovery, alternate command, configuration edit, or console-print path.
The final receipt may be partial or stale after publication failure; nonzero
collector status and any side record must remain visible.

The complete parent-observer.contract was read. Its seven finite observer
steps and retained record shape are unchanged from the baseline. D4 requires
the direct parent to record S and A=S+135s before spawn, retain the actual
collector Popen handle, perform the finite actual wait/poll, and take the
immediate post-return local sample T before reading any receipt. T must be
strictly less than A. A late successful OS return is rejected. Self elapsed
or receipt_prewrite fields cannot discharge D4. A later process needs an
already retained actual OS-exit record and its own same-clock sample before A.
No PID/file-derived exit, substitute file, infinite watcher, or new budget
can establish this fact. This data contract supplies no implemented observer
and there is no actual D4 record for this collector.

The separate SOURCE/COPY SYSTEM payload and existing R9 reserved publisher/
accounting must be bound before use. This raw local adapter is neither that
writer nor artifact acceptance. The reduced retained SOURCE shape was fully
parsed as JSON and canonicalized as data: 209,116 bytes, SHA-256
`b0b8fc94cf7b1c44fc31128aee173e192798cb552726d8909c422c1170b50c13`.
It exceeds the unchanged131,072-byte stdout cap by78,044 bytes before the
omitted mandatory process/timing facts. The cited caller/vector bytes match
the retained size report; this review does not independently clear that caller.
Direct caller-stdout passthrough cannot compose. The separately reviewed SOURCE
payload must capture full caller stdout as initial M1 evidence and emit a small
bound receipt within the original remote120-second phase. SOURCE is separate
implementation/review work; COPY integration remains pending. No raised cap,
compression, raw drop, extra writer, or R9 bypass is supplied or approved.

## Actual static checks and proof limits

The retained checks use reviewer-authored standard-library data processing.
No supplied source, AST, helper, or repository module was executed or imported.
AST parsing and literal extraction are data operations. Checks established:

- Final and baseline source hashes/lengths and exact Git blob bindings.
- Identical R2 folder trees at original/import commits.
- Full inventories of all25 functions, four changed and21 byte-identical.
- Eight complete function blocks equal to their exact source bytes.
- All13 baseline/R2 JSON files parse with unique keys; no NaN/Infinity.
- All19 listed candidate/baseline SHA256SUMS entries match.
- All six patch hunks reconstruct the complete final source from exact f252 bytes.
- Fixed argv/template positions, nine unique flags, input/stream caps, and
  unchanged observer steps/record shape.
- Nine historical public reference byte/hash bindings.
- Canonical retained lower-bound bytes/hash, cap excess, and cited caller/vector hashes.
- One Popen call site, one exact-handle KILL call site, and all five fsync arguments.

The first reviewer block-comparison check failed because it expected an
additional newline that the source segments did not contain. A direct
diagnostic showed all eight blocks already matched. Only that reviewer
comparison was corrected; the failure and correction are retained in
static-checks.json. No candidate change, runtime retry, or result replacement
occurred.

These checks do not prove timing, bounded behavior in the OS, hard preemption,
actual signal delivery, reap, process closure, actual collector CLI exit, D4,
filesystem fault behavior, or power-loss durability. They do not qualify the
actual trusted local3.15 launcher, private route/guard equality, inherited
effective SSH config, remote SYSTEM Python, pinned payloads, complete payload
raw evidence, R9 publication/accounting, or SOURCE/COPY/artifact acceptance.
No hash or syntax result grants runtime or execution approval.

The full old public collector provenance and observations, the local-reader
deadline CLI summary/results, and the parent receipt contract were read.
The old elapsed-derived bound remains conditional historical evidence, not a
new independent exit timestamp or descriptor. Earlier local reader success
does not qualify these collector bytes; its terminal scheduling race was not
reproduced and its OS-stall/disk/native proof gaps remain. No private source
index target or private retained evidence was opened.

Preserve old metadata FAILED_UNVERIFIED, SSH255/noRoute, and unknown remote
state. Preserve M1 FAILED_NOT_ACCEPTED25/25, controller1, port cause UNKNOWN,
whole-flow NI15 UNVERIFIED, and the closed updater operation. No caller R5,
ordinary controller, native, package, signing, activation, performance, update,
or release clearance is created by this review.

No tests/unit/mocks, candidate execution/import, private access, remote/native
operation, process/port query, child agent, signing, or activation ran.
Only this new review folder is committed, with hooks and signing disabled.
