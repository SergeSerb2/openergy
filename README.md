<p align="center"><img src="assets/icon.svg" width="96" height="96" alt="Openergy icon"></p>

<h1 align="center">Openergy</h1>

<p align="center">Share a keyboard, mouse, and clipboard across computers on your local network.</p>

Openergy has a Rust daemon and a native SwiftUI macOS app. One computer hosts the
keyboard and mouse; other computers join it. A screen layout defines where the
pointer crosses between computers. Clipboard sharing supports text, HTML, and
bitmap data, subject to each backend and peer's limits.

**Alpha software.** The source version is `0.1.0`; this does not identify a
published or supported stable release. As of 2026-10-21, no release artifacts
have been published. APIs, configuration, and packaging can change. Keep local
input available while evaluating it. Read the [readiness and verification
limits](docs/plan/readiness.md) before replacing a working input connection.

## Platform status

| Platform | Current scope and evidence |
| --- | --- |
| macOS 15+ | Main development platform: native app and daemon. Automated virtual-backend checks cover app/daemon logic. Native input, permissions, and focus need separate desktop acceptance on the exact OS build. |
| Linux X11 | Headless daemon with capture, injection, and clipboard backend. Existing acceptance uses isolated Xvfb, not a general desktop/distribution matrix. |
| Windows | Headless daemon and CLI. Real named-pipe control and virtual TLS peers were checked on Windows. Native desktop input, mixed DPI, elevated apps, and clipboard acceptance remain pending. |
| Linux Wayland | Unsupported. An X11/Xwayland connection does not establish Wayland desktop capture or control. |

There is no Windows or Linux graphical app or automatic updater. No support
response time or security backport window is promised. Report reproducible
problems through [GitHub issues](https://github.com/SergeSerb2/openergy/issues).
Use [SECURITY.md](SECURITY.md) for sensitive reports.

## Build and set up macOS

Prerequisites: macOS 15 or later, Git, Rust with Cargo, and Xcode or Apple command
line tools with Swift 6 and the macOS SDK. Packaging also needs Python 3.11 or
later. Use the repository's pinned Rust toolchain when present. Building needs
network access for dependencies.

```sh
git clone https://github.com/SergeSerb2/openergy.git
cd openergy
export PATH="$HOME/.cargo/bin:$PATH"
PYTHON=python3 OPENERGY_SIGN_IDENTITY=- CARGO_BUILD_JOBS=2 nice -n 15 apps/macos/scripts/bundle.sh
```

This builds `dist/Openergy.app` for the current architecture. The explicit `-`
signing identity produces a local ad-hoc signature. It is **not** a Developer ID
signature or notarization, and does not establish distribution readiness. For
other architectures and maintainer packaging, see [releasing](docs/releasing.md).

On a Mac where you intend to enable sharing, open `dist/Openergy.app`. The app
can start a daemon and use your account's Openergy state. Do not launch it as a
test in an account whose keyboard and mouse already depend on another service.

1. Open **Permissions**. Join needs Accessibility to control this Mac. Host also
   needs Input Monitoring to capture its keyboard and mouse. Review these grants
   in System Settings → Privacy & Security. Rebuilds or signing changes can need
   a new grant.
2. On the computer with the physical keyboard and mouse, choose **Host**. In
   **Layout**, add each client's screen name and put its rectangle next to the
   host. Names must match exactly; edges must touch to permit crossing.
3. On each controlled Mac, choose **Join** and enter the host's reachable address
   and configured port. **Nearby** is only discovery information, not proof of
   identity or permission to connect.
4. Compare each pending certificate fingerprint with the fingerprint shown on
   the other computer, using a channel you already trust. Approve on both sides
   when requested. Do not approve an unexpected or changed fingerprint.
5. Check that the app reports a connected peer with TLS enabled and trusted.
   Try a harmless text transfer, a screen crossing, and a disconnect while local
   input is still available. Only then consider **Settings → Open at login**.

For fresh CLI setup, Linux, Windows, trust commands, recovery, and migration,
use the [setup guide](docs/plan/setup.md). It links complete
[server](examples/server.toml) and [client](examples/client.toml) configuration
examples. Importing Synergy settings is optional.

## Trust, permissions, and privacy

TLS is enabled by default. Clients pin the server certificate; servers require
trusted client certificates by default (`tls.require_client_trust = true`).
These are local SHA-256 certificate pins, not public certificate-authority or
hostname validation. You must verify the first fingerprint yourself. Disabling
TLS exposes input and clipboard traffic. Disabling client trust lets a server
accept clients without a stored client pin.

A trusted host can send input to a joined computer. Clipboard sharing can copy
sensitive content between peers; it is enabled by default with a 64 MiB local
limit, and the effective limit can be lower. Disable sharing, or its send/receive
directions, when that access is not appropriate. File transfer and drag-and-drop
are not supported. More option limits: [V1-LIMITS.md](crates/config/V1-LIMITS.md).

The app and daemon have no account service, telemetry uploader, or cloud relay.
They do use local network connections and mDNS discovery. State, identities,
trust records, and diagnostic logs stay on the computer unless you share them.
Logs can contain peer names, addresses, fingerprints, paths, and errors. Protect
and redact them. [SECURITY.md](SECURITY.md) describes the local control boundary,
data storage, threat model, and reporting process.

## Compatibility and evidence

Openergy implements a classic Synergy-family wire protocol. Protocol similarity
does not establish compatibility with every product or version. Existing reports
from **2026-10-21** cover these specific peers and environments:

- Deskflow 1.27 and Barrier 2.4 in an isolated Linux/Xvfb Docker lab, including
  TLS/plaintext and both roles: [interop lab](tools/interop-lab/README.md).
- Synergy core **1.21.1-beta**, protocol 1.8, in a private Windows session-0 lab:
  [verification](docs/verification/windows-synergy-1.21.1/README.md). TLS, handshake,
  and clipboard observations do not prove interactive Windows input behavior.
- Openergy's Windows control and virtual network paths:
  [verification](docs/verification/windows-daemon-2026-10-21/README.md).

Input Leap and other versions have no compatibility guarantee. Research under
[docs/re](docs/re) describes the inspected binaries; it is not a current support
matrix or a third-party security assessment.

The macOS injector numbers synthetic mouse events to address the reported macOS
27 activation behavior. The separate test-Mac run on 27.0.1 did **not** reproduce
the legacy activation failure. It is regression evidence on that build, not
proof of a fix on 27.2. See [focus behavior and open acceptance](docs/focus-fix.md).

Historical performance records are in [daemon verification](crates/daemon/VERIFICATION.md).
The 2026-10-21 Apple M5 Pro/macOS 27.2 report records about 9.3 MB idle daemon RSS
and 0.18 ms p99 between virtual capture and sink timestamps across two local
daemons. These are past measurements, not hardware-to-screen LAN latency, current
binary sizes, or promised performance. See the [readiness guide](docs/plan/readiness.md)
for the distinction between virtual, lab, and native desktop evidence.

## Development and maintenance

- [Contributing and safe verification](CONTRIBUTING.md)
- [External E2E runner](tools/e2e/README.md)
- [Architecture and document index](docs/plan/README.md)
- [Maintainer release procedure](docs/releasing.md)
- [Changelog](CHANGELOG.md), [security policy](SECURITY.md), and
  [code of conduct](CODE_OF_CONDUCT.md)

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. Dependencies retain their own terms; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the [research record](docs/re).
Openergy is an independent project based on documented interoperability
research. It is not endorsed by the projects or vendors named here.
