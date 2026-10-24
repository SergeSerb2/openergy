**Native Windows Off-stop controller R2 — independent R4 review**

Review date: 2026-10-23, America/Vancouver. Review worktree:
`/Users/serge/.t3/worktrees/openergy/t3-b5580f67`. The worktree started clean at
`1cc4d03113d8d78be84ad0fbc161610860455f35`.

**Verdict: no actionable P1/P2 findings within the requested source and saved-evidence scope.**
The exact candidate is clear within that bounded scope. This is a new review
round. It adopts no verdict from the interrupted R3 task. It provides no native
execution receipt, PowerShell/C# syntax acceptance, normal-stop measurement, or
general release clearance. The original native flow remains FAILED / UNVERIFIED.
The proposed run also has missing local prerequisites, listed below.

The candidate is commit `7d92633149204f834b4da55e352e7d05fd2abe4a`, directory
`/Users/serge/.t3/worktrees/openergy/t3-dbcc0160/docs/verification/release-integration-2026-10-23/windows-stop-controller-repair-r2-dbcc0160/`.
All source line references below are to the inert files in that directory at
that exact commit. The complete README, NEXT-RUN.md, all six source files,
repair.patch, inputs.json, retained-inventory.json, static-validation.json, and
SHA256SUMS were read fresh. The folder-local .gitattributes was also read.

All 14 candidate files match their committed bytes. All 13 entries in
SHA256SUMS match independently recomputed hashes. The requested bindings are:

- `source/controller.py.txt`: `2bcb95f50e5ce621ebdc74d6c09f4f9ddb0b402370b910600bd8f3632025d54b`
- `source/runtime.ps1.txt`: `d60991664908e4e332dc70b71a99c851779a368bd0541c6734ce8f522c0ff964`
- `source/observer.cs.txt`: `8559ecbca18de3b896407df1048c278633c927134858cc99124ae883e816f5e2`
- `repair.patch`: `a945a3f14243e2b42206f74b4c142fe9cb01dd2511d035bb3148be486692025e`
- `SHA256SUMS` itself: `f7ec0bc86f386c0dff51ae127b641aae1b2870f823ce52585ca6a761e6342b35`

The unified patch was independently materialized only in memory against the six
original Git blobs at `6010722042604f77842db3ce491bee42f52a1265`. Each output
matches its candidate source bytes. The hunk counts are controller 10, observer 4,
postflight 2, qualify 2, and runtime 8. Common is byte-identical with no hunk.
JSON decoding passed for the three candidate JSON files. Python AST parsing
passed for controller.py.txt, without importing or executing it. These are data
checks only. The earlier 178-export and 558-source-blob audits were not repeated.

The review traced these source paths and found no concrete reachable defect that
warrants a P1/P2 report:

- **Shared log reader and readiness.** `observer.cs.txt:94-124` opens one pinned
  stream with read access and read/write/delete sharing. It reads at most 4,096
  bytes per poll, checks the observed length and cumulative offset against 2 MiB,
  rejects shrinkage and offset faults, and retains only marker overlap. Only a
  missing file remains pending. `runtime.ps1.txt:123-145` starts one 15-second QPC
  deadline before daemon resume, checks it around each asynchronous poll, and
  still requires actual public status/config acceptance after the marker. The
  artifact source logger at `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a`,
  `crates/daemon/src/logger.rs:30-41`, has write/append access and no Windows share
  override. Rust's documented default permits read/write/delete sharing. The
  new reader's share flags also permit the existing writer's access. This is
  compatible at the documented API level; it is not native reproduction.
  Sources: [Rust OpenOptionsExt](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode),
  [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew),
  [FileShare](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileshare?view=net-9.0).
- **Native ownership and launch order.** `observer.cs.txt:66-92` registers the
  native process and primary-thread handles while the child is suspended,
  before managed process lookup or job observation. `runtime.ps1.txt:59-65`
  requires executable, direct parent, matching start/creation time, BelowNormal,
  and session 0 before resume. `observer.cs.txt:43-64` and
  `runtime.ps1.txt:190-210` cover owned children that never resumed, record their
  actual exit codes and handle signals, and close retained handles. The explicit
  inheritable NUL/stdout/stderr list and creation flags preserve the stated
  no-new-job/group/desktop boundary. The handle-list and process-creation use is
  consistent with [UpdateProcThreadAttribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)
  and [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw).
  Unresumed recovery waits for the retained process handle after termination, as
  required by [TerminateProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess).
- **CLI, observer, and serialization.** `runtime.ps1.txt:67-114` and
  `observer.cs.txt:126-153` share one absolute 15-second deadline across observer
  setup, child creation/identity, resume, exit observation, and observer
  completion. There is no added two-second observer window. The observer forwards
  the original frames unchanged. Acceptance checks the exact method, non-null
  serialized request/response ID, and native CLI/server PIDs. Targeted reads of
  the artifact's `crates/daemon/src/main.rs:278-395` and
  `crates/ipc/src/lib.rs:45-88` confirm the real CLI's numeric ID and response
  handling; they supply no runtime proof. Active output hashes remain null when
  CLI exit has not been observed. Native network query failures stay explicit,
  and only exact owned-PID rows are saved (`runtime.ps1.txt:149-153`).
- **Stop deadline and failure recovery.** `runtime.ps1.txt:81-105,154-158`
  rejects a first exit observation at or after ten seconds from the observer's
  pre-forward tick. `runtime.ps1.txt:167-210` cancels and settles observers before
  taking the final forwarding tick, preserves the remaining full normal-stop
  allowance, and refuses resumed-child kills if the observer does not settle.
  Resumed-child recovery checks retained ownership and saves a reason, kill tick,
  signal, and exit code. Forced recovery cannot establish normal acceptance.
  The stop-entry 40-second reserve is scheduling allowance for the existing
  15/10/15-second budgets; it does not change the stop contract.
- **Binary reuse, transport, and collection.** `controller.py.txt:121-171` and
  `qualify.ps1.txt:4-26` bind the exact retained executable, immutable build
  receipt, source archive/manifest, original state metadata, and transfer bytes.
  No rebuild, registry copy, or old-guard reset is proposed. The fresh protected
  root and selected before/after snapshots remain required. The postflight
  generation/pipe audit is at `postflight.ps1.txt:14-24`. Collection checks the
  downloaded archive, safe unique members, evidence hashes, executable, and
  immutable receipt before the exact fresh-root removal
  (`controller.py.txt:173-201`). The single 2,400-second outer budget reserves
  30 seconds for one nonrecursive transport recovery
  (`controller.py.txt:43-45,58-112`); the runtime outer wait is 150+30 seconds
  (`controller.py.txt:235-242`). NEXT-RUN.md preserves one full CLI flow and the
  existing authorization. It adds no human-permission gate.

**Current prerequisites are absent; retained-inventory.json is historical.**
File-only checks used exactly the paths bound by the candidate. No directory
search, recovery, substitution, rebuild, or remote lookup was used. The original
private-location receipt exists, but every named retained input below is absent
at its bound location. The private root prefix is intentionally omitted:

- `downloaded/target/x86_64-pc-windows-msvc/release/openergy.exe`
- `evidence.tar`
- `source.tar`
- `source-manifest.json`
- `state.json`
- `downloaded/build-result.json`

The exact `/tmp/oe-stop-c8a3489c-transport-selection.json` is also absent. The
original published source-manifest.json, artifacts.json, and both private-location
receipt files checked by the controller still exist. Their existence does not
establish that any private payload remains. No transport value or private root
was printed. These missing inputs block the proposed execution path. Their
absence is an execution prerequisite failure, not an additional P1/P2 source
finding. The candidate's historical availability statements are not current
retention evidence. This review does not authorize restoration or substitution.

The original README and runtime-result.json were read at evidence commit
`6010722042604f77842db3ce491bee42f52a1265`. They preserve the one active-log
File.ReadAllText sharing failure before healthy status/config, invalid method,
or daemon.stop. Version 0 and absent-status 1 were historical successes. There is
still no stop acknowledgement, normal exit code, normal-stop timing, or running
Off network inventory from that attempt. Its forced-recovery exit code remains
missing. Saved cleanup and selected preservation receipts are inherited evidence,
not independently reproduced host state. The recorded Clippy/build results,
retained artifact binding, and unsampled release-linker limitation are unchanged.

PowerShell/C# syntax and behavior remain UNVERIFIED until a future real native
whole flow. Uncapped inherited stdout/stderr sinks, absent global SSH-job/handle
audits, the unsampled release linker, and unproven total resource bounds remain
disclosed limits. This review does not require hard preemption of arbitrary OS
calls or perfect kernel identity guarantees. No executable receipt or broader
source/runtime/security/release gate is supplied.

Only this review document was authored. No supplied program/helper was executed
or imported. No PowerShell/C# parser or compiler, preflight, product, test, mock,
simulation, remote command, process/pipe/port/OS query, signing, delegation,
integration, push, or publication ran. No candidate, product, original evidence,
guard, installed/shared state, or stopped gate was changed. The separate closed
Sparkle attempt was not used as authority and was not resumed.
