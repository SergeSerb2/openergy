#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
if [[ -e /tmp/.X99-lock || -S /tmp/.X11-unix/X99 ]]; then
  echo 'Refusing to use an existing :99 display' >&2
  exit 2
fi
probe_dir=$(mktemp -d /tmp/oe-ser215-x11.XXXXXX)
chmod 700 "$probe_dir"
export XAUTHORITY="$probe_dir/Xauthority"
touch "$XAUTHORITY"
xauth -f "$XAUTHORITY" add :99 MIT-MAGIC-COOKIE-1 "$(mcookie)"
Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp -noreset -auth "$XAUTHORITY" > "$probe_dir/xvfb.log" 2>&1 &
probe_pid=$!
trap 'kill "$probe_pid" 2>/dev/null || true; wait "$probe_pid" 2>/dev/null || true; rm -rf "$probe_dir"' EXIT
export DISPLAY=:99 OPENERGY_X11_PROBE_ISOLATED=1
for attempt in {1..50}; do
 if xdpyinfo >/dev/null 2>&1; then break; fi
 kill -0 "$probe_pid"
 sleep 0.1
done
xdpyinfo >/dev/null
read -r probe_owner_pid < /tmp/.X99-lock
if [[ "$probe_owner_pid" != "$probe_pid" ]]; then
  echo 'Refusing display :99 because its lock does not name our Xvfb' >&2
  exit 2
fi
timeout --signal=TERM --kill-after=5s 120s target/release/examples/x11_probe language-group
