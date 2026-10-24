# Independent static review of ordinary controller R9

Date: 2026-10-24, America/Vancouver.
Candidate: `e34401c8842c2a93b5c355f4c515a79ee05cfa72`, imported by `6e7b595`.
Reviewer checkout: `/Users/serge/.t3/worktrees/openergy/t3-3f0af888`,
initially clean at `a2a83194efbab6559aacb30def21b8b9890027c3`.

**Result: no concrete actionable P1/P2 finding in the bounded source scope.**
R8-P2-1 is addressed in this exact source revision. The R6 receipt-I/O fallback
repair and all seven R4 repairs remain present. This result comes from an
independent source reading and data-integrity checks. It does not inherit the
implementation verdict. Runtime remains **UNVERIFIED**. This document grants
no runtime clearance, executable approval receipt, or release acceptance.
The ordinary flow remains already authorized and needs qualified inputs.

All candidate locations below are relative to
`docs/verification/ordinary-controller-uncertainty-handoff-r9-2026-10-24-t3-79efd32e/`.

**R8-P2-1 disposition: addressed in source across the reviewed Inventory lifetimes.**

`ownership.py:59-75,87-105,229-253` starts every Inventory without absence
readiness. Only the main controller initializes the journal, before watchdog
or product launch (`controller.py:429-437`). Every full observation locks,
reloads ownership records, restores the journal, writes/fsyncs a begin, runs
the existing probe, and checkpoints state before releasing the lock. This
applies to the main observer, watchdog Recovery, parent fallback Recovery and
the cleanup-launch Inventory (`controller.py:218-228,342-356`;
`watchdog.py:15-18,202-214`). Constructors do not create replacement journals.

For R8's detected private daemon, the unresolved PID/start and original pending
time/count reach the next observer before its probe (`ownership.py:136-204,
272-286`). PPID 1 and a failed next path read cannot remove that token from
selection. The inherited separate probe partition queries missing PIDs and
requires explicit ESRCH for resolution. Reuse cannot clear the token or
authorize replacement argv. Local uncertainty handling remains at
`ownership.py:308-358`; `absent` rejects unresolved state, pending state and
overflow at `503-513`. Expected finite validation errors still checkpoint the
completed state at `365-367,242-246`.

The registered-start index also blocks a successor that lacks independently
available ownership for an earlier registered start (`ownership.py:189-203`).
It cannot reconstruct ownership from metadata. Pending known-owned/direct
starts retain query-only tokens before a receipt can be lost (`214-227`).
The journal carries no role, executable, argv, parent chain, UID or group.
It grants no detail, adoption or signal authority.

**Failure, concurrency and budget audit.**

- **Interrupted or unavailable handoff:** the begin precedes the probe.
  Incomplete processing sets loss before its checkpoint. Death or partial
  writes can leave an unmatched begin or invalid line; restore rejects these.
  Missing/unreadable lock or journal, invalid shape/sequence, cap errors and
  short writes cannot initialize empty replacement state. Lost and overflow
  flags remain sticky in the affected Inventory and in committed successor
  state (`ownership.py:125-204,229-253`; `common.py:154-198`). A complete
  checkpoint that reached the file before an I/O exception still carries the
  detected state. These are source branches, not executed failure injections.
- **Locks and short deadlines:** the uncertainty lock uses nonblocking flock
  and the existing absolute deadline. The file context closes on normal exit
  or exception. Its nested paths take the byte or observation-budget lock;
  neither existing lock path acquires the uncertainty lock in reverse order.
  The enclosing observation/recovery alarm still bounds restore, parsing,
  writes and fsync. No new timer or deadline extension was introduced
  (`ownership.py:107-134,229-253`; `common.py:42-64,103-114`;
  `watchdog.py:194-214`). No lock or timing behavior was executed.
- **Original allowances and capacity:** restore replaces stale local pending
  state with the committed original monotonic start and attempt count. The
  partial-row, race and unresolved paths share `note_pending`; an unresolved
  row/race already charged in that full observation is not charged again
  (`ownership.py:174-180,214-227,312-358`). Resolution can remove pending state;
  handoff itself cannot restart it. The eight-observation/two-second failure
  predicates remain. The pending/unresolved union has a 24000-PID ceiling;
  restore also bounds the registered-start list and combined PID union.
  No entry is evicted to establish absence. Sequence and line ceilings limit
  the journal to 6000 transactions and 12001 lines. After sequence zero, loss
  of the existing bounded observation counter blocks restore before a new
  probe. The original 6000/24000 and main 4976/19904 counter rules remain.
- **Byte reservations and partial I/O:** journal appends use the unchanged
  common writer, including the locked pre-write charge, short-write check,
  retained failed charges and 8 MiB file cap. Main writes use regular space;
  watchdog/fallback/cleanup-launch writes use nonterminal cleanup space.
  They cannot spend the terminal-only MiB. The journal does not get a new
  reserve. A full checkpoint or repeated reads can exhaust byte/time capacity
  before all nominal count maxima fit. The design explicitly permits bounded
  failure for this case; that is not an additional P2 or a reason to expand
  limits (`ownership.py:96-105`; `common.py:117-198`).
- **Failure-first recovery:** observation/restore errors enter the existing
  failure and abort boundary, then targeted recovery (`watchdog.py:205-214`).
  Targeted recovery reloads known receipts, retains fresh data/identities
  before publication, and independently attempts verified app/child cleanup
  (`107-152`). It does not restore uncertainty as ownership, grant first peer
  permission, or certify final absence. The existing peer-recovery action
  after an earlier valid full permission remains unchanged. Journal loss can
  leave the peer retained and end in failed bounded recovery; this is an
  explicit fail-closed limit, not a clean-recovery claim.
- **Stale decisions and lock release:** `absence_guard` reloads/restores under
  the same lock and requires the observation's exact checkpoint sequence.
  An active writer must release that lock first; a newer revision rejects
  the old decision. The lock stays held through watchdog permission/flag
  publication (`watchdog.py:243-251`) and through the cleanup-launch app-tree
  absence check and wrapper-gate write (`controller.py:225-228`). A stale
  receipt alone cannot release that wrapper. `Inventory.final` uses the same
  guard (`ownership.py:515-533`). Exceptions leave local absence unavailable.
  The next existing recovery advance can refresh its observation within the
  original budgets. This is not a guarantee about future undetected processes
  after lock release or an atomic kernel identity lease.

**R6 finding dispositions.**

- **R6-P2-1:** the R7 detection marker, metadata-only unresolved partition,
  staged-read privacy and explicit ESRCH handling remain byte-exact in
  `macproc.py` and `probe.py`. R9 supplies the missing inter-Inventory handoff
  described by R8. Unknown orphans remain unowned.
- **R6-P2-2:** `Controller.cleanup` remains AST-exact to R7. It polls handles
  first, isolates terminal lookup/load/parsing errors, detects actual watchdog
  loss from retained exit/handle state, assigns one fallback before its loss
  receipt, and steps that fallback outside receipt I/O (`controller.py:321-356`).
  It uses the original request/deadline and `normal=False`. A missing receipt
  alone does not establish watchdog death. The terminal producer still needs
  final absence and the exact closed direct-exit ledger (`watchdog.py:169-192,
  263-290`). No receipt failure or fallback behavior was run.

**Separate preservation assessment of all seven R4 repairs.**

1. **Actual Homebrew/Python.app argv0:** actual-interpreter wrapper argv0 and
   exact remaining tokens remain at `ownership.py:49-55,402-417`. Launch-path,
   resolved launcher/hash and actual executable/hash remain distinct at
   `controller.py:178-182,422-425,444-450`. Product argv checks remain exact.
2. **Staged executable/argv and reuse privacy:** `macproc.py:85-105,126-146,
   168-190` remains byte-exact to R7. Coherent staged exe/argv and final BSD
   identity checks precede publication. Reused starts expose metadata only;
   the uncertainty journal does not enter the owned/direct detail partition.
3. **Evidence cannot block retained exits, abort or draining:**
   `Controller.drain`, `collect`, `abort`, `request_cleanup`, `observe` and
   `tick` remain AST-exact to R7/R5. All handles are polled into memory before
   independent writes, abort state precedes evidence, the request pipe remains
   bounded/nonblocking, and failed streams drain/discard. The R6 fallback
   boundary above remains intact. Evidence failure still fails acceptance.
4. **Pre-spawn intents and exact sealed terminal sets:**
   `controller.py:183-201,107-111,333-336` and `watchdog.py:169-192` retain
   intents before spawn, handles before post-spawn writes, one launch per role,
   and agreement of sealed intent/spawn/handle/PID/actual-exit sets. A failed
   cleanup wrapper gate retains its handle/intent and cannot create a second
   launch or vacuous recovered:true.
5. **Narrow same-start exec after controller loss:** `Inventory.identify`
   and `Recovery.targeted` remain AST-exact. The prior exact wrapper gate,
   role/start/group/UID/NI15/interpreter/argv and live matched controller or
   explicit controller ESRCH still govern the direct product transition
   (`ownership.py:390-424`). Metadata or PGID membership cannot adopt an orphan.
6. **Five seconds from successful TERM:** `Inventory.signal`/`_signal`,
   `Recovery.stop` and CLI deadline handling remain AST-exact. Successful
   single-PID delivery is retained with its time under the small alarm mask;
   KILL needs the full grace and a later advance (`ownership.py:446-501`;
   `watchdog.py:56-105,154-167`). Insufficient remaining time remains failure
   inside the same cleanup deadline. No signal or grace timing was tested.
7. **Cross-process byte caps and rename-only archive:** `common.py`,
   `exec_owned.py` and `prepare.py` remain byte-exact to R7. The 32 MiB total,
   8 MiB file, eight 2 MiB product slots, 4 MiB cleanup including 1 MiB terminal,
   and 64 KiB accounting allocations remain. Failed charges remain retained;
   existing first receipts short-circuit. Reserved archival remains a
   same-filesystem rename after verified recovery (`controller.py:482-498`;
   `common.py:216-228`). Journal bytes use the existing writer and reservations.

**Exact bounded read and static integrity scope.**

I read all eight complete R9 source texts (2059 lines), the complete README,
design, operation plan, responses, 251-line/17-hunk patch, input bindings,
source-checks, manifest and static-reader notes/first error. The complete
original index was parsed as data. I read the full R8 review at
`172ee851d99bf1c256235b23fb445cae899332ff`, full R6 review at
`4eaa964eab6f74b2fc5704a1925520866c7d9f9c`, full R4 review at
`cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d`, and all seven R5 responses at
`ec723d8fde7c65e955c015653f89d24f7cc7f091`. Their local bytes matched the exact
bound Git objects. Prior conclusions did not substitute for this review.

A new inline standard-library data reader independently confirmed:

- All 19 R9 package files equal both exact candidate and imported Git bytes.
  All 18 manifest entries match size/SHA-256; all four package JSON files parse.
  All eight sources parse as AST data, with their recorded sizes/line counts/
  SHA-256 values. No supplied AST was evaluated.
- The five claimed unchanged modules are byte-exact to R7. All 72 named
  R9/R7 unchanged function AST comparisons pass without location attributes.
  All 31 R7/R5 protected claims also pass; R9 retains 29 verbatim. The two
  controller functions compare exactly after removing only the declared
  journal initialization/cleanup gate changes. Watchdog `advance` compares
  exactly after removing only its new permission guard.
- All 17 patch hunks independently reconstruct from exact R7 Git bytes in
  memory. Removed bytes, both positions and counts agree; all three results
  equal R9. No patch was applied or source copy materialized on disk.
- All bound R3/R4/R5/R6/R7/R8 inputs (14/1/18/1/16/1 files) match immutable
  Git blob IDs, sizes, SHA-256 and this checkout's bytes. The R5/R7 manifest
  entries also match. The retained prepared JSON matches its original Git
  bytes; this establishes no current payload availability.
- All 87 indexed originals match immutable
  `498190feb00f51039f500ae618ac7de5fb113d86` by blob ID, size and SHA-256 and
  equal this checkout's files. The copied original index is byte-exact to R5.
  Other worktrees, private retained payloads and current runtime paths were
  not inspected. No 421-input, signed-bundle, SDK or native ABI audit was repeated.

Independently confirmed SHA-256 values:

- `controller.py`: `eed1d018cf0e963c41d40a11e148b8b4c4d3a395b140581e8782a5333ef54b39`
- `ownership.py`: `ef5400c711fc80f24a9cbf0d8cf0bc31ff2f237b6820a3ea7d0f8aba4d9a1a4a`
- `watchdog.py`: `b8ee8f6c7a9036f0e263301858996a62aece617bbcc11146540cf08c867b0e8f`
- `repair-r7-to-r9.patch`: `0eea14306d58dc427abf44a525da328c0ff92c34349e8c90bfb0d947dbf82184`
- `manifest.json`: `b92e0285a6f4e9a9d6a8bfc26248febba2023cd8c12221738729e595d7f161a8`

**Saved result, missing prerequisites and proof limits.**

I read the saved original outcome, first controller and cleanup tracebacks,
complete 25-step recovery report and predeclared limits. The original remains
**FAILED / NOT ACCEPTED, 23/25** after the bad shared-PGID assertion and early
peer stop. App OS exit and full late/recovered priority coverage remain missing.
These records establish no product defect and supply no fresh runtime proof.

I read the input-retention README and verified its bytes at `1ea63e55`:
`docs/verification/ordinary-input-retention-2026-10-24-eb2370f8/README.md`.
It reports a matching compiled daemon and retained receipts, but no exact signed
bundle, compiled app, complete pinned Sparkle tree, original icon or raw Swift
workspace-state file at its checked named locations. The compiled daemon is
distinct from the historical signed daemon. This is a bounded prior retention
result, not a fresh search or a claim about all locations. Missing qualified
inputs are a separate prerequisite, not a source P2 or new human permission
gate. No payload was probed, restored, substituted or reassembled here.

The reviewed plan retains one ordinary no-render `--e2e --require-peer`
default25 flow and one launch per role, app/children before peer, 120 s flow,
60 s cleanup and fixed start + 180 s end. Keep 6000 rounds/24000 rows including
1024/4096 cleanup reserve, eight observations/two seconds, all byte allocations
above, 16 MiB IPC frames, 1,000,000-byte stalled write, 300 ms request,
1 s responsiveness, 2 s stalled bound, 3/5/10/15 s waits and 1.5 s supervisor
stop. Private literal loopback/virtual/discovery-off/port-24800-denial settings
remain required before startup. Rust 1.99.1/jobs2/deployment15 remain inherited
lineage only. No predicate or limit was relaxed.

Actual ABI/process identities, NI15 and stop/restart coverage, all direct OS
exits, 25 unchanged passing checks, unchanged signed bundle bytes and complete
cleanup remain **UNVERIFIED**. This review does not demand universal unobserved
child proof or an atomic OS identity lease. It creates no executable approval.

Only this new review document was written. The candidate, prior evidence,
guards, originals and production source remain read-only. Checks used own
file/Git/hash/JSON/AST/patch data readers only. No supplied module, helper,
checker, controller or task AST was imported, executed or evaluated. No unit
test, mock, simulation, synthetic process/interleaving, build, compiler,
preflight, fixture preparation, product/helper launch, signing, process/OS/
port/job query, remote connection, desktop action, credentials/private-key/
production-seed read, delegation, push or publication occurred. Repository
hooks and commit signing are disabled for the review commit.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login,
services/jobs/accounts, firewall, global DNS and toolchain defaults remain
untouched. No TCP24800, UDP5353 or Docker operation occurred. Crafted mDNS,
original provenance/security, IPC/Synergy3 budget, isolated Swift build and
causal bind-trace gates stay stopped. The expired closed Sparkle one-shot
has no retry, renewal, reset or approval reuse.
