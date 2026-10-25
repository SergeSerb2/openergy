# Independent M1 metadata R6 SOURCE/DATA review

2026-10-25, America/Vancouver. **HOLD: one bounded P2 finding, R6-P2-1.
No P1 found. execution=false. Runtime and whole-flow NI15 UNVERIFIED.**
Original R1 and R2 remain CLOSED_SOURCE_ONLY. R4's R3 buffered-publication
finding is CLOSED for the caller-owned source paths. This review grants no
execution, artifact, native, runtime or approval authority and adds no human gate.

Candidate is `2667d808fb7807ba6b2591c4141938f1cefabd90`; review parent/import is
`2482f09a8a598e0ad86154f47d82426c1adffe72`. The checkout started clean there.
All 17 candidate-folder files equal their candidate Git blobs, import blobs
and worktree bytes. Candidate parent is `a26871ad9fdae2d55ff53544aed26ae0cdf2620e`.
The import also contains the separate four-file transport-size report from
`0b06ca9`. That difference changes no R5 byte. Only this NEW review folder is owned.

All caller locations below refer to
[`metadata-caller.py.txt`](../m1-metadata-r5-repair-2026-10-25-t3-c4019bfc/metadata-caller.py.txt).
I read the complete 761-line caller, exact six-hunk patch, README, startup
contract, full changed-function excerpts, bindings, static-check declarations,
CLI, plan, schema, templates, pins and vectors. I read both prior full reviews,
both prior interface READMEs, preparation README, next-operation, metadata and
canonical SDK recipes, file helper, full operator plan, receipt contract and
common's writer interface. Unchanged helper/controller behavior was not newly
qualified. Public saved-file/basis audit bindings remain bounded prior support;
no private raw evidence was opened.

## P2 R6-P2-1: Make a successful summary fit its one-write pipe contract

Primary location: caller **632-640**, especially **638-640**. Failure/exit:
**743-751,754-761**. Result publication: **731-742**. Required duplicated data:
**536-563,624-629,719-736**. The R5 README describes this choice at **58-67**.

`emit_summary` serializes the complete result, sets fd1 nonblocking, and
requires a single `os.write(1, data)` to accept every byte. A short count raises;
`main` returns1; the module uses `os._exit(1)`. There is no pending tail, second
write or shutdown flush. This closes the old late-flush path, but makes normal
success depend on the whole result fitting one immediately writable pipe window.

The separately retained size report at
[`m1-source-transport-size-binding-2026-10-25`](../m1-source-transport-size-binding-2026-10-25/README.md)
has a deliberately incomplete successful 50-command shape of **209,116 bytes**.
I independently rebuilt that shape from the exact vectors and bound public
paths. It equals `minimum-shape.json` byte for byte, SHA-256
`b0b8fc94cf7b1c44fc31128aee173e192798cb552726d8909c422c1170b50c13`.
It omits auxiliary commands, most status/clock fields, outer fields, the active
command, direct-child stream refs and nonzero byte lengths. The result-building
functions and statements remain unchanged from R3. R5 therefore retains this
lower bound for the bound SOURCE path; it cannot produce a small success summary.

A concrete ordinary pipe path is: all53 SOURCE caller commands and durable
result publication succeed, with phase time left; the stdout pipe has, for
example,64KiB of immediately writable space; its reader does not run during
this write. The nonblocking operation can accept that prefix and return a
short count. At least143,580 bytes remain even for the reduced shape. The caller
then exits1 and closes the stream. A reader which next drains promptly, has a
1MiB storage cap and still has phase time cannot recover the unsent tail. It
gets incomplete JSON and an actual failed caller. No command, comparison,
quota, native error or deadline expiry is needed for this failure path.

Static local Apple SDK documentation supports this distinction: `write.2`
lines145-152 documents short nonblocking writes on flow-controlled objects;
lines201-203 documents EAGAIN when no bytes can be written immediately. Local
`sys/pipe.h` lines66-81 declares16KiB ordinary and64KiB large buffers, and
`sys/syslimits.h` line104 declares512-byte atomic writes. Atomic allowance,
pipe storage and reader capture cap are different quantities. Their local
paths, hashes and line ranges are retained in [bindings.json](bindings.json).
These are static documents, not a measurement or qualification of the M1's
kernel or pipe. The finding does not assert that every target schedule fails:
concurrent reader progress can change a large write's return. It identifies a
normal permitted schedule which this caller turns into a failed full CLI run.

The separate local transport's **128KiB stdout cap is not the cause here**.
Per the parent brief, it will receive a small separate SOURCE-launcher receipt.
The SOURCE payload under separate implementation will capture the full caller
with1MiB capacity. Raising that reader cap does not enlarge fd1's immediate
write window or make this one call complete. No unfinished payload or transport
implementation is cleared by this review. COPY uses the same `emit_summary`;
its large result has the same structural delivery dependency.

Minimal repair within the single-attempt rule: after durable full result
publication, retain the exact result reference and emit a separately closed,
compact success receipt sized for a justified one-write pipe allowance. Keep
the full result, statuses, raw refs, failure details and comparisons as evidence.
Bind the compact interface in the parent; still require actual caller exit0,
complete EOF and all mandatory raw predicates before the same phase end. A file
or receipt alone must not accept. Failure fallback may remain partial and failed.
Do not retry a partial summary, add time, add a second writer, change common,
weaken caps or change fixed launch flags/environment. This is a source repair
recommendation only; no repair or output interface was implemented here.

## Prior finding disposition

**R4 R3, CLOSED_CALLER_OWNED_SOURCE_ONLY.** Caller **194-226** retains exclusive
`O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW`, mode0600, pre-write aggregate accounting,
file cap and fsync/directory-sync checks. Its one raw write cannot leave bytes
in a Python output buffer. Short, zero-for-nonempty, write and fsync failures
stop. The `finally` attempts raw close once. If close alone fails, it propagates;
if an earlier write/fsync failure exists, that earlier exception propagates and
the close error enters `publication_errors` separately at **220-221**. There is
no refund, removal, retry, repaired file or durable ref on failure. An empty
write still requires a zero return and successful sync/close checks.

Caller **632-640** places no payload in `sys.stdout.buffer`. Module **754-761**
maps main's0/1 return or any uncaught BaseException to `os._exit`; normal Python
finalization cannot flush this summary, emit a late traceback or run handle
destructors. Pre-admission failure can have no summary. The former explicit
buffered-close/shutdown retry is gone. R6-P2-1 concerns complete normal delivery,
not a retained Python buffer. Raw close/fsync may still stall or have uncertain
OS effects. Unchanged common has its original buffered internals; this closes
the caller-owned R3 sites, not all common/library cleanup or hard OS timing.

**Original R1, CLOSED_SOURCE_ONLY.** Caller **35-38,65-115** keeps integer
`time.clock_gettime_ns(time.CLOCK_MONOTONIC)`, darwin clock6, same host/boot and
the exact identity. `deadline = start +120*NS` remains at **650-653**. The
conservative `common_deadline` samples local time first, shared time later,
deducts1ms and rounds down; it passes local seconds to unchanged common. It
does not add an allowance, patch clocks or invent an offset. These functions
are AST-exact to R3.

Caller **307-354** retains the exact seven completed source-file declarations,
saved audit `f7f9b10fc99cc2196d0fd88a5b93a35a39a9adc5`, and five remote raw-file
checks. The parent's original seven-record inspection, actual helper0/SSH0,
EOF/error/deadline and guard/root/payload bindings must precede NEW inspected_ns
and phase start. The two local records remain parent duties. Old relative
times stay in their original process domains; only their original within-process
duration/order is checked. Qualification is not repeated. Manifest remains
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26`;
facts remain `ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`.
Hashes and a parent statement cannot independently establish the causal act.

COPY **357-394,686-691** keeps helper actual0, reap, both EOF, no uncertainty,
exact clock/order,10s including reap and parent inspection BEFORE private entry.
The caller's later check cannot prove that past gate. SYSTEM clock availability
evidence supplies no new cross-process/private-clock/conversion measurement.

**Original R2, CLOSED_SOURCE_ONLY.** Caller **536-545** stores the active label
and complete bounded collector-status copy and latches first failed COMMAND
before any publication. **547-563** separates attempted paths, partial/unverified
states, durable refs and nulls. A raw write/fsync/close error cannot create a
durable ref. The comparator starts only at **564**, after durable raw/status.
A failed comparator is assigned before its own publication at **585-590**.
**719-742** keeps a failed COMMAND first, then assigned failed comparison;
later writer errors stay in separate fields/stop detail. A source close error
does not replace the retained command cause. `direct_children` reads retained
returncodes without another poll/wait; unknown exit/status stays null. Deadline
loss may prevent publication of these in-memory facts; parent keeps actual
outer partial streams and exit. No source finding reopens R1/R2.

## Closed startup scope

`startup`, **245-249**, and native environment, **271-274**, are byte/AST-exact
to R3. The original ten parent values equal next-operation and startup-contract
data. `normalize_startup`, **252-268**, first requires every original key/value.
SOURCE then requires3.9.6 and isolated/no_site/B/optimize0 and the exact union
of those ten names with all six observed extras. A changed required value,
missing key, missing observed extra or any seventh extra rejects. It deletes
the six names locally without fetching their values, then requires the exact
original effective map. COPY permits no extra and deletes nothing. Its exact
private interpreter, sibling path and `-B` pre-import checks remain at **384-394**.

The public `a26871a` observation reports all ten input values equal, no missing
key and precisely CPATH, LC_CTYPE, LIBRARY_PATH, MANPATH, SDKROOT and
__CF_USER_TEXT_ENCODING. Only LC_CTYPE=C.UTF-8 was queried; the other five
values and all origins remain unqualified. Deletion cannot undo interpreter,
launcher or import side effects before script entry. R5 says this explicitly;
this review gives those side effects no trust qualification. No private3.13
COPY startup observation exists here. Its strict admission can still fail.
There is no arbitrary-extra/value exception, new environment/flag, extra phase
or SYSTEM-derived COPY exception. Normalization occurs at **664**, inside the
original first-error catch and before pins, without renewing the phase.

## Preserved controls and static evidence

- SOURCE still has exactly53 caller commands: SYSTEM `entry-template`, exact
  private `-I -S -B --version`, fixed Python lipo, then50 vectors. It never
  imports common. COPY still has52 caller commands: SYSTEM copy-file checks
  before and after50. Its separate parent SYSTEM entry authentication also
  consumes COPY's original phase. Prefix-before-EVERY-private-entry remains
  a parent duty. `perform` call expressions equal the R3 AST.
- COPY keeps17 input checks before common import, prepared canonical bundle
  ownership/mode/filesystem/source checks, fresh SOURCE v2 records and saved
  basis comparisons. All50 vectors independently equal seven objects by seven
  verbs then deep verify, including direct Autoupdate and daemon objects and
  exact arm64/strict/all-architecture flags. No arbitrary argv, command repeat,
  product invocation, helper fallback or completion phase was added.
- Collector **408-528** is unchanged:10s including0.5s kill/reap, ending at most
  phase-end minus1s; combined1MiB retained DURING reads plus one detection byte;
  fixed cwd/environment, DEVNULL stdin, shell=False, closed fds, retained handles,
  actual exit/reap, both EOF, error, drop/overflow/timeout/kill and close facts.
  There is one Popen site and no later drain. All admission/authentication,
  collection, comparison and publication consume the same120s. Parent must
  observe actual caller exit and complete EOF before that same end; a result
  file is pre-exit evidence. Hard OS preemption/containment remains unproved.
- Only `-verify` sorts strict UTF-8 complete splitlines per stream, retaining
  duplicate and empty lines. Other output stays exact after the declared bundle
  substitution. COPY requires both fresh SOURCE and saved basis. Nulls, first
  causes, partial references and failed observations remain distinct.
- SOURCE sums owned0600 regular initial files and owned0700 directories before
  any write, initializes no ledger and keeps8MiB/file,32MiB total,16MiB product,
  4MiB cleanup and64KiB accounting. COPY's branch is AST-exact and calls only
  existing `common.write_bytes(..., critical=False)` with regular pre-write
  reservations for every new guard, parent envelope/stream, raw/status/comparator
  and result. There is no second writer, reserve borrowing or ledger init/repair.
- All16 candidate SHA256SUMS entries match; all12 JSON files parse. The six-hunk
  zero-context patch equals independent unified-diff generation and reconstructs
  all45,062 caller bytes from exact old bytes in memory. It was never applied to
  a source file. Python3.9 AST comparison finds only changed `Writer._put`,
  `emit_summary`, `main`; added `normalize_startup`; the new extra-key constant;
  and the module entry change. `main` otherwise equals R3. All29 other functions
  and all other original module nodes agree. [function-bindings.json](function-bindings.json)
  gives exact whole-function locations, lengths and hashes.
- All17 pins match tracked Git length/hash; all ten module destinations/bytes
  match materialization, source-checks and exact controller candidate
  `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`. Pins, vectors, schema and both null
  templates remain byte-exact to R3. Canonical SDK data differs from preparation
  only in ten header paths. Existing helper/SDK/controller source reviews keep
  their original bounded scope. All81 prior ref/path/blob/length/hash claims in
  R5 binding arrays match Git and worktree. This is not a fresh logic review
  of those ten unchanged modules.
- Saved basis bridge `ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13` remains bounded
  data support for the entire50-record commands projection, canonical hash
  `b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`.
  Raw wrapper `1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`
  and canonical full wrapper `7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159`
  remain distinct. Public refs/hashes were checked; no private outputs or audit
  were rerun. The complete artifact receipt contract remains external.

## Byte bindings and proof limits

```text
R5 caller       ba757e7a03d890f3b3ad5e9b72c1580bbc4839543a746f81516bc962983de650
R5 exact patch  bdf63037467f9014b31150bdcf72bd4b48b06a2bcfad0dc16e6f06b3c7cbd783
R3 caller       464ab85baebf47e3b373463768e9abb2001d39dd181b4e2a790e1868d2b90597
original caller 26bcd57988734f54b2baa14a2b54886ab9ee09c2db1e342a137a49d23c42e18b
17-pin data     31a17c129e0af6c8c0437458c01fa8c28cffb89f29ee327648a31fed96608352
50-vector data  e3ff5756ab3b35a5c815e50ac1fad6520117836e080849260de87592bd327fa0
helper          bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428
common          20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c
controller      08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4
```

[bindings.json](bindings.json), [static-checks.json](static-checks.json),
[findings.json](findings.json) and [summary-size-check.json](summary-size-check.json)
retain the candidate/import/tree/blob/length/hash bindings, check results,
prior dispositions and exact size reconstruction. The local SDK documents
support general I/O reasoning only. Target pipe capacity, successful complete
summary delivery, clocks/conversion/rates, actual private startup, exceptional
OS behavior, prefix pre-entry order, causal inspection and actual outer
handle/exit/EOF/deadline observations remain UNVERIFIED or parent duties.

Only independent static file/Git/hash/patch/JSON/AST-data readers ran. No supplied
source or AST was executed, evaluated, imported or compiled to executable code.
No runtime, tests, unit tests, mocks, failure injection, native command/probe,
process/port query, remote/network action, private state read/write, private
interpreter, signing, credential, child, preparation, controller, product or
build operation ran. SOURCE50/private-version/prepare/COPY50/controller/product
execution counts are zero. No TypeScript changed. The review/bindings-only
commit disables hooks and signing. The worktree's candidate bytes remain exact.

Historical states remain: strict SDK FAILED_UNVERIFIED, helper1/SSH1; older
M1 app25/25, app0/controller1 FAILED_NOT_ACCEPTED; first strict post65431
EADDRINUSE cause UNKNOWN; whole-flow NI15 UNVERIFIED. Earlier23/25, all failed
records and guards, original second post-bind, bundle-after, outcome/archive
gaps, stripping warning and unknown full outer build duration/exit remain.
Completed source-file pass and bounded saved-data audits keep their original
scope. This review supplies no independent native qualification, artifact gate,
flow acceptance, performance, release, UI or integration result.
