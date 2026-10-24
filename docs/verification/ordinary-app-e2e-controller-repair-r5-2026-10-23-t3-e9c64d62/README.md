# Ordinary E2E controller R5: source only

This is source preparation for fresh independent static review. None of these
modules, controllers, helpers, probes, or supplied validators ran or were
imported. No fixture was prepared. No process, port, product, build, signing,
preflight, test, mock, simulation, synthetic callback, or fake process ran.
There was no delegation, push, publication, credential access, or desktop use.
Only this new folder is owned. Production source and old evidence are unchanged.

The authoritative input root was clean at
`34587fefe1208d5d9d065cfff41df7bd1439bd07` in `t3-52422617`.
This implementation uses the T3-owned `t3-e9c64d62` isolated worktree at that
same base. During this task the authoritative root advanced to `e68432f9d12ad74fecd679f022beb1494e7028ff`
with verification-document changes only; it was clean when inspected. This
worktree stayed at the declared base. Inputs are the complete R3 candidate
`8324371b67b056c7ac8667d3f67830f1115c3c31` in `t3-432ee8ea`, and the full
independent R4 review `cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d` in `t3-f92a997d`.
[input-bindings.json](input-bindings.json) binds their exact files and Git blobs.

The original ordinary run at `498190feb00f51039f500ae618ac7de5fb113d86` remains
**FAILED / NOT ACCEPTED**, with 23/25 checks. Its controller incorrectly assumed
that the daemon PGID equalled the app PGID, then stopped the peer early. The
app's OS exit and late recovered-daemon priorities remain incomplete. No product
defect is established. [original-index.json](original-index.json) is the unchanged
87-file index. Every indexed original byte remains in its original archive; no
raw evidence was copied here. The R3 static checks passed, but R3 never ran.
Those inherited records are audited evidence, not new runtime proof.

All eight corrected source texts are present. `probe.py` is byte-unchanged.
[repair-r4-to-r5.patch](repair-r4-to-r5.patch) is the minimal zero-context delta
from the exact R3 candidate. [responses.md](responses.md) maps all seven R4 P2
findings to the changes. [design.md](design.md) defines the ownership, failure,
and byte accounting contracts. [operation-plan.md](operation-plan.md) is a
finite plan for the later, already authorized ordinary flow.
[source-checks.json](source-checks.json) records AST parsing, byte/hash checks,
and exact patch reconstruction/materialization only. No supplied AST was
compiled, evaluated, or executed. [manifest.json](manifest.json) binds the
finished source package, except the manifest itself.

The ceiling stays 32 MiB aggregate and 8 MiB per evidence file. For strict
accounting, product-owned regular files have an explicit **2 MiB subordinate
kernel limit** inside that ceiling. Eight full 2 MiB slots reserve the app
stderr, both three-file daemon log families, and the raw report. This tighter
product allocation is deliberate; exceeding it fails the future attempt.
It does not change product source, argv, default25, IPC, or routine waits.
The total also reserves 4 MiB for cleanup, including 1 MiB that only terminal
receipts can spend, plus 64 KiB for accounting metadata. No limit is increased.

The later scope stays one launch per role and one unchanged ordinary no-render
`--e2e --require-peer` default25 real-model/IPC flow. Flow is 120 s; cleanup is
60 s, with a predeclared absolute end at start + 180 s. Main sampling leaves
1024 of 6000 rounds and 4096 of 24000 rows for cleanup. Detail uncertainty has
at most eight observations or two seconds. TERM gets five seconds after each
actual successful delivery, inside the original cleanup deadline.

Keep live Mac/Windows sharing, installed apps, production config/TLS/default
IPC, TCC, login items, services, tasks, accounts, firewall, global DNS, and
all toolchain defaults unchanged. No TCP 24800, UDP 5353, Docker, existing
private signing key, credentials, or environment/log dump. Future allowed
products use only fresh private literal loopback, virtual, discovery-off
settings before startup and absolute actual NI15. Rust 1.99.1/jobs2/deployment15
are retained build lineage; this repair runs no build. Running-target Sparkle
launch/relaunch remains separately unapproved. Crafted mDNS, original
provenance/security review, and IPC/Synergy3 import-budget tasks stay stopped.
There is no new user permission gate for the ordinary authorized E2E.

**No future runtime success or acceptance is claimed.** Fresh independent
static review of this exact revision comes next.
