# NI15 controller coverage repair: source package

2026-10-25, America/Vancouver. **Runtime UNVERIFIED. Independent review pending.**
Owner: isolated checkout `/Users/serge/.t3/worktrees/openergy/t3-a6d1c0b1`,
branch `t3/ni15-controller-coverage`, clean base
`b221d9807cbc4c3db5a30f2114399ce20942dcf1`.

This package improves acquisition, then checks required sample coverage. After
each existing exec gate, a serialized Inventory transaction promptly reads the
known PID and controller in process. It records each real row before it starts
the slower full-snapshot helper. It observes the unchanged wrapper until it
sees a verified product, explicit ESRCH, or a bound ends. It adds no product
delay, launcher switch, new process, launch retry, ABI or product-source change.
An ultra-short product can still escape observation. That stage stays
**UNVERIFIED** and cannot pass the coverage gate.

Full observations now retain controller and watchdog detail rows, including
flow completion and cleanup's `all-exits` phase. `command()` observes its real
helper child before the first blocking wait and after timed wait slices. It
retains command intent, PID/start, exact executable/argv, both nice readings,
and actual OS exit. Helper observations grant no product ownership or signal
authority. Short helpers can also escape observation; their gaps fail coverage.

The base is M1 admission candidate
`02c9b174c8e13d4eaa6bf7692f20cbb44c93ae57`, reconstructed from R9
`e34401c8842c2a93b5c355f4c515a79ee05cfa72` plus all seven admission patch hunks.
`source/` holds nine complete inert `.py.txt` files. Four are unchanged from
the admission candidate: `exec_owned.py`, `macproc.py`, `prepare.py`, `probe.py`.
`repair-from-admission.patch` is the precise source-name delta, including the
new coverage module. `base-modules.json`, `input-bindings.json`,
`source-checks.json` and `SHA256SUMS` bind inputs, outputs and static checks.
`manifest.json` binds every package file except itself and `SHA256SUMS`;
`SHA256SUMS` also binds the manifest. The Git commit binds the whole package.

Read `design.md`, `responses.md`, `future-flow-plan.json` and `NEXT-RUN.md`.
The plan has `execution=false`. It is a finite future handoff, not a receipt
that can admit a run. The parent independently reviews this source package.
Ordinary source work has no new human approval gate.

The saved M1 run remains **FAILED_NOT_ACCEPTED**: app25/25 and exit0,
controller exit1 at its first strict postflight bind65431 with EADDRINUSE.
The cause is unknown. This repair establishes no cause or port fix. Its
source-only checks do not accept that run. The older ordinary23/25 failure,
isolation failures, closed Sparkle one-shot and other stopped scopes remain
unchanged. Samples never establish continuous priority, universal containment,
host restoration, native UI/input, release trust or matched performance.

All writes are confined to this folder. No supplied source/module/checker or
AST was imported, evaluated or executed. No runtime, build, product/helper
launch, probe, process/port query, remote access, fixture preparation,
credentials, computer use, unit test, mock or child agent ran. No private run
was reopened. Checks used only new standard-library data readers for Git/file
bytes, JSON, AST structure, hashes and patch materialization. No TypeScript
changed. No AGENTS.md, CONTEXT or ADR file was present in the checked ancestors
or repository inventory. Hooks and signing are disabled for this commit.
