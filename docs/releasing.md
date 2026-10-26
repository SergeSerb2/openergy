# Native verification and draft releases

These scripts build and check local artifacts. They do not install Openergy,
launch its app, alter login items or TCC permissions, or change an existing
daemon. All runtime checks use isolated temporary config, TLS, and IPC paths,
the virtual platform, and explicit non-production port ranges. Port 24800 is
refused. Build jobs use two compiler workers and low process priority on Unix.

## Toolchain and version

Use the Rust version in `rust-toolchain.toml` (currently 1.99.1), with `clippy`
and `rustfmt`. Each workspace crate inherits that same `rust-version` and the
workspace version. The previous 1.85 declaration did not cover the locked
dependencies, some of which require at least Rust 1.88. This repository tests
the pinned toolchain; it does not claim support for an older Rust version.

Install Python 3.11 or later and Git. On macOS, use Xcode with Swift 6 and the
macOS 15 SDK or later. Python supplied by Apple's command line tools may be
too old; put a current Python on `PATH` or set `PYTHON` for `bundle.sh`.
Windows builds need the MSVC Rust toolchain, Visual Studio C++ build tools,
and the Windows SDK. Git Bash is also required by the GitHub workflow steps.
Linux builds need the system C linker/toolchain. No npm or TypeScript step is
needed by these native packages.

Run from the repository root:

```sh
python3 tools/release/release.py check
python3 tools/release/release.py verify \
  --port-range 25440-25459 --lab-port-range 25460-25479
```

In PowerShell, use `python` and put the command on one line, or use PowerShell's
backtick continuation. Select different, disjoint port ranges if another
verification run is active. Each range can contain at most 101 ports. The
Docker lab range must be at least 24900, as required by the external runner.

`verify` fetches the locked dependencies for all targets, then runs
`tools/release/notices/generate-notices.py --check`.
This fills the cache needed by the notice generator's offline Cargo metadata
and fails if the notice inventory is stale. After a dependency/lockfile change,
regenerate and review the notices before verification. It then runs locked
Cargo metadata, formatting, Clippy with warnings denied,
and a locked release build. It starts the real compiled daemon with the
virtual backend and checks the real CLI: version, a missing-daemon error,
IPC status, an invalid-method error, and clean shutdown. It removes its
temporary directory and stops only the child it started.

On Unix, the default then runs the existing external scenario catalogue with
`--no-docker`. The runner reports every excluded Docker scenario. The report
is `dist/verification.json`, or the path passed with `--report`. The
`--no-docker` flag is supplied by the release-readiness runner change and must
be integrated with this workflow. On Windows the default runs the CLI/IPC
smoke check only; the external scenario runner requires Unix process groups.
Windows Clippy and release builds cover the daemon and its dependencies,
rather than the Unix-only scenario runner.

Use `--suite smoke` for a focused native build plus CLI check. On Unix,
`--suite full` includes Docker interop and fails when its prerequisites are
missing. Build the lab image using `tools/interop-lab/README.md` first. No
missing tool or failed scenario counts as a passed check.

macOS verification also compiles the Swift app in release mode with automatic
dependency resolution disabled. It does not execute app E2E, native input,
clipboard, TCC, login-item, or real-device acceptance. Those remain separate
manual gates. In particular, virtual daemon checks do not establish that a
downloaded app can control a desktop.

The macOS app includes Sparkle. See [macOS updates](macos-updates.md) for its
Cloudflare feed, bundle configuration, signing key and universal update archive
workflow. Local ad-hoc bundles omit Hardened Runtime so Sparkle can load without
a Team ID. Named signing identities retain Hardened Runtime. Neither mode alone
establishes notarization or installed-update acceptance.

## Local archives

```sh
python3 tools/release/release.py package \
  --port-range 25440-25459 --output dist/artifacts
```

This builds for the native Rust host only. Supported archive targets are:

| Host | Archive contents | Signing state |
| --- | --- | --- |
| Apple Silicon macOS | `Openergy.app`, with its Rust daemon | Ad-hoc, unnotarized |
| Intel macOS | `Openergy.app`, with its Rust daemon | Ad-hoc, unnotarized |
| x86-64 Linux GNU | `openergy` daemon/CLI | Unsigned |
| x86-64 Windows MSVC | `openergy.exe` daemon/CLI | Unsigned |

The archive also contains an archive-specific README, `docs/releasing.md`,
`examples/server.toml`, `examples/client.toml`, project licenses, third-party
notices, and `BUILD.json`. The README links to installation instructions at the
exact source commit, so source-only relative links do not appear usable in a
binary download. Windows `.ps1`/`.cmd` installation and wrapper scripts are
copied together beside `openergy.exe`. macOS bundles carry licenses
and notices in `Contents/Resources` before signing. Every binary package requires
`LICENSE-MIT`, `LICENSE-APACHE`, and `THIRD_PARTY_NOTICES.md` to exist.
It runs the same notice freshness check as verification. The manifest records
whether source was dirty, including untracked files outside ignored build
directories. Untagged or dirty artifacts cannot pass the draft-release gate.

Packaging checks both the staged daemon and a fresh copy extracted from the
archive through real virtual IPC. It also verifies the extracted macOS code
signature. Each archive gets a JSON provenance sidecar and a SHA-256 sidecar.
The draft gate reads ZIP and gzip-compressed tar archives without extraction.
It requires exactly one `BUILD.json`, at the archive root, as a regular file
of at most 1 MiB. Its parsed JSON must equal the complete provenance sidecar;
JSON spacing and key order can differ. Duplicate, nested, and alternate manifest
paths fail this check. ZIP entries without Unix type bits remain supported.
Manifest aliases that differ only in letter case also fail, because extraction
on a case-insensitive filesystem can overwrite `BUILD.json` with that entry.
Verify the checksum with `shasum -a 256 -c <archive>.sha256` on macOS,
`sha256sum -c <archive>.sha256` on Linux, or `Get-FileHash -Algorithm SHA256`
on Windows. A checksum detects a changed download; it does not authenticate
its publisher.

Archives use sorted file order, fixed timestamps from `SOURCE_DATE_EPOCH` or
the source commit, fixed ownership for tar, and stable executable modes. Cargo
uses `--locked`; Swift does not resolve new dependencies. This makes the
process repeatable and its inputs visible. It is not a claim that compilers,
SDKs, ad-hoc signatures, or binaries from different checkout paths produce
identical bytes. The manifest records compiler versions and macOS Xcode/SDK
versions. Record the OS version too when reviewing a release.

Owned Unix release commands set and verify absolute nice 15. Nested packaging,
build and signing calls retain that value. If the calling process already has
a lower scheduling priority and cannot set nice 15, the command refuses before
executing its payload. Windows keeps its separate priority policy.

Packaging refuses to overwrite an existing artifact. Use a fresh output
directory. Temporary icon files, staged bundles, and extracted checks are
removed on success or error. Cargo and Swift build caches stay in their normal
repository build directories. Failed scenario runs retain their own diagnostic
directory, as documented by the external runner.

For a bundle without an archive:

```sh
PYTHON=python3 apps/macos/scripts/bundle.sh
# Optional universal local bundle: install both Rust macOS targets first.
UNIVERSAL=1 PYTHON=python3 DIST=dist/universal apps/macos/scripts/bundle.sh
```

The default bundle is ad-hoc signed. It never searches the keychain for an
identity. `OPENERGY_VERSION`, if set, must equal the workspace version.
`OPENERGY_BUILD` defaults to 1 and must be a positive integer. Select a fresh
`DIST` directory for each build; an existing app is never removed or replaced.

An operator who already has a suitable signing identity can explicitly set
`OPENERGY_ALLOW_SIGNING=1` and `OPENERGY_SIGN_IDENTITY` for a separate bundle
build. Hash, preference and ambiguous substring selectors also require
`OPENERGY_SIGN_PURPOSE=distribution` (secure network timestamp required) or
`OPENERGY_SIGN_PURPOSE=development` (offline). Full `Developer ID Application:`
names default to distribution; full `Apple Development:` names default to local
development. Conflicting or malformed purpose settings fail before compilation
or bundle changes. Leave the purpose unset for default ad-hoc builds.
`python3.11 tools/release/sparkle.py signing-policy` checks the shared policy
without identity lookup or signing. See the complete
[signing-purpose contract](macos-updates.md#bundle-configuration).
These scripts do not import/export certificates, unlock a keychain, or
create notarization credentials. The automated package and draft workflow
always forces ad-hoc macOS signing and performs no notarization or Windows
Authenticode signing. Optional signing secrets are not required for these checks.

Ad-hoc signing and `codesign --verify` prove bundle integrity, not publisher
trust or Gatekeeper acceptance. Before calling a macOS build trusted, an
operator must separately perform Developer ID signing, notarization, stapling,
and Gatekeeper checks on the exact final artifact. Windows publisher trust
requires separate Authenticode signing and verification. Neither state is
claimed by these draft artifacts. Do not disable system protections to make a
release check pass.

## Public launch acceptance

The maintainer chose to skip GitHub private vulnerability reporting setup on
2026-10-22 and resume hardening. Reporting setup is deferred and does not block
this work. The [security policy](../SECURITY.md) states that no private route is
verified and gives instructions for a non-sensitive contact request.

The maintainer also chose to keep native input and clipboard acceptance as a
release gate. Use a separate test account or machine with recovery input
available. Preserve the current Mac and Windows sharing connection. Record
missing native results as open gates; virtual checks cannot replace them.

## CI without automatic private-repository costs

`.github/workflows/verify.yml` skips every automatic push/PR job while the
repository is private. It does not queue absent self-hosted runners. In a
public repository, standard hosted Linux, macOS, and Windows runners run the
same local verification script. No larger or paid runner label is used.

Fork PR code runs only on disposable hosted workers with read-only repository
permissions and no retained checkout credentials. The workflow does not use
`pull_request_target`, share caches with release jobs, or pass signing secrets.
Do not attach local workers to untrusted automatic PR execution.

Both automatic and manual verification jobs use disposable standard hosted
workers only, and every job skips while the repository is private. There is
no self-hosted override. A dispatch can select a different workflow definition;
a check inside that definition cannot protect a persistent runner against a
modified definition. Removing persistent workers from both entrypoints closes
that boundary. Use the same local verification and packaging CLI while private.
No runner registration, visibility change, branch protection change, or hosted
run is performed by these files.

The labels are the standard public `ubuntu-24.04`, `macos-15` (ARM64),
`macos-15-intel` (Intel), and `windows-2025` workers. These workers are fresh VMs;
no larger/paid label or cross-job build cache is used. See GitHub's
[runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

## Manual draft release workflow

No workflow publishes releases or starts a release because a tag was pushed.
A maintainer must review the source, run the full suite and platform acceptance
checks, then create/push the version tag separately. The workflow treats the
reviewed default-branch history as its trust anchor. It depends on the normal
repository review controls for that branch; ancestry alone does not prove a
human reviewed a commit.

`Manual draft release artifacts` is public-only and `workflow_dispatch` only.
Run its reviewed default-branch definition, with a literal version tag such as
`v0.1.0`. Dispatches from other branches/tags are skipped. All jobs skip while
private, including manual runs, so they neither spend private hosted minutes
nor queue absent local workers. The local CLI below remains available.

The workflow has three separate jobs, each on a fresh hosted VM:

1. **Trusted validation, read token.** Check out `github.workflow_sha`, the
   immutable commit of the default-branch workflow. The trusted
   `trusted_release.py validate` command reads the selected source's Git blobs
   as data. It never imports/runs its Python, Cargo, Swift, or shell files.
   It requires a literal tag, matching local/remote tag object and peeled
   commit, candidate ancestry at or before the frozen workflow commit, and
   the workflow commit still on remote default-branch history. It checks
   workspace/crate/toolchain/app versions, all legal files, and the notice
   inventory's exact Cargo.lock hash. The resolved source SHA, tag-object
   SHA, tag and version become frozen validation-job outputs.
2. **Candidate builds, read token only.** Check out only that resolved SHA on
   the four disposable native workers. Assert OS/CPU architecture, then run
   the existing exact-tag/clean-checkout/version, notice freshness, native
   verification, and archive checks. Assert each archive's expected target
   and source before uploading it. Artifacts expire after one day. Candidate
   code has no repository write or signing token and no publisher workspace.
3. **Trusted publisher, write token.** Check out only the original trusted
   workflow SHA. This job never checks out or executes the candidate source,
   imports candidate modules, invokes Cargo, extracts archives, or runs any
   downloaded program or script. It reads artifact data only. The same
   validator rechecks frozen tag/source/version and reviewed ancestry, then
   requires all four targets, legal-notice provenance, valid checksums, and
   complete equality between each sidecar and its bounded, unique root
   `BUILD.json`. Case aliases, duplicate/nested manifests and special files
   remain errors. The publisher rejects unexpected files and symlinks in
   its download directory. It rechecks the remote tag object and peeled SHA
   after archive validation, immediately before `gh release create --draft
   --verify-tag --target <frozen-source>`. An existing release is refused.

Input values enter shell steps through environment variables and quoted
arguments. Job outputs contain validated version literals and full Git SHAs.
Actions are pinned by commit. The context fields are documented in GitHub's
[context reference](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts).
The publisher does not get a candidate-ref checkout even if the candidate
has an older or changed release helper.

Remote Git reads and release creation are separate server operations. They do
not lock the tag atomically. Keep release tags immutable under repository
controls, and stop/restart the run if a tag changes. Moving either the tag
object or its peeled commit before the final check fails the run. The release
notes identify the frozen source; later tag movement is outside this workflow.
No tag-protection rule or repository setting is changed by these scripts.

## Private/local draft path

The existing local command still requires the candidate checkout itself to be
clean, its exact tag to point to HEAD, all four platform archives, matching
versions/source/notices/signing metadata, checksums and embedded manifests:

```sh
python3 tools/release/release.py draft --tag v0.1.0 --artifacts /path/to/artifacts
```

This command is an explicit maintainer operation. Run it only from a reviewed
local source checkout; it is not the workflow publisher. It creates a draft
only and refuses an existing release. No successful remote mutation is needed
for local verification of its rejection gates.

For data-only verification from a separate clean trusted checkout, keep its
`origin` pointed at the intended repository and use:

```sh
python3 -I tools/release/trusted_release.py validate \
  --tag v0.1.0 --trusted-commit <trusted-workflow-sha> \
  --default-branch main --output /tmp/release-source.json
python3 -I tools/release/trusted_release.py publish \
  --tag v0.1.0 --trusted-commit <trusted-workflow-sha> --default-branch main \
  --source-commit <resolved-source-sha> --tag-object <resolved-tag-object-sha> \
  --version 0.1.0 --artifacts /path/to/artifacts --check-only
```

The second command performs every provenance/artifact gate and rereads the
remote tag, then exits without a GitHub mutation. Omitting `--check-only` is
an explicit draft creation operation for the workflow or a maintainer. It
uses trusted helper code while reading a non-HEAD candidate as Git data.
It never builds or executes the candidate. Neither command authenticates
compiled bytes or replaces native platform acceptance.

Draft notes keep the ad-hoc/unsigned/unnotarized limits and outstanding
acceptance checks. Actions storage quotas still apply to a manual public run;
short retention limits temporary storage. Publication is a separate maintainer
action. These changes do not create a tag, draft, release, hosted run, runner,
signing identity, or production deployment by themselves.
