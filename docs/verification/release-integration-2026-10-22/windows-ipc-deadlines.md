# Windows pipe request deadlines — 2026-10-22

## Scope and result

The [shared request-deadline fix](ipc-read-deadlines.md) previously had real Unix
socket evidence. This follow-up verifies the Windows named-pipe path against
the frozen native MSVC daemon from the
[current Windows transport record](windows-transport-current.md).

The new [real pipe/CLI command](../../../tools/windows/verify-ipc-deadlines.ps1),
source `798f6867f3720ef5ceaafc168f7e6a7225dc1a16`, passed **19/19** checks.
The same script passed **13/19** against the retained pre-deadline Windows
build. Its six failures are exactly the unfinished-request expiry and recovery
cases. Both daemons exited zero and removed their own endpoints.

No runtime code changed in this follow-up. The executable hashes were checked
before and after execution:

| Windows executable | Source | SHA-256 |
| --- | --- | --- |
| Current runtime | `c2d14a695ef9b4e4bcd9bd70c29ee393fe9286e2` | `dbfb1d619a9ed04a90a17bc6509d1419fc2402aad5ebb05464fe2bc3297437fb` |
| Retained pre-deadline control | `1ef54fcf8e866a5799ba38a42674ddeff8452b5c` | `9b68bd675b25210790292cca365671e96b1ef0368f53bf43e05057f7cc70f4d9` |

The older control predates several other hardening changes. It is evidence of
the held-client recovery failure, not a native build differing only by commit
`89b79e1`. Its original executable was only read and copied; it remained unchanged.

## Actual pipe behavior

One client first completes a real request, then stays idle. The other 63 client
slots receive no bytes, a partial prefix, a partial body, or an incomplete body
that gets another byte every 600 ms. Each client starts a real asynchronous
pipe read before the observation window. The test never disposes these client
handles until after checking expiry and attempting public CLI recovery.

| Unfinished request | Current closures | Observed closure after connection, ms | Pre-deadline closures |
| --- | ---: | ---: | ---: |
| No bytes | 16/16 | 10,026–10,068 | 0/16 |
| Partial prefix | 16/16 | 10,030–10,077 | 0/16 |
| Partial body | 16/16 | 10,023–10,065 | 0/16 |
| Continued drips | 15/15 | 10,028–10,080 | 0/15 |

All 63 clients were initially admitted. The test observes each closure in a
declared 9–12.5 second window; the incomplete cohort runs for 11.5 seconds.
The ongoing drips do not restart the server's absolute ten-second deadline.
After expiry, a new real `ctl status.get` succeeds while all client handles
remain open. The completed-request client also sends and receives another
request on its original long-idle connection.

On the older build, all four unfinished groups stayed open, new CLI access
failed until a client was released, and a partial frame after a completed
request did not expire. The remaining 13 controls passed, including completed
idle reuse, fragmentation, ordered pipelining, unknown-method recovery,
oversized-prefix refusal, later valid requests and shutdown with a partial
client. These controls also passed on the current build.

The checks use the actual release executable, CLI, kernel named pipes and
shared request handler. No unit tests, mocks or private function shortcuts
were added.

## Reproduce

Run from session-0 SSH with PowerShell 7 and an already-built native executable:

```powershell
$Report = Join-Path $env:TEMP ('oe-ipc-deadlines-'+[guid]::NewGuid().ToString('N')+'.json')
pwsh -NoProfile -File tools/windows/verify-ipc-deadlines.ps1 -BinaryPath target/release/openergy.exe -ReportPath $Report
```

The driver creates a fresh private fixture, a role-off virtual daemon and a
unique named pipe. Existing report and lab paths are refused. It uses a private
working directory and explicit config/TLS/control/app-data selectors. It does
not open a TCP listener. Fallback process cleanup verifies the executable path,
start time and frozen binary hash before terminating only an owned process.
The report and raw daemon logs are retained for inspection.

Full-file PowerShell parsing and actual execution passed. No Rust source,
manifest, lockfile or dependency changed, so this follow-up reused the already
verified native build. The original
[Windows build/transport evidence](windows-transport-current.md) and
[Mac full-suite evidence](receive-fairness.md) retain their own artifact scope;
neither is relabeled as a new full-suite run here.

## Initial wrapper failure and final limits

The first wrapper run failed before its initial CLI check because the helper
name `Cli` resolved to PowerShell's built-in `Clear-Item` alias. The helper now
has a distinct name and the process works inside the private fixture. The
original failed report and script are retained. Read-only checks confirmed
that neither attempted filename existed in the original SSH working directory,
and that the owned daemon was cleaned up. This is a wrapper failure, not a
failed product deadline result.

The final native audit found zero owned processes. Current, copied baseline,
original retained baseline and script hashes matched their expected values.
Exact reports, closure timings and hashes are in the
[machine record](windows-ipc-deadlines.json).

All work used the virtual backend in Windows SSH session 0. Installed sharing,
native input/clipboard, services, tasks, startup, accounts, credentials,
desktop/windowstation settings and port 24800 were preserved. This does not
establish desktop or positive Windows migration/recovery acceptance. The two
original security review clearances and trusted distribution remain open.
GitHub private reporting setup remains deferred.
