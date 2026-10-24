# Responses to independent R2 P2 findings

This is an implementation response, not a new independent review or execution
clearance. Full byte-exact patches: [controller.diff](controller.diff) and
[proposal.diff](proposal.diff). Unchanged source/fixture/host-effect fields are
checked against the original proposal in [static-checks.json](static-checks.json).

## P2 R1: persistent observed-identity ledger

Replaced the initial `owned` list and later-only `all_seen` set with one
`IdentityLedger`, created before the initial launch. Its key is
PID/UID/start_sec/start_usec. ENTRY to IMAGE counts once. Direct identity reads
and the persistent observer feed the same ledger, including initial readiness,
blocking IPC/OS work, final exit and the full quiet interval.

Each daemon maps to an independently matched app identity. The launched PID
defines the old parent; separate build-2 readiness validates the new parent.
A second old-parent child identity fails without consuming the new-parent
allowance. All supplied rows are recorded before role checks. Sampling continues
after recorded role failures. Independent libproc absence plus ESRCH remains
required for the old app and every observed old child. Recovery records each
old identity's outcome, including unresolved/error outcomes.

Minimal change map: `identity`, `IdentityLedger`, `fixture_processes`,
`observe_until_stopped`, the pre-launch observer lifecycle, handoff exit checks,
and recovery quiet/exit records. Proposal: `identity_ledger`,
`supervisor_stop_limit`, and added observer/deadline limits. Original 12/32/2/2
limits stay unchanged. 50ms plus scan time is sampling, not kernel coverage.

## P2 R2: absolute exchange and whole-operation deadlines

`exact_read` now accepts the exchange's monotonic absolute deadline. `ipc`
recomputes remaining time before connect, send, every header/body recv and after
decode. The first exchange includes connect. Four status/config exchanges,
request/response byte caps and no-retry behavior stay unchanged. HTTP recomputes
remaining time before every recv and each separate header/body send. Both paths
clamp their deadlines to the control cutoff and whole-operation deadline.

A separate recovery thread acts on first fault, control completion or 180s.
It cancels registered I/O, stops direct OS readers and closes the launch/quit
gate. This reserves 60s inside the original 240s ceiling. If an active action
cannot quiesce within 1s, it records retained state and stops all controller
threads. Otherwise it uses the existing delayed owned recovery. A separate
240s timer can stop the controller even while the main or recovery path blocks.
It writes a fixed terminal record without the main/ledger/evidence locks.
Normal completion disarms that timer only after recovery has joined.

Minimal change map: `budget`, `remaining`, `exchange_socket`, `stop_io`, `ro`,
`exact_read`, `ipc`, `Feed`, bounded HTTP lifecycle, `hard_deadline`, `recover`
and main's cleanup handoff. First errors are ordered and retained per source.
There are four bounded-lifetime background threads and no background output
reader. A non-quiescent path blocks cache deletion and ends with retained state;
it cannot resume mutations after controller exit. Proposal: `absolute_deadlines`
and the new 180/60 fields. No strict peak-resource or reproduced timing claim.

## P2 R3: normal quit only for validated NEW app

Removed the failure path that selected any live IMAGE. Recovery has no call to
`normal_quit`. The sole call follows an explicit `VALIDATED_NEW` record with
build2, distinct launchID, matched app/child identities, real IPC readiness,
candidate tree/seal and private config checks, and old identity exit proof.
The quit function requires that record, the same PID/start/UID/path, candidate
build/image hash, and an unused quit allowance before the native API call.

Minimal change map: `normal_quit`, `VALIDATED_NEW` creation in main, and removal
of the old `finally` normal-quit block. Proposal: `final_quit`, clarified
`approval_scope` and `owned_cleanup`. A failure before validation has only the
declared delayed owned recovery or retained-state result. It never normal-quits
the old app. SIGTERM still waits at least10s; SIGKILL/exact job bootout still wait
at least15s. PID/start/UID/canonical path/hash checks remain. No shared identity
unregister was added.

## Review and approval bindings

The new receipt gate also requires a separately hashed independent review
receipt with both corrected file hashes, `independent_review_complete:true`,
`result:clear_for_approval`, and no unresolved findings. New explicit approval
must bind those same hashes, that receipt and the exact scope. No such receipt
or approval exists here. Original source/fixture activation gates remain.
The parent must delegate a fresh independent review before approval or execution.
