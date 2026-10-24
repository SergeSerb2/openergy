# Running-update R4 correction r2: DO-NOT-RUN

This is a narrow source-only response to the two P2 findings in R4
`cf8127d6da7710adfbf04f07ef8bcdd95e782edf`. The previous repair is
`82cb25b6775d69fe1a312bdc1f26486aba0bf287`. This handoff supplies no independent
review clearance, execution approval, live preflight, or runtime acceptance.

Owned worktree: `/Users/serge/.t3/worktrees/openergy/t3-6133e534`, branch
`t3/reconcile-sparkle-ledger`, initially clean at
`80988f51d55eb8be253017c12ee4b0a5d61c4b45`. Only this new versioned folder is owned.
No prior controller, proposal, raw record, or private artifact was changed.

The [controller](controller-DO-NOT-RUN.py.txt) is review text. The
[proposal](operation-DO-NOT-RUN.json) remains `DO-NOT-RUN`,
`approval_received:false`, `new_independent_review_received:false`, and
`runtime_accepted:false`. No review receipt or approval receipt was created.
The parent must obtain a fresh independent review of these exact files before
asking the user to approve the concrete running/relaunch operation.

Frozen SHA-256 identities:

- Corrected controller: `a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567`.
- Corrected proposal: `cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d`.
- Previous controller: `3d08b11c2dda9256314d6bee0f7da7c7651cd0ea45b3f8dc7f8d3a9b1f0b1066`.
- Previous proposal: `667c5ef9a362c3319390f0e5e77adc19e7364a34425074edfd08731158774c6b`.
- Previous freeze: `2e66b88376fb2594125d6b02331ac42fd5cd991ae86b180533d363a07749a083`.
- Full R4 review: `05173d072fbafb3adcad67548f38f96aa9d43448c94e88e730b8e909475921f4`.

[freeze.json](freeze.json) binds every regular file in this new folder except
the root freeze itself. Git binds that manifest. [inputs/repair-r1](inputs/repair-r1)
contains all 17 previous files, including their inputs, diffs, static records,
and first failures, byte for byte from the previous repair commit. The four
original preparation/R2 inputs are also copied at [inputs](inputs) to preserve
the existing controller's input-hash contract. The full R4 is copied as
[inputs/independent-R4-review.md](inputs/independent-R4-review.md). Both reviews
and the previous responses were read. Old inputs are evidence, not approval.

## Narrow corrections

R4-1 uses the existing reentrant ledger lock across each full identity read and
its immediate merge. Parent reads already occur under that lock and now record
all scoped parents before role validation. Bulk process results and stored ready
rows are never replayed into the ledger. The same-PID ENTRY to IMAGE transition
still counts once and saves explicit before/after evidence. Reverse and other
executable transitions still fail. UID, start, path, priority, parent, restart,
and identity-count guards remain. No second acquisition lock or sequence clock
was added. This avoids a new ledger/acquisition lock-order pair.

R4-2 separates observed scope from permission to act. `scoped()` includes
helper-only and rejected rows. `owned()` remains restricted to the existing
target/daemon/driver paths and UID; each signal still needs its full independent
identity, canonical path, ownership, and frozen hash checks. Unknown helpers are
never adopted or signalled. Every retained scoped identity receives an
independent exited/live/unconfirmed outcome. Failed reads and cap overflow have
explicit unconfirmed evidence and make scope incomplete; they cannot yield a
clean result or enlarge the 32-identity ledger.

Cleanup reconciles scope after the quiet interval, after OS capture and before
cache work, after inventory, between individual cache deletion steps, and after
each cache. A fresh live read during the cache phase latches a permanent block
on further removal, even if that process later exits. The ledger lock covers
each current-state/latch check and one unlink/rmdir. Relative directory
descriptors with `O_NOFOLLOW` refuse substituted interior symlinks. A failed
check stops removal and retains remaining state and evidence. It cannot restore
entries already deleted, and it makes no rollback claim.

After the main path and OS readers quiesce and the observer joins, recovery
takes a fresh scoped scan and reconciles all final ledger rows. A live or
unconfirmed identity, incomplete scope, late cache-phase observation, or
observer/ledger failure sets an explicit cleanup failure and retained-state
result before `RECOVERY_DONE`. Main disarms the whole timer only after recovery
completion and join. A deadline/quiescence failure retains the existing fixed
terminal failure path. These are source changes, not reproduced behavior.

[responses.md](responses.md) maps both findings and the preserved R2 fixes.
[controller.diff](controller.diff) and [proposal.diff](proposal.diff) are exact
unified patches against the previous repair. Only five previous proposal fields
change: controller binding, ledger description, whole-deadline description,
cleanup description, and the missing independent review prerequisite. One new
`R4_correction` provenance field binds the previous files and full R4 review.

## Unchanged operation, artifacts, and limits

The operation still allows one initial private no-argument target launch, one
official external updater invocation and its own no-argument relaunch,
independent old-app/all-observed-old-child exit plus candidate readiness, and
one normal quit of the validated NEW app only. No old-app controller normal
quit, extra launch, helper adoption, shared-process action, or retry is added.
The persistent ledger still rejects old-parent replacement children. IPC/HTTP
keep their absolute deadlines and all request, byte, route, and frame caps.

All limit values are unchanged: 180s control plus 60s recovery inside 240s;
10s before controller SIGTERM; 15s before SIGKILL or exact-job bootout; 12 active,
32 total, two app and two daemon identities; four observer IPC requests; four
bounded-lifetime background threads. All action/reader/quiescence guards remain.
Target and daemon must measure absolute nice15. Official upstream Sparkle
helpers retain their normal nice0 exception. No shared process is reniced.

Source and artifacts remain frozen to
`ad280644a96905d9831825bb4f87d1388dbdca73`. No later daemon-stop fix or timing is
attributed to these artifacts. The original six-hour marker expires
**2026-10-24 00:56:25.164527 PDT**. It needs at least 240s remaining at the stated
future preflight check. There is no renewal, new key, re-signing, port fallback,
private file change, or environment repair. The sole port remains literal
`127.0.0.1:56327`, with one bind and no address reuse. No live state is asserted.

Fixture launchd jobs and exact cache roots, LaunchServices registration and
timestamps, normal Gatekeeper/gktool/XProtect work, and unknown persistent
shared state remain disclosed. Private HOME does not prove OS/helper
confinement. There is no restoration, zero-host-effect, release-trust, signing,
publication, or standard visual/manual AppUpdater acceptance claim.

The nominal 50ms observer wait plus scan time can miss brief processes. Final
capture cannot prove that no future kernel event will occur. That limit does
not excuse ignoring an identity actually recorded. Sampled product logs and
post-capture OS output limits remain distinct from strict peak resource bounds.
Independent exec/LS/NSWorkspace evidence, actual readiness, handoff, exits,
cleanup, priorities, and deadline timing remain unverified at runtime.

## Static checks and preservation

[check-static-files.py](check-static-files.py) reads files and committed Git
objects, parses JSON/AST, checks exact input and diff bytes, and compares syntax.
It never imports, evaluates, compiles, or executes controller code. Its
[result](static-checks.json) records 17 exact prior files, 16 prior freeze
entries, four exact original inputs, the exact R4, unchanged proposal limits
and host/fixture fields, six original evidence hashes, and 902 source-blob hash
comparisons for the 451 frozen source files at preparation and artifact root.
It also records unchanged deadline/HTTP/IPC/launch/quit/signal/main ASTs and a
source line map. Static guard checks are not behavioral or independent review
proof. No unit test or mock was written.

The initial broad listing/truncated reads and wrong context filename are
preserved as tool-result transcriptions in [raw/initial-file-read-notes.json](raw/initial-file-read-notes.json).
Prior raw failures remain in the exact previous snapshot. Authored whitespace
checks exclude untouched inputs and standard patch context lines. No private
artifact, cache, marker/config/nonce, signed ZIP/feed, or keys-deleted live state
was re-audited or changed.

No controller, preflight, product, daemon, launcher, updater, helper, simulation,
test, mock, watcher, live process/port/log query, build, signing, key, network,
desktop, or computer-use action ran. No installed app, live sharing, production
config/TLS/default IPC, TCC, login item, service, task, account, firewall, global
DNS, or toolchain default changed. No TCP24800/UDP5353, Docker, push, publication,
secret/credential access, or further delegation occurred. The three
provider-stopped tasks remain out of scope. Source and saved-evidence checks
are not independent runtime reproduction.
