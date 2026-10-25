# Strict postflight bind: inert diagnostic repair

2026-10-25, America/Vancouver. Base `b221d9807cbc4c3db5a30f2114399ce20942dcf1`.
Owner: `t3-619a42c8`; isolated branch `t3/capture-bind-failure-diagnostics`.

**SOURCE ONLY. Runtime UNVERIFIED. Historical M1 attempt FAILED / NOT ACCEPTED.
Cause UNKNOWN.** This changes diagnostic source, not the unknown port cause.
No helper, supplied source, query, fixture, build, product or prior private run
was executed, imported, prepared or reopened. No tests, mocks, computer use,
remote access, credentials, process/port probes or child agents were used.

[failure-diagnostic.patch](failure-diagnostic.patch) applies to the exact R9
controller after the unchanged M1 admission patch. It changes only
`Controller.port`, adds controller imports, and adds `port_diagnostic.query`
and `port_diagnostic.socket_rows`. The complete changed texts are inert
`.py.txt` files in [source](source). No existing source or evidence file changes.
There is no overlap with the other agent's priority functions.

The first exception still stops postflight before port65432 and archival.
The original pre-bind query and snapshot stay intact. Before any failure
receipt, the controller marks failure and retains the bind exception metadata
in memory. An exclusive best-effort first receipt keeps the original filename.
Only a failed postflight bind with `errno.EADDRINUSE` can start one extra
read-only `/usr/sbin/netstat -AanW -p tcp` command. It records bounded selected
socket rows and command metadata before the original exception is rethrown.
There is no second bind, alternate port, reuse option, sleep, flow retry or
new cleanup authority.

The diagnostic query uses at most 0.5 seconds inside the caller's unchanged
port deadline. It reserves time for its own directly owned helper kill/reap,
as clarified by the user, and for publication. It cannot extend flow, cleanup,
postflight or archival deadlines. Its new receipt uses only existing regular
evidence capacity. Missing time, space, output or reliable parsing means
`INCOMPLETE`. No diagnostic result changes the first bind's `FAILED` status.

[design.md](design.md) describes the evidence and limits.
[operation.plan.json](operation.plan.json) is a bounded future plan with
`execution=false`; it is not a launch receipt or new runtime authorization.
[input-bindings.json](input-bindings.json) binds exact source and review inputs.
[source-checks.json](source-checks.json) binds the nine proposed runtime modules,
including the seven unchanged M1 modules. [static-checks.json](static-checks.json)
records AST, JSON, hash and exact patch-materialization checks only.
The first staged whitespace check failed on a blank patch context line;
[the failure record](static-whitespace-first-error.txt) remains. The final
zero-context patch retains exactly the same changed source texts.
[SHA256SUMS](SHA256SUMS) covers every package file except itself; the commit
binds that final manifest. Hooks and signing are disabled for the commit.

The saved app's 25/25 and exit0 do not accept the failed controller attempt.
The older 23/25 failure, isolation failures, missing priority coverage, Windows
native stop, expired Sparkle one-shot and all unverified release/native gates
remain unchanged. No TypeScript changed. The parent owns independent review
and any later authorized fresh operation; this folder grants no retry or consent.
