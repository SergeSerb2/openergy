# M1 build prerequisites resumed — 2026-10-24

The user brought the separate M1 test Mac online and verified its SSH ED25519
fingerprint locally. Both saved SSH aliases presented a different key. A local
hostname route matched the user-verified fingerprint and passed strict SSH host
checking. The parent used a new task-owned known-hosts file. No existing SSH
configuration or saved host-key entry was replaced.

After the user installed and opened Xcode, a new metadata-only SSH check found:

| Prerequisite | Observed value |
| --- | --- |
| Architecture | arm64 |
| macOS | 27.0.1, build 26A434 |
| Xcode | 27.1, build 27A9275 |
| Xcode first-launch check | Exit 0 |
| Swift | 6.4, swiftlang-6.4.0.34.1, clang-2100.3.34.1 |
| macOS SDK | 27.0 |
| Desktop | Logged in |
| Available disk | 391,217,152 KiB at the check |
| System Python | 3.9.6 |

The compiler/SDK versions match the earlier build records. The OS differs from
the original failing host. This check does not reproduce or explain that
host's Swift atomic-write failure. It does not qualify a disposable diagnostic
environment or native macOS 27.2 focus behavior. No app, native input, updater,
manifest resolution, build, signing or package command ran on the M1.

## Prepared local data

A new private 0700 directory retains three data archives. No old target,
Swift output, build database, workspace-state file, credentials or private key
was copied. The local data reader ran at actual nice 15.

| Archive | Bytes | Scope |
| --- | ---: | --- |
| `source.tar` | 3,215,360 | 435 regular product-source files from clean `ed7d96bf53da1a539fa2acde3b7133613221fd4b`; selected paths, not a full Git checkout |
| `rust-toolchain.tar.gz` | 223,257,861 | Existing pinned Rust 1.99.1 toolchain, 225 records and 639,160,693 regular bytes |
| `cargo-seed.tar.gz` | 22,988,930 | All 106 Cargo.lock registry crate archives, each checksum verified, plus selected sparse-index/config records; 210 files |

[build-inputs.json](build-inputs.json) binds archive and receipt hashes and the
private directory label relative to the existing verification-state parent.
The complete byte inventories remain private. The frozen data preparer is
evidence, not an instruction to rerun it. Its exclusive private-parent creation
refuses reuse. Two earlier preparation-reader errors are preserved in
[preparation-first-errors.json](preparation-first-errors.json). Neither started
a product/build command or changed the historical failed results.

Python 3.11+ prerequisite data and the new M1/Windows source-only operation
plans are delegated. [continuation.json](continuation.json) records their task
identities. Fresh plans require review and concrete prerequisite bindings
before execution. The Windows plan must use a separately identified fresh
baseline, not invented historical fixture paths.

## Boundaries

The original ordinary run remains FAILED / NOT ACCEPTED (23/25). The old native
Windows Off stop remains FAILED / UNVERIFIED. The closed Sparkle one-shot and
expired marker are not renewed or reused. No native/update permission is
inferred from this toolchain check. Current runtime, distribution and security
acceptance remain incomplete. No build artifact, release clearance or old
cache-metadata restoration is claimed.

No production code changed. Nothing was pushed or published. The resumed work
prepares new inputs on a separate baseline; the prior stopped experiments and
failed evidence remain unchanged.
