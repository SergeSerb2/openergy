# Ordinary verification input retention

Date: 2026-10-24, America/Vancouver. Scope: static file research only.
Initial checkout commit: `d62129f4305197dd850edf03e51547c1bf0e855a`.

The exact signed ordinary bundle was **not found at the checked named locations**.
The original compiled daemon survives and matches its recorded input hash.
The compiled app, complete pinned Sparkle input tree, original icon, and raw
Swift workspace-state file are absent at the checked named locations.
The complete input set for later file-only reassembly is therefore **INCOMPLETE**.
No bundle was reassembled, signed, launched, or accepted.

This is a bounded absence result. It makes no claim about unsearched locations.
Private absolute paths, raw commands, environments, and logs remain private.
[results.json](results.json) gives every checked location a stable label and a
SHA-256 binding to its exact path. The private journal keeps the corresponding
path and its provenance reference.

## Retained files and missing files

| Input | Current result | Binding and limit |
| --- | --- | --- |
| Original signed ordinary bundle | ABSENT | The prepared root and bundle path are absent. The exact icon staging bundle is also absent. No candidate was available for the required complete 167-entry signed inventory comparison. |
| Original compiled app | ABSENT | The exact app path in the saved architecture command is absent. Expected input SHA-256: `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`. |
| Original compiled daemon | PRESENT, HASH MATCH | 4,493,064 bytes. SHA-256: `23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb`. This is the compiled input, distinct from the historical signed daemon. |
| Pinned Sparkle input tree | ABSENT | Both named cache artifact roots and the additional exact artifact path in saved workspace state are absent. The immutable expected and actual lists agree on 124 historical file/symlink records. No current artifact tree was qualified. |
| Raw Swift workspace state | ABSENT | Both named cache state paths are absent. Expected ordinary-state raw SHA-256: `f7dbaca242e19cbc322f7a9ca321779f4a8e456202c57b716d7868ad8807376d`. The archived parsed state is retained, but it is not the original raw state file. |
| Original icon | ABSENT | The exact icon source is absent. Expected SHA-256: `966ae2468e6c90ea0bacd0f88eca6bc962cf72e49f1fe95a11543809b52b35c2`. |
| Six inherited build receipts | PRESENT, HASH MATCH | All six archived copies and all six exact original source files match the immutable `raw/inherited-build-records.json` hashes. Raw data was retained privately. |
| Source-binding data | PRESENT, CONTENT MATCH | The retained app build-source record has 29 entries. The daemon source-binding record has 397 entries. Their bound file hashes agree with the immutable ordinary input records. Raw-file and normalized-JSON proof are kept distinct below. |
| Current source files at two exact roots | MATCH ordinary bindings | Each root matches all 29 ordinary app and all 397 daemon file bindings. The one historical compiled-source difference in `apps/macos/scripts/bundle.sh` remains recorded. |
| Packaging scripts and license inputs | HASH MATCH | The exact Sparkle packaging script, priority helper, and three license/notice inputs match the recorded bindings at both checked roots. They were read only. |

The historical signed executable hashes remain:

- App: `53ba6f431811eb1178c39b8b7ad7cdb425b0088e7c893b8dcab8e601ad338c23`.
- Daemon: `1c558733a56ada70661f8a8493eae7deaf3813ce1bb572d0592fe86900b6b971`.

These are saved bindings, not hashes of a surviving signed bundle.
The retained architecture receipts record `arm64` for both original inputs.
No architecture tool, binary, or supplied helper was run in this task.

The saved Sparkle resolution pins version `2.10.0`, revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`, and download checksum
`17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
These records do not establish that the artifact bytes still exist.

## Source record proof

The app build-source raw file SHA-256 is
`37c6416b1b8548e93078eafe2c0d18f87608a026294eeb9143d36718e6181282`.
Its compact, sorted JSON SHA-256 is
`69453dd326f6b0309dabb0bf9e8d9cab882afca92912e0e4f48543fc8765595e`.
The retained source-audit record uses the second convention. A static read of
the saved audit source confirms that convention. An initial attempted raw-file
comparison did not match; the differing hashes remain explicit here.
No raw bytes were changed to make that comparison pass.

The daemon source-binding raw file SHA-256 is
`71b2fdddc010351103bcdff785beacbef4d6922b9592bb38d1087fb9125d6022`.
No separate historical raw-file hash for this record was established by this
audit. Its 397 paths and expected file hashes agree with the immutable ordinary
bindings. The published raw hash binds the file read and copied today.

All 29 ordinary app bindings match current bytes at both exact checked roots.
Compared with the older compiled-app source record, 28 of 29 match.
`apps/macos/scripts/bundle.sh` is the same pre-existing difference recorded by
the original preparation. It is not a new mismatch or a fresh compilation claim.
[supplemental-bindings.json](supplemental-bindings.json) records the additional
script, license, normalized-JSON, and saved architecture checks.

## Private retention

One new durable private folder was created under the user-authorized state
parent. Its directories use mode `0700`. All retained files use mode `0600`.
The daemon is stored as data with no executable permission. Its original mode
is recorded as metadata. No original root, marker, nonce, or runtime home was
recreated. No symlink was copied.

The private copy receipt covers 21 retained input/data files, 4,850,035 bytes:
the compiled daemon, six inherited raw receipts, two source-binding records,
and twelve immutable binding/plist/architecture records. Each copy has before
and copied hashes, byte count, stored mode, and original mode. The private
journal includes the source path plan and tree inventory before its final
journal/receipt files were written. The final static check also inventories
the completed private folder.

Private copy receipt SHA-256:
`a5867924194b31b03013ec1312d97573f6e9825330d034818a46229fa2f897ef`.

The receipt and journal are private. Public files expose only redacted bindings.
No private absolute path, endpoint, or raw shared log is included in this report.

## Search bounds and preservation

The finite source plan has 21 top-level locations, below the limit of 64.
It covers the original archive, frozen R7 package, full R8 review, two exact
source roots, original receipt directories, and the exact bundle, compiled
input, cache, state, artifact, and icon locations derived from those records.
Source roots were read through explicit bound file lists, without recursive
worktree search. There was no home-wide or temp-wide scan.

Each artifact tree had a hard limit of 2,000 entries, which is stricter than
2,000 files. Missing artifact roots were not walked. No unrelated system tool,
SDK, credential store, runtime home, production configuration, or Swift security
directory named inside historical command data was accessed.

The main reader charged 29,411,081 content bytes. It reserved an additional
8,388,608 bytes for preliminary and exploratory static reads, for a conservative
main total of 37,799,689 bytes. Supplemental binding checks charged another
384,694 bytes. One own static verifier attempt stopped because its path scan
matched its own literal. The reader was corrected to use general path patterns;
no evidence predicate changed. A further 67,108,864 bytes (64 MiB) conservatively
covers that bounded failed attempt. [validation.json](validation.json) records
the final verification charge and the full conservative audit bound, below
536,870,912 bytes (512 MiB).
Counts describe logical file/Git content read, rather than filesystem block I/O.
Policy/memory checks and task-owned output roots are separate from the 21 source
locations; even counting those exact accesses keeps the top-level total below 64.

The reader compared all 87 original files with immutable commit
`498190feb00f51039f500ae618ac7de5fb113d86`, before and after retention.
It also compared all 16 frozen R7 files with
`d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6` and the full R8 review with
`172ee851d99bf1c256235b23fb445cae899332ff`. All comparisons matched. The final check repeats the
byte/hash checks. [protected-original-index.json](protected-original-index.json)
retains the original file hashes and Git blob IDs without raw contents.

Only this new documentation folder and the one new private folder were written.
No source, previous evidence, frozen candidate, guard, or separate R9 handoff
was edited. The commit disables repository hooks and commit signing.

## Remaining prerequisites and proof limits

A later file-only reassembly decision still needs the exact compiled app bytes,
the complete pinned Sparkle file/symlink tree, original icon, and the required
raw workspace state/receipt inputs. An exact recovered signed bundle would need
the complete original file, directory, mode, hash, and symlink inventory match
before any qualified copy. This task authorizes no reassembly or signing step.

The R9 source handoff repair and review remain separate. Byte equality to a
historical source binding does not prove current whole-app acceptance or source
approval. The original ordinary result stays **FAILED / NOT ACCEPTED, 23/25**.
Current whole-app/runtime acceptance remains **UNVERIFIED**.

No build, compiler, preflight, supplied module import/evaluation, product/helper
execution, fixture preparation, signing, process/OS/port/job query, desktop
driving, network query, test, mock, simulation, push, or publication occurred.
Only own static readers and file/Git/hash/JSON/AST checks ran. No runtime or end
to end check ran because this task expressly forbids them.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login,
services/jobs/accounts, firewall, global DNS, and toolchain defaults were not
changed. No TCP24800, UDP5353, Docker, credential, private-key, or production-seed
access occurred. Crafted mDNS, original security, IPC import-budget, and isolated
Swift build gates remain stopped. The expired Sparkle one-shot remains closed.
