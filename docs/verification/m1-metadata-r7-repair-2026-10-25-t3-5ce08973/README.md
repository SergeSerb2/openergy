# Inert M1 metadata R7 implementation

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
IMPLEMENTED_UNREVIEWED. Independent R8 review is next. Runtime and whole-flow
NI15 UNVERIFIED.** No runtime approval or new human permission gate.

Only this NEW folder is owned. Parent is
`cd97ba975867a571ce1cd2070083f07865d05eec`. Frozen R5 is
`2667d808fb7807ba6b2591c4141938f1cefabd90`, imported as
`2482f09a8a598e0ad86154f47d82426c1adffe72`, folder
`../m1-metadata-r5-repair-2026-10-25-t3-c4019bfc/`. Its caller is45,062 bytes,
SHA-256 `ba757e7a03d890f3b3ad5e9b72c1580bbc4839543a746f81516bc962983de650`.
All17 historical folder files remain exact to frozen Git/import/worktree bytes.

The full [R6 review](../m1-metadata-r6-review-2026-10-25-t3-9e73b3ec/REVIEW.md),
original `df028a8d03ed3aa657aa18c8c170ba4521df3769`, import `75051a1`,
has one P2, R6-P2-1, and no P1. Its deliberately incomplete successful result
is209,116 bytes. R5 attempts the whole result in one nonblocking stdout write;
an ordinary short write can therefore fail the caller after all53 commands and
durable result publication succeed. Increasing the reader cap does not make
that write fit the pipe's immediate allowance. This repair addresses that
bounded issue only. It does not independently close its own review finding.

[metadata-caller.py.txt](metadata-caller.py.txt) is the full inert R7 caller:
46,104 bytes,776 lines, SHA-256
`c9a3b1bcfae0da2eced035c2c9e4048a92dc18340c940148a486cf12348b0987`.
[caller-2667d80-to-r7.patch](caller-2667d80-to-r7.patch) is the exact five-hunk,
zero-context delta:1,712 bytes, SHA-256
`2001349fe188bf0a8c83fd78f617217ce1cc1dd579e477ec926a2d244c6e2566`.
Static reconstruction from exact old bytes produces the complete caller.
Do not patch historical files or execute these repository sources. All new
files are inert mode100644. Hooks and signing are disabled for the commit.

Only `emit_summary` and `main` change. No function is added or removed.
[changed-functions.md](changed-functions.md) includes both complete functions;
[function-changes.json](function-changes.json) gives old/new hashes, lengths
and locations for all33 functions. `main` has exactly three small ref changes:
initialize `result_ref=None`, capture the existing result save's return, and
pass it to the existing summary call. Reversing those three edits restores
the full old main byte for byte. The other31 functions and all other module
nodes agree, including Writer, clock conversion, collector, first-failure
latches, comparator, fresh-source consumption and the `os._exit` entry block.

The full v2 result, raw streams, actual statuses, comparisons and first errors
remain in their original files with their original construction. There is
still exactly one full-result save through the existing Writer. Success stdout
now carries only a closed `openergy.metadata-success.v1` receipt with mode,
explicit original phase start/end and that exact durable path/bytes/SHA return.
No full clock dictionary is copied into the receipt. Parent must verify it in
the separately bound full result and unchanged parent v2 record. A missing ref
or failed publication cannot take the success path. No receipt file or writer
is added. Failed/partial stdout remains the original failed full-result fallback
under the original1MiB ceiling; it can remain incomplete and cannot accept.

[compact-success-contract.md](compact-success-contract.md) states the complete
new interface and mandatory parent checks.
[compact-success.schema.json](compact-success.schema.json) closes both object
key sets. Existing parent/source v2 schema and null templates are byte-exact;
they are not weakened. The new success schema changes stdout only. Fixed CLI
flags, launch/environment values and original schemas do not change.

`emit_summary` uses the existing canonical sorted compact ASCII JSON encoding
plus one final newline. It rejects success bytes above512, requires fd1 to be
a pipe/FIFO, and checks the actual `fpathconf(1, 'PC_PIPE_BUF')` integer >=512.
It then makes one nonblocking `os.write`, requires an exact byte count, and
keeps the original postchecks. All this consumes the same phase deadline.
No supplied source was run to query a pipe here. No new time, loop, writable
wait, tail retry, drain, buffer, console retry or deferred shutdown payload exists.

For a conforming pipe with enough free space, a write within PIPE_BUF is atomic.
An empty conforming pipe can accept this record; a full or insufficiently free
pipe can return EAGAIN. Interrupts, errors and deadline loss still fail. This
does not claim universal successful delivery or measured target pipe capacity.
The exact-count guard also rejects any unexpected short return. R6's retained
static write/pipe documentation supplies the general I/O basis; those SDK
documents were not newly measured or used as target observations here.

[byte-bound-proof.json](byte-bound-proof.json) retains the independent static
JSON byte arithmetic. The exact96-byte long root from the209,116-byte report
still fits, using a20-digit start, conservative21-digit end,7-digit8MiB result
length and64 hex digest characters. SOURCE is at most377 bytes; COPY373,
including newline. The two `success-*-max-shape.json` files are inert maximum
shape data, with synthetic maximal values and digest characters, not observed
receipts. For escaped root content length E, SOURCE <=281+E and COPY <=277+E.
Actual encoding enforces512 even for longer or escaped suffixes; such a root
can fail stdout without changing the earlier root-admission contract.

Parent MUST validate strict compact schema/canonical raw bytes, mode and phase,
exact result path/length/hash, full v2 result and ALL53 SOURCE or52 COPY caller
records, every raw/status/comparison and actual direct-child fact. COPY also
needs its separate SYSTEM parent authentication in the original phase before
private entry. Parent MUST retain actual caller0/reap, complete stdout/stderr
EOF and every error/loss/kill/deadline predicate before the same phase end.
A compact receipt alone grants no metadata, artifact or runtime acceptance.
The full unchanged artifact receipt contract and all later gates still apply.

Preserved controls include the following:

- One120s shared phase includes all authentication, reads/import, commands,
  comparisons, publication and actual caller exit/EOF. Commands keep10s with
  0.5s kill/reap inside it, at most phase-end minus1s. Clock remains integer
  `clock_gettime_ns(CLOCK_MONOTONIC)`, darwin6, same host/boot. No new clock
  conversion, fallback, rate claim or old-time retagging is introduced.
- SOURCE still runs SYSTEM entry-template before private `-I -S -B --version`,
  fixed lipo and50 native vectors. COPY keeps file checks before/after50,
  exact17 pins before common import and both saved-basis/fresh-SOURCE comparison.
  Prefix-before-EVERY-private-entry remains a parent duty.
- Full command status is latched before any raw publication. Comparators run
  after durable raw/status only. First COMMAND/comparator failure, close/write
  errors, complete actual statuses, EOF/nulls and partial refs stay distinct.
- SOURCE keeps exclusive0600 initial writes and no ledger. COPY keeps unchanged
  common regular pre-write reservations with `critical=False` for every new
  file, no ledger init/repair and no second writer. Caps stay1MiB combined
  during command reads,8MiB/file,32MiB aggregate,16MiB product,4MiB cleanup and
  64KiB accounting. Existing initial regular allowance remains12,517,376 bytes.
- R5's exact SYSTEM3.9.6 six-extra-key bootstrap normalization and strict COPY
  ten-key environment remain byte-exact. No COPY exception, flag/env change or
  pre-entry startup trust qualification is added. Raw SOURCE write/close and
  `os._exit` safety stay exact. Original R1/R2 and caller-owned R4-R3 remain
  CLOSED_SOURCE_ONLY within their original review limits.

Parent has actually inspected the seven original completed source records at
`492f067`, within its retained saved-data scope. Their manifest stays
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26` and facts
`ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`.
Later remote five-file verification, causal inspected/start readings and
parent raw/exit/EOF/deadline duties remain. No qualification repeat or private
original read occurred here. The raw basis and bounded bridge keep their
original scope; hashes do not replace causal inspection or full raw checks.

Payload `fa72a0d` (import `dde69c7`) stays an opaque1MiB caller collector with
a separate16KiB outer receipt. It does not parse caller JSON. Parent will bind
the new interface after independent review. Transport `88db0b5` remains held
for two separate narrow fixes. No payload/transport edit or clearance occurs.

[bindings.json](bindings.json) retains exact parent/frozen/import/review
Git/blob/length/hash identities, all17 pins, ten modules, seven unchanged data
files, the81 inherited public binding checks and the prior documentation
declarations. [static-checks.json](static-checks.json) records static outcomes.
The50 vectors independently equal the recipe expansion. All JSON data are
strictly parsed. [SHA256SUMS](SHA256SUMS) binds every other new file.
[finite-plan.json](finite-plan.json) remains execution=false with zero execution
counts, null fresh results and null independent R8. Parent commissions R8 next;
no child or reviewer was started by this task.

Only independent static file/Git/AST/JSON/hash/patch readers and the owned-folder
commit ran. No supplied source or AST was executed, evaluated, imported or
compiled. No tests/unit tests/mocks, runtime/failure injection, remote/network,
native command/probe, process/port query, private state access, private Python,
helper/controller/product, prepare/build/signing, credentials or child agent
ran. No TypeScript changed. End-to-end execution is outside this explicit
SOURCE/DATA scope. Static checks do not prove timing, OS-stall behavior, clocks/
conversion/rates, private startup, actual pipe delivery or final integration.

All historical states remain: strict SDK FAILED_UNVERIFIED, helper1/SSH1; older
M1 app25/25, app0/controller1 FAILED_NOT_ACCEPTED; first strict post65431
EADDRINUSE cause UNKNOWN; whole-flow NI15 and runtime UNVERIFIED. Earlier23/25,
all failed records/guards, original second post-bind, bundle-after, outcome/
archive gaps, stripping warning and unknown full outer build duration/exit
remain. This implementation supplies no independent native, signature, ABI,
artifact, product, performance, release, UI or whole-flow acceptance.
