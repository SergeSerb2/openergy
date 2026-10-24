# Private update bootstrap: independent source and saved-build review

Reviewed on 2026-10-23 (America/Vancouver). Result: **two P2 findings; no P1
finding**. The original fail-closed and early-framework-ordering requirements
remain unmet. This review does not approve a future launch or update run.

## Exact scope

- Candidate: `a0342aef6b4e4712e7bf73c992b788f26d0e98da`.
- Verified actual Git parent: `5a9783738c5b649844805e1614da7ea37cfbf2bb`.
- Review checkout started clean at
  `1fd5144628c0494f1ebf561581e0d84fd7231523`. Candidate files were read as Git
  blobs, without checkout, cherry-pick, build, or product execution.
- Candidate source changes: the 466-line `PrivateUpdateFixture.swift` and one
  first-dispatch line in `OpenergyApp.swift`. All line references below refer to
  the candidate, not the review checkout's source files.
- Reviewed the complete helper, entry dispatch, relevant unchanged AppModel and
  DaemonSupervisor lifecycle code, README contract, build controller as text,
  saved compiler records, inventories, and manifests.

Only this uniquely named review document is owned by this review. No source or
existing evidence was changed. No further delegation, tests, mocks, build,
runtime, network, fixture, server, product CLI, installer, desktop, registry,
keychain, or live-process inspection was performed. No installed app, live
sharing, production config/TLS/default IPC, TCC, login item, service, account,
firewall, DNS, or toolchain default was changed. No TCP 24800 or UDP 5353 activity
was performed. The three provider-stopped tasks were not resumed.

## P2 findings

### R1: Refuse an unreadable Info.plist before normal dispatch

Location: `apps/macos/Sources/Openergy/App/PrivateUpdateFixture.swift:21-26`.

When `Bundle.main.infoDictionary` is nil, line 21 replaces the read failure with
an empty dictionary. The identifier then becomes an empty string, the metadata
key is absent, and the fixture-prefix check cannot detect the fixture. Line 26
returns to normal dispatch. For a no-argument executable entry, that reaches
`OpenergyApp.main()` at `OpenergyApp.swift:41`, including its shared model and
updater initialization at lines 75-76. No private environment, priority setup,
or fixture path validation has run on this branch.

Thus a fixture whose complete plist cannot be loaded can take the production
startup path instead of refusing. This is distinct from a successfully parsed
ordinary production plist with no fixture metadata, which must remain inert.
A malformed or missing whole plist can also hide the identifier used by the
missing-metadata guard. The source branch is sufficient to establish this
failure mode if the executable enters main; this review does not claim that
LaunchServices will launch every malformed bundle.

The candidate README at lines 57-59 explicitly excludes a malformed Info.plist
from the metadata-presence check and shifts validation to future packaging.
That exclusion narrows the original fail-closed requirement. A packaging check
does not make this executable branch fail closed. Do not accept the exclusion
as an approved change to the brief.

Required correction: distinguish a failed plist load from a valid ordinary
metadata-absent plist before returning to production dispatch. Refuse the
failure. If fixture identification must survive loss of the entire plist, its
failure check cannot rely only on that same unreadable dictionary. Preserve
the valid ordinary-bundle path and all production updater/feed/key gates.

### R2: The first image uses CF/Foundation before private-home setup

Locations: `PrivateUpdateFixture.swift:21`, `29-39`, `107-161`, `294-307`;
candidate README lines 191-197.

The external no-argument entry first reads `Bundle.main.infoDictionary`. It
then constructs Context, which uses Foundation URL, Bundle, Date, JSONDecoder,
and FileManager operations for its validation. Only after Context returns does
line 32 call `installEnvironment()`, which sets HOME and CFFIXED_USER_HOME.
The `execv` happens later at line 39. The first image has therefore already
entered CF/Foundation with the launching process's home environment. The saved
Mach-O record also lists AppKit, CoreFoundation, and Foundation as load
dependencies; a first line in main is not a stage before those frameworks load.

The same-PID re-exec supplies the private home before initialization of the
second image. It does not establish the requested ordering before CF/AppKit
initialization for the first image. The comment at helper line 35 and the README
describe the second image's ordering, which is a narrower guarantee than the
original early-framework/default-preference boundary.

This is a concrete ordering mismatch. It is **not** evidence that a particular
CF/AppKit initializer read or wrote a host preference. The saved compilation
cannot resolve those effects, and no runtime audit was authorized here.

Required correction: establish the private launch environment in an entry
stage before the app image/framework initialization that the requirement
protects, including the initial no-argument entry. Keep metadata activation
bundle-carried and fail closed, and preserve the ordinary production path.
Changing the README alone would narrow the requirement, not satisfy it.

## Other source checks and limits

No additional concrete P1/P2 issue was found within the reviewed scope.

- For successfully parsed present metadata, validation precedes normal argument
  dispatch. Exact metadata keys, UUIDs, build, port, marker agreement and finite
  lifetime of at most 21600 seconds are checked. A parsed fixture identifier with
  missing metadata refuses. Environment variables do not activate this helper.
- Root, config, TLS, socket, home, tmp, receipt and executable paths derive from
  metadata Root. Root and private directories require current UID and 0700.
  Private files require 0600 and one link. Canonical-path, tree, symlink and
  executable checks reject the stated escapes. Bundle links can resolve only
  within the validated bundle. Same-UID hostile mutation resistance is not
  claimed; path-check races are not reported as a new security guarantee.
- Strict config bytes select off role, literal loopback, empty remote, no
  autoconnect, TLS off and clipboard off. The installed environment selects the
  explicit config/TLS/socket/daemon/home paths, virtual platform and discovery
  off. Absolute nice 15 is checked before model creation or daemon spawn.
- Both accepted entries validate again. Only no arguments or the exact internal
  `--private-update-fixture --e2e` pair pass for present fixture metadata. The
  no-argument branch re-execs the validated executable. The second branch obtains
  an exclusive nonblocking flock on the precreated private lock. CLOEXEC prevents
  the daemon from inheriting the lock; the app retains it through its lifecycle.
- First explicit NSApplication/model use follows validation, environment setup,
  priority setup and the lock. The activation policy is prohibited. The explicit
  `AppModel(demo: nil)` sees the existing `--e2e` guard before its initializer can
  perform login integration. This path does not call the shared updater, normal
  SwiftUI scenes, AppEndToEnd, or an update/install operation.
- Receipt names use build/PID/fresh launch UUID/event. Writes are exclusive,
  no-follow, directory-relative, at 0600, with a 4096-byte payload bound and file
  fsync. Directory crash durability and receipt authentication are not claimed.
  The marker deadline is checked at entry; no automatic expiry shutdown is
  implemented or accepted as a runtime-lifetime guarantee here.
- The quit delegate cancels readiness, calls `model.shutdown()`, and writes the
  termination receipt. A failure or unexpected run-loop return has an exit-78
  path. DaemonSupervisor.stop can send SIGKILL without joining afterward. Neither
  shutdown return nor a receipt proves child exit; no child-gone field is added.

## Saved build and evidence audit

All checks here recomputed values from committed bytes. They are evidence
audits, not fresh build, artifact-runtime, or host-state observations.

- Manifest: exactly **20** raw records. Every byte count and SHA-256 matches.
  Manifest SHA-256:
  `bcae8d493b8eaf88b0745ef39c4283e462425c6859e80f962f2f3faaca780c78`.
- Recomputed ordered canonical raw inventory ID:
  `b7f7976f721697e1a9f3820d2e1377c46d66311c077d1154faf7f6b261f9bf08`.
- All **25** source inventory records, including **22** Swift source files,
  match the candidate blobs. Recomputed canonical source ID:
  `cd472e15eef4d845972771e52f5be1aae733453b20362f01498273b45ded1e76`.
  Both final owned source Git blob IDs and hashes also match.
- AppUpdater, AppEndToEnd, DaemonSupervisor, AppModel, Package.swift,
  Package.resolved and production Info.plist are byte-identical to the exact
  source parent. The entry diff adds exactly the first dispatch line. The normal
  parsed plist has identifier `dev.openergy.app` and no fixture metadata key.
  These comparisons preserve the production feed/key gates; they are not a new
  review of the stopped release-provenance/security task.
- Saved cache-before, cache-copy and cache-after inventories are byte-identical
  (**777** records each). The six pinned Sparkle source hashes match their saved
  cache inventory entries. Package.resolved pins Sparkle **2.10.0**, revision
  `eef1a539a373c1f1a320624b1130fc5de7b2e100`. Framework version **2.10.0/build
  2064** is recorded in the saved audit; this review did not reopen that artifact.
- The saved command selects jobs 2, private build/cache/config/security/home/tmp
  paths, disabled automatic resolution/update/prefetch/credential lookup, and
  deployment 15.0. Saved compile commands use DEBUG and
  `arm64-apple-macosx15.0`, with `-j2`, and link Sparkle. The source inventory
  includes the exact helper compiled in that whole app build.
- Saved result: one successful build, exit 0, no timeout. Controller elapsed
  **24.783185 seconds**; SwiftPM completion **23.58 seconds**. Recounted **664**
  process observations across **28** sampled PIDs, all nice **15**, including
  **227 actual swift-frontend observations**. Sampling can miss short processes.
- First stdout/stderr hashes match result.json and the manifest. No compiler
  error diagnostic was found. The two deprecation locations, helper lines 114
  and 150, repeat in batch output; deprecated SwiftPM flags remain recorded.
  Saved file/otool text describes arm64 Mach-O, minimum OS 15.0. The saved stdout
  includes SwiftPM's ad-hoc `codesign --sign -` step; this is not Developer ID or
  release-signing proof. No signing tool was invoked by this review.
- The saved product record reports 6,193,424 bytes and SHA-256
  `08c99b78519f942944ee80554f87a1d11b8f8d8413d81bfa9f19cb68dec33b9a`.
  This review did not inspect or execute the private built binary.
- The saved interruption reconciliation retains
  `write_stdin failed: Unknown process id 38332`, the finished result, unchanged
  first-stream hashes, reported no rerun, and the historical check of 29
  build/controller PIDs. These are inherited records, not fresh process checks.
  The README also preserves the earlier `zsh:1: no matches found: crates/openergy*`
  discovery failure. Neither failure was retried by this review.

The build evidence supports one arm64 DEBUG app compile/link. It supports no
universal bundle, current Rust daemon build, prepared fixture artifact, key,
archive, feed, isolation/refusal behavior, termination, install, or relaunch
acceptance. Rust was not invoked by this review or the saved Swift build.

The earlier normal-lifecycle review `a65324a` is context, not a review of this
helper. The user's 17:16 approval covers only stopped temporary case `5c2ed82`,
not a running target or relaunch. Future execution still requires a concrete
proposal and separate approval, including independent child-exit observations.
This review supplies no execution permission and launches nothing.
