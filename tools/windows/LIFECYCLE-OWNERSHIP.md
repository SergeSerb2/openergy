# Windows lifecycle ownership follow-up — 2026-10-22

Two review findings were fixed in the scripts, then exercised against the exact
Windows binary built from integrated source `b36e840ea40b0c455db41afa6617d376b3b31898`
with Rust 1.99.1. This patch changes no Rust/native input code.

## Fixes

1. The supervisor publishes each child's PID and UTC process start ticks along
   with its own identity. Start refuses an already occupied control endpoint.
   Readiness and control requests require a live child with the expected
   executable and start time. The request uses one connected kernel pipe and
   checks `GetNamedPipeServerProcessId` before sending bytes. Checking a status
   CLI and then opening another connection for Stop would leave a replacement
   race. The pipe client uses identification-only impersonation. Stop never
   sends a request when a supervised child cannot be verified.
2. The manager exposes a read-only `Validate` action that checks owner, paths,
   required files, task ownership when relevant, and endpoint ownership.
   Migration Start calls it before looking up or changing Synergy's service.
   Automatic and explicit rollback now attempt Synergy restoration independently
   of Openergy stop failure. Failed stop/restoration retains the saved recovery
   state. No service recovery branch ran in this audit.

## Real verification

No unit tests or mocks were added. The regression driver creates its own temp
install using `-NoStartup -Platform virtual`, plus a separately started owned
virtual daemon on the selected pipe. The original integrated scripts reproduced
`OCCUPIED_START_REJECTED=False`: Start accepted the independent daemon's status.
The patched driver passes:

- occupied endpoint refused; independent daemon still serves real status after
  managed Stop, and exits only after its explicit test-owned stop;
- stale child start ticks rejected;
- a different live test daemon's valid PID/start identity cannot authenticate
  the connected pipe belonging to the real supervised daemon;
- invalid installation user SID and data path rejected by migration Start,
  before any cutover state or service operation;
- Rollback with no saved migration state refused.

Final marker: `WINDOWS_LIFECYCLE_OWNERSHIP_PASS`. The full existing lifecycle
flow also passed with these patched scripts under PowerShell 7 and Windows
PowerShell 5.1: install, Start, duplicate Start, config patch, loopback listening,
crash restart, clean/repeated Stop, restart, uninstall/preservation, and unmanaged
path refusal. Final marker: `WINDOWS_LIFECYCLE_PASS`.

Commands (the patched scripts are copied to a separate driver directory, never
over the exact integrated source archive):

```powershell
pwsh -File "$PatchedScripts\verify-lifecycle-ownership.ps1" -BinaryPath "$Source\target\release\openergy.exe" -ScriptsDir $PatchedScripts
pwsh -File "$PatchedScripts\verify-lifecycle.ps1" -BinaryPath "$Source\target\release\openergy.exe" -Port 25240
powershell.exe -NoProfile -NonInteractive -File "$PatchedScripts\verify-lifecycle.ps1" -BinaryPath "$Source\target\release\openergy.exe" -Port 25242
```

Only temporary configurations, installations and virtual daemons were used.
Ports were within 25240–25259. No task registration, live installation,
Synergy/service change, native clipboard/input, or port 24800 operation ran.
Actual service restoration, scheduled-task and interactive native acceptance
remain unverified. The integrated strict-Clippy failure is recorded separately;
it is not treated as a pass by these runtime checks.
