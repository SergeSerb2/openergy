# Finite later operation, after independent static review

This is a plan, not a run record. None of these operations ran in this repair.
The root will request independent static review of the committed source,
patch, first-failure audit, and ownership design. The ordinary no-render
`--e2e` scope is already authorized; this adds no new human approval step.

1. Review the exact commit and `source-checks.json`. Resolve concrete static
   findings before execution. Verify the four original-file patch bases
   against `498190feb00f51039f500ae618ac7de5fb113d86` and all new support
   modules. Keep every original raw file, first stream, failure, guard, config,
   port result and report unchanged. Do not reuse the original runtime root.
2. Run the new `prepare.py` once, as file-only preparation. It creates an
   exclusive preparation guard and fresh random `/private/tmp/oe-ordinary-432ee8-*`
   root at 0700, with fresh HOME, temporary/cache, peer, and app directories.
   It must refuse an existing prepared receipt, missing/changed bundle, source
   mismatch, symlink escape, extra declared input, or copy/tree mismatch. A
   partial preparation is failed evidence; do not retry it under the same
   guard. No build, re-sign, key, compiler, architecture command, product
   command, process query, network operation or port bind is part of this step.
3. Bind all current app inputs and the exact 397-file daemon input set by path,
   size and SHA-256; retain the Swift `7287e9` and daemon `2485c14` build lineage,
   audited original compiler streams, Rust 1.99.1/jobs 2/deployment 15.0 record,
   124-record pinned Sparkle inventory, and framework resolution. If a compiler
   is needed, stop this plan; compiling is outside this source-only repair and
   this reuse plan. The complete ordinary signed bundle must match the saved
   tree before copy, after copy, and again before launch. Input binary hashes
   are app `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`
   and daemon `23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb`.
   Exact copied signed execution hashes must remain app
   `53ba6f431811eb1178c39b8b7ad7cdb425b0088e7c893b8dcab8e601ad338c23` and
   daemon/peer `1c558733a56ada70661f8a8493eae7deaf3813ce1bb572d0592fe86900b6b971`.
   This is the retained arm64 bundle, not universal proof. Copy all nested
   framework/helpers/license/signature files and symlinks unchanged. Require
   normal `dev.openergy.app`, `CFBundleExecutable=Openergy`, no private fixture
   metadata, and no private fixture launcher. Do not sign again.
4. Capture the new controller's first stdout/stderr and OS exit status in new
   exclusive files. Run it once with `/opt/homebrew/bin/python3`, with bytecode
   writing disabled. It creates its own exclusive runtime guard and verifies
   the committed controller hashes, source and signed tree. It checks private
   ownership/modes and leaves app/config absent for the unchanged harness to
   create its safe Off config. The watchdog must arm and retain its actual
   UID/path/argv/PID/start/group/NI15 before any product launch. Failure to arm
   permits no product launch. All products receive private HOME/config/TLS/
   socket/temp paths, virtual platform, disabled discovery, and denial of
   port 24800 before exec. No daemon override or optional app routine is used.
5. Within the 120 s flow budget, retain the first port-filtered TCP state and
   first exclusive bind result for literal loopback ports 65431 and 65432.
   Hold each reservation until its corresponding launch. No SO_REUSE*, retry,
   fallback port, old port 65371/65372, TCP 24800, UDP 5353, non-loopback peer,
   production IPC or discovery traffic is permitted. A busy port fails this
   single attempt before product launch where possible. Launch exactly one
   peer, confirm its actual owned PID and virtual platform through one status
   CLI, then launch one ordinary app with exactly `--e2e <new-app-dir> --join
   127.0.0.1:65431 --host-port 65432 --require-peer`. No render, private update
   fixture, caller-cancellation mode, update CLI, external daemon, or retry.
6. Observe all real identities and actual nice 15 throughout peer readiness,
   CLI waits, the complete unchanged 25-check routine, its private SIGSTOP/
   stalled-write interval and crash/recovery, and cleanup. Retain partial rows
   and first read races before validation. Require separate initial/recovered
   daemon PID/start/group records with exact current parent ownership. Missing
   stop-state coverage, a missing recovered identity, unexpected process,
   wrong priority, source/tree change, observation/byte/round limit, app/peer
   premature exit, CLI failure or deadline initiates explicit whole-flow abort.
   It cannot silently fall back to a partial success. The independent watchdog
   stays armed; the direct-child collector keeps the app's complete OS exit
   status. Do not infer exit status from Swift `exit()` source or report text.
7. Keep the peer alive until the app finishes, or until declared abort first
   terminates the exact owned app/children and observes their absence. Apply
   the design's existing 10 s normal grace and verified CONT/TERM, 5 s TERM
   grace, KILL and 5 s absence waits, all within the single absolute 60 s
   cleanup budget. No unrelated group signal or uncertain/reused PID signal.
   Stop the peer once through its private public CLI only after the app tree
   is absent. Retain all direct product/CLI/helper OS exit statuses. Require
   absence for every known initial/recovered PID/start and actual group, all
   wrapper stages, and both app/peer socket paths. A stale socket fails; do not
   remove it to pass. Cancel no watchdog on an exception. It may finish only
   on complete absence plus collected exits, or explicit failed bounded recovery.
8. After verified absence, collect watchdog exit/identity/group absence, first
   postflight exclusive observations of both new ports, the complete raw app
   report and first logs, controller failure records, signal ownership receipts,
   shared limits, and complete unchanged signed bundle inventory. Require app
   OS exit 0, exactly 25 unchanged passing checks, two daemon start identities,
   actual NI15 coverage, all product/CLI exits, no unknown owned groups/sockets,
   correct first port observations, and successful bounded cleanup for ordinary
   acceptance. Any failure stays FAILED / NOT ACCEPTED, even if some checks
   passed. If cleanup remains uncertain, retain the explicit failed outcome,
   guard, private root and partial evidence; do not bind ports or claim absence.
   Preserve later report text as aborted/interfered evidence where applicable,
   never as proof of a product connection bug. Do not retry the flow.

After the later operation, source review, inherited build audit, actual ordinary
runtime evidence, and any missing proof must be reported separately. Native
UI/input/clipboard, signing credentials, running-update fixture/replacement,
trusted release, and all three provider-stopped tasks remain outside this plan.
