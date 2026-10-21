#!/bin/bash
# Builds Openergy.app: the SwiftUI menu-bar app (Contents/MacOS/Openergy) plus the Rust
# `openergy` daemon and CLI (Contents/Helpers/openergy), both signed with one identity.
#
#   apps/macos/scripts/bundle.sh                 # release build into dist/Openergy.app
#   UNIVERSAL=1 apps/macos/scripts/bundle.sh     # arm64 + x86_64
#   OPENERGY_SIGN_IDENTITY="-" apps/macos/...    # force ad-hoc signing
#
# Signing: uses $OPENERGY_SIGN_IDENTITY, else the first "Developer ID Application" or
# "Apple Development" identity in the keychain, else ad-hoc. A stable identity keeps the
# macOS privacy permissions (Accessibility, Input Monitoring) across rebuilds.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP_SRC="$ROOT/apps/macos"
DIST="${DIST:-$ROOT/dist}"
APP="$DIST/Openergy.app"
VERSION="${OPENERGY_VERSION:-$(grep -m1 '^version' "$ROOT/Cargo.toml" 2>/dev/null | sed -E 's/.*"(.*)".*/\1/' || true)}"
VERSION="${VERSION:-0.1.0}"
BUILD_NUMBER="${OPENERGY_BUILD:-$(git -C "$ROOT" rev-list --count HEAD 2>/dev/null || echo 1)}"
export PATH="$HOME/.cargo/bin:$PATH"

step() { printf '\n==> %s\n' "$*"; }

ARCHS=("$(uname -m)")
[[ "${UNIVERSAL:-0}" == "1" ]] && ARCHS=(arm64 x86_64)

step "Building the openergy daemon (${ARCHS[*]})"
DAEMON_BINS=()
for arch in "${ARCHS[@]}"; do
  target="aarch64-apple-darwin"
  [[ "$arch" == "x86_64" ]] && target="x86_64-apple-darwin"
  (cd "$ROOT" && cargo build --release --locked --bin openergy --target "$target")
  DAEMON_BINS+=("$ROOT/target/$target/release/openergy")
done

step "Building the Openergy app (${ARCHS[*]})"
SWIFT_ARCH_FLAGS=()
for arch in "${ARCHS[@]}"; do SWIFT_ARCH_FLAGS+=(--arch "$arch"); done
(cd "$APP_SRC" && swift build -c release "${SWIFT_ARCH_FLAGS[@]}")
APP_BIN="$(cd "$APP_SRC" && swift build -c release "${SWIFT_ARCH_FLAGS[@]}" --show-bin-path)/Openergy"

step "Assembling $APP"
rm -rf "$APP"
# The daemon lives in Contents/Helpers: on the default case-insensitive file system,
# Contents/MacOS/openergy and Contents/MacOS/Openergy would be the same file.
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Helpers" "$APP/Contents/Resources"
cp "$APP_BIN" "$APP/Contents/MacOS/Openergy"
strip -x "$APP/Contents/MacOS/Openergy"   # local symbols only; about 60% smaller
if [[ ${#DAEMON_BINS[@]} -gt 1 ]]; then
  lipo -create "${DAEMON_BINS[@]}" -output "$APP/Contents/Helpers/openergy"
else
  cp "${DAEMON_BINS[0]}" "$APP/Contents/Helpers/openergy"
fi

ICONSET="$(mktemp -d)/AppIcon.iconset"
swift "$APP_SRC/scripts/make-icon.swift" "$ICONSET" >/dev/null
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"

sed -e "s/__VERSION__/$VERSION/g" -e "s/__BUILD__/$BUILD_NUMBER/g" \
  "$APP_SRC/Support/Info.plist" > "$APP/Contents/Info.plist"
printf 'APPL????' > "$APP/Contents/PkgInfo"

step "Signing"
IDENTITY="${OPENERGY_SIGN_IDENTITY:-}"
if [[ -z "$IDENTITY" ]]; then
  IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null \
    | grep -E '"(Developer ID Application|Apple Development):' | head -1 | sed -E 's/.*"(.*)"/\1/' || true)"
fi
IDENTITY="${IDENTITY:--}"
# Developer ID builds need a secure timestamp for notarization; local builds skip the network.
TIMESTAMP=--timestamp=none
[[ "$IDENTITY" == Developer\ ID\ Application:* ]] && TIMESTAMP=--timestamp
SIGN_FLAGS=(--force --options runtime "$TIMESTAMP" --sign "$IDENTITY")
codesign "${SIGN_FLAGS[@]}" --identifier dev.openergy.daemon "$APP/Contents/Helpers/openergy"
codesign "${SIGN_FLAGS[@]}" "$APP"
codesign --verify --deep --strict "$APP"
for binary in "$APP/Contents/MacOS/Openergy" "$APP/Contents/Helpers/openergy"; do
  [[ -x "$binary" ]] || { echo "missing $binary" >&2; exit 1; }
done
"$APP/Contents/Helpers/openergy" version
echo "Signed with: $([[ "$IDENTITY" == "-" ]] && echo ad-hoc || echo "$IDENTITY")"

step "Done"
du -sh "$APP" | awk '{print "Size: " $1}'
echo "$APP"
