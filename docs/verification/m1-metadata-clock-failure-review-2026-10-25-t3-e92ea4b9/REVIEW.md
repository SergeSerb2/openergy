# Independent metadata clock and failure repair review

2026-10-25, America/Vancouver. **HOLD: one concrete P2 finding (R3).
No P1 found. Original R1 and R2 are closed in the bounded source review.
SOURCE/DATA ONLY. execution=false. Runtime UNVERIFIED.**

Candidate: `e4ec20bdc6058efaa2fdb272ca88b84a7519116c`, imported at clean
parent `0c95b8a70376bde882c459ef42071906c4bd058c`. All 15 candidate files equal
the candidate Git blobs. The import changes no candidate-folder byte. This
review owns only its new folder. It changes no caller, transport, common,
helper, controller, recipe, historical evidence or acceptance predicate.
There is no new human approval gate.

All caller locations below refer to
`../m1-metadata-clock-failure-repair-2026-10-25-t3-d1e829a9/metadata-caller.py.txt`.
I read the full 715-line caller, full 530-line patch, full new README, full
prior review, old interface README, schemas, templates, closed CLI, plan,
pins, vectors, bindings and static-check records. I also read the preparation
README, next-operation, metadata and canonical SDK recipes, file-helper
interface, full operator plan and receipt contract, and common's writer
contract. Unchanged helper and controller logic were not independently
re-reviewed. Public saved-file and basis-bridge audits remain bounded data
support; no private evidence was opened.

## P2 R3: Prevent a buffered flush retry during cleanup or shutdown

Primary locations: **595-600, 706-715**, with alarm restoration at **92-96**.
The same caller-owned buffered cleanup mechanism also exists at **199-203**.

`emit_summary` writes through `sys.stdout.buffer`, then flushes it. The closed
SYSTEM and private launchers supply neither `-u` nor an unbuffered-output
environment setting. If the phase alarm interrupts a flush while buffered
bytes remain, the exception leaves that buffer alive. `until` then disables
its timer and restores the previous handler. There is no surrounding phase
alarm at this final call. `main` catches the exception, returns 1, and line715
uses normal `sys.exit`. CPython finalization flushes the standard streams
again. Thus one explicit summary call can cause a second output attempt
after the shared deadline, outside `until` and without its postcheck.

A concrete static path is a summary with a buffered tail and a full stdout
pipe. The phase alarm interrupts the tail's flush. The parent later resumes
reading in its failure-reporting envelope. Normal interpreter shutdown can
then send the retained tail after the metadata phase has ended, or block
again while trying. This needs no uninterruptible OS stall. It is an explicit
library cleanup path. The run still fails; this finding concerns the promised
single publication attempt and absence of late publication. A parent kill can
contain it, but does not remove the caller's extra output path.

The source writer has a related inherited path. Its `os.fdopen(fd, 'wb')`
uses a buffer. If line202's explicit flush raises at the deadline with pending
bytes, the context manager closes the stream before leaving `until`. Buffered
close calls flush again. The fired one-shot alarm is no longer armed, and
the enclosing timer is restored only after that close returns. A small source
record can therefore receive another write attempt during exceptional close.
The new failure latch preserves the earlier command facts, but cannot stop
this implicit write. These two sites are one buffered-cleanup finding.

This deduction is supported by the pinned official implementations:
[CPython 3.9.6 buffered flush and close](https://github.com/python/cpython/blob/v3.9.6/Modules/_io/bufferedio.c#L454-L501)
and [its flush error path](https://github.com/python/cpython/blob/v3.9.6/Modules/_io/bufferedio.c#L1747-L1799)
retain pending state on a flush error and flush during close.
[3.13.9 has the same flush error behavior](https://github.com/python/cpython/blob/v3.13.9/Modules/_io/bufferedio.c#L1889-L1940).
Normal finalization flushes stdout in both
[3.9.6](https://github.com/python/cpython/blob/v3.9.6/Python/pylifecycle.c#L1312-L1318)
and [3.13.9](https://github.com/python/cpython/blob/v3.13.9/Python/pylifecycle.c#L1968-L1974).
No failure injection or runtime experiment ran here.

Minimal fix: remove deferred buffered payloads from these caller-owned
publication paths. Use unbuffered exclusive source writes while retaining
the same caps, short-write rejection, explicit fsync and failure states.
Use a bounded raw/nonblocking summary attempt that leaves no Python output
buffer for shutdown to flush. Stop on partial/failure, keep the parent's
partial streams and actual exit, and give no additional time or retry.
Keep the exact launcher flags/environment, unchanged common/time, regular
COPY reservations and one writer. Review the exceptional close/finalization
path as well as the explicit write. This is a requested source repair, not
transport code added by this review or a dependency on the parallel proposal.

## Original R1: Closed in source, with external proof limits

Lines35-38 and62-75 define explicit integer nanoseconds from
`time.clock_gettime_ns(time.CLOCK_MONOTONIC)`, darwin, clock ID6,
same-host/same-boot scope. Missing API, wrong platform/ID, read failure and
negative/non-integer clock values fail. There is no unsupported-clock fallback.
Lines610-638 require canonical unsigned ASCII CLI start, at most20 digits,
an actual integer parent start/deadline, exact identity and exact
`deadline = start + 120000000000`. New command and COPY-entry observations
use that domain. The old process-relative timestamps are not compared with it.

The original issue is independently supported by
[3.9.6's process-local origin](https://github.com/python/cpython/blob/v3.9.6/Python/pytime.c#L772-L821)
and [3.13.9's implementation](https://github.com/python/cpython/blob/v3.13.9/Python/pytime.c#L1074-L1094).
The explicit 3.9.6 API instead calls `clock_gettime` and converts its timespec
to integer nanoseconds; see the official
[time module source](https://github.com/python/cpython/blob/v3.9.6/Modules/timemodule.c#L187-L207).
This supports the source choice. It is not a new measurement.

Lines270-317 consume the completed file pass without rerunning it. The seven
length/hash declarations equal the caller constants, source template and
schema, completed-source manifest, and public audit index. The two local
parent files remain parent inspection duties. The caller checks the five
remote fact/raw/status/guard files against exact bytes, facts/stdout equality,
empty helper stderr, helper exit0, EOF, first error, kill, deadline and
guard/root/argv facts. Original remote start/end/exit arithmetic stays within
that old wrapper. The helper's elapsed value remains a duration. Old local
SSH times stay in the local parent's original record. Neither set is retagged
or offset into a new clock domain.

The new SOURCE parent envelope requires the audit commit and manifest hash,
root, actual helper0/SSH0 and a new causal-inspection statement. It orders
`0 <= inspected_ns <= phase_started_ns`. Parent must inspect the actual seven
records and guards before those new readings, then start the phase before
writing/authentication. The caller cannot establish the parent's past causal
act or reap a past local SSH handle. The README and closed CLI retain that
external boundary explicitly. It is not replaced with hash-only admission.

COPY's parent starts its phase before SYSTEM `entry`, inspects full prefix
facts before the single private `-B` entry, then supplies a hashed bounded
stdin envelope. Lines320-344 require actual exit0, reap, both EOF, no error,
timeout, overflow, drop uncertainty or kill, exact clock identity, ordered
phase/helper/inspection readings and helper duration at most10s. Its later
inspection cannot establish trust before its own entry; parent still owns it.

`common_deadline`, lines99-112, samples local monotonic seconds first and
shared nanoseconds later. It subtracts1ms from the remaining shared duration,
rounds the duration and sum downward, and passes a local absolute deadline
to unchanged common at line206. It discards sample time rather than adding
it back. It does not pass shared absolute nanoseconds to common, patch time,
calibrate an offset, reset a phase or borrow a later budget. The surrounding
shared alarm/postcheck and common's nested local alarm/postcheck remain.
The1ms deduction is not a measured clock-conversion error bound.

The new schema and templates agree on closed keys, clock and manifest. The
caller imposes additional integer, arithmetic, current-time, root, hash,
status and evidence checks. JSON Schema alone cannot enforce causal acts or
all cross-field arithmetic. Both null templates deliberately fail admission.
Schema numeric semantics can admit integral JSON numbers that Python's exact
`int` checks reject; this is stricter executable admission, not a weaker gate.

Prerequisite `b4c7d11ad63dbb6da57077805f12c6b0a0b15d1f` reports SYSTEM3.9.6,
clock6 API availability, three readings and resolution approximately1us.
Its tracked blobs match their exact refs. It does not qualify private3.13,
cross-process equivalence, rate differences, conversion accuracy or runtime.
No clock probe was run by this review. R3 prevents an overall deadline-path
clearance despite closure of the old clock-origin mismatch.

## Original R2: Closed before publication

At lines500-508, immediately after collection, `perform` stores the active
label and full bounded collector-status copy, appends it to the retained
command list, and latches a failed COMMAND status before any raw/status
write, fsync or payload comparison. The failure is distinct from a comparator.
Lines510-525 keep attempted paths, states and durable references separate.
A write, flush, fsync or directory-fsync exception cannot fabricate a durable
reference. Later streams remain not attempted. A writer error stays separately
in `publication_errors`; no later path query fills missing evidence.

Payload comparison begins at line528 only after durable stdout, stderr and
OS-status records. A failed comparison is latched at550-551 before comparator
publication. Its full assigned value survives a later publication exception.
Lines679-688 retain COMMAND failure as the first cause and keep later stop
detail separately. Incomplete raw publication leaves comparison null and
not started. These paths close the original failure-loss finding.

Each created Popen handle stays in global HANDLES with label, argv and its
available collector status. This prevents normal in-function Popen destruction.
The bounded final summaries at587-592 and691-702 use already-observed
`returncode`, without polling, reaping or inferred exits; null remains null.
If collection itself is interrupted before a full status is assigned, that
status may remain null. The code does not invent the missing collector facts.
Parent still owns actual outer caller exit, EOF and deadline observations.
The separate buffered-summary flaw above does not erase the early latch.

## Preserved controls and independent data checks

- The finite `perform` call expressions equal the frozen caller AST. SOURCE
  still runs trusted SYSTEM `entry-template`, then exact private
  `-I -S -B --version`, fixed Python lipo and closed50. SOURCE never imports
  common. COPY hashes all17 inputs before common import, gets only the
  prepared canonical bundle, and runs SYSTEM copy checks before and after50
  in the same phase. Fresh SOURCE receipts require v2 and the new clock.
- All50 unique ordered vectors equal independent recipe expansion: seven
  objects by seven verbs, then deep verification. Autoupdate and daemon use
  their direct executable objects. No arbitrary argv, product command,
  shell, command retry or completion pass was added. Both pin/vector files
  equal the frozen package byte for byte.
- All17 pinned inputs match tracked hashes and lengths. All ten module
  destinations, sizes and hashes match materialization, source-checks and
  exact controller candidate `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`.
  The canonical SDK recipe differs only in the ten header paths. Helper CLEAR
  `20187fe1b286851f1416437c40cc27108269b542`, SDK data CLEAR
  `c490566f5306d79f2537e9bbce3c75db79a279d3` and saved-file audit
  `f7f9b10fc99cc2196d0fd88a5b93a35a39a9adc5` retain their original scopes.
- The basis bridge `ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13` is
  BOUNDED_DATA_CLEAR for the complete50-record commands projection. Its
  public report, projection and checks equal their bound Git blobs. They
  report42 source rows plus8 direct-object supplement rows and100 equal raw
  streams. This review did not read the private wrapper or original outputs
  again, and did not rerun that audit. Full-wrapper and projection hashes
  remain distinct. This data clearance supplies no caller/runtime result.
- Collection retains one Popen call site, fixed environment/cwd, DEVNULL
  stdin, shell=False and closed fds. Nonblocking reads retain at most1MiB
  combined, allowing one overflow-detection byte during reading. Actual exit,
  both EOFs, observed drops/unknown unread bytes, kill delivery, reap and
  close errors remain distinct. There is no later drain. Explicit waits are
  bounded. The10s command end includes0.5s held for kill/reap and ends at
  most phase-end minus1s. Final pipe/selector close time is checked. These
  checks fail on losses; they do not prove hard OS preemption or containment.
- Comparison policy is unchanged. Only labels ending `-verify` sort complete
  strict-UTF-8 splitlines lists per stream, preserving duplicates and empty
  lines. Other output is exact after only the declared bundle substitution.
  COPY compares saved basis and fresh SOURCE. Failed commands cannot pass
  from an exit code or payload alone.
- SOURCE sums initial owned0600 regular files and owned0700 directories
  before its first write; it initializes no ledger. Its fixed cap preserves
  product16MiB, cleanup4MiB and accounting64KiB within32MiB, with8MiB/file.
  COPY keeps one Writer, existing ledger/lock and unchanged
  `common.write_bytes(..., critical=False)` regular pre-write double charges
  for guards, parent envelope/streams, raw/status, comparator and result.
  It does not initialize/reconstruct the ledger or borrow a reserve.
  Explicit fsync errors are recorded as publication failures. R3 limits the
  separate claim about buffered cleanup after a flush exception.
- All14 SHA256SUMS entries match. All11 JSON files parse. The exact patch
  has21 hunks and reconstructs the new caller byte for byte in memory. It
  equals independent unified-diff generation. No patch was applied to any
  source file. Python3.9 AST parsing and structural comparison used syntax
  trees only as data. Nothing supplied was executed, evaluated, imported
  or compiled.

## Exact bindings and proof limits

```text
new caller       464ab85baebf47e3b373463768e9abb2001d39dd181b4e2a790e1868d2b90597
frozen caller    26bcd57988734f54b2baa14a2b54886ab9ee09c2db1e342a137a49d23c42e18b
exact patch      73afa67d656d2dc16f6e6a569d59296edd0ead1010e3597fb0cd39f99757476f
pins             31a17c129e0af6c8c0437458c01fa8c28cffb89f29ee327648a31fed96608352
vectors          e3ff5756ab3b35a5c815e50ac1fad6520117836e080849260de87592bd327fa0
seven manifest   1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26
completed facts  ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192
raw wrapper      1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f
full wrapper     7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159
commands / E     b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0
helper           bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428
common           20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c
controller       08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4
canonical SDK    d90586546d6b3127dca26147b9d090544d29d7a8b229479966dae68ef25d3b11
source-checks    2bde5cf1f9396df5c7d8518caaadcdd69eafc5ff2f39117158efaf87e8d080f5
build bindings   20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617
```

The new caller is42482 bytes/715 lines. The old caller stays30793 bytes/529
lines at frozen commit `621e0e3a9037e15b391e05b964dfcdb25da01515`.
[bindings.json](bindings.json) records full paths, Git refs/blobs, byte lengths
and hashes, including all15 candidate files, ten frozen files,17 pins, ten
modules and public audit support. It also records the exact seven private
raw bindings as declarations, mapped to the public saved audit's distinct
local and retained-remote paths. Those private bytes were not independently
read here. [static-reader-notes.json](static-reader-notes.json) preserves an
optional absent public path and a corrected local/remote basename ambiguity
in the reviewer-owned data reader. Neither changed a candidate predicate.

No supplied code/AST, test/unit/mock, native parser, OS/process/port probe,
remote target query, private interpreter, private read/write, credential,
prepare/copy/controller/product, build, signing, integration or child-agent
action ran. Only static file/Git/JSON/hash/AST data readers and official
primary-source web reads were used. New caller, SOURCE50, private-version,
prepare, copy, controller and product execution counts are all zero.
No TypeScript changed. The commit uses disabled hooks and signing and only
this owned folder. Source proof does not establish clock conversion accuracy,
OS-stall hard bounds, full runtime, whole-flow NI15, native UI, performance,
artifact, signature, ABI, release or integration acceptance. Parent causal
inspection and real outer handles/exits/EOF/deadlines remain external duties.

Historical states remain exact: old strict SDK qualification
FAILED_UNVERIFIED, helper1/SSH1; older M1 app25/25, app0/controller1
FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause UNKNOWN;
whole-flow NI15 UNVERIFIED. Earlier23/25, all old failed records and guards,
original second post-bind, bundle-after, outcome/archive gaps, stripping
warning and unknown full outer build duration/exit remain. No completion,
fresh artifact gate or runtime result follows from this review.
