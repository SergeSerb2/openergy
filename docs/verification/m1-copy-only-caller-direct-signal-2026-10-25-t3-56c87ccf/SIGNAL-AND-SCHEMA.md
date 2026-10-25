# Collector and v3 status contract

SOURCE/DATA ONLY. execution=false. IMPLEMENTED_UNREVIEWED.

The final `proc.poll()` retains its actual signed integer or None. The next
call is the unchanged shared `clock_ns()`; its `finally` preserves reaped=True
for a real returned exit even if the clock raises. A real exit never permits
KILL. It stays an exit even if later timing/publication fails.

Only a None result with its completed sample strictly below BOTH the original
command and phase deadlines can enter the KILL gate. Within that gate, an
overflow or timeout must also be recorded. Other collector errors cannot
authorize KILL. If the sample is late or equal, the original timeout/loss flags
remain. No wait allowance is added in any case.

After those gates, the source assigns kill_attempted=True, then calls
`os.kill(proc.pid, signal.SIGKILL)`. It uses only the PID of the retained,
still-unreaped direct child. There is no intervening poll, OS query, PID lookup,
group target or helper. There is one call site and no retry. The frozen public
library audit in input-bindings.json shows os exporting the POSIX primitive;
the old Popen.kill wrapper could poll and return without signalling. No target
library or C implementation was newly executed or qualified. Trusted target
implementation, no other reaper and exclusive ownership remain assumptions.

kill_syscall_returned starts false and becomes true ONLY after os.kill returns.
It records successful syscall return, never delivery, termination, exit or reap.
An ESRCH/ProcessLookupError or other syscall error leaves it false, preserves
kill_attempted=True, and enters the original kill_or_reap_error/deadline-loss
path. It does not infer an exit or continue with a fabricated code. The original
wait is unchanged and uses only `remaining(deadline)`. A None eligibility result
can be followed by a real wait exit; the two facts remain separate.

The final actual_exit still comes only from proc.returncode. Signed real codes
are never clamped or replaced. Errors, None/nulls, incomplete samples, dropped
bytes, unknown unread bytes and publication failures remain failed evidence.
If an exception escapes before record assembly, a retained handle can still
have null collector status. No completed record is promised for every failure.
There is no communicate, drain, new process, second signal or extra wait.

Scheduling can pass a deadline after the sample; a syscall can stall. The
existing alarms/postchecks reject late success but prove no hard preemption,
signal delivery, containment or reap guarantee. This is static source ordering.

The [schema](child-status-v3.schema.json) applies only to child records under
NEW full COPY `openergy.metadata-observations.v3`; raw OS records gain no schema
key. `collector_status` has the exact core keys from collect. `published_status`
adds both durable stdout/stderr references. `retained_status` permits either
reference to be absent after partial publication; `direct_child` can hold that
status or null. The unchanged collected snapshot predates raw enrichment.
Do not equate all snapshots byte-for-byte across those different stages.

actual_exit and kill_eligibility_exit are integer-or-null WITHOUT a minimum.
Thus negative signal codes remain representable as failure facts. PID is
positive-or-null; timestamps, byte counts and reference byte lengths are
nonnegative. Null eligibility time means no completed sample retained, not
proof that no poll ran. A non-null eligibility exit must agree with the actual
exit and reap; a null sample cannot prove timely eligibility. A signal attempt
requires null eligibility exit and a timely integer sample. Syscall-return
true requires an attempt, but neither field implies a non-null actual exit.
These relations and deadline arithmetic require parent source checks; scalar
JSON schema domains alone cannot prove them.

The success schema keeps actual_exit=0 and eligibility_exit=0, reap=true,
kill_attempted=false and kill_syscall_returned=false. The complete success gate
also needs both EOFs, no errors/unknown/loss/overflow/timeout/drops, exact clock,
serial order, PID/argv and all full raw comparisons. Its other failure flags
stay constrained by the unchanged complete predicate and future exact consumer.
No populated example receipt, synthetic validation result, mock or target
execution was created to reason about these domains.

OLD saved SOURCE `fresh_source` requires v2 exactly. Both it and generic complete
are byte-exact to R9; complete does not read kill_delivered. Old successful
SOURCE therefore remains usable without modifying any old record. SOURCE CLI
is rejected by the alias's first admission gate. Its later original SOURCE
branches and SYSTEM3.9 bootstrap exception remain unchanged unreachable code.
COPY startup still allows only the original ten keys and no extra exception.
