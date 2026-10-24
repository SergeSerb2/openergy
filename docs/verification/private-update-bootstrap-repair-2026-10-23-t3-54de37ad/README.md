# Private bootstrap P2 repair: source and whole-app build only

Prepared on 2026-10-23 (America/Vancouver) in the T3-owned worktree
`/Users/serge/.t3/worktrees/openergy/t3-54de37ad`, branch
`t3/private-bootstrap-isolation`. The initial clean root was
`1fd5144628c0494f1ebf561581e0d84fd7231523`. The held change
`a0342aef6b4e4712e7bf73c992b788f26d0e98da` was applied here as
`8826c95`. Its source and original evidence are retained. The repair owns only
`PrivateUpdateFixture.swift`, `apps/macos/PrivateUpdateBootstrap/`, and this new
evidence folder. The inherited first dispatch line remains unchanged.

Read the full [original brief and evidence](../private-update-bootstrap-2026-10-23/README.md)
and the complete R2 review at
`/Users/serge/.t3/worktrees/openergy/t3-0a934f43/docs/verification/private-update-bootstrap-review-2026-10-23-t3-0a934f43.md`,
commit `62969e5`. That review's SHA-256 is
`ec29c95890a4d61f219428177fdb2c38505e2ff1f48b96527b12d29f38f204c5`.
Its findings and saved evidence were read, not independently reproduced at runtime.
This folder supersedes the original entry/re-exec design and its malformed-plist
exclusion. It does not change the original isolation goal.

## P2 #1: distinguish a failed plist load

The Swift first dispatch now guards `Bundle.main.infoDictionary`. Nil exits 78.
It cannot become an empty dictionary and return to production. A successfully
loaded ordinary metadata-absent dictionary still returns to existing production
dispatch. A fixture identifier without metadata still refuses. Present invalid
metadata still refuses. No environment variable can activate private startup.

The new private launcher has no production branch. It reads the actual
`Contents/Info.plist` through a no-follow descriptor, with a size bound, owner,
regular-file, single-link, canonical-path and mode checks. It must match the exact
fixture XML template and all field checks. Missing, unreadable, truncated, binary,
reformatted, malformed or metadata-absent plist bytes refuse before exec. It does
not ask Bundle, CF, Foundation, LaunchServices, or a subprocess to read the plist.
These are source paths; no positive or negative launch ran.

## P2 #2: establish the environment before the protected image loads

The fixture's public no-argument bundle entry is now a separate C Mach-O image:

```
Contents/MacOS/OpenergyPrivateLauncher   CFBundleExecutable, libSystem only
Contents/MacOS/Openergy                 protected whole Swift app image
Contents/Helpers/openergy               unchanged real supervised daemon
```

The first image uses only libSystem/POSIX interfaces and `_NSGetExecutablePath`
from libSystem. It has no CF/Foundation/AppKit/Sparkle dependency or call. It
validates its resolved executable location, private root, bundle-carried plist,
app and daemon images, and private paths/config. It sets absolute nice 15, umask
077, and Root as its working directory. It then calls `execve` on the validated
Swift image with a fresh, explicit environment. The PID stays the same. There is
no framework-bearing first app image that must repair its home and re-exec.
The external no-argument launch and any later no-argument bundle relaunch select
the same launcher from the fixture's `CFBundleExecutable`.

The exec environment has exactly the original thirteen path/control values:

```
HOME=<root>/home
CFFIXED_USER_HOME=<root>/home
TMPDIR=<root>/tmp/
XDG_CONFIG_HOME=<root>/state
XDG_CACHE_HOME=<root>/home
OPENERGY_CONFIG=<root>/state/config.toml
OPENERGY_CONFIG_DIR=<root>/state
OPENERGY_TLS_DIR=<root>/state/tls
OPENERGY_SOCKET=<root>/state/control.sock
OPENERGY_DAEMON=<root>/target/Openergy.app/Contents/Helpers/openergy
OPENERGY_DAEMON_ARGS=--platform virtual
OPENERGY_DISABLE_DISCOVERY=1
OPENERGY_ALLOW_PORT_24800=0
```

It also supplies `PATH=/usr/bin:/bin` and `LANG=C`. No other inherited environment
entry reaches the Swift image, including development, DYLD, CF, XDG and home
values. This environment exists when the kernel/dyld loads the protected image
and its linked frameworks. The first launcher image still uses the system loader
and libSystem; it is not a replacement OS loader or an OS sandbox. Hostile
same-UID mutation/injection resistance is not claimed.

The Swift image accepts only `--private-update-fixture --e2e` for valid private
metadata. Those internal arguments preserve the existing AppModel login guard;
they do not activate a fixture by themselves. A direct no-argument entry to the
Swift image refuses. Its supported bundle entry is the launcher. The Swift
bootstrap repeats the complete original metadata/path/tree/config/marker checks,
checks that Bundle's entry is the launcher while its actual image is `Openergy`,
and verifies the already-private environment, cwd, umask and nice. It never fixes
these values late. It then takes the existing launch lock and uses the same real
`AppModel(demo: nil)` and supervised private virtual daemon. The lifecycle,
readiness checks, receipts and shutdown code are unchanged.

Marker JSON identity, finite lifetime and allowed-build validation still occur
in the Swift image, after private-home establishment and before the model. The
launcher checks the marker's file safety/readability/size, not its JSON meaning.
A bad marker therefore can load frameworks under the private environment, then
refuse before model creation. There is no production fallback on that branch.
A real refusal, isolation or relaunch remains unobserved.

## Exact fixture packaging contract

The original four-key `OpenergyPrivateUpdateFixture` dictionary, fresh ID/nonce,
root ownership/modes, fixed config bytes, marker lifetime/builds, socket absence,
path/tree bounds and receipt contract remain in force. `launcher.c` additionally
requires the complete plist byte format in `Info.plist.in`. All literal XML bytes,
key order and final LF must match. Only six substitution fields are permitted:
`FIXTURE_ID`, `BUILD`, `ROOT`, `NONCE`, `PORT`, and `PUBLIC_ED_KEY`.

The ID and Root must match the launcher's actual fixed target location. The
nonce must be a distinct lowercase UUID. Port must be a positive canonical
integer in 25000...65535. Build must be a positive decimal string of at most nine
digits. Root must use ASCII letters/digits plus `/`, `_`, `-`, `.` and remain
canonical, shorter than 512 bytes. This deliberately avoids XML escape or entity
interpretation. The socket bound is still less than 104 bytes. The full plist
is at most 8192 bytes. Entry executable paths cannot be supplied in metadata.

`PUBLIC_ED_KEY` carries an already selected fixture public Ed25519 key, encoded
as canonical base64 for exactly 32 bytes. This is syntax validation, not a trust
or signature decision. The source contains no concrete fixture key. No signing
key was read or used. This field preserves the target's key needed by the pinned
external official Sparkle CLI; its source supports `--feed-url`, but no public
key override. The fixture template deliberately has no feed URL. A future
separately approved official CLI plan must supply its private loopback feed.
The template uses fixture short version `0.1.0` and the selected build number.
No production feed/key value or gate changes.

`package-entry.py` is opt-in fixture packaging source. It is not part of normal
`bundle.sh`, and it was never invoked, including help/negative cases. Its only
inputs are `--app` and `--launcher`. It requires an already private unsigned
bundle at the exact target path under a mode-0700 `/private/tmp/oe-updater-fixture-<uuid>`
root. Its existing plist must already have the matching four-key metadata,
private ID, build, public key and `CFBundleExecutable=Openergy`. It cannot use an
ordinary production plist or select the root by environment. It checks the
provided launcher as a thin PIE macOS Mach-O, deployment 15.0, with only the
libSystem dylib load, and no rpath or dyld-environment command. Future preparation
must also bind its exact source/artifact hash; this parser is not authentication.

After all input checks, the script exclusively copies the launcher and writes
the canonical fixture plist through an exclusive pending file, then renames that
plist into place. It creates no root, marker, config, key, feed or archive and
performs no signing, launch, registration or installation. Partial preparation
can leave owned files; it is not runnable acceptance and must not be launched.
It does not restore an ordinary plist after failure. The launcher accepts both
original canonical temporary parent options; this packaging helper selects
`/private/tmp` because this Python environment does not expose Darwin's per-user
confstr name. The original historical raw evidence remains untouched.

The template is embedded into a generated C header in the private compiler
workspace. The build controller records the template and generated header hashes.
The C source is outside SwiftPM's app target and is built separately with clang.
No production package target, script, plist, updater or lifecycle source changed.

## Build and saved evidence

One launcher compile/link and one whole SwiftPM Openergy app compile/link each
exited **0**, without timeout or compiler retry. The first controller ultimately
exited **1** on its strict cache equality assertion after both compilers completed.
This is preserved as a controller failure, not presented as a clean controller run.
No compiler/product command was rerun to resolve it.

The build used only copies of the historical `artifacts`, `checkouts`, and
`repositories` trees in a fresh mode-0700 private root:
`/private/tmp/oe-bootstrap-r2-build-yl3byo9v`. It did not copy old `out`, products,
module caches, scratch/build databases or the old SwiftPM native build database.
The controller wrote a new workspace-state file containing the pinned dependency
state and rewritten private artifact path. The pin was Sparkle 2.10.0, revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`; its artifact checksum remains the
locked `17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.

Both compiler commands ran under `(deny network*)` in a local sandbox-exec
profile. The build environment was a fresh allowlist with private home, tmp,
module, cache/config/security and XDG paths. SwiftPM selected locked dependencies,
disabled automatic resolution/update/prefetch/netrc/keychain use, native build
engine, jobs **2**, and the complete **Openergy** product. Its inner manifest
sandbox was disabled because the outer compiler sandbox denied all network.
Clang used `-O2 -Wall -Wextra -Werror -std=c11`. Both products target native
**arm64 macOS 15.0**. Package-manifest tooling itself reports its SwiftPM 14.0
manifest target; that is not the app product's minimum OS.

The launcher controller elapsed time was **0.414777 seconds**. The app controller
elapsed time was **25.784859 seconds**; SwiftPM reported **24.42 seconds**. The
build controller recorded nice **15**. All **901** sampled descendant observations
across the two compiler runs had absolute nice **15**, including **355** actual
swift-frontend observations and **7** clang observations. Sampling can miss short
children. No relative nice wrapper was used. No unrelated priority changed.

The final read-only file/load-command audit reports:

- Launcher: **35,416 bytes**, SHA-256
  `e1f024c4c919021d3976f92d59491f904f86107421c705e36cbf583509cc4c5c`.
  It has one dylib load, `/usr/lib/libSystem.B.dylib`, `LC_MAIN`, minimum OS 15.0,
  and no rpath or dyld-environment load command. It has no framework load.
- Whole app: **6,191,952 bytes**, SHA-256
  `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`.
  It compiled/linked all **22** Swift app source files and the pinned cached
  Sparkle binary framework. This is a DEBUG app product, not a universal bundle,
  signed fixture, current daemon build, or installer/CLI build.
- The **29** compiled/tooling/source records still match their build-time bytes.
  Canonical source inventory ID:
  `69453dd326f6b0309dabb0bf9e8d9cab882afca92912e0e4f48543fc8765595e`.
- AppUpdater, AppEndToEnd, UpdaterEndToEnd, AppModel, DaemonSupervisor, Package.swift,
  Package.resolved, normal Info.plist and bundle.sh are byte-identical to the root
  `1fd5144`. The inherited first-dispatch file is byte-identical to the held
  change. Every original evidence file is byte-identical to that held change.

The two inherited Swift `String(cString:)` deprecations remain at helper lines
109 and 147, repeated by batch compilation. SwiftPM also reports deprecated
skip-update/native flags and verbose manifest diagnostics. Clang has no source
warning/error diagnostic. The first stdout/stderr streams remain unchanged.
SwiftPM's normal build invokes ad-hoc `codesign --sign -` for its executable;
the arm64 linker also emits a code signature load command. No fixture was signed,
no signing key/keychain identity was selected, and no release-signature trust is
claimed. There was no xcodebuild, Rust/cargo invocation, toolchain-default change,
TypeScript change/check, app bundle assembly or packaging script execution.

## Cache coordination and preserved failures

The original selected cache **byte/symlink** inventories before and after this
build, plus the initial private copy, match exactly: **777 records** each. The
only private-copy difference is `checkouts/Sparkle/.git/index`, with unchanged
size and different bytes. Read-only Git staged-object lists have the same
**602** records, paths, modes and object IDs, with matching digest
`363ab5c9a643195363e3792570f004a6573ce3f5c163d77ebb2542a8c568ac74`.
Both checkouts still point to the exact pin. All selected source/artifact bytes
are unchanged. This is not a claim of inode, timestamp, ACL or other filesystem
metadata invariance. The strict controller assertion is retained rather than
weakened or rerun. Its in-memory prebuild workspace-state hash was not saved
before that failure; this folder makes no fresh before/after claim for that
state file. The current hash is recorded separately.

The parent reported that a separate full-scratch clone reused absolute build
paths and removed 114 old `swift/out` entries, then restored absent entries from
its private clone. These are parent-provided facts, not this task's reproduction
or audit. This task never selected old `out` or build databases, wrote/repaired
parent caches, or inspected that unrelated packaging attempt. The advisory had
no effect on this task's selected original before/after byte/symlink inventories.
No cache metadata invariance or parent recovery verification is claimed here.

`raw/controller-completion.txt` preserves the first cache assertion failure.
The first read-only postbuild audit then failed because `-l` and `-L` output
filenames differed only by case on this case-insensitive filesystem. The dylib
listing overwrote that attempt's initial load-command listing. Those initial
load-command bytes are unavailable; the surviving first output is retained and
identified as the dylib listing. A second read-only audit used distinct labels
but exclusively refused an existing file-command output. It overwrote nothing.
Both tracebacks and audit scripts are preserved. The third read-only audit used
new names for every output and completed. No compiler or product was repeated.
These audit record failures do not change either compiler exit code.
`raw/prebuild-notes.json` also retains the two source-discovery errors and the
Python confstr availability check. All inherited failures remain untouched.

## Acceptance boundary

This change supplies source and whole-app/launcher build evidence only. Neither
artifact ran, including help, refusal cases or no-argument launch. The packaging
script was syntax-read only. No real AppModel was instantiated in this task.
No fixture bundle/root/config, feed, archive, key, server or installer was
prepared or run. No signing key was read/used, and no network access was allowed
to the compiler children. No unit test, mock, computer use, desktop interaction,
installation, login/service/registry/Gatekeeper/TCC/account/firewall/DNS action,
production config/TLS/default IPC, live sharing, push, publication or further
delegation occurred. No TCP 24800 or UDP 5353 activity occurred. The three
provider-stopped investigations remain out of scope and were not resumed.

The source establishes the early-environment order for the supported
bundle-carried launcher entry. Build/load-command evidence is not runtime proof
of zero host preference effects, framework behavior, malformed-input refusal,
Bundle.main resolution after exec, readiness, child exit, install or relaunch.
It also does not contain LaunchServices or external Sparkle helpers. The original
brief's future host-effects scope, independent exact-source review and concrete
runtime approval requirements still apply. No runtime approval is supplied.
Future execution must independently observe the initial no-argument launcher,
private environment before the Swift image, same real model/owned child,
old-child exit, replacement identity, no-argument relaunch and final cleanup.
Neither receipt nor `model.shutdown()` proves child exit.


The [manifest](manifest.json) binds **56** raw records. Its canonical
ordered record inventory ID is `929587353dfe4f5ebb9b79590225597ea5eeda9bcebead5be423e75d49ac577a`.
Manifest SHA-256: `6a5015dd75d57704ce80735a96c7de1d876bfc39fbe0306b1176dc39c2aff323`. The manifest hashes raw
records only, with sorted JSON keys and compact separators; it excludes itself
and this README. Compiler streams, all three failed audit/controller records,
all attempted audit scripts, source/cache inventories and read-only artifact
records are included. `raw/final-audit.json` binds the four owned source blobs.
No artifact binary or private cache is committed.
