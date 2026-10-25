#!/bin/bash
# Private dependency paths only. Keep the normal SwiftPM build engine and sandbox.
set -euo pipefail
case "${1:-}" in
  build|package)
    action="$1"; shift
    exec "$M1_SWIFT" "$action" \
      --cache-path "$M1_SWIFT_CACHE" --config-path "$M1_SWIFT_CONFIG" \
      --security-path "$M1_SWIFT_SECURITY" --disable-keychain --disable-netrc "$@"
    ;;
  *) exec "$M1_SWIFT" "$@" ;;
esac
