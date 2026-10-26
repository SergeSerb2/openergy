# Readiness

Status date: 2026-10-25. Openergy `0.1.0` is alpha software. Keep another way
to control each computer while you try it.

## Latest verification run

2026-10-25, on the commit merged for the public alpha.

- Rust: `cargo fmt --check`, strict workspace Clippy (`-D warnings`), locked
  release build, `cargo deny check`, and the notice check passed.
- Real-process suite: 119/119 scenarios on an Apple M5 Pro (macOS 27.2),
  including the Deskflow and Barrier Docker cases. Linux checks passed in Docker
  (89/89 without the Docker-in-Docker cases) and in an isolated Xvfb server.
- macOS app: a Developer ID signed, notarized, and stapled build was accepted
  by Gatekeeper on this Mac and, as a quarantined download, on a second Mac
  (macOS 27.0.1). There it passed the 25-check headless app flow.
- Windows, Rust 1.99.1 MSVC: release build and strict Clippy passed.
  Private-instance scripts passed: control (23 checks), IPC deadlines (19/19),
  lifecycle (15), lifecycle ownership (11), and pipe identity.
- Footprint: arm64 app bundle 9,276 KiB on disk; daemon alone 8.47 MiB RSS and
  0.0% CPU over 60 s idle. The installed app and daemon used 64.5 MiB together
  as a connected client. See the [comparison](../../README.md#openergy-vs-synergy).
- Fixed in this run: the menu-bar app redrew a hidden animation in the client
  role and used 21–29% CPU. With the fix it uses 0.0% in the same scenario.

## Verified

Each record covers the commit and machine it ran on. Resource numbers depend
on the machine, build, and workload.

| Area | What was checked | Record |
| --- | --- | --- |
| Daemon and CLI | Real processes over local IPC and loopback TCP/TLS with the virtual backend: input, clipboard, trust, restart, recovery. | [round5](../verification/release-integration-2026-10-22/round5.md), [daemon](../../crates/daemon/VERIFICATION.md) |
| macOS app | Packaged app and bundled daemon: 25/25 headless checks against a virtual peer, including reconnect and crash recovery. | [app flow](../verification/macos-app-current-flow-2026-10-23/README.md) |
| macOS updates | Sparkle checks against local test feeds and the live, empty, signed feed (2026-10-23). | [Sparkle](../verification/sparkle-integration-2026-10-23/README.md) |
| macOS focus | Numbered clicks pass all seven focus scenarios on macOS 27.0.1. On macOS 27.2, daily use as a client of a Windows host since 2026-10-21, with clicks focusing windows. | [focus lab](../verification/focus-lab-macos-27.0.1/README.md) |
| Linux X11 | Capture, injection, and clipboard in an isolated Xvfb server on ARM64. The x86-64 package was checked under emulation (2026-10-22). | [X11](../../crates/platform-linux/VERIFICATION.md), [x86-64](../verification/release-integration-2026-10-22/linux-amd64-python311.md) |
| Windows | Daemon and CLI over private named pipes, with virtual TLS peers in both roles. As a host, daily use since 2026-10-21 with a Mac client: keyboard, mouse, and clipboard. | [Windows](../verification/windows-daemon-2026-10-21/README.md) |
| Other peers | Deskflow 1.27 and Barrier 2.4 in a Docker lab, both roles, TLS and plaintext. Synergy core 1.21.1 on Windows, both roles, TLS and clipboard (2026-10-21). | [interop lab](../../tools/interop-lab/README.md), [Synergy](../verification/windows-synergy-1.21.1/README.md) |
| Licenses and secrets | cargo-deny, notice generation, and secret scans of the tree and full history; no real secrets found (2026-10-25). | [audit](../verification/public-release-audit-2026-10-21/README.md) |

## Not yet verified

- Native input on test desktops: held keys across a disconnect, sleep and
  wake, mixed display scaling, and keyboard layouts.
- A controlled side-by-side focus test on macOS 27.2, where the bug is seen.
- Windows as a client: injection, clipboard, elevated apps, and UAC.
- A first install by a new user: granting and revoking permissions, and the
  login item.
- Downloading, installing, and relaunching an update through Sparkle.
- Input Leap and other Synergy versions.
- A lab benchmark against Synergy on the same workload. The README comparison
  is a field measurement on one Mac.

## Known limits

- macOS is the main platform. Windows and Linux have no graphical app or
  automatic updates. Wayland is not supported.
- No file transfer or drag-and-drop. Clipboard formats differ by backend and
  peer. The 64 MiB clipboard limit is local; a peer can accept less. X11
  PRIMARY selection is not shared.
- Secure Input, screen lock, UAC, and OS input policies can pause sharing.
- Config and IPC formats can change before a stable release. More limits are in
  [V1-LIMITS.md](../../crates/config/V1-LIMITS.md).

## Release gates

A stable release needs all of these on the exact commit and artifacts:

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

Alpha releases list every check that did not run. Report security problems
through [SECURITY.md](../../SECURITY.md). The maintainer steps are in
[releasing](../releasing.md).
