# Responses to the two R6 P2 findings

Both findings are accepted and repaired in source. This response is based only
on source reading and static file/Git/hash/JSON/AST/patch checks. No supplied
program or task AST ran. No runtime reproduction or acceptance is claimed.
The exact source delta is repair-r5-to-r7.patch, from R5
`ec723d8fde7c65e955c015653f89d24f7cc7f091`.

## R6-P2-1: retain detected private uncertainty after app exit

`macproc.snapshot` (macproc.py:108-165) keeps `private_candidate=True` on the
metadata row as soon as the preliminary path read identifies the private root.
A staged detail error does not erase this boolean and cannot attach staged
exe/argv. Both enumeration and explicit missing-PID queries reject a reused
start before reading any argv. Unowned coherent details that no longer name
a private executable are discarded before publication.

`Inventory._observe` (ownership.py:106-125,140-202) selects the marker even for
an unknown PPID-1 child in a separate unregistered group. Before accounting or
publication it retains only PID/start metadata for each unregistered detection.
It also retains partial rows selected through an owned parent, so losing that
parent cannot erase their pending identity. A separate optional
`--private-candidates` argument partition in the existing probe call requests
metadata/start and explicit ESRCH coverage. It never moves these tokens into
the owned/direct set that authorizes detail reads without a fresh private path.
`probe.py` only forwards that partition; its targeted route is unchanged.

The unresolved map is independent of `records`, `direct`, groups and gates.
A current private path still has to independently permit a coherent detail
read. The unchanged `identify` policy still has to establish exact ownership.
Unknown orphans cannot be adopted. Signals still require an existing owned
record and the unchanged fresh targeted identity/parent checks. The marker
adds no argv or signal authority. Both applicable start sets must match, even
if a later direct receipt happens to use the same PID as an unresolved start.

Reparenting, a lost path read, a missing enumeration row, a read race or a reused
PID cannot clear the unresolved entry or reset its pending start time. The
entry clears only after complete existing ownership validation, or after a
full observation explicitly reports ESRCH with no current PID row. A complete
but unowned orphan stays unresolved and fails validation. An expired allowance
also retains the entry. The eight-observation/two-second allowance stays in
place, with at most one unresolved increment per full observation. At 24000
metadata entries, one sticky overflow flag fails closed; no entry is evicted
to manufacture absence. This cap creates no extra observation allowance.

`Inventory.absent` rejects pending/unresolved/overflow state for app-role and
app-tree queries as well as final absence. `_final` includes metadata-only
unresolved entries in its result and retains candidate rows as leftovers.
The byte-unchanged watchdog phase logic therefore cannot advance to peer
permission through a discarded detection. Targeted recovery still cannot
permit peer teardown or signal a detection-only child. This concerns already
detected candidates, not discovery of every possible child or an atomic kernel
identity guarantee.

## R6-P2-2: start fallback independently of terminal receipt I/O

`Controller.cleanup` (controller.py:326-353) ends the terminal receipt
lookup/read/parsing try/except before a separate watchdog-loss action block.
Each terminal error sets failed acceptance. The following action block uses
`control_exit`, retained by the byte-unchanged all-handle collector, or the
retained watchdog handle's actual non-None return code. A terminal I/O or
per-iteration deadline exception cannot skip this block. Evidence absence
alone cannot satisfy the actual watchdog-death gate.

When there is no completed valid terminal return, actual exit plus
`fallback is None` constructs one fallback with `normal=False` and the original
cleanup request/deadline. Its constructor uses the remaining original cleanup
deadline, not the shorter expired terminal-read limit. The fallback object is
assigned before the loss receipt is attempted. Its steps stay outside terminal
receipt I/O and use the unchanged fresh ownership/signal checks. The deadline
is never extended, and no unverified child is adopted or signalled. Repeated
terminal receipt errors still fail acceptance. They cannot suppress an
otherwise possible bounded recovery attempt after actual watchdog loss.

## Preserve all seven R4 repairs

1. Actual interpreter and selected launcher remain distinct. Wrapper argv0,
   exact remaining argv, launch binding and both executable hashes are intact.
   `wrapper_argv`, `exact_argv`, Controller `_launch` and `run` are unchanged.
2. `macproc.detail` is unchanged: staged exe/argv/exe/argv, canonical argv0,
   final BSD identity check and metadata-only errors preserve reuse privacy.
   The new detection path adds no detail retry or relaxed identity match.
3. All-handle collection, in-memory abort, bounded pipe wakeup, stream drain/
   discard, evidence writer and signal action boundaries are unchanged.
   The new independent fallback-creation boundary closes the R6 remainder.
4. Pre-spawn durable intents, retained handles, sealed terminal sets, exact
   spawn/exit agreement and the collector's non-vacuous terminal gate remain.
   Only detected uncertainty strengthens the absence gate.
5. `Inventory.identify` and all of watchdog.py are unchanged. Same-start gated
   exec after explicit controller ESRCH remains narrow; unknown orphans get
   no ownership. Metadata tracking is not added to targeted recovery authority.
6. `Inventory.signal`, `_signal` and watchdog stop logic are unchanged. TERM
   delivery time remains per PID/start, with five full seconds and a later
   advance before KILL, inside the original deadline.
7. common.py, exec_owned.py, prepare.py and watchdog.py are byte-unchanged.
   Keep eight autonomous 2 MiB product slots, 4 MiB cleanup including 1 MiB
   terminal, and 64 KiB accounting within 32 MiB, plus the 8 MiB evidence-file
   ceiling. Cross-process pre-write reservations, draining and rename-only
   archival remain unchanged.

The package retains one ordinary no-render `--e2e --require-peer` default25
AppModel/daemon/peer flow, one launch per role, app/children before peer,
120 s flow, 60 s cleanup, fixed start + 180 s, 6000/24000 observations and
1024/4096 cleanup reserve. Frame/request/stalled-write/routine waits are exact.
Actual future NI15, complete identities, app OS exit, all direct exits, late
priorities and cleanup still need real proof after fresh independent review.
The original run remains FAILED / NOT ACCEPTED, 23/25; its 87 files stay exact.
R3 and R5 never ran. No product defect is established. The retained original
private signed bundle is now absent, as recorded by file-only checks; no
rebuild, old-fixture repair, new preparation or runtime attempt was made.
