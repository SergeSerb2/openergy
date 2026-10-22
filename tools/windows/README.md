# Windows install and lifecycle

Windows currently has a daemon and CLI, with no native settings app. Desktop
input and native clipboard acceptance remain pending. A successful virtual test
or session-0 build does not establish interactive desktop readiness.

## Fresh install

Build with the Rust MSVC toolchain. Keep the scripts in this directory together.
No Synergy installation or administrator account is required for a per-user install.
Run these commands in a **non-administrator** PowerShell. Per-user entry points
refuse administrator, SYSTEM, and high-integrity tokens. The command uses a
process-only execution-policy option; it does not change the
user or machine policy.

```powershell
cargo build --release --locked --bin openergy
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\tools\windows\openergy.ps1 Install -BinaryPath .\target\release\openergy.exe
$Install = "$env:LOCALAPPDATA\Programs\Openergy"
$Data = "$env:APPDATA\Openergy"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$Install\openergy.ps1" Start
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$Install\openergy.ps1" Status
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$Install\openergy.ps1" Stop
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$Install\openergy.ps1" Uninstall
```

Install copies the binary and supervisor, creates a config only if it is absent,
and registers a **disabled**, limited-rights logon task for the current user.
Start enables that task and waits for real daemon control readiness. Stop disables
the task and requests a clean daemon stop. Uninstall preserves all config, TLS
identity, trust, and logs. Stop and uninstall never kill processes by name.
Reinstall into the same paths after uninstall to use the preserved configuration.
An existing unmanaged installation directory is refused.

A new configuration uses the computer's name, role `off`, TLS enabled, and
`127.0.0.1:24801`. Edit `config.toml` while stopped to choose `server` or `client`,
set the server's private LAN address or client's `remote = "host:24801"`, and
configure the screen layout. A host must explicitly change `server.listen` to its
chosen LAN address. No firewall rule is added. Exchange and approve fingerprints
before connecting:

```powershell
& "$Install\openergy.exe" fingerprint --config "$Data\config.toml" --synergy
& "$Install\openergy.exe" trust approve 'v2:sha256:REPLACE_WITH_VERIFIED_FINGERPRINT' --config "$Data\config.toml"
```

The supervisor does not set `OPENERGY_ALLOW_PORT_24800`. It passes the caller's
value through unchanged, including an explicit `0` denial. The daemon controls
the standard port 24800 policy; new-install configs use 24801.
Use the same `InstallDir` and `DataDir` arguments on every lifecycle command if
using custom paths. Paths with spaces are supported. A marker checks the user
and selected paths before stop or uninstall. Readiness and control requests also
check the actual kernel pipe server against the supervised child PID and process
start time. Start refuses an occupied endpoint. `Validate` checks an installation
without starting or stopping it.

`-NoStartup` installs without a task. Start then launches the same supervisor as
an owned background process. This also supports confined tests with
`Install -NoStartup -Platform virtual -Port <lab-port>` and explicit temp paths.
The supervisor passes config, socket, and parent PID arguments, holds an
exclusive lifetime lock, stops restarting after a clean exit, and stops after
five rapid failures. Inspect `daemon.stderr.log`, `runner.stderr.log`, and
`control.stderr.log` in DataDir after a failure.

`openergy-run.cmd` is a compatibility entry point. It now **requires the data
directory as its first argument**. The optional second argument is `windows` or
`virtual`. It forwards to the PowerShell supervisor. It does not use a cmd crash
loop, conhost's version-specific headless switch, or default daemon paths.
Existing manually registered tasks must be recreated with the new lifecycle
script. Do not overwrite an active live installation to migrate task formats.

## Optional Synergy migration

`switch-from-synergy.ps1` runs in a **non-administrator** PowerShell, including
`-Install`, `-Start`, `-Commit`, and `-Rollback`. Never elevate this wrapper or
an installed `openergy.ps1`. The manager and supervisor keep the limited user
token. Existing lifecycle PID, start-time, and pipe ownership checks still apply.

Automatic service migration requires a separate, reviewed helper provisioned at
`%ProgramFiles%\OpenergyMigration\openergy-migration-admin.ps1`. See the
[protected helper setup](MIGRATION-SECURITY.md) before using migration. Ordinary
per-user installation does not require this helper. All required payloads are
top-level `.ps1` files in this directory and the Windows release archive.

```powershell
# Run from a normal PowerShell after protected helper provisioning.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\switch-from-synergy.ps1 -CheckBroker
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\switch-from-synergy.ps1 -Start
# After checking the Openergy connection:
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\switch-from-synergy.ps1 -Commit
# Or restore the saved Synergy state:
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\switch-from-synergy.ps1 -Rollback
```

Use the same explicit `InstallDir` and `DataDir` arguments for custom installs.
Start validates the full per-user installation before requesting elevation. Only
the protected helper receives the elevated token. It owns the fixed Synergy
service and machine startup changes. No task is created or run by migration.
Commit requires a pending migration and a ready daemon. Rollback restores the
saved service mode, delayed-start value, startup approval, and running state.
The limited controller restores the prior UI and confirms its limited process.

Recovery state stays protected until UI restoration is acknowledged. UI startup
failure, a failed Openergy stop, or an interrupted exchange leaves retryable
state. Caller loss after cutover triggers service restoration independently;
the helper cannot restart a limited UI without its caller. Run `-Rollback` again
from a normal PowerShell. Keep the protected state until recovery completes.
Legacy writable `DataDir\synergy-state.json` needs administrator review and
manual recovery; it is never accepted as privileged state.

The service, startup, and interactive UI transitions remain unverified in this
audit. Read the [security evidence and limits](MIGRATION-PRIVILEGE-BOUNDARY.md).

## Safe verification

The identity/control drivers require PowerShell 7 and an already-built Windows
binary. Run the lifecycle drivers from a non-administrator terminal. The lifecycle
driver also runs under Windows PowerShell 5.1.
They use real binaries and kernel pipes. They create only private temporary
state and virtual daemons. They do not register tasks or alter Synergy.

```powershell
pwsh -File tools/windows/verify-pipe-identity.ps1 -Exe target/release/openergy.exe
pwsh -File tools/windows/verify-control.ps1 -BinaryPath target/release/openergy.exe -Port 25241
pwsh -File tools/windows/verify-lifecycle.ps1 -BinaryPath target/release/openergy.exe -Port 25240
```

Run the control driver through session-0 SSH. Port arguments are restricted to
25240–25259. No driver uses port 24800 or a native platform. Successful driver
labs are removed. Failures retain control/lifecycle artifacts for inspection.

See [verification](VERIFICATION.md) for results and remaining release gates, and
[lifecycle ownership](LIFECYCLE-OWNERSHIP.md) for the occupied-endpoint and migration
preflight follow-up.

`verify-migration-boundary.ps1` is a separate administrator PowerShell 7 driver.
It uses only disposable ACL fixtures, read-only helper commands, rejected
privileged entry points, and private virtual lifecycle commands when a real
limited child can start. It never mutates a service or task. Its private port
range is 26520–26539. It reports missing interactive checks explicitly.

```powershell
pwsh -NoProfile -File tools/windows/verify-migration-boundary.ps1 -BinaryPath target/release/openergy.exe -Port 26530
```
