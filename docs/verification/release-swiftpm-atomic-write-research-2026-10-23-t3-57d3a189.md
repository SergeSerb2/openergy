# Isolated SwiftPM output-map I/O: causal research

2026-10-23, America/Vancouver. Owned worktree: `t3-57d3a189`, branch
`t3/diagnose-swiftpm-output-map-io`, base `7328df364908fa237cdd242fc0d5ea950ad126a6`.

**Cause remains unproved. No supported per-process repair was established.**
The new source finding is a Darwin-specific atomic-write branch that requests
an item-replacement directory. Public Foundation temp lookup can prefer the
Darwin directory service over `TMPDIR`. Thus private `TMPDIR` and a writable
destination do not establish that all atomic-write operations stay private.
The saved failures do not show which branch, path, or syscall failed.

## Evidence retained and used

These are existing bindings, not a new acceptance or integrity audit:

- Source: `fc33bdc0cbb261d0a096ec459b5ba0dce25ae9ec`.
- First failure: `567eb51d48d88731ecaf0272b1be98e35f1e3a93`, prefix
  `docs/verification/release-swift-isolation-2026-10-23-t3-bc17f311/` (`A`).
- Complete review: `899bdb1af594c3ee1949dc9e83b5272c425f7d63`,
  `docs/verification/release-swift-isolation-review-2026-10-23-t3-73b2eeaf.md`.
- Guard source: `46d2c3b604ca9414cc75fba598f5c0a9e5a7e8dc`.
- App-only failure: `6badf9582ad7faad8404e1d169b1ab4575fa4947`, prefix
  `docs/verification/release-swift-app-qualification-2026-10-23-t3-0966fd74/` (`B`).

I read the complete source changes, bundle script, both READMEs and review.
All 101 files in A and 85 files in B were read/decoded as data, including the
complete streams, recipes, controllers, inventories, result records and guards.
No saved producer was imported or executed. Existing ledger/hash conclusions
are carried forward; retained originals and old cache trees were not reaudited.

A records the real manifest `describe` passing in 10.896334334 seconds. Its
package command then failed in 84.697217875 seconds. B records one app-only
command failing in **18.165488708 seconds**, exit 1, without Cargo or packaging.
Both failures name the release `Openergy.build/output-file-map.json`. Neither
produced an Openergy application frontend invocation or output map. B's final
unprivileged sandbox-log query returned zero records; that excludes no cause.
These are audited saved observations, not fresh reproductions.
The direct records are A's `raw/manifest-1-result.json.gz` and
`raw/package-result.json.gz`; B's `raw/app-build-result.json.gz`, complete
`raw/app-build.stderr.gz` (decoded lines 1–11),
`raw/first-output-inspection.json.gz` and
`raw/sandbox-log-corrected-result.json.gz`.

The guard now rejects the isolated universal combination before `dirname` or
`uname`. It does not explain either arm64 failure. Source remains unmerged and
the full native build remains unqualified. The old successful DEBUG run used
network denial alone; it supplies no filesystem-containment control.

## Actual command and shipped-version boundary

B's `raw/build-recipe.json.gz` and `raw/app-build-start.json.gz` bind the same
argv. Recipe SHA-256:
`ccdb2c82a71f9027c1857a6fde70992454f1ceee521cbe3b062c79f3093ec4fe`.
Define `P` as the saved root
`/Users/serge/.t3/worktrees/openergy/t3-0966fd74/.build/release-swift-20261023-t3-0966fd74-once`
and `X` as
`/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin`.
This is a record of the failed command, not a replay instruction:

```text
/usr/bin/sandbox-exec -f P/package.sb
  /Users/serge/.local/bin/python3.11 P/repo/tools/release/priority.py --
  X/swift build --package-path P/repo/apps/macos --scratch-path P/swift
  -c release --jobs 2 --disable-automatic-resolution --arch arm64
  --build-system native --disable-sandbox --cache-path P/swift-cache
  --config-path P/swift-config --security-path P/swift-security
  --manifest-cache local
  --sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
  --disable-keychain --disable-netrc --product Openergy --verbose
```

Its cwd was `P/repo`. The existing environment binds private HOME, TMPDIR,
module/cache/config/security state and canonical compiler overrides. B's
`raw/package.sb` denies writes outside P, `/dev/null`, and the exact inherited
package/smoke temp patterns; policy SHA-256 is
`d5d45ebb37593f6bcd8b08ebb266953252b4542746e4ad5a65584f0a16308089`.
No global Darwin-temp or Xcode-cache allowance is available.

A's `raw/swift-version.stdout.gz`, `raw/xcode-version.stdout.gz`, and
`raw/paths-bound.json.gz`, plus B's `raw/tools-binding.json.gz`, bind Xcode
**27.1 (27A9275)**, Apple Swift **6.4 (swiftlang-6.4.0.34.1,
clang-2100.3.34.1)**, SDK **27.0**, Rust **1.99.1**, jobs **2**, deployment
**15.0**. Manifest frontend targets of macOS 14.0 are host manifest commands.
They do not establish an application architecture or deployment mismatch.

New static reads only: `X/swift-package` still matches the saved SHA-256
`2ff9c4066c4c493c412a05a6269c882434839d0a3a0da590e1ed0a91f2e5ce9d`.
Its Mach-O UUID is `43a997c7-a135-3323-9e30-b72209d740fb`; its load commands name
`@rpath/libSwiftToolsSupport.dylib` and system Foundation, with link-time
Foundation version **5027.0.62**. This is not the loaded OS framework version.
The current candidate `X/../lib/libSwiftToolsSupport.dylib` has UUID
`49b6cb89-59cc-3fda-b8a9-30314f6d3360`, SHA-256
`cdb917d5e35b153c63683832d152e1bdd68a053b255952b39afa6cd11d631cb7`, and an import
of `Foundation.Data.write(to:options:)`. No historical loader trace binds that
candidate or the actual Foundation image to the failed process.

The inspected SDK headers provide API contracts, not implementation source.
The scoped toolchain `usr/share/swift` and configured SwiftToolsSupport include
location supplied no matching implementation. Public `swift-6.4-RELEASE` tag
lookups returned 404 in SwiftPM and swift-foundation. The compiler version,
Mach-O UUID and link-time SDK version do not identify Apple's SwiftPM,
support-library or Foundation source revisions.
The saved compiler's default target reports macOS 27.2.0; it does not bind
the historical OS build or loaded Foundation UUID.

## Source path, temporary directories and lost error detail

1. **Planning writes the map.** The existing public SwiftPM binding
   `80c1fcb89babc8da56611dbb962c545662523b5e`,
   [SwiftModuleBuildDescription.swift:765–800](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Build/BuildDescription/SwiftModuleBuildDescription.swift#L765-L800),
   defaults `writeOutputFileMap` to true while forming compiler arguments.
   [Lines 989–990](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Build/BuildDescription/SwiftModuleBuildDescription.swift#L989-L990)
   create its parent and write with `atomically: true`. This can throw before
   any application compiler starts. It is compatible with the saved stop.

2. **Foundation receives the atomic write.** The existing support-library
   binding `c574915fe88e942e4c4c93376f022daa2ecf05f9`,
   [FileSystem.swift:687–700](https://github.com/swiftlang/swift-tools-support-core/blob/c574915fe88e942e4c4c93376f022daa2ecf05f9/Sources/TSCBasic/FileSystem.swift#L687-L700),
   calls `Data.write(... options: .atomic)` and maps the caught NSError.
   The installed SDK 27.0 header
   `System/Library/Frameworks/Foundation.framework/Headers/NSData.h:37–40`
   describes an “auxiliary file first”; lines 129–135 describe the subsequent
   move. The [Apple atomic API contract](https://developer.apple.com/documentation/foundation/nsdata/writingoptions/atomic)
   does not promise a caller-selected temporary directory.

3. **The Darwin branch matters.** New public Foundation snapshot
   `d61ccf92d8cdb77075b24b17848ee55ab142fa4c` (2026-10-23),
   [Data+Writing.swift:320–376](https://github.com/swiftlang/swift-foundation/blob/d61ccf92d8cdb77075b24b17848ee55ab142fa4c/Sources/FoundationEssentials/Data/Data%2BWriting.swift#L320-L376),
   tests `_foundation_sandbox_check(getpid(), nil) != 0` under
   `FOUNDATION_FRAMEWORK`. That branch requests `.itemReplacementDirectory`
   for the destination; failure propagates without taking the sibling-temp
   branch. The other branch tries private `_amkrtemp`, then a sibling
   `.dat.nosync…` file
   ([lines 176–310](https://github.com/swiftlang/swift-foundation/blob/d61ccf92d8cdb77075b24b17848ee55ab142fa4c/Sources/FoundationEssentials/Data/Data%2BWriting.swift#L176-L310)).
   Later operations include file writes, fsync and rename. None was traced here.

4. **Portable replacement code does not bind Darwin behavior.** In the same
   snapshot, [FileManager+Directories.swift:73–138](https://github.com/swiftlang/swift-foundation/blob/d61ccf92d8cdb77075b24b17848ee55ab142fa4c/Sources/FoundationEssentials/FileManager/FileManager%2BDirectories.swift#L73-L138),
   the visible temp/volume algorithm is guarded by `!FOUNDATION_FRAMEWORK`.
   The framework branch instead calls private `_URLForReplacingItem(at:)`.
   SDK `NSFileManager.h:195–211` says only the supplied URL's volume determines
   the replacement location. The [Apple API](https://developer.apple.com/documentation/foundation/filemanager/url(for:in:appropriatefor:create:))
   provides no explicit directory-path argument for that location.

5. **TMPDIR is not a universal override.** Foundation's pinned
   [String+Path.swift:473–520](https://github.com/swiftlang/swift-foundation/blob/d61ccf92d8cdb77075b24b17848ee55ab142fa4c/Sources/FoundationEssentials/String/String%2BPath.swift#L473-L520)
   checks `confstr(_CS_DARWIN_USER_TEMP_DIR, …)` before `TMPDIR` on Darwin.
   Apple's public **Libc-1752.100.10**, commit
   `71bbe350ab79eef58113991d817ccc6165061a64`,
   [confstr.c:225–247](https://github.com/apple-oss-distributions/Libc/blob/71bbe350ab79eef58113991d817ccc6165061a64/gen/confstr.c#L225-L247),
   asks `__dirhelper` first and uses `TMPDIR` only on helper failure. These are
   source models, not proof of this process's selected directory or Libc build.
   Official [PR #1064](https://github.com/swiftlang/swift-foundation/pull/1064)
   (2024-12-02) fixed an earlier bug that always fell back to TMPDIR.
   [PR #728](https://github.com/swiftlang/swift-foundation/pull/728)
   (2024-07-11) documents fallback on confstr failure.
   [PR #1477](https://github.com/swiftlang/swift-foundation/pull/1477)
   (2025-08-21) changed `_amkrtemp` handling and added failure detail. Older
   implementations and these merged changes cannot be assumed to match Apple
   Swift 6.4's OS Foundation.

6. **The printed message loses operation and NSError context.** Pinned
   [TSCBasic mapping:107–199](https://github.com/swiftlang/swift-tools-support-core/blob/c574915fe88e942e4c4c93376f022daa2ecf05f9/Sources/TSCBasic/FileSystem.swift#L107-L199)
   maps an immediate underlying POSIX error first. ENOENT becomes `.noEntry`;
   EPERM falls through to `.ioError(1)`. Some other NSError domains retain
   their numeric code too. SwiftPM's pinned
   [Diagnostics.swift:243–251](https://github.com/swiftlang/swift-package-manager/blob/80c1fcb89babc8da56611dbb962c545662523b5e/Sources/Workspace/Diagnostics.swift#L243-L251)
   formats `.ioError` as “while reading” regardless of the operation that threw.
   The displayed path can be the requested map, even when an auxiliary operation
   failed. The diagnostic does not retain a full NSError chain or temporary path.

## Finding and one bounded next action

**Source-backed candidate, not established cause:** under the outer sandbox,
Foundation may fail to create or use an item-replacement location outside the
owned root, despite private TMPDIR and writable output parents. The existing
policy would deny such writes. But no saved record identifies that location,
the sandbox-check return value, the failing operation, or its underlying error.
A sibling-file, write, fsync, rename, or other mapped error is not excluded.
Code 1 and the absent map alone prove none of these alternatives.

The inspected public APIs, recorded CLI help and source provide no supported
per-process setting that forces this framework replacement path into P.
HOME/CFFIXED_USER_HOME and SwiftPM cache flags do not supply that argument;
TMPDIR is already private. This search does not prove that no such facility
exists. Private symbols, guessed environment settings, compiler flag changes,
toolchain swaps and global write allowances are not supported repairs.

**Missing prerequisite:** an approved disposable macOS environment with the
bound Xcode/toolchain and process-scoped Foundation/error and filesystem
instrumentation available. Its OS build and loaded library UUIDs must be
recorded; exact historical Foundation equivalence remains open. Public source
stops at private replacement helpers; the present evidence has neither their
shipped implementation nor a trace.

**One next action, for a separate authorized task:** capture one instrumented
execution of B's exact app-only command in that environment, keeping its policy
structure and private-state bindings. Restrict capture to the owned SwiftPM
process and its atomic-map operation. Record the first failing syscall's
operation, resolved path(s), return/errno, and the Foundation NSError
domain/code, underlying domain/code and file/debug-description keys before
TSCBasic conversion. Stop at that first planning failure, or at 30 seconds or
1 MiB of diagnostic output; preserve a bounded-capture failure if reached.
Keep product/owned compiler processes at absolute NI15. No host-wide logs,
micro-test, policy expansion or further build is part of this action. This
would acquire the missing causal evidence, not qualify a repair.

## This task's boundary

Only this document is changed. All old evidence, first failures and spent
guards remain untouched. New work used Git/file reads, data decoding, scoped
SDK/header and Mach-O reads, and public primary-source requests. No manifest,
build, package, compiler, filesystem probe, sandbox or product command ran.
No tests, mocks, product runtime, permission request, host change, credential access,
delegation, push or publication occurred. The three provider-stopped tasks
remain out of scope. **Zero new runtime or qualification evidence.**
