# Verification records

Dated records of real checks on Openergy. Each record describes the commit,
machine, and setup it ran on. A record does not prove that later commits
behave the same way. For the current status, read
[readiness](../plan/readiness.md).

## Current acceptance tracking

The [native acceptance matrix](native-acceptance.md) links the platform,
install/update, peer and final-candidate tasks. It defines pass, failed, blocked
and skipped, and the evidence fields required for each run. Native desktop
acceptance remains blocked where only virtual, headless or Xvfb checks exist.

The published alpha is `v0.1.0`, source
`e5d86e2389badf0c45fc2fcef8c4f478b3a2a284` (2026-11-02).
[Readiness](../plan/readiness.md#published-alpha-and-source) records its public
archive hashes, tag resolution and the November 2 CI results inspected on
November 3. Those results are separate from the historical records below.

## Current candidate records

These November 3 records cover isolated checks of the release/source. Follow
each report's actual result and remaining criteria; a report being collected
does not supply a pass. None establishes native desktop acceptance.

| Record | Scope and limits |
| --- | --- |
| [update-channel-2026-11-03](update-channel-2026-11-03/README.md) | Real HTTPS clients on Linux and two Macs, source/bundle feed configuration and public signature checks. Curl/Sparkle-style GETs read an empty signed feed; the Python verifier receives 403. Signed-app Sparkle acceptance remains open. |
| [macos-candidate-2026-11-03](macos-candidate-2026-11-03/README.md) | Published signed universal app on M1 macOS 27.0.1: artifact/signature/staple/Gatekeeper checks and 25/25 headless virtual app checks. No quarantine/first-user install proof. The unchanged app refuses the private updater harness before a feed request. |
| [windows-candidate-2026-11-03](windows-candidate-2026-11-03/README.md) | Published unsigned x86-64 package on Windows build 26340, session 0: 23 control checks, 19 IPC deadlines, pipe identity and mutual-TLS virtual smoke passed. Lifecycle/limited-token migration and Synergy checks remain blocked; no interactive acceptance. |
| [linux-candidate-2026-11-03](linux-candidate-2026-11-03/README.md) | Pinned build/audit passed; strict Linux repeat 93/93 and separate Docker interop 30/30 cover all 123 cases. Initial 91/93 and passing retries remain recorded. Xvfb and emulated Linux package TLS smoke passed; no physical X11 acceptance. |
| [stability-wave-2026-11-08](stability-wave-2026-11-08/README.md) | Candidate branch `t3/stability-wave-2` on Linux aarch64: full virtual gate 95/95 (30 Docker excluded), Xvfb probe incl. X-server reconnect, cross-target checks, live feed recheck. First loaded 90/95 run and flake root causes retained. No native desktop acceptance; x86_64 hosted repeat still required. |
| [wayland-discovery-2026-11-09](wayland-discovery-2026-11-09/README.md) | Read-only Wayland discovery probe (design M1): honest no-session exit, guarded headless Weston lane with structured capability facts, portal introspection without sessions. Both roles correctly report unsupported on Weston; Mutter/KWin lanes unverified. No backend or daemon behavior change. |

## Historical records

| Record | What it shows |
| --- | --- |
| [release-integration-2026-10-22](release-integration-2026-10-22/README.md) | Integrated release-preparation checks: the full real-process scenario run, review fixes, and follow-up reports. [round5.md](release-integration-2026-10-22/round5.md) has the resource fixes and the bundle size; [sustained-memory.md](release-integration-2026-10-22/sustained-memory.md) has the long transfer run; [linux-amd64-python311.md](release-integration-2026-10-22/linux-amd64-python311.md) has the x86-64 Linux package check. |
| [macos-app-current-flow-2026-10-23](macos-app-current-flow-2026-10-23/README.md) | The packaged macOS app and its bundled daemon pass 25/25 headless app-to-daemon checks against a virtual peer. |
| [sparkle-integration-2026-10-23](sparkle-integration-2026-10-23/README.md) | Sparkle updater checks against local and live feeds, and the signed empty update feed. |
| [focus-lab-macos-27.0.1](focus-lab-macos-27.0.1/README.md) | Click-focus scenarios on a test Mac running macOS 27.0.1. |
| [windows-daemon-2026-10-21](windows-daemon-2026-10-21/README.md) | The Windows daemon and CLI over private named pipes, with virtual TLS peers. |
| [windows-synergy-1.21.1](windows-synergy-1.21.1/README.md) | Openergy talking to the real Synergy 1.21.1 core on Windows, in both roles. |
| [public-release-audit-2026-10-21](public-release-audit-2026-10-21/README.md) | License, dependency, secret-scan, and history-privacy audit. |
| [private-update-bootstrap-repair-2026-10-23-t3-54de37ad](private-update-bootstrap-repair-2026-10-23-t3-54de37ad/README.md) | Design of the private updater test fixture launcher. |

Other evidence lives next to the code it covers, for example
[crates/daemon/VERIFICATION.md](../../crates/daemon/VERIFICATION.md),
[crates/platform-macos/VERIFICATION.md](../../crates/platform-macos/VERIFICATION.md),
[crates/platform-linux/VERIFICATION.md](../../crates/platform-linux/VERIFICATION.md),
and [tools/interop-lab](../../tools/interop-lab/README.md).

The complete historical evidence, including raw logs and work records removed
from this directory, is kept at the Git tag `evidence-archive-2026-10-25`.
