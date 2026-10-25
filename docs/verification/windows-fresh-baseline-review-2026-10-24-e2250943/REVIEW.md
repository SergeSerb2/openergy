# Independent review of the fresh Windows baseline candidate

Date: 2026-10-24, America/Vancouver.

**Source HOLD: one actionable P2; no P1 found. Native syntax and runtime remain
UNVERIFIED.** This is a static review. It creates no approval receipt and does
not authorize or start a Windows operation.

The exact candidate is `9911db75fa07d0383b81366d319475ded016c066`, folder
`docs/verification/windows-fresh-baseline-plan-2026-10-24-a7b99aef/`.
All source locations below refer to that folder at that commit. This review
started clean at `ce742a5d3c4488501ae706fe824973bd5c23bddf`, branch
`t3/e2250943`, in `/Users/serge/.t3/worktrees/openergy/t3-e2250943`.
The 22 candidate files are byte-identical in the candidate and this checkout.
The candidate's actual parent is `ed7d96bf53da1a539fa2acde3b7133613221fd4b`.
The imported checkout has other documentation ancestry; that does not change
the reviewed package bytes.

## P2-1: keep the exact-owner timeout recovery command within Windows limits

Locations: `source/controller.py.txt:122-158`, especially `:130-133,156-157`.
The new callers concatenate the full safety module at `:228-230,363,380`.

Trigger: the setup, cleanup, or retained-root-audit remote command times out.
`command` then calls the unchanged `recover_transport_owner(original)`.
That method encodes the entire original script as UTF-16LE Base64, inserts
that string into a second script's exact `CommandLine.EndsWith` selector,
and sends the second script through `remote`. `remote` encodes the second
script as UTF-16LE Base64 again and passes it to native `powershell.exe` as
one `-EncodedCommand` argument.

The new safety module is 5,332 ASCII characters. Counting only that module,
and ignoring every caller statement and every recovery statement, gives:

- First encoding: `4 * ceil((2 * 5332) / 3) = 14,220` characters.
- Encoding that embedded text in the recovery command:
  `4 * ceil((2 * 14220) / 3) = 37,920` characters.

That lower bound already exceeds the Windows native Unicode command-line
limit of 32,767 characters, including its terminating NUL. The executable,
options, recovery body, original caller body, and substituted input data only
increase the length. This conclusion does not depend on a particular fresh
selection, TEMP path, SSH alias, default remote shell, or current host state.
A remote shell can have an even smaller limit; no such shell was inspected.

Effect: the declared one-shot exact transport-owner recovery cannot launch
its intended native PowerShell process on these timeout paths. Its zero-exit
assertion cannot establish settlement. A setup timeout can also occur before
the local controller obtains a root receipt. In that case the later known-root
collection/audit branch is unavailable. Native finally blocks and possible
ambient transport behavior do not replace the promised explicit recovery.
No timeout or leftover process was executed or observed in this review.
This is a source defect in the new payload sizes, not a new general demand
for perfect OS-call preemption.

The method's byte equality with R9 does not prevent this regression: R9 did
not concatenate this safety module into those remote commands. The callers'
new data exceeds a limit of the preserved method.

Repair the transport construction so its recovery command has a bounded size
independent of the original script length. Preserve the exact unique-owner
selection, retained-handle/generation checks, descendant restrictions, one
recovery attempt, immediate stdin EOF, and single overall deadline. Add a
static size admission check before launching an affected remote command.
Do not truncate the selector, broaden it to a name-only/PID-only kill, reset
a guard, retry a flow, or extend the 30-second transport reserve. Review the
complete repaired candidate before future execution. No repair was made here.

## Fresh baseline and safety assessment

The complete README, NEXT-RUN, plan, input contracts, result schema, responses,
all eight inert source texts, and every line of the exact R9 diff were read.
The continuation record and original repair/review records were also inspected.
The chain covered R2 `7d92633149204f834b4da55e352e7d05fd2abe4a`, R7
`1d01d672c93b03d089d6d902f5bc7f15c10ffd05`, R8
`fe22c54d890ab7ecbfd58845040aba3ce87d47bc`, R9
`762240ac041a835707b7ae3867b4e15b1e06b8c0`, and R10
`fd8db892485f5d4d7117045c71b7c4bd2ee562ff`. Prior verdicts were not adopted
as clearance for this new candidate.

The replacement baseline is explicitly a new operation. It accepts 1–16
hash-bound, operator-selected IDs and single-component child names, distinct
without regard to case. Admission rejects Windows roots, drives, UNC/device
forms, parent components, separators, streams, reserved device names, trailing
dots/spaces, and shell characters through the combined path and ASCII gates.
Setup creates new children and nonempty NEW sentinels under this operation's
`preservation` directory. It does not map an arbitrary existing TEMP subtree.
Missing historical paths are neither reconstructed nor treated as empty.

`Fresh-Path` requires a normalized non-root local drive path. Setup checks
actual TEMP and every existing ancestor before creation, with an additional
restricted TEMP alphabet for fixed legacy scp. `Fresh-Owned` checks a direct
child name, current SID, protected one-rule FullControl ACL, nonce marker,
and numeric root creation file time. `Fresh-Tree` checks each child for a
reparse point before descent. Removal rechecks ownership/ancestry and deletes
individual entries deepest first. Unknown process references refuse removal;
they grant no kill authority. Setup failures attempt bounded owned cleanup.
Incomplete collection retains the root and attempts one bounded audit.
These are source checks, not native filesystem proof or elimination of the
explicitly disclosed same-user races between checks and operations.

Removing the nine historical fixture requirements does not remove the current
registry/toolchain/default/tool/sharing snapshots, active-compiler refusal,
private registry-copy byte checks, source checks, ownership gates, or private
collection-before-removal requirement. The removed failed-root absence query
was an old historical-path assertion. Its absence is explicit in this new
contract and is not represented as proof about that old path.

The result schema fixes historical preservation to UNVERIFIED, original runtime
to FAILED_UNVERIFIED, historical qualified bindings to 0/9, changed historical
results to false, and original stop requests to zero, including on a future new
PASS. Before/after/postflight state uses the NEW scope. The controller also
requires decoded new before/after equality, exact state bytes, successful
bound build/runtime receipts, postflight success, verified archive membership
and hashes, and successful exact-root removal before new acceptance.

## Preserved build and runtime contracts

Independent comparison confirms that build changes only its source-commit
receipt expression. Runtime changes only its source-commit admission
expression; all bytes from `$binary=` through EOF remain exact R9 bytes.
The observer is exact R9 and R2 bytes. Controller `command`, `remote`,
`recover_transport_owner`, `stage`, and `export` remain exact R9 source.

The R9 direct `Process.GetProcessById` call and narrow `ArgumentException`
catch remain. A missing sampled child records a race before any owner is
created. Returned objects still require retained handles and exact executable,
command-line, generation, parent, live retained-parent, priority, and session
checks. Unknown, reused, or provisional generations gain no kill authority.
The creation-associated Cargo root is registered before launch and retains
its handle before fallible identity or receipt work. The outer build
catch/finally preserves the first error, signals verified live handles before
blocking waits, visits all registered handles, drains streams within the
shared reserve, and independently attempts process/safe-handle disposal.
Forced compiler recovery and missing or late receipts fail the stage.

The sole release command remains locked/offline Rust/Cargo 1.99.1, target
`x86_64-pc-windows-msvc`, jobs 2, MSVC 14.44.35207, SDK 10.0.26100.0, with private
Cargo/target/TEMP paths and auto-install disabled. There is no Clippy command,
compiler retry, global toolchain switch, or old-binary acceptance path.

Current-source admission binds exact clean HEAD/tree, the seven export paths
including Cargo.lock, every Git blob/size/hash/line ending, manifest and archive
bytes/hashes, and safe regular archive membership before transport. The
historical e1ef124 default is refused. The ed7d96bf snapshot is data only; the
future parent's exact clean commit still needs its own retained raw Git export,
manifest, and concrete hashes before any future command. This review created
none of those future execution inputs. The reference's 558 product files do
equal the old product bytes, but its commit/archive identity is separate.

The preserved real flow requires version exit 0, absent-daemon status exit 1,
one private foreground virtual Off daemon, real status.get/config.get and an
invalid-method error, exact wire methods/non-null serialized IDs and actual
CLI/server PIDs, one daemon.stop forwarding attempt, exact stop stdout ACK,
empty stop stderr, stop CLI exit 0, and independent retained-handle daemon
exit 0 with the original controller alive and original daemon generation gone.
The R2 reader retains FileShare.ReadWrite|Delete, one pinned read-only stream,
4,096-byte polls, a 2 MiB cap, only FileNotFoundException as pending, and a
single 15-second pre-resume readiness window that rejects late markers.
Readiness remains separate from health. Observer settlement precedes the
final forwarding-tick snapshot. The full ten-second normal-stop allowance
precedes eligible recovery. Forced recovery cannot supply normal acceptance.
Current Rust CLI/logger/stop-dispatch source was read as text; none was run.

## Finite limits and evidence

The outer workflow keeps one 2,400-second clock and a final 30-second transport
reserve. Setup is 45 seconds including at most 30 seconds of settlement, with
a 60-second outer ceiling. Qualification/postflight each have a 600-second
outer ceiling. Build has 600 seconds of work, one shared 30-second settlement
reserve, final receipts by entry + 630, per-handle waits capped at 10 seconds,
per-stream waits capped at 5, and full 660-second outer admission. Runtime is
150 seconds native/180 outer; CLI and observer share 15 seconds per exchange;
stop entry reserves 40 seconds for CLI 15 / normal exit 10 / recovery 15.
Upload/download/removal ceilings are 90 seconds each. Retained-root audit is
30 seconds native/45 outer. Local Git reads cap at 15 seconds and SSH connect
at 8. These are non-additive ceilings inside the one workflow budget.
P2-1 blocks the declared recovery despite those unchanged timer values.

Source ceilings are 2,048 files, 4,096 archive members, 64 MiB archive, 128 MiB
payload, and 4 MiB manifest. Native tree walks cap at 200,000 entries per tree.
Collection caps at 512 flat unique regular members and a 512 MiB archive.
Actual local NI 15/deployment target 15.0 and Windows BelowNormal/session 0
gates remain at admission/setup/common/product/compiler checks. The source
does not prove priority for an unsampled linker or every external process.

[Independent static data checks](static-data-checks.json) retain these results:

- All 22 package files and 21 manifest members match. All nine JSON files
  decode. All eight source bindings and all 48 public input bindings match.
- Regenerating the complete R9 unified diff from immutable source bytes gives
  the exact saved 55,194-byte patch. Python AST handling was data-only.
- All 178 published historical inventory members and all 179 complete-folder
  files match their recorded hashes and immutable `6010722` bytes. The candidate
  adds only its new package and changes no earlier tracked file.
- All 558 current reference source records match Git blobs, sizes, hashes,
  and LF/CRLF counts. A read-only Git archive derived only in memory has 623
  members, 4,065,280 bytes, and SHA-256
  `7a333dd962419984e480eac465b63ff49fc2339ec8fb1bf4bbc522815d2af012`.
  No archive was written or extracted to disk.

The requested package bindings match exactly:

```text
manifest   c79c4491cb9b95f86ef5cc0e7fb2e975ec41a9b0c4aede7be69142c913de00f7
R9 diff    1412984d383e8b2546a94f068c979fdb34b0650ee55b443b3b092264ba27afad
controller 2852b0a592d87510d066ca3803535a64ac176780880eba8b1328a4f6d939a764
```

## Historical and native proof limits

Original saved README/result/runtime-error records confirm the FileShare
reader failure before healthy status/config, invalid method, or stop. The
original Off result stays FAILED / UNVERIFIED with zero daemon.stop requests.
No old stop ACK, normal exit 0, stop timing, or saved forced-exit code is
supplied. The bounded historical recovery remains exhausted at 0/9 qualified
bindings after 160 charged retrievals. This new operation cannot close that
historical preservation gap. No old paths were requested, guessed, or searched.

Native PowerShell/C# syntax, filesystem/ACL/reparse behavior, process generation
and handle behavior, compiler settlement, real CLI/wire results, cleanup, and
normal stop remain UNVERIFIED. The disclosed arbitrary OS-call preemption,
same-user path races, unknown/unsampled generations, external SSH/job/handle
state, release-linker sampling, uncapped compiler/SSH output, and total
memory/disk limits remain. No native reproduction or frequency is asserted
for P2-1. No hardware, Windows desktop input, full host restoration, packaging,
signing, security/provenance, release, or publication acceptance follows.

Only this new review folder was written. Checks used file/Git/JSON/AST/diff/hash
data readers and literal-length arithmetic. No supplied helper/module/AST was
imported, evaluated, compiled, or executed. No PowerShell/C# parser, test, mock,
simulation, fixture setup, build, product, process/OS/port/job probe, network,
transport, credential/key/selector/private-recovery access, desktop action,
delegation, push, or publication ran. The saved selector's recovery is taken
from the brief; no host availability is inferred. The M1 update supplies no
Windows desktop-input scope. Stopped gates and the expired closed Sparkle
operation remain unchanged, with no approval reuse. Git hooks and commit
signing are disabled for this report's delivery commit.
