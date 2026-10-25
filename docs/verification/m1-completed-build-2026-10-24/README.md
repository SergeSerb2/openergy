# Fresh M1 bundle completed; runtime acceptance remains open

One reviewed ordinary arm64 build completed on the M1. Dependency resolution and
`apps/macos/scripts/bundle.sh` both returned exit 0. No build retry ran.
Product source is `ed7d96bf53da1a539fa2acde3b7133613221fd4b`.
The exact executed controller is source R3
`7f20c9cdd97c1dc313f007b879babc726acbb914`, independently cleared by
[R4](../m1-sampler-R4-review-2026-10-24-t3-ecd3814b/REVIEW.md),
review commit `fb7b0d97ba4842021d8315956800c9aa8ec20dfe`.
This result follows the earlier [disabled machine binding](../m1-sampler-r3-prepared-binding-2026-10-24/README.md).
A separate exclusive operation file enabled this one invocation. The earlier
prepared operations remain unchanged.

| Saved observation | Result |
|---|---|
| Worker operation | 101.05 s, before its absolute deadline |
| Dependency resolution | 9.43 s, exit 0 |
| Bundling | 87.46 s, exit 0 |
| Bundle inventory | 167 entries; 95 regular files; 9 symlinks |
| Regular file bytes | 10,050,709 bytes (9.59 MiB) |
| Observed process rows | 2,008 rows, 90 unique PIDs, all nice 15 |
| Owned keeper | Reaped with exit 0; no recovery signals requested |
| Remaining sampled identities | None at final observation |
| Retention | Exact app archive and 44 evidence files verified locally |

The target used macOS 27.0.1, Xcode 27.1, Swift 6.4, SDK 27.0 and Rust 1.99.1.
The build used private source/tool/cache roots, locked/offline Cargo, two jobs,
arm64 and deployment target 15.0. The ordinary bundle has ad-hoc signatures,
automatic update checks disabled, and no private update fixture metadata.
Its bundle script also ran the isolated bundled daemon's version command.
No app launch, daemon start, IPC model flow, UI input or updater run occurred.
No named signing identity, timestamp or notarization was used.

The output preserves a Rust stripping warning: `rust-objcopy` aborted because
`@rpath/libLLVM.dylib` was missing. Successful Rust stripping is not claimed.
Two Swift `init(cString:)` deprecation warnings also remain in the saved log.
The bundle result does not erase these warnings.

The [machine-readable result](result.json) binds the operation, source, review,
retention receipts and archives. The [44-record index](private-evidence-index.json)
binds all retained raw evidence files without publishing private paths.
Private archive labels are relative to the authorized verification state parent.
The app archive is 4,692,177 bytes, SHA-256
`7eb92de1ae6af988f8925546624a9f69fe20309aa8ea15f13fcf0b9ddf1cea21`.
It remains data locally. Tar retention records file bytes, modes and internal
symlinks; it does not preserve all filesystem timestamps, ACLs or extended attributes.

Saved process rows establish sampled priority and observed cleanup only.
They do not prove every short-lived, escaped or OS-managed process was observed.
There is no universal process containment or full shared OS-state restoration claim.
The M1 uses a new normal-tooling baseline. Its successful build does not establish
the cause of the old isolated Swift `output-file-map.json` I/O failures.
Those failures and the prior ordinary 23/25 FAILED / NOT ACCEPTED result remain unchanged.
The expired Sparkle update approval remains closed, with no retry authorized.

The [independent saved-evidence review](../m1-completed-build-review-2026-10-24-t3-10d5c28a/REVIEW.md)
found no concrete P1/P2. It corrected the timing label: 101.05 seconds covers the
worker after preparation through inventory, not the full controller. The separate
local SSH invocation envelope is 103.82 seconds and includes launch/collection
overhead. An exact full controller duration is not claimed.

Next: bind the reviewed ordinary headless flow to this retained bundle, then
perform one real flow with the existing limits.
Native UI/input, current macOS 27.2 behavior, update/relaunch, release trust and
matched performance comparison remain separate open gates.
