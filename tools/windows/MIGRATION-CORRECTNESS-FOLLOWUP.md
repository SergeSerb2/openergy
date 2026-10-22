# Migration correctness follow-up

Date: 2026-10-22 (America/Vancouver).
Implementation base: `1af5b10ae5bbd2856679f2ded0fd60338a621511`.
Parent integrated it as `71e6f74f7327f0552d84b6ff20e9c9e0ddd56f71`.
The owned Windows files were compared against parent
`5fc17800043eb8a8b6d8346774358ea26ba8bf7e` and were identical before these changes.
This patch does not change Rust, release tooling, or performance charts.

The [original privilege-boundary report](MIGRATION-PRIVILEGE-BOUNDARY.md) remains
historical evidence. Its negative checks did not establish successful limited
migration. Fresh independent source review found these four defects:

| Finding | Code response |
| --- | --- |
| P1: The InOut client constructor requests GENERIC_WRITE, including pipe-instance creation, while the user's pipe ACL grants ReadWrite. A filtered user cannot open the broker. | Open with CreateFileW using explicit `ReadWrite | Synchronize` (`0x12019b`), without GENERIC_WRITE or FILE_CREATE_PIPE_INSTANCE. Wrap its non-inheritable overlapped handle with the connected NamedPipeClientStream handle constructor. Keep Identification SQOS, the 90-second connect deadline, bounded asynchronous I/O, and kernel PID/start/image/token checks. The helper ACL is unchanged. |
| P2: Capturing RegistryBinary through an if-expression enumerates byte[] into Object[], and the type check rejects valid saved values. | Assign the registry return directly inside the branch. Do the same for FromBase64String in Read-State. Explicit null checks preserve an absent value separately from an empty byte array. Save-State uses `-` only for null and a blank Base64 line for an empty array; restore writes empty binary rather than deleting it. |
| P2: Clearing cutover before sending STARTED skips automatic recovery if that reply fails or times out. | Keep cutover armed while writing STARTED and reading STARTED_ACK from the bound controller. Failed writes, caller loss, or missing acknowledgements enter service recovery and retain saved state. Acknowledged Start also retains state for later Commit/Rollback. Existing UI_RESTORED acknowledgement ordering remains. |
| P2: Restore-Service starts a previously running service but never stops a service saved as stopped. A reboot can leave it running at rollback. | Refresh service status and restore both booleans. For saved stopped state, stop unless already stopped/pending stop, then wait for Stopped before restoration can proceed. Failures keep the saved record. |

The Windows pipe access mapping is documented in [Microsoft's named-pipe security
guide](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)
and [.NET reference source](https://github.com/microsoft/referencesource/blob/main/System.Core/System/IO/Pipes/Pipe.cs#L944).
PowerShell collection enumeration is described in
[about_Return](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_return#return-values-and-the-pipeline).

## Exact source and isolation

A new disposable source directory received exactly ten top-level `.ps1`/`.cmd`
files. Windows hashes matched the local manifest before and after execution.
Local files also matched after the run. Prior sources, binary, report, and
fixtures were not overwritten. Archive SHA-256:
`0d6fcf6f78a556a39e72c32d255935e8c07fa2b67dc4b63a6aebad7a8dd2698d`.

| File under tools/windows | SHA-256 |
| --- | --- |
| `openergy-migration-admin.ps1` | `300d7e839ad542dae7c8569256c6be0049516a1b659a90e269da115935f20dfe` |
| `openergy-run.cmd` | `74772411a85032212c7f4ba1be1948233804dcb21166c5caef0a6611ea2c3210` |
| `openergy-run.ps1` | `607ae78154329b10e4c2a6557760e80155ef8988669c2a0c7b0742586fede8c5` |
| `openergy.ps1` | `aa1dac2eb83426a2f981353ce8adba3f7bbe7c97f4b5363b5aeb1883ac218214` |
| `switch-from-synergy.ps1` | `ad5a3607c2033b43fd6d1e3087c5554c172078898bda76dcb0300bd1518d8721` |
| `verify-control.ps1` | `264d6a71e54de650498f6dcf46b806acaf5cc20f2022222082372cf614ab8aeb` |
| `verify-lifecycle-ownership.ps1` | `7510ff5a7a79ac95292cfa87cc2648b4c9748d4ca1754086a221e49af064c4c1` |
| `verify-lifecycle.ps1` | `6e6478b500921c354d5f579c5756cfe816ea399eef6640523ab633ed25789835` |
| `verify-migration-boundary.ps1` | `5ff92857b4fa8e129b6c1a9b85b1d3a89fc9c3b9921e9dab25172fb4815acfef` |
| `verify-pipe-identity.ps1` | `a9a81cc2624ec179a275a66a30090d1ad3e57e4d2bb6dd7bd78e637e944458fb` |

## Available verification

The unchanged external `verify-migration-boundary.ps1` driver was run against the
new scripts with PowerShell 7.6.4 at BelowNormal priority. Its external children
used Windows PowerShell 5.1.26100.9750 and PowerShell 7.6.4 with NoProfile.
No unit tests, mocks, extracted product functions, fake service interfaces, or
trust bypasses were added.

```powershell
pwsh -NoProfile -NonInteractive -File .\verify-migration-boundary.ps1 -ScriptsDir . -BinaryPath <private-path>\openergy.exe -Port 26530
```

Result: exit `0`, `WINDOWS_MIGRATION_BOUNDARY_NEGATIVE_PASS`.
The driver again proved elevated replacement-manager rejection for all five
controller actions, elevated manager/supervisor rejection, protected helper
read-only Check, owner/ACL/DELETE_CHILD/junction rejection, no user-module load
in Check, three actual Network-SID pipe denials, and occupied nonce pipe refusal
before a reply. No recovery record was created. Protected fixtures were removed
after owner, hash, and inventory checks. The post-run source inventory passed.

Read-only framework inventory in separate 5.1 and 7 processes confirmed:

```text
5.1.26100.9750 and 7.6.4:
Void .ctor(System.IO.Pipes.PipeDirection, Boolean, Boolean, Microsoft.Win32.SafeHandles.SafePipeHandle)
ReadWrite+Synchronize=12019b
Boolean SetLastError
Void .ctor(System.Reflection.ConstructorInfo, System.Object[], System.Reflection.FieldInfo[], System.Object[])
```

This confirms API presence and the requested rights mask. It does not prove a
successful controller connection or execution of the new client-open path.
Production entry guards were not bypassed to reach it.

Supplied binary SHA-256 remained
`6a9c83fd4902fd943ea1e184d49be266728e8bb004be58d566967c4e8a7a0b76`.
This is the prior pinned round5 binary, as documented in
[ROUND5-1ef54fc.md](ROUND5-1ef54fc.md). No new Rust build or new binary acceptance
is claimed; the limited lifecycle branch did not reach binary startup.

## Validation limits

Both actual restricted medium-token PowerShell children still exited
`0xc0000142` before script code in SSH session 0. The SSH token also has Network
SID, which the unchanged broker ACL denies. These remain external acceptance
gates. No window-station/desktop ACL, account, credentials, or token policy was
changed to force a successful flow.

Consequently the new limited client connection, registry binary capture and
round trips (including empty binary), STARTED reply failure/acknowledgement
recovery after real cutover, and restoration of a saved stopped service were
reviewed in source, not executed. There was no service/task mutation, live
registry/config/TLS or installed-state change, protected installation, native
input/clipboard use, desktop action, or port 24800. The driver uses only private
26530 when a limited child can start; that branch remained unavailable here.

The full interactive limited-token migration, positive private IPC, service/UI
recovery, and task acceptance gates from the original report remain open. The
available negative flows are green; they are not positive proof of these four
corrected branches. `git diff --check` passed, and tracked cmd/bat index blobs
remain LF.

## Sanitized final command output

```text
PASS ps51 full script parsing
PASS ps51 elevated Install rejects before replacement manager execution
PASS ps51 elevated Start rejects before replacement manager execution
PASS ps51 elevated Commit rejects before replacement manager execution
PASS ps51 elevated Rollback rejects before replacement manager execution
PASS ps51 elevated CheckBroker rejects before replacement manager execution
PASS ps51 manager refuses elevated token
PASS ps51 supervisor refuses elevated token
PASS ps51 protected admin-owned helper and ancestor ACL check
LIMIT ps51 restricted medium process exits 0xc0000142 before user code in SSH session 0; no desktop ACL changed
PASS ps7 full script parsing
PASS ps7 elevated Install rejects before replacement manager execution
PASS ps7 elevated Start rejects before replacement manager execution
PASS ps7 elevated Commit rejects before replacement manager execution
PASS ps7 elevated Rollback rejects before replacement manager execution
PASS ps7 elevated CheckBroker rejects before replacement manager execution
PASS ps7 manager refuses elevated token
PASS ps7 supervisor refuses elevated token
PASS ps7 protected admin-owned helper and ancestor ACL check
LIMIT ps7 restricted medium process exits 0xc0000142 before user code in SSH session 0; no desktop ACL changed
PASS actual kernel migration pipe denies the SSH Network SID
PASS actual kernel migration pipe denies the SSH Network SID
PASS actual kernel migration pipe denies the SSH Network SID
PASS actual broker refuses an occupied nonce pipe before sending any reply
PASS actual kernel ACL rejects same-user ownership despite protected DACL
PASS actual kernel ACL rejects same-user ownership despite protected DACL
PASS actual kernel ACL rejects writable protected-owner candidate
PASS actual kernel ACL rejects writable protected-owner candidate
PASS actual kernel ACL rejects parent DELETE_CHILD replacement rights
PASS actual kernel ACL rejects parent DELETE_CHILD replacement rights
PASS actual kernel path rejects junction ancestor
PASS actual kernel path rejects junction ancestor
PASS protected read-only command does not autoload user modules
PASS protected read-only command does not autoload user modules
PASS read-only checks and rejected peers create no recovery state
PASS protected fixture code remains hash-identical
PASS validated binary remains unchanged
WINDOWS_MIGRATION_BOUNDARY_NEGATIVE_PASS
MEDIUM_INTERACTIVE_ACCEPTANCE_REMAINS_REQUIRED
OWNED_PROTECTED_FIXTURE_HASH_CHECKED_CLEANUP_PASS
POST_VERIFICATION_TEN_FILE_INVENTORY_PASS
OWNED_PROTECTED_FIXTURES_REMOVED
```
