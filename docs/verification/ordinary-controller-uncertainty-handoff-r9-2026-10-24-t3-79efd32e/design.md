# R9 bounded uncertainty transfer and ownership contracts

This is inert source design for fresh independent review, not runtime proof.
R7/R8 and the full R6/R4 reviews plus all seven R5 responses were read as data.
Only the three-file R7 source delta in this package is proposed.

## State, serialization and handoff

Every Inventory starts with `handoff_ready=False`. An empty local map cannot
establish absence. The main controller initializes `uncertainty.lock` and the
sequence-zero checkpoint in `uncertainty.jsonl` once, inside its exclusive new
runtime guard, before starting the watchdog or any product. Initialization
failure stops before those launches. No recovery Inventory initializes or
recreates the journal/lock. Existing receipt reload remains an ownership input;
it is not an uncertainty handoff and cannot make `handoff_ready` true.

Every existing full `observe` call takes the same nonblocking file lock under
its original absolute I/O deadline. It reloads the existing ownership receipts, reads the bounded journal, validates
sequence and metadata shape, and restores the latest complete checkpoint.
Thus independent Inventories serialize changes instead of merging stale copies
or resetting another observer's attempts. No second probe is added.

The checkpoint contains only a sequence, unresolved PID/BSD microsecond-start
metadata, pending PID/original monotonic start/attempt count, a metadata-only
index of previously registered starts, sticky overflow and a sticky lost-handoff
bit. No role, argv, executable, UID, group, parent
chain or ownership gate is transferred in this state. Pending known-owned starts
also retain metadata query tokens before a receipt can be lost. The controller
owner's query-only read races keep their existing pending allowance without
inventing private-child ownership.

The registered-start index is a negative check on ownership handoff, not an
ownership record. A successor must already have each indexed PID/start in its
independently loaded/cached ownership records. Otherwise it sets lost state and
blocks absence. This covers an identity receipt that disappears or cannot be
read after complete ownership validation cleared an earlier unresolved token.
It grants no adoption, argv read, signal, or replacement uncertainty allowance.
Existing valid in-memory records still survive failed publication. The journal
cannot replace an unavailable identity receipt with ownership from metadata.

After sequence zero, restore also requires the existing bounded observation
counter file. A missing/corrupt file cannot enter `budget()`'s initial-zero branch
in a fresh observer. The original counter and byte accounting functions remain
unchanged. This is an absence/observation barrier on loss, not a new budget.

Under the lock, a `begin` record for the next sequence is appended through the
existing byte writer and fsynced before the existing probe can start. Complete
processing sets `uncertainty_complete`, even when the existing finite detail,
ownership, priority or observation-budget validation must raise. The `finally`
boundary appends/fsyncs the complete state before releasing the lock. Its original
pending times/counts and overflow survive expected validation errors. Absence
becomes available only after a complete checkpoint write and no lost-handoff
bit. The caller still handles that validation error as failed acceptance.

A deadline, probe failure or exception before complete processing either records
`handoff_lost=True`, or leaves an unmatched begin/partial checkpoint. A dead
writer releases its OS lock; the durable begin remains. A successor reads the
last complete metadata if available but rejects the interrupted transaction.
It must not probe a fresh allowance or grant absence. A partial JSON line,
missing newline, malformed shape/sequence, missing journal/lock, cap error,
read error or failed lock acquisition fails closed in the affected observer.
No restore defaults missing data to empty. A failed commit leaves the begin,
not a new empty state. If the complete state was in fact written before an
I/O exception, a successor can use those complete bytes; it cannot lose the
saved detection or allowance. The protocol assumes ordinary file semantics;
it does not claim protection against an adversarial rollback of durable files.

Local lost state cannot be cleared by another read. Lost-state checkpoints and
overflow remain sticky for successor Inventories. An unavailable state can
therefore cause bounded failed recovery with the peer retained. This is the
required result when prior absence blockers cannot be recovered. It is never
an app-tree absence certificate. Missing evidence still fails acceptance.

## All observer lifetimes

- **Main controller to watchdog:** normal completion, explicit abort, request
  pipe failure, receipt failure, EOF, parent loss and flow timeout all reach
  the same shared checkpoint protocol. The bounded 512-byte request remains
  unchanged; it carries no potentially truncated uncertainty map. Recovery's
  first full observation must restore the journal before any absence decision.
- **Watchdog to parent fallback:** retained actual watchdog death starts one
  fallback through the unchanged R6 independent action boundary and original
  cleanup deadline. The fallback's Inventory restores the last watchdog/main
  checkpoint. Death inside a full observation leaves an incomplete begin;
  death after a complete checkpoint leaves the original uncertainty allowance.
  Terminal evidence lookup/load/parsing cannot skip fallback construction or
  its steps. A journal failure occurs in `observe`, not in a new constructor
  dependency that could suppress the existing targeted recovery path.
- **Cleanup-launch observer:** its existing full wrapper observation restores
  the same journal and saves its own state through the same lock. Before the
  registered gate can release the one peer-stop CLI, the controller additionally
  requires `absent(data, app_role)` from that full observation. A permission
  receipt or fallback flag alone cannot bypass inherited state at this gate.
  Failure leaves the retained wrapper handle/intention for bounded collection;
  it cannot exec the peer-stop product or allow a second launch.
- **Repeated or replacement Inventory:** no constructor resets shared state.
  Every full call restores it. A stale Inventory overwrites its stale cache
  from the serialized checkpoint before using the probe. Previously completed
  ownership or explicit ESRCH resolution is not undone by a stale map merge.
- **Targeted recovery:** unchanged targeted reads may register a narrowly known
  gated exec and signal freshly verified owned identities even when the journal
  is unavailable. They do not consume unresolved tokens as ownership, resolve
  the uncertainty journal, grant peer permission or certify final absence.
  Once a valid full observation has already permitted peer teardown, the
  existing targeted peer-recovery policy remains unchanged.

No request path, constructor or terminal receipt can itself establish absence.
`Inventory.absent` requires both handoff readiness and all prior predicates.
The watchdog retains its phase predicates and uses that gate for app, tree,
peer and final phases. Its one source change guards peer-permission publication.
`_final` records handoff readiness/loss alongside unresolved metadata/overflow.

## Permission revision and concurrent observers

A full observation records its completed checkpoint sequence in memory. Before
watchdog `peer-request` can set `peer_permitted` or publish permission, an
`absence_guard` takes the same lock and restores/checks that exact sequence.
The lock remains held through the flag and receipt publication. Another full
observer cannot detect/checkpoint new uncertainty between this check and that
permission. An active predecessor must finish or leave its dirty begin before
the guard can read state. A newer complete revision rejects the stale decision;
no grant follows. The next existing recovery advance obtains a new full snapshot
under the same observation/time limits. The guard itself performs no probe,
charges no detail attempt and resets no allowance.

The cleanup-launch wrapper likewise holds this guard through its new app-tree
absence test and registered-gate publication. This closes the separate gap
between its full observation and the gate that permits public peer-stop exec.
A stale revision fails the single launch, retaining its handle/intent; it does
not start an extra observer or permit a second launch. Final inventory calculation
also uses the same revision guard before it can report absence. Guard errors
leave local absence unavailable. All guard I/O stays under existing absolute
and enclosing observation/recovery deadlines. No kernel read/signal atomicity,
universal discovery or future-process guarantee is asserted.

## Original uncertainty allowance and finite storage

A detected candidate survives PPID 1, loss of path classification, omitted
PID enumeration, detail races and PID reuse. Its PID/start token stays in the
separate `--private-candidates` partition. R7's unchanged probe/macproc partition
requires a current private path before reading unowned details. Both applicable
start sets must match before argv reads. Reused starts expose metadata only.
Coherent unowned details outside the private root are discarded. Only unchanged
complete ownership validation, or explicit ESRCH with no current PID row, can
clear a detected start. An orphan with no exact chain stays unresolved.

All pending increments use the restored original start time and count. No
constructor, actor change, request or fallback restarts either. Each unresolved
full observation charges at most one pending increment, as in R7. Eight
observations or two seconds still raises the same failed validation. Existing
global observation limits also apply; this is no extra retry allowance. A
failed/incomplete handoff cannot replace an expired allowance with a fresh one.

The union of retained pending and unresolved PIDs is capped at 24000, the existing
row ceiling. The negative registered-start index is also bounded to 24000 starts;
restore rejects a checkpoint whose combined PID union exceeds 24000. No entry is evicted to pass. Capacity failure sets the sticky
overflow bit. The journal allows at most 6000 full observation transactions and
12001 initial/begin/checkpoint lines, in addition to the unchanged shared
6000-round/24000-row budget and main 4976/19904 limits. These ceilings add no
observations. Failed/incomplete transactions can only reduce available coverage.
The journal remains subject to the unchanged 8 MiB per-file and 32 MiB aggregate
byte ceilings. Full checkpoints can exhaust these earlier; this is explicit
failure, not a promise that all maxima fit simultaneously.

All journal payload uses `common.append` and its existing locked, pre-write
charges. Main journal bytes use the regular share. Recovery/cleanup-launch bytes
use the nonterminal cleanup share; they cannot consume the terminal-only MiB.
The empty lock has zero payload and is created through the same writer. There
is no unaccounted state file, bypass, new reserve or copy archive. Reading the
journal is bounded by the file cap, line cap and original I/O deadline. Waiting
for the uncertainty lock also uses that deadline. It does not extend flow,
cleanup or terminal allowances. The byte lock and observation counter lock stay
unchanged; lock order is uncertainty then the existing byte or budget lock,
with no reverse acquisition by existing paths.

## Preserved identity, recovery and evidence contracts

Complete identity remains PID, BSD start, real/effective UID, actual PGID,
parent PID/start, executable and exact argv. Group membership grants no adoption
or signal authority. Actual Python.app executable/hash and exact wrapper argv0
remain distinct from the selected Homebrew launcher/resolved path/hash. Products
keep exact private paths, virtual flags, parent and socket arguments, and actual
absolute NI15 before/after details. Unknown orphans remain unowned.

Staged detail remains exe/argv/exe/argv, coherence/canonical argv0, then final
BSD identity. Partial or reused rows cannot publish staged argv. No signal
uses the uncertainty journal. CONT/TERM/KILL still require a fresh targeted
read, exact owned identity and parent checks, and a single PID. There is no
process-group or unverified Popen kill fallback. Read-to-signal is not an atomic
kernel identity lease. A known gated same-start wrapper-to-product transition
after controller loss still requires the exact prior gate and explicit controller
ESRCH, with no relaxed orphan adoption.

Abort and all-handle polling precede independent evidence attempts. Handles and
actual exits remain in memory; failed streams drain/discard. The cleanup request
pipe remains nonblocking and at most 512 bytes. Terminal evidence errors remain
separate from actual watchdog-loss detection, fallback assignment and steps.
Durable pre-spawn intents, retained handles, sealed labels/PIDs and exact real
OS exit receipts still must agree before recovered:true. Controller death does
not invent direct exits. App OS exit zero remains an acceptance requirement.

App/children precede peer. Normal app grace stays ten seconds. Each successful
TERM's stored delivery time starts its own full five-second grace; KILL also
requires a later advance and the unchanged five-second absence allowance.
Insufficient remaining time is failure inside the same deadline. CLI deadlines
stay five seconds. Stale sockets are retained and fail final cleanup. The
watchdog is never cancelled on exception. Full known PID/start/group/socket
absence and the closed direct-exit ledger remain required for clean recovery.

Byte accounting stays 32 MiB total and 8 MiB per evidence file. Eight autonomous
2 MiB product slots reserve 16 MiB before startup: app daemon stderr, app
openergy.log/.1/.2, peer openergy.log/.1/.2, and raw report. The inherited
RLIMIT_FSIZE remains 2 MiB. Keep 4 MiB cleanup including 1 MiB terminal, and
64 KiB accounting inside the total. Failed writes keep charges; existing first
receipts short-circuit before further charge. All pipes use finite chunks and
the coordinated writer. After verified recovery, archival renames reserved files
on one filesystem; no second copy or cross-device fallback.

## Fixed product scope and missing proof

One ordinary no-render `--e2e --require-peer` default25 real AppModel/daemon/peer
IPC flow, one launch per role, 120 s flow, 60 s cleanup and fixed start + 180 s
end remain unchanged. Keep 16 MiB frames, 1,000,000-byte stalled write, 300 ms
request, 1 s responsiveness, 2 s stalled bound, 3/5/10/15 s waits and 1.5 s
supervisor stop. Future private literal 127.0.0.1:65431/65432, virtual platform,
discovery off and port-24800 denial must apply before startup. No port was queried.
Rust 1.99.1/jobs2/deployment15 are inherited lineage only.

The exact retained signed app/bundle/daemon/peer inputs are missing according
to the task and R7/R8 records. Their historical signed hashes remain app
`53ba6f431811eb1178c39b8b7ad7cdb425b0088e7c893b8dcab8e601ad338c23`
and daemon/peer `1c558733a56ada70661f8a8493eae7deaf3813ce1bb572d0592fe86900b6b971`.
No path existence, artifact, ABI, signature or runtime check was repeated.
No rebuild, substitute, fixture/preflight, signing, test or runtime was attempted.
All old evidence remains exact and FAILED / NOT ACCEPTED, 23/25. Fresh independent
source review comes first; separate qualified actual runtime evidence remains
required without a new human permission gate.

All stopped gates and the closed expired Sparkle one-shot remain stopped.
No approval reuse, retry, renewal or reset. No live sharing, installed apps,
production config/TLS/default IPC, TCC, login/services/tasks/accounts/firewall,
global DNS/toolchain-default changes, TCP24800/UDP5353, Docker, credential/key
access, desktop driving, remote commands, push or publication belongs here.
