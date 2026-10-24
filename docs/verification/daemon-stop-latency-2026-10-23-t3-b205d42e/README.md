The runtime now leaves its main loop immediately after it sends the public
`daemon.stop` response. It then runs the complete existing shutdown sequence.
The only production change is the loop label and the stop check after
`reply.send(response)` in `crates/daemon/src/runtime.rs`.

This work starts at `d2ab17d4e053437d9f7568b0f6dbf96727bfd6c9` in the
T3-owned worktree `/Users/serge/.t3/worktrees/openergy/t3-b205d42e`.
The full source-only diagnosis README in `t3-d1ff2556` and commit
`a64b33dd53a9ba11f2a887086b2615ef949c80fc` were read before implementation.
The declared 397-file source set adds protocol inputs to that diagnosis's
387-file set. All baseline inputs match the root and `1fd5144` through the
root. [source-comparisons.json](raw/source-comparisons.json) and the per-build
source bindings retain the commands, full path inventories, sizes and hashes.
The source review is separate from the two new runtime observations below.

The source cause is the already-established ordering: IPC enqueues and wakes
before dispatch; stop dispatch sets the stop flag and returns the response;
the current iteration could then enter a normal blocking network poll before
the next loop guard. The new check observes the same stop atomic after the
response send and breaks the outer loop. A failed response send also proceeds
to shutdown if stop is set. It does not depend on a second IPC request or wake.
[runtime-fix.patch](runtime-fix.patch) is the exact production patch.

The check covers the existing shared stop flag, including IPC-server fatal
errors, if it is set at that point. It does not add a new signal flag or parent
watch read. Their existing loop-guard checks and wake behavior are unchanged.
No signal or parent-exit flow was run, so their runtime behavior is not newly
qualified. The runtime-owned public stop store is read on the same thread;
this repair does not need a stronger atomic ordering.

The shutdown tail is byte-identical to the root. It still performs leave and
goodbye, the full existing flush loop with its 250 ms elapsed-time threshold,
input release and cursor restoration, network shutdown, worker Stop commands,
and all three explicit joins. This threshold is not a hard process-exit bound.
No IPC budget, queue, cancellation, discovery, import, native backend, timeout
or rebind implementation changed. Requests queued after acknowledged stop no
longer get another normal-operation turn before this shutdown path.

Exactly one baseline flow ran before the runtime edit and one candidate flow
ran after it. Both passed. Each launched one real virtual daemon with role
Off, discovery disabled, TLS enabled, autoconnect disabled, an explicit private
config, a private TLS identity generated before daemon launch, an explicit
private IPC socket and the live collector as the parent. All fixture paths
were bound before launch. Both configured addresses were literal loopback:
`127.0.0.1:65181` for baseline and `127.0.0.1:65182` for candidate. Role Off did
not start a TCP listener or connect to that port.

The same frozen [collector](collect.py) and [CLI trace overlay](cli-trace-overlay.patch)
ran in both cases. Their recorded hashes match exactly. The only source
difference between the two built source sets is `runtime.rs`. The CLI overlay
adds two diagnostic records around the existing public CLI frame write/read.
It does not replace public callbacks, dispatch, transport, or runtime code.
The overlay was removed byte-for-byte after both flows. A separate final-source
release build passed without it; that binary was not launched. The tested
candidate binary is therefore bound to the final runtime source plus the
declared CLI-only overlay, not an uninstrumented runtime reproduction.

Each flow ran the real CLI `version`, generated the owned TLS fingerprint,
then called public `status.get`, `config.get`, `discovery.list`, an invalid
method, and exactly one `daemon.stop`. Status confirmed the exact owned PID,
virtual platform, role Off and TLS enabled. Config confirmed the literal
private addresses and disabled autoconnect. Discovery returned `[]`.
The invalid method returned exit 1 and `unknown_method`. It is an expected
failure-case result, not a retry. The stop CLI returned exit 0 and exactly
`{"stopping":true}\n`; the traced response envelope had the matching request ID.

The measured baseline request-to-reply interval was **0.199 ms**, and its
request-to-observed-exit interval was **1,254.038 ms**. The candidate intervals
were **0.120 ms** and **252.767 ms**. Reply-to-observed-exit was 1,253.839 ms
for baseline and 252.647 ms for candidate. These are the two collected samples.
They support the narrow change in this role-Off flow. They do not establish a
distribution, a general quantified speedup, a one-second exit contract, or a
connected-server upper bound. No repeat, fishing, retry or extra daemon flow ran.

Both stop timelines use the shared macOS `CLOCK_MONOTONIC` clock. Baseline
request/reply/CLI-end/observed-exit nanoseconds were respectively
`94785068197000`, `94785068396000`, `94785069590000`, `94786322235000`.
Candidate values were `94856085675000`, `94856085795000`, `94856086427000`,
`94856338442000`. The raw [baseline](raw/baseline-flow/stop-timing.json) and
[candidate](raw/candidate-flow/stop-timing.json) records contain request IDs,
full envelopes, launch start, deadlines, exit codes and timing definitions.

`request_start` is sampled immediately before the diagnostic emission and the
existing public CLI `write_frame`; `reply` is sampled after `read_frame` has
returned its decoded JSON. Those intervals include diagnostic overhead. They
are not kernel write/receive timestamps. A separate blocking `waitpid` thread
collects the daemon exit and timestamps its return. This is independent exit
evidence, not an inference from the response or daemon log. Its timestamp is
an observation after exit, with unbounded OS scheduling delay, rather than an
exact kernel exit instant. All values were checked in increasing order across
the CLI and observer processes.

A preflight caught Python 3.9's process-relative `time.monotonic_ns()` origin
before either daemon flow. [clock-preflight.json](clock-preflight.json) retains
that observation. The frozen flow and exec helpers use
`time.clock_gettime_ns(time.CLOCK_MONOTONIC)` instead. Preparation and the first
build's timing fields retain their original process-relative monotonic values;
they are used only within their own processes, never in the stop comparison.
This correction did not require a product run or retry.

The normal-exit deadline was five seconds from the external stop CLI launch,
which is earlier than its request-write trace. Both exact owned processes
exited 0 before that deadline. No SIGTERM, SIGKILL or other cleanup signal was
sent. The daemon PIDs/groups were baseline `52629` and candidate `60363`.
Their actual post-exec `ps` snapshots include UID, parent, group, nice 15,
executable path and start time: `Fri Oct 23 18:47:39 2026` and
`Fri Oct 23 18:48:50 2026`. The stop CLIs also reported actual nice 15 in their
in-process trace. Every product invocation has a same-PID exec-barrier receipt
after an absolute priority assignment, rather than a nested relative wrapper.

Both cleanup records prove collected exit 0, PID absence, process-group
absence and private IPC socket absence. The generated TLS directories were
removed, with only their file names, public certificate hash and key mode
retained. No private key bytes or hashes were archived. The IPC lock files are
ordinary retained private files, not sockets or live endpoints. Build groups
are also checked for absence by the archive validator. Private binaries and
build caches remain ignored under `target/sb`; they are not installed.

Each configured port had one first exclusive bind before launch and one after
cleanup, without `SO_REUSEADDR`, retries or a fallback port. All four binds
passed. Each observation also retains filtered TCP state and the port-scoped
`lsof` result. `lsof` emitted an unrelated mounted-filesystem warning, so its
empty output alone cannot establish complete listener visibility. The original
warning records remain byte-exact in the owned ignored private directory.
[port-warning-redactions.json](port-warning-redactions.json) records their
locations, original hashes and archived hashes. Only the unrelated private
mount identity was redacted in the committed copies; observations did not rerun.
Role Off provides no connected goodbye/handshake evidence and no TIME_WAIT
causation claim. The old 65202 bind failure and old timeout remain failures.

Builds used a fresh private Cargo home with a byte-copied registry, pinned
Rust **1.99.1**, `--locked --offline`, jobs **2**, deployment target **15.0**,
and a fresh private Cargo target cache. Candidate and final-source builds
reused only this task's cache. No old target or Swift state was used or touched.
The copied registry inventory retains source/copy byte hashes. Compiler exec
barriers set absolute NI 15, and live samples show the actual pinned `rustc`
processes at NI 15. The Cargo environment, commands, exits, compiler records,
source bindings and executable hashes remain under `raw/*-build`.

All three release builds exited 0. Each emitted the same toolchain warning:
`stripping debug info with rust-objcopy failed: signal: 6 (SIGABRT)` because
its `libLLVM.dylib` could not load. The complete first warning and subsequent
warnings are preserved in their raw build stderr files. The builds produced
runnable artifacts, but successful stripping is not claimed. The toolchain
was not changed, and no build was retried. Pinned rustfmt also passed.

The tested baseline executable SHA-256 is
`1e4ea648bcada9fafc2a4ac0a59d723d04dbb6c9ee4cb71898d9b7b04a1e2f02`.
The tested candidate SHA-256 is
`4ac294f7be9a1512c10537944f2ce1ffc257a62c9c34fb5c69c9cdf75b74a2ab`.
The final uninstrumented source build has its separate hash receipt. No
signing, packaging, installed-app, full release, hardware, native input,
connected TLS or Windows runtime acceptance is claimed.

The finite commands were, in order:

```sh
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py prepare
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py build baseline
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py flow baseline
# Apply runtime-fix.patch; keep the same CLI overlay and collector.
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py build candidate
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py flow candidate
# Restore only main.rs from its saved exact root bytes.
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/collect.py build final-source
python3 docs/verification/daemon-stop-latency-2026-10-23-t3-b205d42e/validate.py
```

Each collection command's stdout and stderr are separate retained files next
to this README. Exclusive directories prevent accidental reruns. No product
flow failed, timed out or needed a signal. Read-only exploratory path errors
are kept in [reader-errors.txt](reader-errors.txt). The expected invalid-method
responses, build stripping failures, port warnings and all raw flow receipts
remain visible; none was replaced with a passing retry.

The archive validator checks every manifest byte and hash. The final index
check requires every owned file to be staged with its exact on-disk bytes,
with no unrelated staged or untracked path. Whitespace checks apply to the
maintained source, Python collectors and this README. Exact patch artifacts
and raw compiler stderr retain their original whitespace, including blank
lines with spaces from the stripping warning.

The diagnosing-bugs and macOS build-run-debug skills were read. The explicit
two-flow limit and prohibition on tests/mocks govern this bounded Rust CLI
task. No Swift/UI bootstrap, unit test, mock or desktop interaction was needed.
All writes were confined to this worktree. No production config/TLS/default
IPC, live sharing, installed app, TCC, login, service, task, account, firewall,
global DNS, toolchain default, Docker, code-signing key, push or publication action
ran. No daemon fixture used TCP 24800, UDP 5353 or a non-loopback address.
The three provider-stopped tasks were not resumed or reframed. The saved r3
70.532 ms / 256 ms wall-clock evidence remains audited historical evidence,
not an independent reproduction here and not proof of a sole historical cause
or product P2. No further delegation ran.
