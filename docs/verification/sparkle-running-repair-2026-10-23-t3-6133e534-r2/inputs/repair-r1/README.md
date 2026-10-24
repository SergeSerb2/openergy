# Running-update controller repair r1: DO-NOT-RUN

This folder owns the source/file-only response to the three P2 findings in
independent R2 `48c41fbaca076fb46a028f3900265ac1a41e11ce`. It changes the text
controller and proposal from preparation
`9490a0e46eadf73785901b33d040d873b34e1157`. It grants no execution approval.

The worktree is `/Users/serge/.t3/worktrees/openergy/t3-bd21f531`, initially clean
at `059f052f965bb942d104448b064a9d0e68b1fda1`. Only this unique new folder is owned.
The original preparation paths and the separate review worktree were read only.

The [new proposal](operation-DO-NOT-RUN.json) remains `DO-NOT-RUN`, with
`approval_received:false`, `new_independent_review_received:false` and
`runtime_accepted:false`. The [controller](controller-DO-NOT-RUN.py.txt) is text.
It was AST-parsed, never imported, executed, simulated, unit-tested or mocked.

Frozen SHA-256 identities:

- New controller: `3d08b11c2dda9256314d6bee0f7da7c7651cd0ea45b3f8dc7f8d3a9b1f0b1066`.
- New proposal: `667c5ef9a362c3319390f0e5e77adc19e7364a34425074edfd08731158774c6b`.
- Original controller: `e251cf7477ed9accf7c28e15de4cabfad50306e12c1934c9b37155a1f40b2740`.
- Original proposal: `2d23db1f735e607d70f92e29a8e0837425c4908bd935a3950ba60f458dc6f2a6`.
- Original independent R2 document: `87a14f6446ebe11d26fff5b9da9e53ae0ddbc4c1b1e3beff8c102a5003f36cd6`.

[freeze.json](freeze.json) binds all files in this handoff except itself.
The complete original README, controller, proposal and independent R2 review
are byte-exact snapshots in [inputs](inputs). The exact changes are in
[controller.diff](controller.diff) and [proposal.diff](proposal.diff).
[responses.md](responses.md) gives a short response and change map for each P2.

## Operation and approval boundary

The operation still permits one initial no-argument private target launch,
real AppModel and virtual Off daemon readiness, one pinned external Sparkle CLI
invocation, and Sparkle's own NSWorkspace no-argument relaunch. Old app and every
observed old child need independent exit observations. The new exact candidate
app and its child need separate readiness observations. Only the validated NEW
app may receive the single controller normal quit. Owned recovery is finite.
There is no retry or second controller target launch.

The new control cutoff is 180s. It reserves 60s for recovery inside the unchanged
240s whole-runtime ceiling. A persistent identity observer runs before launch
and through readiness, blocked calls, handoff, final exit and quiet recovery.
A separate recovery thread closes active I/O and closes the control-action gate.
A separate whole-deadline timer stops all controller threads with a fixed
terminal failure record if recovery cannot finish. Unquiesced control or reader
state is retained and reported. Recovery never normal-quits the old app.
These are source mechanisms, not reproduced timing results or hard real-time
kernel scheduling guarantees.

The next prerequisites remain missing:

1. A new independent source review of the exact corrected files and fixture
   bindings, plus a separate review receipt. The prior R2 has three unresolved
   findings and supplies no clearance for this correction.
2. New explicit user approval for the exact running/relaunch operation, bound
   to both new hashes and the hash of that independent review receipt.
   The 2026-10-23 17:16:55 PDT stopped-case approval does not apply.
3. Live preflight of source/artifact hashes, modes/ownership, unchanged marker,
   fixture process/job/cache/receipt/socket absence and exclusive port ownership.

No review receipt or approval receipt was created here. The parent will obtain
the next independent review before any approval request or execution.

The six-hour marker is unchanged. It expires **2026-10-24 00:56:25.164527 PDT**.
The sole port remains **127.0.0.1:56327**, with one bind and no renewal, fallback,
or address reuse. No claim is made that the marker or live state is valid now.
Expiry must block the operation, never cause a new marker or repaired state.

## Frozen artifacts and proof limits

Root and both candidate artifacts stay bound to
`ad280644a96905d9831825bb4f87d1388dbdca73`. The later stop fix and its timings were
not imported. No build, signing, key use, packaging or private artifact write
occurred. Bundles, signed ZIP/appcast, inventories, marker/config/nonce,
keys-deleted state, original controller/proposal and prior raw records were not
modified. Private artifact state was not re-audited as live preflight here.

Every original limit remains unchanged. This includes the four observer IPC
requests, frame/byte caps, fixed HTTP routes/method counts, 12 active and 32 total
observed fixture identities, two app/two daemon identities, 10s before recovery
SIGTERM, and 15s before SIGKILL or exact job bootout. New deadline/observer fields
describe the correction. Target/child priority must measure absolute nice15.
Upstream Sparkle helpers retain their normal Nice=0 exception. No shared process
is reniced, signalled or unregistered.

The observer waits nominally 50ms between scans. Scan work adds time, and brief
processes may be missed. This repairs accounting for identities actually
observed; it does not provide complete kernel process coverage. Product log
ceilings remain sampled. OS output limits remain post-capture checks. Neither
establishes a strict peak memory/disk bound. Receipts, nil PID, CLI exit,
shutdown return and termination-request acceptance do not prove exit.

Host effects still include exact fixture launchd jobs/caches, LaunchServices
registration/timestamps, normal Gatekeeper/gktool/XProtect checks and unknown
persistent shared state. Private HOME does not prove OS/helper confinement.
There is no zero-effect, full-restoration, trusted-release or publication claim.

## File-only checks and preservation

[static-checks.json](static-checks.json) records AST parsing and static
file/JSON/Git-object checks. The reader checks all 451 frozen source hashes in
both the preparation commit and declared old source root: 902 blob comparisons.
It verifies original input bytes, six original evidence hashes, manifest hash,
controller/proposal bindings, original limits and fixture fields, exact patches,
and syntactic launch/quit call sites. It never evaluates the controller AST.
The [reader source](check-static-files.py) performs file and Git-object reads.
Its success is metadata consistency, not behavioral or independent review proof.

[First reader/edit records](raw/first-reader-and-edit-errors.json) preserve the
initial filename-search exit, wrong README Git path and truncated reader output.
The wrong-path stderr is a transcription from the tool result, not a recovered
redirected stream. The first AST and static reader checks passed; no source-edit
failure occurred. Existing failures and raw bytes were never rewritten.
Unified patches retain their standard space-only blank context lines; authored
source/JSON/Markdown whitespace checks exclude those patch artifacts.

No app, launcher, daemon, driver, helper, installer, controller, server, network,
live process/port/registry preflight or product execution occurred. There was no
desktop/computer use, unit test, mock, push or publication. No installed app,
sharing, production config/TLS/default IPC, TCC, login item, service, task,
account, firewall, global DNS or toolchain default was changed. No TCP24800,
UDP5353, Docker, credential or existing private key was accessed. The three
provider-stopped tasks were not resumed. No further task was delegated.
