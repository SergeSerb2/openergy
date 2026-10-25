# Independent Windows R3 source review

Date: 2026-10-24, America/Vancouver.

**Source CLEAR for commit `23b41681e4019f086f1f760c9817553d79acff04`:
no concrete P1 or P2 found. Native syntax and runtime remain UNVERIFIED.**
The prior unconditional oversized-recovery defect is repaired in source.
This report creates no approval receipt and starts no operation.

The reviewed package is
`docs/verification/windows-fresh-command-repair-r3-2026-10-24-4d078c08/`.
Its tree is `a22352120b5cba5971c4402d416db4e031295026`; its parent is
`9a540b5ef78bedf9cf10e43eb66cdb359adcdbbc`. The review started clean on
`t3/821487e9` at `1f5b72e491bfbe2872dbe9f233e599ff786d6f64`.
All 24 supplied package files match the immutable candidate commit.
R1 `9911db75fa07d0383b81366d319475ded016c066` and review
`2c8e67ea85dadd2046cf98c9a8cafc75fcc3afbe` were read as independent inputs.
Locations below refer to the R3 package at the reviewed commit.

## Command construction and decoder

`source/controller.py.txt:34-83,189-199,230-242` contains the changed path.
Setup, cleanup, retained-root audit, and all four native stages use `remote`.
It encodes the main script once, inserts that exact encoded argument into the
complete unchanged recovery body, and encodes that recovery body separately.
Both complete argv and every bound launcher layer pass admission before
`command` can launch the main command. Admission records bind both scripts,
wrappers, argv, widths, and launch binding. Recovery uses the precomputed argv
directly and does not call `remote` again. The other two `command` callers are
the unchanged fixed-file upload and download paths.

The source refuses NUL, invalid UTF-8 input, zero or more than 65,536 input
bytes, and more than 10,400 compressed Base64 characters. Encoding, compression,
missing/malformed binding, and width errors propagate before that command's
launch. There is no truncation, ratio assumption, fallback encoder, or selector
shortening. Each actual Windows serialization includes quotes, separators,
UTF-16 code units and NUL. Launcher NUL/encoding/limit failures also propagate.
The separate inner PowerShell 7 gate includes the quoted executable, all flags,
both actual root occurrences, stage names, and failure-receipt path. It runs
before the outer stage launch; compression cannot hide an oversized inner line.

Independent literal arithmetic confirms the 1,588-character wrapper template,
maximum 12,025-unit substituted wrapper, and maximum 32,068-character encoded
argument. The submitted bare-name argv has a 66-unit prefix and therefore a
maximum of **32,135 units including NUL**, with 632 units of margin. The full
recovery source is at most 34,183 UTF-8 bytes, but still must pass its own
compressed-literal cap. Thus an incompressible recovery refuses the main launch.
This closes the old unconditional 37,920-unit lower-bound defect. It does not
promise that every future selection fits every supported route. The Windows
32,767 limit includes the terminator. [Microsoft CreateProcessW documentation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)

The unchanged C# product launcher quotes each argument. The inner-stage gate
also bounds the root width conservatively at 6,143 units. The daemon line has
two root occurrences and less than 1,024 units of other text, quotes, separators,
nonce and ten-digit PID allowance. Its conservative bound is 13,311 including
NUL; the other product CLI lines are smaller. The macOS SSH/scp argv, native
Cargo/tool/tar calls and compiler-generated commands retain their prior scope.
No new compiler-generated command-size or native acceptance claim is made.

At `:48-69`, the decoder reads into a 4,096-byte buffer and checks expanded
length before writing. It then requires exact final byte length and SHA-256,
uses strict UTF-8 decoding, and creates the script block only after those gates
and resource disposal. Its catch keeps the first error. Its finally attempts
all four resource disposals independently, and a disposal error also prevents
script invocation. A decode/parse/terminating error has no success fallback.
The prior inner stage still ends with explicit `exit $LASTEXITCODE`.

Primary sources support the chosen wire representation: Python documents
negative window bits as raw DEFLATE; Microsoft's .NET Framework DeflateStream
selects its raw inflater when no format reader is supplied, using the -15
constant. This is a source compatibility assessment. The installed decoder
was not exercised or identified by this review.
[Python zlib documentation](https://docs.python.org/3/library/zlib.html),
[Microsoft DeflateStream source](https://github.com/microsoft/referencesource/blob/main/System/sys/system/IO/compression/DeflateStream.cs),
[Microsoft ZLibNative source](https://github.com/microsoft/referencesource/blob/main/System/InternalApis/NDP_FX/inc/ZLibNative.cs)

PowerShell documents UTF-16LE for EncodedCommand and failure status for
terminating errors. The public engine source also marks direct script blocks
as non-file invocations and propagates their exit exception. These references
support the wrapper design; they do not establish the installed Windows
PowerShell build's behavior.
[Windows PowerShell command documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_powershell_exe?view=powershell-5.1),
[ScriptCommandProcessor source](https://github.com/PowerShell/PowerShell/blob/v6.0.0-alpha.9/src/System.Management.Automation/engine/ScriptCommandProcessor.cs),
[CompiledScriptBlock source](https://github.com/PowerShell/PowerShell/blob/v6.0.0-alpha.9/src/System.Management.Automation/engine/runtime/CompiledScriptBlock.cs)

## Supplied private launcher observation

The parent's saved metadata, command, argv, receipt, stdout and stderr were read
as data. The receipt SHA-256 matches
`5fd1f40dc8caf85226a9ce047862d5c72a34ef0d6ab68aa02f96775ffc2a2c00`.
Its command/stdout/stderr hashes match; decoded metadata equals stdout. The
saved result is exit 0, zero retries, with no task product/controller execution
reported. The nonempty stderr is CLIXML containing progress records only.
No endpoint, username, full private path, selector hash, PID chain or encoded
payload is exported here. The reviewer made no metadata query or route launch.

The current query records PowerShell 7 as OpenSSH DefaultShell, with CommandOption
and EscapeArguments absent. Its observed shell command exactly fits the supported
literal shape: a 45-unit prefix consisting of the quoted PowerShell 7 executable
and `-c "`, the entire submitted native line, and one closing quote. The child
retains the exact encoded-argument suffix used by recovery selection.

The child command expands the executable to the quoted system Windows PowerShell
path. Its fixed prefix is **111 units**, which adds 45 to the submitted bare-name
prefix. For this observed shape, the inferred maximum child line is therefore
**32,180 including NUL**. The shell line is **32,181 including NUL**, with 586
units of margin under 32,767. The shell admission counts one more unit than that
expanded child line. Both main and recovery commands use the same capped encoded
alphabet, so these are symbolic bounds for that shape, without encoding or
launching a candidate command. R3's 32,135 figure describes submitted argv;
the actual child-prefix expansion must remain in the route evidence.

These facts can support an exact literal-shell template for the observed query.
They do not establish arbitrary future nested routes or an installed argument
construction implementation. The future private `windows_launch_binding` still
must bind the same exact selector, the unchanged launcher/configuration and
argument construction, all relevant layers, and each effective limit. A positive
limit of 32,767 must have that supporting evidence; the observed lengths alone
do not prove a limit. The complete future preparation, source export/manifest,
selection and route binding are not materialized by this report. Unknown,
nonliteral, changed or smaller-limit routes must refuse admission. The two
binding booleans remain admission assertions. This source verdict supplies none
of the missing private execution inputs.

## Ownership, failure paths and prior repairs

All seven native files are byte-exact R1. The complete recovery PowerShell literal
is also byte-exact R1. It still selects the complete original encoded suffix,
requires a unique owner, limits candidates to that owner and sampled descendants,
rechecks creation time, command line and executable, retains a process handle,
checks its start time/path, waits for a signal and disposes it. Zero owners retain
the original explicit outcome. Non-unique or changed identities refuse kills.
No name-only, PID-only, digest-only or broader kill path was added. The native
literal preservation does not eliminate its disclosed sampling/OS race limits.

The command guard, one recovery guard, immediate stdin EOF, strict SSH options,
2,400-second workflow clock and final 30-second transport reserve remain.
Recovery timeout/nonzero exit fails its assertion and cannot supply normal-stop
acceptance. No timer restart or retry was introduced. Local receipt failures,
lost setup responses, external SSH/job state and arbitrary OS-call preemption
retain their stated uncertainty. Collection/audit failure preserves the first
flow error and UNVERIFIED status.

The selection still rejects rooted/drive/UNC/device/parent/stream/separator forms
and admits only bounded, case-distinct ASCII child names. Setup validates actual
normalized TEMP and reparse ancestors before creating a fresh direct child.
Removal retains SID, protected ACL, nonce, numeric creation-time, containment,
per-entry reparse and process-reference refusal gates. Unknown processes grant
no kill authority. Setup failure attempts bounded owned cleanup; incomplete
collection retains the root and attempts one bounded, non-destructive audit.

The R9 build body differs only in its inherited source-binding receipt expression.
The direct `Process.GetProcessById` call and narrow `ArgumentException` catch
remain. Root ownership is registered before launch; its handle is retained before
fallible identity work. Sampled children require exact generation/executable/
command/parent and a live retained parent. The outer catch/finally retains first
errors, signals verified live generations before waits, visits all registered
handles, bounds stream settlement and attempts independent disposal. Forced
recovery, late/missing receipts and unsettled ownership fail the build.
Pinned Rust/Cargo 1.99.1, locked/offline release, jobs 2, target, MSVC/SDK,
private cache/target/TEMP and disabled auto-install remain. There is no repeat
build, Clippy, old binary, global switch or installation fallback.

The observer equals R2 and R9 exactly. R3 runtime from `$binary=` through EOF
equals R9. Its complete flow from `Save 'runtime-start.json'` through EOF equals
R2. This preserves the pinned FileShare.ReadWrite|Delete reader, 4,096-byte
polls, 2 MiB cap, missing-file-only pending case and single 15-second readiness
window from before resume. Readiness remains separate from real health.
Version, absent status, healthy status/config, invalid method, wire IDs/PIDs,
one stop forwarding attempt, exact ACK, empty stderr, stop CLI exit 0 and
independent retained-handle daemon exit 0 remain required. Observer settlement
precedes the final forwarding snapshot. The full ten-second normal-stop allowance
precedes eligible recovery, and forced recovery cannot produce acceptance.

## Retained checks and limits

[Static data checks](static-data-checks.json) retain all 24 package hashes,
22 manifest-member matches, eight source bindings, 73 immutable input-binding
matches, the regenerated exact 11,720-byte R1 patch, controller method/literal
comparisons, unchanged plan sections and unchanged result schema. All 179
historical folder files match their recorded hashes/Git blobs and this checkout.
All 558 ed7d96bf reference source records match their Git bytes and line endings.
No new execution archive was created.

One overbroad reader comparison failed: all runtime bytes from `$binary=` through
EOF do not equal R2. The error and false predicate are retained. The inspected
R2/R3 diff has three changed lines for the earlier fresh-build receipt conversion.
The independent full readiness/stop section comparison and R9 tail comparison
are exact. No source, historical result or failed predicate was changed to hide
the mismatch. No test or native parser was used.

Key SHA-256 bindings:

```text
manifest   f6feb69c95277f49afe9afa83a690ac6f11b2548f7941f93628b7afed2b693a0
controller 1406633c2e098432f118b3a5904ba10183f216f8f99b81b0032137621a6dbd60
R1 diff    a9db31295da536ddd0d1711171b8d547c87f2651e8016ff90b08ae982c547020
```

The original Windows flow remains FAILED / UNVERIFIED with zero daemon.stop
requests. Historical preservation remains UNVERIFIED at 0/9 qualified bindings
after the closed 160-retrieval search. A new operation cannot change those facts.
Native syntax, decoder execution, real launcher/process/handle behavior,
filesystem/ACL/reparse behavior, build settlement, collection, cleanup and stop
remain UNVERIFIED. Same-user path races, unsampled generations, uncapped output,
total memory/disk and arbitrary OS-call preemption remain disclosed proof limits.

Only this review folder was written. Work used file/Git/JSON/XML/AST data readers,
source/diff/hash comparisons, literal arithmetic and primary documentation reads.
No task/helper import, AST evaluation, compressor/decoder exercise, PowerShell/C#
execution/parser, test, OS/process/port/job query, task network transport, build,
product/native flow, credential/key/selector/partial-recovery access, child agent,
candidate integration, push or publication occurred. The separately supplied
parent metadata query is outside these reviewer activities. Git hooks and commit
signing are disabled for the review delivery commit.
