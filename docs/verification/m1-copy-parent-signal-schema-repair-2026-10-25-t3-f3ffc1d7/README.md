# M1 COPY parent signal and receipt-schema repair

2026-10-25, America/Vancouver. **IMPLEMENTED_UNREVIEWED. execution=false.**
This is an inert SOURCE/DATA repair of COPY-P2-1 and COPY-P2-2. An independent
fresh R4 review of these exact new bytes is required before use. No source
CLEAR, execution authority, actual receipt, prepared tuple, or runtime success
is supplied. No new human approval gate is added.

This folder alone is owned at parent
`7e5de706e7457de5f4880cdef180b27536dc0d5e`. Every prior tracked Git blob/mode,
index entry, and actual working byte/mode is bound in the opening inventory
and compared again before the folder-only commit. The commit disables hooks
and signing. Its actual identity and clean status are reported outside the
packet to avoid a self-reference.

The exact input candidate is `85c668f7e63c07e1f9d3cf9e76af05f21898f80d`,
import `cd4fae4155e50e99d033fc78553ad50bbfa7d164`. Its full 120613-byte source
SHA256 is `cfe3be7a67d22160b977a97941f7fa6ed44be322ab6822dd064c66bb93555b47`.
The complete full-review report at
`f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5`, import `931d2c3`, is 21224 bytes,
SHA256 `33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f`.
It remains HOLD_SOURCE, 0 P1 / 2 P2, on those old bytes. Its full 48-function
review found no other concrete candidate P1/P2. This repair does not rewrite
that historical verdict or supply a new independent full review.

## Changes and exact deliverables

[copy-parent.py.txt](copy-parent.py.txt) is the complete new inert source.
[from-85c668f-source.patch](from-85c668f-source.patch) changes only collect;
[from-85c668f-schema.patch](from-85c668f-schema.patch) changes only the full
receipt schema. [from-85c668f.patch](from-85c668f.patch) combines both exact
patches. Every hunk, old/context row, count, and resulting byte was checked
by an author-owned data reader in memory. No supplied file was patched.
[DELIVERY.json](DELIVERY.json) gives their exact lengths and hashes. The
historical prior-from-cleared-source-launcher.patch is also retained byte
exact. It is the old SOURCE-to-COPY patch, not the new repair delta. Its
literal blank context rows retain the complete whitespace exit2 distinction.

COPY-P2-1: collect now calls `os.kill(proc.pid, signal.SIGKILL)` once at the
existing cap/deadline branch. It uses only the retained direct Popen child's
PID. The final actual None poll, immediate shared-clock sample, and both
strict unchanged end checks still precede that call. Between that sample
and the call are only existing local gates and status assignments. There is
no new poll, PID lookup, OS query, group target, or signal wrapper.

The parent's `kill_delivered` key is replaced by `kill_syscall_returned` in
its two collected statuses and all four closed receipt definitions. False
is the initial value. True is assigned only after os.kill returns without
exception. That return proves neither actual delivery nor termination nor
reap. Only the existing actual poll/wait code can set actual_exit/reaped,
before its next clock read. ESRCH and all other exceptions retain the error
and any unknown actual exit; they never synthesize zero, a signal exit, or
reap. An unreaped handle still blocks publication. There is no new cleanup
pass or extra attempt. Wait keeps the actual original remainder.

The signal operation is the trusted POSIX os primitive, with no Popen.kill /
send_signal path. The public library evidence is source text only. It does
not qualify the target's installed interpreter, audit hooks, kernel result,
clock rate, OS stall, or scheduling delay after the final sample. No direct
signal call, poll/wait experiment, or runtime receipt was made in this task.

COPY-P2-2: actual_exit and kill_eligibility_exit now have signed integer/null
domains in both base helper/caller objects and their two success-branch
repetitions: eight domain sites in total. Those four definitions agree. All
other integer minima and maxima, including PIDs, clocks, bytes, counts and
lengths, remain exact. The success branch explicitly requires actual_exit0
and both signal flags false for helper and caller. This strengthens the
success-only schema to match the unchanged source admission. Existing
authentication and 52-record success exits also remain const0. Negative
signal exits belong only to failed evidence; they are never changed to fit
the schema. No synthetic receipt, signal case, or mock was created.

[FUNCTIONS.md](FUNCTIONS.md) contains full before/after collect source.
[function-inventory.json](function-inventory.json) binds all 48 functions,
including nested functions, and the class. The other 47 functions and the
class are byte exact. [GLOBAL-SEGMENTS.md](GLOBAL-SEGMENTS.md) and
[global-segment-inventory.json](global-segment-inventory.json) retain complete
before/after non-declaration segments; all are unchanged. All decoded literal
globals also remain exact.

The full receipt, CONFIG, stdin and console schemas are included. CONFIG's
template, both reviews, all 14 parent-duty values, template hash, and all
nine prepared slots stay null where originally null. execution remains
false. No bound future render or prepared tuple exists. The regenerated
[template-proof.json](template-proof.json) binds the sole CONFIG token,
prefix/suffix, canonical CONFIG and an explicitly defined parse-only AST
normalization. It proves only the current unbound template. Actual future
token replacement/restoration, full payload size and actual parent acts stay
pending. The finite plan records zero target invocations.

## Frozen interfaces and limits

The new flag is parent-only. copy_record_complete and inspect_copy_result are
unchanged. The frozen R9 caller's actual 52 record key sets still contain
`kill_delivered`; their exact success checks still require it false. The
full raw 52/208 comparison gate is unchanged. No global rename or permissive
key-set rule was introduced. The exact R9 caller remains 46908 bytes, SHA256
`01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`.

inspect_new_prepared passes the actual primary and helper to the generic
collection_complete predicate. It has no closed prepare-primary key set and
does not read kill_delivered. Its receipt-primary equality, exact source/
review identity, actual0/reap/EOF/error gates, canonical tuple and full raw
bindings are unchanged. Thus a future prepare parent field rename needs no
COPY relaxation for that key alone. This is a bounded source compatibility
observation, not acceptance of any held prepare source or future interface.

Prepare R4 `c85725b` / import `a302d4b` is HOLD_SOURCE with PREPARE-P2-3 on
`88e4444b8e5604fe9bfde7e768ea31d138ab64be`. Its original fresh-launch and
named-cache cases closed SOURCE_ONLY, but ROOT's common package/import-layout
case remains held. The separate R5 repair, including its closed260/264 layout
and direct-signal branch, was reported running in the task brief. It is not
bound, reviewed, imported, or labelled CLEAR here. Future source/review and
all nine actual prepared slots remain null. The original prepare.py pin is
still `16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`.

The matching Popen.kill pattern is also present in frozen dependencies:

- R9 metadata-caller.py.txt: collect 474-485, with kill_delivered at 485 and
  output at 521. Its finally path polls, clocks, then calls proc.kill at484.
- SOURCE source-payload.py.txt: collect 504-516, proc.kill at515 and
  kill_delivered at516.
- Phase transport-collector.py.txt: settle 412-430, process.kill at427 and
  its returned flag at428. It catches OSError at429-430.

[frozen-signal-patterns.json](frozen-signal-patterns.json) binds their exact
paths, whole-source hashes, enclosing declarations and complete local context.
Those failure paths are not repaired or newly cleared. Earlier narrower
source verdicts are historical, not a whole-dependency verdict from this
task. All old successful SOURCE and zero-exit records stay unchanged; none
was rerun. The COPY happy-path success gate continues to require no signal.

All 17 pins, ten modules, 50 vectors, 212 SOURCE rows, original source/
artifact/library bindings, fixed SYSTEM bootstrap and flat cache gates remain
exact. The private vector stays PRIVATE,-B,caller,copy,ROOT,original-start,
stdin-hash. Startup still has exactly ten keys; COPY stdin has seven and its
entry authentication has nineteen. COPY120/helper10 and local135=130+2+3
keep their original starts and ends. There is no product/watchdog/group
signal authority, renewed allowance, retry, fallback, recopy, reset, refund,
INITIAL writer, ledger creation, or changed target writer. All target output
still uses unchanged common.write_bytes with critical=False and the existing
ledger. Failed authentication still prevents private entry.

The external pending controller boundary remains blocked. Full retained
helper/caller/raw/status/comparison/receipt evidence and actual own SSH0,
collector0, complete EOF, timely D4 before receipt reads, whole-flow NI15,
signature/entitlement assessment and operator acts remain mandatory. Neither
schema conformance nor future source review can perform those acts.

## Static evidence and preservation

Only public Git/text/strict JSON/hash/AST-data tools were used. The new
author-owned static-data readers ran only for data construction/comparison.
No supplied source, checker, caller, helper, collector or preparer was run,
imported, evaluated, or compiled into executable code. No private/state read,
network, remote, SSH, native/runtime/GUI action, test/mock, build, signing,
delegated child, PR/push, or monitor occurred. No TypeScript changed. The
explicit execution=false scope excludes end-to-end execution.

The original four author reader errors, first complete/final author reports,
and the prior reviewer's corrected AST-shape error are retained byte exact as
labelled historical data. They are not new independent checks. The new
author's incorrect guessed R4 path and correction are also retained. No
reader error is hidden or turned into a target retry.

The original full Git whitespace result remains exit2 for literal single-space
blank context rows in the exact patch; its other-files result remains exit0.
The new folder's actual complete and patch-excluded transcripts are retained
separately in whitespace-observations.json. Patch context bytes are never
stripped to make the full result zero. The patch-excluded result is not
reported as the complete result. Manifest/SHA256SUMS bind all packet payloads
without self-hash claims.

This packet proves no signal delivery/reap, startup/cache selection, clock
rate, OS-preemption bound, hostile same-user immutability, crash/power
durability, pipe delivery, continuous NI15, native ABI/signature/UI,
performance, release, or product acceptance. Preserve strict SDK
FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/controller1
FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431 EADDRINUSE cause
UNKNOWN; whole-flow NI15/runtime UNVERIFIED; missing original second
post-bind/bundle-after/outcome/final archive; stripping warning and unknown
complete outer build duration/OS exit; Windows SSH255/noRoute and remote
state UNKNOWN; V-READER-STARTUP UNVERIFIED; all failed files and guards; and
updater CLOSED. No past failed or missing evidence is promoted.
