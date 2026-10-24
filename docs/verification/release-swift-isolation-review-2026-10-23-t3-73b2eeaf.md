# Held Swift isolation source and evidence review

Date: 2026-10-23, America/Vancouver. Owned worktree: `t3-73b2eeaf`.
Branch at start: `t3/73b2eeaf`; confirmed final T3 binding:
`t3/review-swift-sandbox-build`, with the same owned worktree path.
Review base: `059f052f965bb942d104448b064a9d0e68b1fda1`.

**One new P2 finding: the opt-in does not exclude universal builds that can
override its requested native backend. No P1 finding.** This finding does not
explain the recorded single-architecture failure. The known native release
build path remains unqualified. The opt-in remains **UNINTEGRATED**; the saved
package remains **FAILED**. This review is not runtime reproduction or release
clearance.

## Inputs and boundary

Held source: `fc33bdc0cbb261d0a096ec459b5ba0dce25ae9ec`.
Held evidence: `567eb51d48d88731ecaf0272b1be98e35f1e3a93`.
Evidence prefix: `docs/verification/release-swift-isolation-2026-10-23-t3-bc17f311/`.
References to `raw/` below use that prefix at the held evidence commit.
Line numbers refer to decoded text. The comparison build is
`7287e9aa4caf6570fc4116a89908d548a258f302` under
`docs/verification/private-update-bootstrap-repair-2026-10-23-t3-54de37ad/`,
not the older, differently configured bootstrap folder.

I read the complete 185-line README, source diff and full bundle script,
controller, preparation, both collectors, final audit, observation hook,
policy, recipes, environment, command streams/traces, inventories, ledger,
first failures, and result/limit records as data. I decoded and checked the
complete 101-file evidence tree. I read all 97 ledger exports and their
retained originals, including the two narrowly redacted records. I did not
import or execute any saved controller, collector, observer, or project helper.

The only owned change is this document. Git object reads, ordinary file reads,
hash/JSON calculations, and explicitly permitted public primary source reads
were used. Neither held commit was checked out, cherry-picked, or integrated.
No new manifest, build, package, project CLI, compiler, sandbox, process/port
query, fixture, signing, or product command ran. No tests or mocks were written.
There was no delegation, computer use, desktop interaction, credential use,
installed-app action, global configuration change, push, or publication. The
three provider-stopped tasks were not resumed. Public source requests were
limited to the permitted SwiftPM/support-library source repositories.

## P2: reject universal builds before selecting the isolated native mode

Location: `apps/macos/scripts/bundle.sh:30-34`, in combination with lines 62,
75, and 79-84; `docs/releasing.md:137-142`.

With both `OPENERGY_SWIFTPM_OUTER_SANDBOX=1` and `UNIVERSAL=1`, the script accepts
the input and passes `--build-system native --arch arm64 --arch x86_64` to both
Swift build calls. The docs state that this mode selects the native engine.
There is no single-architecture restriction before either Rust build.

SwiftPM's primary source explicitly overrides a requested native engine when
the architecture count exceeds one. In the pinned public source snapshot it
selects `swiftbuild`; Swift 6.3 selects `xcode`. Thus a caller cannot rely on
`--build-system native` alone to retain the backend assumed by this isolation
recipe. See [Options.swift, lines 637-644](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/CoreCommands/Options.swift#L637-L644)
and [the Swift 6.3 implementation](https://github.com/swiftlang/swift-package-manager/blob/swift-6.3-RELEASE/Sources/CoreCommands/Options.swift#L555-L562).
The exact Apple SwiftPM source revision shipped in the saved Xcode was not
available in these records; this is a source-backed compatibility finding,
not an independently observed universal invocation on that binary.

The consequence is silent selection of a different, unqualified build backend
after potentially expensive daemon builds. The outer denial still applies;
this is not evidence of a sandbox escape. Narrow correction: fail before
destination changes and builds when the new opt-in and `UNIVERSAL=1` are both
set, and document the opt-in as single-architecture only. Preserve the ordinary
universal path. Supporting isolated universal builds needs its own backend
qualification, not a claim that the native flag guarantees that backend.

The recorded package forces `UNIVERSAL=0` and uses one `--arch arm64`. Therefore
this finding is separate from its known output-file-map failure. I found no
other concrete new P1/P2 issue in the reviewed source or evidence.

## Actual manifest versus package build arguments

The bound manifest command is the canonical Xcode `swift package`, followed by
these exact shared flags, then `--verbose describe --type json`:

```text
--scratch-path <P>/swift -c release --jobs 2 --disable-automatic-resolution
--arch arm64 --build-system native --disable-sandbox
--cache-path <P>/swift-cache --config-path <P>/swift-config
--security-path <P>/swift-security --manifest-cache local
--sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
--disable-keychain --disable-netrc
```

Here `<P>` is the exact retained private root named by the ledger. Recipe 1
and its start record agree on every argument and on the working directory
`apps/macos`. The command evaluated the actual project manifest. It returned
the real Openergy description, its 22 Swift sources, Sparkle 2.10.0, Swift
language mode 6, and macOS 15.0. Its result is exit 0, **10.896334334 seconds**,
with no timeout or controller error. The forbidden diagnostics are absent.

The package's actual traced command is `release.py package --port-range
65440-65449 --output <P>/output`, followed by the real bundle shell invocation.
The shell does not use the Python observer for its own Swift subprocesses.
Consequently, the Swift build argv is reconstructed from the exact hash-bound
bundle source and environment; it is not a separately captured Swift exec argv.
The reconstruction is the same canonical Swift executable plus `build` and
the shared flags above. It has no `--verbose`, `describe`, or JSON output option.
The retained `.buildSystem_release` contains exactly `native`, consistent with
the requested single-architecture engine. The second `build --show-bin-path`
and the icon command were not reached.

The recipe and package plan contain identical explicit environments. The
package source then adds a private `DIST`, selects its existing Python,
and forces ordinary ad hoc intent and `UNIVERSAL=0`. The bundle derives
`MACOSX_DEPLOYMENT_TARGET=15.0` from the normal plist. The inherited value was
already 15.0. The release and bundle scripts prepend private
`HOME/.cargo/bin` to PATH; that path does not change the absolute selected Swift
or the recorded canonical tool paths. Compiler overrides select genuine
`swiftc` and `swift-frontend`; the observer does not replace them.

Passing build options to `describe` does not perform a native product build.
The manifest compiler actually used `-target arm64-apple-macosx14.0` and the
selected macOS SDK. That is the host manifest executable, not the application
deployment target. The returned package platform is 15.0. There is no evidence
that the manifest preflight exercised release whole-module planning,
output-file-map generation, or universal backend selection. The manifest
compiler commands are logged, but its compiler NI15 was not independently
sampled. These limits are stated in the evidence README.

No argument mismatch establishes `--arch arm64` or the explicit SDK flag as the
cause. A single architecture does not meet the public source's multiarchitecture
backend-override condition. The failed command never supplied a second arch.

## Read-only comparison with successful DEBUG bootstrap `7287e9a`

I checked the exact `raw/app-command.json`, `raw/app-result.json`, controller,
and both complete app streams at that commit. The result hashes match the
streams. That command returned 0 in **25.784859 seconds**; SwiftPM reported
`Build of product 'Openergy' complete! (24.42s)`. It used the same saved Swift
6.4 toolchain version and SDK path. It is a genuine app compilation comparison.

Its differences matter:

- Its outer profile was only `(version 1)(allow default)(deny network*)`.
  It imposed no filesystem write denial. Private environment paths alone did
  not establish the filesystem containment used by the held package.
- It requested `--build-system native`, jobs 2, the actual `Openergy` product,
  and verbose output. It omitted `-c`, so it used DEBUG. It also disabled index
  storage, dependency-cache use and prefetching, and passed deprecated
  `--skip-update`. The held package requests release and the default build set.
- It supplied no `--arch` and no command-line `--sdk`. It supplied the same SDK
  through `SDKROOT`. Its real compiler command used
  `-target arm64-apple-macosx15.0`, `-sdk` with that SDK, `-Onone`, `-DDEBUG`,
  incremental compilation, and its private DEBUG output-file-map path.
- It used `PATH=/usr/bin:/bin`, without the held recipe's explicit `SWIFT_EXEC`
  and frontend override. The held recipe selects canonical toolchain paths.
  Both commands use private scratch/cache/config/security/home/temp paths and
  copied pinned Sparkle state, but use different fresh roots and source inputs.

That successful command proves the DEBUG native compiler path worked in its
recorded environment. It does not prove a release native build under the held
filesystem-denial policy. There is no controlled comparison that changes only
architecture, SDK selection, optimization, or policy. Removing `--arch`, changing
the toolchain, or broadening writes would not be a source-established repair.

## File-map error semantics and cause limit

The package result is exit **1**, **84.697217875 seconds**, without timeout or
controller error. The complete stderr retains the Rust warning and ends with:

```text
error: encountered an I/O error (code: 1) while reading <P>/swift/arm64-apple-macosx/release/Openergy.build/output-file-map.json
```

Primary SwiftPM [filesystem diagnostics](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Workspace/Diagnostics.swift#L243-L251)
use that wording for a generic `.ioError(code:)`; the diagnostic does not
identify a read syscall. Primary native [SwiftModuleBuildDescription](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Build/BuildDescription/SwiftModuleBuildDescription.swift#L789-L800)
materializes the output map when it emits compiler arguments. Its
[file write is atomic](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Build/BuildDescription/SwiftModuleBuildDescription.swift#L989-L990).
The support library delegates atomic writes to Foundation `Data.write` and
maps filesystem errors. Its POSIX mapping preserves codes other than its
explicit special cases. Code 1 is compatible with EPERM; missing-path ENOENT
has a separate mapping. See [atomic writes](https://github.com/swiftlang/swift-tools-support-core/blob/c574915fe88e942e4c4c93376f022daa2ecf05f9/Sources/TSCBasic/FileSystem.swift#L686-L700)
and [error mapping](https://github.com/swiftlang/swift-tools-support-core/blob/c574915fe88e942e4c4c93376f022daa2ecf05f9/Sources/TSCBasic/FileSystem.swift#L107-L139).

This identifies native output-map materialization, including an atomic-write
operation, as a concrete source-backed investigation point. It prevents
treating the message as proof that a pre-existing map could not be read.
The saved postflight found the map and `master.swiftdeps` absent, with their
private parent directories present. It did not capture the failing syscall,
an underlying NSError chain, an auxiliary temporary-file path, or a policy
denial record for that operation. Public source is not an exact source binding
for Apple's shipped SwiftPM/Foundation binaries. Thus an atomic-write denial
is a hypothesis, not a proved root cause. Neither absent output nor code 1
alone proves it. No concrete source-backed cause is established by this review.

The old `3dcf8b2` failure and the new failure stay distinct. The old records
show Darwin `xcrun_db` write attempts and nested `sandbox_apply` EPERM. The new
manifest passed without those diagnostics. They also did not recur in the new
package stream. The separate genuine `xcrun --no-cache` metadata probe still
reported denied Darwin-temp writes despite exit 0, and its first stderr is
retained. There is no claim that all future xcrun uses are now isolated.

## Evidence integrity, bindings, and preservation

The following checks were independently recalculated from saved data and
current retained file reads, without rerunning their producers:

- All **97 exports**, covering **96 distinct retained original paths**, match
  their original/exported byte counts and SHA-256 hashes and decoded payload
  hashes. Only the two port exports differ from original payload bytes. Each
  difference is exactly the declared unrelated mount-path redaction. All 101
  files in the evidence tree are accounted for by the ledger and four root
  documents. Ledger SHA-256:
  `55a4421ec4ae5e1f693f6e98a1f6f18a8a227bb4f8c409f9ede496269a3e7fec`.
- Recipe, operation starts, plan, result, controller, observer, policy, budget,
  qualified-manifest, and stream hash references agree. Manifest recipe SHA-256:
  `c06472c2d87555e9643bf8056eae02925ebfce5ae3609c6e01434eb382700c32`.
  Package plan SHA-256:
  `d714e1afd54d81a145849baa319e3b6a15df0c2f64d905161796ffef9a038dda`.
  Final policy SHA-256:
  `6e983bbd49b060b8143b9df176aac9044e3cde6190d7f185f86d37f0f2ade1a9`.
- All **718 source input** records match their actual paths/blobs in the held
  source commit and the recipe/plan source hashes. The sole nonidentity checkout
  conversion is the declared LF-to-CRLF conversion for
  `tools/windows/openergy-run.cmd`. The start-to-build diff has only
  `apps/macos/scripts/bundle.sh` and `docs/releasing.md`.
- Initial, post-manifest, and post-package preservation snapshots are equal.
  A new read-only file audit also matched both full prior evidence folders,
  all **89 private records**, both old guards, and the old cache inventories:
  **1,309 Swift / 1,159 Cargo entries**, with **1,725 Swift / 1,378 Cargo selected
  metadata objects**. Their recorded modes, ownership, kind, link count, flags,
  mtime, ctime, birthtime, and symlink targets matched. Read atime is excluded.
  ACLs, xattrs, and original prebreach metadata restoration remain unverified.
- Exact copied Sparkle inventories match the old source/artifact/repository
  subtrees. The qualified private checkout changes only `.git/config`, Git
  alternates, and `.git/index`; artifact and repository payloads are unchanged.
  All later seed inventories match that qualified baseline without further
  differences. Revision `eef1a539a373c1f1a320624b1130fc5de7b2e100`, version
  2.10.0 and the resolved pin remain bound. Cargo receives registry data only;
  its target starts empty. No prior app output or build database is cloned.

The first collector had **81 entries**. Its empty active-stdout snapshot was
initially bound to the then-active `collect.stdout`. The first audit correctly
recorded that its later completion line changed the original hash. The first
collector and ledger remain retained. Comparing ledgers shows only that entry
rebound to `collect-stdout-before-completion`; the exported empty bytes and
their hashes did not change. That immutable prefix original is exact.
`collect-final.stdout` separately binds the completed line, SHA-256
`5a8acb07f0f93184005ef9e44af9ebf6b2b70105753d4c3ab82eda2b2ff0724f`.
The two retained collector scripts are byte-identical. Sixteen later entries
bring the final ledger to 97. No package/manifest stream or compiler failure
was silently replaced. The setup patch-context failure, first denied xcrun
output, export-audit issue, and package stop traceback all remain visible.

The manifest and package result hashes match their complete original streams.
Package stderr SHA-256 is
`680dd308613f0313306dc2c52858529645ea485b0e0723ac22d039875b3320f6`;
stdout SHA-256 is
`0d5e8577b95e789eaf90fbdb145167a1213fc68f2a40ade482a67acaea242be6`.
The `rust-objcopy` SIGABRT / missing `@rpath/libLLVM.dylib` warning is present
verbatim. The pinned Rust 1.99.1 toolchain was not replaced. The retained
4,492,840-byte arm64 daemon matches
`9af7163b820e1cbb0edfba2f95088b45e9de27c0902bdeb8b9ed9a57640213b0`;
this review read its bytes and did not execute it.

## Source contracts, policy, priority, and stop boundary

With the opt-in unset or `0`, the build still resolves `swift` through PATH,
uses the same release/jobs/architecture/resolution flags, and adds no isolation
or sandbox flags. The new one-word icon array expands to the previous `swift`
invocation. The new executable and path values are quoted as single arguments;
array expansions preserve spaces. I found no introduced shell-splitting issue.
The flag rejects values other than 0/1. Enabled mode requires nonempty absolute
executable/directory values before builds. These checks do not prove canonical
identity, containment, freshness, or the presence of an outer sandbox. The docs
explicitly assign those requirements to the operator; the evidence controller
checks the concrete private roots and hashes. This is not an automatic outer
policy enforcement feature.

The actual selected SDK is the external, read-only Xcode SDK; it is not a copied
private SDK. `--sdk` selects that SDK, while private HOME/config paths contain
user state. The created `swift-sdks` directory is not passed with
`--swift-sdks-path`. This record does not establish an explicit override for
that optional SDK bundle store. It also does not show a write to a global SDK
store. Both the package platform and normal plist remain at deployment 15.0.
The icon's new canonical Swift/SDK/module-cache arguments have source review
only, since the package stopped before that step.

The source diff does not alter feed/key data, Sparkle configuration, signing
intent validation, archive generation, or priority code. The same Swift build
flags reach both build and bin-path lookup. The same private scratch reaches
Sparkle embedding. The release wrapper preserves the opt-in environment when
it creates the bundle environment. Enabled credential flags affect SwiftPM's
Keychain/netrc lookup. They are not a claim that all later release tools avoid
all OS services. Signing-policy/config checks preceded builds and selected
ordinary ad hoc intent; no signing operation was reached.

The final outer profile denies writes except the new owned root, `/dev/null`,
and the inherited private package/smoke temporary patterns. It denies network
except loopback 65440 and the scoped smoke Unix socket patterns. Relative to
the prior profile, the owned root and selected loopback port change; the denial
structure is retained. Between this real manifest and package invocation the
profile bytes/hash are identical. There is no added Darwin-temp, shared cache,
Xcode-tree, or external-network allowance. This is a write/network policy,
not a read-denial claim. Saved policy and successful commands do not constitute
an independent negative test of every denied operation.

The frozen controller sets absolute NI15 before starting each managed command.
Existing `priority.py` sets/verifies 15 and then execs, including nested calls;
it does not add a relative nice increment. Recalculation gives **845 package
samples / 4,271 observations / 85 observed PIDs**, all NI15, from
0.241415500 to 84.651779042 seconds; maximum gap **0.439747458 seconds**.
There are 884 Rust-compiler observations, 21 clang observations, and one linker
observation. There is no sampled Swift frontend, codesign, or product daemon.
The manifest has 140 samples / 139 observations of one parent identity and no
sampled frontend. Logs prove its compiler commands occurred; they do not measure
those compiler priorities. Short children and unsampled intervals remain open.

Cleanup checks only known sampled PID/start/group identities. It filters newly
attributed children to the managed process group; an unseen or group-escaping
descendant is not covered. This attempt needed no cleanup signal and recorded
no known identity remaining, in **0.000332333 seconds**, against a 60-second
budget. The package temporary path was removed. This is not universal
descendant-cleanup proof. Saved first exclusive binds cover all ten ports
65440-65449 both before and after, without reuse/retry. The lsof result was
incomplete because of its unrelated mount warning, so it supplies no stronger
port-inspection claim. I did not make any new process or port query.

The saved command trace and exclusive guards contain one manifest operation
and one package operation, then the explicit failed-package stop. The budget's
initial `manifest_commands_executed: 0` is a pre-execution declaration, not its
post-run count. No recipe 2/3 or second package is recorded. The profile hook
only observes real calls/returns and subprocess starts; it replaces no callable
or return result. The bundle exception lacks a completed return object, but
the outer result and unmodified stderr bind exit 1. Both earlier failed runs,
including the original 114-removal breach and byte/link recovery, remain failed
and retained. New-baseline equality does not close their metadata limitation.

There are no smoke commands, daemon starts, or smoke returns. The staged and
extracted payload inventories are empty; artifacts/signature commands are empty
and artifact bindings are `{}`. Whole-app completion, bundle assembly, signing,
ZIP/link/mode roundtrip, accepted BUILD/sidecars, staged/extracted IPC flows,
App UI and private launcher acceptance were not reached. No later product or
compiler execution appears in the held evidence. This audit cannot establish
activity outside those retained records.

## Proposed next qualification command; not executed

If a later task qualifies this path, use one app-only **release** build under
the same denial structure, with verbose output. Keep `--arch arm64`, the native
engine, SDK and all shared flags. Do not use the prior unrestricted-filesystem
DEBUG command as a substitute, and do not reset either spent guard.

Use a new owned root and new output/guard files, with only the pinned seeds and
fresh build/module/cache state. `QUALIFY_REPO` must identify a separately owned,
source-bound checkout of the held source; `QUALIFY_ROOT` must identify that new
private root. The controller must supply the saved allowlisted environment
with private-root paths remapped, Rust 1.99.1/jobs 2/deployment 15.0 unchanged,
and no signing variables. The new policy must only substitute its owned root
in the reviewed profile, without widening writes or network. Under that
prepared environment, the narrow proposed payload is:

```sh
/usr/bin/sandbox-exec -f "$QUALIFY_ROOT/package.sb" \
  /Users/serge/.local/bin/python3.11 "$QUALIFY_REPO/tools/release/priority.py" -- \
  /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swift \
  build --package-path "$QUALIFY_REPO/apps/macos" \
  --scratch-path "$QUALIFY_ROOT/swift" -c release --jobs 2 \
  --disable-automatic-resolution --arch arm64 --build-system native \
  --disable-sandbox --cache-path "$QUALIFY_ROOT/swift-cache" \
  --config-path "$QUALIFY_ROOT/swift-config" \
  --security-path "$QUALIFY_ROOT/swift-security" --manifest-cache local \
  --sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk \
  --disable-keychain --disable-netrc --product Openergy --verbose
```

This isolates the unresolved app build from Cargo, bundling, signing, and
product execution. It still needs a finite one-shot controller, absolute NI15
identity observations, complete first streams, bounded cleanup and unchanged
source/seed/old-cache checks. A pass would qualify only that recorded app build,
not packaging or product IPC acceptance. A failure must remain intact and stop
the task. No such command or new qualification state was created in this review.
