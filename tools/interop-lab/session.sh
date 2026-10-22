#!/usr/bin/env bash
set -euo pipefail
umask 077
mkdir -p /lab /run/lab
chmod 700 /run/lab
rm -f /lab/*.pid /lab/*/peer.pid
Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp -noreset > /lab/xvfb.log 2>&1 &
xpid=$!
trap 'kill "$xpid" 2>/dev/null || true; wait "$xpid" 2>/dev/null || true' EXIT
trap 'exit 0' TERM INT
for ((i=0; i<100; i++)); do
  if xdpyinfo >/dev/null 2>&1; then
    xset s off -dpms >/dev/null 2>&1 || true
    touch /run/lab/ready
    wait "$xpid"
    exit $?
  fi
  sleep .1
done
cat /lab/xvfb.log >&2
exit 1
