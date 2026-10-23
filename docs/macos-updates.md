# macOS updates

Openergy packages the official Sparkle 2.10.0 binary framework. `Package.swift`
requires that exact version. Commit `Package.resolved` with dependency changes.
The executable links Sparkle through `@executable_path/../Frameworks`. The bundle
keeps the complete framework, its executable helpers, both XPC services, symlinks,
permissions, resources, and the upstream license in `Resources/Sparkle-LICENSE.txt`.

The intended public feed is
`https://openergy-updates.sergeserbinenko.com/appcast.xml`.
The dedicated Cloudflare bucket is `openergy-releases`. The release owner has
provisioned the bucket/domain; SSL issuance and live hosting proof are parent-owned.
The feed initially has zero releases until signed release gates clear. This local
tooling does not establish Developer ID trust, notarization, publication or a trusted
update installation. The existing trusted draft publisher and its release gates
remain separate.

## Bundle configuration

Use Python 3.11+, the pinned Rust 1.99.1 toolchain, Swift 6 and macOS 15+. Build
with two jobs at priority 15. Neither command installs or launches the GUI app
or starts a native daemon. The bundle script only reads the daemon version with
private config, IPC and TLS paths and discovery disabled.

```sh
# Resolve the real pinned package once. A private scratch path is optional.
nice -n 15 swift package --package-path apps/macos resolve

# Explicit unconfigured fixture: no override values and no require-updates flag.
env -u OPENERGY_UPDATE_FEED_URL -u OPENERGY_UPDATE_PUBLIC_KEY \
  OPENERGY_DISABLE_UPDATES=1 OPENERGY_REQUIRE_UPDATES=0 \
  PYTHON=python3.14 DIST=dist/local apps/macos/scripts/bundle.sh

# Validate the configured source defaults before a build.
OPENERGY_REQUIRE_UPDATES=1 python3.14 tools/release/sparkle.py config

# Distribution candidate: the source public feed/key defaults are used.
# Both Rust macOS targets must already be installed for the universal build.
OPENERGY_REQUIRE_UPDATES=1 \
UNIVERSAL=1 OPENERGY_BUILD=2 PYTHON=python3.14 DIST=dist/update-candidate \
  apps/macos/scripts/bundle.sh
```

`OPENERGY_UPDATE_FEED_URL` and `OPENERGY_UPDATE_PUBLIC_KEY` are public values.
Both must be absent to use source defaults or both valid for an override.
With both absent, a complete public `SUFeedURL`/`SUPublicEDKey` pair in the source
Info.plist supplies release-owner defaults through the same validation. The source
public key is `T6P8IZ5N6ivIZD3SdPRTOKWImB81Lfj2PFQqH4TMZ6o=`. Its private signing
key is held externally by the parent; it is never part of this repository.
Explicit empty values and partial configuration fail. A single environment
override cannot borrow its other value from source defaults. The URL must use
HTTPS, have a host, contain no userinfo or fragment, and use at most 2048 printable
ASCII characters (escape non-ASCII URL components). The key must be canonical
base64 for exactly 32 Ed25519 bytes. `OPENERGY_REQUIRE_UPDATES=1` requires a valid
configured pair, including source defaults. `OPENERGY_DISABLE_UPDATES=1` creates
an explicitly unconfigured local bundle. It cannot combine with require-updates
or environment update overrides, so malformed configuration cannot hide behind it.
The check runs before output directory creation or compilation. Finished unconfigured local
bundles omit `SUFeedURL` and `SUPublicEDKey` entirely and set
`SUEnableAutomaticChecks=false`; no empty key or substitution token is retained.

Configured bundles contain those exact public values and enable automatic
checks. `SUAllowsAutomaticUpdates=true` exposes Sparkle's standard user control;
`SUAutomaticallyUpdate=false` is the initial automatic-download preference.
Users can change it through the updater settings. There is no custom silent
installation or live relaunch policy. The app's updater lifecycle must gate
startup on valid bundle configuration and exclude snapshot/E2E modes. The parent
integration owns `SPUStandardUpdaterController`, settings, menu and updater E2E.

Use a fresh `DIST` each time. Staging is private to that destination and is removed
on success, error or interruption. An existing app is never replaced. To keep
compiler output private, set `CARGO_TARGET_DIR` and `SWIFT_SCRATCH_PATH`; resolve
with `swift package --scratch-path "$SWIFT_SCRATCH_PATH" resolve` first. The
default Swift scratch path is `apps/macos/.build`. Swift binary downloads remain
in its normal cache so another worktree can reuse the pinned dependency.

Default signing remains ad-hoc with hardened runtime, no timestamp and no trust
claim. A named identity requires the existing explicit `OPENERGY_ALLOW_SIGNING=1`
and `OPENERGY_SIGN_IDENTITY` selection. The script does not discover identities,
change Keychain state, add a library-validation exception, or notarize anything.
It signs Sparkle's inner code first, then the framework, daemon and outer app.
Downloader's upstream sandbox/network entitlements are retained. Signature,
architecture, runpath and framework-linkage checks run before the final move.
The original downloaded framework signature is checked before it is re-signed.
Library validation on ad-hoc builds can still block a real app launch; packaged
GUI/updater runtime acceptance is a separate gate and requires the parent flow.

## Local signed update archive

Use the new command for Sparkle updates. The existing `release.py package` creates
native draft archives with `BUILD.json`; its copy/archive/extract contract flattens
symlinks. Those archives are not Sparkle update payloads. This command does not
modify that publisher, create a GitHub release, upload, or change permissions.

The shared feed accepts **one universal archive**. The command checks arm64 and
x86_64 in the app, daemon, Sparkle framework and every helper/XPC executable. It
rejects native-only bundles. Do not place a native-only payload on this channel.

```sh
nice -n 15 python3.14 tools/release/sparkle.py archive \
  --app dist/update-candidate/Openergy.app \
  --version 0.1.0 --build 2 \
  --download-url https://openergy-updates.sergeserbinenko.com/openergy-0.1.0-2-macos-universal.zip \
  --private-key-file /private/external/location/sparkle-ed25519 \
  --output dist/sparkle-0.1.0-2
```

The key file must be outside Git, a regular file with mode 0600 or stricter, and
contain a base64 Sparkle seed (32 decoded bytes) or legacy key (96 bytes). The
command requires an explicit file; it never falls back to a Keychain key. It
passes only the filename to official `sign_update --ed-key-file`, captures tool
diagnostics to avoid leaking malformed key contents, and never logs or copies
private key material. Do not use `generate_keys` in this workflow: its default
behavior writes the login Keychain. Key provisioning and backup are release-owner
tasks; this repository does not create a production key.

`--version` and `--build` must exactly match the bundle. Increase the numeric
build for every update, even if the marketing version stays the same. The URL
must use HTTPS without userinfo, fragment or query and must end in the exact
generated filename. It is an intended download location, not a live network check.
The embedded valid `SUFeedURL` and `SUPublicEDKey` bind the channel and signer.
An unconfigured bundle, mismatched signing key, missing architecture, invalid
signature, wrong version/build, or existing output fails closed.

Outputs appear together in a fresh directory only after all checks pass:

- `openergy-VERSION-BUILD-macos-universal.zip`: `ditto` archive of the whole app,
  with framework symlinks and executable modes preserved. ZIP entry ordering and
  timestamps are then normalized before signing; resource fork data is retained.
- `appcast.xml`: one full-update item with exact version/build, macOS 15 minimum,
  HTTPS enclosure URL, byte length and EdDSA signature. The feed itself is also
  signed by Sparkle 2.10.0's official tool.
- `update.json`: public configuration, artifact/tool identities and output hashes.

The command extracts its own archive with `ditto`, compares files, symlink targets
and modes with the source, and verifies every nested signature again. It uses
official `sign_update --verify` for both archive and feed against the bundle's
public key. Sparkle has no public-key CLI option; verification uses its supported
legacy 96-byte container with a zero private half and the expected public half.
This verification-only container is public data and is removed with staging.

Outputs and XML metadata have stable names and ordering. The ZIP uses fixed 1980
container timestamps and omits access-time metadata. Repeating the command with
the same app and key produces the same archive, signed feed and manifest in the
verified local environment. There are no network-dependent timestamps or generated
delta updates. This is not a promise of identical compiler/signature/ZIP bytes
across machines or compression-library versions. Keep the signed feed
unchanged. Do not use `release.py draft` for these outputs: they have a different
contract and do not satisfy its four-platform provenance requirements.

Before public use, the release owner must supply the durable matching Ed25519
key, build/sign/notarize a universal app under the explicit identity policy,
verify Gatekeeper and real updater behavior, confirm increasing build numbers,
and publish the archive and signed feed at their configured HTTPS URLs. Preserve
all normal release provenance and approval gates. Local EdDSA signing proves a
key match; it does not establish Developer ID trust or permission to publish.

Official references (checked 2026-10-23): [Sparkle setup](https://sparkle-project.org/documentation/),
[programmatic setup](https://sparkle-project.org/documentation/programmatic-setup/),
[2.10.0 release](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0),
[exact package manifest](https://github.com/sparkle-project/Sparkle/blob/2.10.0/Package.swift),
[file-only signer](https://github.com/sparkle-project/Sparkle/blob/2.10.0/sign_update/main.swift),
and [key formats](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Secret.swift).
