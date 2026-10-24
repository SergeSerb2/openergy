# R5 ownership and bounded recovery contracts

## Identity and exec

The complete identity is PID, BSD microsecond start, real/effective UID,
actual PGID, parent PID/start, executable and exact tokenized argv. A group
never grants ownership or signal authority. Product argv and all private
path, virtual, parent and socket arguments remain exact. NI is assigned
absolutely to 15 and observed before/after detail reads. No relative nice
wrapper is introduced. The actual Python.app interpreter identity and hash
are separate from the selected Homebrew launch path, resolved launcher and
hash. A matching launch path cannot substitute for interpreter identity.

A detail read stages exe/argv/exe/argv plus final BSD metadata. Equal PID/start
alone cannot make a mixed exec stage valid. Any mismatch retains metadata only.
Eight observations or two seconds is the existing uncertainty allowance; no
extra retries are added inside one detail call. A PID reuse discards staged
argv, and read/decode errors cannot carry those bytes into evidence. The local
SDK ABI still needs real later validation; no libproc/sysctl call ran here.

Each direct product/helper has a pre-spawn intent and a finite wrapper gate.
The wrapper writes its own start receipt; the controller checks its Popen PID,
current wrapper, actual interpreter argv0, exact remaining wrapper arguments,
and live matched parent before releasing it. Wrapper and product are separate
stages of one start. Gate publication failure can occur after a successful
write; recovery therefore still accounts for both stages. It does not relaunch.

After explicit controller ESRCH, PPID 1 is permitted only for the exact
same-start product authorized by the already verified wrapper gate. The saved
gate must bind the exact group, UID, NI15 and parent identity. An unknown orphan,
a reused PID or a mere group/parent-argument match cannot be adopted. Newly
observed supervised daemons still require a live exact app parent and a start
no earlier than that app. The ordinary routine must show two daemon starts.
Previously verified children can be reparented only after explicit parent
ESRCH. Only ESRCH establishes absence; a PID-list omission does not.

Every CONT/TERM/KILL uses a fresh targeted read and exact identity/parent checks.
Signals are single-PID only. There is no Popen kill fallback for products.
The read-to-signal interval remains separate macOS calls, not an atomic kernel
identity lease. A small SIGALRM mask covers only delivery plus its memory
receipt, so a later evidence timeout cannot erase an actual TERM time.

## Evidence and recovery are separate decisions

The collector first polls all retained product/CLI and watchdog handles and
stores exits in memory. It then attempts independent first receipts. A failed
write cannot skip a later poll. The watchdog uses an independent Inventory and
probe process. A failed normal observer stops the flow and declares abort in
memory before writing evidence. The inherited watchdog stdin pipe carries one
at-most-512-byte cleanup request, with nonblocking write/read and the original
absolute deadline. It supplies no ownership authority. Thus abort publication
failure cannot defer recovery until the flow watchdog expires.

Each evidence action has a separate bounded error boundary. Ownership data is
kept in memory before publication. Recovery can use fresh complete validation
for absence despite sink failure; sink failure always prohibits acceptance.
If full observation/budget fails, a fresh targeted route still registers known
gated exec stages and verifies owned signals. Targeted rows alone cannot grant
peer teardown or final inventory acceptance. Once full validation already
permitted peer teardown, later observation failure cannot disable verified
peer recovery. Unknown private candidates remain uncertainty.

The main controller and watchdog keep the same absolute 120 s flow and 60 s
cleanup budgets, clamped to start + 180 s. An ordinary completion gives the app
tree ten seconds for normal shutdown. Abort handles the app first, then its
verified children. The peer stays alive until fresh full validation proves
the app tree absent. Public peer stop is permitted once. It gets ten seconds,
then verified termination if needed. CLI operation deadlines stay five seconds.
No CLI acknowledgement substitutes for process exit.

One stop routine handles app, child, peer and CLI identities. It tracks actual
successful TERM times by PID/start and actual KILL times separately. KILL
requires at least five seconds after that identity's TERM and a later advance
round. A late delivery with insufficient remaining grace is explicit failure.
The deadline is never extended. KILL has a five-second absence allowance;
insufficient remaining allowance is also explicit failure. No phase timer
advances into immediate escalation. Unknown/reused/partial identities receive
no signal. A stale socket is retained and fails cleanup; it is never unlinked
to manufacture success.

A terminal collector receipt seals further launches and lists every attempted
intent, retained handle PID and collected exit. Recovery requires exact set
and PID agreement across all commands, spawns, retained handles and OS exit
receipts. Missing post-spawn records or controller loss cannot certify clean
recovery. The controller also checks its in-memory retained handles before
returning a terminal result. It keeps polling until closure or the fixed
cleanup deadline. Supervised daemon absence is recorded, but no waitpid status
is invented for app-owned children. App OS exit 0 and all direct OS exits are
required for ordinary acceptance. Loss of the controller still fails acceptance
even if the watchdog later stops all verified products.

The watchdog is never cancelled on an exception. Unexpected watchdog exit
starts separate parent fallback recovery inside the existing cleanup deadline.
A terminal write gets one evidence-only second; control-child reap/identity
receipt has three seconds. These allowances cannot launch/signal products or
extend cleanup. Failed bounded recovery leaves the guard and private evidence.

## Strict payload byte accounting

The aggregate evidence bound is 33,554,432 bytes (32 MiB). Every evidence file
has an 8,388,608-byte (8 MiB) ceiling. This counts the new source package and
all new evidence payloads; the unchanged copied signed bundle is an artifact,
not a log/evidence allocation. All product logs and the raw report are included
even while they live in the private runtime root.

The ledger starts exclusively before preparation and permanently reserves:

- 16 MiB: eight product-file slots of 2 MiB each. These are app daemon stderr,
  app openergy.log/.1/.2, peer openergy.log/.1/.2, and the raw app report.
  Inherited RLIMIT_FSIZE=2 MiB enforces each slot. This is an explicit subordinate
  product allocation under the unchanged 8 MiB per-file ceiling. It can fail a
  future attempt; the source does not pretend every product file retains an
  independent 8 MiB allowance under a 32 MiB aggregate limit.
- 4 MiB: cleanup share, including 1 MiB usable only by terminal receipts.
  Main/stream writes cannot use cleanup space. Recovery observations/signals
  cannot use the terminal-only share. Abort, direct exits, collector terminal,
  bounded terminal recovery, and outcome use terminal priority.
- 64 KiB: accounting infrastructure. The one byte-ledger temporary, ledger,
  lock, and bounded observation counters/temporaries use this fixed allowance.
  Ledger data is capped at 4096 bytes; the constant-size observation counters
  use at most two live writers. No general payload bypasses accounting.

Initial package bytes and all regular writes consume the remaining share.
There is one nonblocking cross-process byte lock. The ledger reserves growth,
fsyncs the temporary and directory after atomic replacement, then permits the
payload write. Failed reservations write no payload. Failed/partial payloads
keep their charge. Corrupt/missing accounting state fails closed, never resets
to zero. First-record checks happen under the same lock before any charge or
temporary, so an existing first receipt uses no new capacity. Exclusive records
conservatively charge twice their length for temporary and final names.

All direct product/CLI and watchdog stdout/stderr go to pipes. The collector
reads at most four 65536-byte chunks per handle per collection. Each retained
chunk reserves bytes before append, under the shared ledger. After the first
sink/cap failure for that stream, it drains/discards further bytes and marks
failure; logs cannot exhaust the cleanup reserve. A pipe failure does not skip
polling or the watchdog's ownership recovery. The hard product-file slots
account for autonomous daemon writes and their rotations without sampled sums.
RLIMIT_FSIZE alone is not the aggregate proof: prior full-slot reservation plus
coordinated reservations for every other writer provide the bound.

After verified absence, reserved raw files move once into the archive by
same-filesystem rename under the byte lock. Preparation checks the filesystem
identity before startup. There is no temporary second payload or cross-device
copy fallback. Reservations remain charged after the move. Parsed report,
config and terminal records use ordinary byte accounting. Unreserved raw
archival writes were removed.

The observation round/row lock is separate. Main sampling cannot cross 4976
rounds or 19904 rows, preserving 1024 rounds and 4096 rows inside the existing
6000/24000 total. Detail/probe/control-action receipts are not invented samples.
A counter or sample-cap error fails the flow and still leaves targeted owned
recovery available. All waits and I/O retain finite absolute deadlines. Kernel
scheduling/uninterruptible calls are not claimed as hard real-time guarantees.

## Product and scope bindings

Production source, the signed ordinary bundle, default25, 16 MiB IPC frames,
1,000,000-byte stalled write, 300 ms request, 1 s responsiveness, 2 s stalled
bound, existing 3/5/10/15 s routine waits, and 1.5 s supervisor stop are unchanged.
The signed app and daemon hashes and full source/framework inventories are
bound by immutable original records; this task does not rerun build/signature
validation. A later invocation uses fresh private paths and only literal
127.0.0.1:65431/65432, virtual platform, discovery off and explicit port-24800
denial before startup. Neither port was queried here. The original archive
stays FAILED / NOT ACCEPTED. Running-update/relaunch and the three stopped
provider tasks remain excluded.
