# R9 collector status contract

SOURCE/DATA ONLY. execution=false. IMPLEMENTED_UNREVIEWED. Independent R10
review is next. This document adds two collector observations to the retained
[compact contract](compact-success-contract.md). Its old R7/R8 review-state
references remain historical. Later parent use must bind exact R9 bytes and
the independent R10 result. This adds no human permission gate.

`kill_eligibility_exit` is the actual return from the one final settle
`proc.poll()` on the retained direct-child handle. It starts null. An integer,
including zero or a negative signal status, is a real returned exit. `None`
is not an exit and does not authorize an inferred status.

`kill_eligibility_observed_ns` is the immediate shared-clock sample after that
poll returns, including a return of `None`. It starts null. A returned sample
is an integer from the unchanged darwin6 CLOCK_MONOTONIC ns API. It is a
post-return observation, not the OS event's exact time or a hard scheduling
bound. No other OS call occurs between the poll return and this clock call.

The clock read is inside a `try/finally`. If the poll returned an exit, the
`finally` marks the real reap even when the sample raises. The original settle
exception handler still retains the error and deadline loss. Null sample
means no completed sample was retained, including an unentered settle block,
a poll exception or a clock exception. Both fields null do not prove that no
poll ran. An integer exit with a null sample preserves the actual exit while
the settle path remains failed. Missing observation facts are not filled in.

For a `None` poll return, KILL requires the retained sample to be strictly less
than BOTH `command_deadline_ns` and the unchanged full result's phase end.
Equality fails the gate. If either predicate fails, `timed_out` and
`deadline_loss` become true before the original bounded wait statement, and
KILL is skipped. The wait still uses `remaining(deadline)` and cannot obtain
an added interval. If the clock raises, control reaches the existing exception
handler before any KILL attempt. There is no retry or second poll in this gate.

A non-null poll return never permits KILL and marks `reaped=True`. The existing
bounded wait statement is retained. Final `actual_exit` still reads only
`proc.returncode`; it is not filled from a timeout, signal request or assumption.
A `None` eligibility result can be followed by a later actual wait exit. The
two facts stay distinct. A non-null eligibility exit remains the final actual
exit even if the sample, remaining-time check or context postcheck fails.

These fields are added only to the existing collector record. Unchanged
`perform` copies them into active/collected command and first-failed-COMMAND
status before publication. The existing OS/status save retains them, and the
retained handle's collector status carries the same facts to `direct_children`.
Existing raw-reference enrichment and snapshot timing remain unchanged. The
parent must preserve and compare these fields across those existing bindings,
along with every existing actual/null/raw/EOF/drop/partial/publication fact.

For R9 records, parent must retain both keys with exact integer-or-null types.
If KILL was attempted, the eligibility exit must be null and the retained
integer sample must be strictly before both original ends. A late sample
with null eligibility exit must retain both failure flags and no KILL. A
non-null eligibility exit must agree with final actual exit and actual reap.
A null sample is not evidence of a timely completed eligibility check. It
cannot erase any settle error or loss. Existing `complete` predicates stay
exact; these observations neither replace them nor independently accept a run.

No key is added to the closed parent v2 envelope or compact success receipt.
Their schema/template bytes and<=512B success byte proof stay exact. Only the
full result's existing status objects grow. Their actual serialized length
and SHA-256 still come from the one original durable result save and its exact
return. No second save, writer, receipt file, ledger action or byte allowance
is introduced. Parent must inspect the exact R9 result construction, all53
SOURCE or52 COPY commands and the full raw/status/comparison bindings. The
retained contract's old phrase about exact R7 construction now refers to this
document's exact R9 additive status construction.

Actual caller0/reap and complete raw stdout/stderr EOF before the SAME original
120s end remain parent duties. Receipt or full result data alone never supplies
an actual caller exit. COPY parent prefix authentication must still precede
private entry inside that original phase. First COMMAND before writes and
comparators after durable raw/status remain exact.

The post-poll sample is a source ordering repair. A later OS call can stall,
and scheduling can pass a deadline after the sample. There is no hard-preemption
claim, timing experiment, target pipe observation or runtime qualification.
No drain, group signal, PID lookup, retry, extra time, second writer, changed
caps, startup exception or new acceptance predicate is introduced.
