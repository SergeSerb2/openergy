# Round 5 Windows verification — 2026-10-22

**Result: green for the scoped Windows gate.** Exact parent source
**`1ef54fcf8e866a5799ba38a42674ddeff8452b5c`**, with **no overlays** or source
changes, passed pinned MSVC strict all-targets Clippy, locked release build, and
actual private CLI, named-pipe, ACL, virtual TLS, and bounds checks. The later
three-file offline-import overlay also passed pinned build/Clippy and actual
normal/oversized import checks, as recorded below. No new product bug was found.
This commit contains only this report.

## Exact snapshot and integrity

The full `git archive` was copied into a new owned Windows temp fixture. A
locally generated manifest recorded all **514** source paths, sizes, and SHA-256
hashes. Windows checked the archive hash, every file hash and size, and the full
inventory before building. Source files then received the read-only attribute.
The build target directory was outside the source tree. After all checks,
Windows repeated the file/inventory/archive checks: **all 514 files unchanged,
no extra source files, no source backwrite**. No prior fixture was changed.

SHA-256 values:

| Artifact | SHA-256 |
| --- | --- |
| Source archive | `490293939f19f01f7598bcea81405a3b56b0caaa0c70bedb1df887e7afb29f90` |
| Full source manifest | `71a91a874771d88c12b8a8deaf139a96449f3b3abcdd7ebb337b9ac39ce2e8ae` |
| Cargo.lock, unchanged | `0cc466112ead33a26cb59fce717d83c202bd470e6ba64394ab7c81ac560c6f3f` |
| Cargo.toml | `1249baacd47b641edbab7350fa10f94d5a08624223c0ef1d7c8e49707c96fb0d` |
| rust-toolchain.toml | `f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1` |
| Release executable, before/after smoke | `9b68bd675b25210790292cca365671e96b1ef0368f53bf43e05057f7cc70f4d9` |

The final commit differs from preparation baseline `fe4432869f7531c0bf8f75d1ebadec40be0647a4`
in exactly these three files. Their hashes also match the security owner's
`/tmp/oe-r5-p2-evidence/report.json`:

| File | SHA-256 |
| --- | --- |
| crates/daemon/src/runtime.rs | `8f2a4bb49f2f70c70acbd92ef71aca3ca95a8f80af2e15fabfc945bfa919e230` |
| crates/net/src/trust.rs | `827860651eed7105e26f9c90e53600745dd339926c9928a3c217d78441f481db` |
| docs/plan/setup.md | `aec8ac1c62c5598b9ec9a91cd2a495b9c0840a64c5840324997be6b389f5fcb8` |

## Pinned native build

Host: `x86_64-pc-windows-msvc`. Rust
`1.99.1 (58cf8f9c3 2026-10-13)`, Cargo
`1.99.1 (5f94df478 2026-08-27)`, Clippy
`0.1.99 (58cf8f9c35 2026-10-13)`. The pinned compiler and required components
were already installed. Free space on the source volume was 1118.18 GiB.
The global default remained `stable-x86_64-pc-windows-msvc`.

```powershell
$env:RUSTUP_TOOLCHAIN = '1.99.1'
$env:CARGO_BUILD_JOBS = '2'
$env:CARGO_TARGET_DIR = Join-Path $Lab 'target'
$env:OPENERGY_ALLOW_PORT_24800 = '0'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
Set-Location (Join-Path $Lab 'source')
cargo +1.99.1 clippy --locked -p openergy-daemon -p openergy-platform-windows -p openergy-net -p openergy-session -p openergy-platform -p openergy-protocol -p openergy-config -p openergy-ipc --all-targets -- -D warnings
cargo +1.99.1 build --release --locked -p openergy-daemon -p openergy-platform-windows
```

Strict Clippy: **exit 0**, zero warnings, **22.08s**. Locked release build:
**exit 0**, **1m 09s**. No suppression, compiler/MSRV fallback, or bootstrap
override was used. These are build durations, not runtime performance results.

## Actual executable checks

All drivers used the hash-checked release executable. Runtime processes used
private config, TLS, and pipe selectors, disabled discovery, and the virtual
platform. TCP ports were **26521, 26525, and 26526**, all within 26520–26539.

- Named-pipe client identification level **2**: `PIPE_IMPERSONATION_PASS`.
- Actual daemon/CLI control: status/connect/disconnect, config save/apply,
  duplicate-daemon refusal, protected current-user-only kernel pipe DACL,
  event/log delivery, fragmented frames, unknown method error, client limit and
  recovery, incomplete-request shutdown, absent-daemon failure, and parent-exit
  lifecycle: `WINDOWS_CONTROL_PASS`.
- Exact-source `crates/net/examples/verify_permissions.ps1` ran against this
  binary. Actual identity/trust CLI operations verified private kernel ACLs on
  TLS files and locks, disposable key ACL repair without identity change, and
  protected trust approval/revocation: `WINDOWS_IDENTITY_TRUST_ACL=PASS`.
- Two actual virtual daemons generated separate identities and saved mutual
  pins through the CLI. Both reported connected peers with TLS enabled and
  trusted. Synthetic virtual clipboard text crossed TLS in both directions,
  followed by a true empty clipboard clear.
- A config patch inside the 16 MiB IPC frame cap whose serialized TOML exceeded
  16 MiB returned `config_apply_failed`. Disk hash and live input setting stayed
  unchanged, and the TLS peer stayed connected. A normal patch then saved,
  applied, and reloaded after a clean virtual daemon restart.
- Malformed fingerprint, absent daemon, and invalid daemon option returned the
  expected failure exits **1, 1, and 2**. No invalid command started a daemon.
- Real offline revocation removed one pin from each 13,983-pin compact trust
  list. Both results were exactly **908,830 bytes**, with every other pin
  preserved. Oversized addition and batch import failed without file changes;
  revoking an absent pin preserved both hashes.
- An additional preflight case used a small server list and a large client
  list. Approval and batch import rejected the oversized client output without
  saving the otherwise valid small server change. A valid small batch import
  saved both roles and reported two records.

Markers: `WINDOWS_ROUND5_TLS_BOUNDS_PASS`, `WINDOWS_ROUND5_PREFLIGHT_PASS`,
`ROUND5_REAL_SMOKE_GREEN`, and `ROUND5_WINDOWS_SOURCE_AND_RUNTIME_GREEN`.
Every owned daemon stopped. Final checks found **no owned process** and **no
listener in the assigned range**. Fallback cleanup checked executable path,
process start time, and binary hash before termination.

Executed entry points in the retained private fixture (runtime/preflight paths
were new when run; use a new fixture for another run):

```powershell
& "$Lab\verify-pipe-identity.ps1" -Exe "$Lab\openergy.exe"
& "$Lab\verify-control.ps1" -BinaryPath "$Lab\openergy.exe" -Port 26521
& "$Lab\verify_permissions.ps1" -Openergy "$Lab\openergy.exe"
& "$Lab\verify-round5.ps1" -BinaryPath "$Lab\openergy.exe" -LabRoot "$Lab\runtime"
& "$Lab\verify-trust-preflight.ps1" -Lab $Lab # requires the preceding bounds fixture
```

The temporary control driver changed only the assigned port range and cleanup
hash checks; the temporary pipe driver added cleanup identity/hash checks. The
ACL driver was unchanged. These driver copies did not alter the source snapshot.
Full driver manifest SHA-256:
`f572ecac12afdad18ff99cf76cfb362dbeace94338a78ad49c0c6fb02ff836f0`.
The focused TLS/bounds driver SHA-256 is
`ea28319f76c2c4aa4f44abb9890595be8b2b8ba49826b2a2d5dfdb93c6a3476c`;
the preflight driver SHA-256 is
`2e6a2063c1041cbd2b224c6326ce86397e6c1f5d939f0945931e8f262ed8560f`.

Source archive, full manifest, executable, drivers, and logs remain in
`%TEMP%\oe-win-r5-1ef54fc-20261022-fw_zoced`. Final summary SHA-256:
`cf62de179fc1f388195e70a2211585dfaafd89c8ef06970548984e433a1f932f`.

## Final offline-import overlay

The original baseline source, archive, executable, and evidence above remain
unchanged. A separate `delta/source` copy received **only** these three files
from **`ccf98477b6485dcd29d384c0444ec19c4ca76a6f`**. Each file also matched the
same Git object on parent **`6f6a99cf4dbdc953d845bcd1ca1b1d37d760c010`**:

| Overlay file | SHA-256 |
| --- | --- |
| crates/config/src/cli.rs | `21df1802c13c89bb4569f1e0f3b0290aa212dfdb1a6d2e7ce9aa5e32d0262246` |
| crates/config/src/lib.rs | `214d8694a6d1b93bffd10b3ccb150da95191b6ce64b83a7a9e347ea9290979a4` |
| crates/daemon/src/main.rs | `ba5a5d6ae3817bed2a9fd6955b9fee751983bc6b1f53c0df9f9e12f76519bb61` |

This proof covers that explicit composite source, not an archive of the whole
later parent commit. Windows checked all **514** final paths, sizes, and hashes
before and after validation. Exactly those three paths differ from the base.
The original 514-file inventory was checked separately and stayed unchanged.
Both copies retain the original Cargo.lock hash above. Source files were
read-only during builds, and the target cache stayed outside both source trees.

- Overlay archive SHA-256:
  `c17e3bcfe26d91f1bb0785c2e3d091b01b610ff0af9fc242792907e7eb5239be`.
- Overlay manifest SHA-256:
  `ad97f42ade085835604418d81beb3212492bfd1f58724e3c416a0a136cc7e8b1`.
- Complete final 514-file manifest SHA-256:
  `d20e1fc22daa141a514f3ad64446c217ee77c415dbdc021de67d65889bf40c80`.
- Final executable SHA-256, checked before and after CLI flows:
  `6a9c83fd4902fd943ea1e184d49be266728e8bb004be58d566967c4e8a7a0b76`.

The same explicit Rust **1.99.1** commands above ran from `delta/source`, using
the owned baseline target directory only as a compiler cache, BelowNormal
priority, and two jobs. Strict all-targets Clippy passed without warnings
(3.36s), and locked release compilation passed (37.25s). The temporary harness
then used an incorrect path to copy the cached executable. Its copy path was
corrected without a product source change. Both commands passed again from
cache (0.29s and 0.33s), and the final executable was retained separately under
`delta/openergy.exe`. The original baseline executable remains unchanged.

Actual CLI flows on that final executable passed:

- Ordinary import wrote TOML with both screens, ranged links, and private port
  26531. Export/reimport preserved those fields. Replacement of an owned
  existing destination worked. An empty import source failed with exit 1 and
  preserved the destination hash.
- A valid **16,777,088-byte** synthetic `db.json` produces daemon TOML above
  16 MiB. The fixture used only synthetic identities and an explicit loopback
  port 26531; no daemon was started for these offline checks.
- Oversized imports with and without `--json` returned exit **1** and
  `config exceeds 16 MiB (16777216 bytes)`. Existing destination hashes stayed
  identical, and missing destinations stayed absent. A nested missing parent
  directory was not created. No temporary output file or TLS state appeared.
- TOML stdout import also refused the oversized result before emitting any
  stdout bytes. Normal imports remained successful.

Markers: `WINDOWS_ROUND5_ORDINARY_IMPORT_PASS`,
`WINDOWS_OVERLAY_IMPORT_SIZE_PASS`, and `WINDOWS_FINAL_OVERLAY_GREEN`.
Final checks again found no owned process and no assigned-port listener. No
TLS/lifecycle/native matrix was repeated for this shared constant/import guard.

Evidence remains under the same private fixture's `delta` directory, including
the manifests, source copy, binary, first/final compiler logs, CLI logs, and
`evidence/final-summary.json`. SHA-256 values:

- Normal import driver:
  `193db0b42917139eacfc53f79b9a32cfb2fbbaddf5346f46fe28374d8b931de7`.
- Oversized import driver:
  `20baba268b89cdfe83a98ebadfb15bc7098c915da575d7b6de13c5b28176439f`.
- Oversized fixture `db.json`:
  `29f3a82e16c9053056d84bdf9da0601a7593ac839254968e6184751d84cfa80e`.
- Final overlay summary:
  `c7c9de7565ebf9ea5196fc9e30b0011325e2219defa32f7e054a8b1b98b0b51f`.

## Limits

No installed Openergy/Synergy, native backend, desktop input/clipboard, service,
scheduled task, startup entry, live config/TLS/default pipe, port 24800, Docker,
push, PR, or release operation was used. No unit tests or mocks were added.
This was a focused Windows virtual check, not a repeat of the parent's full E2E
suite or native desktop acceptance. Cross-OS TLS interoperability, actual
service/task behavior, native input/clipboard correctness, and performance
remain outside this evidence. Earlier reports remain historical and unchanged.
