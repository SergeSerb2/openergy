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

Release preparation on 2026-10-21–22:

- Fix restart recovery, IPC shutdown, config rollback, pin revocation, and CLI
  argument handling.
- Preserve held-key and screen-transition order under queued network traffic.
- Enforce clipboard sharing settings and distinguish unavailable reads from
  intentional clears. Reject malformed format containers.
- Harden TLS identity/trust storage, native resource cleanup, and platform
  clipboard handling.
- Add isolated real-process regression scenarios, version-pinned build checks,
  dependency notices, and repeatable archive packaging with extracted-binary
  smoke checks.
- Document alpha support limits, fresh setup, contribution rules, privacy,
  security reporting, and remaining native/distribution acceptance.

No release artifacts have been published as of this date. This entry does not
claim that native desktop acceptance or distribution gates have passed. See
[readiness](docs/plan/readiness.md) for evidence and open limits, and
[releasing](docs/releasing.md) for the maintainer process.
