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
`docs/verification/public-release-audit-2026-10-21/generate-notices.py --check`.
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

The archive also contains README, these release instructions, project licenses,
available third-party notices, and `BUILD.json`. macOS bundles carry licenses
and notices in `Contents/Resources` before signing. Every binary package requires
`LICENSE-MIT`, `LICENSE-APACHE`, and `THIRD_PARTY_NOTICES.md` to exist.
It runs the same notice freshness check as verification. The manifest records
whether source was dirty, including untracked files outside ignored build
directories. Untagged or dirty artifacts cannot pass the draft-release gate.

Packaging checks both the staged daemon and a fresh copy extracted from the
archive through real virtual IPC. It also verifies the extracted macOS code
signature. Each archive gets a JSON provenance sidecar and a SHA-256 sidecar.
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
build. These scripts do not import/export certificates, unlock a keychain, or
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

## CI without automatic private-repository costs

`.github/workflows/verify.yml` skips every automatic push/PR job while the
repository is private. It does not queue absent self-hosted runners. In a
public repository, standard hosted Linux, macOS, and Windows runners run the
same local verification script. No larger or paid runner label is used.

Fork PR code runs only on disposable hosted workers with read-only repository
permissions and no retained checkout credentials. The workflow does not use
`pull_request_target`, share caches with release jobs, or pass signing secrets.
Do not attach local workers to untrusted automatic PR execution.

A manual verification run can explicitly opt into `self_hosted`. This queues
workers with the `openergy` label and the platform/architecture labels in the
workflow. No runners were configured at the time this workflow was added.
Without them, use the local command above. These files do not register runners,
change repository visibility, change branch protection, or start deployments.
Keep any future local workers dedicated/disposable, with no live Openergy
state or signing credentials. The assigned port ranges must be exclusive to
each worker; use one job per host or provision separate isolated hosts.

## Manual draft release workflow

No workflow publishes releases. No workflow runs a release build because a
tag was pushed. A maintainer must first review the source, run the full suite
and platform acceptance gates, and create/push the intended version tag
separately. Tag creation and publication are outside these scripts.

`Manual draft release artifacts` is `workflow_dispatch` only. It requires
four native self-hosted workers labelled `openergy-release`, with the
OS/architecture labels shown in `.github/workflows/draft-release.yml`, plus
GitHub CLI on the Linux worker. This is a future opt-in path until those
workers are provisioned. Do not dispatch it expecting an absent runner to
be created. It never falls back to paid hosted jobs.

Pass an existing tag such as `v0.1.0`. The workflow:

1. Checks out that exact tag on each native worker, without stored credentials.
2. Requires the tag to match the workspace version and HEAD, a clean
   checkout, consistent toolchain/app metadata, and complete legal notices.
3. Runs the native verification script and creates the local archive, manifest,
   and checksums. macOS remains ad-hoc/unnotarized; other platforms remain unsigned.
4. Transfers build outputs with a one-day Actions artifact retention period.
5. Requires all four platforms, matching commit/tag/version/signing metadata,
   and valid checksums. Only this final job gets `contents: write`.
6. Calls `gh release create --draft --verify-tag` and uploads the checked files.
   It refuses an existing release and never overwrites or publishes one.

Draft notes state the signing limits and the acceptance still required.
Actions storage quotas still apply to a manual run; short retention limits
temporary storage. Review and later publication are separate maintainer
actions. This change does not create a tag, draft, release, hosted job, local
runner, signing identity, or production deployment by itself.
