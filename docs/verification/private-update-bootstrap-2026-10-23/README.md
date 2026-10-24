# Private running-target update bootstrap: source and build only

Prepared on 2026-10-23 in isolated worktree `t3-b61d1a43`, from exact source
parent `5a9783738c5b649844805e1614da7ea37cfbf2bb`. Only the new
`apps/macos/Sources/Openergy/App/PrivateUpdateFixture.swift`, the first dispatch
line in `OpenergyApp.swift`, and this evidence directory belong to this change.
AppUpdater, AppEndToEnd, DaemonSupervisor, AppModel, package metadata, the normal
Info.plist, and all production update feed/key gates remain unchanged.

This is **SOURCE + BUILD ONLY** preparation for a future external official
Sparkle CLI handoff. No target app, daemon, updater CLI, installer, server,
fixture controller, or product command ran, including usage and negative cases.
No fixture bundle, key, feed, update archive, or installer/download control was
prepared here. The compiled executable and copied dependency cache remain private.
No binary artifact or private key is committed. No unit tests, mocks, desktop
automation, app installation, service/configuration change, signing, registration,
Cloudflare action, push, publication, or further delegation occurred. The completed
stopped fixture was not accessed. Stopped provenance/security, mDNS expiry/NSEC/
orphan, and IPC/Synergy3 investigations were not resumed.

## Pinned source basis and scope

The starting prerequisite is
[`macos-updater-install-prerequisites-2026-10-23.md`](../macos-updater-install-prerequisites-2026-10-23.md).
Its Sparkle pin agrees with `Package.resolved`: version 2.10.0, revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`, binary artifact checksum
`17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
The existing cached source at that revision was read, not fetched or rebuilt.
In `Sparkle/InstallerProgress/InstallerProgressAppController.m:383-410`, relaunch
requires `_applicationInitiallyAlive` and uses `NSWorkspace.openURL` with no
old command arguments or environment.

The later normal lifecycle source review was read from Git commit
`a65324a414bf407aaabaa061cdc6ca785b3b1d57`, path
`docs/verification/macos-normal-update-lifecycle-source-review-2026-10-23-t3-ba2ba798.md`.
The parent reports its integration as `9e0105f`. That is inherited source review,
not a review of this new helper and not independent runtime proof.
The completed `5c2ed82a3cd772cc8c14e7b5b472d0b717fec26a` stopped-target scope and
its reported clean R2 do not authorize a running target or a relaunch.

The bootstrap does not instantiate AppUpdater, call its delegates, simulate
installer callbacks, relax signature checks, or run AppEndToEnd. It does not
provide normal updater installation or manual visual UI acceptance.

## Exact bundle metadata contract

The production Info.plist has **no** `OpenergyPrivateUpdateFixture` key. An
ordinary bundle without that key returns immediately to the existing dispatch,
with no fixture environment changes, directory creation, lock, receipt, model,
or NSApplication initialization. Environment variables cannot activate a fixture.

Both future bundles must have the same unique identifier:
`dev.openergy.updater-fixture.<uuid>`, with a canonical lowercase UUID suffix.
There is no fixed identifier or old fixture fallback. A bundle in this identifier
namespace with missing metadata exits 78. Any present but invalid metadata exits
78 before normal argument dispatch or startup, including metadata in a production
bundle ID. A malformed Info.plist that cannot be loaded by Bundle/LaunchServices
is outside the metadata-presence check; future packaging must separately verify
both complete plists and bundle signatures before approval.

`OpenergyPrivateUpdateFixture` is a dictionary with exactly these four keys:

- `Schema`: integer 1. Boolean values are refused.
- `Root`: canonical absolute existing directory, owned by the non-root current
  real/effective UID, mode exactly 0700. Its basename is
  `oe-updater-fixture-<uuid>` using the bundle-ID suffix. Its immediate parent is
  canonical `/private/tmp` or the user's `confstr(_CS_DARWIN_USER_TEMP_DIR)` path,
  independent of inherited TMPDIR. The root cannot contain NUL or exceed 511
  UTF-8 bytes. The socket limit below is usually the tighter limit.
- `Nonce`: a second canonical lowercase UUID, different from the ID UUID.
- `Port`: a whole numeric value in 25000...65535. Boolean values are refused.
  It is used only in the fixed literal loopback off-role config. TCP 24800 and
  UDP 5353 are never selected. The future controller must reserve any endpoint
  it actually uses; metadata alone does not prove exclusive port ownership.

The future preparer must create the root exclusively with fresh UUIDs and nonce.
The bootstrap checks their exact agreement, lifetime, owner, and permissions; it
does not establish global historical uniqueness or defend against a hostile
process running as the same UID. Nothing here writes a global registry/default
to track old identifiers. The root is not created or repaired by the app.

All paths derive from Root; metadata cannot name alternate config or executable
paths. The exact layout is:

```text
<root>/fixture.json                    regular, one link, mode 0600, <=4096 bytes
<root>/launch.lock                     empty regular, one link, mode 0600
<root>/target/                        directory, mode 0700
<root>/target/Openergy.app             the sole runnable target path
<root>/target/Openergy.app/Contents/MacOS/Openergy
<root>/target/Openergy.app/Contents/Helpers/openergy
<root>/state/                         directory, mode 0700
<root>/state/config.toml              exact bytes below, mode 0600, one link
<root>/state/control.sock             MUST BE ABSENT before each entry
<root>/state/tls/                     directory, mode 0700
<root>/home/                          directory, mode 0700
<root>/tmp/                           directory, mode 0700
<root>/receipts/                      directory, mode 0700
```

Every listed directory must already exist. Both app versions must run at that
same target path. An archive candidate at a different preparation path cannot be
launched there using this contract. `CFBundleExecutable` must equal `Openergy`.
Bundle.main, the executable URL, and `_NSGetExecutablePath` must resolve to the
derived paths. Both app and daemon executables must be owned, single-link regular
files with owner execute permission, no group/world write, and no special bits.
The complete bundle traversal allows only owned directories, regular files, and
symlinks whose real targets stay in the same target bundle. Regular hard links,
writable entries, special files, broken links, and links outside it are refused.
The normal versioned Sparkle framework links can remain inside that bundle.

Private directory paths and files must resolve to themselves. The complete
state/home/tmp/receipt trees allow only current-UID directories at 0700 and
single-link regular files at 0600. No symlink or socket entry is allowed in those
trees at entry. Traversals are limited to 8192 descendants and 2047 UTF-8 path
bytes each. The derived control socket must be less than 104 UTF-8 bytes. A stale
socket causes refusal; the bootstrap never unlinks it or kills an old process to
make a relaunch pass. Cleanup and any forced-stop case require a separate plan.

`fixture.json` is JSON with the following required fields. UUID/root/port values
must match the bundled metadata exactly. JSON field names are case sensitive:

```json
{
  "schema": 1,
  "fixtureID": "dev.openergy.updater-fixture.<uuid>",
  "root": "<canonical root>",
  "nonce": "<second uuid>",
  "port": 25001,
  "builds": ["1", "2"],
  "createdAt": 0,
  "expiresAt": 1
}
```

This is a schema example, not valid fixture data. Use actual Unix-second numeric
timestamps: `createdAt <= now < expiresAt`, a positive lifetime of at most 21600
seconds, and finite values. `builds` must contain exactly two distinct positive
decimal strings, each at most nine digits and without a leading zero. The current
bundle's string `CFBundleVersion` must be one of them. Unknown marker JSON fields
are ignored; unknown metadata dictionary keys are refused. The marker is an
ownership/safety prerequisite, not a signature or install authorization.

`state/config.toml` must equal the following UTF-8 bytes, including one final LF.
Substitute the exact metadata Port and Nonce. No comments or extra TOML fields
are accepted. No parser attempts to make arbitrary TOML safe.

```toml
version = 1
role = "off"
name = "updater-fixture-<nonce>"
[server]
listen = "127.0.0.1:<port>"
port = <port>
[client]
remote = ""
autoconnect = false
[tls]
enabled = false
[clipboard]
enabled = false
```

## Entry, environment, and lifecycle

No arguments is the public fixture launch contract. After complete validation,
the bootstrap clears inherited `OPENERGY_*` variables and supplies all of these
explicit values from Root:

```text
OPENERGY_CONFIG=<root>/state/config.toml
OPENERGY_CONFIG_DIR=<root>/state
OPENERGY_TLS_DIR=<root>/state/tls
OPENERGY_SOCKET=<root>/state/control.sock
OPENERGY_DAEMON=<root>/target/Openergy.app/Contents/Helpers/openergy
OPENERGY_DAEMON_ARGS=--platform virtual
OPENERGY_DISABLE_DISCOVERY=1
OPENERGY_ALLOW_PORT_24800=0
HOME=<root>/home
CFFIXED_USER_HOME=<root>/home
TMPDIR=<root>/tmp/
XDG_CONFIG_HOME=<root>/state
XDG_CACHE_HOME=<root>/home
```

It sets process umask 077 and working directory Root. It reads its effective
nice value, refuses if reaching 15 would raise priority, then sets and verifies
absolute nice 15 before model creation or daemon spawn. There is no elevation,
global priority change, or repeated relative nice increment.

The first no-argument entry then `execv`s the same validated executable with
exact internal arguments `--private-update-fixture --e2e`. PID stays unchanged;
there is no second app launch. This gives the second image a private home before
CF/AppKit initialization and uses AppModel's existing `--e2e` guard to skip login
integration without changing AppModel. The top-of-main dispatch catches this
second entry before the normal E2E dispatcher. **All metadata/path/config checks
and the environment/priority setup repeat**; inherited environment is not trusted.
Any other arguments are refused. The internal argument pair alone never enables
the fixture when the metadata is absent. Future controllers must distinguish the
external no-argument LaunchServices launch from this internal same-PID exec.

The second entry exclusively locks the precreated launch.lock for its lifetime.
It creates its own activation-prohibited NSApplication and real
`AppModel(demo: nil)`. It verifies the existing system-integration guard, then
calls `model.start()`. No shared model/updater singleton or normal SwiftUI scene
is used. A 15-second monotonic readiness deadline requires the real AppModel IPC
connection, the supervisor's owned child PID matching the daemon status PID,
virtual platform, off role, the nonce-bearing config/status name, exact literal
loopback config, empty remote, autoconnect false, and TLS/clipboard disabled.
This does not call any model mutation action or start a network sharing role.

Each launch has a fresh UUID. A readiness receipt and one termination receipt use
exclusive names `<build>-<appPID>-<launchUUID>-<event>.json` under receipts. Each
JSON payload is bounded to 4096 bytes and written through a no-follow `openat`
descriptor at 0600, with fsync. The records include fixture ID/nonce, launch ID,
build, app/daemon paths, private config/TLS/socket paths, app PID/nice, last owned
daemon PID, `privateIPCReady`, `shutdownReturned`, and timestamp. Partial files
remain if writing fails; consumers must require complete valid JSON. Directory
crash durability and authenticated receipts are not claimed.

Normal Apple quit follows this fixture's NSApplication delegate. It cancels the
readiness task, captures the owned PID, calls `model.shutdown()`, then writes the
exit receipt. Quit before readiness, readiness mismatch/timeout, or unexpected
run-loop return is a failure. Failure writes a failure receipt when possible and
exits 78; receipt failure also exits 78. There is **no child-gone field**.
DaemonSupervisor.stop can send SIGKILL without a subsequent join. A nil PID, a
shutdown return, receipt, app exit, or CLI success does not prove child exit.

No bootstrap code uses UserDefaults or writes default application paths. This
source design is not runtime proof that CF/AppKit, LaunchServices, OS logging,
or external Sparkle helpers have no shared host effects.

## Future run, separate approval required

No runtime approval is implied by this source change, the build, or the earlier
17:16 stopped-target approval. The parent must obtain an independent review of
this exact source and a concrete execution proposal before any product launch.
This document supplies no executable installer/download controller or CLI command.

At minimum the future proposal must identify both frozen bundles and daemon
hashes, their common fresh root/ID/nonce and marker/config digests, driver identity,
literal loopback endpoints, official CLI command, archive/feed/signature inputs,
process observations, deadlines, and exact cleanup ownership. Preserve the first
failed transcript. Do not retry unchanged cases to turn a failure green.

Before a running-target handoff, separately approve and exercise malformed,
missing, wrong-nonce, expired, and escaping-symlink metadata as appropriate. An
external observer must confirm exit before model/daemon/default-path activity.
Compilation does not verify these refusal paths. The production metadata-absent
case also remains source-only here.

For the actual future handoff, require these observable facts:

1. Old no-argument target is alive and ready through the selected private IPC.
   Record app and owned child PID, process start identity, actual parent, executable
   path/hash, nice value, socket device/inode, config digest, build, and receipt.
2. The unchanged official Sparkle CLI performs validation/download/install and
   requests the real Apple quit event. Record old app termination and independently
   observe old child exit by PID/start identity/path. Detect any late old restart.
   Record SIGTERM versus forced stop; do not infer any of this from stop logs.
3. Observe replacement at the exact target path and Sparkle's no-argument relaunch.
   Require candidate build/tree/executable signatures/hashes, a new app identity,
   and a new owned virtual daemon identity with the new app as actual parent.
   Independently measure bootstrap/daemon nice 15. Confirm the exact same persisted
   config/socket/private home and unchanged config digest, plus new socket ownership
   and real status/config IPC from the new PID. Old-child attachment is a failure.
4. Quit the new fixture normally and record app/child/helper exit and exact owned
   socket/cache/job cleanup independently. Report leftovers and shared host effects.

Pinned Sparkle launcher jobs explicitly use **Nice=0**. That is normal upstream
behavior and must be in the future approved host-effects scope; shell nice 15
does not contain these helpers or a LaunchServices relaunch. The target bootstrap
sets only its own process priority. Transient `<fixture-id>-sparkle-updater` and
`<fixture-id>-sparkle-progress` jobs, helper user caches, installation timestamps,
LaunchServices registration, XProtect/Gatekeeper assessment, OS logs, and possible
shared metadata state need explicit scope and observation. Private HOME and a
defaults domain are not proof of containment for external helpers. Cleanup must
not claim to undo shared registration/assessment history.

This fixture can support external official CLI handoff/relaunch evidence only.
It does not prove normal AppUpdater installation, standard update UI, native
input/clipboard behavior, Developer ID, notarization, Gatekeeper, trusted release
provenance, live Mac/Windows sharing, or publication.

## Build and evidence

The build-only controller in `raw/build-once.py` copied only the historical
`swift/{artifacts,checkouts,repositories}` into a fresh mode-0700 private build
root. It verified identical inventories before use and after compilation.
Only its copied workspace-state artifact path was rewritten. The original
cache was not selected as a build scratch/cache/config/security/module path.
No old runtime fixture path was read. The whole SwiftPM **Openergy app product**
compiled and linked once, with all 22 Swift source files, the copied exact Sparkle
binary artifact, native build engine, jobs 2, locked Package.resolved, update/
prefetch/credential lookup disabled, deployment target 15.0, and private home,
temporary, module/cache/config/security paths. This was a native arm64 debug
build, not a universal package or a current Rust daemon build. Rust was not
invoked and no toolchain default changed. No TypeScript is in this change.

The single build exited **0**, without timeout or retry. Controller elapsed time
was **24.783185 seconds**, including the post-build cache inventory. SwiftPM
reported **23.58 seconds**. All sampled descendants had nice **15**, including
**227 actual swift-frontend observations**. Sampling can miss brief children.
No xcodebuild or Sparkle framework source compilation ran; SwiftPM compiled its
package manifests and linked the existing framework. No bundle script was used,
because the existing bundle script executes a daemon version command.

The compiler emitted two source deprecation diagnostics, repeated by batch
compilation, for array `String(cString:)` conversions at helper lines 114 and 150.
SwiftPM also reports deprecated skip-update/native flags and verbose manifest
diagnostics. They are preserved in the first stdout/stderr and were not hidden
or used to trigger another build. No compiler error occurred.

The build finished before a turn interruption. The subsequent session-resume
call returned `write_stdin failed: Unknown process id 38332`; the existing final
result, logs, and process inventory showed completion, so the build was not
restarted. After the parent confirmed same-task ordinal-1 cancellation and the
live ordinal-2 recovery, a read-only check of all 29 recorded build/controller
PIDs found none present. `raw/resume-reconciliation.json` binds this result and
the unchanged first stream hashes. No owned cleanup or transfer was needed.
A prior read-only discovery command also retained the shell error
`zsh:1: no matches found: crates/openergy*`; narrower source reads followed.

`raw/build-source.json` binds the exact compiled source, including unchanged
production files. `raw/result.json` records the private executable path/size/hash
and full command/stream bindings. `raw/priority-samples.json` retains the actual
samples. Cache inventories and full first compiler streams remain in raw.
`raw/source-audit.json` confirms all compiled source still matches, the entry
change is exactly one line, seven production/package files match the source
parent, and the normal plist has no fixture metadata. These are file comparisons,
not behavior tests. Read-only `file`/`otool` records confirm arm64 Mach-O and
minimum OS 15.0 without invoking the executable. The six pinned source hashes
and the cached framework's actual version 2.10.0/build 2064 are also recorded.
The [manifest](manifest.json) binds exactly 20 raw records. Its evidence ID is
`b7f7976f721697e1a9f3820d2e1377c46d66311c077d1154faf7f6b261f9bf08`;
manifest SHA-256 is
`bcae8d493b8eaf88b0745ef39c4283e462425c6859e80f962f2f3faaca780c78`.
The canonical compiled-source inventory ID is
`cd472e15eef4d845972771e52f5be1aae733453b20362f01498273b45ded1e76`.
The manifest hashes the ordered raw file/bytes/sha256 array with sorted JSON keys
and compact separators. It excludes itself and this README. Final owned source
SHA-256 and Git blob IDs are in `raw/final-audit.json`.

Build success does not prove isolation, a refusal path, process
termination, installation, or relaunch. No runtime acceptance is claimed.
