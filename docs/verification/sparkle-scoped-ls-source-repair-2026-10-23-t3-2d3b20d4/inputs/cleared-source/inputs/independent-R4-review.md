Independent R4 source review of the running-update controller repair

Reviewed on 2026-10-23, America/Vancouver. Result: **two actionable P2 findings;
no P1 finding**. This candidate is **not clear for approval**. The review is
complete, but it supplies no integration, execution, live-preflight, or runtime
approval. No controller or product behavior was reproduced.

The reviewed repair commit is
`82cb25b6775d69fe1a312bdc1f26486aba0bf287`. The complete reviewed folder is
`/Users/serge/.t3/worktrees/openergy/t3-bd21f531/docs/verification/sparkle-running-repair-2026-10-23-t3-bd21f531-r1/`,
called `REPAIR` below. All controller line references are to its frozen
`controller-DO-NOT-RUN.py.txt` at that commit.

Exact reviewed SHA-256 identities:

- Controller: `3d08b11c2dda9256314d6bee0f7da7c7651cd0ea45b3f8dc7f8d3a9b1f0b1066`.
- Proposal: `667c5ef9a362c3319390f0e5e77adc19e7364a34425074edfd08731158774c6b`.
- Freeze manifest: `2e66b88376fb2594125d6b02331ac42fd5cd991ae86b180533d363a07749a083`.
- Original controller snapshot:
  `e251cf7477ed9accf7c28e15de4cabfad50306e12c1934c9b37155a1f40b2740`.
- Original proposal snapshot:
  `2d23db1f735e607d70f92e29a8e0837425c4908bd935a3950ba60f458dc6f2a6`.
- Full original R2 review snapshot:
  `87a14f6446ebe11d26fff5b9da9e53ae0ddbc4c1b1e3beff8c102a5003f36cd6`.

**P2 R4-1: Order process observations before applying the exec-transition rule.**

Locations: controller lines 209-217, 243-254, 258-279, 318-324, and 348-374;
proposal `identity_ledger.lifetime` and `identity_ledger.coverage`.

`raw_identity()` reads occur outside the ledger lock. The observer, main control
path, and recovery path can therefore merge snapshots in a different order from
their acquisition. `IdentityLedger.add()` accepts ENTRY to IMAGE, but rejects an
ENTRY snapshot after an IMAGE snapshot for the same PID/UID/start identity.
There is no observation sequence or stale-snapshot handling.

A valid initial launch can fail as follows. The observer reads ENTRY and adds it
to its local `result`. The launcher then performs the permitted same-PID exec.
The main path's `identity()` reads IMAGE and updates the ledger. At the end of
the observer's scan, `LEDGER.observe(result)` replays its earlier ENTRY snapshot.
Lines 248-249 raise `unexpected exec`. The same replay occurs again at line 322.
Parent reads at lines 267-269 can also advance an app to IMAGE before the bulk
scan replays an older ENTRY row. This does not require a reverse exec, a restart,
or a missed process sample.

`fault('ledger', ...)` makes this a permanent operation failure and starts
recovery. It also prevents cache removal at line 720. Thus ordinary scheduling
can consume the one-shot operation and leave retained state on an otherwise
permitted initial launch or Sparkle relaunch. Counting ENTRY and IMAGE with the
same key does not resolve the ordering error.

Required correction: preserve the acquisition order or freshness of observations
when applying exec transitions. Serialize the relevant reads and merges, or use
an ordered observation record that cannot roll a known IMAGE state back to an
older ENTRY snapshot. Preserve evidence of the permitted forward exec and retain
the UID/start/path, unexpected-exec, priority, restart, and identity-count guards.
Do not treat a stale snapshot as a new process or relax the guards for other
executable transitions. This is a static concurrency finding; no interleaving
was simulated or executed.

**P2 R4-2: Reconcile late observed helpers before reporting completed recovery.**

Locations: controller lines 305-307, 700-720, 721-735, 739-772, and 962-970;
proposal `identity_ledger.exit_proof`, `owned_cleanup`, and
`absolute_deadlines.whole_deadline`.

The full quiet interval and `all(gone(...))` checks finish before the OS log
capture and cache removal. Each cache has a process-absence precheck at line 723.
After the last such precheck, recovery can spend time inventorying and deleting
that cache. It then checks only the socket. In `finally`, it records exit
outcomes only for old app/child identities, stops and joins the observer, and
writes the result. There is no final process snapshot or exit reconciliation for
identities added during that last part of cleanup.

In particular, a delayed fixture helper can appear after the final cache
precheck and be recorded by the persistent observer before it stops. A helper
with the fixture path/ID and correct UID does not violate the two-app/two-daemon
limits and can remain below the 12/32 process limits. Also, `LEDGER.owned()`
includes only the target's three executable paths and the driver, because those
are the only keys placed in `EXECUTABLE_HASHES` at lines 849-852. Helper-only rows
are not covered by that list even if it were checked again.

The result can therefore contain the observed helper identity while still
reporting `errors:[]` and `cleanup_failure:null`. Recovery sets `RECOVERY_DONE`,
main disarms the whole-deadline timer, and `require(not ERRORS)` passes while
that observed helper remains alive. `runtime_accepted:false` correctly reserves
runtime acceptance for a later review, but it does not make this a failed or
retained-state cleanup result. The same gap can permit deletion of a cache while
the newly observed helper uses it.

Required correction: reconcile the final observer ledger and current fixture
process state at cleanup completion. Give every observed scoped identity,
including helper-only rows, an independent exited/live/unconfirmed outcome.
Any live or unconfirmed identity must produce explicit retained state and failed
cleanup, and must block further cache removal. Preserve exact helper ownership
and hash checks before any recovery action; this finding does not authorize
adopting or signalling an unknown process. Only disarm the whole-deadline timer
after this final reconciliation and thread completion. Keep the sampling limit
explicit: a final observation cannot prove that no later kernel event will occur.
The issue here is an identity the observer actually records, not an unavoidable
gap between samples. No late helper was launched in this review.

**Responses to all three prior R2 findings.**

R2 P2 R1 is substantially addressed, but the new ledger is not ready for source
clearance because of R4-1 and R4-2 above. The ledger exists before launch; direct
identity reads and the persistent observer feed it through readiness, OS calls,
handoff, final exit, and quiet recovery. PID/UID/start is the key. The initial
parent is bound to the launched PID. Child-to-parent identities are retained.
An observed second old-parent child is rejected instead of using the candidate
allowance. Old app and every associated old child require libproc absence plus
ESRCH. The original D1/D2/D3 accounting bypass is no longer present. The remaining
ordering and final-reconciliation defects concern observations that are made.

R2 P2 R2 is corrected in source for the stated blocking-I/O defect. Each IPC
exchange uses one absolute monotonic deadline, including first connect. Remaining
time is recomputed before send and every header/body read, then checked after
decode. HTTP recomputes the same connection deadline before each recv and each
header/body send. Deadlines clamp to the control and whole deadlines. The four
IPC requests, frame/byte limits, route/method limits, and no-retry rule remain.

The recovery thread wakes on fault, control completion, or the 180-second cutoff.
It sets cancellation, waits at most one second for the action gate, interrupts
registered socket I/O, kills direct OS-reader children, and joins the HTTP path.
Read-only subprocess capture has a remaining-time limit and a bounded timeout
drain. The separate whole timer uses a fixed terminal failure record and
`os._exit(1)`. It does not wait on the evidence or ledger locks. The terminal
path says `retained_state:true`, uses exit one, and does not claim product cleanup.
These are source mechanisms, not timing or OS scheduling proof. R4-2 concerns the
ordinary recovery-completion path, not a claim that terminal self-exit kills
owned products.

R2 P2 R3 is corrected in source. AST inspection finds exactly one `normal_quit`
call, at line 948, and none in recovery. It follows build-2 readiness, distinct
launch identity, exact candidate inventory/seal/config checks, and old identity
exit checks. `VALIDATED_NEW` binds the app, child, build, launch IDs, and image
hash. The quit function checks the app identity/build/hash and consumes its
single allowance before native calls. Failure before this validation cannot
normal-quit the old app. Recovery retains the ten-second SIGTERM and fifteen-
second SIGKILL/exact-job-bootout delays. No second controller target launch or
shared-identity unregister was added.

**Freeze, source, approval, and evidence checks.**

Read the complete new README, responses, controller, proposal, diffs, input
snapshots, freeze manifest, static-check source/result, and raw reader/whitespace
records. Recomputed every freeze entry's size, mode, and SHA-256: all 16 entries
match, with exactly `freeze.json` excluded by its declared rule. All 17 files,
including the freeze, equal their bytes in the repair commit.

The four original snapshots equal their immutable Git objects in preparation
`9490a0e46eadf73785901b33d040d873b34e1157` and full R2 review
`48c41fbaca076fb46a028f3900265ac1a41e11ce`. Recomputed both exact unified patches.
All JSON parses, and both controller texts AST-parse. No AST was evaluated.
There are exactly two main `launch_once` calls, for the initial entry and CLI.
The proposal/controller hash and path bindings match.

The seven changed original proposal fields are `approval_scope`, `controller`,
`final_quit`, `limits`, `missing_prerequisites`, `owned_cleanup`, and
`supervisor_stop_limit`. Every other original field is equal. All original limit
values are unchanged; five new limit fields add the cutoff, recovery reserve, thread,
and quiescence limits. The seven added top-level fields describe deadlines,
ledger, review receipt, original inputs, repair provenance, and static validation.

The 451 source-hash records and all original artifact references are unchanged.
Recomputed the six evidence hashes and the original evidence-manifest hash from
preparation Git objects; all match. The original manifest remains
`2246f960192284303a240e07c43f48408ba53a72cbc40bdde5a4217d9f17d7f3`.
The full R2's 473-record artifact/evidence audit is used as audited context.
Its private artifact reads, signatures, deletion receipt, dependency inventories,
and build observations were not independently repeated. No private bundle,
marker/config, public-signature file, or key-deleted live state was re-audited.

Source and artifacts remain bound to
`ad280644a96905d9831825bb4f87d1388dbdca73`. The later daemon-stop fix and its timing
are not part of this candidate. The original supervisor's 1.5-second fallback,
receipts, nil PID, and shutdown return still do not prove child exit. Target and
daemon must actually measure nice 15. Official upstream helpers retain the
normal nice-0 exception. No shared process can be reniced or signalled.

The new approval gate binds both corrected hashes and a separately hashed
independent review receipt with `result:clear_for_approval`, no unresolved
findings, and `independent_review_complete:true`. This document has two unresolved
P2s and cannot supply that clearance. No JSON review receipt or approval receipt
was created. Corrected files need fresh hash bindings and another independent
review before the root can present them for an execution decision.

New explicit running/relaunch approval remains missing. The 2026-10-23
17:16:55 PDT approval covers completed stopped cases only. Live preflight must
still establish immutable source/artifact/mode/ownership bindings, a fresh
stopped private target, absent fixture processes/jobs/caches/receipts/socket,
and one exclusive bind to literal `127.0.0.1:56327`. The six-hour marker remains
frozen and expires 2026-10-24 00:56:25.164527 PDT. It must have at least 240 seconds
remaining at the declared preflight check. Expiry blocks this operation; it must
not cause renewal, a new key, another port, or environment repair.

Host effects remain disclosed: exact fixture launchd jobs/caches,
LaunchServices registration/timestamps, normal Gatekeeper/gktool/XProtect work,
and unknown persistent shared state. Private HOME does not prove confinement.
There is no full restoration, zero-effect, release-trust, or publication claim.
The observer's nominal 50 ms wait plus scan time can miss brief processes.
Sampled product logs and post-capture OS-output checks are not strict peak
memory/disk bounds. Independent exec/LS/NSWorkspace evidence and real runtime
readiness, handoff, exit, and cleanup remain unverified.

**Ownership and verification limits.**

This task owns only this new document in
`/Users/serge/.t3/worktrees/openergy/t3-37c4f272`, branch
`t3/review-running-update-controller`, initially clean at
`7328df364908fa237cdd242fc0d5ea950ad126a6`. The repair and all prior evidence stayed
read-only. The initial broad read exceeded the tool-output limit, so the source,
diffs, and prior context were read in bounded slices. A `wc` call included the
two directory operands and reported those as directories. Neither read changed
a file. The independent static hash/AST reader completed without a failed check.
The supplied static-check program was read, not run, because its entry point
writes records in the repair folder.

Only source/file/Git reads and static hash/JSON/AST calculations were used before
authoring this document. No controller, preflight, helper, process/port/registry/
log query, compiler, product, daemon, driver, server, installer, signing tool,
key command, simulation, unit test, mock, browser, desktop, or computer-use action
ran. No key, secret, credential, or private signing material was accessed. No
installed app, live sharing, production config/TLS/default IPC, TCC, login item,
service, task, account, firewall, global DNS, or toolchain default changed. No
TCP24800/UDP5353 traffic, Docker startup, push, publication, or delegation occurred.
The three provider-stopped tasks remain out of scope. This source review is not
independent runtime reproduction.
