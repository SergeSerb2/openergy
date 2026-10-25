# Independent review of the completed M1 bundle evidence

Date: 2026-10-24, America/Vancouver. State label:
`m1-resume-private-20261024/completed-build-aa30bb8df0bd`.

**The saved evidence supports one completed ordinary bundle build. No concrete
P1 or P2 was found in this evidence scope. Native app and release acceptance
remain UNVERIFIED.** One supplied timing label needs correction below.

I read inert files, Git objects, JSON, source, tar members, plists and Mach-O
data. I streamed both archives without extracting the app. I did not import or
execute supplied code, contact a host, probe a running system, build, sign,
launch an app or daemon, use the network, or delegate work. Only this review
and [review-bindings.json](review-bindings.json) were added. These documents
create no executable receipt or approval.

## Independent comparisons

- The retention receipt, export receipt and both archive hashes match the four
  supplied hashes. Archive sizes are 4,692,177 and 78,260 bytes.
- All 44 evidence tar members match the export manifest and retained raw files
  in name, length and SHA-256. There are no duplicate, extra, missing, link or
  special evidence members. The export's result equals `build/result.json`.
- All 167 app inventory records match actual tar member types, modes, lengths,
  hashes and link text. The archive has one additional app-root directory:
  168 members total, 95 regular files, 10,050,709 regular bytes, 9 symlinks and
  64 directories including the root. All symlinks resolve within the archived
  app, including paths through the framework's `Versions/Current` symlink.
- Retained `controller.py`, `swift-entry.sh` and `source-bindings.json` are
  byte-identical to commit `7f20c9cdd97c1dc313f007b879babc726acbb914`.
  R4 commit `fb7b0d97ba4842021d8315956800c9aa8ec20dfe` names that candidate
  and records SOURCE_CLEAR with zero P1/P2 findings. The controller SHA-256 is
  `cd5dc620e6adbec09798e600a99f55c0368b8eeff10d1117e1f7793ca1baf4ee`.
- The prepared and executed operations differ only in `execution` and the
  absolute deadline. The executed operation equals the runtime bound operation
  byte-for-byte; its digest matches both start and launch-intent records. All
  three recipe digests match. The launch intent records retry count zero.
- All 702 bound non-docs source records match raw Git blob bytes, lengths and
  executable modes at `ed7d96bf53da1a539fa2acde3b7133613221fd4b`, totaling
  4,773,386 bytes. The saved workspace closure equals the reviewed closure:
  11 members, 13 manifests and 24 local path edges. The source archive and live
  build tree are not members of this export; their pre/post checks are recorded
  controller behavior, not a second independent inspection of the build host.

## Result and claim correction

There is one recorded resolve and one recorded bundle command. Both exit records
show zero. Their monotonic command-to-exit spans are **9.432977791 seconds** and
**87.455429375 seconds**. The output records daemon and app compilation, bundle
assembly, ad-hoc signing and completion, consistent with the retained artifact.

**101.050310709 seconds is not total controller time.** In the exact controller,
`started` is set after input validation, extraction and environment preparation.
`elapsed_operation_seconds` ends when the worker prepares the bundle inventory.
It excludes earlier controller work and later parent finalization. The first
retained process sample through the terminal result spans **103.025884334
seconds**, already longer. A complete controller invocation duration is not
retained. This corrects a claim label; it does not invalidate bundle completion.

The 2,008 process rows contain 90 PIDs and 90 PID/UID/start identities. Times are
nondecreasing; every row has the keeper's UID and process group and nice15.
The keeper identity is stable across 491 observations; the last two rows are
keeper observations. The exact controller permits normal release only after an
inventory containing the keeper alone. The terminal result reports keeper
reaping with wait status zero, no remaining sampled identities, no stop request
and no uncertainty. No recovery
signal records are present. These observations support the scoped result, not
universal descendant containment. The 14,525-byte output matches the terminal
byte count; the result reports zero dropped bytes and completion before deadline.

## Artifact scope and limits

The saved tool records and exact controller guards support the ordinary arm64,
macOS27.0.1 / OS build26A434, Xcode27.1 build27A9275, Swift6.4, SDK27.0 and
Rust/Cargo1.99.1 baseline. Both product Mach-O files are thin arm64 with minimum
macOS15.0 and ad-hoc CodeDirectory flags. The five bundled Sparkle executables
retain arm64 and x86_64 slices; this is not a universal product build. Header
inspection is not an independent cryptographic signature or Gatekeeper check.

The actual app plist names `Openergy`, version0.1.0/build1, deployment15.0 and
`SUEnableAutomaticChecks=false`. It has no `OpenergyPrivateUpdateFixture` key,
feed URL or public update key. None of the five Info.plists has private-fixture
metadata, and the inventory has no private launcher. This establishes ordinary
bundle metadata, not removal of fixture-related source code or updater proof.

The log preserves **rust-objcopy SIGABRT (signal6), missing
`@rpath/libLLVM.dylib`**. No successful Rust stripping claim is supported.
Two Swift `String(cString:)` deprecation warnings also remain in the saved log.
The build's isolated daemon `version` check is inherent in the reviewed bundle
script; it establishes no daemon service or native app flow.

This export proves no native UI/input/clipboard flow, update flow, trusted
distribution, notarization, release acceptance, hard resource quota, continuous
process observation or host restoration. Filesystem timestamps, ACLs and xattrs
were not preserved. The old failed isolation result stays failed, and its Swift
atomic-write cause stays unproved. Ordinary acceptance stays **FAILED / NOT
ACCEPTED, 23/25**; Windows native stop stays **FAILED / UNVERIFIED**. The expired
Sparkle operation stays closed. No historical predicate or spent guard changed.
