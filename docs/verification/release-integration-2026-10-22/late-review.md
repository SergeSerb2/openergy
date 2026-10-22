# Late review and archive record

Date: 2026-10-22 (America/Vancouver). Source-stage alpha; no release publication
or repository visibility change occurred.

## Windows migration

The privilege split in `71e6f74` keeps the per-user manager and controller at
normal integrity. Fixed privileged Synergy operations use a separately
provisioned, administrator-owned helper. Routine elevation does not execute the
LocalAppData manager. The helper checks protected ownership and replacement
rights, uses a fixed system PowerShell host, and authenticates the controller
through kernel pipe/process/token information. See the
[boundary record](../../../tools/windows/MIGRATION-PRIVILEGE-BOUNDARY.md) and
[provisioning contract](../../../tools/windows/MIGRATION-SECURITY.md).

Independent round 9 source review found one P1 and three P2 defects. Integrated
commit `2f49206` addresses all four:

- Open the client pipe with explicit ReadWrite/Synchronize rights, without
  requesting pipe-instance creation. Keep Identification SQOS and peer checks.
- Preserve RegistryBinary arrays, including the distinction between absent and
  empty values.
- Keep automatic recovery armed until the controller acknowledges `STARTED`.
- Restore a service saved as stopped, including after a later automatic start.

Independent round 10 review of `2f49206` confirmed these responses and found a
further P2: a running service can have Disabled startup mode, but cannot restart
after cutover. Integrated commit
`e10e4a413c3a556471d9879053a8f9d744eea5d2` refuses that snapshot before saving
state, stopping the service, or killing processes. The controller explains how
to retry. Disabled-plus-stopped remains supported in source. The parent checked
the final guard ordering and protocol response; this is source inspection, not
an independent final verdict or a successful service test.

The [correctness record](../../../tools/windows/MIGRATION-CORRECTNESS-FOLLOWUP.md)
preserves each finding, response, exact ten-file script inventory, and actual
Windows PowerShell 5.1/PowerShell 7 results. Full-file parsing and private
negative public-command checks passed after the last fix. They cover elevated
entry refusal, protected-owner/ACL/path refusal, module isolation in Check,
Network-SID pipe denial, occupied nonce-pipe refusal, and owned-fixture cleanup.

Successful limited-token IPC, binary registry round trips, reply/acknowledgement
failure recovery, and actual service/startup/UI restoration remain unverified.
Restricted PowerShell children exit `0xc0000142` before script code in SSH
session 0. No service/task/registry mutation, helper provisioning, installed
configuration change, desktop action, or policy weakening was used to force
those checks to pass. The original live input connection was not used.

## Release provenance review

Commit `88c199a` separates frozen default-branch validation, disposable read-only
candidate builds, and trusted publication that reads candidate archives only as
data. Both workflows skip all jobs while the repository is private. The
[release boundary record](trusted-release-boundary.md) contains the source
inventory and 82 real local CLI checks. Hosted execution and remote draft
creation were not attempted.

The additional independent delegated review failed because its provider flagged
possible cybersecurity risk. It returned no findings or verdict. That review
was not counted as passed or retried through another route. The prior PR
security finding remains open for review of the implemented response.

## Clean macOS archive after the Windows fixes

Packaging ran from clean source
`e10e4a413c3a556471d9879053a8f9d744eea5d2` with the pinned Rust 1.99.1 and Swift
6.4 toolchains. It checked notice freshness, built the local ad-hoc bundle,
exercised staged and freshly extracted daemon CLI/virtual IPC, and verified the
extracted code signature. Every daemon used private state, the virtual backend,
and assigned loopback ports outside 24800.

| Artifact | Result |
| --- | --- |
| Archive | `openergy-0.1.0-aarch64-apple-darwin-adhoc.zip`, 3,614,031 bytes |
| Archive SHA-256 | `636b8445b5ae51b0cf1f3374d4ba3e34f73b6001285a9811a7670a1d7d318219` |
| App regular-file payload | 6,324,424 bytes across nine files |
| Daemon SHA-256 | `1405abcc08a4f8b175d4116a413ac0426c5e708578e9e250487ff13d18ae97e5` |
| App executable SHA-256 | `724de37fd2e482e425f0f1f5ec7af64f80b353ed29e35ddf2e3c1b60e0d21d0d` |
| Provenance | Embedded BUILD.json equals the sidecar; dirty=false; README links to this exact source |

Both executable hashes equal the `88c199a` package. Its 25 headless app steps and
seven startup refusals therefore apply to the same executable bytes. The
daemon also matches the earlier 88-case virtual and sustained-workload records.
Those workloads were not rerun or relabeled as `e10e4a4` source runs. The archive
remains local, ad-hoc signed, unnotarized, and unpublished.

## Remaining gates

The PR stays open while security findings and final review remain outstanding.
Positive Windows migration acceptance requires a separate recovery-capable
interactive environment. Native macOS/Windows desktop acceptance, focus and
permissions, sleep/wake and login behavior, trusted signed distribution, Intel
Mac/x86-64 Linux artifact acceptance, and a verified confidential reporting
route remain open. See [readiness](../../plan/readiness.md). None of the scoped
checks proves that every workload is stable, leak-free, or secure.
