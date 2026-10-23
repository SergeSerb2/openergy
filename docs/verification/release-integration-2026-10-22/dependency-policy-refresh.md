# Dependency policy refresh — 2026-10-22

## Result and exact scope

The unchanged dependency policy passes against refreshed advisory data at source
`7c1e488ab2a62c81ce01bf6f7d1a42ee4590c9bf`. The all-features combined graph and
five explicit platform graphs each pass advisories, bans, licenses and sources.
Every scan reports zero advisory errors or warnings. No advisory exception,
dependency update or policy relaxation was needed.

This follow-up uses the real cargo-deny command and existing Cargo configuration.
It does not compile or execute Openergy. The observed Mac daemon is unchanged:
3,707,520 bytes, SHA-256
`683de1fc9c64c062dc98482975f3f8dc05542426af36716a3a4fb299c406b5a4`.
The earlier [runtime evidence](receive-fairness.md) keeps its own scope.

## Advisory and registry freshness

The pinned [upstream cargo-deny 0.20.2 release](https://github.com/EmbarkStudios/cargo-deny/releases/tag/0.20.2)
was downloaded into a private temporary directory. Its archive SHA-256 matched
both the upstream checksum asset and the previously recorded value:
`fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3`.
No global tool installation or default toolchain change was made.

The actual `fetch db index` command succeeded. Subsequent scans also fetched
the advisory database. The clean local
[RustSec database](https://github.com/RustSec/advisory-db/tree/640bf5a9f06e66d8721e8772766531fe77c8630d)
was at `640bf5a9f06e66d8721e8772766531fe77c8630d`, with last commit time
`2026-10-22T12:49:47+02:00`. A separate `git ls-remote` confirmed this was still
the remote HEAD after all scans. The machine record retains UTC observation
times; this document's date uses America/Vancouver.

Registry freshness was checked separately from cargo-deny's cached-index read.
Actual HTTPS requests to the official sparse index covered **104 crate names
and all 107 external package versions** in locked, all-features Cargo metadata.
No version was yanked. Every returned package checksum matched Cargo.lock.
The record includes request URLs, response hashes, HTTP freshness headers and
per-version results. This compares registry and lockfile metadata; it is not a
new download-and-rehash audit of every crate archive.

Cargo metadata includes 11 workspace packages and 107 external packages.
Cargo-deny's activated combined graph has 93 packages. These are different
inventories: the larger locked inventory is also used by the existing notice
generator. Neither count means all packages are linked into every binary.

## Explicit platform coverage

All checks ran on the Mac host with pinned Rust 1.99.1 and unchanged policy.
The platform selectors inspect dependency graphs; they do not establish native
runtime or cross-compilation acceptance.

| Dependency graph | Packages | Policy result | Advisory findings |
| --- | ---: | --- | ---: |
| Combined, all features | 93 | All four checks passed | 0 |
| `aarch64-apple-darwin` | 73 | All four checks passed | 0 |
| `x86_64-apple-darwin` | 73 | All four checks passed | 0 |
| `x86_64-unknown-linux-gnu` | 79 | All four checks passed | 0 |
| `x86_64-pc-windows-msvc` | 75 | All four checks passed | 0 |
| `aarch64-unknown-linux-gnu` | 79 | All four checks passed | 0 |

The combined scan retains two duplicate-version warnings, for `windows-sys`
and `winnow`, and three unused-license allowances: `BSD-2-Clause`, `CC0-1.0`
and `Zlib`. Platform-specific warning sets are retained in the machine record.
These are policy warnings, not advisory findings. Dependencies and allowances
were left unchanged because this scan found no defect requiring an update.

The first explicit-target command repeated `--target`, which cargo-deny 0.20.2
rejects before scanning. Its reporting wrapper then tried to parse that plain
text error as JSON. Both failures are retained. Each final platform check uses
one supported `--target` argument and exits zero.

## Reproduce and integrity

With the pinned tool available, run from the repository root:

```sh
cargo-deny --locked --color never fetch db index
cargo-deny --locked --all-features --color never check --show-stats
cargo-deny --locked --all-features --target x86_64-pc-windows-msvc --color never check --show-stats
python3 docs/verification/public-release-audit-2026-10-21/generate-notices.py --check
```

Repeat the third command once per target listed above. The actual scan commands
also enabled information logs and JSON output; their full argument lists are
in the [machine record](dependency-policy-refresh.json). The
[combined log](dependency-policy-refresh.ndjson) preserves diagnostic and summary
values, with only the absolute checkout root replaced by `<repo>`.

The notice freshness check passed. Hashes of all **114 tracked Rust, Cargo,
toolchain, dependency-policy and notice inputs** matched before and after the
commands. Cargo.lock, manifests, deny.toml and THIRD_PARTY_NOTICES.md stayed
unchanged. No unit tests or mocks were added.

## Remaining release gates

This is a dependency-policy observation at one advisory database revision.
It does not clear the two original independent security review findings or
prove the program free of vulnerabilities. Native input/clipboard acceptance,
positive Windows migration/recovery and trusted signed/notarized distribution
remain open. Installed sharing and production state were preserved.
GitHub private reporting setup remains deferred at the maintainer's request.
