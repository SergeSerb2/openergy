# Ownership and failure design

## Source contracts

All product pointers below refer to source root
`059f052f965bb942d104448b064a9d0e68b1fda1`. The saved app inventory binds all
22 target Swift files, Package.swift, Package.resolved, and normal source
Info.plist to the reviewed `7287e9aa4caf6570fc4116a89908d548a258f302` build.
The saved daemon inventory binds the complete declared 397-file set to
`2485c1425e4d2f414eb22742dfd73f355d47390c`. The differing old bundle.sh was
not a Swift compile input and did not run in the original ordinary packaging.
The later preparation binds its current bytes separately and never runs it.

- `apps/macos/Sources/Openergy/App/OpenergyApp.swift:8,28-36` dispatches private
  metadata first, then ordinary `--e2e` before normal SwiftUI application setup.
- `App/AppEndToEnd.swift:37-61,457-478` validates private paths, exclusively
  creates safe Off config, then constructs/starts the real model. The controller
  must leave `app/config` absent. No config callback or product launcher is
  substituted.
- `App/AppEndToEnd.swift:44-45,261,495-500` disables rendering and uses the
  ordinary no-render run loop. AppModel's `--e2e` system-integration guard and
  AppUpdater's normal-start guard remain unchanged.
- `Daemon/DaemonSupervisor.swift:43-48,65-74,93` resolves the bundled helper and
  supplies `daemon --parent-pid <app> --socket <private> --config <private>
  --platform virtual`, with the complete app environment. Foundation `Process`
  retains the actual child PID. This source makes no promise that its PGID
  equals the app PID.
- `DaemonSupervisor.swift:105-120` has a 1.5 s product stop grace. Its restart
  starts at 500 ms, with bounded backoff. These product behaviors do not imply
  a one-second OS exit guarantee.
- `AppEndToEnd.swift:195-238` performs its own SIGSTOP, real stalled IPC write,
  SIGCONT, SIGKILL, invalidation, and supervised recovery. The default routine,
  1,000,000-byte payload, 300 ms request timeout, 1 s responsiveness, 2 s bound,
  3 s invalidation, and 15 s recovery waits are unchanged. The controller only
  observes these steps. It must observe the stopped state and both identities
  or fail the evidence attempt.
- `Daemon/IPCSocket.swift:31` retains the 16 MiB frame limit. `crates/daemon/src/
  adapter.rs:16-23,50-62` selects virtual platform and excludes discovery.
  Explicit denial of port 24800 is present before every product exec.
- `crates/daemon/src/lifetime.rs:31-48,102-136,165-188` installs signal handlers
  and watches parent exit on macOS. `runtime.rs:1508,1541,1646-1664` enters full
  shutdown on stop/signal/parent exit, including worker joins. A public stop
  acknowledgement is not proof of process exit. `ipc_server.rs:60-69` removes
  only the socket inode that the daemon owned. A leftover socket fails cleanup;
  this controller never unlinks it to manufacture absence.

The process ABI is read from the installed macOS SDK's `sys/proc_info.h`
(`proc_bsdinfo`, `PROC_PIDTBSDINFO=3`) and `sys/sysctl.h` (`KERN_PROCARGS2=49`).
No network lookup or runtime ABI call ran in this repair. `macproc.py` declares
that layout but needs later real validation. AST parsing is not ABI validation.

## Identity records

The key is `(PID, start_tvsec, start_tvusec)`, not PID or PGID alone. Every
complete row records effective/real UID, PPID, actual PGID, status, both start
fields, actual nice values before/after detail reads, canonical executable,
and tokenized kernel argv. Detail reads are bracketed by BSD identity reads.
Known PID/start tokens are checked before argv reads. A reused PID retains
metadata and an explicit reuse failure; the stranger's argv is not captured.
Canonicalization handles `/tmp` versus `/private/tmp`; argv tokens after argv0
must match exactly. Private config/socket/virtual/parent flags cannot match by
substring.

Each externally launched peer, app, and CLI is a direct controller child.
`exec_owned.py` sets and checks absolute nice 15, hashes the target executable,
writes a same-PID start receipt, and waits behind a finite registration gate.
The controller matches a real post-spawn Python wrapper to the receipt, its
retained Popen PID, exact helper argv, controller UID/path/argv/start, then
opens the gate. The wrapper execs the exact product argv and environment. It
does not add relative nice increments. Wrapper and product stages retain the
same start identity as separate records. The actual Python executable path
comes from the running controller; the Homebrew launcher symlink is not
assumed to equal the kernel's Python.app executable path.
The wrapper clears its own I/O alarm immediately before exec, because
ITIMER_REAL survives exec and caught signal handlers do not. The independent
watchdog process remains armed; no helper alarm is passed to the product.

A new supervised daemon must have the canonical executable under this new
bundle, exact daemon argv/private config/socket/virtual flags, an independently
observed current app parent with matching UID/executable/argv/start, and a
start time no earlier than that parent. Its PGID is recorded without requiring
any relation to the app PID. The first and routine-recovered daemon remain
distinct start identities. Exactly two must be observed for ordinary success.
An unseen orphan is never adopted from `--parent-pid` or a group. A previously
verified child may be recorded as reparented to PID 1 after its original
parent's absence; new ownership is not inferred from that transition.

Group matches expand final inventory only. They never confer ownership and
never authorize a group signal. Unexpected or reused PID/group occupancy
fails absence checks. No stranger is terminated to clear a group. Official
Sparkle helper priority exceptions remain intact: `--e2e` does not start the
normal updater, and this controller neither renices nor signals helpers from
group membership. An unexpected child prevents ordinary acceptance.

`Inventory.signal` performs a new independent real snapshot before each
single-PID CONT/TERM/KILL. It checks UID, executable, exact argv, start, actual
group, and the matched current parent/start (or an already verified orphan
whose original parent is absent). It refuses partial reads, reused PIDs and
changed identities. There is no Popen terminate/kill fallback for products.
The bounded read-only probe/tool subprocess is an unreaped direct child and
may be killed/reaped by its own Popen handle if its read times out.

macOS does not provide Linux pidfd-send-signal here. The final verified read
and single-PID signal are separate OS calls. This source does not claim an
atomic kernel lease across that interval. All observed identity mismatches
refuse the signal and remain failed evidence. Independent review must assess
this platform limit; no runtime proof is inferred from the source checks.

## Failure architecture

1. The normal observer publishes selected partial raw rows before ownership
   and priority validation. Transient detail races are retained and retried
   for at most eight observations or two seconds. Missing/denied/short reads
   are uncertainty. Only a separate exact ESRCH query proves a known PID
   absent. A PID-list omission is insufficient.
2. Main observation failure retains its first exception and rows, writes an
   explicit whole-flow abort, and requests cleanup. It disables that failed
   normal sampler. It does not stop the peer or cancel the watchdog.
3. The detached watchdog is armed before the first product launch. It has the
   immutable absolute flow deadline of start + 120 s and a hard end of start
   + 180 s. It survives controller exceptions. A missing request at 120 s
   triggers its own declared abort. Its recovery observer is a separate
   Inventory instance and read-only probe process, not a callback to the failed
   main sampler. Recovery observation errors are recorded without unwinding
   the cleanup state machine.
4. On abort, recovery sends verified TERM to the app first, after CONT if
   observed stopped. It allows five seconds before verified KILL, then five
   seconds for absence. This stops the supervisor before child cleanup. It
   next handles each exact supervised child with the same TERM/KILL grace.
   Under normal completion, it first gives app/children ten seconds to finish
   normal shutdown. Expiry explicitly aborts before controller termination.
5. The peer stays alive until fresh validated observation establishes that
   the owned app and children are absent. Only then is public `daemon.stop`
   permitted once on the private socket. The main collector launches that
   CLI and continues nonblocking exit collection. Recovery observation stays
   active during this CLI wait. Public-stop acknowledgement is retained but
   does not advance absence. The peer gets ten seconds, then verified TERM
   with five seconds before KILL and five seconds for absence, all clamped to
   the original cleanup deadline. No flow or public-stop retry occurs.
6. Each CLI has an absolute five-second operation deadline. A timed-out CLI
   gets exact-identity TERM then a five-second grace before KILL. It cannot
   strand the all-exits phase. Recovery does not terminate the peer merely
   because a CLI wait failed while the app is active.
7. The controller continues `Popen.poll()` exit collection during abort and
   cleanup even when observation failed. Each direct product/helper child has
   a durable OS exit code, signal number when applicable, and collection time.
   The supervised daemons are the app's children: the controller records
   their start/absence, not an invented waitpid status. The app's complete
   direct-child OS status is mandatory. If the controller itself dies, the
   watchdog cannot reap its former children; missing receipts force failed
   bounded recovery even if products later disappear.
8. The watchdog exits only after all known PID/start identities and actual
   groups are absent, both private sockets are absent, and all direct-child
   exits are collected, or after an explicit failed bounded-recovery outcome.
   The parent does not cancel it. Unexpected watchdog exit causes separate
   parent fallback recovery and continued collection; the flow remains failed.
   Watchdog PID/start/path/argv/group/UID/nice, exit code and group absence are
   separately recorded. Its own terminal record has a one-second file-write
   budget, and the parent has a three-second control-child reap/receipt budget;
   neither permits further product execution or extends product cleanup.

If recovery enumeration fails, recorded owned identities remain available
for targeted reads that bypass PID enumeration. Those reads can register a
known wrapper's product stage and perform fresh pre-signal verification.
This route retains app-first TERM/KILL grace and then handles
known supervised children. That route cannot certify
absence or permit peer teardown from stale rows. Continuing uncertainty ends
as failed bounded recovery with the guard and partial evidence intact. No
failed observation, deadline, signal, CLI response, or empty report can become
PASS. Peer teardown after declared abort is labelled controller interference;
any later report is retained as partial/aborted evidence, not a product
connection regression.

## Limits and I/O

The old 6000-round/24000-row caps are shared through a nonblocking advisory
file lock and bounded counter writes. Main observations stop at 4976 rounds
or 19904 rows, preserving 1024 rounds/4096 rows for recovery. The decisive
main row is written before the main-budget failure. Exhaustion is failure,
not a sampling gap labelled success. Retained signal-verification receipts
are control-action evidence, as in the original controller, not sample rows.

Every explicit file/pipe/process operation has an absolute monotonic deadline.
File operations use main-thread SIGALRM scopes. Read-only process probes have
their own alarm and parent pipe timeout; part of that same budget is reserved
for killing/reaping the unreaped probe child. Product waits use poll loops,
never an unbounded CLI wait. Evidence is exclusive/atomic for first records;
only counters are replaced under a lock. Product files have RLIMIT_FSIZE=8 MiB.
Evidence writers enforce the 8 MiB stream and 32 MiB aggregate bounds. Failed
evidence writes cannot create a successful cleanup record. Kernel scheduling
or an uninterruptible OS call is not claimed as a hard real-time guarantee;
independent watchdog failure remains an explicit failed outcome.

The frame size, default routine waits and product shutdown code are unchanged.
There is no new one-second product exit claim. The one-second probe deadline
is an observation I/O budget and can itself fail the flow.
