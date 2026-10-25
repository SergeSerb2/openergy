# Inert local M1 metadata transport envelope

2026-10-25, America/Vancouver. **SOURCE/DATA ONLY. execution=false.
Independent review REQUIRED. Runtime UNVERIFIED.** No future execution approval
is created. Only this folder is owned. The new source is
[transport-collector.py.txt](transport-collector.py.txt). No caller, controller,
common module, native command, phase budget or product source changes.

The imported metadata-basis bridge is BOUNDED_DATA_CLEAR at
`ce7fe4a1af63af88e0b0b2cacab288e4a46a0d13`. Caller
`621e0e3a9037e15b391e05b964dfcdb25da01515` remains HOLD under review
`54bc355f5f733008a444c45cafe1e4aa7b819777`. Its clock and first-failure repair
is separate work. This transport neither supplies that repair nor integrates
the held caller. [bindings.json](bindings.json) retains the public source hashes.

Read the [old collector provenance](../metadata-collector-provenance-2026-10-25/README.md)
and its [observations](../metadata-collector-provenance-2026-10-25/observations.json).
Its old local35s outer timeout and capture-then-write pipes did not impose a
general streaming cap. Its outer query files were uncapped. Those saved results
stay scoped historical evidence. This proposal does not rerun or replace them.
The [next ordinary preparation](../m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/README.md),
[receipt contract](../m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44/receipt-contract.md)
and [caller R1 README](../m1-artifact-metadata-caller-2026-10-25-t3-43cef465/README.md)
remain the parent boundary.

## Closed future interface

This repository text stays inert. After independent source review and all
existing parent bindings, the parent can materialize exact reviewed bytes as
an owned0400/0600 regular local file. The only proposed coordinator is trusted
local macOS CPython3.15 with `-I -S -B`, without optimization. No private Python
is used by this transport. The observer clock contract needs CPython>=3.10;
that broader clock contract does not broaden the coordinator version.

Exactly nine named arguments are required, each once, with no optional flags,
shell strings or arbitrary remote command fields:

```text
TRUSTED_LOCAL_CPYTHON_3_15 -I -S -B MATERIALIZED_COLLECTOR
  --phase metadata-source|metadata-copy
  --profile ABSOLUTE_PREBOUND_M1_PROFILE
  --profile-sha256 EXACT_PROFILE_SHA256
  --payload ABSOLUTE_PREBOUND_SYSTEM_PYTHON_STDIN_PAYLOAD
  --payload-sha256 EXACT_PAYLOAD_SHA256
  --run-dir NEW_ABSOLUTE_LOCAL_RUN_DIRECTORY
  --collector-sha256 EXACT_MATERIALIZED_COLLECTOR_SHA256
  --envelope-start-ns PARENT_LOCAL_MONOTONIC_NS_BEFORE_SPAWN
  --clock-sha256 SHA256_OF_CANONICAL_LOCAL_CLOCK_DESCRIPTOR
```

This is an inert vector description, not a launch instruction. Parent clock
descriptor canonicalization is sorted compact ASCII JSON plus one newline.
The observer contract supplies its exact fields. The parent binds the trusted
coordinator launcher separately. `sys.executable` can resolve to a toolchain
path. The collector records the resolved path, full version and implementation.
An invalid CLI or clock qualification returns failure without console data.

[profile.template.json](profile.template.json) has unresolved nulls and cannot
pass admission. The real profile has exactly `schema`, `source_guard`, `argv`.
`source_guard` is exactly the absolute path and SHA-256 reference of the actual
already user-verified M1 source-file-qualification guard. Its `argv` is that
guard's whole original SSH vector, byte-for-byte as JSON string values. Parent
must compare it to the actual saved guard before binding the WHOLE profile hash.
The collector does not open that guard, discover a host, or validate a claim of
user verification. That claim remains an explicit parent admission duty.

The closed vector is `ssh` or `/usr/bin/ssh`, the exact ordered public `OPTIONS`
in the source, one selector token, then the single literal remote command
`/usr/bin/python3 -I -S -B -`. `OPTIONS` is the public16-token sequence at
`windows-missing-pid-repair-r9-2026-10-24/source/controller.py.txt:30`.
That public occurrence establishes syntax only. It does not establish the
private M1 guard's values. A mismatch blocks use; do not add options, change
their order, substitute an alias, or adapt the profile to make it pass. The
selector grammar is closed ASCII and rejects whitespace, leading options and
shell syntax. The exact hash-bound vector is passed unchanged to Popen, with
the local executable fixed to `/usr/bin/ssh`, no shell, no PTY request, no
upload command and no repository module execution. PATH/LANG/LC_ALL are the
fixed source constants. Parent must retain compatibility with that environment.
Parent must already have evidence that the verified SSH configuration introduces
no proxy, PTY, forwarding or host/config mutation. A closed argv alone cannot
establish effective SSH configuration. Missing evidence blocks use. This task
does not read that configuration, run a query, or add disabling options.
No profile argv, selector, raw profile value or exception text is published.
Only bound file references, hashes, admitted sizes and verified flags are saved.

Profile, payload and materialized collector opens traverse directory components
with directory-relative O_NOFOLLOW handles. Leaves use O_NOFOLLOW|O_NONBLOCK,
then fstat must show regular, current-owner, one-link0400/0600 files. Reads cap
WHILE reading: profile128KiB, payload1MiB, collector128KiB. Each read is at most
64KiB and remaining capacity plus one overflow byte. A FIFO, device, symlink,
oversized or unstable input fails before SSH. Linked and opened file identity,
mode, owner, size, link count and change times must remain equal through the read.
There is no `read_bytes()` followed by a cap.

The already hash-checked payload bytes become one exclusive0400 local snapshot,
`payload.stdin`. The source reopens that snapshot read-only and retains its FD
as SSH stdin. Later changes of the original input pathname cannot change the
supplied bytes. The input FD has finite regular bytes; there is no feeding pipe
or file-upload command. The parent must protect admitted files and the run
directory from concurrent same-user edits. This is a cooperative local owner
boundary, not a defense against a hostile process with the same identity.

## One envelope, one attempt

Parent captures the local start BEFORE collector spawn. Work admission, imports,
input reads, snapshot writes, direct SSH spawn and capture use the original
start+130s boundary. At most2s of local kill/reap follows, ending no later than
start+132s. Final publication has at most3s and ends no later than start+135s.
The actual early settle/publication bounds are the minimum of their fixed end
and stage entry plus their allowance. No inner stage renews the135s envelope.
The remote metadata caller retains its separate unchanged120s phase. No remote
clock is compared with the local monotonic clock. This transport cannot create,
reset, translate, or certify that remote phase.

The run directory must not exist. It is created0700 under a no-follow parent
handle. `once.guard.json` is exclusive0600 and fsynced with its directory BEFORE
SSH spawn. Guard and receipt bind the collector, payload and whole profile
hashes, source references, local clock descriptor, resolved Python version,
deadline arithmetic, caps and retries0. They never disclose profile contents.
There is one Popen call and no retry, completion pass, fallback input root,
guard reset, overwrite, unlink, remote cleanup or host/config/known-host edit.
Parent must assign the one run path once; a new nonce is not permission to retry.

Only the actual direct local SSH Popen handle is retained. Nonblocking stdout
and stderr reads have128KiB and64KiB prefix caps and192KiB combined. A read can
observe one additional overflow byte. It never writes that byte to the prefix.
The first observed cap, exit, timeout, read or other failure is retained in
memory before any following write. A publication error is recorded separately
and cannot replace an earlier failure. Streams record observed bytes, actual
written prefix bytes/hash, known read-dropped bytes, EOF, overflow, read-drop,
unknown unread byte count and fsync result. Unknown unread bytes are null until
real EOF. No EOF is inferred from process exit or a closed local pipe.

Failure closes unread pipes without a drain pass. On a cap or deadline, only
the exact retained SSH handle can receive KILL, after its poll still reports
unreaped/alive. Other failures have only the bounded actual wait; an unreaped
handle remains explicit and no later cleanup pass occurs. No PID
search, process group, product, remote signal, cleanup or discovery is permitted.
Kill return does not prove delivery or remote closure. The source polls/waits
the actual handle and samples its local exit observation immediately after the
successful poll/wait, before receipt writes. Actual exit is null if not reaped.
No PID, status or closure is fabricated. SSH exit0 alone does not establish a
remote child exit or metadata validity. Remote child exit stays null and current
state UNVERIFIED; a separate reviewed payload receipt must supply any such fact.

Exclusive raw prefix files retain partial data on failure. They are fsynced
inside final publication when possible. A write/fsync failure retains its
partial file; a receipt can be absent or partial. A final failure can attempt
one separate bounded `publication-failure.json` inside the SAME final window.
It never rewrites a receipt or conceals that receipt's possibly stale flags.
Maximum file caps are payload1MiB, stdout128KiB, stderr64KiB, guard16KiB,
receipt32KiB and secondary failure16KiB: at most1,310,720 local file bytes.
There is no output on the collector console. Parent can use DEVNULL for both
console handles and must not introduce uncapped outer pipe capture.

Filesystem calls, spawn, scheduler delays, signals and wait can stall inside
the OS. O_NONBLOCK does not turn regular-file writes/fsync into hard real-time
operations. Deadline checks occur before and after cooperative calls. A stall
that returns late fails closed; a call that never returns cannot publish a
timely failure or guarantee cleanup. This proposal claims no hard OS preemption,
universal containment, continuously measured absence or timely CLI exit.

## D4: independent observation of this collector's own exit

[parent-observer.contract.json](parent-observer.contract.json) is a finite data
interface and concrete direct-parent recipe. It creates no wrapper process,
watcher, thread, scheduled task or infinite wait. The trusted same-Mac direct
parent uses CPython>=3.10 and a matching Mach clock descriptor. It records its
start and absolute end before retaining the actual collector Popen handle.
It waits only until the original135s absolute end. After a successful actual
wait/poll, it MUST sample monotonic_ns immediately, BEFORE reading any receipt.
That independent observation must be strictly less than the bound absolute end.
A late observation is rejected even if receipt.json says TRANSPORT_COMPLETE.

Timeout, missing actual exit, incompatible host/clock, nonzero collector exit,
missing/partial/stale receipt, dropped streams or missing payload predicates
reject acceptance. The observer returns bounded failure data on timeout; it
does not wait indefinitely or start a later watcher. It does not infer an exit
from a PID or a file. A later process can qualify a saved actual-exit record and
then make its own same-clock observation BEFORE the same absolute end, but it
cannot invent that actual OS-exit record or use a sample at/after the end.
Collector self-elapsed/prewrite fields cannot discharge D4. The independent
observer has the same cooperative OS-stall limitation; its late sample fails.

## Parent integration and proof limits

This is LOCAL raw transport capture. It is NOT the R9 post-prepare reserved
publisher or accounting writer. Parent MUST bind each exact source/copy SYSTEM
Python stdin payload and the existing R9 reserved publisher/accounting path
before use. There is no supplied remote payload, remote upload, remote data
writer, private-Python entry or artifact constructor here. When separately
assigned in the future, only the exact pinned remote payload may run. This
adapter's files cannot become a prepared-copy receipt by copying or renaming
them around R9. Parent must inspect and publish complete raw payload facts via
the existing reserved writer, preserving all full receipt predicates.

[finite-plan.json](finite-plan.json) keeps bindings null and execution false.
It creates no approval and no artifact, source, copy, signature, ABI, native,
performance, release or runtime acceptance. Independent review is still required.
No new human approval cycle is introduced. The parent retains the held caller's
findings and all old failures; no old guard or state is reset.

Validation used only repository/Git reads, reviewer-owned JSON/hash processing
and static AST parsing of text. No supplied source, AST, helper or module was
executed or imported. No tests, mocks, remote/native action, process/port query,
private read/write, credential, product, build, signing, integration, TypeScript
or child-agent work ran. Runtime behavior, time/stream bounds, signal/reap and
durability remain UNVERIFIED. Static checking cannot prove them. The owned
folder commit uses hooks and signing disabled. Function inventory and static
observations are retained in [static-checks.json](static-checks.json).
