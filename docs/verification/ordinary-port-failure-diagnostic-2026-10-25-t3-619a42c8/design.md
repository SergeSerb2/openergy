# Failure-boundary design

## Exact source and scope

The input is R9 `e34401c8842c2a93b5c355f4c515a79ee05cfa72`, with M1 admission
`02c9b174c8e13d4eaa6bf7692f20cbb44c93ae57`. The eight original modules
reconstruct from the admission patch as static text. Their hashes agree with
the public saved-attempt review. Its private archive and original run paths
were not opened. All original files remain byte-exact.

Only `Controller.port` and two new module functions change behavior. All other
controller functions retain identical text and AST. `common.py`, `macproc.py`,
`ownership.py`, `watchdog.py`, `exec_owned.py`, `prepare.py` and `probe.py` are
unchanged from M1 admission. This retains the R9 uncertainty journal, parent/
watchdog handoff, app-first peer ordering, absence gates, direct-child exit
collection, guarded launch, NI15 assignment and existing observation limits.
The product and its 25 predicates do not change.

## First failure and publication

The original pre-bind `/usr/sbin/netstat -an -p tcp`, regex-selected rows,
`state_before_first_bind`, single IPv4 loopback bind and success record stay
present. No original snapshot is replaced by the later query. On an exception,
the controller first sets `failed=True` and retains `first_port_failure`, with
`success=false`, `status=FAILED`, errno, exception type, requested address and
the original snapshot. It retains the traceback with an explicit 8192-character
limit. The original Python exception is rethrown with a bare `raise`.

The bracket starts immediately before Python's bind call and ends at entry to
its exception handler. It includes Python and deadline-context unwinding; it
is not an exact kernel syscall bracket. `bind_returned_monotonic` is null on
bind failure. `bind_succeeded` distinguishes a failed bind from a later success-
receipt failure. Receipt errors cannot trigger a bind-failure query.

The first receipt uses the original exclusive filename, `first` semantics and
cleanup charge class. Its maximum write interval is a subset of the same port
deadline. `Evidence.attempt` catches ordinary sink/deadline failures. A failed
write leaves the original exception and in-memory failure intact. The separate
diagnostic record also embeds that failure, so its successful publication can
retain the failure if the first sink was unavailable. If both sinks fail,
durable evidence is unavailable: the source promises a failed exception and
in-memory retention, not a successful file write.

## One richer query, with bounded ownership

Only postflight `EADDRINUSE`, after a bind that did not succeed, enables the
new query. The unchanged `run` reaches postflight only after product collection,
recovery, watchdog exit and its independent absence check. No query is added
to a live flow or preflight failure. No diagnostic calls collect, observe,
cleanup, recovery, product launch or product/group signal functions. The
unchanged outer postflight loop stops on the original exception; it cannot
try the second port or reach final archival as a diagnostic convenience.

The fixed argv is `['/usr/sbin/netstat', '-AanW', '-p', 'tcp']`. No shell,
continuous mode, lsof, process enumeration or secondary command is used. The
existing standard TCP query path necessarily enumerates TCP rows internally;
the additional query has the same narrow protocol scope. Only rows whose
local or foreign numeric port exactly equals the failed declared port survive
in its record. Unrelated rows exist only in a bounded temporary buffer.

`-a` requests server sockets, `-n` numeric endpoints and `-W` avoids address
display truncation. `-A` requests protocol-control-block display. It addresses
the omission risk in the old default view, which can omit unconnected or
closing records. This is a request for a richer standard display, not proof
that the target netstat/kernel exports every PCB. The local installed manpage
is bound as static data: it documents these flags and endpoint/state columns,
but does not establish target-M1 implementation identity or full visibility.
No deployed ABI, kernel rejection branch, PID ownership or complete namespace
inventory is inferred. Unsupported flags or layout produce incomplete context.

The parser locates the protocol token after optional leading PCB fields. It
records local and foreign endpoints, state, queues, exact selected row text,
and separate local/remote port matches. It applies no connected, LISTEN or
TIME_WAIT state allowlist. Thus LISTEN, CLOSED, LAST_ACK, FIN_WAIT, CLOSING and
other states are retained when the tool reports them in the recognized layout.
IPv6 and wildcard rows remain visible with their literal endpoints; a matching
remote port is never treated as a local bind conflict. Unexpected layouts,
malformed rows, decoding loss, long rows, missing header or row overflow are
explicit parse limits. Empty selected rows never establish absence or cause.

The query packet records exact argv, cwd, C locale overrides, invocation and
spawn timestamps, read and absolute deadlines, direct PID, actual retained
`Popen` return code when known, reap state/time, kill request/return brackets,
EOF flags, bytes actually read per stream, output cap, timeout, truncation,
bounded stderr and structured errors. Read counts are not total bytes emitted
when EOF is missing. A kill call returning is not proof of signal delivery.
The query's timestamps remain separate from the bind bracket. Snapshots after
bind cannot establish atomic bind-time state or select its rejection cause.

The parent-provided clarification preserves the existing helper contract:
only the retained, unreaped direct netstat child can be killed on timeout or
collection error. No PID search, PID/group authorization, product signal,
host signal or cleanup sweep is added. The handle is retained before I/O.
If the same deadline cannot reap it, the packet remains incomplete and the
handle stays in a diagnostic-only in-memory map. There is no claim that an
uninterruptible OS operation can always be reaped inside a fixed deadline.
The code neither extends the deadline nor inserts that helper into the product
intent/handle/exit sets. The unchanged product recovery path is already closed.

## Time and space remain inside existing caps

The new query deadline is `min(original_port_deadline - 0.05, now + 0.5)`.
Less than 0.05 seconds available means no spawn and `INCOMPLETE`. Eighty percent
of its interval is available for spawn/read/wait; the remaining interval is
reserved for direct-helper kill/reap. Nonblocking pipes and a single continuous
read loop drain that one command. There is no sleep, repeated snapshot or
poll-for-a-passing-socket loop. All parsing is under the enclosing query alarm.
The later publication deadline is also a subset of the original port deadline.
The original two-second port budget, 120-second flow, 60-second cleanup,
20-second evidence archival budget and existing exit bounds remain unchanged.

The combined stdout/stderr read cap is 262144 bytes. The parser retains at
most 64 matching rows of at most 1024 bytes, four unparsed matching line
prefixes and 4096 stderr bytes. It records every associated truncation flag.
The entire encoded diagnostic has an additional 98304-byte limit. A record
that cannot fit is not written; its in-memory publication error remains and
the bind exception still escapes. There is no fallback copy or larger packet.
The exclusive diagnostic writer uses `critical=False`, so the existing byte
ledger double-charges temporary/final records from regular capacity before
payload growth. It cannot spend the 4 MiB cleanup reserve, its terminal MiB,
product reservation or accounting reservation. The 8 MiB file, 32 MiB aggregate,
6000 round and 24000 sample caps are unchanged; this adds no process samples.

## Result boundary

`COLLECTED_CONTEXT_ONLY` means the one helper exited0, both streams reached
EOF, parsing completed and no collection limit/error was recorded. It is not
the bind result. All other cases are `INCOMPLETE`. Cause is always `UNKNOWN`.
Any row is later context, not proof of TIME_WAIT, a leak, a retained owner or
a particular kernel rejection. A zero exit, empty rows, successful recovery
or missing diagnostic cannot turn the first bind into a pass.

This task used source/data checks only. Runtime behavior, target command layout,
actual timeout/reaping behavior, diagnostic publication and full-flow NI15
remain UNVERIFIED. It repairs evidence collection source only. It does not
repair, explain, reopen, retry or accept the historical EADDRINUSE.
