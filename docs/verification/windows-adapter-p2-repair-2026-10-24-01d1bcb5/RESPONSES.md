Both R6 P2 findings are addressed in source. This is an implementation response,
not a new independent verdict. `execution=false`.

P2-1: rooted Windows fixture selection escapes TEMP

The exact reviewed locations were R5 `controller.py.txt:73-76,173` and
`common.ps1.txt:16-29`. The controller's first selection pass now rejects
`p.root`, `p.drive`, absolute paths, `..` components, and empty parts. This gate
runs during construction, before the first transport operation. The complete
ordered tuple of nine labels, separately reviewed private selection hash, and
nonempty factual binding remain required. No selection or fixture was prepared.

After setup returns the exact task TEMP parent, mapping performs only lexical
Windows path operations. It normalizes that parent, requires it to be absolute,
rechecks each relative selection, normalizes each join, and compares case-normalized
`commonpath(parent, joined)` to the exact normalized parent. It separately rejects
equality with the parent. It rejects aliases among the nine normalized mapped
roots. A mapping failure precedes state-file upload and qualification, so the
unchanged `Shared` reader receives no outside mapping from this controller.

No path string was guessed, corrected, or tested with a crafted example.
`common.ps1.txt` remains 6,364 bytes with SHA-256
`c09ce6f29820bd3805fec5398f49de9fe797d71e57a0e3e4f233f56f51043ce0`.
The validation belongs at admission and mapping; common needs no new parser or
reader behavior. `observer.cs.txt` remains 12,054 bytes with SHA-256
`8559ecbca18de3b896407df1048c278633c927134858cc99124ae883e816f5e2`.
These are both exact R2 bytes too.

The parent-supplied selector update supersedes R5's unresolved-selector record
for planning only. The parent recovered the unique existing selector from the
historical R6 signature and 54 known public variants, without a connection or
credential read. This task neither verifies nor exposes that private value.
Future admission still checks its private hash and historical-selector flag.
The nine missing path facts remain a blocker; an empty selection cannot pass.

P2-2: terminating errors bypass owned compiler recovery

The exact reviewed locations were R5 `build.ps1.txt:37-65`, its stage wrapper,
and timeout-only transport recovery. The repair keeps native compiler recovery
inside the build stage, including for timely nonzero returns.

The new owner entry exists before `Process.Start`. After successful creation,
the first handle operation pins that process's `SafeHandle`. Stdin closure,
asynchronous stream setup, CIM identity, priority/session checks, sample work,
output waits, hashes, artifact checks/copy, source preservation, and receipt work
are inside the lifecycle. The root remains the creation-associated object;
recovery never reopens it from a saved PID.

Descendant admission retains a handle before identity work. It checks the saved
CIM generation, managed start time, exact executable and command line, parent
PID, and the still-live retained parent generation. A known PID is never rebound
to a later generation. A candidate that cannot establish ownership is never
killed. A retained candidate already signalled during observation is recorded
as a race. Other identity faults remain failures. Actual BelowNormal/session-0
checks remain required for root and admitted descendants.

On every caught failure, the outer `finally` first visits the exact root, then
each retained verified descendant. It signals live verified generations before
blocking on waits. Each wait uses the same remaining recovery deadline, with a
10-second per-handle ceiling. Signal and actual exit code come from retained
objects. Stream completion uses the same reserve with a 5-second wait ceiling.
A nested `finally` visits every registered handle for process disposal and
explicit safe-handle disposal, even if a wait or stream fails. Unverified
candidates receive disposal only. There is no tree kill, unknown-PID kill,
generation reuse, new recovery query, job assumption, or flow retry.

The first error is write-once in memory. Later recovery, stream, receipt, and
closure errors are separate records. Ownership records include PID, available
identity, ownership verification, kill reason/tick/call, retained-handle signal,
actual exit code or explicit null, process disposal, and handle closure. Missing
signals or closure remain failure. Forced recovery cannot establish normal exit.
Completed error-path stream data has separate recovery output names. It cannot
replace a failed ordinary output receipt. Final receipt write faults also force
a nonzero stage result after settlement. Storage faults can prevent a durable
receipt; the source does not claim that an unwritable sink can save one.

The controller requires the full existing 660-second outer build allowance
before admitting Cargo, inside the 2,400-second overall deadline and before the
final 30-second transport reserve. Native work keeps a single 600-second ceiling.
Owned settlement gets one shared 30 seconds, capped at build start + 630 seconds.
Final native receipts also must precede that 630-second ceiling. The remaining
outer margin is unchanged. The pinned release command remains one command;
there are zero Clippy commands and no retry.

`run` now attempts its single postflight/collection path in `finally` whenever
setup has returned a root. A collection exception cannot replace an earlier
flow exception. A failed build still skips product runtime. Failed or incomplete
collection retains the root under the original verification/removal gates. The
existing transport methods are unchanged; they are fallback containment, not
proof of native compiler cleanup.

Preserved R2 contracts and history

Qualification, runtime, postflight, common, and observer are exact R5 bytes.
Runtime's launch/config block and complete body from `function Pipe-Exists`
through final exit are exact R2 bytes. This preserves the shared reader's
`FileShare.ReadWrite|FileShare.Delete`, narrow `FileNotFoundException` pending
case, pinned handle, 4,096-byte reads, 2 MiB cap, and one 15-second readiness
deadline. It preserves transparent wire forwarding, exact method and serialized
non-null wire ID, real CLI/server PIDs, real status/config/invalid-method checks,
exact stop ACK, full ten-second stop allowance, observer settlement, independent
exit-handle checks, actual exit 0, and original generation/parent ownership.

The selected baseline, source checks, ACL/reparse checks, preservation comparison,
collection-before-removal gates, private raw records, and old failed-root absence
check remain required. All 178 old members and their hash bindings are unchanged.
The old executable/raw receipt/state/evidence are still missing. Source archive
and manifest recovery remain exact `e1ef124` data. Neither was rebuilt here.
The published derived receipt hash is distinct from the missing raw receipt hash.

The historical Off flow remains FAILED / UNVERIFIED. The old Clippy/build PASS
does not validate this new seven-source candidate. PowerShell/C# syntax, real
cleanup, unsampled compiler generations, arbitrary OS-call preemption, uncapped
output/memory/disk use, external transport/job/handle state, and native acceptance
remain proof limits. A fresh independent whole-candidate review is next.
