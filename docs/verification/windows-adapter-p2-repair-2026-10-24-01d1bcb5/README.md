Source-only implementation handoff, 2026-10-24, America/Vancouver.
`execution=false`. Fresh independent review is required next.

This package repairs both P2 findings against exact R5 commit
`6b7a029ffbd8e41b411408bbcaf9b219e1f6592c`. It contains all seven inert source
texts and an exact [patch from R5](repair-from-r5.patch). Only controller and build
text change. Common, observer, qualification, runtime, and postflight remain R5
bytes. No source was materialized as a program or run.

The full R6 review at `7ce44f6a02890b89cffca407a78497e0ba9bbddb` has SHA-256
`e19ebf2089afffda0d874e84c81962f865bf9d9ceffa309d34162910f0fd71b5`.
The required R2 package at `7d92633149204f834b4da55e352e7d05fd2abe4a` and full R4
review at `6b0e4f12469a28f89badbd6bb3dc80ee0de45f40` are bound in
[old-input-hashes.json](old-input-hashes.json). The R5 source, plan, input, recovery,
static, and hash records were read as data. R2's complete source and patch were
covered by full source reads, byte comparisons, and exact patch materialization.
R4's source-only verdict does not cover this new whole candidate.

The controller rejects root, drive, absolute, parent-traversal, and empty paths
before transport. Mapping normalizes the exact TEMP parent and each joined path,
requires a strict descendant, and requires nine distinct normalized roots. It
keeps the nine exact historical labels and private selection hash. It invents no
path. Common and observer need no change.

The build now registers its process object before creation and pins the root
handle before stdin, stream, identity, or receipt work. A lifecycle
`try/catch/finally` covers creation through the final receipt. Every caught failure
settles the root and verified descendant handles, preserves the first error,
records signal/actual exit/closure, and fails the stage. Recovery does not depend
on CIM succeeding again, a transport timeout, SSH exit, or ambient job cleanup.
The controller also reaches the single postflight path after a stage exception.

Read [RESPONSES.md](RESPONSES.md), [DESIGN.md](DESIGN.md), and the complete finite
[NEXT-RUN.md](NEXT-RUN.md) for control flow, budgets, and proof limits.
[static-materialization.json](static-materialization.json) records in-memory
application of R2, R5, and new patches. [source-preservation.json](source-preservation.json)
records all seven candidate hashes, all 178 immutable historical members, and all
558 source-manifest members. [SHA256SUMS](SHA256SUMS) binds this package.

The parent reports that it uniquely recovered the exact existing selector by
matching the historical R6 signature against 54 known public configuration
variants. This task did not inspect, copy, locate, hash, or use that private
selector. The nine exact historical relative fixture paths remain missing. They
still block runtime. No new human-permission gate is added to the authorized
virtual CLI scope.

R5's exact source archive/manifest recovery remains bound to source
`e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a`. This task did not recreate an archive,
inspect private recovery files, or recover the old executable, raw build receipt,
state, or evidence archive. The derived published build receipt cannot replace
the missing raw receipt. No historical evidence was rebuilt or substituted.

The old `6010722042604f77842db3ce491bee42f52a1265` virtual Off flow remains
**FAILED / UNVERIFIED**. Its native strict Clippy and release-build PASS apply
only to their historical source/binary scope. This new whole candidate has no
native syntax, build, runtime, normal-stop, signing, or release acceptance.

Only this new folder is owned. The checkout started clean at
`d7b5b9ea38b92139dda72d1c139efbeb16ed9c10`. All previous files and guards remain
unchanged. Checks used only local file/Git/hash/JSON/AST/patch data operations.
No supplied helper/module/checker/AST was imported, compiled, evaluated, or run.
Python AST parsing was data-only. No PowerShell/C# parser, compiler, test, mock,
simulation, fixture preparation, preflight, product, native/remote command,
process/OS/port/job query, desktop drive, network connection, credential/key/private
seed access, delegation, push, or publication ran.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login,
services, tasks, accounts, firewall, global DNS, and toolchain defaults stay
unchanged. No TCP 24800, UDP 5353, or Docker action occurred. Crafted mDNS,
original provenance/security, IPC/Synergy 3 import budget, and isolated Swift
gates stay stopped. The closed expired Sparkle one-shot has no retry, renewal,
reset, or approval reuse.
