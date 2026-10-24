# Running-target Sparkle fixture: files prepared, DO-NOT-RUN

Prepared in the T3-owned worktree `/Users/serge/.t3/worktrees/openergy/t3-aa79bac9`,
from clean root `ad280644a96905d9831825bb4f87d1388dbdca73`. This change owns only
this new folder. No production source changed. No target, launcher, daemon,
external updater CLI, Sparkle helper or installer ran. No server bound and no
feed request occurred. The future runtime controller is **text only**.

The [exact operation](operation-DO-NOT-RUN.json) has `approval_received: false`
and `status: DO-NOT-RUN`. Its SHA-256 is
`2d23db1f735e607d70f92e29a8e0837425c4908bd935a3950ba60f458dc6f2a6`.
The [controller text](controller-DO-NOT-RUN.py.txt) SHA-256 is
`e251cf7477ed9accf7c28e15de4cabfad50306e12c1934c9b37155a1f40b2740`.
Neither file grants approval. No runtime approval receipt exists.

The missing prerequisites are an independent review of these exact files and
fixture bindings, then a **new explicit user approval** of the concrete operation.
Execution also requires an unchanged live marker, fresh stopped private target,
absent fixture processes/jobs/caches/receipts/socket, and an exclusive bind to
the exact selected port. A stale or missing marker blocks preflight. It must not
be refreshed to make this proposal pass. The completed `5c2ed82` stopped-target
approval at **2026-10-23 17:16:55 PDT** does not apply.

## Private file artifacts

There is one newly allocated fixture root:
`/private/tmp/oe-updater-fixture-915c0654-e982-43e4-be3e-a44fa05799eb`, mode0700.
The fixture ID is
`dev.openergy.updater-fixture.915c0654-e982-43e4-be3e-a44fa05799eb`.
The driver adds `.driver`. The independent nonce is
`a2fbc1a6-e2b2-4bf4-9e1f-7f5577527b34`.

Exact root paths, marker bytes/identity/timestamps, fixed config text, public key,
and selected port are in [fixture.json](fixture.json) and the operation. Private
state/home/tmp/receipt/TLS directories are0700. Marker/config/empty precreated
launch lock are0600, single-link regular files. `state/control.sock` is absent.
The marker permits exactly build strings `1` and `2` for21600s. The deadline is
finite and frozen; it is an entry prerequisite, not an app shutdown timer.
Port56327 had no TCP owner at the recorded read-only observations. No port is
reserved now. The future controller must own `127.0.0.1:56327` exclusively, once,
without reuse or fallback. Off-role daemon config does not request a TCP listener.

The private layout is:

```text
target/Openergy.app       signed build1 at the sole supported runnable path
old/Openergy.app          exact retained build1 snapshot; do not launch here
candidate/Openergy.app    signed build2 archive source; do not launch here
roundtrip/Openergy.app    exact ZIP extraction; do not launch here
driver/sparkle.app        new unique private driver copy; never run in this task
public/openergy-fixture.zip
public/appcast.xml
fixture.json, launch.lock
state/config.toml, state/tls/, home/, tmp/, receipts/
driver-home/, driver-tmp/, runtime-evidence/  (empty future writable paths)
build/                   fresh scoped daemon build and immutable copied registry
```

Both fixture versions carry the reviewed POSIX entry and private Swift bootstrap.
`CFBundleExecutable=OpenergyPrivateLauncher`. The app code is the reviewed native
arm64 **DEBUG** artifact, not a final release candidate. Both versions carry the
same fresh native arm64 release daemon. A differing outer launcher signature is
expected because each seal binds its own build plist. The unsigned code payloads
are equal; [common-code-identity.json](raw/common-code-identity.json) defines and
binds that comparison without changing either image.

Key artifact hashes, SHA-256:

- Source-bound retained unsigned launcher:
  `e1f024c4c919021d3976f92d59491f904f86107421c705e36cbf583509cc4c5c`.
- Source-bound retained DEBUG Swift app:
  `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`.
- Fresh daemon before local signing:
  `ed7e8fa1872ab10717842bea6ca04ace7bdc74d13e991481c7a065c383c2334a`.
- Signed daemon, both bundles:
  `efcca5dae7615614e5a005be6a28f18961ee8db312e0718b3d5336c30673a90c`.
- Signed Swift app, both bundles:
  `d4d2b53add8f9d3b622ee55470f9a0bef82a7ee3c974428bcfee9ad5bf46ae19`.
- Signed launcher build1:
  `fe8f224b517104ed8e0aec2d927540addd3b6253c76689019d7114f07dcb3584`.
- Signed launcher build2:
  `33b5d432f014ad9f07ae29e5b6c706c4601fbe6fab534eb1f3b8a464f68d0b11`.
- ZIP, **4,649,358 bytes**:
  `9dd967f30ffcba5388e1d093e84243e1db9feb4ed6c7f66299609112cc6dbf69`.
- Signed appcast, **1,149 bytes**:
  `f4b565d1ab3a0a59bf20dd858bc5a528b04c911b281aeef0bf7a9b475ce2fea2`.

Complete byte/mode/link inventories, driver executable hash, tree identity hashes,
public signatures, and source identities are in the operation and
[artifact report](artifact-report.json). Executables, archives, dependency copies,
and key material stay outside Git.

## Source and file-only validation

Read the full original bootstrap README, repaired README, original R2 review,
and R4 review. The candidate `7287e9aa4caf6570fc4116a89908d548a258f302`
is integrated as `c62bfdb`. R4 `f190240318f4e2fca68a869ad0bb78225dbb4e7b`
is integrated as `616b1ad`. Both prior P2 source findings are closed. Those reviews
and this preparation do not independently reproduce any runtime behavior.

The complete saved29-record app/tooling inventory was compared. All22 Swift app
sources, Package.swift, Package.resolved, plist, launcher source/template/packager
match. The only mismatch is the separately audited `bundle.sh` tooling file.
The retained build used direct SwiftPM/clang commands and never invoked that
script. The current script is also never invoked here. Both script hashes and
their exact Git diff are retained. This distinction does not waive a compiled
app input: the app source set and exact executable hashes all match.
The complete777-entry selected cached dependency inventory also matches.

The old daemon's declared source hashes matched, but its complete dependency/
compiler/build-state input chain was not bound here. Its product was not reused.
One fresh scoped daemon compile used the current root's Rust sources, pinned
Rust1.99.1, jobs2, explicit `aarch64-apple-darwin`, deployment15.0 and offline mode.
Only registry dependency content was copied into fresh private state; no Cargo
target/build database or old Swift build state was cloned. The registry source
and private copy stayed byte-identical. An outer compiler sandbox denied all
network and writes to the source and selected original/copy dependency paths.
No toolchain default changed.

The build exited0 in46.44s compiler time. **703** sampled descendant observations
were nice15, including **393** rustc observations. Sampling can miss brief children.
The actual daemon product was never executed, so its runtime priority remains
unverified. The compiler preserved a warning: `rust-objcopy` died withSIGABRT while
stripping because `@rpath/libLLVM.dylib` could not load. No compiler retry, toolchain
repair, strip retry, or product run followed. This is a successful compile with
a retained failed stripping substep, not a clean-warning build. OS crash-diagnostic
effects outside the captured compiler streams were not audited.

The daemon, both signed bundles, ZIP and signatures were already frozen when the
user supplied the later stop-fix coordination. They remain bound to root
`ad280644a96905d9831825bb4f87d1388dbdca73`. Root3183a9e/f19cfc0 and its reported
Off-flow timings were not imported, built, or independently verified here. The
source update does not change this declared artifact's identity or shutdown bound.

Both unsigned private bundles were first assembled at the exact target path,
then passed through reviewed `package-entry.py --app ... --launcher ...` before
outer signing. Build2 was moved to its candidate preparation path after signing;
build1 remains the target. Both packager invocations succeeded. No packager source
change, launcher execution, or failure-case execution was needed.

Normal inside-out ad-hoc codesign used options0 and no credential. Nested and
outer seals, all-architecture deep/strict verification, valid framework links,
arm64 presence, load commands and entitlements passed. No disable-library-validation
entitlement exists in any inspected app/driver/framework code image. The DEBUG
Swift image retains its recorded Xcode Swift runtime rpath. This is disclosed
development linkage, not distribution or notarization acceptance.

The new driver was copied only after the retained stopped-root repaired driver's
full byte/mode/link inventory and deep seal matched. Its exact602-entry official
source export also matched. Only the new copy received the new private ID/defaults
domain and local outer signature. The old driver stayed unchanged. The historical
Xcode-build registration breach remains recorded and unaccepted; no Xcode scheme,
registration, execution-policy command, or upstream build ran here.

A new disposable Ed25519 fixture seed was generated at0600 under the0700 root.
Only the explicit file-only CryptoKit generator and pinned official `sign_update`
used it. The official verifier accepted the exact ZIP enclosure signature and
the complete signed feed. The ZIP round-trip byte/mode/link tree and deep seal
match build2. The seed and public verification container were deleted in finally.
Only public verification metadata remains. No existing key, production credential,
Keychain or account identity was accessed. Prior public signed fixture data could
not bind this new ID/key/source/archive, so it was not reused. The production
`sparkle.py` dev.openergy.app/HTTPS/universal archive contract is unchanged;
private loopback packaging is a separate file-only path.

## Future finite operation and limits

The controller checks exact proposal/review/approval, source, artifact, inventory,
marker, mode, ownership, process/job/cache/socket and port gates before a launch.
It has one initial no-argument entry and one external official CLI invocation.
Only Sparkle may relaunch the bundle. The controller requests at most one normal
app quit through the exact process-addressed NSRunningApplication API.

The proposed future proof is old private AppModel/virtual Off readiness, complete
real archive download, normal old-app quit, independent old app/daemon exit,
exact build2 replacement, Sparkle NSWorkspace no-argument bundle relaunch through
the POSIX entry, new private model/daemon readiness and normal final exit.
PID/start microseconds/UID/PPID/executable paths, actual target/daemon nice15,
private environment, real status/config IPC, socket identities, config digest,
receipts and whole bundle seals are separate evidence inputs. Nil PID, a receipt,
shutdown return, app exit or CLI success cannot prove old daemon exit.

The unchanged supervisor waits up to1.5s afterSIGTERM, then can sendSIGKILL without
a join. The observer must corroborate child exit independently and detect late
old restarts. There is no stronger shutdown bound or forced-stop result claim.
The controller's own recovery sends noSIGTERM before its10s graceful deadline
and noSIGKILL or exact job bootout before15s. Each signal requires the original
UID/start/path identity. It never signals shared gktool/lsregister or strangers.

The operation lists exact HTTP/IPC request, route, frame, byte, process and time
limits. The loopback server has no address reuse, redirect, Range, alternate port,
body upload or repeated method/route. HTTP connection lifetime is bounded to2s.
The whole proposed runtime has a240s ceiling. Some log/resource byte checks are
sampled or performed after capture; they do not prove strict peak resource bounds.
No extra installer negative or runtime malformed-fixture cases are included.
Unsafe or stale file inputs refuse before launch. There are no retries or
environment/config repairs to turn a failed case green.

The approval scope explicitly includes running target/relaunch, private model/
daemon startup and quit, normal upstream external helper Nice=0, fixture launchd
jobs/caches, LaunchServices registration/timestamp notifications, and normal
Gatekeeper/gktool/XProtect assessment. Target/daemon nice15 does not imply helper
nice15. No unrelated process is reniced. Normal checks stay enabled.
Persistent registration/assessment state and reversibility are unknown. There
is no full shared-state restoration promise or shared-identity unregister.
PrivateHOME does not establish external-helper/OS-service confinement.

Pre/post read-only evidence and exact owned cache/job cleanup actions are in the
proposal. Cache cleanup requires initial absence, currentUID, canonical path,
bounded inventory and unchanged observed device/inode. A remaining exact job can
be removed only after program path and frozen helper hash checks. Unexpected
remnants cause failed cleanup with retained evidence. Bundles, build files,
private preferences/config and all evidence are deliberately retained for review.

No kernel execution/dyld trace is supplied. The future noarg command, samePID,
image path/environment and OS receipts must be assessed with the pinned source.
Sampling can miss short entry/helper events. Missing independent exec/LS/
NSWorkspace evidence stays UNVERIFIED. The controller does not self-certify
runtime acceptance; first raw results need independent review.

## Preservation and acceptance boundary

The first preparation stopped on the unused bundle.sh inventory mismatch before
any compiler or product command. The first continuation refused an existing
sandbox record. The second continuation caught a source-set comparison that
included Package.swift as an app source. Every traceback and attempted controller
is retained byte-exact in raw. Corrected reads then used the same single root;
marker/config/nonce were not renewed. No compiler, packager, signing, feed or
archive command was repeated. All59 subsequent preparation commands passed.
The final metadata audit is file-only. The future controller was AST-parsed only.

The first final freeze audit also stopped on its driver code comparison. It had
excluded the signature bytes and LC_CODE_SIGNATURE fields but not the resulting
16-byte change in __LINKEDIT.filesize. Direct byte comparison found only offsets2048
and2900 differed before the signature. The corrected reader subtracts signature
size from the linkedit file size and retains all other pre-signature bytes.
Both normalized identities then match. The first audit source and traceback are
retained. That traceback was transcribed from the first tool stderr; the original
stderr was not separately redirected to a file. This preservation limit is explicit.
No signing or packaging command was repeated.

The complete Git whitespace check reports trailing spaces in the preserved
compiler stderr and the captured Git patch context. Those raw bytes are retained.
The whitespace check for authored files, excluding raw/static evidence, passes.

The earlier historical bootstrap cache failure, first two audit failures, lost
first load-command limitation, and original/stopped-case controller failures were
not rewritten. [history-before.json](raw/history-before.json) and the final audit
bind all earlier evidence bytes. The first frozen proposal/controller draft is
also retained after static review added an explicit inventory-hash gate. No
runtime approval or execution existed for either draft.

Bundle.main, noarg entry/relaunch, private environment, real readiness, old child
exit and handoff remain **UNVERIFIED**. This can only support a future external
official CLI/harness path. Standard visual/manual AppUpdater-controller behavior,
native input/clipboard, DeveloperID, notarization, trusted release and publication
remain open. No unit tests, mocks, GUI/computer use, installed-app operation,
live sharing, production config/TLS/default IPC, TCC/login/service/task/account/
firewall/global DNS/toolchain-default change, Docker startup, push or publication
occurred. No TCP24800/UDP5353 activity or network request occurred. The three
provider-stopped tasks were not resumed.
