# Prepared stopped-target update fixtures

This is file-only preparation for a future real Sparkle CLI installation check.
No CLI, updater installer, old/candidate app, server, launchd job or target daemon
was executed or started here. Installation and relaunch remain unverified.

The current proposed scope is [operation-v2-DO-NOT-RUN.json](operation-v2-DO-NOT-RUN.json).
It supersedes the incomplete helper/cache-only scope in the original preparation
snapshot and `raw/operation-DO-NOT-RUN.json`. Those original records remain unchanged.

The private fixture is `/private/tmp/oe-stopped-update-llil939f`. Its unique host
ID is `dev.openergy.updater-fixture.5b4a5cb7d61d444eaf4a3d673e1df4a5`.
The old bundle has build 1; the candidate has build 2. Both are private copies
of the previously verified universal app, with fixture-only metadata and public
key. Their nested signatures and archive round-trip trees verified. The candidate
ZIP is 7,492,869 bytes. It was not uploaded.

A new ephemeral CryptoKit Ed25519 key signed the archive and two feeds. The
official Sparkle tool accepted the archive signature and rejected a signature
whose first byte was changed. Both complete feeds have valid feed signatures.
Their unsigned content differs only in that enclosure signature. The archive
and bundled public key are identical across cases. This is offline artifact
validation, not signature rejection by a running updater. The temporary private
seed and verification container were removed. The production signing key was
never read.

The original compiled CLI remains unaccepted as recorded in the
[build-effects report](../sparkle-cli-build-preparation-2026-10-23/README.md).
A new private copy keeps its compiled CLI code, replaces only its private embedded
framework with the exact cached official Sparkle 2.10.0 framework, gives the
driver a fixture-specific ID/defaults domain, and applies a local ad-hoc outer
signature with options `0`. Deep/strict resource-seal verification and arm64
metadata checks pass for this copy. The original CLI binary still has its exact
recorded hash. No Xcode invocation, LaunchServices registration or execution-policy
registration command occurred during this file-only repair. This proves a resource
seal, not library loading, installation readiness, Developer ID or notarization.

The first preparation controller exited 1 only while writing final metadata: it
tried to name the external signer relative to the fixture root. All 14 preparation
commands had already completed with their expected exits, and its `finally`
removed the seed. A separate finalizer recovered metadata and prepared the private
driver without repeating signing or packaging. The original tool-stream error was
observed by the parent, but its initial controller and stdout/stderr were not frozen
as files before correction. The retained corrected controller is explicitly a later
revision and was not rerun. This preservation limit is kept in `results.json`.

The v2 proposal preserves the original negative/positive CLI commands, private
paths and loopback feed URLs. The future controller must own
`127.0.0.1:25600` before any request, check all immutable fixture hashes, require
that the unique target is stopped, and inventory fixture helper/cache effects.
Sparkle's pinned progress agent only relaunches a target it observed alive at
setup. These bundles have no argument-free private bootstrap and must not be
launched. A running-target or relaunch test is outside this prepared operation.

Executing the proposed operation requires scope for fixture-specific temporary
launchd jobs and helper caches, installation metadata changes, timestamp updates
that notify LaunchServices, and Gatekeeper assessment. Pinned Sparkle's
`Autoupdate/SUPlainInstaller.m:82-115,354-383` includes the timestamp notification
and conditional `/usr/bin/gktool scan`. The ad-hoc candidate meets the source
conditions for scanning on the same-volume path on this supported OS. Neither
effect requires target relaunch. These are source-derived effects, not an observed
installer run. Persistent registration/assessment state is not proven temporary
or fully reversible; job/cache cleanup would not establish full shared-state
restoration. Normal scanning and notifications must remain enabled.

Current temporary-files/no-service-change limits remain in force. The prepared
copy's successful resource seal does not authorize execution. The source pin,
hash, current proposal and preservation bridge are recorded in
[scope-r2-binding.json](scope-r2-binding.json).

[results.json](results.json) binds public metadata, command receipts, whole-bundle
trees and controller revisions. No executable, application archive, private key
or TLS identity is committed. The [corrected scope review R2](review-r2.md) found
no P1/P2 and closed R1's scope omission. Runtime acceptance remains open. The
execution-scope decision is pending; no approval is assumed.
