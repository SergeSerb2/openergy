# One retained child direct signal boundary

SOURCE/DATA only. Parent repair only. No signal or target child ran.
Read the entire COPY review f33a1fd/import931d2c3 and its public subprocess
source evidence. That actual public3.9 file and all excerpts were rehashed as
text. Popen.kill calls send_signal, which polls again and can call waitpid.
It can skip os.kill when that poll sees exit. It also suppresses ESRCH. Therefore
its return cannot truthfully set a delivery flag or preserve the sampled gate's
no-intervening-OS-query order. The frozen prepare collect has precisely this
same method call and kill_delivered assignment after its explicit eligibility
poll/sample/two-end predicate.

The exact parent-only delta is one call and its status field rename. All other
collect bytes, including launch admission, cap/read logic, first-error latching,
real exit observation, wait remainder, close/finally/tail and returned buffers,
are byte exact. After fresh poll=None and its immediate shared sample, and only
for overflow or timeout, both unchanged strict ends must still be open. The
next OS operation in that branch is os.kill(proc.pid, signal.SIGKILL). Attribute
reads and in-memory flag assignments do not query the OS. proc is the original
retained, still-unreaped direct Popen child. No process lookup, PID acquisition,
group target or other signal authority appears. No other reaper or overlapping
writer is authorized. Ordinary scheduling pauses and OS stalls still apply.

kill_attempted becomes true before the call. kill_syscall_returned becomes true
only after the syscall returns without error. It certifies only that return.
It certifies no delivery, child termination, real exit code, reap or EOF. A
ProcessLookupError/ESRCH or other failure stays the actual kill_or_reap_error,
latches failure and cannot fabricate exit/reap. Existing wait remains on only
the original reap_end remainder; no extra wait allowance, retry or signal is
added. observe_exit retains genuine signed poll/wait codes, including negative
signal exits, before its next clock sample. No new code coerces those values.

This replaces kill_delivered with kill_syscall_returned in both new parent
entry-helper and prepare statuses. It is a rename, not an additive alias. The
schema tag stays openergy.prepare-file-child.v1; closed consumers of the old
key need an interface update. child-status.schema.json gives the exact new
closed key interface with signed actual_exit/kill_eligibility_exit domains.
No historical SOURCE/caller/COPY status is changed. No actual schema instance
or future receipt is synthesized.

The full COPY inspect_new_prepared and collection_complete were inspected;
copy-integration-source-excerpts.txt binds their exact complete bodies. They
access prepare status fields generically, use collection_complete(primary),
and compare receipt.primary to the same full primary JSON. Neither demands a
closed prepare-primary set or uses kill_delivered. Thus the parent field rename
is source-compatible at those inspected accesses. COPY's native collector and
native-record checks remain its distinct interface; this repair changes none
of them. Full COPY remains HOLD_SOURCE under its own two P2 findings. This
bounded integration observation supplies no source or runtime CLEAR for COPY.

frozen-dependency-signal-audit.json records untouched same-pattern uses in R9
caller.collect, SOURCE launcher.collect and the phase collector.settle. Related
common.command also calls Popen.kill under a different older command contract.
Those are UNVERIFIED_SEPARATE_AUDIT_REQUIRED, not silently CLEAR. Their narrower
historical review results remain as recorded. prepare/helper do not add a new
signal site here; only the parent collector changes. No upstream common,
caller, prepare, helper, collector or historical status file was edited.
Target library identity, runtime reachability, timing, delivery and reap need
separate authorized evidence and remain UNVERIFIED.
