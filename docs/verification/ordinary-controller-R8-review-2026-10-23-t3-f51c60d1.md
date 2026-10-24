# R8 independent static review of ordinary controller R7

Date: 2026-10-23, America/Vancouver.
Candidate commit: `d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6`.
Reviewer: isolated T3 worktree `t3-f51c60d1`, branch
`t3/ordinary-controller-r8-review`, initially clean at
`a99d065c27e65a46071f88db744ab29050d78cb1`.

**Verdict: one actionable P2 finding; no P1 finding. This exact source revision
is not clear for the next reviewed ordinary flow.** The unresolved-candidate
state does not survive the handoff from the main observer to recovery. Resolve
that source gap and review the resulting exact revision first. The missing
retained signed payload is a separate execution prerequisite, not another P2.
The ordinary flow remains already authorized. This review adds no human
permission gate and creates no executable receipt or general release clearance.

All candidate line numbers below refer to the immutable candidate folder:
`docs/verification/ordinary-controller-repair-r7-2026-10-23-t3-ad9d06e1/`.
The inspected disk copy is under
`/Users/serge/.t3/worktrees/openergy/t3-ad9d06e1/`.

**R8-P2-1 — Carry detected uncertainty into the Inventory that decides peer permission.**

Primary locations: `ownership.py:64-66,74-79,106-125,344-353`.
Handoff and consumers: `controller.py:124-146,345,355-362`;
`watchdog.py:15-18,221-248,338`.
Selection after a lost path read: `macproc.py:131-147` and
`ownership.py:111-114,142-145`.

R7 retains a detected private PID/start in `self.unresolved_private`, and
`absent` correctly rejects that map and `private_overflow`. However, both are
local to one Inventory object. Each new Inventory starts with an empty map,
empty pending state and a false overflow flag. `reload` reads only
`identity-*.json`, which contain owned identities. It does not read unresolved
detections or the main observation stream. The watchdog creates its Recovery
only after the cleanup request, and that constructor creates a new Inventory.
The parent's fallback also creates a new Recovery. Neither handoff supplies
the main Inventory's unresolved metadata, original uncertainty time/count or
overflow state. The cleanup request contains only normal/start/deadline fields.

The reachable trigger uses an already detected candidate, as follows:

1. The main observer sees a new private supervised daemon in its own group.
   Preliminary path classification succeeds, but coherent details fail. R7
   stores its PID/start in the main Inventory's unresolved map. No ownership
   identity or direct-wrapper receipt exists for this daemon.
2. The app exits and the daemon becomes PPID 1 before the watchdog's first
   cleanup observation. Cleanup can follow an observer/flow failure; failed
   acceptance does not relax the app/children-before-peer order.
3. Fresh recovery observations can still read the daemon's BSD metadata, but
   the preliminary path lookup fails. This is one of the uncertainty cases
   the new retention mechanism is intended to cover. In the fresh Inventory,
   this PID is absent from the unresolved map and the owned/direct token set.
   With no current marker, owned parent or registered group, selection drops
   the metadata row. The probe receives no unresolved token for it and need
   not obtain explicit ESRCH for this detected start.
4. Explicit ESRCH for the known app and previously registered app children
   can now make `tree_gone` true while the detected daemon remains in the full
   snapshot. If path uncertainty continues through the phase decisions,
   recovery reaches `peer-request` and publishes peer-stop permission. The
   controller can launch the public peer-stop CLI before that daemon is gone.

Even a saved main-observation row with `private_candidate=True` does not
prevent this path: recovery never consumes that detection state. The main
map can remain correctly unresolved throughout. Losing the state at this
ordinary handoff bypasses both its absence block and its finite uncertainty
allowance. The same reset also drops a retained overflow block. A later full
path/detail read can fail recovery, but cannot undo peer teardown.

Carry the already detected, bounded PID/start uncertainty and its fail-closed
state into the recovery observer before it can grant peer permission. Preserve
the original uncertainty allowance across the handoff. An unavailable handoff
must not manufacture app-tree absence. Keep this state separate from ownership:
it must grant no adoption, reused-process argv publication or signal authority.
This finding does not require discovery of every possible child, an atomic
kernel identity lease, an extra launch, or a longer deadline.

**Assessment of the two R6 findings and R7 responses.**

- **R6-P2-1: repaired within one Inventory, incomplete across recovery handoff.**
  `macproc.py:126-146,152-165` keeps the preliminary private marker through a
  failed staged read and checks both applicable start sets before detail
  reads. Unresolved tokens remain separate from owned/direct tokens. A reused
  start returns metadata only. A coherent unowned result outside the private
  root is discarded. `ownership.py:116-125` retains detection before budget
  accounting/publication; `147-202` retains it through local read uncertainty,
  reuse and missing enumeration. Full ownership validation or explicit ESRCH
  without a current row clears it. The local attempt accounting and sticky
  24000-entry cap fail closed. `absent` and `_final` include this local state.
  These protections do not transfer to the fresh recovery Inventory, as
  described in R8-P2-1. No relaxed orphan adoption or signal policy was added.
- **R6-P2-2: addressed in source for the reported receipt-I/O failure path.**
  `controller.py:325-353` collects handles first, then separates terminal
  lookup/load/parsing from the watchdog-loss action. An exception in the first
  boundary sets failed acceptance and reaches the next boundary. Actual death
  requires retained `control_exit` or a non-None handle return code; unreadable
  or missing evidence alone is insufficient. Construction uses `normal=False`
  and the original cleanup request/deadline. The object is assigned before
  the loss receipt, and its steps remain outside terminal I/O. Later iterations
  reuse that object. The existing early terminal return still requires sealed
  and closed in-memory direct-handle/intent sets. The producer's terminal path
  requires full final absence and the exact closed ledger before recovered:true
  (`watchdog.py:169-192,263-289`); a bounded failed terminal is not acceptance.
  I found no additional reachable P1/P2 in the changed receipt/fallback
  sequencing under that producer protocol. No watchdog, receipt-failure or
  fallback behavior was executed.

**Separate preservation assessment of all seven R4 repairs.**

1. **Actual interpreter and selected launcher: preserved in source.**
   `ownership.py:49-55,243-259` keeps actual-interpreter wrapper argv0 and exact
   remaining tokens. Product argv checks stay exact. `controller.py:178-182,
   419-425,440-446` keeps actual executable/hash separate from selected launcher,
   resolved launcher/hash, and checks the watchdog argv. The relevant functions
   are AST-equal to R5. No live interpreter identity was queried.
2. **Staged executable/argv and reuse privacy: preserved, with the adjacent
   handoff finding above.** `macproc.detail` is unchanged at `85-105`: two exe
   and two argv reads, coherence and final BSD identity precede publication.
   Errors retain no staged argv. The new full-snapshot partition checks
   unresolved starts without adding them to detail authority. `targeted` is
   unchanged. R8-P2-1 concerns lost negative absence authority, not a new
   relaxed detail or signal predicate.
3. **Abort, all-handle collection, pipe wakeup and drain despite evidence
   errors: preserved, with the R6 fallback remainder repaired.**
   `controller.py:45-169` is AST-equal to R5. Polling retains all real exits
   before publication; abort/request state precedes evidence; failed streams
   drain/discard. The at-most-512-byte nonblocking request pipe remains at
   `controller.py:135-144,431-435` and `watchdog.py:302-322`. The independent
   fallback action now survives terminal-read exceptions. Common evidence
   errors still fail acceptance without directly authorizing signals.
4. **Pre-spawn intents and exact sealed terminal sets: preserved for direct
   children.** `controller.py:183-201` retains attempts and writes intent before
   Popen, then retains the handle before post-spawn publication. Collection and
   the parent terminal gate require closed retained sets. The byte-unchanged
   watchdog checks intent/spawn/handle/exit labels, PIDs and collector identity.
   An unclosed direct launch cannot certify recovered:true. This direct-child
   protocol does not carry an unregistered supervised child's detection map.
5. **Narrow gated same-start exec after controller loss: preserved.**
   `Inventory.identify` is AST-equal to R5 (`ownership.py:211-285`). The exact
   wrapper gate, role, start, group, UID, NI15, interpreter and argv remain
   required. A new direct product needs a live matched controller or PPID 1
   with explicit controller ESRCH. Existing parent transitions also require
   explicit ESRCH. Unknown orphans and group matches confer no ownership.
6. **Successful TERM time and full five-second grace: preserved.**
   `Inventory.signal`/`_signal` and all watchdog source are unchanged.
   `ownership.py:329-339` retains successful delivery with the single-PID signal
   under the small alarm mask. `watchdog.py:62-103` uses per-PID/start delivery
   time and a later advance before KILL. Normal, targeted and CLI cleanup share
   this rule. Late delivery fails within the original deadline; it does not
   shorten grace or extend cleanup. No signal or timing behavior was tested.
7. **Cross-process byte reservations and rename-only archival: preserved.**
   `common.py`, `exec_owned.py`, `prepare.py` and `watchdog.py` are byte-equal to
   R5. The 32 MiB total includes eight autonomous 2 MiB slots, 4 MiB cleanup
   with 1 MiB terminal-only space, and 64 KiB accounting. The 8 MiB evidence-file
   ceiling, shared pre-write lock/reservations, conservative failed charges,
   first-record short circuit and drain/discard path remain. New observation
   and terminal fields use the same accounted writers. The archival path is
   unchanged in `Controller.run`: verified recovery precedes same-filesystem
   rename of reserved files, with no copy fallback. This is source preservation
   of the stated allocation, not a fresh aggregate-runtime or product bound.

**Independent static integrity checks and read scope.**

I read the complete R7 README, responses, design, operation plan, all eight
source texts, complete 131-line/26-hunk minimal patch, input bindings,
source-checks and manifest. I read the complete exact R6 review
`4eaa964eab6f74b2fc5704a1925520866c7d9f9c`, the complete R4 review
`cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d`, and all seven R5 responses at
`ec723d8fde7c65e955c015653f89d24f7cc7f091`. Prior verdicts did not determine
this verdict.

A new inline standard-library reader treated task files only as bytes, JSON,
text and AST data. It did not import or evaluate supplied code. It confirmed:

- All 16 R7 disk files equal their exact candidate Git objects. All 15 manifest
  entries match byte counts and SHA-256. All three R7 JSON files parse.
- All eight sources parse as AST data, with exact recorded line/byte/hash
  counts. All 31 named protected functions are independently AST-equal to R5,
  ignoring source-location attributes. The four claimed unchanged modules are
  byte-equal. Only controller, macproc, ownership and probe differ.
- All 26 patch hunks reconstruct in memory from exact R5 Git bytes. Removed
  bytes, old/new positions and counts agree; all four outputs equal R7. No
  patch was applied and no temporary source copy was created.
- All 14 R3 and 18 R5 bound files, the R4 and R6 reviews, match their declared
  immutable Git blobs, disk bytes, sizes and hashes. The root copy of the R6
  review equals its exact bound review. The retained prepared JSON record also
  matches its original Git bytes; that does not establish current artifact bytes.
- All 87 original-index files match commit
  `498190feb00f51039f500ae618ac7de5fb113d86` by blob ID, size and SHA-256, and
  equal both this worktree's and the authoritative `t3-52422617` disk copies.
  No original was changed. I did not repeat the unrelated 421-input or signed
  bundle audits.

Independently confirmed candidate SHA-256 values:

- Controller: `a204f8e87ba35c990c3e6eb96d9ed2ffde1bf28c3f8acbfa31b22a3248faf66f`
- Minimal patch: `fde7c0c0c9c2c32755cc13823fab95242ddc0e2badbb240a7d54a8855b37db1e`
- Manifest: `3e433c05c352b245ad46c3988f31d481fa04993c1c60981e6ace7cf11d048b77`

**Saved records, fixed scope and remaining proof.**

I audited the saved original outcome, first controller failure, first cleanup
traceback, complete 25-step recovery report and predeclared limits. They retain
the shared-PGID assertion, early peer stop, 23/25 result, absent app OS exit and
missing late priority coverage. The original remains **FAILED / NOT ACCEPTED**.
These are saved records, not a fresh reproduction. They establish no product
defect. R3, R5 and R7 have never run; this review adds no fresh runtime proof.

The reviewed source/plan keeps one ordinary no-render `--e2e --require-peer`
default25 AppModel/daemon/peer IPC flow and one launch per role. App/children
must be gone before peer teardown; R8-P2-1 blocks that source qualification.
The fixed limits remain 120 s flow, 60 s cleanup, start + 180 s end, 6000 rounds
and 24000 rows with 1024/4096 reserved for cleanup. The local detail allowance
remains eight observations/two seconds, subject to the handoff gap. The same
16 MiB frames, 1,000,000-byte stalled write, 300 ms request, 1 s responsiveness,
2 s stalled bound, 3/5/10/15 s waits and 1.5 s supervisor stop remain bound.
Future actual NI15 and private literal loopback/virtual/discovery-off config
must apply before startup. Rust 1.99.1/jobs2/deployment15 are inherited build
lineage only.

The supplied prerequisite state says the exact retained signed bundle, app
and daemon under `/private/tmp/oe-ordinary-79498-l7aurhai` are absent. I did not
query those payload paths or treat historical hash/existence records as current
bytes. The reuse-only next-flow plan cannot start without those exact inputs.
This missing payload is not a new P2, permission gate, or reason to suggest a
substitute or rebuild. After source repair/review, actual identities/ABI, NI15,
stop/restart observations, all direct OS exits, 25 unchanged passing checks,
unchanged bundle bytes and full cleanup still require the real authorized flow.

No other concrete actionable P1/P2 was found in the four changed sources and
their directly adjacent ownership, privacy, recovery, deadline and archival
paths. This bounded conclusion does not cover every possible child or provide
an atomic read/signal guarantee.

Only this new review document was written. No source edit or integration was
made. No supplied program, module, helper, validator or task AST was executed,
evaluated or imported. No fixture, preflight, product, build, signing, unit test,
mock, fake process, interleaving simulation, process/port/live-OS query, desktop
driving, network, delegation, push or publication occurred. Checks used only
file/Git/hash/JSON/AST/patch reads. The review commit disables repository hooks
and commit signing. Other worktrees were read-only.

The root `continuation-2210.json` boundaries remain in force: no live-sharing,
installed-app, production config/TLS/default IPC, TCC, login, service, task,
account, firewall, global DNS or toolchain changes; no TCP24800/UDP5353, Docker,
credentials or private-key access. The three provider-stopped tasks and isolated
Swift build gate stay stopped. The separate Sparkle one-shot is closed after
its preflight failure, with no retry, renewal or approval reuse.
