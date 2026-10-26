# Public-release license, provenance, and privacy audit

Date: 2026-10-21, America/Vancouver. Baseline:
`06b61287afd7f31aa3002d252389f2c2b17f73f5`. This is an audit of that tree and
its reachable history, plus the owned sanitation and notice changes here.
It is not a release approval or a claim that pending cross-scope fixes passed.

The notice generator, `dependency-licenses.json`, and the supplementary
`licenses/` texts this audit added now live in
[tools/release/notices](../../../tools/release/notices). Release tooling reads
them from there.

## Scope and safety

Reviewed 376 tracked paths, including Rust/Swift source, manifests, scripts,
research reports, generated image fixtures, logs, JSON, and decompressed
focus-lab reports. Inspected the root and independent spike manifests.
The Swift packages declare no external package dependencies. The spike has
no external Rust dependencies; its package omits license metadata even though
the repository has MIT/Apache-2.0 license texts.

No Openergy or Synergy executable ran. No native input, clipboard operation,
GUI control, TCC request, SSH connection, daemon, or listener was used.
No installed app, live configuration, keychain, login item, or unowned process
was changed. No builds, unit tests, or TypeScript checks were needed for these
documentation, policy, and replay-path changes. No git history was rewritten.

## Changes within the audit scope

- Added `THIRD_PARTY_NOTICES.md`: all 108 external package/version records
  resolved by the locked root Cargo graph, including build tools and all
  platforms. License texts retain their upstream copyright and permission
  notices. `ring` includes its BoringSSL, fiat, and once_cell notices;
  `unicode-ident` includes its additional Unicode terms.
- Added X11 keysym data attribution. `unicode_keysyms.rs` names Xorg
  `keysymdef.h` as its data source but had no supporting notices. The generator
  and exact original header revision were not recorded. The retained permission
  header was checked against the X.org historical mirror, not claimed as the
  original generating file.
- Added a deterministic notice generator and a package/license-file hash
  inventory. The published `asn1-rs-impl 0.2.0` package omits license files;
  its MIT text was retrieved from the exact revision in `.cargo_vcs_info.json`:
  `a20e5f7319c896737ad0f2557037817b91ad854f`. The supplementary text and URL
  are retained here. Unknown package license layouts fail for manual review.
- Enabled development-dependency license checks and rejected yanked crates in
  `deny.toml`. No advisory was ignored. Wildcard denial remains enabled.
- Replaced unsupported clean-room labels in the research reports with the
  actual source-informed/static method and added provenance limits. Kept
  interface evidence, schema descriptions, hashes, and confidence boundaries.
- Replaced five personal home-path values in the three compressed focus-lab
  JSON reports. Verified equality with their original decoded JSON after only
  this string substitution. Kept all result/event values. Gzip metadata now has
  zero mtime and no original filename.
- Removed the hardcoded personal Windows profile path from `network.py`.
  The driver now requires `OPENERGY_WINDOWS_LAB_ROOT` before it can run.

## Real scan results

Tools were downloaded to a private temporary directory from their upstream
GitHub releases. Archive SHA-256 values matched the upstream checksum assets:

```text
cargo-deny 0.20.2 aarch64-apple-darwin
fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3
gitleaks 8.30.1 darwin_arm64
b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5
```

`cargo metadata --locked --offline --format-version 1` resolved 119 packages:
11 workspace packages and 108 external packages. No manifest or lockfile was
changed. `dependency-licenses.json` records the root lockfile hash and full
SHA-256 for every retained package license text.

`cargo-deny --color never check licenses sources` **passed, exit 0** after the
policy changes. `cargo-deny --color never check` **failed, exit 3**, before and
after those changes, on the following real findings:

- `RUSTSEC-2025-0134`: `rustls-pemfile 2.2.0` is unmaintained. This is an
  informational maintenance advisory, not a reported exploitable vulnerability.
  The scanner reports no patched version. See the
  [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2025-0134.html).
- 23 versionless local path dependency entries fail the wildcard policy in
  seven manifests. There were also warnings for duplicate `windows-sys` and
  `winnow` versions and three unused allowed-license entries. These warnings
  are retained in the logs, not represented as vulnerabilities.

The RustSec database was fetched successfully at
`4b587f00996ebe5c05a73eaef105ef241e876414` (last commit
`2026-10-21T08:20:38+02:00`). `cargo-audit` was not installed; cargo-deny's real
RustSec check supplied the advisory evidence. No advisory was invented or
suppressed. Logs here replace the checkout prefix with `<repo>` and remove
trailing whitespace. Findings and result lines are preserved.

`gitleaks git . --log-opts=HEAD --redact=100 --ignore-gitleaks-allow
--max-archive-depth 2 --timeout 120` **passed, exit 0**. It reported 95
content-bearing commits and approximately 3.92 MB scanned, with zero findings.
`git rev-list --count HEAD` counts 106 reachable commits, including merges.
An independent object pass read all 686 distinct named blobs reachable from
HEAD, decoded gzip/JSON, and checked key/token patterns and personal paths.
It found no credential-pattern or private-key marker matches. Gitleaks's
historical archive handling is supplemented by this explicit decoded-object
pass; the scanner's zero count alone is not an archive-coverage proof.

A separate current-tree snapshot scan includes new audit files, explicitly
decompressed and decoded JSON, and all tracked text. Gitleaks reported zero
findings. This is a bounded pattern scan and manual inspection, not proof that
all possible secret formats or copied expressions are absent.

The real `generate-notices.py --check` command passed, exit 0. A temporarily
stale notices file made the same command fail, exit 1; after restoration it
passed again. Both edited Python files parsed successfully. Running the real
Windows replay driver with `OPENERGY_WINDOWS_LAB_ROOT` unset failed immediately,
exit 1, with the required-directory error, before any SSH or daemon command.
No Windows replay was run. The compressed-report equality check passed for all
three files; their original overall results remain true, false, and true for
`native`, `builtin-legacy`, and `legacy`, respectively.

The tracked binary material consists of generated PNG/TIFF/DIB verification
fixtures, malformed-image fixtures, and gzip JSON reports. The image generator
is retained in `crates/platform-macos/examples/mac_probe.rs`. The PNG EXIF data
checked here contains color-space and dimension fields, not an author or GPS
record. No Mach-O, ELF, PE, application archive, vendored product font,
source-map file, or copied target bundle was found in the tree or the 686
historical blobs. No copied target implementation body was identified in the
tracked source or reports. This inspection cannot prove independent authorship.

## Publication findings outside this worktree's edit ownership

The parent and relevant owners received these findings. Do not treat their
resolution as verified by this baseline audit.

1. **Maintenance gate:** `crates/net/src/identity.rs:65,72` uses the unmaintained
   PEM wrapper. Small fix: use `rustls::pki_types::pem::PemObject` through the
   existing rustls dependency; remove `rustls-pemfile` from
   `crates/net/Cargo.toml` and root `Cargo.toml`, then regenerate `Cargo.lock`.
   Rerun the real TLS identity/trust flows and cargo-deny after integration.
2. **Wildcard gate:** add `version = "0.1.0"` beside the local path dependency
   entries in `crates/{daemon,net,session,platform-macos,platform-linux,platform-windows}/Cargo.toml`
   and `tools/e2e/Cargo.toml`. Alternatively, explicitly mark packages private
   only if they are not intended for publication. Do not disable wildcard
   checks globally to hide these failures. `allow-wildcard-paths` only exempts
   private crates in the audited cargo-deny version.
3. **Current personal paths:** `crates/daemon/VERIFICATION.md:223–229` and
   `crates/net/VERIFICATION.md:135` retain machine-specific temporary-directory
   identifiers. Replace only those path prefixes with `<temp>` and preserve
   command/result evidence. No current secret or personal-path finding was
   identified in `tools/e2e` or `tools/interop-lab`; those files were reviewed
   but not modified here.
4. **Distribution notices:** `apps/macos/scripts/bundle.sh` does not include
   license/notices files in the assembled bundle at this baseline. Copy
   `LICENSE-MIT`, `LICENSE-APACHE`, and `THIRD_PARTY_NOTICES.md` to
   `Contents/Resources` before signing, and keep them with each bare-daemon
   archive. `LICENSE*`/`NOTICE*` globs alone miss `THIRD_PARTY_NOTICES.md`.
5. **README provenance/trademark claims:** the final license paragraph's
   absolute no-upstream-code claim is not proved by this audit. Describe the
   independent project and source-informed interoperability research with its
   limits. Its Symless vendor-name attribution is stale against the current
   [vendor EULA](https://symless.com/synergy/legal/eula). Use neutral project
   names and no-endorsement wording rather than an unverified ownership claim.
6. **Spike package metadata:** `spikes/macos-input/Cargo.toml` has no `license`
   field. It is private (`publish = false`) and has no dependencies. Add
   `license = "MIT OR Apache-2.0"` to make its intended repository terms
   explicit if the spike stays in the public source distribution.

## Reachable-history privacy assessment

Publishing this branch's history publishes earlier file contents even if HEAD
is scrubbed. The following concrete locations remain reachable. The artifact
`history-privacy.json` records exact blob IDs, paths, and safe line references;
it deliberately omits the personal values. JSON line references use decoded,
pretty-printed JSON so escaped slashes do not hide path values.

- `crates/config/VERIFICATION.md`: two old blobs retain a personal checkout
  path, introduced by `ebb2067` and later changed by `981d617`.
- `crates/platform/VERIFICATION.md`: the original blob from `0b3dabf` retains
  personal checkout paths.
- `crates/platform-macos/verification/{build,clippy,examples-build}.txt`:
  original blobs from `9821823` retain personal checkout paths.
- `crates/platform-macos/verification/shared-contract/{build,clippy}.txt`:
  original blobs from `8754d5c` retain personal checkout paths.
- Those historical checkout paths were scrubbed at HEAD by `2c3d6d1`.
- `crates/daemon/VERIFICATION.md` has three reachable versions containing
  machine temporary-directory identifiers; `crates/net/VERIFICATION.md` has one.
- The baseline versions of the three focus-lab `.json.gz` files and Windows
  `network.py` retain the personal home/profile values scrubbed in this audit.

These are confirmed privacy exposures, not credential leaks or evidence of
proprietary implementation code. A new sanitation commit cannot remove them
from published history. No concrete credential or copied-proprietary-binary
blocker was identified by this audit. The publication owner must account for
the retained personal paths when selecting what history to publish; this task
does not authorize a history rewrite.

## Recheck after integration

```sh
python3 tools/release/notices/generate-notices.py
python3 tools/release/notices/generate-notices.py --check
cargo deny check
gitleaks git . --log-opts=HEAD --redact=100 --ignore-gitleaks-allow --max-archive-depth 2
```

The notice generator uses the real locked Cargo metadata and cached package
license files, with no Openergy execution. Changing the lockfile requires
regeneration. Repeat the decoded compressed-evidence privacy scan too; the
ordinary text search does not see escaped JSON paths inside gzip archives.
