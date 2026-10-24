Independent operation, source, and artifact review of running-update preparation

Reviewed on 2026-10-23, America/Vancouver. Result: **three P2 findings; no P1
finding**. The frozen controller needs corrections before an execution proposal
can be cleared. This review grants no integration or runtime approval. It is
source and saved-evidence review, with direct artifact reads as data. It is not
live preflight or independent execution.

The target is `9490a0e46eadf73785901b33d040d873b34e1157`. The preparation folder is
`docs/verification/sparkle-running-prep-2026-10-23-t3-aa79bac9/`, called `PREP`
below. All controller line references refer to that commit's
`PREP/controller-DO-NOT-RUN.py.txt`, not this checkout's current product source.

Exact frozen identities:

- Proposal: `2d23db1f735e607d70f92e29a8e0837425c4908bd935a3950ba60f458dc6f2a6`.
- Controller: `e251cf7477ed9accf7c28e15de4cabfad50306e12c1934c9b37155a1f40b2740`.
- ZIP: `9dd967f30ffcba5388e1d093e84243e1db9feb4ed6c7f66299609112cc6dbf69`,
  4,649,358 bytes.
- Signed appcast:
  `f4b565d1ab3a0a59bf20dd858bc5a528b04c911b281aeef0bf7a9b475ce2fea2`, 1,149 bytes.
- Preparation evidence manifest:
  `2246f960192284303a240e07c43f48408ba53a72cbc40bdde5a4217d9f17d7f3`.

**P2 R1: Keep the process ledger active across readiness and OS evidence calls.**

Locations: controller lines 444, 457-473, 484-504, 511-519, and 527-530;
`operation-DO-NOT-RUN.json` fields `limits` and `supervisor_stop_limit`.

The initial loop puts observed app and daemon identities in `owned`, but never
adds them to `all_seen`. The two-daemon and 32-identity checks use only
`all_seen`, which starts receiving observations in the update loop. Between the
old readiness observation and that loop, `ready()` and `os_evidence('old-ready')`
perform blocking IPC and OS evidence work without process sampling. There is
also no restart ledger in the final normal-exit loop.

A concrete missed case is old daemon D1 becoming ready, then exiting during the
old-ready OS capture. The unchanged supervisor can start D2 after its 500 ms
backoff. If D1 is absent by the first update sample, the update ledger sees D2
and the eventual candidate daemon D3: only two daemon PIDs. `gone(old_child)`
checks D1, and the new-child inequality also compares D3 only to D1. The observed
old-parent restart is not rejected. D2 can exit with the old app, and all later
checks can pass despite three daemon identities over the operation. A restart
observed before the initial ready receipt is similarly omitted from the cap.

This follows from the controller and `DaemonSupervisor.swift:78-81,135-151`.
No restart was executed here. The issue includes identities that the initial
loop actually observes and an interval with no observer at all. The disclosed
possibility of missing brief processes in 50 ms sampling does not account for
these gaps.

Required correction: start one persistent identity ledger with the initial
launch and retain it through final exit and the quiet cleanup interval. Count
each PID/start identity once, including initial observations. Associate daemon
identities with their independently matched old or new parent. Reject an old
parent spawning a replacement child instead of spending the candidate's second
daemon allowance on it. Keep the ledger active while the main control path
captures OS evidence or waits for IPC. Retain the separate ESRCH checks for both
old app and old daemon.

**P2 R2: Enforce absolute I/O deadlines inside the blocking operations.**

Locations: controller lines 68-69, 241-264, 329-358, 470, 516, and 614;
proposal limits `IPC_timeout_seconds`, `HTTP_connection_timeout_seconds`, and
`whole_runtime_seconds`.

`ipc()` sets a two-second socket timeout once. `exact_read()` then starts a new
`recv()` for every incomplete fragment, with no monotonic deadline or `budget()`
check. A response that continues supplying a fragment within each timeout can
hold the main thread well beyond two seconds and beyond the 240-second operation
ceiling. The 65,536-byte frame cap bounds bytes, but not elapsed time. Both old
and new readiness call this path while the target and daemon are running. The
HTTP thread does not independently start cleanup at 240 seconds, and the final
elapsed-time assertion cannot stop an in-progress read.

The HTTP handler also computes the remaining connection time only before
`sendall(response)`, then reuses that timeout for `sendall(data)`. Time spent
sending headers is not deducted before sending the body. These two sends can
therefore exceed the stated absolute two-second connection lifetime.

Required correction: give each IPC exchange a monotonic absolute deadline,
clamped to the operation deadline. Recompute the remaining time before connect,
send, and every frame read. Recompute the HTTP remainder before the body send.
Ensure the outer runtime deadline can interrupt blocking work and enter the
bounded owned recovery path. Preserve the four observer requests, frame caps,
and no-retry rule. The disclosed sampled log and post-capture output limits do
not supply these missing time bounds. This is a static deadline finding; no
slow-response fixture, server, or controller ran in this review.

**P2 R3: Restrict the controller's normal quit to the validated new target.**

Locations: controller lines 299-314, 523-525, and 538-544; proposal fields
`final_quit` and `approval_scope`.

The normal success path calls `normal_quit(new_app)`. The `finally` path instead
selects any live `IMAGE` entry from `owned` when `quit_sent` is false. This includes
the old app. `normal_quit()` checks UID, start identity, and executable path, but
has no new-build or validated-new-identity gate.

For example, an updater failure while the old app is still alive records the
failure, then reaches `normal_quit(old_app)`. An initial readiness failure can
reach the same action if the old image was observed. This controller-issued
Apple quit is outside the frozen `final_quit` contract, which permits only the
independently matched new PID. The shared executable path does not distinguish
the two builds. The `quit_sent` flag limits the count but does not fix the target.

Required correction: retain an explicit validated new-app identity and permit
the single controller normal quit only for that identity. A failure before that
identity exists must use the separately described, delayed owned recovery path
or report a retained process. Do not silently expand the approval scope to an
old-app normal quit. This review authorizes no such scope change or execution.

**Source and operation checks.**

Read the complete preparation README, proposal, controller, artifact report,
manifest, preparation/audit sources, and raw records. Read the full original
bootstrap README, repair README, R2 review at `62969e571f96ac7c3ba0d7a82bf4069544d24e46`,
and R4 review at `f190240318f4e2fca68a869ad0bb78225dbb4e7b`. Read the full 300-line
C launcher, 39-line plist template, 146-line packager, 463-line Swift bootstrap,
entry dispatch, and the relevant real model, supervisor, IPC, and pinned Sparkle
source paths. Earlier findings remain closed in source for the supported entry.

The C entry requires zero external arguments, derives the literal private
target from its own path, validates the exact fixture plist, and has no
production branch. It establishes the fresh private environment before
`execve` loads the Swift image in the same PID. The Swift entry refuses a nil
whole plist, requires the internal argument pair, checks the already-private
environment, and validates marker identity, two builds, and finite lifetime
before creating the real model. The private `--e2e` path keeps the model's
system-integration guard active. No fallback app, normal updater singleton,
native input, or AppEndToEnd controller is selected on that path.

The operation's root is
`/private/tmp/oe-updater-fixture-915c0654-e982-43e4-be3e-a44fa05799eb`, with literal
`127.0.0.1:56327`, exactly builds `1` and `2`, and independent nonce
`a2fbc1a6-e2b2-4bf4-9e1f-7f5577527b34`. Its frozen six-hour marker spans
2026-10-23 18:56:25.164527 PDT through 2026-10-24 00:56:25.164527 PDT.
It is an entry gate, not an automatic shutdown timer. The controller requires
240 seconds remaining and does not renew it.

Before launch, the text requires proposal/controller/approval bindings, an
explicit running-target and relaunch scope, an independent-review assertion,
and an explicit refusal to reuse the stopped-case approval. It checks source
and inventory hashes, marker bytes, ownership, modes, private paths, initially
empty writable trees, and absence of fixture processes, jobs, caches, receipts,
socket, and a prior run lock. It binds the exact loopback port once without
reuse or fallback. These are reviewed code gates, not observations that those
conditions hold now. The approval receipt is a later human input; its Boolean
review field does not turn this review's unresolved findings into clearance.

The pinned official source export matches all 602 saved entries. Its CLI uses
the supplied host bundle as the application bundle, selects the supplied feed,
and requests an immediate normal update. The user driver chooses installation.
`InstallerProgressAppController.m:196-264,292-308,372-417` matches running apps by
bundle ID and resolved path, sends normal termination requests, remembers
whether the application was initially alive, and uses `NSWorkspace.openURL`
without restoring old arguments or environment. The controller has one initial
Popen and one updater Popen. Its success path has no second target launch.
Real Bundle.main resolution, Apple quit, no-argument relaunch, and the new PID's
readiness still require independent runtime observations.

The readiness checks separately use libproc UID/start/path/parent/nice identity,
owned-process environment, private socket identity and child lsof ownership,
and real framed status/config results. Old app and old daemon exit require
independent identity absence corroborated by ESRCH. New app/child PIDs and
launch ID must differ, with the child parented to the new app, the exact
candidate tree, and unchanged private config. Receipts and CLI exit alone do
not satisfy those checks. R1 describes the remaining restart gap.

The frozen supervisor still sends SIGTERM, waits up to 1.5 seconds, and can send
SIGKILL without joining. Its receipt, nil PID, and shutdown return cannot prove
child exit. All 451 proposal source hashes equal both the preparation commit
and declared daemon source root `ad280644a96905d9831825bb4f87d1388dbdca73`.
No later `3183a9e`/`f19cfc0` stop fix or timing is attributed to these artifacts.

The HTTP text permits only GET/HEAD on the two fixed routes, once per method
and route, with at most four accepted method/route combinations, 8,192 header
bytes, no upload/Range/redirect, and backlog two. HEAD sends the file length
without a body. Both GET bodies total exactly 4,650,507 bytes. Completion also
requires both whole files and no recorded server error during the handoff loop.
The observer IPC path makes two status/config requests per launch, with 256-byte
payload and 65,536-byte response caps. The real AppModel separately makes its
normal subscription and core-read requests. R2 prevents accepting the stated
absolute time ceilings as implemented.

The 12-active/32-total and two-app/two-daemon limits are sampled process claims;
R1 identifies where even observed identities escape their accounting. Product
log ceilings of 1 MiB per initial stream and 8 MiB per CLI stream are checked in
the update loop. OS capture bytes are checked after capture. These checks do
not establish strict peak memory or disk bounds. No kernel exec/dyld trace,
independent LS/NSWorkspace event proof, or signal-outcome trace is supplied.
Missing observations remain unverified.

The helper jobs retain upstream Nice=0 at `SUInstallerLauncher.m:90,278`.
Only the target/daemon are required to measure nice 15; no shared helper is
reniced. The named updater/progress jobs and the eight exact cache roots are
declared. LaunchServices registration/timestamp notifications, normal
Gatekeeper/gktool/XProtect checks, and unknown persistent shared state are
disclosed. Private HOME supplies no external-helper confinement or full
restoration guarantee. No trust or zero-host-effect claim is supported.

Recovery signals recheck the original UID/start/path. The graceful loop allows
ten seconds before controller SIGTERM; SIGKILL and exact job bootout require
fifteen seconds. Job cleanup checks the exact label, canonical owned program
path under an allowed root, and a frozen helper hash. Cache deletion requires
initial absence, current UID, canonical paths, bounded contents, and unchanged
device/inode between observation and removal. Remaining processes block cache
removal. There is no shared-identity unregister, shared-process signal, stale
socket unlink, alternate port, environment repair, or retry. R3 identifies the
normal-quit scope mismatch before those delayed recovery actions.

**Recomputed source and artifact evidence.**

- Read all 474 committed preparation files, totaling 5,962,985 bytes. Every one
  of the 473 manifest records matches its size and SHA-256, with no extra or
  missing paths. The compact sorted-key inventory digest recomputes to
  `a11be31ecd6932d2eb16fbe9bfa82e411c5f1c8dd326931cce8e6c37eff78860`.
- All six proposal evidence hashes match. All 263 historical-file hashes match.
  The original 20-record and repaired 56-record raw manifests also match their
  actual committed bytes. These are preservation checks, not repeat execution.
- All 59 preparation command/result pairs bind to the retained preparation
  source SHA `591a90166a877ef5694f913e2e8e3128f4dd56830649e928014038652e72a0d9`.
  Each records exit zero and matching original stdout/stderr hashes. The list
  has one daemon build, two entry-packager invocations, and file-only metadata,
  copy, archive, signing, public verification, and key-generation work. No
  target, launcher, daemon, external update driver, or installer execution is
  present in that list. All 91 additional static-audit exits and stream hashes
  also match. No listed command was repeated by this review.
- All 22 Swift app inputs and the other compiled/package/launcher inputs match
  the saved 29-record source inventory. Its sole current mismatch is the
  separately recorded, unused `bundle.sh`. The retained build commands directly
  invoke SwiftPM/clang, and preparation does not invoke that script. All 777
  selected copied Swift dependency records match their private originals.
  The original and copied Cargo registry each match the complete 21,716-entry
  saved byte/link inventory. This establishes no inode, timestamp, ACL, or
  other metadata restoration claim for those caches.
- Direct reads match the retained launcher (35,416 bytes,
  `e1f024c4c919021d3976f92d59491f904f86107421c705e36cbf583509cc4c5c`), DEBUG
  Swift app (6,191,952 bytes,
  `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`), and
  fresh pre-sign daemon (4,493,528 bytes,
  `ed7e8fa1872ab10717842bea6ca04ace7bdc74d13e991481c7a065c383c2334a`).
  The latter is bound to the declared older source, not the later stop fix.
- Direct byte/mode/link inventories match all 167 entries of target build1,
  its retained old snapshot, candidate build2, and the round-trip build2 tree.
  Their canonical identities match the report: build1
  `e9069aa62a0b50e16e59504dfd0bd838e2c1a3ddd5cab550f1799e328af4fa82`, build2
  `cf831b64029184f57afc2c1672ab90ed6b143780b31962d44c0ca0303bbbbf59`.
  Both plists select `OpenergyPrivateLauncher` and the same private metadata.
  The six private public-data file hashes match, including marker/config,
  empty launch lock, public key, feed, and ZIP.
- Both copied and retained drivers match their complete 159-entry inventories.
  The new driver inventory ID is
  `73c6c7597672fcfd1d6ed4615194b4d9d4e368828a8131d373e226c40f676303`; its
  executable hash is
  `cf689e05feb90b8e8d24b057ea34c6e9373bbfa18aba2222c827c4cfdea954a6`.
  The historical Xcode registration breach stays disclosed and unaccepted.
- Direct Mach-O parsing confirms arm64/macOS15 and the launcher's sole
  `/usr/lib/libSystem.B.dylib` load, with no rpath or DYLD environment command.
  The signed and original launcher/Swift payloads match after removing signature
  data and normalizing the signature load fields and signature contribution to
  `__LINKEDIT.filesize`. Old and candidate code identities also match the saved
  narrower comparison. The Swift Xcode DEBUG runtime rpath remains present and
  disclosed. This is no final distribution or runtime loading acceptance.
- The retained/new driver bytes differ before the signature only at offsets
  2048 and 2900. Signature size changes 480 to 496; `__LINKEDIT.filesize` changes
  3808 to 3824. The corrected normalization preserves every other pre-signature
  byte and recomputes the shared identity
  `a75cea8530e2125325fa9f462c1c37c3c009cecba7a070b712058180765ade26`.
  This normalization is not whole-file equality, signature trust, or runtime proof.
- The ZIP hash and length match. Reading its 400 entries as data finds no
  duplicate or escaping path, CRC error, or candidate file-byte/link mismatch.
  Saved extraction and direct round-trip inventories supply the mode checks.
  Public enclosure/feed metadata and saved official verifier exits were audited.
  No signer or verifier was invoked here; public cryptographic verification was
  not independently repeated. No new codesign or trust-assessment tool ran.
- The saved daemon compile selects pinned Rust 1.99.1, native arm64, deployment
  15.0, locked/offline dependencies, and jobs two. Its first streams record
  46.44 seconds and overall exit zero. Recount gives 260 samples, 703 descendant
  observations, and 393 rustc observations, all nice 15. Sampling can miss brief
  children; runtime daemon priority remains unverified. The rust-objcopy
  SIGABRT/libLLVM stripping warning remains intact. No strip/build retry or
  toolchain repair is inferred, and crash-diagnostic host effects were not audited.
- The deletion receipt records removal of the new disposable seed and public
  verifier container. Key-generation/signing command records select the fresh
  fixture path. This review read no seed, existing private key, credential,
  Keychain, or secret contents. Saved entitlements contain no library-validation
  exception. Ad-hoc seals and public signatures establish no Developer ID,
  notarization, Gatekeeper trust, release provenance, or publication acceptance.

**Preservation and missing prerequisites.**

The first unused-tooling mismatch, existing-sandbox refusal, and source-set
comparison failure remain in raw with their attempted sources. The first
freeze comparison failure is preserved separately. Its provenance expressly
calls the traceback a transcription from tool stderr; the original stderr was
not separately redirected. The corrected freeze source changes the documented
signature/linkedit comparison. It does not recover an original stream or repeat
signing/packaging. The before-gate proposal/controller drafts also remain.

The earlier failed bootstrap cache assertion, failed audits, overwritten first
load-command limitation, and stopped-case failures remain in the historical
hash set. Later successful metadata reads are not recovered first observations.
Raw trailing spaces remain preserved. This review's first local artifact hash
reader stopped because the default Python lacks `hashlib.file_digest`; the
read-only audit then used the existing Python 3.11 interpreter. Initial reads
also found that PREP is absent from this review branch; all target records were
then read from their exact Git objects. These review-tool errors changed no file.

The independent review is complete with R1-R3 unresolved. A corrected controller
and proposal need fresh hash bindings and another source review before any
execution clearance. This document must not be used to synthesize an approval
receipt or mark an independent review as passed. The existing proposal remains
`approval_received:false` and `DO-NOT-RUN`. The 17:16:55 PDT approval for completed
stopped-target cases `5c2ed82` still does not cover running or relaunching a target.

New explicit user approval of the exact corrected operation remains missing.
Live ownership, process/job/cache/receipt/socket absence, stopped-target state,
unchanged marker with sufficient remaining time, immutable artifact/source
bindings, and exclusive port ownership remain execution prerequisites. None
was exercised as live preflight here. Expiry must block the frozen proposal;
it must not trigger automatic marker renewal or environment repair.

Bundle.main after exec, both no-argument entries, early private environment at
the protected load boundary, real model/child readiness, independent old-child
exit, replacement, actual Sparkle relaunch, final exit, helper cleanup, signal
outcomes, and shared host effects remain unverified at runtime. Standard visual
or manual AppUpdater behavior, native input/clipboard, trusted release, and
publication remain outside this result.

Only this new review document is owned by this task. The review worktree is
`/Users/serge/.t3/worktrees/openergy/t3-2c45371a`, branch
`t3/review-running-update-prep`, initially clean at
`059f052f965bb942d104448b064a9d0e68b1fda1`. No preparation record, bundle,
manifest, source, or prior failure was edited. No prepared controller, preflight,
server, live port/process/OS-registry query, product, launcher, driver, daemon, helper,
installer, build, signing, key operation, unit test, mock, desktop action, or
computer use ran. No installed app, live sharing, production config/TLS/default
IPC, TCC, login/service/task/account/firewall/global DNS/toolchain-default change,
TCP24800/UDP5353 action, Docker startup, network access, push, publication, or
further delegation occurred. The three provider-stopped tasks remain out of scope.
