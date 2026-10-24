# Windows missing-PID observation repair R9

Source-only implementation response to R8-P2-1 in candidate
`1d01d672c93b03d089d6d902f5bc7f15c10ffd05`. Fresh independent review of
this exact seven-source candidate is required. Runtime remains UNVERIFIED.

Only the PID lookup expression in `source/build.ps1.txt` changes. It now calls
`[Diagnostics.Process]::GetProcessById([int]$p.ProcessId)` directly. The existing
`catch [ArgumentException]`, bounded race record and continue remain unchanged.
Other errors still propagate to the existing failure/settlement lifecycle.

The [R8 review](../windows-adapter-p2-review-2026-10-24-81ddd0c1/REVIEW.md)
shows why the former Get-Process cmdlet translated a missing-PID exception into
a different error type. The direct local overload documents ArgumentException
for a process that is no longer running. [Microsoft API contract](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.process.getprocessbyid?view=net-9.0).
The [official PowerShell source](https://github.com/PowerShell/PowerShell/blob/v7.5.4/src/Microsoft.PowerShell.Commands.Management/commands/management/Process.cs#L202-L228)
confirms the old cmdlet conversion path. No process lookup was executed here.
This is the alternative correction explicitly offered by the independent review.

The returned Process object still goes through Build-Owner, retained-handle,
creation-time, executable, command-line, parent and priority/session checks.
A missing lookup creates no owner. No check after the lookup, signal authority,
lookup retry, compiler retry, fallback error suppression or deadline changes.

All six other source texts are exact R7 bytes. The R6 TEMP containment and full
compiler lifecycle repairs remain unchanged. The R2 log-sharing reader, real
CLI/wire/stop path and the complete inherited budget/one-shot contract remain
unchanged. The full inherited plan is
[the R7 next-run plan](../windows-adapter-p2-repair-2026-10-24-01d1bcb5/NEXT-RUN.md).
The new [proposal](proposal.json) binds these exact source bytes; it is not an
execution or approval receipt.

The [patch](repair-from-r7.patch) is a single changed line in one hunk.
[Input bindings](input-bindings.json) bind all prior R7 files and the full R8
review to immutable Git bytes. [Static data checks](static-checks.json) record
exact source comparison, patch materialization, Python-controller AST parsing,
and preservation of all 178 original evidence files. No PowerShell/C# parser,
compiler, code import, test, mock or simulation ran. The manifest binds all
package files except itself; Git binds the manifest.

The original native result remains FAILED / UNVERIFIED. This task establishes
no native syntax, build, missing-PID race, ownership settlement, normal-stop or
release acceptance. Unknown or unsampled generations, arbitrary OS-call timing,
and uncapped external resources remain the previously stated proof limits.
Exact fixture-path facts and qualified current baselines are still prerequisites.
Historical executable/raw receipt/state/archive bytes were not reconstructed.

Only this new folder was written. Production source, all prior evidence and
spent guards remain unchanged. No product, build, signing, fixture preparation,
preflight, process/OS/port/job query, native/remote command, desktop operation,
credential/key/production-seed access, push or publication occurred. External
reads were limited to the two primary source/API pages linked above. Live
sharing, installed apps, production config/TLS/default IPC, TCC, login/services/
jobs/accounts, firewall, global DNS and toolchain defaults remain unchanged.
No TCP24800/UDP5353 or Docker action occurred. The stopped experiments and
isolated Swift build gate remain stopped. The expired Sparkle one-shot is closed.
