# Implementation response and next review

The concrete first failure is retained in the exact
[stderr](inputs/first-failure/raw/controller.stderr),
[LS error record](inputs/first-failure/raw/preflight/0007-preflight-ls-first-error.json),
[result](inputs/first-failure/result.json),
[README](inputs/first-failure/README.md) and
[selected postflight](inputs/first-failure/selected-file-postflight.json).
Their source commit is `1cc4d03113d8d78be84ad0fbc161610860455f35`.
No partial database output is reconstructed. No original error is replaced by
this source work. The one approved invocation remains failed and closed.

The failure was work on the whole shared LS database under a fixed observation
budget. `os_evidence` now substitutes one exact-ID native metadata child for
that command. It keeps the original phase call sites and all other evidence
commands. The child queries only the original target ID and driver ID. It has
strict local count/string/output gates and no scan, registration, mutation,
launch, retry, alternate path or port fallback. It uses the unchanged `ro()`
failure path and five-second deadline; it does not guarantee that any future
query completes. See the [API rationale](source-api-rationale.md).

All five earlier P2 repairs remain in unchanged controller ASTs:

- Persistent restart accounting: `IdentityLedger`, `identity`,
  `fixture_processes`, `observe_until_stopped`, readiness and main are unchanged.
  The ledger begins before launch; old-parent replacement and all-old-child
  exit checks remain. Counts do not reset at readiness.
- Absolute deadlines and recovery: `budget`, `remaining`, `exact_read`, `ipc`,
  `Feed`, `ro`, `stop_io`, timer and recovery are unchanged. Control remains
  180 seconds with 60 seconds reserved inside 240. Recovery delays stay 10/15.
- NEW-only normal quit: `normal_quit` and its sole main call remain unchanged.
  Independent build/launch/app/child/config/image validation still precedes it.
  Recovery has no normal-quit call.
- Ordered same-PID exec observations: the existing ledger RLock still spans
  full acquisition and immediate merge, including parent reads. Stored ready
  rows and bulk scans are not replayed. ENTRY-to-IMAGE forward evidence remains.
- Final all-scope reconciliation: scoped versus owned identities, incomplete
  scope failures, cache live-observation latch, relative traversal, per-step
  guards, post-observer-join final scan, and timer-disarm order are unchanged.
  Metadata URL results never enter that ownership or process ledger.

The original limits object is equal in full. This retains one initial launch,
one updater, one validated NEW quit, four IPC reads, exact HTTP routes, bytes
and request counts, process/thread/identity caps, fixed cache roots, observer
join, no retry/fallback and absolute nice15 requirements. Helper nice0 remains
disclosed. All 451 source-hash records, inventory/artifact references, source
root, nonce, marker/config, public key, signed archive/feed, receipt contract,
approval scope and host-effects fields remain equal to the cleared proposal.
The actual old approval and guards are untouched. No executable receipt is made.

The copied proposal changes only four prior fields and adds two provenance/API
fields. `approval_received`, `new_independent_review_received`, fixture approval
and `runtime_accepted` remain false. Added `runnable`, `clear_for_approval` and
`runtime_authorized` flags are also false. Historical R6 clearance cannot clear
the new hashes, and the old 21:01 approval cannot authorize another invocation.

The next independent source review must read the exact controller, helper,
proposal and patches, recompute freeze/static preservation, and examine the
ctypes ABI, releases, native absence-domain check, bound ordering, outside-path
redaction and narrowed claim. It must confirm preservation of all five P2
repairs and all original immutable bindings. This handoff is implementation
work, not that independent clearance.

Retention is a current blocker from the user's 22:10 report, not a condition
silently repaired here. Exact private artifacts must be available. The old
marker cannot be renewed and expires 00:56:25.164527 PDT on October24. Terminal
guards/raw evidence cannot be reset, moved or deleted. A viable prospective
operation must be concrete, independently reviewed and separately consented.
There may be no permissible run under the remaining frozen fixture and time
window. No approval request or next runtime is granted by this source task.
