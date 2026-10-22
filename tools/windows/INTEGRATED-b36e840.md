# Integrated Windows validation — 2026-10-22

Exact source: **`b36e840ea40b0c455db41afa6617d376b3b31898`**.
Lifecycle scripts additionally verified with patch **`74c57db542c53f4b5c58908ddcb5cd21fbdad514`**,
from a separate driver directory. The exact source archive was never patched.

**Result:** pinned MSVC release build, real virtual IPC/TLS/lifecycle, and real
identity/trust ACL checks pass. **Strict Windows Clippy fails** on the exact
source. This is an unresolved release gate, not equivalent green proof from the
older compiler or another target.

## Compiler and source integrity

- Rust `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`.
- Explicit `cargo +1.99.1`, `RUSTUP_TOOLCHAIN=1.99.1`, jobs=2, BelowNormal.
  No compiler/MSRV bypass. Global default remains `stable-x86_64-pc-windows-msvc`.
- A new owned Windows temp directory received `git archive b36e840`; old audit
  source and production files were untouched. All **477** original source files
  remained byte-identical. No files appeared outside the build target directory.
- Cargo.lock SHA-256, equal before/after and to the Git object:
  `7c293887f5785f252387741d9e47ffab39e016e699ad0e14ed3c29992c5bb2d0`.
- Windows release executable SHA-256:
  `65a92d63e2a189adcb1606d26611816774640578b2f82c1aa7ce811b3381e86d`.

```powershell
$env:RUSTUP_TOOLCHAIN = '1.99.1'
$env:CARGO_BUILD_JOBS = '2'
$env:OPENERGY_ALLOW_PORT_24800 = '0'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
cargo +1.99.1 build --release --locked -p openergy-daemon -p openergy-platform-windows
cargo +1.99.1 clippy --locked -p openergy-daemon -p openergy-platform-windows -p openergy-net -p openergy-session -p openergy-platform -p openergy-protocol -p openergy-config -p openergy-ipc --all-targets -- -D warnings
```

Release build passed (1m 31s). Strict Clippy exited 101:

```text
crates/platform-windows/src/clipboard.rs:189:14
chunks_exact(2): clippy::chunks_exact_to_as_chunks
help: consider using as_chunks::<2>().0.iter()
```

No lint suppression or compiler fallback was used. The requested report-only
validation did not change this Rust source. The later parent session guard
change `29fa8ba` is outside this exact-source proof; no unrelated native rebuild
was repeated for that change.

## Real checks

The exact compiled binary passed:

- `tools/windows/verify-pipe-identity.ps1`: identification-only pipe client,
  `PIPE_CLIENT_IMPERSONATION_LEVEL=2`, `PIPE_IMPERSONATION_PASS`.
- `tools/windows/verify-control.ps1 -Port 25241`: real CLI aliases, config
  save/apply, duplicate-daemon refusal, protected current-user-only kernel DACL,
  event/log delivery, fragmented requests, unknown method, 64-client cap and slot
  recovery, stop with an incomplete request, absent daemon, and parent exit.
  Marker: `WINDOWS_CONTROL_PASS`.
- **`crates/net/examples/verify_permissions.ps1 -Openergy $Binary`**: ran the real
  offline identity and trust CLI. Checked the protected TLS directory, cert/key,
  identity/trust locks, both trust stores, repair after deliberately adding an
  Everyone-read ACE to only the disposable key, identity preservation, and private
  atomic trust replacement/revocation. Marker: `WINDOWS_IDENTITY_TRUST_ACL=PASS`.
- Three real Windows virtual TLS daemons: generated separate identities,
  approved mutual pins through the CLI, confirmed `tls.enabled` and
  `tls.trusted` at the peers, forwarded virtual key/button input and clipboard
  both ways, forwarded a true zero-format clear, released held input on
  disconnect, and served retained clipboard to a late TLS peer while the
  server's clipboard read was unavailable. All stopped cleanly. Marker:
  `WINDOWS_VIRTUAL_TLS_CLEAR_PASS`.
- Invalid daemon option exited **2** before startup; absent daemon status exited
  **1**; malformed trust fingerprint exited **1**. These were real CLI calls
  with explicit private config/socket/TLS selectors.

The existing full lifecycle driver passed with the patched scripts on both
PowerShell 7 and Windows PowerShell 5.1 (`-NoStartup -Platform virtual`, ports
25240/25242). The new ownership driver passed occupied-endpoint refusal,
independent-daemon survival, stale start-time rejection, same-connection kernel
server-PID verification, invalid owner/path migration preflight, and no-state
rollback refusal. See [lifecycle evidence](LIFECYCLE-OWNERSHIP.md).

Replay entry points:

```powershell
& "$Source\tools\windows\verify-pipe-identity.ps1" -Exe $Binary
& "$Source\tools\windows\verify-control.ps1" -BinaryPath $Binary -Port 25241
& "$Source\crates\net\examples\verify_permissions.ps1" -Openergy $Binary
& "$PatchedScripts\verify-lifecycle.ps1" -BinaryPath $Binary -Port 25240
& "$PatchedScripts\verify-lifecycle-ownership.ps1" -BinaryPath $Binary -ScriptsDir $PatchedScripts
# Retained owned lab driver, outside the immutable source archive:
& "$Lab\drivers\verify-virtual-tls.ps1" -BinaryPath $Binary -LabRoot $FreshPrivateTlsDir
```

The owned source, build logs, source hash manifest, binary, and TLS driver remain
under `%TEMP%\oe-win-final-b36e840-20261021` for follow-up. Successful lifecycle
labs removed their temp installs. No owned final test daemon remained.

## Limits

Every daemon run used **`--platform virtual`**, isolated temp paths, disabled
discovery, and loopback ports within 25240–25259. No port 24800, native backend
startup, native clipboard/input/hooks, desktop interaction, production install,
service modification, task registration, login item, default setting, push or
PR operation ran. Actual service restoration, scheduled-task behavior, native
Windows desktop correctness, cross-user/remote-SMB pipe access, and
mixed-integrity input acceptance remain unverified. No unit tests or mocks
were added. Strict Windows Clippy remains failed for the exact tested source.
