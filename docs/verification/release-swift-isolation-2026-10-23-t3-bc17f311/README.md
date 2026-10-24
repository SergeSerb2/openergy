# Manifest isolation repair; single native package FAILED

Date: 2026-10-23, America/Vancouver. Owned worktree: `t3-bc17f311`.
Start: `3183a9ed0de957dd471c6e1e8a76d1331f190d22`.
Build source: `fc33bdc0cbb261d0a096ec459b5ba0dce25ae9ec`.

**One real manifest preflight passed. Exactly one new real package command
then ran and failed. No retry or later compiler/build/product command ran.**
The package exited 1 after **84.697217875 seconds**, without timeout.
No new archive or package artifact is accepted. No staged or extracted runtime
flow completed. This is not release clearance.

## Diagnosis and source change

The prior failed attempt at `3dcf8b2f7ccb1fa91d5fce43cdc4fd917c33dd03`
was read from its complete saved controller, profile, stdout and stderr in
`t3-8c51bae6`. It used `/usr/bin/swift`, attempted a Darwin user-temp
`xcrun_db` write outside the private root, and failed to apply SwiftPM's nested
sandbox. Its policy/Unix-IPC-only preflight had not evaluated the real manifest.
Those saved records were audited, not independently reproduced here.

Installed CLI help and tool paths were checked before source changes. Xcode
27.1 provides Swift 6.4 and SDK 27.0. Its SwiftPM default is `swiftbuild`; the
installed CLI supports the deprecated `native` engine, private cache/config/
security/SDK paths, `--manifest-cache local`, and `--disable-sandbox`.
The developer directory contains no separate `usr/bin/xcrun` executable.
A read-only genuine `/usr/bin/xcrun --no-cache` SDK query still attempted denied
Darwin-temp cache writes despite exiting 0. Its first exact output is retained.
No write permission was added for it. It is not treated as a successful cache
isolation repair.

The only source changes are `apps/macos/scripts/bundle.sh` and
`docs/releasing.md`. `OPENERGY_SWIFTPM_OUTER_SANDBOX=1` explicitly selects a
canonical Swift executable, native engine, private paths, and disables the
redundant inner SwiftPM sandbox. It also disables SwiftPM Keychain/netrc
credential lookups. The icon command uses that same Swift executable, SDK and
private module cache. The caller must already enforce an outer filesystem and
network policy. The normal default engine/sandbox path is unchanged.

**The new opt-in needs independent source review and full native build
validation before general release use.** No independent review was performed
in this task. Its manifest path passed; its whole-app package path remains
blocked. Shared feed/key, signing-intent, archive and priority contracts, and
app/daemon production source were not changed.

## Bounded preflight and exact recipe

[budget.json](raw/budget.json.gz) declared a maximum of three distinct corrected
manifest recipes before the first manifest command. **Only recipe 1 ran.**
Its SHA-256 is
`c06472c2d87555e9643bf8056eae02925ebfce5ae3609c6e01434eb382700c32`.
[manifest-recipe-1.json](raw/manifest-recipe-1.json.gz) retains the exact argv,
full allowlisted environment, working directory, source hashes and policy hash.
The actual command was canonical Xcode `swift package`, with the same private
paths and build flags used by the bundle opt-in, followed by
`--verbose describe --type json`. It evaluated the actual
`apps/macos/Package.swift`, produced the real Openergy description, and exited 0
in **10.896334334 seconds**. Its exact compiler/linker commands and results are
in [manifest stderr](raw/manifest-1.stderr.gz) and
[manifest stdout](raw/manifest-1.stdout.gz). It had no denied-cache-write,
`Operation not permitted`, or `sandbox_apply` diagnostic.

This preflight evaluated the project manifest. It did **not** exercise native
app output-file-map generation. That gap matters to the later failure.

All canonical Swift/clang/linker paths, executable hashes, SDK path/settings,
tool versions and recipe/plan hashes are bound in
[paths-bound.json](raw/paths-bound.json.gz). Rust remained **1.99.1**, jobs **2**,
Cargo offline, native **arm64**, deployment target **15.0**. HOME, temporary,
module/cache/config/security paths, Cargo home/target and Swift scratch were
private. Cargo target and Swift app build database/output were fresh. Only
pinned Sparkle artifacts, checkout and repository were copied from the old
Swift state; no old output/build database or full scratch clone was copied.
Cargo home received registry data only, with no credentials/config/bin.
Sparkle revision, payload hashes, private Git origin/alternates and resolved
pin were checked. Git index byte churn alone was permitted; no difference was
observed. Before packaging, source/cache/prior evidence invariance, path
binding, final-policy private Unix IPC and first exclusive ports all passed.

The outer [package.sb](raw/package.sb) retains filesystem write denial outside
the new owned root and the release tools' private package/smoke temp patterns.
It retains network denial except scoped loopback port 65440 and private smoke
Unix sockets. It grants no global Darwin/Xcode/user-cache writes or external
network access. No permission broadening was used.

## Single package result and stop boundary

[package-plan.json](raw/package-plan.json.gz), SHA-256
`d714e1afd54d81a145849baa319e3b6a15df0c2f64d905161796ffef9a038dda`,
binds the real `release.py package --port-range 65440-65449 --output <private-root>/output`
command to clean source commit `fc33bdc0cbb261d0a096ec459b5ba0dce25ae9ec`.
Identity and signing-purpose variables were unset. The ordinary ad hoc signing
policy was validated, but signing was not reached.

The daemon built from the empty Cargo target. Swift then failed during native
app build planning:

```text
error: encountered an I/O error (code: 1) while reading <private-root>/swift/arm64-apple-macosx/release/Openergy.build/output-file-map.json
```

[package stderr](raw/package.stderr.gz) and
[package stdout](raw/package.stdout.gz) retain the first complete output.
The earlier manifest cache/nested-sandbox diagnostics did not recur in this
package command. Read-only postfailure inspection found that the named map
file was absent. The cause of its failed generation/read remains unresolved;
no second build or package was used to investigate it.

The pinned toolchain also emitted its existing `rust-objcopy` missing
`@rpath/libLLVM.dylib` warning. It is retained verbatim; no toolchain was swapped.
The compiled daemon is retained privately, SHA-256
`9af7163b820e1cbb0edfba2f95088b45e9de27c0902bdeb8b9ed9a57640213b0`,
4,492,840 bytes, arm64 Mach-O. It was **not executed**.
[failure-limits.json](raw/failure-limits.json.gz) binds this file and the stop
boundary. Whole-app compilation/link completion, assembly, signatures, ZIP,
mode/link roundtrip, BUILD/sidecars, and both staged/extracted
version/absent-daemon/status/invalid-method/stop flows were not reached.
The integrated public nil-plist entry source is hash-bound, but neither App UI
nor private fixture/launcher runtime was exercised.

## Preservation, priority and cleanup

Before, after manifest, immediately before package, after package, and final
read-only audit, all old cache byte/link inventories and selected metadata
matched: **1,309 Swift / 1,159 Cargo entries**, and **1,725 Swift / 1,378 Cargo
metadata objects**. Both complete prior evidence folders and **89 private
records**, including both old one-shot guards and ledger-bound originals,
matched. All **718 tracked source inputs** matched the new baseline and their
build-commit Git blobs, with the declared Windows CRLF checkout conversion.
The only start-to-build source changes are the two build/config documentation
files named above. Source binding is saved in
[source-bindings.json](raw/source-bindings.json.gz).

New metadata checks include kind, mode, owner/group, link count, flags, mtime,
ctime, birthtime and link target; explicit read atimes are excluded. ACLs and
extended attributes were not checked. Equality against this new baseline does
**not** establish original prebreach metadata restoration, which remains
**UNVERIFIED**. Both older failed attempts remain FAILED and immutable,
including the first attempt's 114-removal breach and recorded byte/link recovery.
No old cache or record was restored, reset or edited here.

The controller set absolute NI15 before each owned command. The package has
**845 samples / 4,271 observations / 85 observed PIDs**, all at NI15. Actual Rust
compilers, canonical clang and an `ld` linker were observed with PID/start/
parent/group identities. Samples cover 0.241415500–84.651779042 seconds; maximum
gap was 0.439747458 seconds. No Swift frontend, codesign or product daemon was
observed in the package attempt. The manifest preflight's samples observed its
parent only; its compiler commands are in the real verbose log, but compiler
NI15 is not independently established by those samples. Short processes and
unsampled intervals remain unproven. See
[priority coverage](raw/priority-coverage.json.gz).

The watchdog was 900 seconds. Identity-bound cleanup took **0.000332333 seconds**
against a 60-second budget, needed no signal, and found no sampled owned
identity remaining. The owned package temporary path is absent. This does not
prove universal descendant cleanup. First exclusive loopback binds for every
port 65440–65449 passed before and after, with no reuse flag or retry. `lsof`
reported incomplete inspection of an unrelated mount; only its mount path is
redacted in exported port records. The exact originals remain private and
hash-bound. The exclusive binds supply the port evidence.

## Retained records and exclusions

[copy-ledger.json](copy-ledger.json) binds **97 exports** to retained private
originals under `/private/tmp/oe-swift-qualified-t3-bc17f311-lkmax1kz`.
[export-audit.json](export-audit.json) checks all original/export/payload hashes.
The complete new controller, observation hook, policy, exact recipes,
stdout/stderr, first-error records and fresh Swift state are retained. The
observer recorded real call/return results without replacing functions or
results. Controllers are frozen evidence, not replay instructions.

A source patch context mismatch was corrected before execution; no partial
source edit occurred. The first collector exported its active stdout before
its completion line. That empty snapshot was rebound to an exact retained
prefix original; the first collector/ledger, audit finding and final stdout
are retained. No compiler/product failure output was changed or replaced.

No GUI/computer use, app/launcher/CLI updater, installer, installed/running
target, relaunch, signing key/credential/Keychain lookup, Gatekeeper/Launch
Services, service/account/task/global setting, TCC/login item, native
input/clipboard, production config/TLS/default IPC, DNS/live sharing,
TCP24800/UDP5353, Docker, push, publication, or external network fixture was
used. No unit tests, mocks or delegation were added or run. The three
provider-stopped tasks remain out of scope. Saved-evidence/source review is
separate from independent runtime reproduction.
