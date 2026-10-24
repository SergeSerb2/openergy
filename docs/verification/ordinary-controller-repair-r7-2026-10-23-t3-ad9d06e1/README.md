# Ordinary controller R7 repair: source only

This package repairs only R6-P2-1 and R6-P2-2. It is ready for fresh independent
source review. It has no runtime acceptance. No supplied module, helper,
validator, probe, controller or AST was imported or evaluated. No fixture,
product, build, signing, preflight, test, mock, simulation, fake process,
synthetic callback, process/port/live-OS query or desktop operation ran.
There was no delegation, push or publication.

Only this new folder is owned. The T3 isolated worktree is `t3-ad9d06e1`, branch
`t3/ordinary-controller-r7-repair`, starting clean at
`1cc4d03113d8d78be84ad0fbc161610860455f35`. The authoritative `t3-52422617`
root was also clean at that commit. Production and prior evidence are read-only.

The exact R5 input is `ec723d8fde7c65e955c015653f89d24f7cc7f091`; the full
R6 review is `4eaa964eab6f74b2fc5704a1925520866c7d9f9c`.
[input-bindings.json](input-bindings.json) binds all 18 R5 files, the full R6
review, and the retained R3 `8324371b67b056c7ac8667d3f67830f1115c3c31` and
R4 `cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d` input files. All bound bytes,
sizes, SHA-256 values and Git blobs were checked. Old archives are not recopied.

All eight inert source texts are included. Only `macproc.py`, `ownership.py`,
`probe.py` and `controller.py` differ from R5. `common.py`, `exec_owned.py`,
`prepare.py` and `watchdog.py` are byte-unchanged.
[repair-r5-to-r7.patch](repair-r5-to-r7.patch) is the exact minimal zero-context
source delta, with paths relative to the source folder. [responses.md](responses.md)
answers each new finding and records preservation of all seven R4 repairs.
[design.md](design.md) and [operation-plan.md](operation-plan.md) retain the
finite ordinary flow. [source-checks.json](source-checks.json) records static
AST, exact patch materialization, file/Git/hash/JSON and unchanged-source checks.
[manifest.json](manifest.json) binds every package file except itself; Git binds
the manifest. No supplied task code ran during these checks.

R6-P2-1 now keeps a boolean detection marker and bounded unresolved PID/start
metadata across reparenting, omitted enumeration and detail uncertainty.
Unresolved detection blocks app-role absence and peer permission. It grants
no ownership, adoption, argv permission or signal authority. The probe keeps
unresolved query tokens separate from owned/direct tokens. Fresh private-path
classification still gates unowned detail reads. Reused starts return metadata
only. The eight-observation/two-second allowance is unchanged.

R6-P2-2 now checks retained actual watchdog exit/handle state and constructs
one fallback in a separate action boundary from terminal evidence lookup and
parsing. Receipt failure fails acceptance and cannot skip that boundary.
Fresh ownership checks and the original cleanup deadline still apply.

The original run `498190feb00f51039f500ae618ac7de5fb113d86` remains
**FAILED / NOT ACCEPTED (23/25)**. Its shared-PGID controller assumption and
early peer stop do not establish a product defect. App OS exit and late priority
coverage remain missing. All 87 indexed original files match their immutable
Git objects and both inspected disk copies. R3 and R5 never ran. These are
saved records and static checks, not fresh runtime observations.

**Missing runtime prerequisites:** a file-only existence check found the
retained `/private/tmp/oe-ordinary-79498-l7aurhai` root and its signed
`Openergy.app`, app executable and daemon/peer executable absent. The later
reuse-only plan cannot start from these missing inputs. This task did not
rebuild, restore old fixtures, prepare replacements or start a runtime attempt.

The later ordinary no-render `--e2e --require-peer` default25 flow remains
already authorized. Fresh independent source review comes first. Keep one
launch per role, app/children before peer, 120 s flow, 60 s cleanup and fixed
start + 180 s end. Keep 6000/24000 observations with 1024/4096 reserved for
cleanup, all frame/request/stalled-write/routine waits, future actual NI15,
and private literal loopback/virtual/discovery-off settings before startup.
Keep eight 2 MiB product slots, 4 MiB cleanup including 1 MiB terminal, and
64 KiB accounting inside 32 MiB total, with an 8 MiB evidence-file ceiling.
Rust 1.99.1/jobs2/deployment15 remain build lineage only.

The three provider-stopped tasks and isolated Swift build gate stay stopped.
The separately approved Sparkle one-shot at 21:01 failed its five-second
LSdump preflight before product launch and is closed. No retry, renewal or
approval reuse applies. Live sharing, installed apps, production config/TLS/
default IPC, TCC, login/services/tasks/accounts/firewall/global DNS/toolchain
defaults, credentials and existing private keys remain unchanged. No TCP 24800,
UDP 5353, Docker, desktop driving, push or publication belongs to this repair.
