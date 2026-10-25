# Compact caller success contract

SOURCE/DATA ONLY. execution=false. IMPLEMENTED_UNREVIEWED. Independent R8
review is next. This contract changes successful caller stdout only. The full
`openergy.metadata-observations.v2` result, parent v2 boundary, null templates,
pins, vectors, startup and all artifact/run receipt gates stay unchanged.

The receipt has exactly five keys: `schema`, `mode`, `phase_started_ns`,
`phase_deadline_ns`, `result`. Schema is `openergy.metadata-success.v1`; mode
is `source` or `copy`. `result` has exactly `path`, `bytes`, `sha256`. It is the
unmodified return from the single existing
`writer.save('metadata-' + mode + '-result.json', result)` call. Its path is
the absolute existing root plus `/metadata-source-result.json` or
`/metadata-copy-result.json`. Bytes and SHA-256 bind the exact full canonical
UTF-8 result file, including its final newline. No file is read again, created,
rewritten, rehashed by a second writer, or replaced to construct the receipt.

Successful result serialization and publication follow R5's original path.
Only a successful save return supplies `result_ref`. That local starts null.
A save or enclosing deadline failure sets the existing sequence flag false.
No compact receipt is attempted for that failed sequence. A true sequence with
missing ref also fails before stdout. The full result has no self-reference
or receipt fields; its construction and existing file name stay exact.

`phase_started_ns` is the original result's `started_ns`.
`phase_deadline_ns` is the original start+120000000000ns. The receipt omits the
full clock dictionary to keep the byte bound small. Parent MUST check that
the separately hash-bound full result and original parent v2 record carry the
exact `openergy.shared-clock.v1` identity, darwin CLOCK_MONOTONIC6, integer ns,
same host and boot. Receipt numbers alone do not identify a clock. Parent v2's
existing integer bounds and every order predicate still apply. The schema's
conservative 21-digit deadline ceiling adds no admissible parent value or time.

Encoding is the existing `encoded`: sorted keys, compact separators,
`ensure_ascii=True`, UTF-8, exactly one final newline. Complete success bytes,
including that newline, MUST fit512. The new schema is closed at both object
levels. JSON schema alone does not enforce the byte ceiling, canonical raw
encoding, arithmetic, exact root, causal order or external observations.

The same original shared deadline encloses encoding, `fstat`, `fpathconf`,
setting nonblocking mode, the pre-write clock check, the single write and its
postcheck. Success requires fd1 to be a FIFO/pipe and actual
`os.fpathconf(1, 'PC_PIPE_BUF')` to return an integer >=512. These source checks
have not run on the target. There is no fixed target capacity assertion.

For a conforming pipe, a nonblocking write of at most PIPE_BUF is atomic: when
enough space is available, the pipe accepts the complete receipt as one write;
when insufficient space is available, it can return EAGAIN with no bytes.
An initially empty conforming pipe has room for this <=512-byte record. A full
pipe can fail. No universal successful-delivery guarantee follows. Interrupts,
closed reader, descriptor faults, alarms and other OS errors can also fail.
The exact return-count check rejects even an unexpected short count. The
static basis is the write/pipe/syslimits documentation already retained by R6,
not a new measurement of the M1 pipe or a reader schedule. PIPE_BUF, storage
capacity and the collector's1MiB capture ceiling are distinct quantities.

There is one stdout `os.write` site and one attempt per caller. No loop,
writability wait, retry, tail completion, drain, late flush, new phase or
second console attempt exists. fd1 stays nonblocking. The unchanged module
entry uses `os._exit`, so no Python payload waits for a shutdown flush. The
failure branch still attempts the complete failed/partial R5 result under its
original1MiB ceiling. That fallback can be short, absent or malformed and must
remain failed. It never uses the success schema or turns failure into success.
First command/comparator causes, publication errors, nulls and partial refs
stay in the full durable result when publication succeeds. If it fails, no
durable full-result claim is made; outer partial evidence remains essential.

Parent MUST apply these rules before accepting the finite metadata sequence:

1. Bind exact reviewed R7 caller bytes/hash, independent R8 verdict and original
   fixed launch/env/cwd/parent-record/root/once-guard bindings. Do not accept
   the inert maximum-shape data as an observation. Do not repeat qualification.
2. Retain the actual caller handle, raw stdout/stderr, OS status and observation
   stamps. Require actual caller0/reap, complete EOF on both streams, no error,
   overflow, loss, timeout, kill or uncertainty, and all original capture and
   shared120s deadlines. Check exit and EOF observations before the same end.
   A receipt or file is pre-exit data and supplies no actual exit fact.
3. Parse complete stdout strictly: reject duplicate keys, non-finite numbers,
   extra values/trailing data, wrong exact types, keys or schema. Require its
   exact canonical raw encoding plus final newline and <=512 bytes. Validate
   `compact-success.schema.json`. Require mode and both phase values to equal
   the actual original parent boundary and full result; deadline=start+120s.
4. Require `result.path` to equal the exact bound root and mode-specific fixed
   file name. Inspect the retained original file through the existing bounded
   data boundary. Require exact bytes and lowercase SHA-256, not a projection,
   canonical rewrite, alternate file or inferred reference. Require its v2
   schema, clock, mode, phase, expected basis and every existing success field:
   sequence true,50 native commands, no first error, first failed COMMAND,
   first failed comparison or publication error. Reject result-publication
   error fields and missing/extra success data relative to the unchanged
   caller's exact construction. Verify the once record's parent hash/start.
5. Inspect all53 SOURCE caller records (entry-template, private version, lipo,
   ordered50) or all52 COPY caller records (file checks before/after ordered50).
   COPY's separate parent SYSTEM entry authentication is also mandatory in the
   original phase before private entry. Retain every direct-child actual status
   and all collector/EOF/error/drop/kill/reap/10s-with0.5s-reap/phase predicates.
   Verify full ordered record sets and equal duplicated status/handle fields,
   all durable raw/status/comparison path/length/hash bindings and publication
   states, raw byte caps and actual comparisons. Only verify labels use sorted
   complete UTF-8 lines with duplicates/empty lines; all other comparisons stay
   exact after the declared bundle substitution. COPY requires both saved basis
   and fresh SOURCE. Check first causes/null/partial states without filling gaps.
6. Check the full unchanged artifact receipt contract and every remaining
   parent gate separately. Compact schema, actual caller0, a full result or50
   matching native rows alone grants no artifact, prepare, product, signature,
   NI15, runtime, release or whole-flow acceptance.

Parent has actually inspected the seven original completed source records at
`492f067`, within that report's saved-data scope. Later remote five-file checks
and new shared inspected/start readings remain in their original order. Do
not repeat qualification or retag old relative clocks. Prefix authentication
before EVERY private entry remains a parent duty.

Payload `fa72a0d` (import `dde69c7`) is an opaque1MiB caller collector with a
separate16KiB outer receipt. It does not parse caller JSON. Parent must bind
this compact interface after independent review through its existing data
boundary; payload bytes are unchanged here. Transport `88db0b5` remains held
for two separate narrow fixes. This package changes neither and clears neither.
No new human permission gate or runtime approval is introduced.
