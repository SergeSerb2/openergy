# Ordinary AppEndToEnd controller repair: source only

This folder is a new owned implementation for independent static review. No
file here has been imported or executed as a helper. No app, daemon, peer,
product CLI, watchdog, timer, process/port probe, build, signing operation, or
runtime preflight ran. No new private fixture or bundle copy was prepared.
`prepare.py` is text for a later file-only preparation step.

The input is immutable commit
`498190feb00f51039f500ae618ac7de5fb113d86`, directory
`docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8`.
[original-index.json](original-index.json) binds all 87 original files by Git
blob, size, and SHA-256. All were read from Git objects. They remain unchanged.
This includes the complete raw directory, original controller, first streams,
both controller tracebacks, first archive-validator failure, recovery records,
report, guards, and original source/build/artifact records. The new work does
not repair, overwrite, or reinterpret that archive as a successful run.

The original result remains FAILED / NOT ACCEPTED: 23/25 app checks passed.
The controller incorrectly required daemon PGID == app PID. The app was
89176; its initial daemon was PID 89180, PPID 89176, PGID 89180. The assertion
preceded identity publication. Cleanup stopped the peer, hit the same
assertion, and cancelled the watchdog. The app continued and later reported
connection/rejoin failures. Its OS exit status and the later recovered
daemon's PID/start/group/priority were not collected. These gaps remain gaps.
Controller interference establishes no product bug. This is a saved-evidence
audit, not independent runtime reproduction.

[controller-repair.patch](controller-repair.patch) is the exact original-to-new
patch for the four original Python files plus four new support modules.
It uses zero context; check it with `git apply --check --unidiff-zero` against
file-only copies of the four originals, never against the preserved archive.
[source-checks.json](source-checks.json) records text/AST/hash checks only.
[design.md](design.md) explains process identity and recovery.
[operation-plan.md](operation-plan.md) gives the finite later operation and
acceptance conditions. Source checks do not establish that this controller
works on real processes. The root must request the independent static review
before the separately authorized ordinary flow. No further human approval is
introduced here.

The intended scope stays one unchanged ordinary bundled `--e2e --require-peer`
25-check model/real-IPC flow, no rendering, with private virtual daemons and a
real private loopback peer. It retains the Swift `7287e9` and daemon `2485c14`
input bindings, ordinary Info.plist, signed bundle bytes, and full embedded
Sparkle tree. It is not running-update fixture execution, updater installation,
native UI/input/clipboard verification, trusted-release proof, or a stopped
provider task. No product source or default checks changed.

The 120 s flow and 60 s cleanup bounds, 16 MiB IPC frame cap, existing routine
waits, 1,000,000-byte stalled write, 300 ms request timeout, 8 MiB file/log cap,
32 MiB evidence cap, 6000 observation rounds, and 24000 sample rows remain.
The latter two are now shared across observers. Main sampling reserves 1024
rounds and 4096 rows for cleanup. Reaching the main share aborts the flow; it
does not skip observations. No runtime limit is increased. Ports 65431/65432
are new preselected literals; neither was queried or bound in this task.

Only this folder is owned. There was no delegation, unit test, mock,
simulated process, fake callback, computer use, desktop action, installed-app
change, live-sharing or production-config/TLS/default-IPC change, credential
access, TCC/service/account/firewall/DNS change, Docker, push, or publication.
TCP 24800 and UDP 5353 remain prohibited. The three provider-stopped tasks
remain out of scope.
