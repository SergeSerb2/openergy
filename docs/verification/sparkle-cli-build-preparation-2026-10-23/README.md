# Sparkle CLI build preparation and host-limit breach

Parent: `d83807d542a86a99ad83978cc460028d972373e1`. Date: 2026-10-23, America/Vancouver.

**The first real Xcode build exited 0 and produced `sparkle.app`. It did not meet
the private-only host limits.** Xcode ran LaunchServices registration and
execution-policy registration tasks. The later blocked preflight does not erase
that build or its effects. No updater, installer, target app, test, sharing daemon,
or network fixture was executed. Installer runtime remains **unverified**.

The complete build command and timing are in [build.result.json](raw/build.result.json).
The byte-exact build output is retained in [build.stdout.gz](raw/build.stdout.gz).
[results.json](results.json) binds source, tools, commands, artifacts, logs, timing,
and the read-only cleanup audit. Compressed records use gzip with mtime 0.

The build ran from **16:29:06.527648 to 16:29:32.893063 PDT**, elapsed
**26.365235541 seconds**, PID `41387`, under `/usr/bin/nice -n 15` with `-jobs 2`.
Its waited-child CPU deltas were 27.994454 user seconds and 9.796351 system seconds.
These are process accounting values, not machine-wide compiler CPU measurements.
The settings queries also ran at 16:29:02.418278–16:29:06.526938 and
16:31:03.731243–16:31:07.846731. The failed dry-run probe ran at
16:31:07.847633–16:31:08.118254. All receipts include local UTC offsets.

Observed host registration
--------------------------

Xcode's `registered-launchservices.txt` records these exact private products:

- `/private/tmp/oe-sparkle-cli-build-njko0i6t/products/Updater.app`, bundle ID
  `org.sparkle-project.Sparkle.Updater`, version `2.10.0`, build `2064`.
- `/private/tmp/oe-sparkle-cli-build-njko0i6t/products/sparkle.app`, bundle ID
  `org.sparkle-project.sparkle-cli`, version `2.10.0`, build `2064`.

The log records `builtin-lsregisterurl` calling the system `lsregister` with
`-f -R -trusted` for both paths. The receipt and completed tasks are observed;
the shared LaunchServices database was not queried for final state. The attempted
`REGISTER_APP_WITH_LAUNCH_SERVICES=NO` setting did **not** suppress these tasks.

Eight completed `RegisterExecutionPolicyException` tasks name products under
that same `products/` directory: `InstallerStatus.xpc`, `Installer.xpc`,
`InstallerConnection.xpc`, `Downloader.xpc`, `Updater.app`, `Autoupdate`,
`Sparkle.framework`, and `sparkle.app`. The exact paths and available bundle IDs
are in [cleanup-audit.json](raw/cleanup-audit.json). These task executions are
observed. Persistent exception state was not read back. Upstream Swift Build
[source](https://github.com/swiftlang/swift-build/blob/main/Sources/SWBTaskExecution/TaskActions/RegisterExecutionPolicyExceptionTaskAction.swift)
describes an ExecutionPolicy call through an XPC connection to `syspolicyd`.
That is source-derived context, not a measurement of this host's persistent state
or a verified source revision for its installed build engine. Installed tool and
build-engine hashes are recorded separately.

The later `-dry-run` probe exited **64** with
`xcodebuild: error: option '-dry-run' is no longer supported`.
Its error handler wrote an additional result bundle outside the requested private
TMPDIR:
`/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/ResultBundle_2026-23-10_16-31-0008.xcresult`.
That bundle exists; its files are hashed in the cleanup audit. It remains intact.

Produced artifacts and source binding
------------------------------------

The retained build root is `/private/tmp/oe-sparkle-cli-build-njko0i6t`.
It contains the source export, DerivedData, objects, products, compiler caches,
original controller, complete logs, metadata, and this private artifact:

`sparkle-cli-2.10.0-arm64-build-only.tar`

SHA-256: `35cc7148dbd830b991e36f063095785ee321ef87ba80180acb28f56c51b5e05b`.

The CLI binary is `products/sparkle.app/Contents/MacOS/sparkle`, 69,528 bytes,
SHA-256 `401330f492a51597729815f55f6edaad9b3addda2ca2a13258b352c1b6ca3778`.
The app includes Sparkle.framework, Autoupdate, Updater.app, Downloader.xpc and
Installer.xpc. All six app-bundled Mach-O files are arm64 with macOS 15.0 minimums.
The build also produced the standalone helper products named above and static
libraries. The app tree manifest, including modes and symlink targets, has file
SHA-256 `61722d47e3f088efeea1831e7c468313339e1bc381f0cc999db786ffda67e41a`.

Only metadata tools inspected these products: `file`, `otool`, and display/verify
modes of `codesign`. The binaries have linker ad-hoc signatures with no Team ID.
No signing key or named identity was read, used, or created. Named/automatic signing
was disabled. `codesign --verify --deep --strict --verbose=4 sparkle.app` exited 1:
`code has no resources but signature indicates they must be present`.
No post-build signing or resource-seal repair was attempted. This is a compiled
prerequisite artifact, not a trusted or installation-ready bundle.

Source was exported with `git archive` from the existing exact SwiftPM checkout,
with Git optional locks disabled. The historical combined state and checkout were
read-only inputs. No source fetch was needed. The official source pin is
Sparkle **2.10.0**, commit `eef1a539a373c1f1a320624b1130fc5de7b2e100`, Git tree
`1973159ee5d6ae334396a84b3e00e924ac94ce7c`.
The source archive SHA-256 is
`b2afa878a44e8a3e1609ddd07964e7f7938bf7e8e54d30b648961d412f5cf781`.
The 602-entry source inventory digest is
`5342ed8f91a16eafc114a4dd6b85a48ebe67a7d5d4c12c7770c154efbcebb629`
using SHA-256 of Python `json.dumps(inventory, sort_keys=True).encode()`.
The post-build source inventory matches exactly.

The official [sparkle-cli scheme](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/Sparkle.xcodeproj/xcshareddata/xcschemes/sparkle-cli.xcscheme),
build configurations, reachable scripts and ten production targets were inspected
before building. No test target was selected. The reachable scripts strip unused
framework files, set Git version metadata, and link helpers. Release packaging and
XCFramework-generation scripts were excluded. Vendored bsdiff and ed25519 source
was reused. The unrelated swift-argument-parser reference was not built. No project
manifest or dependency lockfile changed.

Tools: full Xcode **27.1 / 27A9275**, Apple clang **21.0.0 / clang-2100.3.34.2**,
macOS SDK **27.0 / 26A425**, host macOS **27.2 / 26B5112a**, Python **3.11.15**.
The exact paths and SHA-256 values are in [tool-hashes.json](raw/tool-hashes.json).
Compiler output, DerivedData, objects and successful-build products used the new
private root. The host-registration tasks and fallback result bundle above prevent
a claim that every write stayed within those paths.

Cleanup audit and reusable preparation
--------------------------------------

No unregister, execution-policy removal, launchd, defaults, TCC, login, account,
firewall, or installed-app change was attempted. The four recorded Xcode client
PIDs were absent at the read-only audit. This does not inventory or terminate
shared compiler services. All source exports, build artifacts, logs, and the error
result bundle were retained. No evidence or shared output was deleted or rewritten.

No safe shared-state reversal is established. These products use upstream shared
bundle IDs. There is no pre-build LaunchServices or execution-policy snapshot.
Unregistering by identity could affect another copy. A cleanup plan needs evidence
of the exact registered paths, domain and pre-existing associations before any
mutation; the current receipt alone cannot supply that baseline.

[prepare_sparkle_cli.py](../../../tools/release/prepare_sparkle_cli.py) now supports
only source export and a static blocked preflight. It never invokes Xcode, a
product, a server, signing, or a service command. It requires Python 3.11+ and creates
new mode-0700 roots. It records the unsafe build recipe as `build-recipe-DO-NOT-RUN.json`.
The first build used an earlier controller; its exact hash, private copy and command
remain in the evidence. The final controller does not describe the first attempt.

```sh
python3.11 tools/release/prepare_sparkle_cli.py prepare \
  --source-repository /absolute/path/to/existing/Sparkle/repository
# Inspect the returned root's source-review.json and source-files.json.
python3.11 tools/release/prepare_sparkle_cli.py preflight \
  --root /private/tmp/oe-sparkle-cli-build-RETURNED \
  --reviewed-source-sha256 5342ed8f91a16eafc114a4dd6b85a48ebe67a7d5d4c12c7770c154efbcebb629
# Expected: exit 1, BLOCKED; no Xcode invocation or compilation.
```

The real final controller path passed source export, blocked preflight and invalid
root rejection. Its product directory stayed empty and no build log was created.
See [controller-validation.json](raw/controller-validation.json). No unit tests,
product runtime, GUI, or installer tests ran. No TypeScript files changed.

Remaining prerequisites are a host/workflow that permits or verifiably excludes
Xcode registration, a complete private ad-hoc resource seal, future fixture-only
key material, and uniquely isolated versioned fixture apps. Future installation
must account for Sparkle's transient launchd jobs/helper caches and relaunch losing
arguments/environment, as identified by the prior research task. No key or fixture
app was prepared here. Download, signature validation during installation, helper
launch, replacement, daemon handoff, relaunch, cleanup and trusted distribution all
remain unverified. The current host limits were not relaxed.
