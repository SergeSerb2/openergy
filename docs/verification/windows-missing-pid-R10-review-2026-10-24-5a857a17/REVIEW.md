**Windows missing-PID repair R9: independent R10 source review**

Review date: 2026-10-24, America/Vancouver.

**Verdict: no actionable P1/P2 findings in the exact source and saved-data scope.**
R8-P2-1 is addressed in source. This conclusion follows from fresh source reads,
independent binding checks, and primary documentation/source. It adopts no prior
review verdict. Native binding, PowerShell/C# syntax, build, handle settlement,
normal stop, and runtime acceptance remain **UNVERIFIED**. This report supplies
no executable receipt, integration, native acceptance, or release approval.

The exact reviewed commit and this review's clean starting parent are
`762240ac041a835707b7ae3867b4e15b1e06b8c0`. The package is
`docs/verification/windows-missing-pid-repair-r9-2026-10-24/`.
All source locations below refer to that package at that commit. This review
used only the isolated checkout
`/Users/serge/.t3/worktrees/openergy/t3-5a857a17`, branch
`t3/windows-r9-repair-review`. No other checkout was changed.

All seven complete inert source texts were read: build, common, controller,
observer, qualification, runtime, and postflight. The complete R9 README,
input bindings, manifest, static checks, first-reader error, preservation JSON,
proposal, patch, and attributes were read as data. Every preservation entry was
independently checked. The R7 README, design, responses, next-run plan, proposal,
prerequisites, preparation errors, static records, and bound input/preservation
records were also covered. Earlier source behavior was traced through exact
byte comparisons and source differences, rather than inferred from verdicts.

The full R8 review was read at `fe22c54d890ab7ecbfd58845040aba3ce87d47bc`,
`docs/verification/windows-adapter-p2-review-2026-10-24-81ddd0c1/REVIEW.md`.
Its 17,474 bytes match SHA-256
`0d591ef0c84aa23b70be8f2f776912fddf6981fb24751f253a40323c0810012d`.
The exact R7 patch base is `1d01d672c93b03d089d6d902f5bc7f15c10ffd05`,
`docs/verification/windows-adapter-p2-repair-2026-10-24-01d1bcb5/`.
All 21 R7 files match that commit and current checkout bytes.

R8's trigger was a CIM-sampled compiler child that exited before the PID lookup,
while its retained parent remained live. The former cmdlet converted the .NET
`ArgumentException` into `NoProcessFoundForGivenId` with a
`ProcessCommandException` and no inner exception. That bypassed the local
`ArgumentException` catch, failed the stage, and entered owned Cargo recovery.
The conversion and exception inheritance were checked directly in the official
[PowerShell v7.5.4 Process.cs source](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L217-L245)
and its
[error construction](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L334-L352).

R9 changes only `source/build.ps1.txt:74` to the direct local call
`[Diagnostics.Process]::GetProcessById([int]$p.ProcessId)`.
The local overload documents `ArgumentException` when that PID is no longer
running. [Microsoft API contract](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.getprocessbyid?view=net-9.0).
PowerShell's documented typed catches select the specified exception class and
derived classes. [Microsoft catch documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_try_catch_finally?view=powershell-7.5#syntax).
The official engine source also checks the runtime exception's inner exception
when choosing a catch handler. Method-invocation wrapping retains the underlying
exception. [Handler selection](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/System.Management.Automation/engine/runtime/Operations/MiscOps.cs#L1531-L1667),
[method-invocation wrapping](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/System.Management.Automation/engine/runtime/Operations/MiscOps.cs#L2146-L2171).
Thus the existing typed catch at `:74-76` matches the documented missing-PID
exception from this direct call. This is a source-based conclusion, not a
PowerShell execution result.

The unchanged catch records the observation race and continues before
`Build-Owner` is called. A failed lookup grants no ownership. A returned object
still enters `Build-Owner` at `:77`, retains its safe handle at `:79`, and passes
the generation, executable, command-line, parent-PID, live retained-parent,
priority, and session checks at `:80-85`. Other errors retain the existing
failure/settlement path. The repair adds no broad catch, ownership shortcut,
lookup retry, build retry, or timer reset.

The full R6 review was read at `7ce44f6a02890b89cffca407a78497e0ba9bbddb`,
`docs/verification/windows-prerequisite-recovery-r5-review-2026-10-24-76aa12eb/REVIEW.md`.
Its 12,080 bytes match SHA-256
`e19ebf2089afffda0d874e84c81962f865bf9d9ceffa309d34162910f0fd71b5`.
Both R6 repair dispositions remain supported by the current source:

- TEMP containment remains at `source/controller.py.txt:70-77,177-188`.
  Before transport it rejects root, drive, absolute, parent-component, and empty
  paths. Mapping requires an absolute normalized TEMP parent, strict normalized
  containment, and nine distinct normalized roots. Exact ordered historical
  labels and the private selection hash remain required. The source adds no
  guessed path or empty selection. These are lexical gates; no target path or
  filesystem case was evaluated here.
- Compiler recovery remains at `source/build.ps1.txt:57-61,93-225`.
  The root object is registered before creation. Its creation-associated handle
  is the first retained handle after successful start, before stdin, stream,
  identity, or receipt work. The outer catch preserves the first error once;
  later errors are separate. Settlement visits the root and retained verified
  descendant generations, signals live owned processes before blocking waits,
  records actual signal/exit observations, and independently attempts process
  and safe-handle disposal. It performs no new CIM lookup or PID-based discovery
  for native build recovery. Provisional descendants have no kill authority.
  SSH exit or an ambient job is not used as proof of compiler settlement.

The one work deadline is 600 seconds. The shared settlement deadline is the
earlier of settlement entry plus 30 seconds and build start plus 630 seconds.
Waits use the remaining shared reserve. Final receipts retain the 630-second
ceiling; failed or late writes keep the stage nonzero. The controller requires
the full 660-second outer build allowance before the final 30-second transport
reserve of the single 2,400-second workflow (`controller.py.txt:137-144`).
It attempts one postflight/collection path for a known root in `:255-275` and
preserves an earlier flow exception if collection also fails. Build failure
skips runtime. These source checks do not prove preemption of arbitrary OS calls
or completion of collection after transport/storage failure.

The bound R2 package is `7d92633149204f834b4da55e352e7d05fd2abe4a`,
`docs/verification/release-integration-2026-10-23/windows-stop-controller-repair-r2-dbcc0160/`.
The full R4 review was read at `6b0e4f12469a28f89badbd6bb3dc80ee0de45f40`,
`docs/verification/release-integration-2026-10-23/windows-stop-controller-R4-review-b5580f67.md`.
Its 10,311 bytes match SHA-256
`71a1a7c51b4bc867ecc15c090047882b629a9ff77406a5fd8db3246a3c818366`.
R4's source-only disposition supplies no native proof for R9.

Common and observer remain exact R2 bytes. Runtime's block from `$binary=` to
`$result=` and its complete body from `function Pipe-Exists` through final exit
remain exact R2 bytes. The controller's `command`, `remote`, and
`recover_transport_owner` methods match R2 and R5. Its `bind_source`, `collect`,
and `export` methods match exact R5 `6b7a029ffbd8e41b411408bbcaf9b219e1f6592c`.
All five native source files other than build remain exact R5 bytes.

The R2 log reader still uses read access with `FileShare.ReadWrite|Delete`, one
pinned stream, 4 KiB polls, and a 2 MiB length/offset cap. Only
`FileNotFoundException` remains pending. Readiness keeps one 15-second deadline
from before daemon resume and rejects late markers. The real CLI/wire flow
still requires actual status/config/invalid-method results, exact methods and
matching non-null serialized IDs, real CLI/server PIDs, exact stop ACK, the full
ten-second normal-stop allowance, independent retained-handle signal, original
controller lifetime, and daemon exit 0. Observer settlement precedes recovery's
final forwarding tick. Forced recovery cannot establish normal acceptance.
Ownership, cleanup, selected before/after preservation, and verified collection
before exact fresh-root removal remain required.

Independent data checks established the following:

- R9 has exactly 16 regular package files and 15 unique manifest entries,
  covering every file except the manifest itself. All sizes, SHA-256 values,
  exact Git bytes, and checkout bytes match. All five R9 JSON files decode.
- All 22 R9 input bindings match their exact commit, Git blob, size, SHA-256,
  and checkout bytes. They cover all 21 R7 files and the full R8 review.
  All 216 R7 old-input records, 20 R5 input records, and 25 R2 input records
  also match. The additional R2 writer-source binding matches exact `e1ef124`.
- R7 has 21 files/20 checksums; R5 has 21 files/20 checksums; R2 has 14 files/13
  checksums. All package bytes and checksum sets match. All 7 R7, 9 R5, and 3 R2
  JSON files decode.
- All 178 published relative inventory members match their recorded sizes and
  hashes, immutable `6010722042604f77842db3ce491bee42f52a1265` bytes, and current
  checkout bytes. Separately, every file in the entire 179-file historical
  folder matches Git and the R9 preservation record. The inventory itself is
  the additional folder file. No 178-versus-179 count predicate was weakened.
- All 558 source-manifest members match exact
  `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a` Git blobs, sizes, hashes, CRLF/LF
  counts, and current source bytes. The published 188,309-byte manifest hash is
  `a208cbb73d29b4939fcd4a43eae67769d1763823fc6a77979bb7abe96ef243dc`.
  Its historical archive binding remains 4,065,280 bytes and
  `775e04301b1f3816ccf39fd088e1100da4344be5b171df7330b0049a7f589adf`.
  No archive was recreated or private recovery retention inspected.
- An independent unified-patch data reader applied the single hunk to immutable
  R7 bytes only in memory. One deletion and one addition produce the exact R9
  build text. An independently regenerated diff also matches the saved patch.
  The six other complete modules are byte-identical to R7. Python controller
  AST parsing was data-only, with no import, compilation, evaluation, or execution
  of supplied code or AST.
- Proposal differences are confined to schema/review/base metadata, the repair
  description, and source bindings. Budgets, one-shot counts, pinning, flow,
  failure, collection, and missing-fact fields are unchanged. There remains one
  pinned locked/offline release command, jobs 2, zero Clippy commands, and no
  retry. No guard or acceptance predicate was changed.

The preserved `static-reader-first-error.txt` records the author's initial
wrong whole-folder count assertion and its correction to two separate checks.
It also records removal of an extra README EOF blank line before the final
manifest. This review verified the final bytes, both evidence sets, and the
current single terminal README newline. It did not reproduce intermediate
authoring states or erase their error record.

The R9 build text is 18,281 bytes, SHA-256
`bc42365d060d9cda6d86642f7282970bebf3892c9681bd71e937376b76bb18b8`.
The 2,467-byte R9 manifest has SHA-256
`031f15d74056fcc1b4b750603dc4ae81d6aaa0e2d8ae3cf8b3444b9e1b784c80`.
The other source SHA-256 values are:

- common: `c09ce6f29820bd3805fec5398f49de9fe797d71e57a0e3e4f233f56f51043ce0`
- controller: `10431430e9a218418addc4cc47d31521b42aabaed6dced439e25ba4a89da334e`
- observer: `8559ecbca18de3b896407df1048c278633c927134858cc99124ae883e816f5e2`
- postflight: `77456c370740da9ef7e86a4bbb9046342a239c7b9d9b72b9da34c43eb9bdbd2a`
- qualification: `3d9f926395adbdeee29518a9fdaa164ddb7545db67747f3f2ec9023ac2729d22`
- runtime: `986b6246e79f8ecb9ddefa32bcfa23ca0aec6c31af44b97915c45bc481f9e615`

The current brief reports private recovery of the exact existing transport
without host contact. No selector, private path, key, credential, or configuration
was inspected here. The published fixture-recovery README/result at
`d3cd115dd64ccd5fbf94a9ed9723d446ac198c36` were read as saved data: the bounded
160 retrievals across six threads are exhausted, with 0/9 qualified bindings
and no valid `baseline-selection.json`. Those exact historical path facts and
their private hash binding remain missing. This review makes no new host query,
recovery attempt, guessed selection, or human-permission gate. A new Windows
flow cannot run with these missing inputs. The separate ordinary-source R9
review does not supply its missing app inputs or an isolated Mac.

The original `6010722` native Clippy/build PASS remains limited to exact
`e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a` and its historical artifact. Off stop
remains **FAILED / UNVERIFIED**: the old active-log reader failed before healthy
status/config, invalid method, or stop. There is no historical stop ACK, normal
stop measurement, normal exit acceptance, or saved forced-exit code. The old
executable/raw receipt/state/runtime archive were not recovered or substituted
by this review. Unknown or unsampled compiler generations, release-linker
sampling, external SSH/job/handle state, arbitrary OS-call preemption, and
uncapped output/memory/disk use remain explicit proof limits. None is promoted
to a new automatic P2 or a universal guarantee.

Only this new review file is owned. All prior source, evidence, guards, and
production files remained read-only. Checks used fresh file/Git/hash/JSON/AST/
patch data readers and essential primary public documentation/source reads.
No supplied module/helper/checker/AST was imported, evaluated, compiled, or
executed. No PowerShell interpreter/parser, C# parser/compiler, test, mock,
simulation, fake-PID lookup, build, preflight, product, fixture preparation,
process/OS/port/job query, native/remote/SSH connection, desktop action,
credential/private-key/production-seed access, signing, delegation, integration,
push, or publication ran. Git hooks and commit signing are disabled for the
single report delivery commit.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login,
services, jobs, accounts, firewall, global DNS, and toolchain defaults were not
changed. No TCP 24800, UDP 5353, or Docker action occurred. Stopped experiments,
isolated Swift work, and causal-trace gates remain stopped. The expired closed
Sparkle one-shot was not retried, renewed, reset, or reused as approval.
