#!/bin/sh
# Runs the focus lab against the real Openergy injector on a TEST Mac (it moves the pointer).
#   tools/focus-lab/run-openergy.sh legacy   # numbers off: shows the macOS 27 bug where it exists
#   tools/focus-lab/run-openergy.sh native   # numbered clicks: the fix
# Needs: dist/ from tools/focus-lab/build.sh and target/release/examples/focuslab_inject
# (cargo build --release -p openergy-platform-macos --examples), or set LAB_INJECTOR/LAB_DRIVER.
LAB_MODE="${1:?usage: run-openergy.sh legacy|native}"
HERE="$(cd "$(dirname "$0")" && pwd)"
LAB_INJECTOR="${LAB_INJECTOR:-$HERE/../../target/release/examples/focuslab_inject}"
LAB_DRIVER="${LAB_DRIVER:-$HERE/dist/focus-lab}"
LAB_REPORT_DIR="${LAB_REPORT_DIR:-$HERE/reports}"
LAB_RUN_DIR="$(mktemp -d /tmp/openergy-focus.XXXXXX)"
chmod 700 "$LAB_RUN_DIR"
LAB_INPUT_SOCKET="$LAB_RUN_DIR/input.sock"
mkdir -p "$LAB_REPORT_DIR"
sw_vers > "$LAB_REPORT_DIR/$LAB_MODE-os.txt"
"$LAB_INJECTOR" serve --socket "$LAB_INPUT_SOCKET" --click-numbering "$LAB_MODE" \
  --i-understand-this-moves-the-pointer > "$LAB_RUN_DIR/service.log" 2>&1 &
LAB_SERVICE_PID=$!
i=0
while [ $i -lt 100 ]; do
  grep -q '^ready ' "$LAB_RUN_DIR/service.log" && break
  kill -0 "$LAB_SERVICE_PID" 2>/dev/null || break
  sleep 0.05; i=$((i + 1))
done
if ! grep -q '^ready ' "$LAB_RUN_DIR/service.log"; then cat "$LAB_RUN_DIR/service.log"; exit 1; fi
"$LAB_DRIVER" run --injector cmd \
  --cmd-template "\"$LAB_INJECTOR\" --socket \"$LAB_INPUT_SOCKET\" --click-numbering $LAB_MODE {action} {button} {x} {y}" \
  --report "$LAB_REPORT_DIR/$LAB_MODE.json" --i-understand-this-moves-the-pointer
LAB_RESULT=$?
"$LAB_INJECTOR" --socket "$LAB_INPUT_SOCKET" stop >/dev/null 2>&1
wait "$LAB_SERVICE_PID" 2>/dev/null
cp "$LAB_RUN_DIR/service.log" "$LAB_REPORT_DIR/$LAB_MODE-service.log"
echo "Focus Lab exit ($LAB_MODE): $LAB_RESULT"
exit "$LAB_RESULT"
