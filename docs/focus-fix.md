# Native synthetic-click activation fix

On affected macOS 27 clients, a synthetic click can reach a control in an inactive app without activating the app or raising its window. A title-bar click can still activate it. [Deskflow issue #9852](https://github.com/deskflow/deskflow/issues/9852) records this difference between content clicks and title-bar clicks.

The event-number field is the cause identified by the upstream fix. Mouse events created with `CGEventCreateMouseEvent` have a default `kCGMouseEventNumber` of zero. The macOS 27 behavior requires a rising number for each down, starting above the HID system's mouse-down count. A matching drag and up must retain that down's number. [PR #9978](https://github.com/deskflow/deskflow/pull/9978), merged July 20, 2026, addresses background activation and the stricter first-click behavior. [PR #9991](https://github.com/deskflow/deskflow/pull/9991), merged July 27, retains that number on drags so title-bar window movement works.

This lab implements the supplied behavior independently in Swift. It does not copy upstream GPL implementation code. The code here is a verification implementation; it does not yet modify Openergy's production injector.

## Algorithm

Maintain an `Int64 lastNumber`, initially zero, plus `pressedNumber[button]`. Handle button state separately for left, right, and other buttons:

```text
on down(button):
    seed = HID left-down counter + HID right-down counter + HID other-down counter
    number = max(lastNumber, seed) + 1
    lastNumber = number
    pressedNumber[button] = number
    event.mouseEventNumber = number

on drag(button), up(button):
    event.mouseEventNumber = pressedNumber[button]
    on up: clear this button's pressed state

on move with no held button:
    leave mouseEventNumber unset (zero)
```

Read the counters through `CGEventSource.counterForEventType(.hidSystemState, eventType:)` for left, right, and other mouse-down. Re-read the seed on **every down** to stay above physical input that occurs between synthetic clicks. Guard integer exhaustion instead of wrapping. Do not assign a new number to each drag step or to the matching up. State belongs to the injector's lifetime and must survive individual primitive requests. The first synthetic down matters as much as later ones.

Track click count separately per button. Compare each down with its previous down using `NSEvent.doubleClickInterval` and a Euclidean distance threshold of four screen points. A close, timely click increments the count; otherwise it starts at one. A drag that crosses that distance ends the multi-click sequence for the next down. Store the down's click count and apply it to all corresponding drag and up events through `kCGMouseEventClickState`. Post at `kCGHIDEventTap`.

The lab's legacy mode uses the same click-count logic but leaves event numbers unset, with a nil mouse-event source. This keeps the field under test clear. Its numbered mode adds the algorithm above. It sets pointer deltas for moves/drags as well as global locations.

## Why the old focus helper misfires

The helper watches a down, finds a candidate window using `CGWindowListCopyWindowInfo`, activates its owning app, and performs an Accessibility raise. Its window guess can refer to a normal window behind a menu or popover. Clicking a CodexBar status-menu item can therefore activate Finder or another app beneath that menu. A list of ordinary windows does not encode the full native menu tracking and activation rules.

With a correctly numbered event sequence, WindowServer can perform activation as part of native event processing. No separate hit-test, `NSRunningApplication.activate`, or AX raise should run for that click. Menu and popover tracking can consume the click normally without a helper activating an unrelated window. The production fix should remove or disable the competing workaround after the native path is verified. This lab never changes the user's workaround or Synergy processes.

## Verification plan

Build and copy [Focus Lab](../tools/focus-lab/README.md) to a separate macOS 27 test machine. Grant its required TCC permissions there. Keep the old helper and other injectors out of the controlled run. The driver uses only fixture commands to establish initial focus. It never repairs activation after a scenario's click.

Run legacy and numbered in separate fresh driver processes. S7 executes first, before any synthetic click. Compare exact OS builds, fixture action logs, active/key notifications, window order, frames, and menu actions. Do not accept injected event fields alone as proof.

| Scenario | Expected native outcome |
| --- | --- |
| S1 | Click exposed content button in inactive A while B is front: A activates, becomes key, rises above B, and logs button action. |
| S2 | Click inactive A's title bar: A activates. |
| S3 | Drag A's title bar in ten steps: its CG frame moves (+120,+80), within ±2 points. |
| S4 | Click a status-menu item over A if geometry allows: item action occurs; menu closes; A stays inactive. |
| S5 | With menu open, click empty inactive A content outside the menu: menu closes; record activation without asserting it; flag any button action through the menu. |
| S6 | Double-click A content: its mouse-down log contains click count two. |
| S7 | Perform S1 as the first click in a fresh injector process. Check the first numbered down is greater than its HID seed. |

On affected macOS 27 builds, legacy should fail S1 and S7 while still proving control delivery. Numbered should pass all scenarios. S5 intentionally makes no claim about whether A must activate. Older or changed OS builds may not reproduce the legacy failure; retain `sw_vers` in the report. Compare on macOS 14–26 as a regression check when those machines are available.

The menu fixture uses native menu tracking with three custom view rows to expose item coordinates through public AppKit APIs. Its README documents geometry and row-behavior limits. Repeat menu selection with CodexBar, a standard `NSMenu` row, and relevant popovers on the test Mac. Also exercise right/other button sequences, a drag after physical clicks, and a client reconnect before shipping a production change. The automatic suite currently exercises the left button; the injector implements separate button state for later extensions.

On the live development workstation, verification is restricted to release compilation with zero warnings, bundle signature verification, `--help`, read-only `doctor`, and `run --dry-run`. These checks do not establish that the macOS 27 bug is fixed at runtime. Store the later legacy and numbered JSON reports as the acceptance evidence.
