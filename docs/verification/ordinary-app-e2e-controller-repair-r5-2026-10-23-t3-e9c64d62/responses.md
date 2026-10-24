# Response to all seven R4 P2 findings

All are accepted. This is an implementation response based on source reading.
It is not a fresh process observation, runtime reproduction, or acceptance run.
The exact patch is `repair-r4-to-r5.patch`; the file/function anchors below
identify its changes without depending on shifting line numbers.

1. **P2-1: actual interpreter and launch path.** `ownership.wrapper_argv` now
   uses the independently observed controller `python_executable` for wrapper
   argv0. `exact_argv` still requires the exact argument count and remaining
   tokens; product matching is unchanged. `Controller.run` records the actual
   interpreter executable/hash separately from `python_launch_binding` (the
   selected `/opt/homebrew/bin/python3`, its resolved launcher, and its hash).
   `_launch` checks both file bindings before each launch. The real watchdog
   argv0 must resolve to the actual interpreter. The framework launcher and
   Python.app executable need not be the same file. No current Python process
   identity was queried by this task.

2. **P2-2: same-start exec and reuse privacy.** `macproc.detail` stages two
   executable reads and two argv reads, then checks the final BSD identity.
   Both pairs must agree, and canonical argv0 must agree with the executable.
   Nothing mutates the caller's metadata row until all checks pass. A mixed
   stage yields metadata-only `read_race`, which uses the existing eight-read/
   two-second pending allowance. A changed PID/start discards all staged
   fields. Read/decode error text contains only a type name, never staged argv
   bytes. The snapshot and targeted callers assign only the returned coherent
   row. No interleaving was simulated.

3. **P2-3: evidence cannot gate collection or recovery.** `Controller.collect`
   polls every retained product/CLI handle and the watchdog before any write;
   it stores exits in memory. It then publishes each exit independently.
   `Evidence.attempt` retains bounded error state, and acceptance fails on any
   such error. `abort` and `request_cleanup` set in-memory state before I/O.
   One inherited nonblocking stdin pipe carries at most 512 bytes to wake the
   independent watchdog even when the evidence sink is unusable. It conveys
   a deadline/request, never ownership. Inventory keeps fresh data and owned
   identities before publication. Failed sample publication still permits
   fresh ownership validation; targeted recovery has independent failure
   boundaries. Pre/post-signal receipt errors cannot suppress a verified
   signal. Existing first receipts return under the common lock before any
   reservation, temporary write, or ledger access. Pipe logs continue to drain
   and discard after their first sink failure. No failed evidence can PASS.

4. **P2-4: incomplete post-spawn ledger.** The exclusive `*-command.json` is a
   durable pre-Popen intent. `launch_attempts` also retains every attempt in
   memory. Handles enter `processes` before post-spawn writes. A failed spawn
   receipt aborts; the handle still gets polled. A new immutable
   `collector-terminal.json` is possible only after further launches are
   sealed and every retained handle has a real OS exit. Recovery requires
   equality of intent labels, sealed retained-handle labels, spawn labels and
   exit labels, with exact PIDs and collector identity. An intent without a
   verified identity also prevents absence. The controller independently
   checks its retained handles before accepting a terminal result. Missing
   spawn/exit receipts or a lost controller cannot yield vacuous recovered:true.
   Unverified wrappers get no signal and no role can launch twice.

5. **P2-5: known gated exec after controller loss.** `Inventory.identify` accepts
   a new product stage for the exact previously gated wrapper start only when
   its saved wrapper gate matches role, parent identity, UID, NI15, actual
   group, interpreter and exact wrapper argv. The current product must match
   exact UID/start/group/executable/argv. It must either have the live matched
   controller or PPID 1 with explicit ESRCH for that controller PID. Enumeration
   omission or a reused controller PID is insufficient. Existing verified
   orphan transitions also require explicit parent ESRCH. No unknown orphan
   can gain ownership by group or `--parent-pid`. Targeted recovery uses this
   same policy. Direct OS exit receipts remain mandatory after controller loss.

6. **P2-6: TERM grace after actual delivery.** `Inventory.signal` returns a
   successful-delivery receipt and stores it in memory before post-publication.
   A tiny SIGALRM mask covers only the single kill syscall and in-memory time
   record. `Recovery.stop` keeps TERM and KILL times per PID/start, shared by
   normal recovery, targeted recovery and CLI timeout handling. KILL requires
   five actual seconds after TERM, and a later advance round. No phase-entry
   timer or absolute CLI deadline authorizes escalation. Late TERM or KILL
   with insufficient grace/absence time records an explicit bounded failure;
   it never extends the cleanup deadline or shortens TERM grace. Phase changes
   use one decision per advance. No process/timer behavior was exercised.

7. **P2-7: aggregate bytes and terminal space.** `common.initialize_bytes`,
   `byte_lock`, `charge`, and `write_bytes` use one bounded, nonblocking
   cross-process lock and a fail-closed ledger. Each writer durably reserves
   bytes before growth. Failed/partial writes keep their charge. Exclusive
   records reserve twice their size for temporary/final names. Initial source
   and evidence bytes are included. Products never write stdout to regular
   files directly: finite nonblocking pipe reads enter the same ledger.
   Seven private rotating-log slots plus the raw report each hold 2 MiB,
   enforced by inherited RLIMIT_FSIZE before product exec. The full 16 MiB
   reservation precedes startup; it is not a sampled sum. The 2 MiB product
   allocation is a subordinate limit under the unchanged 8 MiB ceiling, and
   can explicitly fail an attempt. It accounts for both log rotations, which
   the prior source omitted. The 4 MiB cleanup reservation includes a protected
   1 MiB terminal share; streaming logs cannot spend it. Accounting metadata
   has a separate fixed 64 KiB allowance. Archival renames reserved files only
   after absence, on the same filesystem; no second copy or EXDEV fallback.
   Non-reserved config/report records use normal reservations. Capacity or sink
   failure follows P2-3 and cannot disable polling or ownership recovery.

All unchanged private scope, default25 checks, IPC limits and routine waits
remain source-bound. The original failure and every raw archive byte remain
FAILED / NOT ACCEPTED. Real identities, ABI, priority coverage, exits and
cleanup must still be established by the later single ordinary flow.
