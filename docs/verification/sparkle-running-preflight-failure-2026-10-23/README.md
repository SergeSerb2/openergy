The exact approved operation **failed at live preflight and exited 1**. The
LaunchServices read-only `lsregister -dump` exceeded its unchanged five-second
command limit. No target, daemon, updater, installer or relaunch started.
No bind for the fixture HTTP server, HTTP request, IPC request or normal app
quit occurred. Runtime acceptance remains **UNVERIFIED**. No retry ran.

The user selected **“Approve this exact one-shot run”** at
2026-10-23 21:01:00 PDT. That approval binds the frozen proposal, controller,
scope and separate independent R6 review receipt. It does not reuse the
17:16:55 stopped-target approval. The actual approval receipt is preserved in
[raw/approval.json](raw/approval.json), SHA-256
`af28e5dfd12c05f9a464bf4180e9e7361d5fb85f5510cae70a1eb9946b8abeee`.
The earlier [approval request](../sparkle-running-approval-2026-10-23/README.md)
and its historical false fields remain unchanged.

The one invocation used the original cleared controller at
`/Users/serge/.t3/worktrees/openergy/t3-6133e534/docs/verification/sparkle-running-repair-2026-10-23-t3-6133e534-r2/controller-DO-NOT-RUN.py.txt`.
No supplied code was edited, copied into a different execution location or
patched at runtime. The cleared identities remain:

- Controller: `a934611e4bef5d4b355ae11279d6b1aaef0dadf665503efcb788df4869453567`.
- Proposal: `cb15b124d6c76c77bad90d05bcc4ea1384aa55aaac48c959bb03ff3cfceba41d`.
- Separate R6 review receipt: `6af6a1df11c504917b8631e502491135f723b19c6bdcd956c5e78dc56f5adfc4`.

The command used `/opt/homebrew/bin/python3`, observed as Python 3.15.0,
through the reviewed `tools/release/priority.py -- COMMAND` launcher. It sets
and checks absolute nice 15 before exec. The controller also sets and checks
15 before preflight. There is no independent per-PID priority sample for this
short attempt or its metadata readers. No product priority observation exists.
The interpreter selection supports the unchanged `hashlib.file_digest` call;
no toolchain or dependency changed.

The [invocation](raw/invocation.json), empty [stdout](raw/controller.stdout),
complete [controller stderr](raw/controller.stderr), and
[result](result.json) bind the attempt. Unified exec session 15053 returned
exit code 1. There is no independently captured kernel exit timestamp or
whole-preflight start timestamp. The failed command's timeout value was
4.99583949999942 seconds, inside its five-second absolute deadline. That is
not the entire operation duration.

Seven controller JSON records survive, starting with eight absent fixture
cache roots. Four exact-job queries return 113, meaning the two fixture jobs
were absent in those preflight observations. The scoped-port lsof query
returns 1 with no stdout and a private mount warning. Its completeness is
limited. The next record is the LaunchServices timeout. The command's partial
registry output was not saved by the controller and is not reconstructed.

The exact traceback stops at `main` line 972, before the server bind,
`runtime-once.lock`, ledger/timer/thread startup and `launch_once` at line 999.
This control-flow ordering, together with absent initial/CLI stream files,
receipts, socket and runtime lock, supports zero product launch attempts.
It is not a host-wide process-absence guarantee. The controller's direct
read-only child timeout path kills that child and attempts bounded drain/reap.
Its PID and exit status were not independently saved. No broader metadata-child
cleanup or full shared OS-state restoration is claimed.

The [selected file postflight](selected-file-postflight.json) rereads six
frozen private file hashes, all still exact. Target build remains 1 and
candidate remains 2. The five private writable directories have zero immediate
entries; the private socket and runtime lock are absent. These checks use
file reads only. No postfailure product, process, job, port or LaunchServices
probe was run. The marker, archive, appcast, public key, configuration and
launch lock were not repaired, renewed or changed.

[copy-ledger.json](copy-ledger.json) binds all 12 private originals to their
exports. Eleven exports are byte-exact. One derived preflight JSON replaces
the entire lsof stderr warning to remove private mount details. Its original
bytes and hash remain private and unchanged. See
[privacy-audit.json](privacy-audit.json). No unfiltered LaunchServices dump,
credential, signing seed or existing private key was exported. The outer
guard and original seven records remain terminal and are not reset.

This closes the one approved invocation. It supplies no successful live
preflight, update, no-argument startup, old-child exit, replacement, relaunch,
new readiness, final quit, helper/cache cleanup, visual AppUpdater or trusted
distribution acceptance. No source repair or second runtime is included.
The rest of the authorized project work continues separately.
