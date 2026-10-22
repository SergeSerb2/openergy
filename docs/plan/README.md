# Documentation map

Start with the [project README](../../README.md). Openergy is alpha software;
historical design targets and successful lab runs do not establish a supported
release or a complete native desktop compatibility matrix.

| Document | Purpose |
| --- | --- |
| [Setup](setup.md) | Fresh host/client setup, trust, platform prerequisites, recovery, and optional migration. |
| [Readiness](readiness.md) | Scope of current evidence, historical measurements, and remaining acceptance. |
| [Docs verification](docs-verification-2026-10-21.md) | Isolated CLI checks for the setup examples and integration dependencies. |
| [Contributing](../../CONTRIBUTING.md) | Development rules and safe real-process verification. |
| [Security](../../SECURITY.md) | Permissions, trust boundaries, local data, and sensitive reporting. |
| [Releasing](../releasing.md) | Maintainer build, artifact, and publication procedure. |
| [Architecture r2](architecture-r2.md) | Implementation design and protocol contracts, with later amendments. |
| [UI contract](ui-contract.md) | App/daemon process model and IPC shapes. |
| [Integration contract](integration-contract.md) | Additions to transport/backend construction and live apply. |
| [Architecture r1](architecture-r1.md) | Superseded design draft, retained for history. |
| [Focus behavior](../focus-fix.md) | Synthetic-click algorithm and limits of the existing test-Mac evidence. |

Design documents contain work-package instructions, proposed interfaces,
performance targets, and the implementation state at the time of writing.
Read them as engineering records. Check current source and dated verification
records before treating a detail as implemented or accepted. For example,
Windows and X11 backends have progressed beyond the original stub plan, while
interactive Windows acceptance remains open.

The [external runner](../../tools/e2e/README.md), platform crate READMEs, and
[dated verification records](../verification) describe how evidence was
collected. Research under [docs/re](../re) is scoped to inspected artifacts.
The [changelog](../../CHANGELOG.md) distinguishes unreleased source from releases.
