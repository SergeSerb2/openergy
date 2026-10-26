# Verification records

Dated records of real checks on Openergy. Each record describes the commit,
machine, and setup it ran on. A record does not prove that later commits
behave the same way. For the current status, read
[readiness](../plan/readiness.md).

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
