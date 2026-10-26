# Readiness

Status date: 2026-10-25. Openergy `0.1.0` is alpha software. No release
artifacts are published yet. Keep another way to control each computer while
you try it.

## Latest verification run

<!-- fresh-verification: add the current-commit build and test record here -->
Pending. The record of a fresh build and test run on the current commit goes
here.

## Verified

Each record below is dated and covers the commit and machine it ran on. A
later commit can behave differently until it is checked again.

| Area | What was checked | Record |
| --- | --- | --- |
| Daemon and CLI | Real processes over local IPC and loopback TCP/TLS with the virtual backend: input, clipboard, trust, restart, recovery. 117/117 scenarios, including 30 Deskflow/Barrier Docker cases (2026-10-22). | [round5](../verification/release-integration-2026-10-22/round5.md), [daemon](../../crates/daemon/VERIFICATION.md) |
| macOS app | Packaged app and bundled daemon: 25/25 headless checks against a virtual peer, including reconnect and crash recovery (2026-10-23). | [app flow](../verification/macos-app-current-flow-2026-10-23/README.md) |
| macOS updates | Sparkle checks against local test feeds and the live, empty, signed feed (2026-10-23). | [Sparkle](../verification/sparkle-integration-2026-10-23/README.md) |
| macOS focus | Openergy's numbered clicks pass all seven focus scenarios on macOS 27.0.1 (2026-10-21). | [focus lab](../verification/focus-lab-macos-27.0.1/README.md) |
| Linux X11 | Capture, injection, and clipboard in an isolated Xvfb server on ARM64. The x86-64 package was checked under emulation (2026-10-22). | [X11](../../crates/platform-linux/VERIFICATION.md), [x86-64](../verification/release-integration-2026-10-22/linux-amd64-python311.md) |
| Windows | Daemon and CLI over private named pipes, with virtual TLS peers in both roles (2026-10-21). | [Windows](../verification/windows-daemon-2026-10-21/README.md) |
| Other peers | Deskflow 1.27 and Barrier 2.4 in a Docker lab, both roles, TLS and plaintext. Synergy core 1.21.1 on Windows, both roles, TLS and clipboard (2026-10-21). | [interop lab](../../tools/interop-lab/README.md), [Synergy](../verification/windows-synergy-1.21.1/README.md) |
| Licenses and secrets | cargo-deny, notice generation, and secret scans of the tree and history (2026-10-21). | [audit](../verification/public-release-audit-2026-10-21/README.md) |

Resource numbers (memory, CPU, size) come from these records. They depend on
the machine, build, and workload. The README comparison with Synergy uses
numbers from different setups, so it is not a matched benchmark.

## Not yet verified

- Native input on real desktops: screen crossings, held keys across a
  disconnect, sleep and wake, mixed display scaling, and keyboard layouts.
- The macOS focus fix on macOS 27.2, where the bug is seen. The bug did not
  occur on 27.0.1, so that run cannot confirm the fix.
- Interactive Windows use: input hooks, injection, clipboard, elevated apps,
  and UAC. The Windows checks ran in a non-desktop SSH session.
- A first install on a clean Mac: Developer ID signing, notarization,
  Gatekeeper, granting and revoking permissions, and the login item.
- Downloading, installing, and relaunching an update through Sparkle.
- Input Leap, other Synergy versions, and live input from a Synergy server to
  an Openergy client on a Windows desktop.
- Wayland. It is not supported.

## Known limits

- macOS is the main platform. Windows and Linux have no graphical app and no
  automatic updates.
- No file transfer or drag-and-drop. Clipboard formats differ by backend and
  peer. The 64 MiB clipboard limit is local; a peer can accept less. X11
  PRIMARY selection is not shared.
- Secure Input, screen lock, UAC, and OS input policies can pause sharing.
- Config and IPC formats can change before a stable release. More limits are in
  [V1-LIMITS.md](../../crates/config/V1-LIMITS.md).

## Release gates

A release needs all of these on the exact commit and artifacts:

1. Build, format, lint, dependency and license checks, and the real-process
   scenario suite pass.
2. Native input and clipboard checks pass on a separate test machine or
   account that has another input path. Virtual checks do not count.
3. A fresh macOS install passes: signing, notarization, permissions, login
   item, and recovery.
4. The focus check passes on an affected macOS build.
5. Interactive Windows checks pass before Windows is called desktop-ready.
   Real X11 desktop checks pass before broad Linux support is claimed.
6. Third-party peers are retested, with versions, roles, TLS, and trust
   settings recorded, including a wrong-pin case.
7. A private security reporting route is set up (see [SECURITY.md](../../SECURITY.md)).

Release notes must list every check that did not run. The maintainer steps are
in [releasing](../releasing.md).
