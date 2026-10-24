# macOS updates

Openergy packages the official Sparkle 2.10.0 binary framework. `Package.swift`
requires that exact version. Commit `Package.resolved` with dependency changes.
The executable links Sparkle through `@executable_path/../Frameworks`. The bundle
keeps the complete framework, its executable helpers, both XPC services, symlinks,
permissions, resources, and the upstream license in `Resources/Sparkle-LICENSE.txt`.

The public feed is
`https://openergy-updates.sergeserbinenko.com/appcast.xml`.
The dedicated Cloudflare R2 bucket is `openergy-releases`. Its custom domain
serves the signed empty feed over HTTPS. The channel page is
`https://openergy-updates.sergeserbinenko.com/index.html`.
The feed has zero releases until release gates clear. Hosting the feed does not
establish Developer ID trust, notarization or a trusted update installation.
The existing trusted draft publisher and its release gates remain separate.

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
  PYTHON=python3.11 DIST=dist/local apps/macos/scripts/bundle.sh

# Validate the configured source defaults before a build.
OPENERGY_REQUIRE_UPDATES=1 python3.11 tools/release/sparkle.py config

# Universal ad-hoc candidate: the source public feed/key defaults are used.
# Both Rust macOS targets must already be installed for the universal build.
OPENERGY_REQUIRE_UPDATES=1 \
UNIVERSAL=1 OPENERGY_BUILD=2 PYTHON=python3.11 DIST=dist/update-candidate \
  apps/macos/scripts/bundle.sh
```

`OPENERGY_UPDATE_FEED_URL` and `OPENERGY_UPDATE_PUBLIC_KEY` are public values.
Both must be absent to use source defaults or both valid for an override.
With both absent, a complete public `SUFeedURL`/`SUPublicEDKey` pair in the source
Info.plist supplies release-owner defaults through the same validation. The source
public key is `T6P8IZ5N6ivIZD3SdPRTOKWImB81Lfj2PFQqH4TMZ6o=`. Its private signing
key is held in `~/.config/openergy-release/sparkle-ed25519.key` with mode 0600;
it is never part of this repository. Keep a private backup of that key. Future
updates must use the matching key.
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
startup on valid bundle configuration and exclude snapshot/E2E modes. One retained
`SPUStandardUpdaterController` owns the normal app updater. Check for Updates is
available in the app menu, menu-bar panel and Advanced settings. Advanced settings
also expose Sparkle's automatic-check and automatic-download preferences.

Use a fresh `DIST` each time. Staging is private to that destination and is removed
on success, error or interruption. An existing app is never replaced. To keep
compiler output private, set `CARGO_TARGET_DIR` and `SWIFT_SCRATCH_PATH`; resolve
with `swift package --scratch-path "$SWIFT_SCRATCH_PATH" resolve` first. The
default Swift scratch path is `apps/macos/.build`. Swift binary downloads remain
in its normal cache so another worktree can reuse the pinned dependency.

Default local signing is ad-hoc without Hardened Runtime, with no timestamp or
trust claim. An ad-hoc app has no Team ID and cannot load Sparkle through
Hardened Runtime's library validation. A named identity retains Hardened Runtime
and requires the existing explicit `OPENERGY_ALLOW_SIGNING=1`
and `OPENERGY_SIGN_IDENTITY` selection. Both the bundler preflight and direct
`sparkle.py sign` command use this signing-purpose policy:

- With the identity unset or `-`, leave `OPENERGY_SIGN_PURPOSE` unset. Signing
  uses `--options 0 --timestamp=none` and stays offline.
- A full name starting with `Developer ID Application:` defaults to
  `distribution`. A full name starting with `Apple Development:` defaults to
  local `development`. Each prefix must have a nonempty name after it.
- For a certificate hash, an existing identity preference, or another common-name
  substring, explicitly set `OPENERGY_SIGN_PURPOSE=distribution` or
  `OPENERGY_SIGN_PURPOSE=development`. The selector passes unchanged to `codesign`.
- Distribution always uses `--options runtime --timestamp`, whatever selector
  form you choose. It needs the secure timestamp service and network access.
  A service failure fails signing; there is no retry without a timestamp.
- Development uses `--options runtime --timestamp=none` and stays offline.
  This includes explicitly chosen ambiguous local development selectors.

An absent purpose for an ambiguous selector, an empty or unknown purpose, a
purpose with ad-hoc signing, or a purpose that conflicts with a known certificate
prefix fails before compilation, signing or bundle changes. Empty selectors,
surrounding whitespace and control characters also fail. This validates the
operator's intent; it does not prove which certificate a selector resolves to,
private-key access, timestamp availability or distribution trust.

Check this policy without building, signing, querying identities or making a
timestamp request. `RELEASE_SIGN_SELECTOR` and `LOCAL_SIGN_SELECTOR` below mean
operator-supplied selectors for separately authorized identities; no identity
value is supplied by this example:

```sh
# Distribution: secure timestamp required, including when the selector is a hash.
OPENERGY_ALLOW_SIGNING=1 OPENERGY_SIGN_IDENTITY="$RELEASE_SIGN_SELECTOR" \
  OPENERGY_SIGN_PURPOSE=distribution python3.11 tools/release/sparkle.py signing-policy
# Local development: no network timestamp.
OPENERGY_ALLOW_SIGNING=1 OPENERGY_SIGN_IDENTITY="$LOCAL_SIGN_SELECTOR" \
  OPENERGY_SIGN_PURPOSE=development python3.11 tools/release/sparkle.py signing-policy
```

Use the same environment with `apps/macos/scripts/bundle.sh`, or with
`python3.11 tools/release/sparkle.py sign --app "$OWNED_APP" --arch arm64 --arch x86_64`
for an already assembled universal private bundle. Those commands perform real
signing with the selected key. Distribution also contacts the timestamp service.
The script does not discover identities,
change Keychain state, add a library-validation exception, or notarize anything.
It signs Sparkle's inner code first, then the framework, daemon and outer app.
Existing nested entitlement metadata is retained. Signature,
architecture, runpath and framework-linkage checks run before the final move.
The original downloaded framework signature is checked before it is re-signed.
No build receives a library-validation exception entitlement. Packaged runtime,
trusted signing, notarization and installed update acceptance are separate gates.

## Local signed update archive

Use the new command for Sparkle updates. The existing `release.py package` creates
native draft archives with `BUILD.json`; their copy/archive/extract path preserves
and validates the framework's symlinks. Those archives are not Sparkle update
payloads. This command does not
modify that publisher, create a GitHub release, upload, or change permissions.

The shared feed accepts **one universal archive**. The command checks arm64 and
x86_64 in the app, daemon, Sparkle framework and every helper/XPC executable. It
rejects native-only bundles. Do not place a native-only payload on this channel.

```sh
nice -n 15 python3.11 tools/release/sparkle.py archive \
  --app dist/update-candidate/Openergy.app \
  --version 0.1.0 --build 2 \
  --download-url https://openergy-updates.sergeserbinenko.com/releases/openergy-0.1.0-2-macos-universal.zip \
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

## Cloudflare channel

`deployment/cloudflare/updates.json` pins the dedicated bucket, hostname and
Wrangler version. `tools/release/deploy_updates.py bootstrap` creates an unused
channel with the signed empty feed and channel page. It refuses existing DNS
records or an unexpected existing bucket. Repeating it against the configured
channel preserves the existing feed. It never uploads an application archive.
Bootstrap requires the existing `CLOUDFLARE_API_TOKEN` environment variable.

Read-only verification needs no Cloudflare credential:

```sh
python3.11 tools/release/deploy_updates.py verify --report /tmp/openergy-feed-check.json
```

Select a fresh report path. If local name lookup is unavailable, `verify` accepts
`--resolve-ip ADDRESS` after checking that address against public Cloudflare DNS.
This changes only that HTTPS request. It retains hostname, SNI and certificate
validation. It does not change system DNS. Use the ordinary URL for application
configuration.

After release acceptance, upload the signed universal ZIP under its exact
`releases/` object name, then publish the matching signed `appcast.xml` last.
Preserve its bytes and signature. Keep archive URLs immutable. The current
bootstrap feed deliberately offers no application update.


Official references (checked 2026-10-23): [Sparkle setup](https://sparkle-project.org/documentation/),
[programmatic setup](https://sparkle-project.org/documentation/programmatic-setup/),
[2.10.0 release](https://github.com/sparkle-project/Sparkle/releases/tag/2.10.0),
[exact package manifest](https://github.com/sparkle-project/Sparkle/blob/2.10.0/Package.swift),
[file-only signer](https://github.com/sparkle-project/Sparkle/blob/2.10.0/sign_update/main.swift),
and [key formats](https://github.com/sparkle-project/Sparkle/blob/2.10.0/common_cli/Secret.swift).
