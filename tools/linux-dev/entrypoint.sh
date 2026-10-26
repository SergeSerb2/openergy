#!/bin/bash
set -euo pipefail
export CARGO_BUILD_JOBS=2
Xvfb :99 -screen 0 1280x800x24 -noreset -nolisten tcp +extension XTEST +extension XInputExtension +extension RANDR +extension XFIXES >/tmp/xvfb.log 2>&1 &
trap 'kill $(jobs -pr) 2>/dev/null || true' EXIT
for _ in {1..100}; do xdpyinfo >/dev/null 2>&1 && break; sleep .05; done
xdpyinfo >/dev/null 2>&1 || { cat /tmp/xvfb.log >&2; exit 1; }
# Cargo.lock is generated in a disposable copy; the source mount stays read-only.
tar -C /src --exclude=./target --exclude=./.git -cf - . | tar -C /work -xf -
exec nice -n 15 "$@"
