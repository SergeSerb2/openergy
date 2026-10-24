Independent R6 static review of the running-update R4 correction

Reviewed on 2026-10-23, America/Vancouver. Result: **clear for approval of the
concrete operation; no actionable P1/P2 finding**. All five prior findings are
addressed in the reviewed source. This is source clearance for the parent's
approval presentation. It grants no execution approval, live-preflight pass,
integration approval, or runtime acceptance.

The candidate is unmerged commit
`95c59a27be0f9c95ed207eac4180193a493bb5f2`. The reviewed folder is
`/Users/serge/.t3/worktrees/openergy/t3-6133e534/docs/verification/sparkle-running-repair-2026-10-23-t3-6133e534-r2/`,
called `CANDIDATE` below. Controller line references refer to the exact
`controller-DO-NOT-RUN.py.txt` in that folder and commit.

Exact cleared SHA-256 identities:

- Controller: `a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567`.
- Proposal: `cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d`.
- Freeze manifest: `ecab76f52af1fafdf5b7299cead6a799548ace0b6943a816808bd1424b2e4fb2`.
- Full R4 input: `05173d072fbafb3adcad67548f38f96aa9d43448c94e88e730b8e909475921f4`.
- Previous controller: `3d08b11c2dda9256314d6bee0f7da7c7651cd0ea45b3f8dc7f8d3a9b1f0b1066`.
- Previous proposal: `667c5ef9a362c3319390f0e5e77adc19e7364a34425074edfd08731158774c6b`.
- Previous freeze: `2e66b88376fb2594125d6b02331ac42fd5cd991ae86b180533d363a07749a083`.

**R4-1: acquisition/merge ordering is corrected in source.**

The full identity read at lines 213-216 occurs inside the existing ledger
RLock, held until its immediate merge completes. Static AST inspection finds
only two calls to `raw_identity`: line 213 and the parent read at line 280.
The parent read occurs within `observe`'s same RLock. Its scoped result is
added immediately, before parent-role validation. The sole call to `observe`
is line 216. No scan-result or stored-ready replay remains.

This covers `launch_once`, direct main/readiness/environment/exit reads, the
persistent observer, recovery signals and scans, and final reconciliation.
The preliminary path read in `fixture_processes` is a filter. It does not
create a full identity for later merge: the code rereads the full identity
under the lock and checks that it is still scoped. An unresolved filter match
is saved as unconfirmed and fails scope completeness. It is not accepted as
a fresh full identity.

An observer's ENTRY read therefore cannot wait outside the merge lock while
a main or parent read merges IMAGE first. The actual acquisition and merge
order is serial. ENTRY to IMAGE keeps one PID/UID/start key and saves explicit
before/after forward-exec evidence at lines 265-266. Reverse or other observed
scoped exec changes still raise `unexpected exec`. Rejected rows are retained
for reporting, but the ledger fault persists. A later valid read cannot clear
that fault. An out-of-scope current executable cannot satisfy `same` for an
exit, quit, or signal check against the prior fixture identity.

`match_ready` at lines 312-319 compares the stored ready identities with the
already acquired ledger identities and validates their roles; it cannot
reinsert older rows or reset counters. Main's later validation uses that same
check. The native quit also requires a fresh matching app identity and exact
build/image hash. Cached readiness does not authorize a signal to another
process. PID reuse or an unexpected current path fails independent checks.

The PID/UID/microsecond-start key, 32-row cap, two-app/two-daemon limits,
initial PID binding, parent associations, old-parent replacement rejection,
and target/daemon nice15 checks remain. The role/count validator is AST-equal
to the previous repair. The 12-active check remains in the persistent observer.
Rows rejected before entering a scan's returned list still cause a permanent
failure; excluding them from that list cannot yield successful acceptance.

No second acquisition lock or opposing lock order was introduced. Launch and
validation can take ACTION_LOCK before the ledger lock; ledger paths do not
take ACTION_LOCK or IO_LOCK. Evidence and error code do not acquire the ledger
lock while holding SAVE_LOCK or ERROR_LOCK. Reentrant ledger calls, including
parent reads and reconciliation, use the same RLock. No static lock cycle was
found. This is not an executed concurrency or scheduling result.

**R4-2: final observed scope and cache guards are corrected in source.**

`scoped` at lines 325-327 includes helper-only and rejected full rows. `owned`
at lines 321-323 separately restricts recovery candidates to the current UID
and the four existing executable paths populated from frozen target/daemon/
driver inventories. Observation grants no new action permission. Helper-only
rows are never passed to direct process signals. Exact-job bootout still needs
the fixed label, delay, canonical UID-owned program under an allowed root,
and frozen helper hash. Unknown helpers are neither adopted nor signalled.

Every new retained identity is saved before validation. At the 32-row cap,
overflow has a separate unconfirmed outcome and sets `scope_incomplete`; it
does not enlarge the ledger. Incomplete identity reads also save unconfirmed
evidence, mark incomplete scope and latch failure. A full-scan error likewise
makes reconciliation incomplete. These paths cannot silently report clean
cleanup, including when the ledger itself has no remaining live rows.

`reconcile_scope` at lines 398-425 takes a fresh scan and independently checks
all retained scope with `gone`. It processes any new parent or identity added
during those checks. Exited requires libproc absence plus ESRCH. A live match
is reported live; reuse, an unavailable read, or an incompatible path is
unconfirmed. Neither a receipt nor a nil supervisor PID substitutes for exit.

Reconciliation follows the full five-second quiet interval. Cache work begins
only after the OS capture, CONTROL_DONE, reader checks and observer/ledger error
checks. `cache_started` is set under the ledger lock. From then onward any fresh
live scoped read permanently latches `cache_blocked`, even if that process later
exits. Incomplete scope and all-scope outcomes also block removal.

Each cache has a fresh check before inventory. Inventory is followed by another
check before deletion. `remove_cache` at lines 436-472 holds the ledger lock
across a fresh scan, outcome/latch/error checks and one unlink or rmdir. It
repeats that sequence between all destructive steps and checks after each cache
and the full cache loop. The observer can acquire the lock between steps; its
already merged rows cannot be omitted by the removal guard. A helper recorded
during inventory or between steps blocks further removal. A rejected row or
read failure cannot pass the guard merely because it is not signal-owned.

Root device/inode, current UID, directory type and canonical path are checked
again. Descriptor-relative traversal uses O_NOFOLLOW for the opened directory
components and checks ownership. It does not follow a substituted interior
symlink into another tree. The fixed cache roots, initial-absence requirement,
bounded inventory and refusal of unexpected types remain. A partial deletion
failure retains the remaining state and evidence; it makes no rollback claim.

In recovery's finally block, CONTROL_DONE and reader completion precede observer
stop/join. The HTTP thread must also be stopped. The fresh final scan and
all-scope reconciliation at line 861 occur after these writers quiesce, before
the result and RECOVERY_DONE. This covers a helper recorded after the last cache
check or during final work. Live/unconfirmed scope, incomplete scope, a latched
cache observation, or observer/ledger errors produce an explicit cleanup failure,
error and retained state. The result includes individual scoped outcomes.

Main waits for RECOVERY_DONE and recovery join before disarming the whole timer.
Failed quiescence, failed final reporting or deadline exhaustion cannot fall
through to clean acceptance: the guarded thread takes the fixed terminal failure
path, or the independent whole timer does so. It reports retained/unconfirmed
state and exits one. This is not a claim that controller self-exit kills products.

The code does not prove absence of future kernel events after its final capture.
It also cannot guarantee that a process starts outside a scan's PID snapshot
will be seen before the next destructive step. That disclosed sampling limit
does not discard a row already merged or an unresolved filter match. No new
actionable gap for an identity actually recorded was found.

**R2 R1: continuous restart accounting remains corrected.**

The ledger and observer start before the one initial launch and remain active
through readiness, blocking IPC/OS evidence, the updater handoff, final exit,
quiet recovery and cache work. Counts do not restart at readiness. The initial
launched PID fixes the old role. Independently matched child-parent identities
persist. A second old-parent child fails without using the candidate allowance.
Main checks old app and every observed old child independently, and recovery
keeps individual old-member outcomes even after failure. R4-1 and R4-2 now close
the ordering and completion gaps identified in the otherwise corrected ledger.

**R2 R2: absolute deadlines and bounded failure remain corrected.**

IPC uses one monotonic two-second deadline per exchange. The first includes
connect. It recomputes the remainder before send and every frame fragment and
checks it after decode. HTTP uses one connection deadline and recomputes before
each receive and each header/body send. Both clamp to control/whole deadlines.
The four status/config requests, 256/65,536-byte IPC caps, exact HTTP routes,
methods, counts, byte caps and no-retry rule remain unchanged.

Recovery starts on fault/control completion or the 180-second cutoff, with
60 seconds reserved within the unchanged 240-second ceiling. Cancellation,
the one-second action gate, owned socket interruption, direct OS-reader kill/
bounded reap, HTTP join and observer accounting across waits remain. The four
background threads and read-only command time/output limits remain. Long scans,
inventory/hash/file operations or evidence writes can fail to finish normally;
the independent whole timer does not take action, ledger or evidence locks.
It uses the fixed retained-state terminal path. No hard real-time OS guarantee
or strict peak-memory/disk bound is inferred.

Budget, remaining-time, I/O, reader, HTTP, hard-timer, launch and main ASTs are
equal to the previous repair. Recovery still waits at least 10 seconds before
controller SIGTERM and 15 seconds before SIGKILL or exact-job bootout. Signal
targets retain PID/start/UID/path, canonical ownership and frozen hash checks,
including another identity read after hashing. No shared action was added.

**R2 R3: validated NEW-only normal quit remains corrected.**

AST inspection finds exactly one call to `normal_quit`, in main at line 1054,
after independent build-2 app/child readiness, distinct PIDs and launch ID,
candidate inventory/seal/config validation, and old identity exits. VALIDATED_NEW
binds the app, child, build, launch IDs and image hash. The quit function checks
that binding, current app identity and candidate build/hash, and consumes its
single allowance before native calls. Recovery has no normal-quit call. Failure
before validation cannot normal-quit the old app. There remain exactly two
`launch_once` call sites: the initial no-argument private entry and one official
external updater invocation. Only Sparkle may perform its no-argument relaunch.

**Freeze and preservation checks.**

Read the complete current controller, proposal data, README, responses, both
full reviews, exact patches, static-check sources/results, freeze files and raw
records. All previous/original controller texts were AST-parsed as data and
reviewed through the complete patches against the current source. Duplicate
input copies were verified byte-equal. No supplied program was run or imported;
no AST was evaluated. The supplied static readers were read, not executed.

Independent file/Git/hash/JSON/AST reads establish:

- All 35 freeze entries match size, mode and SHA-256, with exactly the root
  freeze excluded. All 36 regular files equal their bytes at `95c59a27`; Git
  records mode100644 throughout. There are no symlink entries or extra files.
- All 17 prior files equal repair commit `82cb25b6775d69fe1a312bdc1f26486aba0bf287`.
  All 16 prior freeze entries and that freeze's hash match.
- The four original inputs equal preparation `9490a0e46eadf73785901b33d040d873b34e1157`
  and full R2 `48c41fbaca076fb46a028f3900265ac1a41e11ce`. Full R4 equals
  `cf8127d6da7710adfbf04f07ef8bcdd95e782edf`. Provenance hashes bind those files.
- Both generations of controller/proposal unified patches recompute exactly.
  All 13 current JSON files parse without duplicate keys. All six Python/text
  source files AST-parse. These counts include the frozen results/manifests.
- Exactly five prior top-level proposal fields change: `absolute_deadlines`,
  `controller`, `identity_ledger`, `missing_prerequisites` and `owned_cleanup`.
  The only added field is `R4_correction`. Nested changes are confined to the
  whole-deadline text, controller path/hash, ledger coverage/exit/lifetime text,
  missing prerequisites and cleanup text. The new controller binding matches.
- All original limit values equal the original proposal, and the entire limits
  object equals the previous repair. The 451 source path/hash records are equal
  across all three proposals. The six original evidence hashes and preparation
  evidence-manifest hash were recomputed from Git blobs and match.

The saved 902 source-blob comparisons and the prior artifact/signature/dependency/
build/deletion audits are audited context. They were not repeated as product or
artifact validation. No private artifact, marker/config/nonce, signed archive/feed,
public signature or key-deleted live state was re-audited. The recorded earlier
first failures and raw bytes are preserved. Initial combined review reads hit
tool-output limits; bounded follow-up reads covered the omitted text. The
independent static checks had no failed assertion. A quick memory search had
no relevant task hit; no memory-derived claim supports this review.

**Missing gates and unchanged host effects.**

The proposal still says DO-NOT-RUN, approval_received:false and runtime_accepted:false.
Its missing-review flags describe its frozen authoring state; this separate
document supplies the independent source result. No JSON review receipt or
approval receipt was created. The controller still requires a separately hashed
review receipt for these exact hashes, plus a separate explicit user approval
bound to the exact scope and receipt. The 2026-10-23 17:16:55 PDT approval covers
completed stopped-target cases only. Running/relaunch approval remains absent.

Live preflight remains missing: unchanged immutable source/artifact bindings,
modes/ownership, the fresh stopped private target, absent fixture processes/jobs/
caches/receipts/socket and one exclusive bind to literal `127.0.0.1:56327`, with
no address reuse or fallback. The frozen marker expires **2026-10-24
00:56:25.164527 PDT** and must have at least 240 seconds remaining at the stated
preflight check. Expiry blocks the operation. No marker renewal, new key,
re-signing, port change or environment repair is authorized by this review.

Source and artifacts remain `ad280644a96905d9831825bb4f87d1388dbdca73`, before the
later stop fix. No later shutdown timing is attributed to them. Marker/config/
nonce, artifact hashes, archive/feed signatures, public key and signing/deletion
descriptions remain equal to the original inputs. Target/daemon must actually
measure absolute nice15; official Sparkle helpers retain normal nice0. Exact
fixture jobs and eight cache roots, LaunchServices registration/timestamps,
normal Gatekeeper/gktool/XProtect work and unknown persistent shared state stay
disclosed. Private HOME proves neither helper confinement nor full restoration.
No trust, release-provenance or publication claim follows.

Both no-argument entries, Bundle.main/private environment at the load boundary,
real model/virtual Off child readiness, old app/all-old-child exit, actual external
Sparkle install/relaunch, new readiness, normal final quit, helper/cache cleanup,
priorities, deadlines and independent exec/LS/NSWorkspace observations remain
unverified at runtime. Standard visual/manual AppUpdater acceptance remains open.

Only this review document is owned. The T3-owned worktree is
`/Users/serge/.t3/worktrees/openergy/t3-bbbc0189`, branch
`t3/review-sparkle-running-repair`, initially clean at
`34587fefe1208d5d9d065cfff41df7bd1439bd07`. All candidate/prior files stayed
read-only. No controller/preflight/helper/watcher/process/port/log/registry query,
app/daemon/driver/updater/installer, build/sign/key/network action, simulation,
unit test, mock, browser or desktop/computer-use action ran. No secret or existing
private signing credential was accessed. No installed app, sharing, production
config/TLS/default IPC, TCC, login item, service, task, account, firewall, global
DNS or toolchain default changed. No TCP24800/UDP5353 traffic, Docker startup,
push, publication or further delegation occurred. The three provider-stopped
tasks remain out of scope. Source and saved-evidence review is not independent
runtime reproduction.
