# Focus lab on macOS 27.0.1 (build 26A434)

Run on 2026-10-21 on a separate test Mac (Apple silicon, logged-in desktop, no physical input
during the runs), driven remotely. The Openergy injector came from commit `137d332`
(`crates/platform-macos/examples/focuslab_inject`), the lab from `tools/focus-lab`.
Each run used a fresh injector service, with S7 (the very first synthetic click) first.

Public-release sanitation: the three compressed JSON reports replace five
personal home-path values with `<test-home>`. All other decoded JSON values,
including results, timestamps, event sequences, and request IDs, are preserved.
The gzip wrappers were regenerated with a zero timestamp and no original filename.
These reports remain historical evidence; this sanitation did not rerun the lab.

| Run | Injector | S7 first click | S1 content click | S2 title bar | S3 title drag | S4 menu item | S5 click outside menu | S6 double-click |
|---|---|---|---|---|---|---|---|---|
| `native.json.gz` | Openergy, numbered clicks | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `legacy.json.gz` | Openergy, numbers off | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `builtin-legacy.json.gz` | Lab's Synergy-style injector (NULL source) | PASS | PASS | PASS | FAIL* | PASS | PASS | PASS |

\* Run before the S3 change below.

## What this shows

- Openergy's numbered-click injector works end to end on macOS 27: a click in the content of
  an inactive window activates and raises it, title-bar clicks and drags work, a click on a
  menu-bar app's menu item does not activate the window behind it, and double-clicks arrive
  as double-clicks.
- On this build (27.0.1) the click-activation bug does **not** reproduce: Synergy-style
  synthetic clicks also activate windows. The bug was reported on macOS 27 betas and is seen
  daily on macOS 27.2 (26B5112a). Confirming the cure therefore still needs a run on 27.2:
  `tools/focus-lab/run-openergy.sh legacy` then `native` (see tools/focus-lab/README.md).

## Lab changes made during this run

- Setup only: when a fixture cannot bring itself to the front (macOS 14+ cooperative
  activation refuses `NSApp.activate()` from a background host), the driver uses the
  Accessibility frontmost attribute. This runs only between scenarios, never between a
  scenario's synthetic events.
- S3: macOS anchors a title-bar drag at the first drag event, with every injector. The
  scenario now starts with a 1-point drag, like a physical mouse, and expects the window to
  follow the remaining (+119, +80) motion.
