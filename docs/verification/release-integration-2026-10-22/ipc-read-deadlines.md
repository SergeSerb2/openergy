# Incomplete IPC request recovery — 2026-10-22

## Failure and cause

The resumed hardening pass found that a client could connect to local IPC and
never finish its first request or a later frame. The socket's short read wait
only checked shutdown; it did not expire incomplete requests. Sixty-four such
connections could occupy every client thread indefinitely and prevent new CLI
connections. This is a local reliability problem. The IPC trust model still
allows other processes running as the same user to issue control requests.

The baseline Rust source was `b19ff091e84723c65b1f93a4a28cafeebf5e8875`.
Two real-daemon runs reproduced the failure. The final control held one client
that had completed a request and 63 clients split between no bytes, a partial
header, a partial body, and a body receiving periodic extra bytes. After 11.5
seconds, all incomplete clients were still open and a fresh public CLI status
call failed. The existing client still answered. Closing one stalled client
restored CLI access, distinguishing occupied slots from a stalled session owner
or failure to reap finished threads.

## Change and regression path

Runtime fix: `89b79e1433cf32b64df465b89fcd486cc8823949`.
The first request and every incomplete frame now have an absolute ten-second
read deadline. Partial bytes do not restart it. A completed request may remain
idle indefinitely for subscriptions. If another frame is buffered, its read
budget starts after the preceding response. Shutdown and the 16 MiB frame cap
keep their existing behavior. No dependencies or wire fields changed.

The committed [external regression command](../../../crates/daemon/scripts/verify_ipc_deadlines.py)
starts the actual virtual daemon, uses actual Unix sockets and the public CLI,
and cleans up only its own process and socket:

```sh
python3 crates/daemon/scripts/verify_ipc_deadlines.py \
  --openergy target/release/openergy --report /tmp/ipc-deadlines.json
```

The same final script failed six of 19 checks on the baseline and passed
**19/19** on the fixed binary. All 63 incomplete clients expired while the
healthy idle client remained usable. Fragmented and pipelined complete
requests, method-error recovery, a later incomplete frame, oversized-header
refusal, and shutdown with a pending frame were also exercised. Both baseline
and fixed daemons exited zero and removed their owned socket.

The existing daemon/CLI audit also passed **15/15** with the fixed binary.
The broader virtual suite passed **88/88**, with **30 Docker scenarios
explicitly excluded** and zero cleanup failures. These exclusions are not passes.
The locked release build, strict workspace/all-targets release Clippy, and
workspace formatting check passed with Rust 1.99.1. Build jobs used two workers
and reduced process priority. Exact hashes and the broader virtual-suite result
are in the [machine record](ipc-read-deadlines.json).

Both baseline and fixed daemon executables were **3,674,144 bytes**. In the new
S15 run, server/client peak RSS was 9,355,264 / 9,273,344 bytes; idle CPU was
0.04996% / 0.01665% over separate 60-second windows. For 1,000 virtual moves,
external IPC-to-receiver-ledger latency was p50 0.711 ms, p95 0.787 ms and p99
0.860 ms, with a 2.738 ms maximum. This used plaintext loopback, not physical
input or a network product comparison. Run-to-run differences do not establish
a latency or memory improvement caused by the request deadline.

## Scope and owner choices

Checks ran on macOS 27.2 with private temporary configuration, control and TLS
paths. The new regression uses the off role, virtual backend, discovery disabled
and production-port opt-in denied. No installed process, live input connection,
native clipboard, permission setting, service, task or login item was changed.
The shared client handler also serves Windows pipes, but this run does not
establish Windows pipe runtime acceptance. Native acceptance remains a release
gate at the maintainer's request.

On 2026-10-22, the maintainer chose to skip GitHub private vulnerability reporting
setup and resume hardening. That supersedes the earlier reporting prerequisite
for this work. No private reporting channel is claimed as available. The
repository remains private. The two existing PR security threads, positive
Windows migration recovery and trusted distribution remain open.
