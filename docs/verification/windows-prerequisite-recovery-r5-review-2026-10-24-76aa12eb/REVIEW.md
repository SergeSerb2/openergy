**Windows prerequisite recovery R5: independent source review, 2026-10-24**

Verdict: **two actionable P2 findings; no P1 finding.** This is a source and
saved-data review. It provides no runtime acceptance, source integration,
executable receipt, or permission to run a native stage.

The exact held candidate is `6b7a029ffbd8e41b411408bbcaf9b219e1f6592c`, folder
`docs/verification/release-integration-2026-10-23/windows-prerequisite-recovery-r5-86e0bfd1/`.
All finding locations below refer to that folder at that commit. Its actual Git
parent is `15de5953704d811533d0769236c51c3dfe4b8e30`. The integrated R5 copy at
`89acb738189d2c3ba978d8c39d50140df5e94dfb` has the same 21 file contents. This
review checkout started clean at parent base
`133a44cbdfec90fec0ad9f34eab0923b588c454c` in
`/Users/serge/.t3/worktrees/openergy/t3-76aa12eb`. Content equality was checked;
the original held commits are not asserted to be ancestors of the integrated
copy.

**P2: reject rooted Windows fixture paths before transport.**

Location: `source/controller.py.txt:73-76`, consumed at `:173`; traversal is in
`source/common.ps1.txt:16-29`.

Trigger: one otherwise hash-bound, correctly labelled selection entry starts
with a Windows root separator, has no drive, and contains no `..` component.
`PureWindowsPath.is_absolute()` requires both a root and a drive on Windows.
Thus this entry passes every check at lines 74-75. The later join keeps the TEMP
drive but replaces its directory with the rooted selection. It does not remain
under `original_temp`.

Effect: the source can admit a fixture tree outside the selected task TEMP
parent and recursively read its entries, file bytes for hashing, and ACLs under
one of the nine accepted labels. The separately reviewed selection hash binds
the data; it does not repair this containment check. This defeats the explicit
relative-path gate before the first transport operation. No bad selection or
outside-tree read was observed or created in this review.

Fix: reject `p.root` as well as `p.drive`, absolute paths, and `..` before any
transport. At mapping time, also require each normalized joined path to be a
strict descendant of the exact normalized TEMP parent. Preserve the nine labels,
exact historical path requirement, private hash binding, and refusal on missing
facts. Do not guess a replacement path. The finding follows from reading the
controller and local standard-library path source; no crafted path was evaluated.

**P2: keep compiler ownership recovery active on terminating build errors.**

Location: `source/build.ps1.txt:37-65`; the new build admission is at
`source/controller.py.txt:243-245`. The stage exception wrapper is at `:138-141`,
and transport recovery is conditional on a timeout at `:98`.

Trigger: Cargo is still running when a terminating error occurs after launch,
for example in the `Get-CimInstance` poll at build line 46. Identity acquisition,
receipt writes, and output waits after launch also have fallible paths. The
build body has no encompassing `try/finally`. Its kill loop runs only when the
normal polling loop reaches 600 seconds.

Effect: the exception leaves that loop and the native wrapper writes the stage
error and exits 1. The build child is not explicitly terminated, waited on, or
disposed on that path. A timely nonzero stage result does not trigger the outer
transport-timeout recovery. Postflight observes owned processes but does not
recover them. Therefore the newly admitted compiler work can outlive its failed
stage without the declared bounded owned recovery. Root retention preserves
files but does not settle live compiler ownership. The old build's successful
receipt does not cover this error path, and the R2 binary-reuse candidate had no
compiler stage. Ambient SSH/job behavior was not inspected and cannot supply a
source guarantee here.

Fix: cover the lifecycle from successful process creation through final receipt
with a bounded `try/catch/finally`. Retain the root handle before fallible
observation, preserve the first error, and settle the exact owned root and
verified descendant generations on every failure exit. Record signal, actual
exit code, and handle closure. Keep the existing step/recovery/overall budgets,
one release command, zero Clippy commands, and no retry. This is a source control
flow finding; no compiler or injected failure was run.

**Preserved R2 behavior and historical limits**

The complete bound R2 candidate at
`7d92633149204f834b4da55e352e7d05fd2abe4a` and R4 review at
`6b0e4f12469a28f89badbd6bb3dc80ee0de45f40` were covered. All R2 records and the
complete repair patch were read. The six-file source was assessed through the
complete R5 texts, both exact patches, and byte comparisons. The R4 document
was read in full. Its no-actionable-P1/P2 verdict applies to its source-only R2
scope; it supplies no native syntax or runtime proof for this adapter.

R5 `common.ps1.txt` and `observer.cs.txt` are byte-identical to R2. Runtime bytes
from `$binary=` to `$result=` and from `function Pipe-Exists` through final exit
are identical. Only build admission and artifact/receipt labels changed. The
controller's `command`, `remote`, and `recover_transport_owner` methods are
byte-identical to R2.

The read-only log stream still uses `FileShare.ReadWrite|FileShare.Delete`, one
pinned handle, at most 4,096 bytes per poll, and the 2 MiB length/offset limit.
Only `FileNotFoundException` is caught as pending; directory, access, sharing,
and other faults are not suppressed. The config directory is created first.
The single 15-second readiness deadline starts before daemon resume and rejects
late markers. The marker still does not establish health. Transparent wire
forwarding, exact method and serialized non-null IDs, real client/server PIDs,
exact stop ACK, independent retained-handle exit 0, the full ten-second stop
allowance, observer settlement, ownership checks, and failure/cleanup receipts
remain R2 source bytes. Neither finding changes that assessment.

The added build source specifies one direct pinned Rust/Cargo 1.99.1 release
build, locked/offline resolution, jobs 2, native MSVC target, private Cargo/target/
TEMP paths, and no Clippy command. Its fresh receipt and runtime copy use actual
new artifact size/hash, not the old binary hash as an acceptance target. The
new baseline compares current before/after selected state. Private collection
adds state, transfer metadata, native sources, and fresh receipt bindings.

The complete seven R5 inert texts, adapter.patch, README, proposal, NEXT-RUN,
baseline contract, input/missing/path/transport/recovery/durable/static/hash
records were read. PowerShell and C# syntax were inspected as text only. No
PowerShell/C# parser, import, compiler, or supplied helper was used. The encoded
stage wrapper, explicit native argument list, fixed transport options, immediate
stdin EOF, private outputs, safe-member checks, and removal gates were traced.
The missing exact selector and nine fixture path bindings remain data
prerequisites. They are not findings by themselves. Since their values are
absent, this review establishes no target-specific alias syntax, remote-shell
path quoting, or actual host-path compatibility. No selector, secret, SSH config,
key, known-host inventory, or private payload location was sought.

The original saved evidence at
`6010722042604f77842db3ce491bee42f52a1265` remains **FAILED / UNVERIFIED** for
virtual Off stop. The native strict Clippy and release-build PASS receipts for
`e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a` remain historical evidence. Version
exit 0 and absent-daemon status exit 1 passed. The Off daemon then launched and
the active-log reader failed with the Windows sharing error before healthy
status/config, invalid method, or `daemon.stop`. There is no historical stop ACK,
normal-stop timing, normal daemon exit acceptance, or forced-recovery exit code.
No release linker was sampled; its priority remains UNVERIFIED.

**Independent data checks and exact bindings**

All 21 R5 files match the held commit and integrated `89acb738` copy. All 20 R5
SHA256SUMS entries match. All 14 R2 files and its 13 checksum entries match their
bound commit. All 20 R5 input records and all 25 R2 input records match Git blob,
size, SHA-256, and current checkout bytes. The R4 file is 10,311 bytes, SHA-256
`71a1a7c51b4bc867ecc15c090047882b629a9ff77406a5fd8db3246a3c818366`.
JSON decoding and Python AST parsing passed as data checks, without supplied
module import, evaluation, or execution.

Both patches were independently applied only to immutable bytes in memory.
Every output matches its candidate. R5 hunk counts are controller 7, qualify 2,
runtime 2, postflight 2, build 3; common/observer have no hunks. R2 counts are
controller 10, observer 4, postflight 2, qualify 2, runtime 8; common is unchanged.

Every one of the original 178 published inventory members matches its recorded
size/SHA-256 and exact `6010722` Git bytes. The inventory itself also matches its
bound input. No original member, source, guard, or evidence file was edited.

The exact source-manifest bytes are 188,309 bytes, SHA-256
`a208cbb73d29b4939fcd4a43eae67769d1763823fc6a77979bb7abe96ef243dc`.
A data-only Git archive was derived in memory from the published recipe and
immutable source. It matches 4,065,280 bytes, SHA-256
`775e04301b1f3816ccf39fd088e1100da4344be5b171df7330b0049a7f589adf`.
All 558 members independently match their Git blobs, sizes, SHA-256, CRLF counts,
and LF counts. No archive was written. This confirms the published recovery
bindings; it does not recheck the private recovered files' current retention.

The old executable, raw build receipt, state, evidence archive, and exact
transport remain missing according to the bound recovery records. The published
derived build-result hash
`65c8aa80c8e179201039111724bb5735ee62696ab2e5ed9414e20323e8b44cbe` differs from
the missing raw receipt hash
`f52b45d2cbea71c19cc1b6d43eee6b6ce99b2072b852a5cd29f688f677213eca`.
No old executable or raw receipt was recovered or synthesized by this review.

Exact R5 source SHA-256 values:

- `source/build.ps1.txt` (8,803 bytes): `d4978e13b1e20b70f4aaf06e097d98666c224e67c48e36598147da5113c648d8`
- `source/common.ps1.txt` (6,364 bytes): `c09ce6f29820bd3805fec5398f49de9fe797d71e57a0e3e4f233f56f51043ce0`
- `source/controller.py.txt` (19,995 bytes): `880333ebb6039261033eae04b22e8df2f6b87f30fa1ec4fe7c377213bd3910fb`
- `source/observer.cs.txt` (12,054 bytes): `8559ecbca18de3b896407df1048c278633c927134858cc99124ae883e816f5e2`
- `source/postflight.ps1.txt` (4,749 bytes): `77456c370740da9ef7e86a4bbb9046342a239c7b9d9b72b9da34c43eb9bdbd2a`
- `source/qualify.ps1.txt` (4,272 bytes): `3d9f926395adbdeee29518a9fdaa164ddb7545db67747f3f2ec9023ac2729d22`
- `source/runtime.ps1.txt` (21,487 bytes): `986b6246e79f8ecb9ddefa32bcfa23ca0aec6c31af44b97915c45bc481f9e615`
- `adapter.patch` (37,264 bytes): `f2879ba5e979ffa52d8728ea560a6a1664fa05574100787df329679ec455843b`
- `SHA256SUMS` (1,728 bytes): `cbaeb903eeaa7d36b6566bafff2aad6d591a4c1b36e4cc59472b02e82393933c`

Only this new review document was authored. No task code, parser compilation,
native query, process/OS/port/job query, connection, build, preflight, product,
test, mock, simulation, signing, delegation, push, or publication ran. No private
payload, transport value, authentication data, or fixture was read or prepared.
PowerShell/C# syntax and all new runtime behavior remain UNVERIFIED. Uncapped
output sinks, unsampled processes, external SSH/job/handle state, total resource
bounds, and preemption of arbitrary OS calls remain proof limits.

Live sharing, installed apps, production config/TLS/default IPC, TCC, login/
services/tasks/accounts/firewall/global DNS, and toolchain defaults were not
changed. No TCP 24800/UDP 5353 or Docker action occurred. Crafted mDNS, original
provenance/security, IPC/Synergy 3 import budget, and isolated Swift build gates
remain stopped. The expired closed Sparkle one-shot was not retried, renewed,
reset, or used as approval authority.
