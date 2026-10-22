# Windows migration privilege boundary

Date: 2026-10-22 (America/Vancouver). Base:
`2b902d9921d3dd5c93e997edb0b0335594fcde44`. This is a script-only follow-up.

Finding: [PR2 security review](https://github.com/SergeSerb2/openergy/pull/2#discussion_r4346064391).
The elevated migration wrapper could execute a same-user replacement of the
per-user manager from LocalAppData before installation checks.

## Change and trust decision

The controller now refuses elevated execution before loading the manager. The
manager and supervisor also require limited tokens. A separate protected helper
handles only fixed Synergy service/startup operations. It never executes the
per-user manager, saved task actions, or code from per-user state. Start retains
the full installation preflight. Rollback restores the service independently of
an Openergy stop failure. Existing child/pipe identity checks remain intact.

The helper requires admin/SYSTEM ownership, restrictive ACLs, and protected
ancestors. Same-user ownership alone fails. Reparse points and replacement
rights fail. Routine elevation uses the fixed System32 Windows PowerShell host,
`-NoProfile`, a clean launch environment, no module autoload, no compilation or
temporary DLL, and checked protected system dependencies. All new payloads are
top-level `.ps1` files; no release packaging change is needed.

The private pipe binds both real kernel peer PIDs to creation time, image, SID,
and token. It denies Network SID and remote clients and uses the first-instance
flag. Identification does not grant server impersonation.

Recovery state now waits for the limited UI acknowledgement. A limited,
same-user UI process must exist before the controller acknowledges. A privileged
service process cannot satisfy that check. UI failure, caller loss, or stop
failure leaves retryable state. Caller-loss automatic recovery can restore the
service only; it cannot start the limited UI alone. No full-restoration claim is
made for that path.

One administrator must independently review/pin and provision helper bytes.
[Setup instructions](MIGRATION-SECURITY.md) give a concrete protected copy path:
read once as data, compare an independently trusted hash, then write those same
bytes with an admin owner and protected ACL at creation. No writable elevated
bootstrap is supplied. Ordinary per-user install remains nonadmin. Legacy
writable migration records require human review/recovery, not automatic trust.

## Exact copied source

The disposable archive contained exactly ten top-level script/command files.
Local hashes were checked against the extracted Windows inventory before use.
The same ten hashes matched after verification; no Windows backwrite occurred.
The base checkout, Cargo.lock, prior archived sources, and earlier reports were
not modified. Archive SHA-256:
`725f05dea2386b44f94e5419e7be97db06ec326e0db5aee8a1fc6834df0af1fc`.

| File under `tools/windows/` | SHA-256 |
| --- | --- |
| `openergy-migration-admin.ps1` | `08fdd4b05d54d26009b81f2e5184a9afe36dd9874f410e9bafb65c472449892e` |
| `openergy-run.cmd` | `74772411a85032212c7f4ba1be1948233804dcb21166c5caef0a6611ea2c3210` |
| `openergy-run.ps1` | `607ae78154329b10e4c2a6557760e80155ef8988669c2a0c7b0742586fede8c5` |
| `openergy.ps1` | `aa1dac2eb83426a2f981353ce8adba3f7bbe7c97f4b5363b5aeb1883ac218214` |
| `switch-from-synergy.ps1` | `035a7b41f34af1a94697129d145e4484a7734edf630791d0a3eb4e8b361c0eb2` |
| `verify-control.ps1` | `264d6a71e54de650498f6dcf46b806acaf5cc20f2022222082372cf614ab8aeb` |
| `verify-lifecycle-ownership.ps1` | `7510ff5a7a79ac95292cfa87cc2648b4c9748d4ca1754086a221e49af064c4c1` |
| `verify-lifecycle.ps1` | `6e6478b500921c354d5f579c5756cfe816ea399eef6640523ab633ed25789835` |
| `verify-migration-boundary.ps1` | `5ff92857b4fa8e129b6c1a9b85b1d3a89fc9c3b9921e9dab25172fb4815acfef` |
| `verify-pipe-identity.ps1` | `a9a81cc2624ec179a275a66a30090d1ad3e57e4d2bb6dd7bd78e637e944458fb` |

## Real command verification

Windows PowerShell `5.1.26100.9750` and PowerShell `7.6.4`. External child
processes used `-NoProfile -NonInteractive -File`. The driver ran BelowNormal.
It used actual Windows ownership, ACL, token, junction, and named-pipe APIs;
no unit tests, mocks, or isolated function checks were added.

Replay with an already-built Windows binary and a disposable source copy:

```powershell
pwsh -NoProfile -NonInteractive -File .\verify-migration-boundary.ps1 -ScriptsDir . -BinaryPath <private-path>\openergy.exe -Port 26530
```

| Flow | Result |
| --- | --- |
| All copied PowerShell scripts parsed in separate 5.1 and 7 processes | Pass |
| Elevated wrapper Install/Start/Commit/Rollback/CheckBroker with an owned replacement manager | Rejected before replacement execution in both engines; canary absent |
| Elevated manager and supervisor | Rejected in both engines |
| Read-only Check from an admin-owned protected disposable helper | Pass in both engines |
| Same-user owner despite restrictive DACL | Rejected in both engines |
| Protected owner with user write rights | Rejected in both engines |
| Parent DELETE_CHILD replacement rights | Rejected in both engines |
| Junction ancestor | Rejected in both engines |
| User module path containing hostile Management/ScheduledTasks modules | Read-only Check passed in both engines; module canary absent |
| Actual helper Probe pipe, with the SSH Network-SID token | Kernel access denied, three attempts |
| Actual separate server occupies the chosen nonce pipe | Helper refused first-instance creation before any reply |
| No protected recovery state from read-only/rejected flows | Pass |
| Protected fixture code and supplied binary unchanged | Pass |
| Owned protected fixture cleanup | Pass after owner/hash/full-inventory checks |

Final driver exit: `0`. Markers:
`WINDOWS_MIGRATION_BOUNDARY_NEGATIVE_PASS`,
`MEDIUM_INTERACTIVE_ACCEPTANCE_REMAINS_REQUIRED`, and
`OWNED_PROTECTED_FIXTURE_HASH_CHECKED_CLEANUP_PASS`. Six earlier owned protected
fixtures were also removed after checking the lab pointers, exact helper hashes,
admin owner, no reparse points, and complete allowed file inventory. Disposable
source/log copies remain private for inspection.

Supplied virtual binary SHA-256:
`6a9c83fd4902fd943ea1e184d49be266728e8bb004be58d566967c4e8a7a0b76`.
It is the earlier pinned round5 composite binary documented in
[ROUND5-1ef54fc.md](ROUND5-1ef54fc.md), not a new build of this PR base. Its hash
was checked again after the driver. Rust was read-only checked as
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`, host
`x86_64-pc-windows-msvc`. No Rust source changed or new build gate was claimed.

## Limits and remaining acceptance

The SSH token is elevated, has the Network SID, and runs in session 0. It has
no linked UAC token (GetTokenInformation error 1312). A real restricted medium
token was created with CreateRestrictedToken and medium integrity, but both
PowerShell engines terminated with `0xc0000142` before script code ran. An
earlier CreateProcessAsUser attempt had the same result. No desktop/window
station ACL, account, credential, live process, service, or task was changed
to work around this limitation.

Thus a successful limited controller-to-helper handshake, kernel PID/start-time
mismatch checks beyond the Network-SID denial, and the new limited virtual
lifecycle flow remain unverified in this environment. The occupied kernel pipe
refusal is proven; this does not claim a successful counterfeit-response
rejection through the limited controller. The previous occupied-daemon and
marker evidence remains historical evidence, not a pass of this new guard.

Actual service stop/start/start-mode changes, machine startup writes, protected
Program Files provisioning, logon task registration/control, limited Synergy UI
restoration, and recovery acknowledgement/disconnect after real cutover were
not executed. Recovery sequencing was checked in source, not simulated with
fake services or substitute protocols. Read-only Check/Probe do not reach those
operations. Full interactive migration acceptance remains required on a
separate authorized Windows test account/machine.

No installed Openergy/Synergy, live config/TLS/default pipe, native input or
clipboard, desktop, startup setting, or port 24800 was touched. The only optional
virtual TCP port in this driver is 26530, within 26520–26539; the limited child
could not start here, so that lifecycle branch did not bind it.

`git diff --check` passed. Tracked cmd/bat index blobs were checked as LF. Prior lifecycle fixes, typed clipboard changes, and
the 1ef54fc/ccf9847 native compiler/runtime evidence remain unchanged.

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
```
