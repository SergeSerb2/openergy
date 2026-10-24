# Private verifier repaired; app flow passed; overall run FAILED

This r3 changes only the private controller and an `AppEndToEnd.swift` overlay.
The source parent is `71a40b7c2a5cf22cd0bf18a8b602f9ead51de681`.
One new whole-app build passed. One new runtime completed all 12 app checks,
including trusted readiness, a connected idle interval, one public disconnect,
one trusted rejoin, and exact public readback. The controller exited 1:
normal peer shutdown exceeded its one-second wait, then strict cleanup failed
the first exclusive peer-port rebind. No build, runtime, or bind retry ran.
This is not full release acceptance.

The prior failures remain failures: `e31213ef8edf4979a2487a692114071b97c4c200`
and `907ab5bdcb13da846c4b171cd90e40da77487807`. Their archived files and both
historical private fixture trees were hashed before and after this work.
[reconciliation.json](reconciliation.json) and [raw/postflight.json](raw/postflight.json)
record no byte changes. Before the build and again before runtime, the eight
known prior PIDs were absent. The five prior owned groups were also absent at
reconciliation. Their exclusive `run` directories remain terminal guards.
No previous case was repeated, signalled, or given a new port probe.

## Verifier repair

The client's peer name is a role label. `crates/session/src/client.rs` sets it
to `server` on `ServerConnected`. The r2 harness and controller incorrectly
required the owned server's configured name at that location. This is an
impossible predicate; the saved r2 `connected/peers=1` record does not establish
that its other predicates passed.

[harness-overlay.patch](harness-overlay.patch) checks the `server` role label.
It keeps every prior readiness condition and adds explicit config/status TLS,
remote-address, and nonempty supervised-PID checks. Every check now records the
model status/config/trust snapshot and all 20 healthy-condition booleans.
A failed app check makes bounded real `status.get`, `config.get`, and
`trust.list` calls on the two explicit private sockets before shutdown.
That failure-only app diagnostic branch compiled but was not exercised here.

[measure.py](measure.py) independently reads both real private IPC endpoints
before idle and after rejoin. Each record contains full raw status/config/trust
responses, the app event snapshot, and 32 named controller predicates. It binds
`config.name` and `status.name` to `idle-peer-f553ae64-r3`, the owned server PID,
its virtual platform and exact literal listener, and the exact TLS fingerprint.
The app's peer address/fingerprint, TLS enabled/trusted flags, trust-list
membership, empty pending/error state, supervised PID, and reverse client TLS
identity must also match. The configured-name identity check was preserved on
the server's real IPC, not discarded. See
[before idle](raw/peer-connected-before-idle.json) and
[after rejoin](raw/peer-connected-after-rejoin.json).

The r2 exec/readiness barrier remains. Only an exact private executable with
UID 501, expected parent/group, OS start identity, and observed nice 15 is
accepted. Sampling starts after both the launch and trusted-readiness gates.
The controller uses one absolute `setpriority(..., 15)`; children launch without
`nice` wrappers. [controller-repair.patch](controller-repair.patch) shows the
changes from the immutable r2 controller. The executed r3 controller and overlay
were frozen before the successful build and were not edited after runtime.

## Build and single flow

The fresh root is `/private/tmp/oe-idle-r3-f553ae64-nnze4977`. Its whole-app
release build passed in 27.849353 seconds, with two Swift jobs, deployment target
15.0, private module caches, pinned Sparkle 2.10.0 cache copies, and automatic
resolution disabled. Rust verification returned 1.99.1. The daemon was copied;
its production inputs match the reference artifact. Only standalone net
examples differ in Rust source since that reference. No Rust rebuild ran.
The current full app harness was built with this private overlay; all other
app inputs match the source parent. All 768 historical dependency-cache files
remained unchanged. Signing used only a private ad-hoc signature.
[preparation](raw/preparation.json), [build](raw/build.json), and
[ready artifacts](raw/ready.json) contain the input and artifact hashes.

The build driver PID 45018 and the observed owned compiler descendants all
showed nice 15. The global build compiler inventory ranged from three to five
processes. Two unrelated Swift processes, at nice 0, also appeared in all 35
runtime samples. This record makes no blanket claim about unobserved compiler
processes, and no CPU-idle claim.

The accepted runtime identities were peer PID/group 51089, app PID/group 51094,
and supervised app-daemon PID/group 51096. All showed UID 501, stable start
identities/private paths and effective nice 15. Both daemons used the virtual
platform and discovery was off. Config files existed before their daemons
started. The two exclusively reserved ephemeral ports were 65202 and 65203.
The app joined only `127.0.0.1:65202`. All public control paths used private Unix
sockets. No normal disconnect, rejoin, or teardown selected an unowned address.

The app's sleep requested 30 seconds. Its actual `steady_begin` to `steady_end`
interval was 31.519039 seconds. OS sampling retained 35 batches, including 33
from steady begin through steady end. The target period was one second;
observed batch-start gaps were 0.533827 to 1.021503 seconds, including the
separately triggered end sample. Each batch records its duration and sequential
per-PID RSS, cumulative CPU counters, threads, FDs, OS state, identity and nice.
See [raw/os-samples.json](raw/os-samples.json) and [results.json](results.json).
All 12 app checks passed and the app exited 0.

There were 41 traced app requests plus 19 CLI attempts, conservatively 60 of 128.
The latter includes three failed diagnostic attempts after the peer stop timeout.
No app or controller IPC request ran during steady idle. There was one public
trust approval, two public connection requests and one public disconnect.
[raw/request-accounting.json](raw/request-accounting.json) retains the counts.

The inherited sampler incorrectly labels raw Mach CPU ticks as nanoseconds.
The executed controller and raw records retain that label byte for byte.
[counter-unit-annotation.json](counter-unit-annotation.json) records the
post-run read-only OS timebase ratio, 125/3, and the Apple source route.
`results.json` uses explicit raw-tick fields and rational derived-nanosecond
values. No CPU percentage, bound, speedup, or zero-CPU conclusion is asserted.
The counter route is supported by Apple's
[task accounting source](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/kern/task.c#L6387-L6392)
and [rusage source](https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/kern/bsd_kern.c).
The exact deployed kernel source version and effective counter resolution were
not established. This remains a finite OS-counter baseline with compiler overlap.

## First failures and cleanup

The first runtime failure was `peer.wait(timeout=1)` after the public
`daemon.stop` response. It remains in
[raw/runtime-first-failure.txt](raw/runtime-first-failure.txt). The subsequent
three private peer diagnostic reads also failed; their stdout/stderr/exit
receipts remain in [raw/public-cli.json](raw/public-cli.json). They are not
retries of shutdown or the flow. Cleanup sent one SIGTERM to the owned peer
group; no SIGKILL ran. The peer ultimately exited 0. Its normal-stop timeout
remains a failure. The app daemon's separate exit code was not collected.

[raw/cleanup.json](raw/cleanup.json) records all three PIDs and groups absent,
both IPC socket paths absent, and both generated TLS directories removed after
metadata-only inventories. No TLS key bytes were read or archived. Total runtime
through cleanup was 37.906356 seconds, below the 60-second limit.

For port 65202, `lsof` returned exit 1 with empty stdout/stderr, but the first
fresh exclusive bind failed with `Errno 48`. Both results now survive in separate
fields. A filtered contemporaneous `netstat` record contains two TIME_WAIT rows
with that exact loopback endpoint. These are OS state observations; they do not
establish the bind failure's cause. Port 65203 had empty `lsof`/exit 1 and its
first exclusive bind succeeded. The strict immediate exclusive-rebind predicate
was not relaxed. Cleanup remains FAILED. No later bind probe replaces it.

The controller also records the actual signals sent. Its `stop_group` helper
skips an already-exited direct child before signalling its group; this differs
from r2's unconditional group probe. Both remaining owned groups and the separate
daemon PID are still checked. This private cleanup-control difference is
explicitly retained for parent review; it is not a production repair.

[raw-records.json](raw-records.json) binds 43 byte-exact raw files to their
original paths and SHA-256 values. [evidence-manifest.json](evidence-manifest.json)
binds the complete archive. A read-only archive check first used the wrong
relative-path base; [archive-validation-first-failure.json](archive-validation-first-failure.json)
preserves that failure. Removing its unused comprehension let the corrected
archive check pass, without another build or runtime. The commands ran once each:

```sh
python3 docs/verification/macos-idle-resources-r3-2026-10-23/measure.py
python3 docs/verification/macos-idle-resources-r3-2026-10-23/measure.py --measure /private/tmp/oe-idle-r3-f553ae64-nnze4977
```

No production logic, installed app, live sharing, production config/TLS/default
IPC, TCC, login, service, account, firewall, DNS or toolchain default changed.
No TCP 24800, UDP 5353, excluded fixture port, desktop driving, unit test, mock,
Docker, global dependency, bootstrap, delegation, push, publication or Cloudflare
action ran. The three provider-stopped tasks were not resumed. The parent still
needs to review this source and evidence independently.
