# Inert metadata clock and first-failure repair

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
Independent repair review REQUIRED before use. Runtime UNVERIFIED.**

This folder alone is owned. Parent is `11d5d49683357e1469c100c960310a491b84f126`.
The frozen caller is `621e0e3a9037e15b391e05b964dfcdb25da01515`, SHA-256
`26bcd57988734f54b2baa14a2b54886ab9ee09c2db1e342a137a49d23c42e18b`.
It and all historical files stay byte-exact. This repair addresses both P2s
in the [full independent review](../m1-artifact-metadata-caller-review-2026-10-25-t3-0ac2fca9/REVIEW.md),
commit `54bc355f5f733008a444c45cafe1e4aa7b819777`: R1 clock domains and R2 raw
publication before first-failure retention. This implementation is not an
independent clearance of itself.

[metadata-caller.py.txt](metadata-caller.py.txt) is the FULL replacement inert
source. [caller-old-to-new.patch](caller-old-to-new.patch) is the exact old/new
unified delta. Apply it only to a separate materialized copy of the frozen
caller; do not change the historical repository file. [bindings.json](bindings.json)
and [SHA256SUMS](SHA256SUMS) bind the old and new bytes. No executable bit or
entry permission is supplied here.

Before design I read all 529 old caller lines, the full old README, pins,
vectors, bindings, finite plan and review. I also read the full preparation
README, helper, metadata and canonical SDK recipes, next-operation, operator
plan, receipt contract, and unchanged common writer contract. These remain
the authority for all unchanged behavior. The new plan supplies no artifact,
prepare, controller, product, release or runtime gate.

## Explicit clock and consumer conversion (R1)

Every NEW absolute timestamp uses this exact closed identity:

```json
{"schema":"openergy.shared-clock.v1","platform":"darwin","api":"time.clock_gettime_ns","clock":"CLOCK_MONOTONIC","clock_id":6,"unit":"nanoseconds","scope":"same-host-same-boot"}
```

Parent, SOURCE SYSTEM Python and COPY private Python must use that API and
identity on the same M1 host and boot. Missing API, wrong platform/clock ID,
failed read, or invalid value stops. There is no fallback to `time.monotonic`,
wall time or another clock. Integer nanoseconds are units, not a resolution
claim. Parent evidence `b4c7d11` reports target SYSTEM3.9.6 API availability,
clock ID6 and resolution approximately 1e-6 seconds. Its three observations
and raw stdout hash are bound in bindings.json. Only those tracked public
blobs were read; the new private query root was not read. That observation
does not measure cross-process equivalence or qualify the conversion/private
interpreter. No new query ran here.

The [Python time documentation](https://docs.python.org/3.13/library/time.html#time.monotonic)
records the macOS cross-process change in 3.10. The pinned
[3.9.6 implementation, lines772-821](https://github.com/python/cpython/blob/v3.9.6/Python/pytime.c#L772-L821)
subtracts a process-local origin. The
[3.13.9 implementation, lines1074-1094](https://github.com/python/cpython/blob/v3.13.9/Python/pytime.c#L1074-L1094)
does not. I read those official sources. The repair therefore uses an explicit
OS clock for NEW cross-process observations instead of assuming that the two
interpreters' `time.monotonic()` absolute values agree.

The shared phase end is **parent start +120,000,000,000ns**. Parent starts
COPY before entry authentication. It starts SOURCE after the completed file
pass has been inspected, before writing its new parent envelope and launching
the caller. Reads, pin checks, import, authentication, command capture,
comparison, publication, pipe close, kill and reap all consume that same phase.
No caller/helper entry or writer call resets it. Each command has at most10s;
the collector still holds0.5s inside that10s for kill/reap and ends no later
than phase-end minus1s. Nonblocking reads still cap combined retained output
at1MiB DURING reads, with one overflow-detection byte, explicit EOF/drop facts
and no later drain.

Unchanged common consumes a **local `time.monotonic()` seconds deadline**.
`common_deadline` samples that local clock first, then the shared clock. It
discards the sample interval, subtracts1ms from the remaining shared budget,
rounds the duration and local sum downward, and passes only that conservative
local end to `common.write_bytes`. It does not pass shared absolute ns to
common, invent an offset, change common/time, or retain an offset calibration.
The surrounding shared `until` alarm and postcheck remain authoritative;
common's own nested alarm/postcheck remains unchanged. The1ms is a conservative
deduction, not an allowance or measured error bound. Scheduling, clock-rate
effects and uninterruptible OS stalls are not hard real-time guarantees.

Parent must retain its actual outer handle, actual OS exit, both EOFs, drops,
errors and kill/reap observations, all stamped with the NEW shared clock.
The actual caller exit must be observed within the SAME120s phase. A caller
result file precedes its own exit and cannot establish that fact. An outer
failure-reporting envelope grants no extra work, command or publication time.

## Completed source-file admission, without retagging or repetition

The already completed qualification has facts SHA-256
`ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`.
Saved-data audit `f7f9b10fc99cc2196d0fd88a5b93a35a39a9adc5`, imported in
`d79ed01`, supports **file pass only**. The parent has already selected a NEW
evidence root ending4b20357c1503. This package retains generic `EVIDENCE` data;
it creates or searches no root. A different root cannot satisfy the exact
retained helper guard and its argv/root binding.

[completed-source-bindings.json](completed-source-bindings.json) binds the
seven permitted saved files: local parent result and once guard, plus the five
retained remote fact/raw/status/guard files. I read only these seven leaves,
5,505 content bytes, at most1MiB each. The reader used directory-relative
`O_DIRECTORY|O_NOFOLLOW` for each component and `O_NOFOLLOW|O_NONBLOCK` for each
regular leaf. It checked size/type and stable metadata before/after reading.
It did not inventory the private directory or open another private path.
Only hashes and sanitized fields are published; no SSH argv, alias, endpoint
or PID is published.

The SOURCE envelope requires the exact seven length/hash bindings and audit
commit. Parent must inspect all seven original records, the completed facts,
actual helper0/SSH0, complete EOF, first-error/kill/retry/deadline facts and
both guards' payload/review/vector/root bindings. The local parent/SSH guard
stays with the parent; it is not copied into remote/public caller data. Only
AFTER that inspection does parent read a NEW `inspected_ns`, then a NEW
`phase_started_ns`. The envelope hashes the exact manifest and binds the root,
actual exit observations and causal statement. Null template fields do not
admit. A hash or parent assertion alone cannot replace the required inspection.
The caller cannot independently observe the past local SSH process or prove
the parent's external causal act; that remains a declared parent boundary.

Inside its SOURCE phase, the caller also reads the five exact existing remote
files under EVIDENCE. It checks their fixed raw hashes/lengths, identical facts
and stdout, empty stderr, actual helper exit0, complete EOF, no first error or
kill, the source status, guard argv/root/once binding and original own-process
deadline facts. It never invokes `source` qualification again. The old remote
`~0.x` times remain raw SYSTEM3.9 process-relative values. The old local parent
times keep their original domain. Only within each original process may their
elapsed/deadline predicates be assessed. Neither old set is retagged, offset,
or compared to a NEW absolute clock. The helper's emitted `elapsed_seconds`
also remains its own duration, including for future helper calls.

## Closed interface and unchanged order

[closed-cli.json](closed-cli.json) specifies exactly four positional arguments:

```text
source|copy EVIDENCE START_CLOCK_MONOTONIC_NS PARENT_RECORD_SHA256
```

The start is canonical unsigned ASCII decimal, at most20 digits. The hash is
the lowercase SHA-256 of the exact parent bytes. [parent-boundary.schema.json](parent-boundary.schema.json)
is closed data for `openergy.metadata-parent-boundary.v2`. The caller enforces
shape, identity, type, arithmetic and evidence predicates; schema validity
alone is not a gate. [source-parent.template.json](source-parent.template.json)
and [copy-parent.template.json](copy-parent.template.json) are deliberately
incomplete. They contain no actual NEW times, root or fresh authentication.

SOURCE uses the parent's exact `/usr/bin/python3 -I -S -B` launcher and
`EVIDENCE/metadata-source-parent.json`, bound by the CLI hash. Source performs
SYSTEM `entry-template` authentication, then exactly the private
`-I -S -B --version`, fixed Python lipo, then closed50. It never imports common.
These commands and all source evidence use the same shared120s phase.

COPY parent first takes the shared phase start. It runs the pinned SYSTEM
helper `entry`, captures bounded streams and actual exit including kill/reap
within10s, and inspects the full prefix facts BEFORE the single private
`-B` caller entry. Its NEW entry observation carries the exact clock,
`started_ns`, `finished_ns`, `parent_inspected_ns`, actual status, reap, EOF,
overflow/drop/timeout/error/kill fields and bounded base64 streams. The caller
checks phase-start <= helper-start <= finish <= inspection <= current clock
< phase-end, with helper duration <=10s. This later check cannot authenticate
an interpreter before entry; parent still owns that causal gate.

COPY receives the hashed parent JSON on stdin, capped at1MiB and closed by the
parent. No unmetered post-prepare parent file or extra writer process is added.
The caller checks every17 pin and the closed vector/basis hashes before common
import. It uses only the canonical private copy from prepared.json. It reads
fresh SOURCE **v2** records, runs SYSTEM `copy` file checks BEFORE closed50 and
AFTER closed50, in the same120s. The source receipt and every command record
must carry the new clock/schema; old records cannot satisfy these interfaces.

The ten modules, helper, limits, source bindings, recipes and disabled artifact
template remain exact. `input-pins.json` and `command-vectors.json` are copied
byte for byte. There are exactly50 unique ordered native vectors: seven
objects times seven verbs, then deep verify. Autoupdate and daemon remain
direct code objects. The raw saved basis stays
`1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`;
it was NOT read again. The later bounded bridge audit
`ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13`, imported in `15e9f45`, is
BOUNDED_DATA_CLEAR. Its tracked report/projection/checks were read as data.
It establishes that the historical corrected EXPECTED equals the wrapper's
entire50-record `commands` value. Both command projections have canonical
SHA-256 `b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`.
The full wrapper canonical hash remains
`7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159` and its
raw hash stays1670...; its four envelope fields explain the different shape.
All50 complete rows and100 raw streams match42 source rows plus8 direct-object
supplement rows in that bounded audit. This closes the provenance gap left
UNVERIFIED by the earlier audit; it supplies no new metadata/runtime result.
All hashes, vectors and comparison policy stay unchanged. No transport code
or transport acceptance is added by this repair.

## First observed command failure and publication (R2)

Immediately after `collect`, `perform` retains the active label and a full
bounded collector-status copy in the single Writer's in-memory command list.
It latches `first_failed_command` BEFORE raw/status writes or fsync whenever
that actual status fails. This is a COMMAND failure record. It is not a
payload comparator. A later writer error cannot replace it as the first cause.

Each stream/status publication has an attempted path, a state and a separate
durable reference. States distinguish not-attempted, attempted, durable, and
failed-or-partial-unverified. Only a successful writer+fsync return supplies a
durable length/hash reference. A write/fsync failure may have left an absent,
partial or linked but unverified file. No fabricated raw reference or later
file query fills that gap.

Only AFTER durable stdout, stderr and actual OS-status records does payload
comparison begin. Its rules are unchanged: only `-verify` labels sort complete
UTF-8 splitlines per stream, preserving duplicates and empty lines; every other
output is exact after only the declared bundle substitution. COPY compares
both saved basis and fresh source. A previously assigned failed comparison
stays protected if its later publication fails. An incomplete raw publication
has `comparison=null` and `comparison_state=not_started`.

Publication errors have their own bounded messages in `publication_errors`.
The result keeps the active command, collected statuses, partial raw refs,
first failed COMMAND, first failed comparison and separate stop detail. It
also includes every retained direct handle's label, argv, actual returncode
and full available collector status. Null exit/status stays null. The summary
does not poll, reap, infer an exit, or drop earlier handles. Pipe/selector-close
errors are retained as collection errors. The one bounded stdout summary
attempt carries these facts if the original phase still has time. A partial
summary is not retried. No late file writer or completion path exists; if the
deadline is gone, the parent retains outer partial streams and actual exit.

SOURCE keeps exclusive0600 initial writes and no ledger. It sums existing
initial files before any write and keeps8MiB/file,32MiB total,16MiB product,
4MiB cleanup and64KiB accounting reservations. COPY uses the same Writer and
unchanged common `write_bytes(..., critical=False)` regular pre-write double
reservations for EVERY new guard, parent envelope, stream, status, comparator
and result. No reserve borrowing, ledger initialization/reconstruction, second
writer, command repeat, retry, completion pass or late publication allowance
was added.

## Static verification and limits

[static-checks.json](static-checks.json) records AST/JSON/hash/patch checks and
exact changed/added functions. AST was parsed as data with3.9 grammar; neither
source nor AST was executed, evaluated, imported or compiled. Static patch
reconstruction must produce the full new caller byte for byte. All17 pins,
ten module bindings,50 recipe vectors and historical file bytes are checked.
The exact patch retains required single-space empty context lines; Git flags
those patch-data lines as whitespace. The non-patch files pass whitespace
checking. These are source/data checks, not behavioral, failure-injection, clock-rate,
deadline, writer or end-to-end proof. Independent repair review remains needed.

No supplied code/API probe, tests/unit/mocks, native parser/command, process/
port query, private Python, remote operation, private write, credentials,
prepare/copy/controller/product action, build, signing, integration or child
agent ran. Only official documentation/source network reads occurred here.
No TypeScript changed. Hooks and signing are disabled for the owned-folder
commit. The new caller, source50, private-version, prepare, copy, controller
and product execution counts in this task are all zero.

The old strict SDK attempt remains FAILED_UNVERIFIED, helper1/SSH1. The older
M1 app25/25, app0/controller1 attempt remains FAILED_NOT_ACCEPTED; first strict
post65431 EADDRINUSE cause UNKNOWN; whole-flow NI15 UNVERIFIED. Earlier23/25,
all old failed records and guards, original second post-bind, bundle-after,
outcome/archive gaps, stripping warning and unknown full outer build duration/
exit remain. Nothing here fills those gaps or grants artifact/native/UI/
performance/release/integration acceptance. [finite-plan.json](finite-plan.json)
stays false; the full unchanged receipt contract and every parent gate remain.
