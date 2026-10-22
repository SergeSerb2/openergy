# Idle IPC wake and Linux parent lifetime

Date: 2026-10-22 (America/Vancouver). The [machine record](idle-wake-and-parent.json)
keeps source IDs, binary/report hashes, original failures, measurements, and
review responses. All processes used private virtual state and assigned
loopback ports. Linux containers had no external network. Installed software,
native input, clipboard, TCC, services, and port 24800 were not used.

## Idle listener

The Intel daemon under Rosetta exceeded the existing 0.1% idle CPU gate in two
60-second checks: 0.166529% and 0.149850%. The listener woke every 50 ms even
without a request. Commit `2201e73` replaces that periodic poll with a blocking
poll and a private socket pair that wakes shutdown. The pair is created during
bind, before platform workers. Joining closes the owned writer and wakes the
listener without connecting to its pathname. Client deadlines, frame and queue
limits remain in place. This adds two descriptors, no threads or dependencies.

An isolated candidate check passed the same gate: server 0.099891%, client
0.049946%. The later full run also passed: server 0.083256%, client 0.033302%.
These are separate finite runs on Rosetta, with coarse process CPU ticks at
these small values. They do not establish a physical Intel or whole-app CPU
ratio. Independent round 11 source review found no concrete P1/P2 regression;
it did not repeat the runtime checks.

## Linux parent exit

The Linux baseline at `bf0b8aa` failed connected S20 at step 13: the raw peer
missed CBYE. Its unreaped zombie parent still passed `kill(pid, 0)`. The role-off
case passed. This was reproduced with the old listener as well as the candidate,
so it preceded the IPC wake change.

Commit `4a65290` adds an owned Linux pidfd and nonblocking exit observation. On
kernels or policies without `pidfd_open`, it reads at most 4096 bytes of
`/proc/PID/stat` before the existing ESRCH check. S20 then passed, including
under a real inherited seccomp policy that denies `pidfd_open` with ENOSYS.

Independent round 12 found a P2 in that first fallback: a main thread can call
`pthread_exit()` while a worker remains alive. The leader becomes a zombie;
that alone does not prove process exit. A real C pthread parent reproduced the
finding through the virtual daemon and public `ctl status.get` command. With
the syscall denied, the daemon stopped while the leader was Z with two threads.

Commit `6b83420` requires both Z and a thread count of exactly one. It parses
bytes after the final closing parenthesis, including when the process name
contains spaces, parentheses, or non-UTF-8 bytes. The count is field 20, after
the leader state in field 3. See the [field definitions](https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html)
and [kernel exit accounting](https://github.com/torvalds/linux/blob/v6.12/kernel/exit.c).

The fixed command passed 30 status calls over 3.3 seconds while the worker lived.
After the worker ended, the still-unreaped parent had Z/one thread; the daemon
exited zero and removed its socket. This passed with a pidfd observed in the
daemon's descriptors and with the kernel denial active and no pidfd present.
Both S20 scenarios passed again under each policy. Linux ARM64 locked release
build and strict daemon/all-targets Clippy passed. No product hook or mocked
function was used. Independent round 13 found no concrete introduced P1/P2 in
the scoped response. It was source inspection only. PID reuse, inaccessible
`/proc`, and snapshot races remain limits of the fallback.

## Mac regression and archive

Both builds use Rust 1.99.1 and minimum macOS 15.0. The Intel build runs through
Rosetta on ARM64 hardware. Both locked release builds and strict workspace,
all-targets Clippy passed. The universal app/daemon also compiled and its two
Mach-O slices declared minimum macOS 15.0. That is build evidence, not physical
Intel acceptance. The SDK emitted an x86_64 deprecation warning during the
universal build; the slice minima do not establish future hardware support.

| Virtual run at `4a65290` | Result |
| --- | --- |
| ARM64 | 88/88 passed; 30 Docker exclusions; zero cleanup failures |
| Intel/Rosetta full attempt | 87 passed; one environment failure; 30 exclusions; zero cleanup failures |
| Intel P07 with corrected port range | 1/1 passed; zero cleanup failures |

The Intel full attempt was assigned ten ports, but P07 declares thirteen nodes.
It failed before starting, with `no free port in configured range`. Its focused
replay used thirty assigned ports. The original failure is retained; this is
not reported as a clean 88/88 single run.

| S15 measurement | ARM64 | Intel/Rosetta |
| --- | ---: | ---: |
| Server/client peak RSS, bytes | 9,469,952 / 9,420,800 | 7,266,304 / 7,200,768 |
| Server/client CPU over 60 s | 0.049958% / 0.016657% | 0.083256% / 0.033302% |
| 1000 virtual moves, p50/p95/p99, ms | 0.797584 / 0.882584 / 0.936791 | 0.814625 / 0.904875 / 1.130250 |

The latency boundary includes external IPC injection and receiver ledger
observation on plaintext loopback. It is not physical input-to-screen latency.
The RSS gate remains 30 MB, CPU 0.1%, and p99 5 ms. No limit was relaxed.

Clean source `6b834202e712755222c8ef9418441874a5ecca5d` produced the local ARM64
archive below. Its daemon bytes equal the ARM64 full-run input. The rebuilt
Intel daemon, with the same minimum target and signing settings, also equals
the Intel full-run input. The Linux-only final fix did not change those bytes.

| Archive item | Value |
| --- | --- |
| ZIP bytes | 3,614,432 |
| App regular-file bytes | 6,324,472 across nine files |
| ZIP SHA-256 | `2b688809f55862ea1d12086a1424c3a024cbf7e79e90e5d28c7d972135fb8036` |
| ARM64 daemon SHA-256 | `d2895ed27b52f61dfd4f16581469b984a2e32354a67c94f3fd83996558733573` |
| Packaged app SHA-256 | `eef8600f4a505f05c4567aa68f23c3ecc09c4c39189abe87b713d6f96042e6ed` |
| Intel daemon SHA-256 | `758f5ba22c4541737e5568e4a2758910bcbc6ee6efa85701420bacc72f27c356` |

Packaging checked notice freshness, staged and freshly extracted daemon CLI/
virtual IPC, and the extracted code signature. BUILD.json equals the sidecar,
dirty=false, and README links to exact source. The freshly extracted app passed
25 headless flows and seven unsafe-startup refusals, including owned process
and socket cleanup. No rendering or native desktop was exercised. The archive
is local, ad-hoc signed, unnotarized, and unpublished.

## Remaining acceptance

The [late Windows/release review limits](late-review.md) remain open. Native
focus/input/clipboard, permissions, login and wake behavior, successful Windows
service restoration, confidential reporting, trusted distribution, and
physical Intel/x86_64 Linux artifact acceptance still need evidence. Older
sustained-workload results retain their older binary scope. These finite checks
do not prove universal stability, security, or absence of leaks.
