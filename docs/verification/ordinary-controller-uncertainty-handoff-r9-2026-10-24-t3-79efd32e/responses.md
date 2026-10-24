# Response to R8-P2-1 and preservation of earlier repairs

Date: 2026-10-24, America/Vancouver.

**R8-P2-1 is accepted. This package proposes its source repair and awaits fresh
independent review.** No runtime reproduction, acceptance, or general clearance
is claimed. The exact input is R7 d56c65eb8da5f3d8b4fd58fc0e933360754ba6d6;
R8 is 172ee851d99bf1c256235b23fb445cae899332ff. The full prior R6 review at
4eaa964eab6f74b2fc5704a1925520866c7d9f9c, R4 review at
cc116c594f08fd0c182bc2b18cfebbffe9ed9d6d and all seven R5 responses at
ec723d8fde7c65e955c015653f89d24f7cc7f091 were read. Only this new folder changes.

## R8-P2-1: all Inventory lifetimes retain detected uncertainty

R7's unresolved map, pending allowance and overflow were local to one Inventory.
`reload` read only owned identity receipts. Fresh watchdog/fallback observers
could therefore drop a detected PPID-1 daemon when its next preliminary path
read failed. Failed acceptance did not prevent early peer teardown.

`Inventory.observe` now wraps each existing full observation in one serialized
uncertainty transaction. All callers use the same bounded journal, including
main, watchdog Recovery, parent fallback Recovery and cleanup-launch. The main
controller initializes it once before watchdog/product launch. A constructor
starts with absence unavailable and never initializes a recovery replacement.
The existing bounded request/pipe remains unchanged. The watchdog adds only a
revision guard around its existing peer-permission branch.

The source path for R8's detected-candidate trigger is now:

1. A complete main observation checkpoints the detected PID/start and original
   pending time/count even when finite validation fails and aborts the flow.
   Detection remains separate from owned identities. Overflow is checkpointed.
2. Before the watchdog's first full probe, it restores this state under the
   shared lock. The separate unresolved partition asks for metadata/start and
   explicit ESRCH, with no extra argv or ownership authority.
3. A PPID-1 metadata row with a failed path read is selected by the inherited
   token. It retains the same pending clock/count. A missing enumeration row
   requires explicit ESRCH; reuse cannot erase the token or expose new argv.
4. `absent` rejects unresolved/pending/overflow and unavailable handoff state.
   Existing watchdog app/tree/peer/final decisions cannot grant peer permission
   through an empty local map. Full complete ownership validation or explicit
   ESRCH can still resolve uncertainty under the original policy.

These are static branch readings, not an executed interleaving or simulation.

## Handoffs and evidence loss

A durable accounted `begin` precedes the probe. Complete processing, including
expected validation errors, checkpoints the exact allowance in `finally`.
Interruption before complete processing records lost state or leaves a dirty
begin. Death cannot leave a clean empty successor. Missing lock/journal,
unreadable bytes, malformed/truncated records, sequence/cap failure, failed
begin/commit or a dirty transaction blocks absence. No state is reconstructed
as zero. A lost handoff may end in bounded failed recovery with the peer retained.

If a previously resolved ownership receipt disappears or cannot be loaded,
the journal's metadata-only registered-start index also blocks absence. The
index requires an independently available existing ownership record; it cannot
adopt a process. No new detail budget starts from that missing receipt. Existing
owned records in memory still support fresh targeted verification. Missing or
corrupt observation counters after sequence zero also block a fresh observer
before `budget()` could create initial counters again.

Actual watchdog death still constructs one fallback independently of terminal
receipt I/O. It restores the same journal using the original cleanup deadline.
A death during its predecessor's observation is a dirty/lost handoff. A death
after a full checkpoint transfers the same unresolved metadata, pending clock,
attempt count and sticky overflow. New journal failure occurs in observation;
it adds no new constructor I/O dependency to fallback creation.

The existing cleanup-launch observer also restores/checkpoints the same state.
Its new app-role absence guard runs after its existing full observation and
before the registered wrapper gate. A saved peer-permission receipt or fallback
flag cannot exec the one peer-stop CLI through unavailable inherited state.
Permission and final absence also bind to the full observation's journal
revision. The same lock stays held through watchdog flag/permission publication
and the wrapper registered-gate publication. A newer or active observer cannot
be skipped between a snapshot and its permission. A changed revision rejects
that stale decision; only the next existing full recovery advance can refresh
it, within the original limits. The guard adds no probe or detail attempt.

Failure keeps the wrapper handle/intent for bounded polling and fails acceptance;
it grants no signal and no second launch. Targeted recovery remains unable to
authorize peer teardown or final absence. It can still stop freshly verified
owned app/child identities, and retains the existing later peer-recovery rule
after earlier valid full permission.

## Same finite limits

Pending time and count are serialized, not restarted or merged from stale
Inventories. Eight observations/two seconds still fails validation. The union
of pending/unresolved PIDs has the existing 24000-entry ceiling, with sticky
overflow and no eviction to manufacture absence. The negative start index is
also bounded; invalid/oversized state fails closed. At most 6000 transactions
and 12001 journal lines fit under the unchanged 8 MiB file ceiling and 32 MiB
aggregate reservation writer. This can fail before maximum coverage; it cannot
add observations, retries, time or byte capacity.

Main journal writes use regular space; recovery/cleanup-launch uses nonterminal
cleanup space. No journal payload can consume the terminal-only MiB. The empty
lock also uses the common writer. There is no new reserve, bypass or archive
copy. Lock/read/write waits use the original absolute observation/cleanup I/O
deadlines. Targeted verified recovery and all-handle polling remain independent
of failed journal/evidence publication. Missing state cannot establish absence.

One ordinary no-render `--e2e --require-peer` default25 flow and one launch per
role remain fixed. App/children precede peer, inside 120 s flow/60 s cleanup/
start + 180 s. Keep 6000 rounds/24000 rows, main 4976/19904 and cleanup reserve
1024/4096, eight 2 MiB product slots, 4 MiB cleanup including 1 MiB terminal,
64 KiB accounting, 8 MiB file/32 MiB total, and five full seconds after each
successful TERM. Product predicates, frame/request/stalled-write/waits and
private configuration are unchanged. No extra observation was added.

## R6 repairs remain intact

- **R6-P2-1:** macproc/probe are byte-unchanged from R7. Detection markers,
  separate unresolved tokens, start/reuse checks, staged private details and
  explicit ESRCH resolution remain. Shared uncertainty transfer closes R8's
  lifetime gap. The new negative start index grants no ownership.
- **R6-P2-2:** `Controller.cleanup` is AST-exact to R7. All-handle collection,
  terminal-read and watchdog-loss try boundaries, retained actual exit test,
  one fallback assignment before loss publication and steps outside terminal
  I/O are unchanged. All watchdog functions except `Recovery.advance` are AST-exact; that function
  differs only by wrapping the existing peer-request branch in the revision guard. Journal errors
  cannot move fallback construction under terminal receipt reads.

## All seven R4 repairs remain intact

1. **Actual interpreter and selected launcher.** `wrapper_argv`, `exact_argv`
   and identity checks stay AST-exact. Controller `_launch` and `run` retain
   their complete R7 AST after removing only the new absence gate and journal
   initialization respectively. All launcher/actual interpreter/hash/argv checks
   remain exact; product argv is not weakened.
2. **Staged reads and reuse privacy.** Complete macproc/probe sources remain
   byte-exact to R7. No unresolved/index token enters the owned/direct detail
   partition. Reused identities expose metadata only. No additional detail
   attempt is introduced inside the existing probe.
3. **Evidence versus recovery.** `collect`, `drain`, `abort`, `request_cleanup`,
   `observe` (controller) and `tick` remain AST-exact; common.py is byte-exact.
   Abort/real-exit memory state precedes independent writes, pipe wakeup is
   unchanged, failed streams drain/discard, and first records stay immutable.
   Required uncertainty state may block absence, but grants no action authority
   and does not disable the existing fresh targeted verified-owned actions.
4. **Post-spawn ledger.** Pre-spawn intent, retained handles, one-launch guard,
   sealed collector and exact intent/spawn/handle/real-exit sets remain. The
   complete watchdog terminal function is AST-exact. A failed cleanup wrapper
   gate cannot yield a vacuous recovered:true or a second peer-stop launch.
5. **Narrow same-start gated exec.** `Inventory.identify` and targeted watchdog
   recovery are unchanged. Exact role/start/group/UID/NI15/interpreter/argv,
   saved wrapper gate and live matched controller or explicit controller ESRCH
   remain required. Unknown orphans and metadata-only indices gain no adoption.
6. **Actual TERM time and grace.** `Inventory.signal`, `_signal`, watchdog
   stop logic, alarm mask, delivery memory receipts, later-advance guard and
   full five-second TERM grace are unchanged. No deadline is extended.
7. **Aggregate bytes and archival.** common.py, exec_owned.py and prepare.py are byte-exact. Watchdog
   signal/targeted/terminal/finish/main functions remain AST-exact. Shared pre-write reservations, failed charges,
   eight 2 MiB product slots, protected cleanup/terminal/accounting shares,
   pipe draining and rename-only reserved archival remain. The new journal
   uses those same charges and can fail within them.

## Static proof and remaining limits

The independent data reader checked all eight ASTs, all bound old Git/file/
size/hash records, the complete 87-file original index, and exact patch hunk
materialization. It rechecked all 31 R7-to-R5 protected function claims. R9
retains 29 verbatim; the two controller changes normalize exactly as described
above. `source-checks.json` records final hashes and structural checks.
`manifest.json` binds the complete package except itself; Git binds the manifest.
These checks do not execute branches, validate timing/ABI or prove real cleanup.

Original 498190feb00f51039f500ae618ac7de5fb113d86 stays FAILED / NOT ACCEPTED,
23/25. All 87 bytesets, failures and guards are preserved. The original shared-
PGID assertion/early peer stop do not establish a product defect. App OS exit,
complete late NI15 coverage and later real 25/25/identity/stop/restart/cleanup
proof remain absent. Exact signed app/bundle/daemon/peer bytes are missing per
the task and retained R7/R8; current payload paths were not queried. No rebuild,
substitution, artifact restoration/preparation, preflight, build/signing, runtime,
unit test, mock, simulation, process/OS/port/job query, network, desktop action,
remote command, credential/key access, delegation, push or publication occurred.
No supplied module/helper/controller/checker or task AST ran or was imported.

Fresh independent source review is next. The ordinary flow has no new human
approval gate. All stopped tasks and the isolated Swift build gate remain
stopped; the expired closed Sparkle one-shot has no retry/renewal/reset or
approval reuse. This package creates no runtime acceptance or release clearance.
