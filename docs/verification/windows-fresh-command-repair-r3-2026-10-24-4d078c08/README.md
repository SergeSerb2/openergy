# Windows fresh baseline command repair R3

`execution=false`. This package proposes one source repair for P2-1 in the
[authoritative review](../windows-fresh-baseline-review-2026-10-24-e2250943/REVIEW.md),
commit `2c8e67ea85dadd2046cf98c9a8cafc75fcc3afbe`. The exact base candidate R1 is
`9911db75fa07d0383b81366d319475ded016c066`,
[fresh baseline package](../windows-fresh-baseline-plan-2026-10-24-a7b99aef/README.md).
This is an implementation response. Independent review of this exact R3 package
is pending. No approval, private input, fixture, guard, or remote root was created.

All [eight complete sources](source/) are inert text. Only `controller.py.txt`
changes. The seven native source texts are byte-identical to R1, including the
entire safety module. The [minimal R1 diff](diff-from-candidate-r1.patch) and
[source bindings](source-bindings.json) identify every changed byte.

The existing SSH `powershell.exe -EncodedCommand` protocol stays. Each normal
script and its precomputed exact-owner recovery script use a bounded in-memory
raw-DEFLATE literal. The wrapper checks the full expanded UTF-8 byte length and
SHA-256 before creating the script block. It keeps the first error and attempts
each memory/hash resource disposal. Recovery reads no external script file.
It selects the complete original encoded argument with the exact unchanged
selector and retains all original generation, handle, descendant, and kill gates.

Both complete outer commands must pass size admission before the original can
launch. Each admitted outer command has at most **32,135 UTF-16 units including
the executable, options, spaces, and NUL**, below 32,767. A separate check admits
the complete nested PowerShell 7 command, including the quoted executable path
and encoded root paths. See [command bounds and limits](COMMAND-BOUNDS.md).
Oversized, malformed, incompressible, or unrepresentable inputs fail before launch.
No compression ratio, convenient TEMP path, or trusted root path is assumed.

The [finite plan](plan.json), [next-run instructions](NEXT-RUN.md), and
[responses](RESPONSES.md) retain all work, settlement, and collection budgets.
The build remains one locked/offline Rust/Cargo 1.99.1 release build, jobs 2,
with zero Clippy repeats. The real R2 CLI flow, FileShare.ReadWrite|Delete reader,
and full ten-second normal-stop allowance remain exact R1 bytes.

The original runtime remains **FAILED / UNVERIFIED**, with **zero daemon.stop
requests**. Historical preservation remains **UNVERIFIED**, with **0/9** qualified
bindings after the closed 160-retrieval search. No old binding or result was
reconstructed, reopened, or changed. A future new pass cannot change those facts.

[Static checks](static-checks.json), [input bindings](input-bindings.json), and
[package manifest](manifest.json) retain data-only evidence. Checks use file/Git
bytes, JSON, Python AST as data, string arithmetic, diffs, and checksums. No task
helper or AST was imported, evaluated, compiled, or executed. No compression or
decompression helper was exercised. No PowerShell/C# parser, test, mock,
simulation, native check, fixture setup, product/build, signing, OS/process probe,
network, remote command, credential/selector/private-recovery access, delegation,
push, or publication ran. The delivery commit disables hooks and signing.

Native syntax, decoder interoperability, launcher behavior, actual ACL/reparse
and process behavior, cleanup, collection, and stop remain UNVERIFIED. The
unknown remote shell can impose another limit. Same-user races, arbitrary
OS-call preemption, unsampled generations, external SSH/job/handle state,
release-linker sampling, uncapped output, and total memory/disk remain proof
limits. This operation stays unexecuted until fresh whole-package source review
and exact future source, input, transport, and command binding. The required fresh
`preparation.windows_launch_binding` is absent here. Unknown routes fail admission.
