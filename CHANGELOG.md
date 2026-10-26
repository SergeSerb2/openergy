# Changelog

All notable changes to Openergy are listed here. A version heading marked
"unreleased" has no published release artifacts.

## 0.1.0-alpha — unreleased

First public source release.

### Added

- Rust daemon and `openergy` CLI that share keyboard, mouse, and clipboard
  over the classic Synergy-family protocol (1.6–1.8), in host or join role.
- Native SwiftUI menu-bar app for macOS 15+: host and join, screen layout,
  permissions, trust approval, logs, open at login, and Sparkle update checks.
- Backends for macOS, Linux X11, and Windows, plus a virtual backend for tests.
  Windows and Linux run as headless daemons.
- Clipboard sharing for text, HTML, and images, with a 64 MiB default limit
  and per-direction switches.
- TLS on by default with SHA-256 certificate pinning in both directions, and
  local control over a private Unix socket or Windows named pipe.
- mDNS discovery of nearby hosts (shown as information only, not as trust).
- Import and export of Synergy configuration, and a Windows script that
  replaces a Synergy server with Openergy at logon.
- Numbered synthetic clicks on macOS, to address the macOS 27 window-focus
  problem seen with other Synergy-family clients.
- An external real-process test runner, Docker interop labs for Deskflow and
  Barrier, and release tooling that builds, smoke-tests, and records the
  provenance of archives.

### Fixed

- The macOS app no longer redraws animations in hidden windows. In the client
  role, the menu-bar panel's connection line kept animating while closed and
  used 21–29% CPU. Visible animations are capped at 30 frames per second.

### Security

- Bounded reads for config, trust, identity, and IPC data; special files and
  symlinks are refused.
- Trust imports are checked as a whole before either list is saved; pin
  revocation takes effect for live connections.
- Packet metadata counts against transport queue limits, and X11 image
  conversion is size-checked before allocation.

### Known limits

- Native desktop input and clipboard are not yet verified on Windows, and only
  in an Xvfb server on Linux. Wayland is not supported.
- No file transfer or drag-and-drop.
- Config and IPC formats can change before a stable release.

See [readiness](docs/plan/readiness.md) for what has been verified and the
release gates, and [releasing](docs/releasing.md) for the maintainer steps.
