# Config TOML size policy: final P2 verification

Base: `2052b58eacc233c7f112e3fac4bbed1474ff74f8`. Local macOS CLI
verification only; no distribution or full-suite integration claim.

`openergy_config::MAX_CONFIG_FILE_LEN` owns the 16 MiB (16,777,216 byte)
daemon TOML policy. Startup and runtime saves reuse it. Offline import
serializes once and checks before `write_atomic` or TOML stdout output.
JSON-only inspection and Synergy-format export retain their existing limits.
CLI `--apply` remains unsupported; live import uses `config.import_synergy`
with `apply: true`, which already reaches the checked runtime writer.

## Real interface results

- Reported baseline reproduced: a 16,777,088-byte `db.json` replaced a valid
  destination with 16,777,605 bytes and exited 0; startup then exited 1.
- Fixed import refuses that input's TOML output, with and without `--json`:
  existing bytes stay identical; absent destinations, parent directories and
  temporary output files are not created. TOML stdout is empty on refusal.
- Exactly 16,777,216 serialized bytes import and start on the virtual backend.
  One byte more is refused and preserves the exact-cap destination.
- Small file/JSON imports, normal replacement, export, live apply and restart
  pass. Oversized live apply preserves disk and live config. Legacy and modern
  fixture stdout, JSON, warnings, empty-input and error behavior match baseline.
- A Synergy-format export above 16 MiB succeeds from an input within its
  existing separate import limit. No general export-output cap was added.

22 scenario checks pass across 43 recorded CLI calls. Virtual daemons used
private temporary paths, disabled TLS/discovery, and `127.0.0.1:26660`.
All owned daemons exited; no TLS state remains or was created. No unit tests,
mocks, native backend runs, installed state, services, or protected ports were
used. The existing review inputs and other validation targets were unchanged.

## Build and evidence

Locked release build, scoped strict release Clippy (`openergy-config` and
`openergy-daemon`, `--no-deps`, `-D warnings`), `cargo fmt --all -- --check`,
and `git diff --check` pass. Builds used nice 15, two jobs and the private
`/tmp/oe-r6-config-target`; all local daemon dependency crates were forced to
rebuild after copying the cache. The parent's target was not changed.

Binary: `/tmp/oe-r6-config-target/release/openergy`.
SHA-256: `22f116f5c54df27cbae2d11b7479770c56aa02060e0bfa7996f66d0aa0dc04f6`.

Source SHA-256:

- `crates/config/src/cli.rs`: `21df1802c13c89bb4569f1e0f3b0290aa212dfdb1a6d2e7ce9aa5e32d0262246`
- `crates/config/src/lib.rs`: `214d8694a6d1b93bffd10b3ccb150da95191b6ce64b83a7a9e347ea9290979a4`
- `crates/daemon/src/main.rs`: `ba5a5d6ae3817bed2a9fd6955b9fee751983bc6b1f53c0df9f9e12f76519bb61`

Private evidence: `oe-r6-config-ny_7npq0/verification.json` under the local
temporary directory (machine-specific parent path omitted).
That directory also contains the baseline result, daemon logs, generated
fixtures, and a SHA-256 manifest of tracked Rust/Cargo build inputs. Build and
Clippy logs are `/tmp/oe-r6-config-build.log` and
`/tmp/oe-r6-config-clippy.log`; the CLI harness is
`/tmp/oe-r6-config-verify.py`. These temporary artifacts are not committed.
