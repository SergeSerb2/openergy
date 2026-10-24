# Ordinary controller R9 uncertainty handoff repair — source only

Date: 2026-10-24, America/Vancouver.

This is the implementation response to the one actionable **R8-P2-1** in
R7 `d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6`. The full R8 review is
`172ee851d99bf1c256235b23fb445cae899332ff`. **Fresh independent source review
of this exact revision is required next. Runtime acceptance is NOT ESTABLISHED.**
The ordinary flow remains already authorized. This repair adds no human
approval gate and does not run that flow.

Only this new folder is owned. The isolated checkout is
`/Users/serge/.t3/worktrees/openergy/t3-79efd32e`, initially clean at parent
`133a44cbdfec90fec0ad9f34eab0923b588c454c`. All prior source, evidence, first
failures, guards and original files remain read-only. No other worktree was
changed. There was no delegation, push, publication or network access.

The package includes all eight inert source texts. Only `ownership.py`,
`controller.py` and `watchdog.py` differ from R7. The five other modules are
byte-unchanged. The watchdog adds only a guard around peer-permission publication. [repair-r7-to-r9.patch](repair-r7-to-r9.patch) is
the exact zero-context source delta from R7, with source-folder-relative paths.
[design.md](design.md) describes all observer lifetimes and loss behavior.
[responses.md](responses.md) addresses R8 and preserves both R6 repairs and
all seven R4 repairs. [operation-plan.md](operation-plan.md) is the complete,
unexecuted finite future plan. Its exact signed inputs are missing.

Every full Inventory observation now uses one serialized, bounded uncertainty
journal. It restores previously detected PID/start metadata, original pending
start times/counts, and sticky overflow before its probe. A negative metadata index also blocks
absence if a previously registered ownership receipt/cache is unavailable. A durable begin
record precedes the existing probe; a checkpoint follows complete processing,
including finite validation failures. Missing, corrupt, incomplete or unreadable
handoffs block absence. Interruption cannot turn unpublished detection into
an empty successor Inventory. The existing cleanup-launch observer must also
prove inherited app-tree absence before releasing the peer-stop wrapper gate.

A revision guard holds that same lock through watchdog peer-permission
publication, the peer-stop wrapper gate and final absence calculation. A newer
observer revision rejects the stale decision. The existing next recovery advance
must obtain its next ordinary full observation inside the same budgets.

The journal grants negative absence authority only. It conveys no ownership,
role, executable, argv, group, launch gate or signal authority. Unresolved tokens
remain in the separate metadata-only probe partition. Exact ownership, staged
exe/argv reads, reuse privacy and fresh single-PID signal checks are unchanged.
The journal creates no additional probe, product launch, timer or retry allowance.
Any capacity/I/O failure still permits the existing targeted verified-owned
recovery path; it cannot authorize peer teardown through missing state.

[input-bindings.json](input-bindings.json) binds the complete R7 package, full
R8/R6/R4 reviews, R3/R5 inputs and all seven R5 responses to exact Git objects,
sizes and SHA-256 values. [original-index.json](original-index.json) is an exact
copy of the prior 87-file index. All 87 originals were checked against immutable
`498190feb00f51039f500ae618ac7de5fb113d86` and this checkout's bytes. The original
remains **FAILED / NOT ACCEPTED (23/25)**: shared-PGID assertion, early peer stop,
missing app OS exit and incomplete late priority coverage. Saved evidence does
not establish a product defect or fresh runtime proof. R3/R5/R7 never ran.

[source-checks.json](source-checks.json) records only file/Git/hash/JSON/AST and
patch materialization checks by a new simple standard-library data reader.
No supplied task program, module, helper, checker, controller or AST was
imported, evaluated or executed. No unit test, mock, simulation, synthetic
callback/process, fixture, preparation, preflight, product/build/signing,
process/OS/port/job query, desktop driving, credential/key access or remote
command occurred. [manifest.json](manifest.json) binds every other package
file; the final Git commit binds the manifest.

The exact retained signed bundle/app/daemon/peer inputs under
`/private/tmp/oe-ordinary-79498-l7aurhai` are missing per the task and retained
R7/R8 records. Their current paths were not queried here. No rebuild,
substitution, restoration, fixture repair or preparation is authorized by this
task. Historical hashes are bindings, not proof that those bytes now exist.
Actual ABI/identities, NI15, stop/restart coverage, all direct OS exits, 25/25
unchanged checks, unchanged bundle bytes and complete cleanup remain unverified.

Scope stays one ordinary no-render `--e2e --require-peer` default25 flow, one
launch per role, app/children before peer, 120 s flow, 60 s cleanup, fixed start
+ 180 s end, 6000 rounds/24000 rows with 1024/4096 cleanup reserve, and eight
observations/two seconds per detail uncertainty. Keep 32 MiB total/8 MiB file,
eight 2 MiB product slots, 4 MiB cleanup including 1 MiB terminal and 64 KiB
accounting, plus five seconds after each successful TERM. All product predicates,
IPC/frame/stalled-write/routine limits and private configuration remain exact.

Crafted mDNS, original provenance/security, IPC/Synergy3 import budget and the
isolated Swift build gate stay stopped. The expired closed Sparkle one-shot
has no retry, renewal, reset or approval reuse. No live sharing, installed-app,
production config/TLS/default IPC, TCC, login/services/tasks/accounts/firewall,
global DNS or toolchain-default change; no TCP24800/UDP5353 or Docker.
