# Pinned Sparkle input recovery

Date: 2026-10-24, America/Vancouver. Scope: bounded dependency retrieval as data.
Initial isolated checkout: `d8fa4945e88cb381b48aceab36eb105ee9268f23`.

**MATCH: the pinned dependency data is durably retained.** The complete historical
124-record file/symlink inventory matches the official archive and the private
copy. The ordinary app input set remains **INCOMPLETE**. The compiled app, original
icon, and original raw Swift workspace-state JSON remain missing according to the
prior retention result; those locations were not probed again.

[recovery-bindings.json](recovery-bindings.json) contains every historical and
recovered record, the source hashes, limits, and private receipt binding.
[validation.json](validation.json) binds the report and static checks.

## Exact provenance

Local `apps/macos/Package.swift` requires Sparkle exactly `2.10.0`.
Local `apps/macos/Package.resolved` pins revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100` in the official Sparkle repository.
Both files were read as data. No SwiftPM command or manifest evaluation ran.

The primary source is the official
[Package.swift at that exact revision](https://github.com/sparkle-project/Sparkle/blob/eef1a539a373c1f1a320624b1130fc5de7b2e100/Package.swift).
The retrieved [raw pinned manifest](https://raw.githubusercontent.com/sparkle-project/Sparkle/eef1a539a373c1f1a320624b1130fc5de7b2e100/Package.swift)
is 1,002 bytes, with SHA-256:
`a19cb9509d49c83d64a7ef43d7c06bbb90d739ec5b7b1703689b35acbe863b43`.

The manifest declares unique literal `version`, `tag`, `checksum`, and `url`
constants. Its unique Sparkle binary target refers to those constants. The reader
checked the exact URL template and replaced its single literal tag marker with
`2.10.0` as text. It did not execute or evaluate Swift code.

The resulting official archive URL is
[Sparkle-for-Swift-Package-Manager.zip, release 2.10.0](https://github.com/sparkle-project/Sparkle/releases/download/2.10.0/Sparkle-for-Swift-Package-Manager.zip).
The archive is **10,193,895 bytes**. Its downloaded and stored SHA-256 is:

`17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`

That hash equals both the checksum in the pinned primary manifest and the frozen
checksum in the original `raw/framework-resolution.json`. Equality was required
before opening the ZIP or materializing any archive member.

## Complete inventory proof

The original immutable `raw/framework-artifact-bindings.json` still has identical
`expected` and `actual` lists, each with 124 records. The mapping is exactly:

- Archive prefix: the empty string (archive root).
- Historical prefix: `artifacts/sparkle/Sparkle/`.
- Historical path = that prefix plus the exact archive member path.

This is one explicit mapping. No member was filtered, renamed, or tried under
another prefix. The comparison covers the complete set of non-directory members.
All **115 regular files** match their exact paths, sizes, and SHA-256 values.
All **9 symlinks** match their exact paths and target text. Missing records,
extra records, and differing records are all **zero**. A fresh read of the
materialized private tree produced the same complete inventory.

The regular files total **27,348,967 bytes**. The archive has 221 entries:
97 explicit directories and 124 file/symlink entries. Its total uncompressed
member data is **27,349,180 bytes**, including 213 bytes of symlink target text.
The copied data tree has 98 directories including its root. Its directory set
also equals the archive directory set plus that root.

Historical records bind file bytes and link targets. They do not supply a full
historical directory/mode inventory. This recovery makes no historical mode or
filesystem-metadata equality claim.

## Bounds and reader behavior

Each own worker that hashed or read archive data first called
`setpriority(PRIO_PROCESS, 0, 15)`. Actual `getpriority` was **15** before work
and at completion for both workers. Only the calling worker was changed. A later
small static receipt read also set and confirmed its own NI value at 15.

The network used **one metadata GET**, totaling 1,002 body bytes, against limits
of four fetches and 1 MiB. It used **one initial archive GET**, with one official
GitHub 302 redirect to `release-assets.githubusercontent.com`. Both final
responses were HTTP 200. The archive stayed below the 128 MiB limit. There was
no archive retry. TLS certificate and hostname verification remained enabled.
The reader disabled environment proxy use and supplied no authentication,
cookie, private key, SSH setting, or user configuration.

Retrieval, archive validation, materialization, copy verification, private
receipt creation, and file/directory fsync completed **95.496668 seconds** after
the original worker started. The same original 180-second wall deadline remained
in force across both workers, including the interval between them.

The first reader stopped before any archive request because it assumed inline
binary-target arguments. The pinned manifest uses named constants. The next
reader consumed the same saved metadata bytes with exact text checks. No source
was fetched again, no checksum or inventory predicate changed, and no HTTP,
checksum, inventory, or size failure was bypassed. The first reader result,
stop reason, and initial receipt remain private and are covered by the final
private receipt.

The own Python reader used isolated stdlib imports and ZIP data reads only.
It validated all 221 members before materialization. It enforced at most 2,000
entries, 512 MiB total uncompressed member data, and 128 MiB per member. It
rejected absolute/traversal/NUL paths, duplicate and case/Unicode alias names,
encrypted members, unsupported compression/types, devices, and members below
symlink ancestors. It validated every symlink chain, rejected escapes, cycles,
and dangling targets, and checked member content and CRC through complete reads.
No supplied code, helper, module, program, or library was executed or loaded.

## Durable private retention

Exactly one new private folder was created under the authorized state parent.
Its relative label is:
`sparkle-pinned-input-recovery-2026-10-24-bdfbf824`

Exact pointers relative to the authorized private state parent:

- Data: `sparkle-pinned-input-recovery-2026-10-24-bdfbf824/data`.
- Archive: `sparkle-pinned-input-recovery-2026-10-24-bdfbf824/Sparkle-for-Swift-Package-Manager.zip`.
- Copy inventory: `sparkle-pinned-input-recovery-2026-10-24-bdfbf824/qualified-copy-inventory.json`.
- HTTP bindings: `sparkle-pinned-input-recovery-2026-10-24-bdfbf824/qualified-http-bindings.json`.
- Final receipt: `sparkle-pinned-input-recovery-2026-10-24-bdfbf824/qualified-retention-receipt.json`.

The final receipt is 37,921 bytes, with SHA-256:
`7dce50e63988399ba6b062383286b6d74c7b8a501bb3defdb06b023ee1ce6c8a`.
A separate static read of the stored receipt confirmed that hash.

The folder has 130 regular files, totaling **37,867,596 bytes**, plus 9 safe
relative symlinks and 100 directories including the private root. Counts include
the archive, retained metadata and local binding copies, initial reader records,
copy inventories, and final receipt. The final receipt lists all other files and
links; it excludes itself and is bound by the public hash above.

All regular files have mode `0600`, with no execute bits. All directories have
mode `0700`. Original ZIP modes are metadata only. Symlinks retain their exact
relative targets and stay within the new data root; regular-file permission
claims do not apply to the symlink objects. Each file and directory was fsynced,
including the private root and its parent. Retention does not depend on temp
storage. No archive or binary is included in Git. Signed CDN redirect URLs and
raw HTTP headers remain private; the static record binds their exact journal.

## Preservation and proof limits

The prior retention report at commit `1ea63e55` remains the absence evidence.
No old private path was probed. The old artifact tree, raw workspace JSON,
compiled app, and icon were not recreated. Retained copies of the frozen parsed
resolution and inventory are explicitly source-binding copies, not reconstructed
historical raw state or receipts. Existing evidence, guards, and caches were not
changed. The final Git checks bind that preservation to the initial checkout.

Recovered byte equality proves dependency data identity only. It does not prove
ABI compatibility, architecture, signing, signature validity, framework loading,
current app acceptance, or a successful build. The original ordinary result
remains **FAILED / NOT ACCEPTED, 23/25**. Current whole-app acceptance remains
**UNVERIFIED**. The complete ordinary input set remains **INCOMPLETE**.

No build, app preparation, compiler, preflight, fixture, signing, codesign, lipo,
Swift package, supplied helper, product registration, runtime, unit test, mock,
simulation, process/OS/port query, installed app access, desktop automation,
credential/key/production-seed read, remote-host query, push, or publication ran.
The only public HTTP reads were the pinned official manifest and release asset.

No production source, installed app, live sharing, production config/TLS/default
IPC, TCC, login, service/job/account, firewall, global DNS, toolchain default,
TCP24800, UDP5353, Docker, or separate thread was changed or used. Isolated Swift
build, causal trace, crafted mDNS, original security, and IPC budget gates remain
stopped. The expired one-shot Sparkle operation remains closed, without retry,
renewal, reset, or approval reuse. This task is a new dependency data retrieval.

Only this new report/static-binding folder and the one new private folder were
written, apart from Git's requested local commit metadata. The commit disables
repository hooks, fsmonitor hooks, and commit signing. No runtime validation was
attempted because it is outside this task's authorization.
