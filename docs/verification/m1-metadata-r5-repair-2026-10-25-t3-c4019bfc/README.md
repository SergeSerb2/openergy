# Inert M1 metadata R5 implementation

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
IMPLEMENTED_UNREVIEWED. Independent R6 review required before actual use.
Runtime and whole-flow NI15 UNVERIFIED.** This is an implementation handoff,
not an independent review or acceptance result. No new human permission gate.

Only this new folder is owned. Parent is
`a26871ad9fdae2d55ff53544aed26ae0cdf2620e`. The immediate frozen candidate is
`e4ec20bdc6058efaa2fdb272ca88b84a7519116c`, imported as `0c95b8a`:
`../m1-metadata-clock-failure-repair-2026-10-25-t3-d1e829a9/metadata-caller.py.txt`,
SHA-256 `464ab85baebf47e3b373463768e9abb2001d39dd181b4e2a790e1868d2b90597`.
The original caller remains exact at
`621e0e3a9037e15b391e05b964dfcdb25da01515`, SHA-256
`26bcd57988734f54b2baa14a2b54886ab9ee09c2db1e342a137a49d23c42e18b`.

[metadata-caller.py.txt](metadata-caller.py.txt) is the complete inert R5 source.
[caller-e4ec20b-to-r5.patch](caller-e4ec20b-to-r5.patch) is the exact unified
delta against the immediate frozen caller. Its zero context avoids whitespace
inside patch data; use only exact old bytes, never fuzz. Static reconstruction
produces the full R5 bytes. Do not patch the historical repository file.
[changed-functions.md](changed-functions.md) gives every changed or added
function in full, with its scope, line numbers and hash, plus the full changed
module entry block. [bindings.json](bindings.json) and [SHA256SUMS](SHA256SUMS)
bind the source, patch, data and retained public inputs. All files remain inert
mode100644; no execution bit or activation is supplied.

The full frozen caller, original caller, both full interface READMEs, R1/R2
review, R4 report, schemas, templates, pins and plans were read. The fixed
preparation, next-operation, metadata and canonical SDK recipes, file helper,
operator plan, receipt contract and common writer contract were also read.
The R4 report is [retained here](../m1-metadata-clock-failure-review-2026-10-25-t3-e92ea4b9/REVIEW.md),
original `aec21c2cb235c1da74eb2d7985c534135670552d`, imported as `5c8602a`.
Its original R1 shared-clock and R2 early-failure findings remain
**CLOSED_SOURCE_ONLY**, as reported there. R3 receives this unreviewed repair;
this folder does not clear itself. The parallel transport candidate is outside
this change and supplies no prerequisite or acceptance here.

## R3 caller-owned publication

`Writer._put` keeps exclusive `O_WRONLY|O_CREAT|O_EXCL|O_NOFOLLOW`, mode0600,
the existing file/aggregate caps, pre-write initial accounting and fsync rules.
SOURCE now passes the bytes directly to one `os.write`. It rejects every short
return, including zero, and does not retry, refund, remove or repair a partial
file. An empty record still needs the exact zero return and successful fsync.
There is no `fdopen`, Python output buffer or explicit/implicit flush.
The existing file and directory sync checks still follow a successful write.
Only a full successful return provides a durable bytes/hash reference.

The raw descriptor is closed once in `finally`, including after an alarm,
write failure, short return or fsync failure. Close cannot flush Python payload.
A close failure with no earlier failure propagates. With an earlier failure,
the earlier exception propagates and the close error is retained separately in
`publication_errors`. Close is never retried, including when its result is
uncertain. No cleanup budget is added. A close or fsync can still suffer an OS
stall; source inspection is not a hard real-time guarantee.

`emit_summary` retains the existing1MiB encoded cap and shared deadline. It
sets fd1 nonblocking, rechecks remaining time and makes at most one raw
`os.write(1, data)` call. EAGAIN, a short return, an alarm or any other exception
fails the summary and the caller. It neither loops, waits for writability,
drains, completes a tail, switches transport nor obtains another deadline.
It leaves fd1 nonblocking and places no payload in `sys.stdout.buffer`.
One raw call does not promise complete output or atomicity for a large summary.
Kernel-accepted bytes can be observed later; parent still needs complete EOF
and the actual caller exit within the same phase. A partial summary is partial
evidence and cannot accept a run, even if a result file exists.

The module entry block calls `os._exit` with main's actual0/1 result. An uncaught
pre-admission exception maps to1 without a traceback writer. This bypasses
Python finalization, buffered standard-stream flushes and object destructors.
No finalizer can issue a second summary or late traceback. No exit is inferred:
parent must retain and observe the real OS status. Invalid pre-admission input
can leave no summary; missing facts stay missing. In-phase normalization
failures still use main's existing closed first-error/result/summary path.

COPY's common writer is byte-exact and outside this publication repair.
Every COPY new guard, parent envelope/stream, raw/status/comparison and result
still uses the existing common regular reservation with `critical=False`.
No ledger initialization, reserve borrowing, second writer or common cleanup
change exists. The explicit conservative `common_deadline` conversion is exact.

## Closed, mode-specific bootstrap

The [public startup observation](../m1-system-startup-prerequisite-2026-10-25/README.md)
at parent `a26871a` reports all ten supplied values equal and none missing for
the exact `/usr/bin/python3 -I -S -B` SYSTEM3.9.6 launch. It reports precisely
six extra key names: `CPATH`, `LC_CTYPE`, `LIBRARY_PATH`, `MANPATH`, `SDKROOT`,
`__CF_USER_TEXT_ENCODING`. Only `LC_CTYPE=C.UTF-8` was queried. The other five
values were not queried; the source of each addition was not traced. Those
limits remain. This task read public tracked data only and ran no new probe.

[startup-contract.json](startup-contract.json) records the executable contract.
`startup(root)` and the parent's ten-key input map stay byte-exact. At the
original in-phase environment-check site, before pin reads or metadata work,
`normalize_startup` does this:

1. Require every one of the ten keys present with its exact original value.
2. For SOURCE require Python3.9.6, isolated1/no_site1/B1/optimize0, and the
   **exact union** of those ten keys and the six observed extra names. All six
   must be present. Any other key, missing key or changed required value stops.
3. Delete those six names only from this caller's own `os.environ`. Do not
   read, log, hash, trust or export their values. Then require the exact
   original ten-key effective environment again.
4. For COPY require the exact original ten-key set and values from the start.
   It deletes nothing. A private startup with any extra key fails closed.

The exact six-name SOURCE set is deliberately narrower than a subset whitelist.
A different SYSTEM version or bootstrap key set needs separate evidence and a
new explicit source contract. Values are discarded because the metadata work
requires the original controlled environment, not startup additions. Their
presence is evidence about names only, with no value or origin authority.
Deleting environment entries does not undo interpreter/launcher activity that
already occurred before script entry; that startup trust remains a parent
launcher boundary. The fixed native/helper child environment is still a newly
constructed `native_environment(root)` passed explicitly to Popen. Discarded
values cannot flow into its argv or environment.

The SYSTEM observation does **not** qualify private3.13 startup. COPY retains
its separate exact interpreter/`-B` flags, sibling-path and pre-import gates.
Actual private startup is unmeasured here and can still reject. Parent still
owns full prefix authentication and inspection BEFORE every private entry.
No launch flag, parent input value, wrapper argument or global environment
setting changes. Source normalization stays within the original phase and
inside the original first-error catch; it grants no extra work time.

## Preserved clocks, evidence and sequence

The only NEW absolute clock remains integer
`time.clock_gettime_ns(time.CLOCK_MONOTONIC)`, darwin clock6, same host/boot,
with the exact `openergy.shared-clock.v1` identity. No old relative timestamp
is retagged, offset or compared with new absolute values. Parent starts one
SOURCE120s phase after its completed-record causal inspection, and one COPY120s
phase before entry authentication. No import, write, command or cleanup resets
either phase. Command10s includes0.5s kill/reap and ends at most phase-end
minus1s. Shared/local conversion,1ms conservative deduction, rounding,
alarms, postchecks and parent actual exit/EOF duties stay exact.

Completed source-file evidence remains the exact seven-file manifest
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26`,
facts `ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`,
saved audit `f7f9b10fc99cc2196d0fd88a5b93a35a39a9adc5` (import `d79ed01`).
Parent must inspect all seven original records, actual helper0/SSH0, raw
streams, EOF/errors/deadline and guard/root/payload bindings BEFORE the new
inspection reading, then phase start. The caller still checks the five fixed
remote files and their own old within-process facts. Hashes do not substitute
for the parent's causal act. No source-file qualification is repeated.

SOURCE still performs SYSTEM `entry-template`, then exact private
`-I -S -B --version`, fixed Python lipo, then exactly50 ordered native vectors.
SOURCE never imports common. COPY retains17 pins before common import,
prefix-before-private-entry as parent duty, fresh SOURCE v2 consumption and
SYSTEM copy-file checks BEFORE and AFTER its50 within the same phase.

The early command-failure latch remains before any raw/status write or fsync.
Payload comparison still follows durable raw/status only. First command
failure, assigned comparator failure, publication errors, partial/missing raw
references, all retained handles and actual observed statuses remain distinct.
Null stays null. Collector combined1MiB cap during reads, actual exit/reap,
both EOFs, overflow/drop uncertainty, order and comparison rules stay exact.
Only `-verify` sorts complete strict-UTF-8 splitlines, retaining duplicates
and empty lines; all other text is exact after the declared bundle substitution.

Pin data `31a17c129e0af6c8c0437458c01fa8c28cffb89f29ee327648a31fed96608352`,
vectors `e3ff5756ab3b35a5c815e50ac1fad6520117836e080849260de87592bd327fa0`,
all ten module bytes, parent schema and both null templates remain exact.
Schema/templates need no interface change because bootstrap is caller-local.
All quotas stay8MiB/file,32MiB aggregate,16MiB product,4MiB cleanup and64KiB
accounting. SOURCE initializes no ledger. COPY uses only the existing ledger.

[retained-evidence-bindings.json](retained-evidence-bindings.json) keeps the
completed-file declarations, artifact/source assessment and public basis bridge
as bounded prior support. Raw basis hash stays
`1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`.
Bridge `ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13` (import `15e9f45`) supports
the entire50-record commands projection only, canonical SHA-256
`b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`.
It grants no new caller, transport, artifact or runtime acceptance. No private
basis, source-file, artifact or historical payload was reopened.

## Static checks and remaining limits

[static-checks.json](static-checks.json) records AST with Python3.9 grammar,
JSON, tracked Git length/hash, exact patch reconstruction, whole-function
comparison, independent50-vector expansion and scope checks. Supplied source
and AST were never executed, evaluated, imported or compiled. These are static
data checks, with no unit tests, mocks, failure injection or runtime experiment.
The changed function list is exactly `Writer._put`, `emit_summary`, `main`,
plus added `normalize_startup`; the constant and module entry block also change.

[finite-plan.json](finite-plan.json) remains execution=false, with null fresh
results and null independent R6 review. Parent will commission that review;
no reviewer or child agent was started here. The one later exact real SOURCE
and COPY attempt remains a parent task after independent review and its existing
gates. This implementation supplies no artifact receipt, prepare, controller,
product, release or integration execution.

Only static file/Git/AST/JSON/hash/patch work and the owned-folder commit ran.
No supplied/proposed caller or helper, private state, remote/network access,
native command/probe, process/port query, private interpreter, tests/builds,
credentials, signing, activation or children were used. Hooks and signing are
disabled for the commit. No TypeScript changed. Static checks cannot prove
deadline timing, cross-process clock equivalence, conversion accuracy,
exceptional OS behavior, complete nonblocking summary delivery or actual private
startup. Parent actual handles/exits/EOF, causal inspection and pre-entry
authentication remain external duties.

All historical FAIL/UNKNOWN remain: strict SDK attempt FAILED_UNVERIFIED,
helper1/SSH1; older M1 app25/25, app0/controller1 FAILED_NOT_ACCEPTED; first
strict post65431 EADDRINUSE cause UNKNOWN; whole-flow NI15 UNVERIFIED. Earlier
23/25, all failed records and guards, original second post-bind, bundle-after,
outcome/archive gaps, stripping warning and unknown complete outer build
duration/exit remain. No new acceptance follows from this R5 implementation.
