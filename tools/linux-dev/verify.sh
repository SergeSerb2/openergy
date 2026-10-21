#!/bin/bash
set -euo pipefail
export PATH=/usr/local/cargo/bin:$PATH
export OPENERGY_X11_PROBE_ISOLATED=1 LANG=C.UTF-8
python3 /work/tools/linux-dev/fixtures.py
case ${1:-all} in
 check) cargo build --release -p openergy-platform-linux -p openergy-platform-windows; cargo clippy -p openergy-platform-linux --all-targets -- -D warnings ;;
 *) cargo run --release -p openergy-platform-linux --example x11_probe -- "${1:-all}"
    if [[ ${1:-all} == all || ${1:-all} == clipboard ]]; then python3 /work/tools/linux-dev/verify-pixels.py; fi ;;
esac
