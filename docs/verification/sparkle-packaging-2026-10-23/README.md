# Sparkle package and local release evidence

Verified 2026-10-23 in an isolated worktree from
`41ad7c70fbc95d22c2e11c7f10e0790ea7369067`. `results.json` records source hashes,
official dependency identities, actual bundle architectures/configuration,
artifact hashes, command failure results and cleanup. The Cloudflare URL below
was an intended destination only. No network feed or bucket was published here.

The real native and universal Rust/Swift builds passed. The complete embedded
Sparkle framework retained its license, symlinks, helpers and XPC services.
Original and re-signed nested code passed all-architecture checks. The main app
and daemon have macOS 15.0 load-command minimums. Every embedded Sparkle executable
requires macOS 15 or earlier. The app links the bundled framework by `@rpath` and
has `@executable_path/../Frameworks` as a runpath. Both final bundles and archive
extraction lived outside the SwiftPM scratch tree. These were actual artifacts;
the app GUI and updater were not launched.

Actual commands used Python 3.14, pinned Rust 1.99.1, two jobs and priority 15:

```sh
W=/tmp/openergy-sparkle-packaging-20261023
PY=/opt/homebrew/bin/python3.14

nice -n 15 swift package --package-path apps/macos --scratch-path "$W/swift" resolve

env -u OPENERGY_UPDATE_FEED_URL -u OPENERGY_UPDATE_PUBLIC_KEY \
  OPENERGY_DISABLE_UPDATES=1 OPENERGY_REQUIRE_UPDATES=0 CARGO_TARGET_DIR="$W/rust" \
  SWIFT_SCRATCH_PATH="$W/swift" PYTHON="$PY" DIST="$W/explicit-disabled" \
  bash apps/macos/scripts/bundle.sh

OPENERGY_REQUIRE_UPDATES=1 \
OPENERGY_UPDATE_FEED_URL=https://openergy-updates.sergeserbinenko.com/appcast.xml \
OPENERGY_UPDATE_PUBLIC_KEY="$(cat "$W/fixture-public.txt")" \
UNIVERSAL=1 OPENERGY_BUILD=2 CARGO_TARGET_DIR="$W/rust" \
SWIFT_SCRATCH_PATH="$W/swift" PYTHON="$PY" DIST="$W/configured-cloudflare" \
  bash apps/macos/scripts/bundle.sh

"$PY" tools/release/sparkle.py verify \
  --app "$W/configured-cloudflare/Openergy.app" --arch arm64 --arch x86_64

"$PY" tools/release/sparkle.py archive \
  --app "$W/configured-cloudflare/Openergy.app" --version 0.1.0 --build 2 \
  --download-url https://openergy-updates.sergeserbinenko.com/openergy-0.1.0-2-macos-universal.zip \
  --private-key-file "$W/disposable-ed25519" --scratch "$W/swift" \
  --output "$W/cloudflare-update"
# Repeated with --output "$W/cloudflare-update-repeat". All three files matched.

"$PY" tools/release/release.py check
bash -n apps/macos/scripts/bundle.sh
git diff --check
```

The fixture key came from `Curve25519.Signing.PrivateKey()` in a temporary Swift
CLI invocation using CryptoKit. It wrote only a base64 seed file (mode 0600) and
a public-key file within the private work directory. A second disposable key
exercised signer mismatch. Neither used `generate_keys` or the Keychain. Both
private files were deleted after verification. No private key or binary/archive
is committed. The public fixture key and signed output hashes are retained in
`results.json`; they are not production defaults.

A real universal bundle also passed with an ephemeral source-default pair before
the production public key was available. That fixture pair was removed. The
release owner then supplied the real PUBLIC feed/key pair, which is now in the
source Info.plist. Another actual universal build passed with both environment
overrides absent and `OPENERGY_REQUIRE_UPDATES=1`. Its finished bundle contained
the exact supplied production public values. No production private key was used
or received. A final configured fixture build confirmed that an explicit complete
environment pair overrides the source production defaults.

The final explicit-disable build uses `OPENERGY_DISABLE_UPDATES=1` and no override
values. It omits both feed/key entries and disables checks. Eleven invalid
configuration paths fail before compilation, including disable/require conflicts,
an invalid disable flag and disable with overrides. Absent overrides now preserve
the real configured production defaults.

The archive command ran the real official 2.10.0 `sign_update` tool to sign the
ZIP and feed, then verify both against the public key embedded in the bundle.
Its `ditto` extraction preserved every file, mode and symlink and passed nested
signature checks again. ZIP container metadata is normalized before signing.
An initial repeat exposed `ditto` access-time differences; after normalization,
the ZIP, signed XML and JSON repeated byte for byte for the same app/key.

Real `bundle.sh` failures covered partial URL-only configuration, explicit empty
values, HTTP, URL userinfo, a fragment, an overlong URL, malformed base64, wrong
public-key size and the disable conflicts described above. Every call used an
existing destination sentinel and absent private compiler directories. Each
failed before compilation, preserved the sentinel and created no staging or
compiler directory. No invalid value is treated as permission to use a fallback.

Five real `sparkle.py archive` failures covered an incorrect build, a different
file-only signing key, existing output, an unconfigured bundle and a native-only
app. The native-only fixture used the actual built universal app/daemon, thinned
to arm64 and re-signed through `sparkle.py sign`, before calling the full archive
command. Every failure preserved its destination and removed staging. The
different-key failure was repeated after ZIP normalization and failed official
signature verification against the expected public key.

Initial packaging checks also exposed universal `otool` file headers and this
toolchain's multi-architecture `lipo -verify_arch` parsing. The final command
reads dependency lines only and checks each architecture separately. All final
flows passed. Xcode emitted an x86_64 deprecation warning naming macOS 27; actual
Mach-O minimum versions were checked as described above.

Limits: these artifacts are ad-hoc signed, unnotarized local fixtures. No GUI
loading, updater installation, trusted relaunch, production signing key, live
HTTPS feed, Developer ID, Gatekeeper or notarization result is claimed. No
library-validation exception was added. The parent owns runtime/updater E2E,
live hosting proof, root notice entries, and the existing
`release.py` copy/ZIP symlink repair. Installed sharing, TCC, login items,
services, credentials and settings were not changed.
