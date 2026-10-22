# Config release-readiness audit

Date: 2026-10-21. Scope: `crates/config` and new C-prefixed external
E2E scenarios. Parent: `06b6128`. No dependency or root-manifest changes.

## Concrete fixes

- A Synergy client stored its remote host without the separate core port.
  The daemon then used its default port. Imports now put `core/port` or
  `developers.corePort` into host-only client endpoints. Explicit endpoint
  ports stay unchanged. IPv4, DNS names, and bracketed or bare IPv6 are covered.
- A directory import validated the intermediate local server identity after
  replacing db screens with classic screens. It could reject valid settings
  before applying the authoritative core INI name. Validation now runs after
  that final overlay. Invalid final identities still fail without writing.
- `export-synergy-conf` ignored `OPENERGY_CONFIG`. It could read the platform
  default instead of the selected isolated config. Explicit `--config` wins,
  then `OPENERGY_CONFIG`, then the existing platform default. An empty explicit
  environment selector fails rather than falling back.
- The config CLI's atomic writer synced file data but omitted the parent
  directory sync after rename. Unix writes now sync both. Failure before
  rename preserves the old file; a directory-sync error after rename reports
  uncertain durability of the new file. Temp files remain exclusive and 0600.
- Duplicate or empty active computer IDs could produce ambiguous local
  identity and corner-setting joins. Imports now reject them. A nonobject
  `local.json` is also rejected rather than treated as missing settings.
- A FIFO at an expected import filename could block the CLI before its byte
  limit applied. Inputs must now be regular files before opening them.
- `clipboard.max_bytes` accepted values above the session's 64 MiB hard cap.
  The config now rejects larger values with the supported maximum in the error.
  The default stays 64 MiB. `MAX_CLIPBOARD_BYTES` exposes that config limit.

The existing verifier now uses a fresh private directory, explicit config,
socket and TLS selectors, disabled discovery, and two reserved ports from
`--port-range`. Its old Barrier byte-roundtrip assertion was stale: the current
exporter writes the startup dialect as an informational comment. The verifier
now checks the representable screen, link, switch and hotkey settings for that
case, and retains byte equality for the Synergy fixture.

## Real verification

All process builds use `nice -n 15` and `CARGO_BUILD_JOBS=2`. All daemon runs use
the virtual backend. Active TCP endpoints use only assigned loopback ports
25160-25179. The external runner also receives lab range 25180-25199; no Docker
or external peer is needed for this scope. Offline fixture addresses are never
resolved or contacted. No native input, clipboard, permissions, live settings,
or installed application is used.

```sh
nice -n 15 env CARGO_BUILD_JOBS=2 cargo build --release --locked \
  -p openergy-daemon -p openergy-e2e
cargo fmt -p openergy-config -- --check
nice -n 15 python3 crates/config/verify.py --port-range 25160-25179
nice -n 15 python3 crates/config/verify-release.py
nice -n 15 target/release/openergy-e2e run --scenario 'C*' \
  --port-range 25160-25179 --lab-port-range 25180-25199 --keep
nice -n 15 target/release/openergy-e2e run --scenario 'S14-*' \
  --port-range 25160-25179 --lab-port-range 25180-25199 --keep
nice -n 15 target/release/openergy-e2e run --scenario 'S19-*' \
  --port-range 25160-25179 --lab-port-range 25180-25199 --keep
```

Final results: release build and formatting passed; the broad verifier passed
83 real CLI invocations; the release verifier passed 42 real CLI invocations;
C1-C3, S14 and S19 passed (5/5 external scenarios). The release verifier covers
both TOML export rejection and classic import rejection above the clipboard cap.
No unit tests, mocks or test doubles were added. `git diff --check` passed.

C1 and C2 failed through real commands before their fixes, then passed. C1
checks the imported endpoint, a real connection and restart. C2 checks the
final server identity, real connection, and unchanged output after an invalid
final import. C3 failed on the original binary accepting an excessive cap. It
checks acceptance at 64 MiB, rejection at 64 MiB plus one byte and 256 MiB,
unchanged file bytes and live config after failure, and restart persistence.

`verify-release.py` also kills an owned real export process after observing
its replacement temp file. It requires whole old or whole new output, then
checks that a fresh CLI process can replace it with a complete 0600 document.
This is process-crash evidence, not a power-loss simulation. Synthetic files
and raw reports remain in private temporary directories; no personal paths,
addresses or hostnames are copied into this evidence file.

## Integration notes and limits

The CI owner requested `is_multiple_of(1024)` for the new workspace MSRV of
1.99.1. That equivalent change is included. This isolated parent still declares
1.85, so ordinary Clippy reports `incompatible_msrv` for the new method. Code
lint checks pass with a temporary external `clippy.toml` containing
`msrv = "1.99.1"`:

```sh
nice -n 15 env CARGO_BUILD_JOBS=2 CLIPPY_CONF_DIR=<temporary-config-directory> \
  cargo clippy --release --locked -p openergy-config -- -D warnings
```

Clippy reports the expected manifest/config MSRV mismatch warning. Integrate
the CI owner's MSRV update before the final ordinary workspace lint gate.
The compiler used here is Rust 1.99.1. No MSRV suppression was committed.

Cross-scope findings from source inspection, for the parent to route:

- `crates/daemon/src/runtime.rs::write_config` has a separate writer. It syncs
  the temp file, then renames without syncing the directory. The config CLI
  fix does not change that daemon path.
- `crates/daemon/src/main.rs::parse_opts` overwrites duplicate value flags,
  can consume another flag as a value, and callers ignore extra positional
  arguments. Runtime role/remote overrides happen after TOML validation.
- The daemon's Linux config default ignores `XDG_CONFIG_HOME`, unlike config
  export and the documented config location.
- Config accepts/imports named-key and mouse lock hotkeys, including Meta and
  AltGr. The current session server matcher handles only key-down events with
  a single-character key and a smaller modifier set. Those bindings need
  session-owner verification or explicit support limits.

No Linux or Windows runtime, third-party Synergy peer, full workspace suite,
power-loss test, or production deployment was run by this config task. The
default import-directory discovery remains macOS-specific; use `--from` on
other platforms. The parent owns full integration and release validation.
