# Alpha readiness and evidence

Status date: **2026-10-22**. Openergy is source-stage alpha software, version
`0.1.0`. No release artifacts have been published as of this date. Preparing
source for public review does not establish production readiness, authorize a
visibility change, or imply a supported binary release.

## What the evidence covers

| Surface | Existing evidence | Limit |
| --- | --- | --- |
| Daemon and CLI | [Real-process verification](../../crates/daemon/VERIFICATION.md), IPC, loopback TCP/TLS, and virtual input/clipboard scenarios. | Virtual ledgers do not prove native capture/injection or recovery after every OS failure. |
| macOS app | Historical app/daemon E2E report in the daemon verification record. | It checks app logic with isolated state. It is not a complete first-install, TCC, native UI, or signing acceptance run. |
| macOS native backend | [Backend verification](../../crates/platform-macos/VERIFICATION.md) and [27.0.1 focus lab](../verification/focus-lab-macos-27.0.1/README.md). | The focus run did not reproduce the legacy activation bug. Current-build 27.2 fix acceptance remains open. |
| Linux X11 | [X11 verification](../../crates/platform-linux/VERIFICATION.md) in isolated Xvfb. | No broad distribution, window manager, physical hardware, or Wayland claim. |
| Linux x86-64 artifact | [Python 3.11 package and real-command checks](../verification/release-integration-2026-10-22/linux-amd64-python311.md) under AMD64 emulation, including extracted executable checks. | Not physical x86-64 desktop or performance acceptance. |
| Windows | [Daemon/control verification](../verification/windows-daemon-2026-10-21/README.md), private pipes, virtual peers, TLS, and session-0 native start/stop. | Interactive hooks, actual injection, clipboard, mixed DPI, layout behavior, and elevated apps need desktop acceptance. |
| Third-party peers | [Docker interop lab](../../tools/interop-lab/README.md): Deskflow 1.27 and Barrier 2.4. [Windows report](../verification/windows-synergy-1.21.1/README.md): Synergy core 1.21.1-beta. | Results apply to the recorded builds, configurations, roles, and formats. Input Leap and other versions are not established by protocol similarity. |

Each report is a dated record of its own revision and environment. Passing
counts in those reports are historical, not the current branch's status.
The parent integration run must collect fresh evidence after changes merge.
An unavailable environment is a missing check, not a passed check.

The [2026-10-22 integration record](../verification/release-integration-2026-10-22/README.md)
records the release-preparation branch's fresh checks and later focused fixes.

## Measurements are scoped

The [2026-10-21 daemon report](../../crates/daemon/VERIFICATION.md) records an
Apple M5 Pro on macOS 27.2, including about 9.3 MB idle daemon RSS, about 19 MB
after an 8 MiB virtual clipboard transfer, and 0.03% idle CPU over 60 seconds.
It also records a 0.18 ms p99 sample over 1,000 virtual moves, from server
capture to client sink timestamps across two local daemons. This used plaintext
loopback transport and excluded external IPC observation overhead. It is not
physical input-to-screen latency over a LAN.

The external runner's S15 timing has a different boundary: external injection
through IPC to the receiving daemon's virtual ledger. Its results cannot be
compared directly with the capture-to-sink timestamp sample. Binary size, resident memory,
CPU, and latency depend on toolchain, build options, OS, workload, and machine.
Re-measure the artifact you intend to distribute. No performance guarantee or
unqualified comparison with another product follows from these records.

## Known functional and support limits

- macOS is the main development platform. Windows and Linux have no graphical
  app or supported automatic updater. Wayland is unsupported.
- File transfer and drag-and-drop are not implemented. Clipboard format support
  differs by backend and peer; the 64 MiB default is a local limit, not a promise
  that another peer will accept it. X11 PRIMARY is not routed by v1.
- Language synchronization, half-duplex lock behavior, and modifier key-ID
  remapping have limits listed in [V1-LIMITS.md](../../crates/config/V1-LIMITS.md).
- Secure Input, screen lock, UAC, permission changes, and OS input policies can
  interrupt sharing. There is no claim that the daemon can bypass those rules.
- Configuration and IPC can change before a stable release. No stable support,
  update, security backport, or response-time commitment is defined.

## Acceptance still needed for a release claim

On 2026-10-22, the maintainer chose to **keep native input and clipboard checks
as a release gate**. Run them only on a separate test account or machine with
recovery input available. Preserve the current Mac and Windows sharing
connection. Virtual checks do not close this gate.

Maintainers must attach results for the exact commit and artifacts, using
[docs/releasing.md](../releasing.md). At minimum, distinguish:

1. Build, formatting, lint, dependency/license review, and real-process E2E
   checks on the integrated source.
2. Fresh macOS installation, signature/notarization state, permissions granted
   and revoked, login behavior, and local recovery on supported OS builds.
3. Native screen transitions, held-key/button disconnect recovery, sleep/wake,
   mixed scaling, keyboard layouts, and clipboard formats on intended desktops.
4. A controlled legacy/native focus comparison on an affected macOS build.
   Historical user reports alone do not close this gate.
5. Interactive Windows acceptance before describing Windows as desktop-ready,
   and real X11 desktop acceptance before claiming broad Linux support.
6. Third-party peer versions and roles actually retested, with TLS/trust settings
   and negative pin/unknown-peer cases recorded.
7. GitHub private vulnerability reporting enabled and verified when the
   repository is made public, before public launch. The maintainer selected
   this route on 2026-10-22. The repository is still private; activation and
   verification remain open. This choice does not authorize a visibility change.

Keep any missing checks explicit in release notes. A draft artifact, an ad-hoc
signature, a startup result, or a green CI job cannot substitute for the missing
native or distribution evidence. Use [SECURITY.md](../../SECURITY.md) for the
threat model and private report path, and [CONTRIBUTING.md](../../CONTRIBUTING.md)
for safe verification.
