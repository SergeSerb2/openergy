# Scoped LaunchServices source repair — DO-NOT-RUN

This folder contains an inert source repair for the closed Sparkle operation's
first preflight failure. It is **not runnable or clear for approval**. No new
execution, review or approval receipt exists. Approval and runtime acceptance
remain false in [the copied proposal](operation-DO-NOT-RUN.json).

Owned folder: `docs/verification/sparkle-scoped-ls-source-repair-2026-10-23-t3-2d3b20d4/`
in `/Users/serge/.t3/worktrees/openergy/t3-2d3b20d4`, branch
`t3/sparkle-scoped-ls-capture`. The checkout started clean at
`1cc4d03113d8d78be84ad0fbc161610860455f35`. Only this folder is changed.

The corrected [controller](controller-DO-NOT-RUN.py.txt) replaces the global
LaunchServices dump with the small inert
[Python3.15 ctypes helper](launchservices-DO-NOT-RUN.py.txt). The helper queries
only the frozen app and driver bundle identifiers. It uses one direct child
through the existing `ro()` at each existing capture phase: preflight, old-ready,
new-ready and postflight. Each child makes at most one native call per ID.
The five-second absolute command deadline, direct-child kill, bounded drain/reap,
reader tracking, persistent observer and existing post-capture cap remain intact.
No whole database scan, timeout exception, retry or fallback was added.

[controller.diff](controller.diff), [proposal.diff](proposal.diff) and
[helper.diff](helper.diff) are exact minimal unified patches. Controller changes
are limited to two review-warning lines and `os_evidence`. Its other functions
and classes are AST-equal to the cleared source. Four existing proposal fields
change: controller binding, missing prerequisites, the LS observation description,
and its proof limits. Two new fields bind the helper and this repair's provenance.

All 36 files from the cleared `95c59a27be0f9c95ed207eac4180193a493bb5f2`
folder are byte-exact in [inputs/cleared-source](inputs/cleared-source).
Four original input files are also copied directly under `inputs/` to preserve
the controller's existing input-hash gates. These are inert historical inputs.
Their old relative provenance references describe that old folder. The full
[R6 review](inputs/independent-R6-review.md) is from
`6ad28c7835a09dc40a1a8141caa1551bc91a9979`; it clears only the old hashes.
Full R2/R4 reviews and earlier exact patches remain in the copied source tree.

Frozen corrected SHA-256 identities:

- Controller: `2a25d273ebf5b79783de44b7d39c38e4916b7475a2ba4901b092691506826175`.
- Helper: `b4672e2c65d604ab793247083283ac140056923a3341ee331a2bdba893e03644`.
- Proposal: `589cee62e7a0446f19d55d4e29060e6770bcd86a16f5043ce69a3e8765911fff`.

[freeze.json](freeze.json) binds every regular file in this folder except itself,
including the exact source, inputs, patches, API evidence and static result.
The commit binds the freeze manifest. No self-hash or approval receipt is implied.

The API's ABI, ownership, absence handling, redaction and bounds are explained in
[source-api-rationale.md](source-api-rationale.md), with primary Apple links and
hashed local SDK excerpts. [responses.md](responses.md) maps the exact timeout,
all five preserved repairs, and the next review requirements.

## Closed first failure and current blockers

The user approved one exact invocation at **2026-10-23 21:01:00 PDT**.
That invocation used the unchanged reviewed controller and exited one at
`main:972` → `os_evidence(preflight):539` → `ro:511`. The whole-database
`lsregister -dump` timed out after **4.99583949999942 seconds** under the unchanged
five-second command limit. There were seven saved preflight records. Product
launch, bind, HTTP, IPC, updater, relaunch, quit and runtime timer startup had not
begun. No retry ran. This is saved evidence, not a fresh reproduction or a
host-wide process-absence claim.

Byte-exact failure copies from `1cc4d03113d8d78be84ad0fbc161610860455f35`
are in [inputs/first-failure](inputs/first-failure): full README, result, selected
file postflight, exact stderr and first LS error. The full original export stays
at `../sparkle-running-preflight-failure-2026-10-23/`. Its actual approval receipt
has SHA-256 `af28e5dfd12c05f9a464bf4180e9e7361d5fb85f5510cae70a1eb9946b8abeee`.
That receipt and original guards remain immutable and were not copied into an
executable receipt or reused. The selected postflight is a **21:11 historical
file observation**, not current private artifact retention proof.

The user's current task reports that by **22:10 PDT** the `/tmp` checkpoint and
several exact private fixture/raw artifacts were absent. Retention is therefore
a missing prerequisite. This task does not probe or restore those files. It
does not substitute another ROOT, marker, configuration, nonce, archive, appcast,
key or signed bundle. The original marker expires **2026-10-24 00:56:25.164527 PDT**
and requires at least 240 seconds remaining at its original check. Renewal was
never authorized. Expiry blocks the frozen operation.

The closed one-shot, original terminal raw evidence and guards also block treating
this source as a permissible second invocation. The copied controller retains its
empty-raw and one-shot checks. Do not reset, move or delete those guards or raw
directories. Exact private artifacts must be available, and a viable new concrete
operation must receive fresh independent review and its own explicit consent.
This handoff supplies neither gate and grants no next runtime. It makes no claim
that retention alone would reopen the old operation.

## Static verification boundary

[check-static-files.py](check-static-files.py) is this task's own file-only reader.
Its [result](static-checks.json) records strict JSON parsing, Python AST parsing
without evaluation, exact Git/input/freeze preservation, field and AST equality,
helper literal bindings, and in-memory materialization of all three patches.
It never imports or executes any supplied controller/helper/checker. It performs
no behavior check. The supplied historical checkers were not run.

No runtime or end-to-end check is authorized. No compiler, fixture preparation,
signing, key operation, app/daemon/updater/installer launch, process/job/port/OS
query, lsregister invocation, test/mock/simulation, desktop driving or delegation
occurred. Web access was limited to official primary API documentation. Local
source, saved evidence, Git objects and SDK headers were read as data.

Live sharing, installed apps, production configuration/TLS/default IPC, TCC,
login items, services, tasks, accounts, firewall, global DNS and toolchain defaults
remain untouched. There was no TCP24800/UDP5353 action, Docker, credential or
existing signing-seed access, push or publication. The three provider-stopped
tasks and isolated Swift build gate remain stopped. API metadata proves neither
exec nor NSWorkspace delivery nor full security/registration restoration.
