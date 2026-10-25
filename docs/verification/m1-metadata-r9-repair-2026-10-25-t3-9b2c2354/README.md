# Inert M1 metadata R9 repair

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
IMPLEMENTED_UNREVIEWED. Independent R10 review is next.** This implementation
does not close its own finding or grant execution, runtime or approval authority.
It adds no human permission gate. All existing files stay unchanged.

Parent is `81252214f7c391c465510193acf48115ec3d95db`. The frozen caller is
`13fccbe1d547198d5025748d20af5fb5c3a8c287`, imported at
`0dad1c00d0a27f8266ed8e138f780a9d94656cdc`, in the
[R7 folder](../m1-metadata-r7-repair-2026-10-25-t3-5ce08973/README.md).
Its caller SHA-256 is
`c9a3b1bcfae0da2eced035c2c9e4048a92dc18340c940148a486cf12348b0987`.
The full [R8 report](../m1-metadata-r8-review-2026-10-25-t3-28dba3a1/REVIEW.md)
is `8326f49945824c71e1b58d421f107a3cfafff3da`, import `51533bd`.
The [retained original brief](../m1-metadata-r8-review-2026-10-25-t3-28dba3a1/BRIEF.md),
full R8 report and full R7 caller/contracts were read as static data.

R8 has one open P2, R8-P2-1. At R7 lines472-477, the settle poll can return
`None` after the command deadline. Code then attempts KILL before reading the
clock again. An alarm or later deadline rejection cannot undo that signal.
R8 closed R6-P2-1 in source for the compact success receipt. Original R1/R2
and caller-owned R4 source closures remain within their original scopes.

The full inert [caller](metadata-caller.py.txt) is46,908 bytes,789 lines,
SHA-256 `01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`.
The exact zero-context [patch](caller-13fccbe-to-r9.patch) is1,504 bytes,
three hunks, SHA-256
`5d4c7357d0bdd6d5702788605a65a4fc7cb17108a365ad2d9398feeccc8ec736`.
Static patch reconstruction yields every new caller byte. These are inert
mode100644 files. Do not apply the patch to historical files or execute it here.

Only `collect` changes. No function is added or removed. The complete changed
function is in [changed-functions.md](changed-functions.md). All33 old/new
function ranges, lengths and hashes are regenerated in
[function-changes.json](function-changes.json). The other32 functions, Writer
class, module constants/imports and `os._exit` entry remain byte/AST-exact.
Reversing only the three explicit collector edits restores the full R7 caller.

At new lines474-487, the retained direct handle is polled once. The code saves
its actual result and immediately samples the same shared clock, including for
`None`. KILL requires a `None` result and a sample strictly before both the
unchanged command deadline and phase end. A late `None` latches `timed_out`
and `deadline_loss` and skips KILL. The original
`proc.wait(timeout=remaining(deadline))` remains at488. No expired allowance
is extended. Its fresh remaining-time check can reject before the wait call.

A non-null poll result is a real reap observation. The small `finally` around
the clock read preserves `reaped=True` even if that read raises. The existing
final `actual_exit` still comes from `proc.returncode`. A clock exception
remains a failed settle observation. A null poll result remains null; a later
actual wait can separately supply the final exit. No exit is inferred.

The only new record facts are `kill_eligibility_exit` and
`kill_eligibility_observed_ns`. Their exact meaning, null cases and parent
coupling are in [repair-contract.md](repair-contract.md). They are retained
in the existing OS/status file, command snapshots, first-command failure and
direct-child status through the unchanged paths. No new file, writer, ledger
operation or compact receipt field is added. Parent must bind the exact R9
status construction and independent R10 result before later use.

The immediate post-poll sample gates a new signal attempt. It does not provide
hard preemption. Scheduling can resume late after a sample, and a later OS
call can stall. Alarms, original postchecks, bounded actual wait and retained
failures remain. No timing scenario, signal, process or mock was run here.

The following contracts remain exact:

- The one120s shared phase uses integer `clock_gettime_ns(CLOCK_MONOTONIC)`,
  darwin6, same host and boot. It includes admission, authentication, reads,
  collection, KILL/reap, comparisons, publication and parent's actual caller
  exit/EOF before the same end. Commands retain10s including0.5s for reap,
  with the original phase-minus1s command bound. No second time allowance.
- SOURCE retains SYSTEM entry-template before private `-I -S -B --version`,
  Python lipo and50 vectors:53 commands. COPY retains parent full-prefix
  authentication before private entry,17 pins before common import, file
  checks before/after50:52 commands. All10 modules and50 ordered vectors match.
- SOURCE retains exclusive0600 initial writes with no ledger. COPY retains
  unchanged common regular pre-write reservations, `critical=False` for every
  new write, with no ledger initialization or repair and no second writer.
  Caps remain1MiB combined while-read,8MiB/file,32MiB total,16MiB product,
  4MiB cleanup and64KiB accounting; initial allowance12,517,376 bytes.
- `perform` still latches the first failed COMMAND before writes. Comparators
  start only after durable raw/status. Actual exit/null, EOF, dropped/unknown
  bytes, first causes, close/publication errors and attempted/partial refs stay.
  No drain, retry, group signal, PID lookup or new target exists.
- SYSTEM3.9.6 retains its exact six-key normalization and original ten-key
  effective map. COPY has no SYSTEM-derived exception. Startup, CLI, pins,
  parent schema and null templates remain byte-exact and non-admitting.
- R7 success receipt remains<=512B, actual FIFO and PC_PIPE_BUF>=512 checks,
  the exact durable full-result reference, original single full-result save,
  one nonblocking raw stdout attempt and `os._exit`. All statuses/raw/comparisons
  remain in the full result. Failed full fallback can stay partial and failed.
  No delivery guarantee or target pipe measurement follows from these bytes.

Twelve copied contract/data files remain byte-exact to R7. Their historical
R7/R8 review-state text is retained as provenance. This README,
`repair-contract.md` and the regenerated false plan state the current R9/R10
status. The receipt schema, byte proof and maximum-shape data do not change.
The new status fields consume existing full-result capacity; they grant no
extra bytes. The unchanged serialization/file caps still apply.

[bindings.json](bindings.json) binds original/import/current R7 and R8 files,
the new caller/patch, all retained inputs, pins/modules and144 rechecked
historical public binding claims. Thirty R7/R8 files agree across original
Git, import and current bytes/blobs/mode100644. Both old SHA256SUMS files match.
Recipe expansion independently reproduces50 vectors. Static JSON serialization
reproduces SOURCE377B/COPY373B maximum receipt shapes. These are data shapes,
not observed receipts. [static-checks.json](static-checks.json) and
[static-ast-observations.json](static-ast-observations.json) record the checks.
[SHA256SUMS](SHA256SUMS) covers every other new file.

[finite-plan.json](finite-plan.json) remains execution=false, with all execution
counts zero, no fresh evidence/results and no R10 verdict. Parent commissions
R10 next; no child or reviewer was started here. Transport `1c46c16` is SOURCE
CLEAR only under `a7e22725`, imported at parent8125221. Runtime/D4/payload
integration remains open. Payload `f78f2be0` is awaiting independent review.
The local prepared packet contains OLD R7 bytes, remains disabled and was not
opened or modified by this work. No payload or transport repair is included.

Only independent static file/Git/AST/JSON/hash/patch readers and the owned-folder
commit ran. No supplied source/check or AST was executed, evaluated, imported
or compiled to executable code. No tests/unit tests/mocks, private access,
remote/network/native calls, process/port query, private Python, caller,
source qualification, prepare/controller/product, build/signing, credentials,
activation or child agent ran. No TypeScript changed. End-to-end execution is
outside this explicit SOURCE/DATA scope. Commit hooks and signing are disabled.

No timing, hard OS-stall bound, clock equivalence/conversion/rate, private
startup, actual pipe delivery, durability under faults, parent integration,
native metadata or whole-flow acceptance is proven. Parent still must retain
actual caller/transport exits and complete raw/EOF facts before the same end;
the full artifact/operator/receipt contracts remain external duties.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause UNKNOWN;
whole-flow NI15/runtime UNVERIFIED; earlier23/25 FAILED; all failed files/guards;
missing original second post-bind, bundle-after, outcome/archive; stripping
warning; unknown full outer build duration/exit; and the closed updater.
