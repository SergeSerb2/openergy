# Documentation map

Start with the [project README](../../README.md).

| Document | Purpose |
| --- | --- |
| [Setup](setup.md) | Host and client setup, trust, platform prerequisites, recovery, and optional migration from Synergy. |
| [Readiness](readiness.md) | What is verified, what is not yet verified, and the release gates. |
| [Architecture](architecture-r2.md) | Design and protocol contracts, with later amendments. |
| [UI contract](ui-contract.md) | macOS app and daemon process model and IPC shapes. |
| [Focus behavior](../focus-fix.md) | The synthetic-click focus fix and the limits of its test evidence. |
| [Contributing](../../CONTRIBUTING.md) | Development rules and safe real-process verification. |
| [Security](../../SECURITY.md) | Permissions, trust boundaries, local data, and reporting. |
| [Releasing](../releasing.md) | Maintainer build, artifact, and publication steps. |

The architecture and UI contract are design records from 2026-10-21. They
include the original work plan and performance targets. Where they differ from
the source code, the source code is right. For example, the Windows and X11
backends went beyond the first plan, and interactive Windows checks are still
open.

[Verification records](../verification/README.md) and the platform crate
READMEs describe how evidence was collected. Research under [docs/re](../re)
covers only the binaries it inspected.
