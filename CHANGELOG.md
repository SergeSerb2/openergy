# Changelog

Changes here describe source development. A Cargo version or an entry under
Unreleased does not mean that a release was published.

## Unreleased — 0.1.0 alpha

Initial source implementation, recorded on 2026-10-21:

- Rust daemon and CLI for keyboard, pointer, and clipboard sharing.
- Native SwiftUI macOS menu-bar app with host/join, layout, permissions, and trust
  controls.
- TLS certificate pinning, local control IPC, and Synergy configuration import
  and export.
- macOS, X11, Windows, and virtual backends. Windows native desktop acceptance
  remains pending; Wayland is unsupported.
- External real-process E2E runner and isolated interoperability labs.
- Public setup, security, contribution, and readiness documentation.

No release artifacts have been published as of this date. This entry does not
claim that native desktop acceptance or distribution gates have passed. See
[readiness](docs/plan/readiness.md) for evidence and open limits, and
[releasing](docs/releasing.md) for the maintainer process.
