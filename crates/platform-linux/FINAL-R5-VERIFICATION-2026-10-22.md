# Linux final hardening verification — 2026-10-22

All checks below passed on isolated Linux ARM64. No product source changed in
this verification. The base checks and the final config overlay checks are
separate runs; this report does not claim a full matrix rerun on the overlay.

## Exact source and build identity

Base: `1ef54fcf8e866a5799ba38a42674ddeff8452b5c`, exported with
`git archive` to an owned temporary directory. A fresh archive comparison
confirmed all **514 files**. The manifest records each relative path, SHA-256,
and permission mode. Archive bytes follow `.gitattributes`, including CRLF for
Windows command files.

Overlay: a separate copy of that base, replacing **only** these three files
with exact blobs from `ccf98477b6485dcd29d384c0444ec19c4ca76a6f`.
Each blob also matches parent `6f6a99c`.

| Overlay file | SHA-256 |
| --- | --- |
| `crates/config/src/cli.rs` | `21df1802c13c89bb4569f1e0f3b0290aa212dfdb1a6d2e7ce9aa5e32d0262246` |
| `crates/config/src/lib.rs` | `214d8694a6d1b93bffd10b3ccb150da95191b6ce64b83a7a9e347ea9290979a4` |
| `crates/daemon/src/main.rs` | `ba5a5d6ae3817bed2a9fd6955b9fee751983bc6b1f53c0df9f9e12f76519bb61` |

| Inventory | Files | Manifest SHA-256, identical before and after |
| --- | --- | --- |
| Base | 514 | `4bb92cbe69d97b7ac7b7a33d5cb2fcf78428ef56de1c5a5ad0a4e37f3b9f93e0` |
| Base plus three-file overlay | 514 | `e453dc93c5aa66f98b22ba211745f96e55015ac3afab79c3c1c380bef131b3c7` |

Both runs used the existing ARM64 image `oe-linux-dev-wp8:rust-1.99`, image ID
`sha256:aa40e749da1db39335f227757e52ea058a0a1e9d10883d59e23f9aab34a81886`.
Actual compiler output was `rustc 1.99.1 (58cf8f9c3 2026-10-13)`;
host/target was `aarch64-unknown-linux-gnu`, with Cargo 1.99.1.
Each run confirmed nice 15 and two Cargo jobs. Existing dependency caches were
reused. All **96 Rust source files** received fresh mtimes only in the
container's disposable `/work` copy before compilation. The base build log
confirms compilation of all **11 workspace packages**. The overlay build log
confirms fresh compilation of both config and daemon.

The source mounts were read-only. File contents and modes matched before and
after both runs, and the host exports matched those inventories afterward.
`Cargo.lock` stayed unchanged:
`0cc466112ead33a26cb59fce717d83c202bd470e6ba64394ab7c81ac560c6f3f`.

## Build commands

The existing release and Linux helper scripts were read before execution.
These commands ran inside Docker, with `CARGO_BUILD_JOBS=2` and inherited
nice 15. No packaging or publishing command ran.

```sh
# Exact base.
python3 tools/release/release.py check
cargo build --release --locked --workspace --target aarch64-unknown-linux-gnu
cargo clippy --release --locked --workspace --all-targets --target aarch64-unknown-linux-gnu -- -D warnings
cargo build --release --locked -p openergy-platform-linux --example x11_probe --target aarch64-unknown-linux-gnu

# Exact base plus the three overlay files.
cargo build --release --locked -p openergy-daemon --target aarch64-unknown-linux-gnu
cargo clippy --release --locked -p openergy-config -p openergy-daemon --all-targets --target aarch64-unknown-linux-gnu -- -D warnings
```

| Artifact | SHA-256, identical before and after runtime checks |
| --- | --- |
| Base `openergy` | `2f115bf600436e5e91e71f00f0afc4fc6ebc6ec2cbeb3736f3dd73b316140323` |
| Base `openergy-e2e` | `9ed9891dab5f8f4a8e73b8b8ec8c4812569199ced6b9c0dbfd2d927b4d9ce8dc` |
| Base `x11_probe` | `d4d0542af121c37ba37fcce38ed4bec0321822aa37f26482596355ab2b77ec95` |
| Overlay `openergy` | `ce89bc95477bdeeb38d351dff8f0d03b2fd752c81a7156355dad67571cf28b82` |

## Real interface results

The base passed the release script's public CLI smoke: shipped version,
absent-daemon error, virtual IPC status, invalid method, and clean stop.
Four further private CLI checks passed: trust reads above 1 MiB refuse without
mutation; two-role trust approval preflight preserves both files on serialized
expansion; revocation compacts 15,999 remaining pins within 1 MiB; startup
refuses config above 16 MiB before creating its IPC socket.

The external runner passed **10 scenarios**, with zero failures, exclusions,
or cleanup failures. Every run used `--no-docker --keep`,
`--port-range 26560-26579`, and `--lab-port-range 26580-26599`:

- `R5-client-config-retains-clipboard`
- `R5-dead-corner-negative-origin`
- `R5-dead-corner-positive-origin`
- `R5-double-tap-stationary-timeout`
- `R5-server-live-alias-merge`
- `R5-server-live-alias-rename`
- `Q18-empty-clipboard-clear`
- `D2-restart-disconnected-workers`
- `S19-config-patch`
- `R5-incoming-clipboard-cancellation` (from `crates/net/examples/r5-scenarios`)

Selections were `R5-*`, `Q18-*`, `D2-*`, and `S19-*` in the normal catalogue,
plus the incoming-cancellation catalogue. All daemon processes used the
virtual backend, private config/socket/TLS paths, and disabled discovery.

The base's real public Linux backend passed the focused Xvfb checks:

- `r5_x11_resources.py` refused 4096- and 8192-side compressed image expansion
  while preserving the prior text selection and change stamp. Its report
  records the exact probe hash and 18,833,408-byte observed peak RSS.
- `x11_probe clipboard` passed small image/text/HTML conversions, byte-exact
  10,498,508-byte INCR transfers in both directions, and oversized-read refusal.
- Stable ownerless selection and explicit clear remained true empty data.
  Zero-byte supported UTF-8/HTML stayed readable. Unsupported-only owners,
  refused/missing conversions, and ownership changes returned unavailable.
  PRIMARY remained separate from shared CLIPBOARD.
- `verify-pixels.py` independently confirmed RGBA preservation, including the
  large INCR image.

On the overlay, **nine checks across nine real CLI calls** passed. Normal
plain/JSON imports wrote equal readable TOML, export read it, and normal TOML
stdout retained its summary and readable config. A valid **16,777,088-byte**
Synergy `db.json` that expands beyond the 16 MiB TOML cap was refused:

- With and without `--json`, existing destination bytes stayed identical.
- With and without `--json`, missing destinations and parent directories were
  not created; no temporary output files appeared.
- Without `--out`, refusal emitted zero stdout bytes and the 16 MiB error.

The overlay CLI checks created no IPC socket or TLS state. They did not repeat
the base's network, lifecycle, or Xvfb matrix. No integration defect was found.

## Evidence and limits

Private evidence root: `/tmp/oe-linux-final-1ef54fc-lrwxXB`.
`evidence/` retains the base's separate 514-file `source-before.json` and
`source-after.json`, build/Clippy logs, forced-mtime list, binary hashes,
CLI results, E2E JSON/artifacts, and Xvfb logs/report. `overlay-evidence/`
retains its separate inventories, `overlay-files.json`, `host-integrity.json`,
build/Clippy logs, binary hashes, and `import-results.json`. The external
verification scripts remain beside these directories. An earlier base harness
attempt used an unsupported offline trust CLI flag; its evidence is retained
in `evidence-attempt1/` and is not counted as a passing runtime run.

Only owned containers and disposable state were used; both containers exited
and were removed. No host desktop/input/clipboard, installed application or
live state, TCC, SSH, services, login items, port 24800, unit tests, mocks,
push, PR, or release operation was used. The assigned lab range was supplied
to the runner but no interop lab ran. The fixed-port queue RSS script was not
run because its ports fall outside this assignment. The single Xvfb RSS sample
is not a general memory bound.

This is ARM64 Linux build and isolated virtual/Xvfb evidence. The release
script's supported Linux distribution target is x86_64; ARM64 is not claimed
as a supported packaged release. Native desktop acceptance, Wayland, real
hardware input, third-party interop, and other operating systems remain
outside this verification.
