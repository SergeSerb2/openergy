#!/bin/bash
# Build only: never install, launch the app, change TCC, or select a keychain identity.
# PYTHON must be Python 3.11+. See docs/releasing.md for distribution limits.
# UNIVERSAL=1 builds arm64 + x86_64 (install both Rust targets first).
# OPENERGY_SIGN_IDENTITY defaults to '-' (ad-hoc); named identities require explicit opt-in.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP_SRC="$ROOT/apps/macos"
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=2
PYTHON="${PYTHON:-python3}"
owned() { "$PYTHON" "$ROOT/tools/release/priority.py" -- "$@"; }
[[ "$(uname -s)" == "Darwin" ]] || { echo "bundle.sh requires macOS" >&2; exit 1; }
# Validate signing intent and public updates before any builds or destination changes.
SPARKLE_TOOL="$ROOT/tools/release/sparkle.py"
"$PYTHON" "$SPARKLE_TOOL" signing-policy
"$PYTHON" "$SPARKLE_TOOL" config
VERSION="$("$PYTHON" "$ROOT/tools/release/release.py" version)"
MACOS_MIN="$("$PYTHON" -c 'import plistlib,sys; print(plistlib.load(open(sys.argv[1], "rb"))["LSMinimumSystemVersion"])' "$APP_SRC/Support/Info.plist")"
export MACOSX_DEPLOYMENT_TARGET="$MACOS_MIN"
BUILD_NUMBER="${OPENERGY_BUILD:-1}"
[[ "$BUILD_NUMBER" =~ ^[1-9][0-9]{0,8}$ ]] || { echo "OPENERGY_BUILD must be a positive integer (at most 9 digits)" >&2; exit 1; }
[[ "${UNIVERSAL:-0}" =~ ^[01]$ ]] || { echo "UNIVERSAL must be 0 or 1" >&2; exit 1; }
IDENTITY="${OPENERGY_SIGN_IDENTITY:--}"

DIST="${DIST:-$ROOT/dist}"
mkdir -p "$DIST"
DIST="$(cd "$DIST" && pwd -P)"
APP="$DIST/Openergy.app"
[[ ! -e "$APP" && ! -L "$APP" ]] || { echo "Output already exists; select a fresh DIST directory" >&2; exit 1; }
WORK="$(mktemp -d "$DIST/.openergy-bundle.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
STAGED_APP="$WORK/Openergy.app"

step() { printf '\n==> %s\n' "$*"; }

ARCHS=("$(uname -m)")
[[ "${UNIVERSAL:-0}" == "1" ]] && ARCHS=(arm64 x86_64)
TARGET_DIR="$(cd "$ROOT" && cargo metadata --locked --no-deps --format-version 1 | "$PYTHON" -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
step "Building the daemon (${ARCHS[*]})"
DAEMON_BINS=()
for arch in "${ARCHS[@]}"; do
  target="aarch64-apple-darwin"
  [[ "$arch" == "x86_64" ]] && target="x86_64-apple-darwin"
  (cd "$ROOT" && owned cargo build --release --locked --bin openergy --target "$target")
  DAEMON_BINS+=("$TARGET_DIR/$target/release/openergy")
done

step "Building the app (${ARCHS[*]})"
SWIFT_ARCH_FLAGS=()
for arch in "${ARCHS[@]}"; do SWIFT_ARCH_FLAGS+=(--arch "$arch"); done
SWIFT_SCRATCH_PATH="${SWIFT_SCRATCH_PATH:-$APP_SRC/.build}"
mkdir -p "$SWIFT_SCRATCH_PATH"
SWIFT_SCRATCH_PATH="$(cd "$SWIFT_SCRATCH_PATH" && pwd -P)"
SWIFT_FLAGS=(--scratch-path "$SWIFT_SCRATCH_PATH" -c release --jobs 2 --disable-automatic-resolution "${SWIFT_ARCH_FLAGS[@]}")
(cd "$APP_SRC" && owned swift build "${SWIFT_FLAGS[@]}")
APP_BIN="$(cd "$APP_SRC" && owned swift build "${SWIFT_FLAGS[@]}" --show-bin-path)/Openergy"

step "Assembling the bundle"
# Helpers avoids an Openergy/openergy collision on case-insensitive volumes.
mkdir -p "$STAGED_APP/Contents/MacOS" "$STAGED_APP/Contents/Helpers" "$STAGED_APP/Contents/Resources"
cp "$APP_BIN" "$STAGED_APP/Contents/MacOS/Openergy"
strip -x "$STAGED_APP/Contents/MacOS/Openergy"
if [[ ${#DAEMON_BINS[@]} -gt 1 ]]; then
  lipo -create "${DAEMON_BINS[@]}" -output "$STAGED_APP/Contents/Helpers/openergy"
else
  cp "${DAEMON_BINS[0]}" "$STAGED_APP/Contents/Helpers/openergy"
fi
for name in LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md; do
  if [[ -f "$ROOT/$name" ]]; then
    cp "$ROOT/$name" "$STAGED_APP/Contents/Resources/$name"
  else
    echo "Bundle requires $name; integrate the legal notices before building" >&2
    exit 1
  fi
done
ICONSET="$WORK/AppIcon.iconset"
owned swift "$APP_SRC/scripts/make-icon.swift" "$ICONSET" >/dev/null
iconutil -c icns "$ICONSET" -o "$STAGED_APP/Contents/Resources/AppIcon.icns"
"$PYTHON" "$SPARKLE_TOOL" configure-plist --template "$APP_SRC/Support/Info.plist" \
  --output "$STAGED_APP/Contents/Info.plist" --version "$VERSION" --build "$BUILD_NUMBER"
"$PYTHON" "$SPARKLE_TOOL" embed --app "$STAGED_APP" --scratch "$SWIFT_SCRATCH_PATH"
plutil -lint "$STAGED_APP/Contents/Info.plist"
printf 'APPL????' > "$STAGED_APP/Contents/PkgInfo"
for binary in "$STAGED_APP/Contents/MacOS/Openergy" "$STAGED_APP/Contents/Helpers/openergy"; do
  [[ -x "$binary" ]] || { echo "Missing bundle executable" >&2; exit 1; }
  for arch in "${ARCHS[@]}"; do lipo "$binary" -verify_arch "$arch"; done
done

step "Signing with the explicitly selected mode"
VERIFY_ARCH_FLAGS=()
for arch in "${ARCHS[@]}"; do VERIFY_ARCH_FLAGS+=(--arch "$arch"); done
"$PYTHON" "$SPARKLE_TOOL" sign --app "$STAGED_APP" "${VERIFY_ARCH_FLAGS[@]}"
# Version reads are isolated too. Never start the application or native backend here.
mkdir -p "$WORK/check/tls"
DAEMON_VERSION="$(OPENERGY_CONFIG="$WORK/check/config.toml" OPENERGY_CONFIG_DIR="$WORK/check" \
  OPENERGY_SOCKET="$WORK/check/control.sock" OPENERGY_TLS_DIR="$WORK/check/tls" \
  OPENERGY_ALLOW_PORT_24800=0 OPENERGY_DISABLE_DISCOVERY=1 \
  owned "$STAGED_APP/Contents/Helpers/openergy" version)"
[[ "$DAEMON_VERSION" == "openergy $VERSION" ]] || { echo "Bundled daemon version mismatch" >&2; exit 1; }
mv "$STAGED_APP" "$APP"
if [[ "$IDENTITY" == "-" ]]; then
  echo "Ad-hoc signed, unnotarized local build. This is not a trusted distribution build."
else
  echo "Explicit identity used. Signing alone does not establish notarization or Gatekeeper acceptance."
fi
step "Done"
du -sh "$APP" | awk '{print "Size: " $1}'
echo "$APP"
