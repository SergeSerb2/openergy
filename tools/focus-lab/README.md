# Focus Lab

A macOS 14+ SwiftPM verification lab for synthetic mouse activation. It has no third-party dependencies or unit tests. It uses Swift 6 language mode and strict concurrency. AppKit and mutable lab state stay on the main actor.

**Run live scenarios only on a separate test Mac.** A live run moves the pointer, changes focus, opens windows and menus, and posts mouse events. Do not run it on the workstation that you currently control through Synergy. `build.sh`, `--help`, `doctor`, and `run --dry-run` do not launch fixture apps or post events.

## Build

From the repository root:

```sh
tools/focus-lab/build.sh
tools/focus-lab/dist/focus-lab --help
tools/focus-lab/dist/focus-lab doctor
tools/focus-lab/dist/focus-lab run --dry-run
```

The script runs `swift build -c release` with warnings treated as errors. It assembles and ad-hoc signs these artifacts in `tools/focus-lab/dist/`, then runs `codesign --verify`:

- `focus-lab`: driver.
- `FocusLab A.app`: `dev.openergy.focuslab.a`.
- `FocusLab B.app`: `dev.openergy.focuslab.b`.
- `FocusLabMenuBar.app`: `dev.openergy.focuslab.menubar`, an `LSUIElement` accessory app.

A and B contain the same `FocusLabFixture` binary. Each opens one standard window, with a large button and a text field. Blue and orange distinguish the windows. They accept `--title`, `--frame x,y,width,height` (outer window frame in CG coordinates), `--color blue|orange|red|green` or a quoted `'#RRGGBB'`, `--log <path>`, and `--commands <path>`. The last two options are required. Minimum frame size is 400×300 points.

## Copy and permissions on the test Mac

Copy the whole `dist` directory. Keep the driver beside the three bundles. The build uses the current host architecture; build on the test Mac for a different CPU architecture. A Swift 6 toolchain is required to build. There is no installation step or package dependency download.

```sh
ditto tools/focus-lab/dist /path/to/transfer/FocusLab
# On the separate macOS 27 test Mac:
cd /path/to/FocusLab
./focus-lab doctor
```

In System Settings → Privacy & Security, grant **Accessibility** to the executable or terminal host that macOS identifies for this run. Grant **Input Monitoring** if macOS reports it missing and the test setup needs it. Restart that terminal/driver after a permission change, then run `doctor` again. `doctor` calls `AXIsProcessTrustedWithOptions` with prompting disabled, `CGPreflightPostEventAccess`, and `CGPreflightListenEventAccess`. It prints all three checks. It never installs an event tap or asks for permission. Native live runs require Accessibility and Post Event access; Input Monitoring is recorded but is not required because the driver does not listen to global input. An external injector needs its own permissions.

An ad-hoc signature is not notarization. Handle Gatekeeper or transfer quarantine through the test Mac's normal approval UI if it blocks execution. Rebuilding may change the permission identity. Run `doctor` after every copy or rebuild. Screen Recording is not needed: the lab uses its own AppKit geometry and public window-order metadata, not screenshots.

Use a logged-in, unlocked desktop with at least 900×650 usable points on the primary display. Keep the menu bar visible, leave room for the `FLab` item, and keep unrelated windows, notifications, Stage Manager, and full-screen Spaces out of the test area. Do not provide physical or remote input during a run. On the test Mac, disable the old focus workaround and competing injectors before testing; otherwise they can hide failures or steal focus. This lab never finds, changes, or terminates those processes.

## Run on the separate macOS 27 test Mac

```sh
./focus-lab run --injector legacy --report reports/legacy.json \
  --i-understand-this-moves-the-pointer

./focus-lab run --injector numbered --report reports/numbered.json \
  --i-understand-this-moves-the-pointer
```

Use a new process for each run. `--bundles <directory>` overrides where the driver finds the bundles. Live runs refuse to start without an explicit injector, report path, and pointer acknowledgement. Failed setup still writes a report with failed, unrun scenarios. A live run returns 0 only when all assertions and pointer cleanup pass, 1 for a failed run, and 2 for usage or report-writing errors. `doctor` returns 0 when its read-only checks complete, even when permissions are absent.

The driver launches the bundle **binaries**, waits for JSONL readiness, and configures A behind B with partly overlapping windows. Setup activation happens through fixture commands. No focus or AX raise workaround runs after a test click. S7 runs **first**, before any injected click in this driver process, with a new injector whose counter starts at zero. Then S1–S6 run. Each scenario establishes its own preconditions. Fixture `front` commands are setup only.

| Scenario | Required evidence |
| --- | --- |
| S1 | Inactive A content button receives an action; A becomes active, key, and above B. Activation/key notifications must follow the click. |
| S2 | Clicking A's title bar activates A; key state and window order are also checked. |
| S3 | Ten drag steps move A by (+120,+80) CG points, within ±2 points on each axis. |
| S4 | A real status menu opens. A chosen item logs its action, the menu closes, and A stays inactive. The report says whether the item was over A. |
| S5 | A click in empty A content outside the menu closes the menu and produces no button action through it. Active/key/frontmost states are observations, not activation assertions. |
| S6 | Two content clicks produce a logged `mouseDown` with `clickCount: 2`. |
| S7 | S1 runs as the first click of the fresh injector process. Its first down has no earlier synthetic down. |

`legacy` creates mouse events with a nil source, sets click state, and leaves the event-number field at its default. On affected macOS 27 builds, S1 and S7 should fail activation even when button actions prove delivery. S2 and S3 can still pass. `numbered` adds the independent counter algorithm described in [focus-fix.md](../../docs/focus-fix.md); it should pass all scenarios. S5's activation behavior is deliberately not a pass criterion. An unaffected OS can pass legacy too, so retain the exact `sw_vers` values.

The driver prints a result table and writes JSON with OS version, permissions, preconditions, per-scenario assertions, frontmost application, fixture logs, window order, and injection records. The sibling `.evidence-<UUID>` directory retains JSONL logs and fixture stderr. Native injection records include HID seed, event number, click state, coordinates, and tap. S7 also checks that the first numbered down exceeds its HID seed. Never infer a successful fix from the event records alone: the fixture state and action logs must agree.

Cleanup attempts to release held buttons, restore the original pointer, and terminate only the three child processes it launched. Ctrl-C and SIGTERM request this cleanup. SIGKILL, a crash, or power loss cannot guarantee cleanup. The driver does not restore the previously active app or rearrange other apps. Use only a disposable test desktop.

## Plug in the real Openergy injector

```sh
./focus-lab run --injector cmd \
  --cmd-template '"/absolute/path/openergy-lab-adapter" {action} {button} {x} {y}' \
  --report reports/openergy.json --i-understand-this-moves-the-pointer
```

The template is parsed into executable plus arguments, without a shell. It supports single/double quotes and backslash escapes. Every placeholder is required. Actions are `move`, `down`, `up`, and `drag`. Buttons are `left`, `right`, and `middle`. Coordinates are decimal CG global points. The wrapper runs once per primitive, must return within three seconds, and must exit zero only after its input has been accepted. Its stdout/stderr are discarded. A failing primitive fails the scenario. A failed down still causes cleanup to attempt an up.

The real injector must preserve button, event-number, and click-count state across wrapper invocations, usually through its running service. A stateless new injector process per primitive is invalid. Start a fresh service before each run to make **S7** a first-click test of that service. In `cmd` mode the lab can prove only that S7 is the driver's first click; it cannot prove the external service has never injected a click. Process launch latency must be small enough for S6 to fit within the system double-click interval. Reports cannot inspect event numbers inside the external program.

## Geometry and manual queries

All reported coordinates use **global logical screen points**, with a top-left origin on the primary display. The AppKit conversion uses the first screen's top edge, not the currently focused screen. Frame size includes the title bar. Retina backing pixels are not used.

Each app watches an atomic JSON command file supplied with `--commands`. Write a new `requestID` for every command. Responses are `query` records in its `--log` JSONL file, with the matching ID. Example command payload:

```json
{"requestID":"manual-query-1","command":"query"}
```

Write a temporary file, then rename it onto the command path so the app sees a complete request. A/B responses include `titleBarCenter`, `buttonCenter`, `emptyContentPoint`, `frame`, window number, and active/key/main states. `front` requests setup activation. `frame` takes a `frame` object with `x`, `y`, `width`, and `height`; `quit` terminates that fixture. Menu queries include `statusButtonCenter` and `statusButtonFrame`. While tracking, they also include `menuFrame` and the three item frames/centers. `closeMenu` cancels tracking without posting input.

The status item uses a native `NSStatusItem`/`NSMenu`. The three items use custom `NSView` rows because public `NSMenu` has no item-rectangle query. `NSMenuDelegate` records open/close. Each custom row sends its item's target/action on mouse-up, then cancels tracking. This keeps the menu in WindowServer's native menu tracking while exposing real view frames. It does not test standard-row keyboard navigation, highlight behavior, every third-party status menu, or arbitrary popovers. Geometry appears only after menu layout; the timers explicitly include the event-tracking run-loop mode, as described in [Apple's menu-view documentation](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MenuList/Articles/ViewsInMenuItems.html). Missing/hidden status items or missing menu geometry produce FAIL, never a guessed menu click. Repeat the relevant menu/popover checks manually with CodexBar and other target apps on the test machine.

## Safe local verification

```sh
tools/focus-lab/build.sh
tools/focus-lab/dist/focus-lab --help
tools/focus-lab/dist/focus-lab doctor
tools/focus-lab/dist/focus-lab run --dry-run
```

These commands verify compilation, packaging, signatures, argument handling, preflight output, and scenario planning. They provide **no runtime proof** of activation, dragging, or menu behavior. Only the later live reports on the separate test Mac can provide that proof. The driver uses one `@preconcurrency import ApplicationServices` for the SDK's AX prompt-key constant, which is declared as a mutable C global. Our Swift state remains isolated under Swift 6 strict concurrency; the POSIX signal flag is a `sig_atomic_t` accessed by the handler and main loop.
