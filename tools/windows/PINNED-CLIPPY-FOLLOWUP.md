# Pinned Windows Clippy follow-up — 2026-10-22

**Result: green.** Rust 1.99.1 strict Windows all-targets Clippy and the locked
MSVC release build pass. Real virtual IPC and identity/trust ACL checks also pass
with the new executable. The [original b36 validation report](INTEGRATED-b36e840.md)
remains unchanged as evidence of the original failure.

## Change and exact validation source

Owned source fix: **`99d43b16c0d2f1e6db629edbe01e33c4f9e58516`**.
`crates/platform-windows/src/clipboard.rs` now uses
`as_chunks::<2>().0.iter()` instead of `chunks_exact(2)`. The preceding odd-byte
length rejection remains in place, so UTF-16 decoding has the same behavior.
No lint suppression, MSRV bypass, or compiler fallback was used.

A new owned temp source directory was extracted from the retained exact
`b36e840ea40b0c455db41afa6617d376b3b31898` archive. It received only two overlays:

- Windows clipboard source from `99d43b1`, SHA-256
  `451bc0555457ce95f7f5d2ba7ae657834925cd708b186e14678686b12ce8dc80`.
- Session clipboard source from `c69b81de6af72242250cb93fd7eb58155a797483`,
  the parent's nine added format-guard lines, SHA-256
  `b65912bcdc8e852d2b03e4955966bf127f3b6f02ad52ae790ff1ba469c37ef18`.

All **477** files in the original b36 source tree remain byte-identical to its
pre-build manifest. The new source differs from that manifest in exactly those
two overlay files. Cargo.lock remains unchanged, SHA-256:
`7c293887f5785f252387741d9e47ffab39e016e699ad0e14ed3c29992c5bb2d0`.
The old build target directory was reused only as a compiler cache. The original
b36 executable was retained separately and still has its recorded hash
`65a92d63e2a189adcb1606d26611816774640578b2f82c1aa7ce811b3381e86d`.

## Pinned compiler gate

Rust `1.99.1 (58cf8f9c3 2026-10-13)` and Cargo
`1.99.1 (5f94df478 2026-08-27)`, native `x86_64-pc-windows-msvc`.
The global default toolchain was not changed. Commands ran from the new source
directory, with BelowNormal priority and two build jobs:

```powershell
$env:RUSTUP_TOOLCHAIN = '1.99.1'
$env:CARGO_BUILD_JOBS = '2'
$env:OPENERGY_ALLOW_PORT_24800 = '0'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
cargo +1.99.1 clippy --locked -p openergy-daemon -p openergy-platform-windows -p openergy-net -p openergy-session -p openergy-platform -p openergy-protocol -p openergy-config -p openergy-ipc --all-targets -- -D warnings
cargo +1.99.1 build --release --locked -p openergy-daemon -p openergy-platform-windows
```

- Strict Clippy: **exit 0**, all eight selected packages, no warnings, 4.13s.
- Release build: **exit 0**, 38.59s.
- New executable SHA-256:
  `db3586f5a57a97ae189f12c47b0a3d68486741690084971d19eb62f219a7ab7b`.

## Real executable checks

The following existing drivers ran against that new release executable:

```powershell
& "$Source\tools\windows\verify-pipe-identity.ps1" -Exe $Binary
& "$Source\tools\windows\verify-control.ps1" -BinaryPath $Binary -Port 25241
& "$Source\crates\net\examples\verify_permissions.ps1" -Openergy $Binary
```

- Pipe client identification level **2**: `PIPE_IMPERSONATION_PASS`.
- Real CLI/control paths, protected current-user-only kernel pipe DACL,
  duplicate-daemon rejection, event/log delivery, fragmented frames, unknown
  method, client cap and recovery, stop, absent-daemon failure, and parent-exit
  handling: `WINDOWS_CONTROL_PASS`.
- Real identity/trust CLI and kernel ACL verification, including private files
  and locks, repair of an intentionally broadened disposable key ACL, identity
  preservation, and trust-store updates: `WINDOWS_IDENTITY_TRUST_ACL=PASS`.
- Final integrity and cleanup: `HISTORICAL_SOURCE_UNCHANGED=477`, exactly two
  expected overlays, `NO_OWNED_FOLLOWUP_DAEMONS`, `PINNED_WINDOWS_GREEN_GATE`.

Source, logs, and the new executable remain under
`%TEMP%\oe-win-clippy-followup-20261022`. The historical source, archive, manifest,
and retained executable remain under `%TEMP%\oe-win-final-b36e840-20261021`.

The iterator-only change did not repeat the full earlier TLS/lifecycle suite.
That evidence remains in the historical report and
[lifecycle ownership report](LIFECYCLE-OWNERSHIP.md). Every runtime daemon in this
follow-up used the virtual platform, private temp state, and loopback port 25241.
No native desktop, clipboard, input/hooks, service, scheduled task, startup,
production installation, port 24800, push, or PR operation ran. Native desktop
behavior and real Synergy/service restoration remain unverified. No unit tests
or mocks were added.
