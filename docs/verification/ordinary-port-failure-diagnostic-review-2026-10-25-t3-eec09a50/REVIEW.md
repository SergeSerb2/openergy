# Independent source/data review: strict postflight bind diagnostics

2026-10-25, America/Vancouver. **SOURCE CLEAR: no concrete actionable P1/P2
found in this candidate. Runtime remains UNVERIFIED.** This is a source review,
not runtime acceptance, an executable receipt, approval, or a historical retry.

Candidate: `2aefaf4056a8d35a005f38e7856ffad89d771a82`, imported as
`e4a2be2d2e92433d8b42c943bacc158b1687023d`. Reviewed folder:
`docs/verification/ordinary-port-failure-diagnostic-2026-10-25-t3-619a42c8`.
Reviewer checkout: `/Users/serge/.t3/worktrees/openergy/t3-eec09a50`, initially
clean on `t3/review-port-failure-diagnostic` at parent
`f9ea16761c0333ac8c965d2aee441e508a8b390d`.

The source references below use the candidate's `source/controller.py.txt`
and `source/port_diagnostic.py.txt`. The shared writer references use R9
`docs/verification/ordinary-controller-uncertainty-handoff-r9-2026-10-24-t3-79efd32e/common.py`.
M1 admission changes only two path constants in that common module.

## First failure stays failed

`controller.py.txt:246-265` retains the original pre-bind query, selected rows,
snapshot, single IPv4 loopback bind, successful receipt, and reservation behavior.
There is one bind call, no reuse option, and no added sleep or retry.

At `266-284`, the handler sets failure in memory before a receipt attempt. It
keeps the original snapshot, errno, exception type, requested address, bounded
traceback, and bind bracket. `first_port_failure` is assigned before the sink.
The original exclusive receipt uses the same cleanup charge class as before.
The unchanged `Evidence.attempt` catches ordinary publication/deadline errors
and retains the error count (`common.py:201-213`). A failed receipt does not
replace the active bind exception.

Only postflight EADDRINUSE with `bind_succeeded=False` enters the diagnostic
branch (`controller.py.txt:285`). A successful bind followed by a receipt error
cannot enter it. Query and diagnostic-publication exceptions have separate
handlers at `303-317`. The final bare raise at `319` preserves the original
exception after these diagnostic operations. The existing socket-close finalizer
is unchanged. Failure metadata remains in memory if both receipt attempts fail;
durable retention is not guaranteed by source alone.

The bracket starts before the Python bind call and ends at handler entry. It
includes Python/context unwinding. It is not an exact kernel syscall bracket.
The packet explicitly separates later query timestamps from bind failure.

`Controller.run` is byte- and AST-identical to admitted M1 source. Its postflight
loop at `controller.py.txt:541-542` does not catch this exception. Therefore this
source adds no second-port attempt, archival, bundle-after record, or passing
outcome after the first failed postflight bind. No diagnostic result can accept
the saved attempt.

## Time, capacity, and helper lifetime

`controller.py.txt:292-302` sets the query deadline to the earlier of original
port deadline minus 0.05 seconds and current time plus 0.5 seconds. Insufficient
time skips spawn and records INCOMPLETE. The later publication interval is also
inside the original deadline (`307-314`). The outer query alarm includes parsing.
The unchanged nestable `io_until` restores the remaining enclosing timer; it
does not grant a fresh enclosing interval (`common.py:42-64`). No port, flow,
cleanup, postflight-archival, or process-exit allowance increases.

`port_diagnostic.py.txt:103-137` partitions the available query interval 80/20
between spawn/read/wait and direct-helper kill/reap. The one Popen call has a
fixed argv and no shell. Both pipes are nonblocking. The read loop limits the
combined stdout/stderr bytes before each read. It does not use an unbounded
communicate buffer. The static limits are 262144 combined bytes, 64 selected
rows, 1024 bytes per retained row prefix, four unparsed matching prefixes,
4096 retained stderr bytes, and 98304 encoded packet bytes. Read/parse limit
exhaustion blocks the query's COLLECTED_CONTEXT_ONLY status. The caller checks
the encoded packet cap separately. JSON escaping can make a packet exceed that
cap before nominal row capacity is reached; the source then retains a publication
error instead of increasing the cap. Query status is assigned before publication.
A later cap/sink failure can leave that context status collected, with a separate
in-memory publication_error/publication_failed marker. Neither the status nor
the in-memory marker proves durable publication. The original bind still fails.

The first failure receipt retains its original critical default. The new
diagnostic receipt explicitly uses `critical=False` (`controller.py.txt:313-314`).
The unchanged writer charges temporary/final payloads before growth under the
byte lock (`common.py:117-182`). It retains failed charges and rejects capacity
that would consume the existing cleanup, terminal, product, or accounting
reservations. Initial input charging remains unchanged. The 8 MiB file ceiling,
32 MiB aggregate ceiling, 4 MiB cleanup reserve including its terminal MiB,
eight 2 MiB product slots, and 64 KiB accounting reserve do not grow. No process
sample or observation round is added by this diagnostic source.

The helper handle is placed in the diagnostic-only map before subsequent I/O
or formatting (`port_diagnostic.py.txt:110-117`). The only added kill call targets
that retained direct child after poll reports it unreaped (`143-160`). There is
no PID search, product signal, system signal sweep, or group signal path. This
matches the parent's explicit direct-helper timeout clarification.

Actual return codes are retained when poll/wait has reaped the child, including
the exception path (`152-166`). If reaping cannot finish in the same deadline,
the handle stays in that map and the result remains INCOMPLETE. The code closes
the pipes without enlarging time or adding the helper to product exit ledgers.
Retention lasts only while the Controller object remains alive. The candidate
adds no later reaper or proof that an OS-stalled spawn/kill/reap can finish on time.
This is a stated source/OS limitation, not a cleanup PASS: product recovery was
already recorded before this helper started and cannot certify its absence.
The enclosing operation still fails. No actual helper lifetime was exercised.

## Supported flag and parser claims

The bound local static manual `/usr/share/man/man1/netstat.1` matches the
candidate's exact 17003-byte hash. Its SYNOPSIS permits the selected combination.
It documents `-A` as PCB-address/flow-hash display, `-a` as including sockets
normally omitted for servers, `-n` as numeric addresses, and `-W` as avoiding
address truncation in certain displays. It does not independently prove that
every unconnected or closing PCB is exported by the target M1 tool/kernel.
The candidate correctly leaves target implementation identity and visibility
unverified. This review did not query netstat, render its manual, or fetch source.

The parser scans the first three tokens for tcp/tcp4/tcp6/tcp46, so its recognized
layout permits up to two leading fields. It then expects queues, dot-suffixed
numeric/wildcard local and foreign endpoints, and a state token
(`port_diagnostic.py.txt:37-74`). Those are source assumptions, not target-layout
qualification. It keeps local and remote port matches separate and has no state
allowlist. It does not certify that a remote-only row conflicts with a local bind.
Recognized rows of any state can survive the filter; no claim about all possible
OS layouts or kernel records follows.

Missing header, stdout decode loss, recognized malformed rows, selected-row
clipping, and selected-row overflow make parsing incomplete (`75-79`). Command,
timeout, exit, EOF, read-cap, stderr-retention, and parse errors also block
COLLECTED_CONTEXT_ONLY (`177-185`). Unexpected output remains a qualification
limit; no parser was executed against real or synthetic rows in this review.
That status describes bounded later context only. Empty rows do not establish
absence. All cause fields remain UNKNOWN. The previous causal review's missing
bind-time rejection observation stays missing.

## Independent integrity evidence and retained failures

The complete saved M1 REVIEW.md and bindings.json, independent R9 and admission
reviews, old cleanup/rebind causal review and its diagnostic/research notes,
candidate package, changed source texts, shared writer, and admission patch
were read as source/data. Exact checks used new standard-library data readers
and raw Git only. No supplied code or supplied checker ran.

- All 12 candidate files match original candidate, imported commit, parent, and
  checkout. All 11 SHA256SUMS entries match the complete package except the
  checksum file itself. All five candidate JSON files parse. Original and
  imported full-index binary diffs are identical: 99666 bytes, SHA-256
  `6fdf282c8b347ab745a8396e0374a2af22a7b87ec5f3e35f17914d84ff4d2835`.
- All 24 declared dependency records match exact Git blobs, modes, lengths,
  line counts, hashes, and checkout bytes. All eight R9 modules also match
  `e34401c8842c2a93b5c355f4c515a79ee05cfa72`. The review binds 145 distinct
  repository inputs in total.
- Both seven-hunk patches reconstruct in memory with exact old/new positions,
  counts, and removed bytes. Admission reconstructs all eight saved M1 hashes
  at `02c9b174c8e13d4eaa6bf7692f20cbb44c93ae57`. The diagnostic patch reproduces
  both candidate text files exactly. No runtime module was materialized on disk.
- All nine proposed module texts parse as AST data. Seven M1 modules are
  byte-identical. All 14 other Controller methods are byte- and AST-identical.
  Replacing only Controller.port and removing the two new imports makes the
  complete controller AST equal to admitted M1, including main and FlowAbort.
  Source-checks sizes, line counts, and all nine hashes agree. Limits are exact.
- All 87 older ordinary evidence files match their immutable original Git
  records and this checkout: 410149 bytes. Their FAILED / NOT ACCEPTED, 23/25
  result remains unchanged.

The candidate's first whitespace-check FAILED/exit2 record remains byte-exact;
its losslessly escaped output hash also agrees. The candidate's final-check
PASSED is a separate retained claim, not a rewrite of that first failure.
My first static reader also failed, before any write: it compared a three-field
measurement to a record with an extra unchanged_from_r9 field. The bindings
retain its traceback and correction. Comparing the three declared measured
fields explicitly passed without changing candidate or historical evidence.

Exact proposed hashes:

- Controller: `b542f9841d42fd5d7e67e405699b408f1d440a103b675bb79b97c9c87e1e7104`.
- Port diagnostic: `9d35bf1adbde9209562d9b4cfa211b75574a0fb1922dc0297761880c2d6d3985`.
- Candidate folder Git tree: `9c278847c832891288e887c9f99815050b958202`.

## Proof boundary and handoff

Saved M1 remains FAILED_NOT_ACCEPTED: app25/25 and app OS exit0, controller OS
exit1, first strict postflight65431 EADDRINUSE despite the empty saved scoped
pre-bind rows, cause UNKNOWN. Scoped known-identity recovery does not accept
the attempt. Whole-flow NI15 remains UNVERIFIED. Second post-bind, original
bundle-after/final outcome, and archival remain missing. This review rechecks
the public saved-review bindings; it does not reopen the private archive or
repeat the saved runtime audit.

Target command/layout/visibility, actual timeout/kill/reap behavior, durable
publication under failure, uninterruptible OS bounds, and full-flow priority
remain UNVERIFIED. The separate priority candidate was not reviewed or composed.
Its Controller.port overlap requires later composition and independent review.
This proposal alone neither accepts nor reruns the failed attempt. The older
ordinary failure, isolation failures, Windows native stop, expired Sparkle
one-shot, native/release gates, and other stopped work remain unchanged.

Only this new review folder was written. There was no supplied program import
or execution, runtime/helper/query/remote/native parser/process/port probe,
test/mock, preparation, build, signing, product/network/credential access,
private write, child agent, candidate repair, integration, push, or publication.
No TypeScript changed. Hooks and commit signing are disabled for this
documentation-only commit. No new approval or executable receipt is created.

[bindings.json](bindings.json) has SHA-256
`ddb3379e08d292fb4aa68f1a97683492e3fc447efbfb4f28480f787a7a122c10`.
SHA256SUMS binds the report and bindings; the commit binds all three files.
