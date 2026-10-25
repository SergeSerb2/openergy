# Independent M1 metadata R10 SOURCE/DATA review

2026-10-25, America/Vancouver. **SOURCE CLEAR for this exact metadata caller
and its bounded source/data interface. No open bounded P1/P2 found.
R8-P2-1 is CLOSED_SOURCE_ONLY. execution=false; runtime_acceptance=false;
future_execution_approval_created=false.** R6-P2-1 and the original R1/R2 and
caller-owned R4 publication closures remain CLOSED_SOURCE_ONLY. This report
supplies no runtime, artifact, parent-flow or approval grant and adds no gate.

## Commission and exact identity

The candidate is `9b50dd7e3a672eafe319be33141cb875d54a471e`, imported as
`4c6658e51e41fdc5794f91309ce9e716d5722ab8`. Candidate parent is
`81252214f7c391c465510193acf48115ec3d95db`; import parent is
`263c75859ab643ef0e5b435b3f81bd9ffe6739c4`. The checkout started clean at that
import. All 23 candidate-folder files equal original Git, imported Git and
working bytes, including mode100644, blob, length and SHA-256. The folder tree
is `e8bd51b7d8de49ff0659c23c253eb1f8a0f627b7` in both commits. Their whole
commit trees differ. Their full-index binary folder deltas agree exactly:
261,672 bytes, SHA-256
`29047b49bd23c93eeb880344f4353282b6acff71a66f6e28406d0d8430e4ef9a`.

All caller locations below refer to the full
[R9 caller](../m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/metadata-caller.py.txt).
It is 46,908 bytes / 789 lines, SHA-256
`01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`.
The exact [patch](../m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/caller-13fccbe-to-r9.patch)
is 1,504 bytes, three zero-context hunks, SHA-256
`5d4c7357d0bdd6d5702788605a65a4fc7cb17108a365ad2d9398feeccc8ec736`.
Its baseline is the R7 caller at
`13fccbe1d547198d5025748d20af5fb5c3a8c287`, import
`0dad1c00d0a27f8266ed8e138f780a9d94656cdc`, SHA-256
`c9a3b1bcfae0da2eced035c2c9e4048a92dc18340c940148a486cf12348b0987`.

[BRIEF.md](BRIEF.md) retains the complete original R8 brief and this R10
commission. I read the full R8 review, all 789 final caller lines, exact patch,
complete changed-function excerpt, R9 README and repair/compact contracts,
schemas, null templates, startup/CLI, plan, pins, vectors, byte proof and
binding/check data. The original/R3/R5 interface READMEs, three earlier full
caller reviews, preparation README, next operation, metadata recipe, canonical
SDK data, file helper, full operator and receipt contracts, and common's writer
contract supplied context. Unchanged helper/controller logic was not newly
qualified. Dependency reports were read only in their stated scopes.

## R8-P2-1 response: closed in source

The full prior review is `8326f49945824c71e1b58d421f107a3cfafff3da`, imported
as `51533bd0b265e5941faa967da02351d4b1146437`, retained at
[R8 REVIEW.md](../m1-metadata-r8-review-2026-10-25-t3-28dba3a1/REVIEW.md).
Its only open objection was R7 collect472-477: the final poll could return
`None` after the deadline, followed by KILL before any new clock reading.
Later alarms or rejection could not undo that signal. R8 recommended retaining
the actual poll result, immediately sampling the shared clock, preserving a
real exit, and permitting KILL for `None` only before both original ends.

R9 **421,474-487,522-523** implements exactly that bounded response. Only
`collect` changes among 33 top-level/method functions. All other 32 functions
are byte/AST-exact. The full Writer class and all other module AST nodes,
constants/imports and `os._exit` entry are exact. The nested alarm handler is
included in the unchanged `until` function. Reversing the three explicit edits
restores every baseline caller byte. Whole-function ranges and hashes are in
[function-bindings.json](function-bindings.json).

The final settle block **473-489** has these source paths:

- **Actual returned exit:** 474 retains the poll's actual integer, including
  zero or a negative status. The next call is the shared `clock_ns()` at476.
  The small `finally` at477-480 sets `reaped=True` for a non-null returned exit,
  including when the clock read raises. The `None` branch cannot signal this
  exit. Final `actual_exit` still comes only from `proc.returncode` at514.
- **Timely `None`:** 481-485 requires the retained sample to be strictly less
  than both `deadline` and `phase_deadline`. Only then does it mark the attempt
  and call the same retained direct handle's single `proc.kill()` site. Actual
  delivery is marked only after the call returns. There is no other target.
- **Late or equal `None`:** 487 sets both `timed_out` and `deadline_loss`, and
  skips KILL. The unchanged wait at488 still evaluates `remaining(deadline)`.
  That fresh check cannot create a new interval after the original end.
- **Poll, clock, KILL or wait exception:** the existing handler at490-492
  retains `kill_or_reap_error` and deadline loss. A clock exception precedes
  KILL. A returned real exit keeps its reap fact. An uncompleted sample remains
  null. A KILL request, timeout or failed wait does not invent an actual exit.

The original 10s ceiling and its 0.5s internal kill/reap reserve stay at
**410-413**. The unchanged wait, EOF/drop/close tail, actual-returncode source
and final time checks remain. A timely `None` followed by successful real wait
can yield a later actual exit; it does not rewrite the earlier eligibility
result. A KILL attempt or deadline/error flag still fails `complete`531-541.
If a later exception escapes before record assembly, collector status can stay
null on the retained handle. Neither this repair nor its contract promises a
completed record after every clock failure or substitutes a fabricated status.

Both observations start null. `kill_eligibility_exit` is the returned poll
value, not a signal-derived exit. `kill_eligibility_observed_ns` is the completed
post-return sample, not the OS exit time. An unentered settle block, a poll
exception or a clock exception can leave the sample null. Both null fields do
not prove that no poll ran. The parent must retain their exact integer/null
types and check their coupling under the new
[repair contract](../m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/repair-contract.md).

`perform` **550-558** copies the entire collector status into active/collected
commands and first failed COMMAND before any write. The two new fields thus
follow the existing status publication and direct-child paths. Existing raw-ref
enrichment and snapshot timing remain distinct; this does not assert equality
of every key in snapshots taken before and after enrichment. Parent must compare
the observations across their bindings and preserve all existing differences.
The final record is AST-exact after removing only the two additive keywords.

No unresolved source objection remains to the R8 repair. This is an ordering
closure only. Scheduling can pass a deadline after the sample, and a subsequent
OS call can stall. The existing alarms and postchecks do not prove hard
preemption, universal containment, timing or signal/reap success. No timing
scenario, failure injection, mock or live child was used to reach this verdict.

## Preserved caller and parent contracts

R1 remains CLOSED_SOURCE_ONLY. **35-38,65-115,677-680,702-706** retain integer
darwin CLOCK_MONOTONIC6 ns and the same-host/same-boot identity. Each SOURCE or
COPY phase remains original parent start +120,000,000,000ns. Admission, full
authentication, pin/read/import work, collection, KILL/reap, comparison and
publication consume that phase. Parent must observe actual caller exit/reap
and complete stdout/stderr EOF before the SAME end. A result file is pre-exit
data. `common_deadline` still samples local first, shared later, deducts1ms and
rounds down to common's unchanged local domain. No offset, retagging, repeated
qualification, reset or new rate/error bound is introduced.

The seven-file completed SOURCE manifest, five remote-file checks and external
parent inspection boundary **307-354,713** are exact. Manifest remains
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26`;
facts remain `ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`.
The tracked `492f067` report records the parent's completed seven-original
inspection. This review verifies that public report, not the private causal
act. Later five-file checks and NEW remote inspected/start samples remain
required in order. Old relative clocks keep their original domains.

SOURCE **708-714,728-739** keeps SYSTEM `entry-template` before private
`-I -S -B --version`, fixed Python lipo and exactly50 ordered native vectors:
53 caller records. It never imports common. COPY **693,715-742** keeps all17
pin checks before common import, parent prefix authentication BEFORE private
entry, canonical prepared bundle checks, fresh SOURCE v2 consumption and file
checks before/after50:52 caller records. The parent's separate SYSTEM entry
authentication also fits COPY's original phase. Later caller inspection cannot
establish pre-entry trust in an already-running private interpreter.

Collection **408-528** retains the 1MiB combined while-read cap plus one
overflow-detection byte, actual exits/nulls, both EOFs, known drops and unknown
unread bytes, close errors and retained direct handles. One Popen site remains,
with fixed environment/cwd, DEVNULL stdin, shell=False and closed fds. There is
no drain, command repeat, PID/group lookup, other signal target or extra time.

SOURCE **165-226** keeps owned0600 regular initial data, owned0700 directories,
exclusive raw writes and no ledger. COPY uses the same single Writer and exact
common regular pre-write reservations with `critical=False` for every new
guard, parent envelope/stream, raw/status, comparator and result. Existing
ledger/lock, double temporary/final charging and exclusive final link stay.
No ledger init/repair, second writer, refund or borrowed reserve appears.
Caps remain 8MiB/file and32MiB total, with16MiB product,4MiB cleanup and64KiB
accounting inside that total; initial regular allowance is12,517,376 bytes.
The new status bytes consume existing capacity and grant no extra allowance.

R2 remains CLOSED_SOURCE_ONLY. **550-577,600-608,748-770** retain first failed
COMMAND before publication, attempted/partial/durable refs, comparator only
after durable raw/status, assigned comparator failure before its publication,
separate publication errors/stop detail, and actual/null direct-child facts.
Only `-verify` sorts complete strict-UTF-8 splitlines, keeping duplicates and
empty lines. Every other comparison stays exact after declared bundle
substitution. COPY needs saved basis and fresh SOURCE. Unknowns cannot pass.

The caller-owned R4 buffered-publication closure remains CLOSED_SOURCE_ONLY.
SOURCE **194-226** still has one raw exclusive0600 write, exact-count check,
fsync and one close, with earlier failure preserved and close error separate.
No deferred Python output is added. The module entry **782-789** still uses
`os._exit`. Unchanged common internals and hard OS faults remain outside that
caller-owned closure.

Startup **245-268,384-394** is exact: SOURCE requires SYSTEM3.9.6 and the exact
ten required values plus all six named extra keys, then discards those six
locally and checks the original effective map. Missing or seventh extras fail.
Only LC_CTYPE was historically queried; the other five values and all origins
remain unqualified. Deletion does not undo startup effects. COPY permits the
original ten keys only, with no SYSTEM-derived exception or new observation.

## Compact receipt and data checks

R6-P2-1 remains CLOSED_SOURCE_ONLY. **645-667,759-779** retains the exact return
from the single durable full-result save, <=512-byte closed success receipt,
actual FIFO and integer PC_PIPE_BUF>=512 checks, one nonblocking stdout write,
exact-count rejection and same-phase postchecks. Missing ref or failed save/
enclosing publication cannot take success. No second save, writer, reread,
retry, drain, buffer or extra deadline appears. Failure fallback remains full,
bounded, possibly partial/absent and failed. Full raw/status/comparison data
remain in the exact R9 result construction.

Independent constant-shape reconstruction exactly reproduces SOURCE377B and
COPY373B, including LF, with their retained hashes. The fixed root is96 ASCII
bytes. For escaped root content length E, the bounds remain281+E and277+E;
E<=231/235 suffices for512. Generic admitted roots are not universally bounded
to these lengths; the caller's actual encoded-size rejection remains. The
maximum-shape21-digit end is conservative data, not an admitted parent
observation: parent v2 retains its stricter20-digit end bound. Schema alone
does not prove canonical raw bytes, arithmetic, causal order or external gates.
Actual target FIFO/PIPE_BUF, available space and delivery were not queried.
EAGAIN and other write failures remain possible. There is no delivery guarantee.

[bindings.json](bindings.json), [static-checks.json](static-checks.json),
[function-bindings.json](function-bindings.json) and
[byte-bound-check.json](byte-bound-check.json) retain the independent checks:

- All23 candidate files match original/import/worktree. All22 manifest hashes
  match. All30 R7/R8 files match their original/import/current refs; their20/8
  manifest entries match. All37 JSON files across these three folders strictly
  parse with unique keys and no non-finite constants.
- The independent zero-context diff equals the supplied patch. In-memory hunk
  reconstruction yields every final caller byte. Reversing only the three
  collector edits restores the baseline exactly. No source file was patched.
  Python3.9 AST data confirms all33 declared function bindings, only collect
  changed, the immediate clock/strict gate/finally structure, unchanged wait,
  additive record fields and exact full changed-function excerpt.
- All17 pins match current tracked bytes. All ten module destinations, sizes
  and hashes agree with materialization, source-checks, pins and exact controller
  candidate `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`. All50 unique ordered
  vectors equal independent recipe expansion: seven objects by seven verbs,
  then deep verify, including direct Autoupdate/daemon objects and exact flags.
  Canonical SDK data differs from preparation only in ten header paths.
- All12 copied contract/data files are byte-exact to R7, including schemas,
  null templates, startup, compact contract, byte proof and maximum shapes.
  The retained compact schema keywords accept the two synthetic maximum data
  shapes; the parent null templates remain non-admitting. All144 inherited
  public ref/path/blob/length/hash claims were separately rechecked. These are
  repeated identity checks, not144 behavior qualifications. The false R9 plan
  retains zero execution counts, null fresh results and no self-issued R10.

The first reviewer data reader expected five R8 JSON files; there are six.
Its count assertion failed before any write. The reader count was corrected
to16+15+6=37. That correction changed no candidate input or predicate. It is
retained in static-checks.json.

## Unresolved integration obligations and proof gaps

Parent must bind this exact caller and R10 report to its final inputs. It must
strictly parse canonical raw success bytes, check exact types/keys/mode/phase/
clock, retain the exact full file bytes/hash/path, and inspect every one of53
SOURCE or52 COPY caller records and all durable raw/status/comparison refs.
The new eligibility facts must satisfy the repair contract alongside every
existing actual/null, EOF, drop, first-cause, partial and publication predicate.
Require actual caller0/reap and complete EOF without error/loss/kill/deadline
uncertainty before the SAME original phase end. A compact receipt, file,
sequence flag, hash or native50 alone supplies no acceptance or actual exit.

Transport `1c46c16` has SOURCE CLEAR only under `a7e22725`, imported8125221.
The tracked263c758 observation records actual exit1 for the real empty CLI;
this is a failure-branch observation with DEVNULL console, no collected EOF,
connected-flow or D4 proof. I read only its public retained data. Payload
`f78f2be0` remains separately under review as supplied; its public template's
final caller and review slots are null. This review neither clears that source
nor renders/binds a payload. Four-input materialization remains parent work.
The public local-packet report states OLD R7 bytes and disabled status; the
private packet was not opened or changed. Profile75ef628 and local coordinator
190e11f remain bounded prerequisites only, not route/clock/D4/runtime proof.

The public saved audit f7 and basis bridge ce7 retain their bounded scopes.
Raw wrapper1670..., full canonical wrapper7fd7... and entire50-row commands
projectionb46d... remain distinct; no private basis was reopened. Parent
causal inspection, pre-entry authentication and complete artifact/operator/
receipt gates remain external duties. No source hash or file can grant runtime
acceptance.

Only reviewer-authored static file/Git/hash/patch/JSON/AST-data readers and
the owned report/bindings commit ran. No supplied source, check or AST was
executed, evaluated, imported or compiled to executable code. No tests, unit
tests, mocks, private access, remote/network/native/process/port query, child,
private Python, source metadata, preparation, controller, product, build,
signing, credential or activation ran. All candidate execution counts are zero.
No TypeScript changed. End-to-end work is excluded by this explicit commission.
Only this NEW folder is owned; commit hooks and signing are disabled.

No actual while-read behavior, timing, clock equivalence/rate/conversion,
private startup, pipe delivery, signal/reap success, OS-stall bound, fault/power
durability, connected transport, D4, parent integration, native metadata,
signature/ABI, UI, performance, release or whole-flow acceptance is proven.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause UNKNOWN;
whole-flow NI15/runtime UNVERIFIED; earlier23/25 FAILED; all failed files and
guards; missing original second post-bind, bundle-after, outcome/archive;
stripping warning; unknown full outer build duration/exit; and the closed
updater. The completed source-file pass and this SOURCE CLEAR add no runtime
or approval grant.
