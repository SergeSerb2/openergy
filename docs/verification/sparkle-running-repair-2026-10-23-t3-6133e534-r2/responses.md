# Responses to the two R4 P2 findings

Implementation response only. A fresh independent review is missing. This
document grants no running/relaunch approval or runtime acceptance. Exact
patches are [controller.diff](controller.diff) and [proposal.diff](proposal.diff).
The full reviews and previous responses remain byte-exact in [inputs](inputs).

## P2 R4-1: order actual acquisitions and merges

`identity()` now holds the existing `IdentityLedger.lock` before calling
`raw_identity()` and through the immediate `observe()` merge. The only other
raw read is the daemon-parent read inside `observe()` under that same RLock;
its scoped result is immediately added before parent-role checks. No code
holds a snapshot while waiting to acquire the ledger lock for its merge.

`fixture_processes()` uses `identity()` for each complete row and does not
replay its result list. The observer also does not replay that list.
`match_ready()` checks already-recorded identities and roles without replaying
the stored ready rows. These changes cover observer, main, recovery, parent
reads, and the previous repeated bulk replays.

`add()` still accepts only an unchanged executable or ENTRY to IMAGE for a
single PID/UID/start identity. It saves explicit forward-transition evidence.
Reverse/other execs fail. Counts do not reset at exec or readiness. All original
priority, UID/start/path, parent, restart, and 12/32/2/2 checks remain. Rejected
full scope rows are retained before validation, so they can be reported later;
this does not grant permission to act.

No acquisition-sequence scheme or second lock was added. Acquisition and merge
share one reentrant lock. The new cache guard uses that same lock. Ledger code
does not acquire ACTION_LOCK or IO_LOCK; the hard timer remains independent of
ledger/evidence locks. This is source reasoning, not a concurrency execution
or a runtime deadlock proof.

Changed locations: `identity`, `IdentityLedger.add/observe/match_ready`,
`fixture_processes`, and `observe_until_stopped`. Proposal: `identity_ledger`.
AST records enumerate both raw-read sites and the sole `observe` call site.

## P2 R4-2: final all-scope outcomes and guarded cache work

`scoped()` returns every retained observed identity, including helper-only and
rejected rows. `owned()` is a separate, narrower action-candidate list. Existing
signal ownership/path/hash checks and exact-job bootout gates are unchanged.
No unknown helper is adopted, signalled, reniced, or unregistered.

`reconcile_scope()` takes a current fixture scan, then independently checks
every scoped identity with `gone()` and records exited/live/unconfirmed.
Absence still needs libproc plus ESRCH. Any newly added parent/identity during
these reads also receives an outcome. Incomplete scans, unresolved scope reads,
or identity-cap overflow explicitly fail scope completeness. Overflow stays
outside the bounded in-memory ledger and has a saved unconfirmed outcome;
the unchanged 32-identity cap is not enlarged to permit cleanup.

Recovery calls reconciliation after the full quiet interval and before cache
work after OS capture. Once cache work begins, any freshly observed live scoped
row latches `cache_blocked`. It cannot be cleared when a helper later exits.
`require_cache_quiet()` checks all-scope outcomes, this latch, and observer/
ledger failures. Each cache inventory is followed by another check. Instead of
one opaque `rmtree`, removal checks between each individual unlink/rmdir and
after each cache. One check and one destructive step share the ledger lock.
Descriptor-relative traversal with `O_NOFOLLOW` prevents following substituted
interior symlinks. Device/inode, UID, canonical-root, type, and bounded-inventory
checks remain. Any failed check stops further removal and retains remaining
state and evidence; already removed entries are not claimed restored.

In `finally`, main/reader quiescence and observer/HTTP joins precede a fresh
current scan and final all-scope reconciliation. This includes a helper recorded
after the last cache check or during final OS work. Any live/unconfirmed row,
incomplete scope, observer/ledger failure, or cache-phase latch adds an explicit
cleanup failure and error with `retained_state:true`. The result includes the
individual outcomes. This happens before `RECOVERY_DONE`; main's unchanged
timer-disarm path follows recovery completion and join. A thread/deadline
failure keeps the existing terminal retained-state behavior.

Sampling and a final scan cannot prove no future kernel event. No such claim is
added. They also cannot excuse an identity already recorded. This source patch
does not prove late-helper behavior, OS timing, or deletion behavior at runtime.

Changed locations: `IdentityLedger.add/owned/scoped`, `fixture_processes`, new
`reconcile_scope/require_cache_quiet/remove_cache`, and `recover`. Proposal:
`identity_ledger`, `owned_cleanup`, and `absolute_deadlines.whole_deadline`.

## All three R2 corrections and approval boundaries remain

R2 R1: The same persistent ledger starts before the only initial launch and
remains active through final exit and quiet recovery. PID/UID/start keys,
old-parent binding, child association, old-parent restart rejection, and
independent old-app/all-old-child exits remain. The role/count validator is
AST-identical to the previous repair.

R2 R2: Absolute IPC/HTTP deadlines, 180s control plus 60s recovery within 240s,
cancellation/action gates, direct OS-reader bounds, and the independent fixed
terminal timer remain. IPC, HTTP, reader, timer, launch, and main function ASTs
are unchanged. Recovery still waits 10s before SIGTERM and 15s before SIGKILL or
exact-job bootout. All proposal limit values are equal to the previous repair.

R2 R3: There is still exactly one `normal_quit` call, in main after validation of
the exact NEW build/launch/app/child identity, readiness, candidate inventory/
seal/config, and old exits. Recovery has none. The quit and signal functions
are AST-identical to the previous repair. No old-app normal quit or second
controller target launch is added.

All source/artifact bindings remain at `ad28064`. Marker expiry remains
2026-10-24 00:56:25.164527 PDT, without renewal/key/re-sign/private changes.
Host effects remain exact fixture jobs/caches, helper nice0, target/daemon
nice15, normal LS/GK/gktool/XProtect work, and unknown persistent shared state.
No restoration, trust, runtime, or publication claim is made.

The proposal keeps `DO-NOT-RUN`, `approval_received:false`, independent review
missing, and the exact prior operation scope. Fresh independent review of the
new controller/proposal hashes and a separate hashed review receipt must precede
the parent's user approval request. The stopped-only approval remains invalid.
No receipt or approval was synthesized. Only static file/hash/AST checks ran;
no controller/helper/watcher, preflight, product, simulation, test, or mock ran.
