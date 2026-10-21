# Separate macOS 27 test machine only

No live step below was run on the development Mac. Keep a local USB keyboard and
mouse available. Use a disposable logged-in desktop, without the user's live
Synergy connection, private clipboard, or unrelated input injectors. Never use
port 24800. Do not run synergy-core. Keep report files and exact OS builds.

## 1. Build and permission identity

Run from the merged repository on the separate test Mac, in its normal terminal:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
sw_vers
xcodebuild -version
rustc --version
cargo build --release
cargo build --release -p openergy-platform-macos --examples
tools/focus-lab/build.sh

LAB_ROOT="$(pwd)"
LAB_INJECTOR="$LAB_ROOT/target/release/examples/focuslab_inject"
LAB_PROBE="$LAB_ROOT/target/release/examples/mac_probe"
LAB_DRIVER="$LAB_ROOT/tools/focus-lab/dist/focus-lab"

codesign --force --sign - "$LAB_INJECTOR"
codesign --verify --strict "$LAB_INJECTOR"
"$LAB_PROBE" perms
"$LAB_DRIVER" doctor
"$LAB_INJECTOR" --help
```

Keep the exact injector executable path and signature stable after granting
permissions. In System Settings → Privacy & Security, grant **Accessibility**
to `focuslab_inject` or the terminal responsible process that macOS identifies.
Grant the driver its own Accessibility/Post Event access too. Restart the host
terminal/driver after changes and repeat both preflight checks. The service also
preflights its own Accessibility/Post Event access when it starts.

For capture later, grant **Input Monitoring** to the actual daemon/app responsible
process, not only `mac_probe` or the command wrapper. Grant the signed Openergy app
when it launches the daemon as its child. A terminal run can have a different TCC
identity. Record which executable/host appears in each Privacy pane and which
preflight grants change. Do not infer the minimal required grant set from the
fully granted development Mac.

On this test machine only, exercise the merged daemon's explicit permission IPC
with `kind` equal to `accessibility`, `input_monitoring`, and `post_events`.
Confirm there is no request during create, status polling, or dry probes. Revoke
and restore each grant in the test matrix. This closes the macOS 27 grant question.

## 2. Fresh service per Focus Lab run

Use at least 900×650 usable primary-display points. Keep the menu bar and `FLab`
item visible. Disable competing focus helpers on this test desktop. Do not supply
physical or remote input during a run. S7 must run first, with no earlier synthetic
down in that service. Do not reuse one service for the two modes.

The following shell block launches a fresh service, waits for its ready line,
runs the exact external-command protocol, then stops the service even if the lab
fails. Execute it once with `LAB_MODE=legacy` and once with `LAB_MODE=native`.

```sh
LAB_MODE=legacy   # repeat this whole block later with LAB_MODE=native
LAB_RUN_DIR="$(mktemp -d /tmp/openergy-focus.XXXXXX)"
chmod 700 "$LAB_RUN_DIR"
LAB_INPUT_SOCKET="$LAB_RUN_DIR/input.sock"
LAB_REPORT_DIR="$LAB_ROOT/crates/platform-macos/verification/test-mac"
mkdir -p "$LAB_REPORT_DIR"
sw_vers > "$LAB_REPORT_DIR/$LAB_MODE-os.txt"

"$LAB_INJECTOR" serve --socket "$LAB_INPUT_SOCKET" \
  --click-numbering "$LAB_MODE" --i-understand-this-moves-the-pointer \
  > "$LAB_RUN_DIR/service.log" 2>&1 &
LAB_SERVICE_PID=$!

# Stop owns the held ledger. SIGTERM also requests balanced cleanup.
trap '"$LAB_INJECTOR" --socket "$LAB_INPUT_SOCKET" stop >/dev/null 2>&1 || kill -TERM "$LAB_SERVICE_PID" 2>/dev/null; wait "$LAB_SERVICE_PID" 2>/dev/null' EXIT INT TERM

LAB_READY=0
for LAB_ATTEMPT in $(seq 1 100); do
  if rg -q '^ready ' "$LAB_RUN_DIR/service.log"; then LAB_READY=1; break; fi
  if ! kill -0 "$LAB_SERVICE_PID" 2>/dev/null; then break; fi
  sleep 0.05
done
if [ "$LAB_READY" != 1 ]; then
  cat "$LAB_RUN_DIR/service.log"
  exit 1
fi

"$LAB_DRIVER" run --injector cmd \
  --cmd-template "\"$LAB_INJECTOR\" --socket \"$LAB_INPUT_SOCKET\" --click-numbering $LAB_MODE {action} {button} {x} {y}" \
  --report "$LAB_REPORT_DIR/$LAB_MODE.json" \
  --i-understand-this-moves-the-pointer
LAB_RESULT=$?

"$LAB_INJECTOR" --socket "$LAB_INPUT_SOCKET" stop
wait "$LAB_SERVICE_PID"
trap - EXIT INT TERM
cp "$LAB_RUN_DIR/service.log" "$LAB_REPORT_DIR/$LAB_MODE-service.log"
printf 'Focus Lab exit: %s\n' "$LAB_RESULT"
```

Run this block in a fresh shell without `set -e`, so the expected legacy failure
still reaches cleanup. Keep the sibling `.evidence-*` directory beside each JSON
report. The service log must say the first synthetic down has not occurred at
ready time. `cmd` mode cannot inspect external event numbers; pair these OS results
with `mac_probe events-dry` from the same build, and inspect the first down in the
service with a test-machine event observer if independent field proof is needed.
Do not run a preparatory click against the service before S7.

Required outcomes (record observations even if legacy does not reproduce):

- **S7 first:** inactive A's first content click delivers the action, activates A,
  makes it key, and raises it above B in Native mode. No earlier service down.
- **S1:** the same content activation/action/window-order checks pass.
- **S2:** an inactive title-bar click activates A and makes it key/front.
- **S3:** ten title-bar drag steps move A (+120,+80) global points, within ±2.
- **S4:** status-menu item action occurs, the menu closes, and A stays inactive.
  Record whether geometry put the item over A. Also repeat with CodexBar, standard
  NSMenu rows and relevant popovers; custom fixture rows do not cover all menus.
- **S5:** outside-menu click closes the menu; no button action goes through it.
  Record activation but do not assert either activation outcome.
- **S6:** the fixture logs a mouse-down with clickCount 2.

Native must pass all seven. On an affected macOS 27 build, Legacy should fail
S1/S7 activation while still delivering the control action. If Legacy passes,
retain the report and exact OS build: that machine did not reproduce the defect.
Also compare macOS 14–26 where available. Never call event field readback alone an
activation fix.

## 3. Native backend matrix

Use the real integrated daemon/app and normal input on this separate test Mac:

1. Compare left/right/middle/additional buttons; double/triple clicks; held-button
   drags; simultaneous buttons; physical clicks between synthetic downs; a new
   connection before the first synthetic down. Check menu/modal/nonactivating-panel
   routing without AX raise or app-activation helpers.
2. Enter at CINN coordinates and click before any DMMV. Check that the click uses
   the entry point. Repeat leave/enter and cursor hide nesting. Network loss during
   a key hold and a drag must release both, restore association, and show the cursor.
3. Type the keymap sample in a disposable editor using en/fr/de layouts: letters,
   Shift, Caps, Control, Command+C, both modifier sides, accents, dead-key prefixes,
   repeats and unmapped Unicode. Change layout while holding a key, then release.
   Keycode CGEvents' default Unicode field can reflect physical source state;
   verify actual text and shortcuts, not just the advisory Unicode readback.
4. Start the tap; verify absolute and captured relative movement, all modifiers,
   fractional trackpad/wheel movement and self-event filtering. Stop immediately
   after start repeatedly. Revoke Input Monitoring/Accessibility during capture.
   Re-enable storms must stop after six disable notices inside ten seconds, emit
   CapturePermissionProblem, leave safely, and avoid a CPU spin.
5. Use a disposable clipboard: UTF-8, CR/LF text, HTML, PNG, TIFF, alpha DIB, a near
   64 MiB payload, malformed/unsupported DIB and an over-cap image. Confirm useful
   errors, own-write loop suppression, and another app reading PNG/TIFF. There is
   no promise of rollback if final NSPasteboard replacement fails. Unsupported
   DIB variants must fail explicitly; capture real Windows compatibility fixtures.
6. Test negative-origin/Retina/rotated displays and hot-plug. Verify one debounced
   geometry callback after 250 ms quiet, updated union/DINF, and correct cursor
   points. Sleep/wake and lock/unlock must reach the io thread, leave/release, and
   recheck grants/geometry before resume. Secure input must report enabled and
   `unknown` when a public owner name is unavailable.
7. With `prevent_sleep` enabled, inspect `pmset -g assertions`: the Openergy
   assertion exists only during remote-active use and is gone after leave,
   disconnect, sleep/lock handling and normal shutdown. Do not create assertions
   merely to poll status.
8. Measure idle RSS/CPU and input latency on the integrated daemon. The safe local
   probes do not establish the <30 MB, ~0% or <1 ms targets.

## 4. Daemon smoke after WP3/WP6 integration

Do not mistake Step 0's `platform: macos` status string for a running backend.
First confirm daemon code constructs this backend, uses the shared Backend type,
retains lifecycle notices, and dispatches the permission trait override. Those
integration points are listed in README.md. If they are absent, record BLOCKED;
do not count the following commands as native runtime proof.

On the separate test Mac only:

```sh
LAB_DAEMON_DIR="$(mktemp -d /tmp/openergy-daemon.XXXXXX)"
cat > "$LAB_DAEMON_DIR/config.toml" <<'TOML'
version = 1
role = "off"
name = "mac-test"
[server]
listen = "127.0.0.1:27185"
port = 27185
heartbeat_ms = 3000
[client]
remote = "127.0.0.1:27186"
autoconnect = false
heartbeat_ms = 3000
[clipboard]
enabled = false
send = false
receive = false
max_bytes = 67108864
[macos]
click_numbering = "native"
TOML

./target/release/openergy daemon --platform macos --foreground \
  --config "$LAB_DAEMON_DIR/config.toml" --socket "$LAB_DAEMON_DIR/daemon.sock" \
  > "$LAB_DAEMON_DIR/daemon.log" 2>&1 &
LAB_DAEMON_PID=$!
# Wait for its socket, then require real permissions/geometry in status.
for LAB_ATTEMPT in $(seq 1 100); do
  [ -S "$LAB_DAEMON_DIR/daemon.sock" ] && break
  kill -0 "$LAB_DAEMON_PID" 2>/dev/null || break
  sleep 0.05
done
./target/release/openergy status --socket "$LAB_DAEMON_DIR/daemon.sock"
./target/release/openergy ctl permissions.request --socket "$LAB_DAEMON_DIR/daemon.sock" --json '{"kind":"accessibility"}'
./target/release/openergy ctl permissions.request --socket "$LAB_DAEMON_DIR/daemon.sock" --json '{"kind":"input_monitoring"}'
./target/release/openergy ctl permissions.request --socket "$LAB_DAEMON_DIR/daemon.sock" --json '{"kind":"post_events"}'
./target/release/openergy ctl config.patch --socket "$LAB_DAEMON_DIR/daemon.sock" --json '{"patch":{"role":"server"}}'
# On the test desktop, check physical capture and all lifecycle cases above.
./target/release/openergy ctl config.patch --socket "$LAB_DAEMON_DIR/daemon.sock" --json '{"patch":{"role":"off"}}'
./target/release/openergy ctl daemon.stop --socket "$LAB_DAEMON_DIR/daemon.sock"
wait "$LAB_DAEMON_PID"
```

Then connect to a second disposable Openergy peer on loopback port 27186 or a
separately authorized test host. Use WP3's merged `VIRTUAL.md` and WP7's merged
scenario interface for the virtual peer commands; their JSON payload contract is
not in this checkout. Exercise CINN, keys, drags, clipboard, forced network loss,
reconnect and normal shutdown through the real daemon. Retain status/log evidence
without logging key identities or clipboard payloads. The pending group/modifier
contract gaps must be resolved before claiming complete Synergy language sync.
