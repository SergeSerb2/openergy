#!/bin/bash
set -euo pipefail

lab_dir="$(cd "$(dirname "$0")" && pwd)"
cd "$lab_dir"

# All outputs stay inside this lab. No fixture is launched by this script.
export CLANG_MODULE_CACHE_PATH="$lab_dir/.build/clang-module-cache"
swift build -c release --scratch-path "$lab_dir/.build" -Xswiftc -warnings-as-errors
bin_dir="$(swift build -c release --scratch-path "$lab_dir/.build" --show-bin-path)"
dist_dir="$lab_dir/dist"
mkdir -p "$dist_dir"

make_app() {
    local name="$1" identifier="$2" binary="$3" accessory="$4"
    local bundle="$dist_dir/$name.app"
    mkdir -p "$bundle/Contents/MacOS"
    cp "$bin_dir/$binary" "$bundle/Contents/MacOS/$binary"
    cat > "$bundle/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>$binary</string>
  <key>CFBundleIdentifier</key><string>$identifier</string>
  <key>CFBundleName</key><string>$name</string>
  <key>CFBundleDisplayName</key><string>$name</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>LSMinimumSystemVersion</key><string>14.0</string>
  <key>LSUIElement</key><$accessory/>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST
    /usr/bin/plutil -lint "$bundle/Contents/Info.plist"
    /usr/bin/codesign -s - --force "$bundle"
    /usr/bin/codesign --verify --deep --strict --verbose=2 "$bundle"
}

make_app "FocusLab A" "dev.openergy.focuslab.a" "FocusLabFixture" false
make_app "FocusLab B" "dev.openergy.focuslab.b" "FocusLabFixture" false
make_app "FocusLabMenuBar" "dev.openergy.focuslab.menubar" "FocusLabMenuBar" true
cp "$bin_dir/focus-lab" "$dist_dir/focus-lab"
/usr/bin/codesign -s - --force --identifier dev.openergy.focuslab.driver "$dist_dir/focus-lab"
/usr/bin/codesign --verify --strict --verbose=2 "$dist_dir/focus-lab"
printf 'Built and verified: %s\n' "$dist_dir"
printf 'Safe commands: dist/focus-lab --help | doctor | run --dry-run\n'
