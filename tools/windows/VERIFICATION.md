# Windows readiness audit — 2026-10-21

Base commit: `06b61287afd7f31aa3002d252389f2c2b17f73f5`.
Scope: `crates/platform-windows`, daemon `windows_pipe.rs`, and `tools/windows`.
No unit tests, mocks, deployment, native input, native clipboard operation,
service change, scheduled-task registration, or production config change ran.
Windows checks used an owned temporary source/build directory through SSH
session 0. Mac checks used only the virtual backend. TCP listeners used only
127.0.0.1 and assigned ports 25240–25259. The reserved lab range was 25260–25279.
Port 24800 was never bound or connected.

## Issues fixed

- **Pipe client impersonation:** the real CLI previously let a pipe server
  impersonate its token. A real local pipe endpoint observed .NET
  `TokenImpersonationLevel.Impersonation` (3) before the fix and
  `Identification` (2) afterward. `CreateFileW` now supplies explicit security
  quality-of-service flags. The client also checks the actual server process
  token's user SID before writing a request. This protects against a different
  account squatting on the public pipe name with a permissive DACL. Existing
  protected per-user server DACLs, first-instance ownership, remote rejection,
  and overlapped cancellation remain in place. Invalid Unicode selectors and
  overlong explicit pipe names fail instead of creating ambiguous aliases.
- **Clipboard own-write feedback:** the old stamp incremented when the message
  thread later processed `WM_CLIPBOARDUPDATE`. The worker could sample the old
  stamp immediately after a successful write and then broadcast its own update
  as a new local copy. The backend now reads the kernel clipboard sequence
  synchronously. There is no separate notification counter. Native text reads
  also require a UTF-16 NUL terminator.
- **DIB validation:** the old check accepted a bare bitmap header with missing
  palette or pixels. Reads and writes now check dimensions, planes, depth,
  compression combinations, palettes, bit masks, declared/pixel lengths, integer
  overflow, and V5 profile bounds before handing data to Windows. Compressed
  payloads receive structural/declared-length checks; their codecs are not decoded.
- **Native input lifecycle/layout:** stop and startup-failure cleanup unregister
  raw mouse input with the required null target. Stop resets pending capture
  state. Both logical-key and VK-to-scan translation use the foreground
  application's keyboard layout. Absolute injection sets physical-pixel DPI
  awareness on the injection thread itself.
- **Fresh install and startup:** the previous path required Synergy and a
  manually populated Program Files directory. A standalone per-user manager
  now implements Install/Start/Status/Stop/Uninstall, supports custom paths with
  spaces, preserves existing config and TLS state, starts fresh configs off,
  uses limited logon rights, and checks actual daemon control readiness.
  The supervisor has a kernel lifetime lock, exact process-start identity,
  explicit config/socket paths and parent PID, clean-stop handling, bounded rapid crash
  retries, and a stop sentinel. It does not enable port 24800 or kill by name.
- **Migration safety:** exactly one action is required. Missing saved state
  prevents Commit/Rollback from changing services. Commit requires a ready
  daemon. Start failure attempts rollback. Synergy process matching uses exact
  executable paths. Prior service/startup/UI state is saved. UI rollback uses a
  unique limited task and avoids an existing UI launcher. No generic one-shot
  task or unrelated cmd wrapper is deleted or stopped.

Native fixes above are source-reviewed and compile-checked. They have **not**
passed desktop input/clipboard acceptance in this audit.

## Commands and results

Windows used the already-installed `nightly-x86_64-pc-windows-msvc` toolchain
(`rustc 1.99.0-nightly (969b803cb 2026-08-09)`),
with `CARGO_BUILD_JOBS=2` and the SSH shell set to BelowNormal priority. No
compiler or dependency was installed or updated.

```powershell
$env:RUSTUP_TOOLCHAIN = 'nightly'
$env:CARGO_BUILD_JOBS = '2'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
cargo build --release --locked -p openergy-daemon
cargo clippy --locked -p openergy-daemon -p openergy-platform-windows --all-targets -- -D warnings
pwsh -File tools/windows/verify-pipe-identity.ps1 -Exe target/release/openergy.exe
pwsh -File tools/windows/verify-control.ps1 -BinaryPath target/release/openergy.exe -Port 25241
pwsh -File tools/windows/verify-lifecycle.ps1 -BinaryPath target/release/openergy.exe -Port 25240
powershell.exe -NoProfile -NonInteractive -File tools/windows/verify-lifecycle.ps1 -BinaryPath target/release/openergy.exe -Port 25242
```

Build and clippy passed with zero warnings. The identity driver passed with
`PIPE_CLIENT_IMPERSONATION_LEVEL=2` and `PIPE_IMPERSONATION_PASS`. Before the
fix, the same real-CLI probe failed with level 3.

The control driver passed CLI aliases, config save/apply, duplicate daemon
rejection, protected current-user-only kernel DACL inspection, event/log
subscription delivery, fragmented frames across read deadlines, typed unknown
method errors, the 64-client cap and slot recovery, clean stop with an incomplete
request, absent-daemon failure, and parent-watched shutdown. Final marker:
`WINDOWS_CONTROL_PASS`.

The lifecycle driver passed fresh install without Synergy, off-role startup,
isolated virtual config/pipe selection, duplicate Start, real config patch and
assigned-port listening, crash recovery, clean supervisor exit, repeated Stop,
preserved config on restart/uninstall, binary removal, and refusal to install or
uninstall an unmanaged directory. Final marker: `WINDOWS_LIFECYCLE_PASS`.
The lifecycle passed under both PowerShell 7.6.4 and Windows PowerShell 5.1.
All product scripts also passed both parsers. No task or service branch ran.

Mac commands:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --locked -p openergy-platform-windows \
  --target x86_64-pc-windows-msvc --all-targets -- -D warnings
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked \
  -p openergy-daemon -p openergy-e2e
# Repeat once for S3-*, S4-*, S8-*, S9-*, R2-*:
target/release/openergy-e2e run --scenario 'S3-*' \
  --port-range 25240-25259 --lab-port-range 25260-25279 --report /tmp/oe-windows-audit-s3.json
```

All five selected real virtual scenarios passed: ordered input, disconnect
release, text clipboard, large DIB transport, and daemon lifecycle. These are
common-daemon regressions, **not** native Windows clipboard or SendInput tests.
No full catalogue or third-party interop readiness claim is made here.

Final Windows executable SHA-256:
`3b319e69455254ab2441afae6c59b1b23274f3ceaa22dfdee686043beee1c6fc`.
The final run ended with `FINAL_WINDOWS_PASS`; no owned test daemon remained.
The isolated Windows source/build directory is retained for follow-up.

## Remaining gates and integration requests

- **Net owner:** in this base snapshot, Windows `restrict_key` is a no-op and
  `atomic_write` inherits the destination directory's ACL. Enforce a private
  kernel ACL on both initial temporary key creation and existing keys before
  reading them. Include permissive-parent and atomic-replacement cases. The
  installer must not be the only defence for offline CLI/custom config paths.
- **Root README owner:** replace the migration-only Program Files installation
  recipe with the standalone [Windows path](README.md). The cmd supervisor now
  requires an explicit DataDir argument; old hand-written tasks need migration.
- **Native desktop:** verify hardware capture, held-key/button transitions,
  AltGr/dead keys, foreground layout changes, mixed-DPI monitors, native
  text/HTML/DIB round trips, clipboard contention, and elevated/UIPI apps on an
  isolated interactive Windows account. This audit was explicitly forbidden
  from performing them.
- **Native failure recovery:** Windows can silently remove low-level hooks
  after timeout. Hook health/reinstall and an independent capture fail-safe are
  still absent. Source/clipboard reply waits and message-thread joins can still
  block if a native API or thread stalls. These remain release risks and need
  safe interactive failure testing; detaching a captured thread would be unsafe.
- **Scheduled tasks and migration:** parser/source review passed. Register,
  logon, logoff, service cutover, and rollback acceptance need a disposable
  Windows account/VM. They were not run against the user's live connection.
- **Pipe boundaries:** current-user DACL shape and real CLI QoS were tested.
  Another-user login, remote SMB attempts, and mixed-integrity tokens were not.

Microsoft references: [pipe impersonation control](https://learn.microsoft.com/en-us/windows/win32/ipc/impersonating-a-named-pipe-client),
[pipe access rights](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights),
and [raw-input unregister contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerrawinputdevices).
