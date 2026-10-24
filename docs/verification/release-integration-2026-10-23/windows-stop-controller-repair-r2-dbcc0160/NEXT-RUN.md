Finite future full-flow proposal. None of the commands below ran in this task.
The next step is the requested fresh independent source/evidence review. The
same already-authorized CLI scope applies after that review; this plan adds no
user-permission gate. The separate running Sparkle/relaunch decision does not
apply.

Use the retained verified executable. Its bytes were read locally as data and
match 4,139,520 bytes / SHA-256
`4984ef1222bbcd551dbebdda9db2fc623ff42a3560c5261806f09ad8f3dbda2f`.
The original private-location receipt identifies it at
`downloaded/target/x86_64-pc-windows-msvc/release/openergy.exe` under its retained
private root. The source tar, source manifest, original state metadata, and
immutable original build receipt are also available. The controller checks them
again before any remote setup. If a required byte sequence is unavailable or
differs, record a prerequisite failure and stop. Do not compile or recover by
running the old controller.

Materialize the reviewed text sources once in a new local run directory. This
keeps all output and the new local guard apart from the source proposal and old
evidence. Run from the existing task launch context with actual local nice 15.
The controller checks nice 15 and the command-local deployment target before
remote work. An unmet prerequisite consumes this new attempt; it does not permit
guard reset or retry.

```sh
set -eu
repair_root=/Users/serge/.t3/worktrees/openergy/t3-dbcc0160/docs/verification/release-integration-2026-10-23/windows-stop-controller-repair-r2-dbcc0160
mkdir "$repair_root/future-run"
for name in controller.py common.ps1 qualify.ps1 observer.cs runtime.ps1 postflight.ps1; do
  cp "$repair_root/source/$name.txt" "$repair_root/future-run/$name"
done
MACOSX_DEPLOYMENT_TARGET=15.0 /usr/bin/python3 "$repair_root/future-run/controller.py"
```

Do not use Python optimization (`-O`); the inherited controller uses assertions
for bindings and prerequisites. Bind the materialized source hashes to
`SHA256SUMS`, apart from the single declared `__NONCE__` replacement in transferred
runtime text. The future `workflow-bindings.json` records those transferred
bytes. Do not materialize or invoke `build.ps1`, the original controller, or any
other saved task helper.

The complete transport and native command path is finite:

1. The new local `reuse-controller-r2-once.guard` is created exclusively. One
   monotonic 2,400-second overall deadline begins. Every external stage has an
   exclusive label guard. The last 30 seconds are reserved for owned transport
   recovery; no stage gets a new overall timer. Old local guards, original Windows
   root names, and the selected old-fixture roots remain unchanged and are never
   reused. The old Windows runtime root remains absent.
2. The controller reads only the existing R6 transport-selection metadata to use
   the configured alias. It does not print the alias or access a key or secret.
   Preserve its fixed SSH options: BatchMode, 8-second connect ceiling, strict
   known-host checks, no known-host updates, no ControlMaster, no agent forwarding,
   and the existing keepalive settings. SSH control stdin is immediate EOF.
   Fixed-file `scp -O -q` transfers finish before stage control. There is no
   stdin script payload and no transport retry.
3. One setup control invokes native Windows PowerShell through that existing
   transport, sets its own BelowNormal priority, requires session 0, and creates
   only `oe-stop-win-r2-dbcc0160-e1ef124-<new nonce>` below the original task TEMP
   parent. An existing root is a refusal. Apply a protected current-user-only ACL
   to the new root and verify its owner and rules. Do not change an old root ACL.
   Require the installed PowerShell 7 and Windows tar paths. Do not run a compiler
   or inspect installed apps or credentials.
4. Transfer the five native sources, immutable 558-file source tar/manifest,
   original selected-fixture state metadata, immutable retained build receipt,
   exact executable as `openergy.exe`, and transfer manifest. The new private root
   supplies all future mutable state. The old build receipt keeps its old path
   and build-stage `executed=false` field unchanged. New runtime evidence records
   the actual new executable path and binary reuse separately.
5. Invoke `qualify.ps1` once through installed
   `C:\Program Files\PowerShell\7\pwsh.exe -NoLogo -NoProfile -NonInteractive
   -EncodedCommand <UTF16LE_BASE64_OF_REVIEWED_STAGE_WRAPPER>`. Each wrapper
   preserves the actual native exit code and first error text. Qualification
   checks the transfer hashes, native PowerShell source parse, immutable source
   tar and all 558 extracted read-only file bindings, and before snapshots of the
   original selected fixtures, registry, pinned toolchain, selected defaults/tool
   binaries, and live-sharing identity. It copies no registry, Cargo credentials,
   or global config. It runs no Rust/Cargo version or compiler command. A failed
   prerequisite ends the product path; collection can still run once.
6. Invoke `runtime.ps1` once through the same PowerShell 7 path. Its native
   acceptance ceiling remains 150 seconds. The outer wait is the derived
   150-second stage plus the existing 30-second transport-recovery allowance,
   capped by the one overall deadline. Compile/import only the reviewed private
   observer source as the original real flow requires. This is instrumentation,
   with no product rebuild. Verify the controller's real creation/start identity,
   parent chain, BelowNormal priority, session 0, and retained handle before it
   supplies `--parent-pid` to the product.
7. Before product startup, create private profile/appdata/localappdata/config/data/
   cache/IPC/temp paths and set all existing child-local overrides. Write the
   literal private config: role Off, `127.0.0.1:59367`, port 59367, empty client
   remote, autoconnect false, TLS off, clipboard/send/receive off. Set discovery
   disabled and the existing 24800 refusal override. Require fresh private daemon
   and observer pipe names. Check the literal port with the original exclusive
   IPv4/IPv6 loopback bind-only calls; never listen or connect. If occupied, fail
   this attempt. Do not choose a different port or retry.
8. Run exactly the following real product commands, in this order, with the exact
   transferred executable path. Every product process is created suspended with
   BelowNormal priority, no new console, the original explicit private output/NUL
   handles, no new job/group, and no desktop/window-station field. Register native
   process/thread handles before managed identity work. Require the direct
   controller parent, matching creation/start time and executable, actual
   BelowNormal/session 0, then resume once.

```text
<fresh root>\openergy.exe version
<fresh root>\openergy.exe status --socket <fresh private daemon pipe>
<fresh root>\openergy.exe daemon --config <fresh root>\runtime\config\config.toml --platform virtual --socket <fresh private daemon pipe> --parent-pid <retained controller PID> --foreground
```

Version must report `openergy 0.1.0` and exit 0. Absent-daemon status must exit 1,
have empty stdout, and retain the original error check. These cases stay in scope
because the original flow included them. The foreground daemon stays alive under
its retained controller parent.

9. Use one absolute 15-second readiness deadline from immediately before daemon
   resume. Read only the fresh active log with the reviewed bounded shared reader.
   Missing file can remain pending; other file faults fail explicitly. Accept no
   late marker. The marker and pipe-presence record do not establish health.
   Then run the remaining real public CLI commands once each. Their private
   observer forwards the original length-prefixed request/response bytes without
   changing them or creating a substitute response.

```text
<fresh root>\openergy.exe status --socket <fresh private observer pipe>
<fresh root>\openergy.exe ctl config.get --socket <fresh private observer pipe> --raw
<fresh root>\openergy.exe ctl fixture.invalid.c8a3489c --socket <fresh private observer pipe> --raw
```

Require healthy status: exact retained daemon PID, platform virtual, version
0.1.0, role Off, idle, no last error, no peers. Require the existing config checks
for private literal listen/port, autoconnect false, TLS off, clipboard off. Require
the original invalid-method exit 1 and exact `unknown_method` error. Preserve raw
wire JSON, matching non-null serialized ID, exact method, real client/server PIDs,
and QPC forwarding/receipt ticks for each exchange. One absolute 15-second CLI
deadline includes observer creation, child creation/identity/resume, CLI exit, and
observer completion; there is no extra observer wait. After those cases, retain
the original while-Off exclusive bind and the single native TCP/UDP inventory,
filtered to the owned daemon PID. A query fault is a failure, not an empty result.

10. Require enough native stage time for the existing 15-second CLI ceiling,
    10-second normal-stop allowance, and 15-second native recovery allowance.
    Issue one stop command, with no second stop path:

```text
<fresh root>\openergy.exe ctl daemon.stop --socket <fresh private observer pipe> --raw
```

Require CLI exit 0, empty stderr, exact stdout `{"stopping":true}`, a real matching
stop response/wire ID from the retained daemon PID, and independent retained OS
handle signal with daemon exit 0. The controller must still be alive with its
original start identity. The original daemon generation must be gone; PID reuse
alone is not a remaining owned process. The first OS-exit observation must precede
the absolute 10-second deadline from the observer's pre-forward tick. Save both
forwarding ticks and response-receipt/exit-observation ticks; do not call these
kernel send or kernel exit timestamps. Do not claim a one-second contract.

11. Any failed case fails the flow. Cancel/settle the observer and snapshot its
    final forwarding tick before recovery. If forwarding began, preserve the full
    10-second normal-stop allowance, even if the CLI or observer already failed.
    Never resend stop. Recover only the exact owned retained handle with matching
    executable, creation identity, and direct parent; record the reason, kill
    tick, signal, actual exit code, and handle closure. An unresumed owned child
    uses its original native handle. An identity mismatch or unsettled observer
    means no unsafe kill and unresolved cleanup, never PASS. Use the existing
    recovery ceilings; do not restart the overall timer. The transport-timeout
    fallback is also one-shot and checks exact encoded owner plus saved descendant
    generations, retains and checks handles, records exit codes, and has no
    recursive recovery. Forced exit never substitutes for normal exit acceptance.
12. Run the existing collection stage once in PowerShell 7. Save after-source and
    selected shared snapshots, compare them to before bytes, audit the new root's
    ACL/reparse state, and audit all saved runtime process generations and the two
    private pipes after the runtime controller has exited. Keep raw first errors
    and failed results even when cleanup passes. Archive the receipts, original
    build receipt, and exact executable. Download once, check the archive hash,
    reject unsafe/duplicate members as the saved controller does, verify every
    evidence member and the executable/receipt hashes, then permit removal of
    only the exact new root under its original TEMP parent with no owned process
    left. Do not remove old fixtures or reset an old guard. If collection or its
    bindings fail, retain the new root and report it; do not repeat a flow.
13. Capture the real outer controller exit code. Bind the new source/transfer/
    artifact hashes and before/after preservation records. Report full-flow PASS
    only if every required real acceptance and cleanup receipt passes. Otherwise
    report the exact failed stage and retain normal-stop/timing as UNVERIFIED.
    Never change the original FAILED / UNVERIFIED evidence. No source-only check
    in this handoff is runtime acceptance.

Keep all existing boundaries: live sharing and installed apps/config/TLS/default
IPC stay intact; no TCC/login/service/task/account/firewall/global-DNS/toolchain
default change; no TCP 24800/UDP 5353, discovery traffic, native input/clipboard,
desktop/window-station change, new job/process group, Docker, credential/key
access, push, or publication. The three provider-stopped tasks and isolated Swift
build gate stay stopped. No optional suite or cross-platform performance claim
is part of this one flow.
