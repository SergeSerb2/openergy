# Independent M1 metadata R8 SOURCE/DATA review

2026-10-25, America/Vancouver. **HOLD: one bounded P2, R8-P2-1.
No P1 found. No overall SOURCE CLEAR. execution=false. Runtime and whole-flow
NI15 UNVERIFIED.** R6-P2-1 is CLOSED_SOURCE_ONLY. Original R1/R2 and the
caller-owned R4 buffered-publication closure remain CLOSED_SOURCE_ONLY.
This review has no runtime, execution, artifact or approval authority. It adds
no human permission gate.

Candidate is `13fccbe1d547198d5025748d20af5fb5c3a8c287`, reviewed at clean import
`0dad1c00d0a27f8266ed8e138f780a9d94656cdc`. Original candidate parent is
`cd97ba975867a571ce1cd2070083f07865d05eec`; import parent is
`afb68e54dc5474844ea46252f897c32fc6120e9b`. Their R7 folder trees and all 21
files agree exactly, including mode100644, Git blob, length and SHA-256.
This does not assert equality of the two whole-commit trees. Only this NEW
review folder is owned or changed. [BRIEF.md](BRIEF.md) retains the assignment.

All caller locations below refer to the complete
[R7 caller](../m1-metadata-r7-repair-2026-10-25-t3-5ce08973/metadata-caller.py.txt).
I read all 776 lines, the exact patch, README, full changed functions, compact
contract/schema, byte proof and maximum-shape data, CLI, startup contract,
parent schema, null templates, pins, vectors, plan, bindings and check claims.
I also read the original and R3/R5 READMEs, all three prior full caller reviews,
preparation README, next operation, metadata recipe, canonical SDK data,
file helper, full operator and receipt contracts, and common's writer contract.
Relevant separate payload/transport findings and public saved-data/prerequisite
reports were inspected as context. No private original, target tree or saved
raw basis was opened. Unchanged helper/controller logic was not newly reviewed.

## P2 R8-P2-1: Gate KILL after the eligibility poll returns

Primary location: **collect, lines472-477**, especially **473-475**.
Supporting locations: **until,84-94**; command/reap limits **410-413**;
subsequent failure checks **479-486,500-502**.

The final settle path enters `until(deadline)`, then calls `proc.poll()`.
When that actual call returns `None`, it sets `kill_attempted` and calls
`proc.kill()` without a new clock reading. The next `remaining(deadline)` is
at477, after KILL, to bound the wait. The late flags and normal-exit context
postcheck can reject the run, but cannot prevent a signal already attempted.

A concrete permitted ordering uses a command whose deadline D is well before
the outer phase end. At84, `until(D)` obtains a positive remaining duration,
for example20ms. Scheduling then delays the process50ms before91 arms the
relative timer. The outer phase timer has not expired. Line91 arms the stale
20ms duration after D. The eligibility poll now returns `None` promptly and
475 attempts KILL before the newly armed alarm fires, but after the command's
original10s end. Line477 then detects expiry. No uninterruptible OS operation,
new PID, product target or accepted run is needed for this source path.
An interval returning late from the poll has the same missing post-return gate.

This violates the fixed command ceiling that includes kill/reap. It is a
new operation after a call returns, distinct from inability to preempt an
operation already in progress. The alarms help interrupt many delays; their
relative setup and later postcheck do not supply the missing pre-signal check.
This is an inherited R5 function, byte-exact in R7, not a regression caused
by compact stdout. It is independently located in this caller. The separate
PAYLOAD-P2-1 and transport P2-02 describe the same class of gap in other code;
their pending repairs cannot change this caller's bytes.

Minimal repair: retain the eligibility poll's actual returned code and take
an immediate shared-clock reading after it, including `None`. Preserve a
real returned exit. If it returned `None`, issue KILL only while that reading
is strictly before the unchanged command deadline and phase end. A late
reading must retain failure/uncertainty and skip KILL. Keep the retained direct
handle, actual/null rules, original bounded wait,10s including0.5s reap,
fixed phase, first causes and caps. Add no PID lookup, group/other target,
retry, drain, second writer or new allowance. A subsequent operation can
still stall; this recommendation makes no hard real-time guarantee.

No timing scenario, signal, stall, mock or runtime experiment was executed.
R1's explicit shared domain/conversion, R2's early latch and R4's removed
deferred buffer remain closed in their original scopes. This new ordering
finding prevents overall deadline-path SOURCE CLEAR.

## R6-P2-1 response: Closed in source for the compact success interface

The original R6 review is `df028a8d03ed3aa657aa18c8c170ba4521df3769`, import
`75051a1cd825ed3e21c06cdfb7ad45a4e91965e3`. Its 209,116-byte reduced successful
shape remains exact canonical JSON, SHA-256
`b0b8fc94cf7b1c44fc31128aee173e192798cb552726d8909c422c1170b50c13`.
A1MiB reader cap does not enlarge an immediately writable pipe window.

R7 **670,751,762** introduces only null initialization, capture of the single
existing full-result save return, and passing that return to `emit_summary`.
Reversing those three text edits restores the entire R5 `main` byte for byte.
The v2 result construction, file name, all statuses, raw references, first
causes and comparison records stay exact. No new result or receipt file,
second Writer, reread, reconstructed reference or ledger operation is added.

`Writer.save` **235-242** serializes the full result once. `_put` **194-226**
returns `{path,bytes,sha256}` only after the existing write/close/sync path
and enclosing deadline postcheck. The additional enclosing save block
**747-754** also must complete. A save exception or that block's deadline
failure marks the sequence false before emission. A captured ref from a
save followed by a failed enclosing postcheck therefore cannot take success.
A true sequence with a missing ref raises at636 before stdout. Failed result
publication cannot issue a compact success receipt, even if a partial or
pre-exit file exists. Parent still must check exact retained bytes and hash.

Success **634-645** encodes exactly five keys: schema, mode, original phase
start, original phase end, and the unchanged save return. The nested ref has
exactly path/bytes/sha256. Both schema objects are closed. Mode comes from
the fixed CLI. Start is canonical ASCII decimal, at most20 digits; end is
the unchanged start+120000000000ns. The ref length is the actual encoded full
result length, capped at8MiB. SHA is the lowercase digest of those exact
bytes, including LF. Path comes from the original canonical root and fixed
mode-specific result name. No caller input supplies an alternate result ref.

Encoding remains sorted compact ASCII JSON, UTF-8 plus one LF. Independent
data serialization exactly reproduces the two maximum-shape files:

- SOURCE:377 bytes, SHA-256
  `f7b30ffde216714390e80ff5f60a2d86de3c0ee4edbf4e097fb55ae6f936cd33`.
- COPY:373 bytes, SHA-256
  `be5c80be5160d11fd5df69e72a09107cf76d52a87505e734fb2c747e3f1f65c8`.

Those bounds use the exact96-byte root,20-digit start, conservative21-digit
end,7-digit8MiB length and64 hex digest characters. The parent v2 schema has
a stricter20-digit end ceiling; using21 here safely overbounds its valid
receipts. Maximum-shape data is synthetic and is not an admitted parent
observation. All keywords used by the retained compact schema were checked
against these data. JSON Schema alone does not enforce exact Python integer
types, canonical raw bytes, arithmetic, root identity or external predicates.

For root content of JSON-escaped byte length E without quotes, SOURCE is at
most281+E and COPY277+E. Thus E<=231/235 suffices for the512-byte limit. The
generic caller's root admission does not impose that bound. A longer or
escaped root can fail even after durable result publication. R7 states this
limit; it does not promise delivery for every admitted root. The separately
bound SOURCE payload fixes the96-byte root; its guard/original-evidence
binding and parent integration remain external duties. There is no root
substitution or new admission rule in this repair.

The same phase encloses serialization, FIFO `fstat`, actual integer
`fpathconf(1,'PC_PIPE_BUF')>=512`, nonblocking setup, pre-write clock check,
one raw exact-count write at654 and postchecks. The finite<=512 record removes
R6's dependency on fitting the complete>=209116-byte result in one pipe
window. For a conforming pipe with sufficient space this is within PIPE_BUF's
atomic allowance. Full/insufficient space can still yield EAGAIN; interruptions,
faults, short returns or deadline loss still fail. No target pipe was queried,
measured or qualified. Existing local SDK documents were read as static
support only. They do not establish target capacity or a universal schedule.

There is one stdout write site, no loop or second attempt, no writable wait,
tail/drain, changed launch flags/environment, extra phase or pending Python
payload. The failed branch **646-649** and shared write tail are AST-exact to
R5. Failed full fallback can remain partial/absent and failed. Module entry
**769-776** remains byte-exact with `os._exit`. No deferred shutdown flush or
late traceback is restored. These facts close R6-P2-1 only in source.

## Preserved source contracts and prior closures

Original R1 remains CLOSED_SOURCE_ONLY. **35-38,65-115,664-667,689-693** retain
the exact darwin CLOCK_MONOTONIC6 identity and integer ns. Common receives
only a conservative local duration conversion: local sample first, shared
sample later,1ms deducted and rounding downward. No fallback, offset,
retagging of old~0.x clocks, phase reset or new rate claim exists. R8-P2-1
is a separate post-poll action gate, not a reopened clock-origin mismatch.

Original R2 remains CLOSED_SOURCE_ONLY. **536-545** stores active command and
all bounded status facts and latches first failed COMMAND before publication.
**547-563** preserves attempted/partial/durable refs. Comparator work begins
only after durable raw/status at564. Its first failure is assigned before
comparator publication. **734-757** retains COMMAND precedence, then assigned
comparison, separate stop detail/publication errors and nulls. `direct_children`
uses retained actual returncodes without another poll or invented exit.

Caller-owned R4 R3 remains CLOSED_SOURCE_ONLY. SOURCE **194-226** keeps one
exclusive raw write, exact-count rejection, fsync, one raw close and existing
directory sync. Close has no deferred Python payload; a secondary close error
is separate from the earlier failure. No retry/refund/delete/repair is added.
The summary tail and `os._exit` remain unbuffered. This closure does not cover
unchanged common internals, hard OS stalls or universal durability.

Startup **245-268,384-394** stays byte/AST-exact. SOURCE requires SYSTEM3.9.6,
the original flags and exact ten required values plus all six named extras:
CPATH, LC_CTYPE, LIBRARY_PATH, MANPATH, SDKROOT, __CF_USER_TEXT_ENCODING. It
discards those six locally without trusting/exporting their values and checks
the original map. Missing or seventh extras reject. COPY permits only the
original ten keys and deletes nothing. The public SYSTEM observation did not
query five values or trace origins; deletion does not undo pre-entry effects.
No private COPY startup observation or SYSTEM-derived COPY exception exists.

SOURCE **695-720** keeps trusted SYSTEM entry-template before exact private
`-I -S -B --version`, fixed Python lipo, then50 ordered native vectors:53 caller
records. It never imports common or repeats source-file qualification. COPY
**680,702-729** keeps17 pins before common import, prefix-before-private-entry
as parent duty, canonical prepared bundle checks, fresh SOURCE v2 and file
checks before/after50:52 caller records. Its separate parent SYSTEM entry
authentication must also fit the original phase before private entry.

Collector **408-528** retains while-read1MiB combined capture plus one detection
byte, actual direct status/reap, EOF, drops/unknowns, close errors and fixed
cwd/environment/DEVNULL/shell=False/closed fds. One Popen site exists; no drain
or repeat exists. The nominal command end remains min(start+10s,phase-end-1s)
with0.5s held inside it for kill/reap. R8-P2-1 is the specific missing gate
within that unchanged implementation; no overall timing clearance follows.

SOURCE **165-188,198-202** sums owned0600 regular initial files and owned0700
directories before any write, with no ledger. COPY **190-192,222-225** uses
only the existing ledger and exact common regular `critical=False` pre-write
reservations for every guard, parent envelope/stream, raw/status/comparator
and result.8MiB/file,32MiB total,16MiB product,4MiB cleanup,64KiB accounting
and12,517,376 initial regular bytes remain. Common's double temporary/final
charge, exclusive final link and locking remain unchanged. No second writer,
ledger init/repair, reserve borrowing or new receipt file is added.

Comparison **531-533,569-583,599-621** still sorts only complete strict-UTF-8
splitlines for `-verify`, preserving duplicates and empty lines. Other text
is exact after the declared bundle substitution. COPY needs both saved basis
and fresh SOURCE. Failure, missing data and nulls cannot become a pass.

## Independent bindings and actual static checks

[bindings.json](bindings.json), [function-bindings.json](function-bindings.json),
[static-checks.json](static-checks.json), [byte-bound-check.json](byte-bound-check.json)
and [findings.json](findings.json) retain the exact identities and results.

- All21 R7,17 R5 and7 R6 files equal their original Git, imported Git and
  current bytes. All20/16/6 SHA256SUMS entries match. All15/12/5 JSON files
  strictly parse with unique keys and no non-finite values.
- The1,712-byte five-hunk zero-context patch equals independent diff generation
  and reconstructs all46,104 bytes in memory. No source file was patched.
  Python3.9 AST data finds exactly33 functions: only `emit_summary` and `main`
  changed;31 are byte/AST-exact. No function was added/removed. All other module
  nodes, Writer class and entry block agree. Both full function excerpts and
  all declared function ranges, lengths and hashes match.
- All17 pins match tracked bytes. All ten module destinations/hashes/lengths
  match exact controller candidate `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`,
  materialization and source-checks. All50 unique vectors independently equal
  recipe expansion: seven objects by seven verbs, then deep verify, including
  direct Autoupdate/daemon objects and all architecture/strict flags. Canonical
  SDK data differs from preparation only in ten header paths.
- Seven interface data files remain byte-exact to R5: CLI, pins, vectors,
  parent schema, both null templates and startup contract. Null templates
  remain non-admitting. All81 original R5 public binding claims were separately
  rechecked; all144 R7 public claims also match Git/worktree. These counts
  include repeated identities, not independent behavioral qualifications.

```text
R7 caller       c9a3b1bcfae0da2eced035c2c9e4048a92dc18340c940148a486cf12348b0987
R7 exact patch  2001349fe188bf0a8c83fd78f617217ce1cc1dd579e477ec926a2d244c6e2566
R5 caller       ba757e7a03d890f3b3ad5e9b72c1580bbc4839543a746f81516bc962983de650
17-pin data     31a17c129e0af6c8c0437458c01fa8c28cffb89f29ee327648a31fed96608352
50-vector data  e3ff5756ab3b35a5c815e50ac1fad6520117836e080849260de87592bd327fa0
helper          bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428
common          20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c
controller      08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4
```

## Unresolved parent obligations and proof limits

Parent must separately bind/validate the compact interface, strict raw encoding
and exact types, mode/phase/clock, exact result path/bytes/hash, full v2 result
and all53 SOURCE or52 COPY records. Retain every actual status, raw stream,
comparison/publication state and duplicated handle/status binding. Require
actual caller0/reap, complete stdout/stderr EOF and no loss/error/kill/deadline
failure, with actual observations before the SAME original120s end. A compact
receipt, result file, sequence Boolean or native50 alone cannot accept.

The SOURCE wrapper treats stdout as opaque; it supplies no compact parser or
payload acceptance here. Its late-poll repair and transport R4 review remain
separate work. No parent integration, rendered payload, route, D4 or COPY
publication integration is cleared. Parent must retain transport actual exits
and all original outer deadlines as well as remote caller facts.

Public report `492f067` retains the parent's seven-original inspection. This
review verifies the tracked report, not that private causal act. Later remote
five-file checks and NEW inspected/start readings remain mandatory. Manifest
`1761bb0f0a999d26ea27d2ebf656d554a083f0f017429d3ee76e8edd8ad26a26` and facts
`ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192` stay exact.
Saved audit f7 supplies file-pass evidence only. Bridge ce7 supplies the entire
50-record commands projection only: canonical
`b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`.
Raw wrapper `1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f`
and canonical full wrapper
`7fd71953cfbbb71bc194a17412840e4b2420a081e6615e2865cc7abf4103c159` remain
distinct. Hashes do not replace raw checks, causal inspection or pre-entry trust.

Local prerequisite `190e11f` qualifies only the observed local API/version/flags;
it ran no caller and supplies no remote/cross-process/clock-rate/D4 result.
No timing, actual pipe delivery, target PC_PIPE_BUF/capacity, signal delivery,
OS-stall bound, clock conversion accuracy, private startup, durability under
faults or containment is proven here. Full artifact/preparation/operator/run
receipt gates remain external. No native, signature, ABI, product, continuous
NI15, UI, performance, release or whole-flow acceptance follows.

Only independent static file/Git/AST/JSON/hash/patch data readers and the
review-folder commit ran. No supplied source/check or AST was executed,
evaluated, imported or compiled to executable code. No tests/unit tests/mocks,
failure injection, private access, remote/network/native/process/port query,
private interpreter, child agent, prepare, COPY, controller, product, build,
signing, credential or activation ran. All candidate execution counts are zero.
No TypeScript changed. Commit hooks and signing are disabled. Candidate and
all existing tracked files remain unchanged.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; first strict post65431 EADDRINUSE cause UNKNOWN;
whole-flow NI15/runtime UNVERIFIED; earlier23/25 FAILED; all failed files/guards;
missing original second post-bind, bundle-after, outcome/archive; stripping
warning; unknown full outer build duration/exit; and the closed updater.
The completed source-file pass supplies no new native metadata or flow result.
