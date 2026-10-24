**Windows adapter P2 repair: independent whole-candidate source review**

Review date: 2026-10-24, America/Vancouver.

**Verdict: one actionable P2 finding; no P1 finding.** Both stated R6 defects
are repaired in source. The new compiler observation path has a separate P2
regression below. This verdict is independent of R4 and R6. Runtime remains
**UNVERIFIED**. This document provides no native acceptance or executable
approval receipt.

The exact candidate is `1d01d672c93b03d089d6d902f5bc7f15c10ffd05`, folder
`docs/verification/windows-adapter-p2-repair-2026-10-24-01d1bcb5/`.
All candidate source locations below refer to that folder at that commit.
Its 21 files match the imported copy at review parent
`a2a83194efbab6559aacb30def21b8b9890027c3` and the current checkout bytes.
This review started clean at that parent in isolated checkout
`/Users/serge/.t3/worktrees/openergy/t3-81ddd0c1`.

**P2-1: catch the cmdlet's missing-PID error as an observation race.**

Source: `source/build.ps1.txt:74-76`. The snapshot and parent checks are at
`:65-73`. The caller and failure recovery are at `:121-123,145-163`.

Trigger: the CIM snapshot includes a compiler child whose retained parent is
still alive. That child exits before `Get-Process -Id ... -ErrorAction Stop`
looks up its PID. This is an ordinary observation race during compilation.

The local catch only accepts `ArgumentException`. PowerShell's cmdlet catches
the underlying `Process.GetProcessById` `ArgumentException` itself. It emits a
`NoProcessFoundForGivenId` error with a `ProcessCommandException` and null inner
exception. `ProcessCommandException` derives from `SystemException`, not
`ArgumentException`. With `-ErrorAction Stop`, the error bypasses this local
catch. This follows from the official
[PowerShell 7.4.6 implementation](https://github.com/PowerShell/PowerShell/blob/v7.4.6/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L201-L327)
and its
[exception declaration](https://github.com/PowerShell/PowerShell/blob/v7.4.6/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L2788-L2814).
The same conversion is present in the
[PowerShell 7.5.4 implementation](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L203-L329).
Typed catches select the specified exception type and derived types, as described
in Microsoft's
[catch documentation](https://github.com/MicrosoftDocs/PowerShell-Docs/blob/main/reference/7.4/Microsoft.PowerShell.Core/About/about_Try_Catch_Finally.md#syntax).

Effect: a completed compiler child can fail an otherwise valid release build.
The outer catch saves this lookup error as the first build failure. Settlement
then kills the still-running owned Cargo root and any retained verified children.
The build stage returns nonzero, skips runtime, and consumes the one permitted
build attempt. The intended race record and `continue` at lines 75-76 never run.
This is a source-level failure trigger. No such race was executed or observed
in this review. It is separate from the explicit limit on unsampled generations.

Fix: classify only the cmdlet's exact `NoProcessFoundForGivenId` error as a
missing-process race, record it, and continue without inferring ownership.
Rethrow other lookup errors. Alternatively, use the direct .NET PID lookup with
its matching exception type and keep the subsequent retained-handle and identity
checks. Preserve the existing deadlines, one-shot guard, priority/session checks,
and refusal to kill an unverified generation. Do not suppress all process errors
or retry the build.

**Disposition of the two R6 findings**

R6 was read in full at `7ce44f6a02890b89cffca407a78497e0ba9bbddb`, file
`docs/verification/windows-prerequisite-recovery-r5-review-2026-10-24-76aa12eb/REVIEW.md`.
Its 12,080 bytes have SHA-256
`e19ebf2089afffda0d874e84c81962f865bf9d9ceffa309d34162910f0fd71b5`.
The exact R5 base is `6b7a029ffbd8e41b411408bbcaf9b219e1f6592c`, folder
`docs/verification/release-integration-2026-10-23/windows-prerequisite-recovery-r5-86e0bfd1/`.

R6 P2-1, rooted fixture selection: **addressed in source**.
`controller.py.txt:70-77` keeps the private selection SHA-256 and exact ordered
nine labels. It rejects root, drive, absolute, parent-component, and empty paths
before any transport. `:177-188` requires an absolute normalized TEMP parent,
normalizes each join, rejects equality with that parent, checks case-normalized
common-path containment, and requires nine distinct normalized mapped roots.
Only then does it save/upload state. The unchanged native `Shared`/`Snapshot`
reader receives these mapped roots on this flow. The original ACL/reparse checks
remain. No crafted path was evaluated, simulated, or passed to a filesystem.
These are lexical admission checks, not target-specific filesystem acceptance.

R6 P2-2, terminating build errors bypassing owned recovery: **addressed in source
for the reported lifecycle escape**. `build.ps1.txt:57-61,104-114` registers the
process object before `Start` and retains its creation-associated safe handle
before stdin, output, identity, or receipt work. Root identity failure still
leaves the created root eligible for recovery. Prelaunch failures cannot skip
cleanup of a previously launched step because this candidate has only one step.

The lifecycle at `:93-225` catches work errors and enters settlement on normal
and failed paths. Root and verified descendant handles remain retained through
artifact and source checks. Ownership records survive disposal and final receipt
work. The first error uses
explicit script scope and a write-once null check. Later errors use a separate
list. Helper list additions and void writes do not add success-stream objects
to owner or identity results. The normal stdout/stderr wait short circuit does
not skip the final settlement loop, which visits both non-null stream tasks.

Descendant admission at `:63-91` retains a handle, checks the snapshot generation,
managed identity, executable, command line, parent PID, and still-live retained
parent. Known PIDs are not rebound to new generations. A provisional entry is
disposed but cannot be killed before verification. The new finding concerns
lookup error classification before that admission, not unsafe PID ownership.

Settlement at `:147-206` visits the root first and signals verified live handles
before blocking waits. It uses no new CIM lookup, tree kill, SSH assumption, or
ambient job assumption. Actual signals and exit codes come from retained objects.
Each wait uses the same remaining reserve. The nested finally independently
attempts process and safe-handle disposal for every registered entry. Missing
exit/closure evidence, forced recovery, stream faults, and exceeded budgets fail
the stage. No blanket proof for all children or unknown generations is asserted.

Receipt attempts at `:207-224` follow handle settlement. Each fallible receipt
write records an error. The final write also has a deadline check afterward.
If that write fails or completes late, the stage remains nonzero. A partial or
stale on-disk receipt cannot admit runtime because the controller also requires
the build-stage exit to be zero. An unwritable sink can prevent durable evidence.
That is an explicit failure limit, not a receipt success claim.

The native work deadline is 600 seconds. Settlement uses the earlier of its
entry plus 30 seconds and build start plus 630 seconds. Handle waits cap at
10 seconds and stream waits at 5 seconds within that shared reserve. Final
receipts also use the 630-second ceiling. `controller.py.txt:139-140` admits the
build only when its full 660-second outer allowance fits before the final
30-second reserve of the single 2,400-second workflow. These are checked bounds,
not proof that arbitrary metadata or filesystem calls are preemptible.

`controller.py.txt:255-275` attempts one postflight/collection path in finally
whenever setup returned a known root. It preserves an earlier flow exception
when collection also fails. Build failure skips runtime. Incomplete collection
retains the root through the unchanged binding and removal gates. This does not
prove postflight completion when transport, setup metadata, or overall time fails.

**Whole-candidate scope and preserved R2 contracts**

All seven complete inert candidate source texts were read. The complete README,
DESIGN, RESPONSES, NEXT-RUN, repair patch, proposal, prerequisites, old-input,
preparation-error, static-validation, static-materialization, preservation, checksum,
and attributes records were read as data. Every entry in the large old-input and
preservation records was independently checked against its bound bytes.

The complete required R2 package at
`7d92633149204f834b4da55e352e7d05fd2abe4a`, folder
`docs/verification/release-integration-2026-10-23/windows-stop-controller-repair-r2-dbcc0160/`,
and complete R4 review at `6b0e4f12469a28f89badbd6bb3dc80ee0de45f40` were covered.
R4's file is
`docs/verification/release-integration-2026-10-23/windows-stop-controller-R4-review-b5580f67.md`,
10,311 bytes, SHA-256
`71a1a7c51b4bc867ecc15c090047882b629a9ff77406a5fd8db3246a3c818366`.
All R2 and R5 package files were read as data. Their source was traced through
the full candidate texts, earlier controller/build texts, exact byte comparisons,
and both complete earlier patches. R4's verdict was not adopted.

Only build and controller differ from exact R5. The other five candidate sources
are byte-identical to R5. Common and observer are also exact R2 bytes. Runtime's
block from `$binary=` to `$result=` and its entire body from `function Pipe-Exists`
through final exit match R2. Controller methods `command`, `remote`, and
`recover_transport_owner` match R2 and R5. `bind_source`, `collect`, and `export`
match R5.

The read-only log stream retains `FileShare.ReadWrite|FileShare.Delete`, its
pinned handle, 4,096-byte poll limit, and 2 MiB length/offset cap. Only
`FileNotFoundException` remains pending. The private config directory precedes
launch. One 15-second readiness deadline starts before daemon resume and rejects
late markers. Real status/config checks remain separate from the marker.

The observer forwards actual CLI frames unchanged. Acceptance keeps the exact
method, serialized matching non-null ID, real CLI/server PIDs, method-specific
status/config/error checks, exact stop ACK, full ten-second stop allowance,
independent retained-handle signal, original controller lifetime, and normal
exit 0 without forced recovery. Observer cancellation settles before recovery takes the final
forwarding tick. Unsettled observers and changed ownership refuse unsafe kills.
Unresumed children retain their native-handle recovery path. Forced recovery
cannot establish normal-stop acceptance. The artifact's CLI/wire/logger source
at `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a` was also inspected as text.

There is one pinned release build command, zero Clippy commands, and no retry.
Locked/offline resolution, jobs 2, direct Rust/Cargo 1.99.1, MSVC 14.44.35207,
SDK 10.0.26100.0, and private Cargo/target/TEMP paths remain. New artifact
acceptance uses its actual fresh size/hash and AMD64 PE32+ check. It does not use
the old binary hash as a new-build target. New selected before/after baselines,
transfer hashes, source checks, private collection, and removal gates remain.

**Independent data checks and exact bindings**

All 21 candidate files and all 20 checksum entries match. All 21 R5 files and
20 checksum entries match. All 14 R2 files and 13 checksum entries match.
All 216 candidate old-input records, all 20 R5 inputs, and all 25 R2 inputs
match their exact Git blob, size, SHA-256, and current checkout bytes.
JSON decoding passed for all 7 candidate, 9 R5, and 3 R2 JSON files.
Python controller AST parsing was data-only, with no AST evaluation or import.

An independent unified-patch reader applied each patch to immutable Git bytes
only in memory. Every result matches the exact target. Hunk counts were R2:
controller 10, observer 4, postflight 2, qualify 2, runtime 8. R5: controller 7,
qualify 2, runtime 2, postflight 2, build 3. New candidate: build 2, controller 5.
No executable source was materialized.

Every one of the original 178 published members matches its recorded size,
SHA-256, exact `6010722042604f77842db3ce491bee42f52a1265` Git bytes, and current
checkout bytes. All 558 source-manifest members match exact `e1ef124` Git blobs,
sizes, SHA-256 values, CRLF counts, LF counts, and current source bytes.
The published manifest is 188,309 bytes, SHA-256
`a208cbb73d29b4939fcd4a43eae67769d1763823fc6a77979bb7abe96ef243dc`.
The bound historical archive is 4,065,280 bytes, SHA-256
`775e04301b1f3816ccf39fd088e1100da4344be5b171df7330b0049a7f589adf`.
This review did not recreate an archive or inspect private recovery retention.

Exact candidate source SHA-256 values:

- `source/build.ps1.txt`, 18,271 bytes: `f8559887d542227a204e79a554377736fdd61dc5d1a00386ae48214fbbf69df2`
- `source/common.ps1.txt`, 6,364 bytes: `c09ce6f29820bd3805fec5398f49de9fe797d71e57a0e3e4f233f56f51043ce0`
- `source/controller.py.txt`, 21,578 bytes: `10431430e9a218418addc4cc47d31521b42aabaed6dced439e25ba4a89da334e`
- `source/observer.cs.txt`, 12,054 bytes: `8559ecbca18de3b896407df1048c278633c927134858cc99124ae883e816f5e2`
- `source/postflight.ps1.txt`, 4,749 bytes: `77456c370740da9ef7e86a4bbb9046342a239c7b9d9b72b9da34c43eb9bdbd2a`
- `source/qualify.ps1.txt`, 4,272 bytes: `3d9f926395adbdeee29518a9fdaa164ddb7545db67747f3f2ec9023ac2729d22`
- `source/runtime.ps1.txt`, 21,487 bytes: `986b6246e79f8ecb9ddefa32bcfa23ca0aec6c31af44b97915c45bc481f9e615`
- `repair-from-r5.patch`, 28,026 bytes: `8bc9037ec0b24ccdcd485e2922c18fc05f013959dc5021ec14d18659f7291644`
- `SHA256SUMS`, 1,719 bytes: `fba9cdb6f9e9aba32565e6e4a13d78a690437b1e3a98a4ffa456ef4c5a6dae58`
- `old-input-hashes.json`, 89,643 bytes: `7332e1ad6933d9d0373adba1a028e4eb004607e90f185dc6daf4ba66fc782db4`
- `source-preservation.json`, 235,825 bytes: `48208a68eba5ea559cb8f24b32da77c10b38083c75fc5d36da7e6d926b9e5d9f`

**Historical status, prerequisites, and proof limits**

The original `6010722` receipts preserve native strict Clippy and release-build
PASS only for exact source `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a` and its
historical artifact. Version exit 0 and absent-daemon status exit 1 passed.
The Off flow failed at active-log sharing before healthy status/config, invalid
method, or stop. It has zero stop requests, no established product stop bug,
no stop ACK, no normal-stop timing or exit acceptance, and no saved forced exit
code. The release linker was not sampled. Its priority stays UNVERIFIED.
That flow remains **FAILED / UNVERIFIED**.

The user reports private recovery of the exact existing transport through one
historical signature match among 54 known public variants, without host contact
or key reads. That planning fact supersedes R5's older unresolved-selector record.
This review did not read, locate, hash, or verify the selector. The nine exact
historical fixture path facts remain missing. Their private hash and exact-label
requirements are unchanged. No guess or reduced fixture requirement is offered.
These are missing facts, not a new human permission gate.

The old executable, raw build receipt, selected state, and runtime evidence
archive remain unrecovered per the bound records and current brief. Private
retention was not queried. The published derived build-result hash
`65c8aa80c8e179201039111724bb5735ee62696ab2e5ed9414e20323e8b44cbe`
does not equal the missing raw receipt hash
`f52b45d2cbea71c19cc1b6d43eee6b6ce99b2072b852a5cd29f688f677213eca`.
No historical input was synthesized or replaced.

PowerShell and C# syntax received text inspection only. Native syntax, build,
ownership settlement, actual handle closure, normal stop, and all runtime
acceptance remain **UNVERIFIED**. Unknown or unsampled generations, external
SSH/job/handle state, uncapped output/memory/disk use, total resource bounds,
kernel identity perfection, and preemption of arbitrary OS calls remain limits.
No frequency or native reproduction of the new P2 trigger is claimed.

Only this new review document was authored. Prior sources, evidence, guards,
and production files remained read-only. Checks used independent file/Git/hash/
JSON/AST/patch data readers and essential public primary SDK/documentation text.
No supplied module, helper, checker, or AST was imported, executed, or evaluated.
No PowerShell/C# parser or compilation, test, mock, simulation, crafted-path or
process-interleaving check, build, preflight, fixture preparation, product,
process/OS/port/job query, host contact, desktop driving, credential/private-key/
production-seed read, signing, delegation, push, or publication ran.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login,
services, jobs, accounts, firewall, global DNS, and toolchain defaults were not
changed. No TCP 24800, UDP 5353, or Docker action occurred. Crafted mDNS,
original provenance/security, IPC/Synergy 3 import budget, isolated Swift build,
and causal bind-trace gates remain stopped. The expired closed Sparkle one-shot
was not retried, renewed, reset, or used as approval authority. The delivery
commit contains only this new report, with hooks and commit signing disabled
for that Git command. It supplies no native or release approval.
