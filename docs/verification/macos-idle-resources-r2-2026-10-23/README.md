# Launcher repair and one new attempt: idle baseline UNVERIFIED

The controller launcher repair is implemented. The one new runtime attempt
passed exact executable, UID, parent, group, and effective nice 15 checks for
the owned app, its daemon, and the peer. It then failed the app harness's full
trusted-readiness predicate. It produced **no 30-second baseline and no app
resource samples**. No runtime retry or cleanup rebind retry ran.

The app exited 1. Its report records `state=connected peers=1` at the failed
predicate. TLS establishment appears in both owned daemon logs. These facts do
not establish the full readiness contract. The harness did not save a final
status/config/trust snapshot on that failure, so the failed subcondition is
unknown. No Openergy defect is inferred from this record.

## Source and launcher repair

The source parent is `5a9783738c5b649844805e1614da7ea37cfbf2bb`, in the isolated
checkout `/Users/serge/.t3/worktrees/openergy/t3-c57ae290`. The supplied root
checkout `/Users/serge/.t3/worktrees/openergy/t3-52422617` was never a write
target. Only this new evidence directory is committed.

The inherited failure is commit `e31213ef8edf4979a2487a692114071b97c4c200`,
directory `docs/verification/macos-idle-resources-2026-10-23`. Its controller,
first failure, exit evidence, and empty OS samples remain unchanged and
**UNVERIFIED**. Its successful build and 0.864-second cleanup are inherited
evidence only. They are not this run's results. [before.json](before.json)
records its exact file hashes and the old private fixture's hashes;
[raw/postflight.json](raw/postflight.json) records no changes.

[measure.py](measure.py) replaces nested relative `nice` increments with one
absolute `os.setpriority(PRIO_PROCESS, 0, 15)` on each controller invocation.
It records priority before and after the change. Children launch directly and
inherit that priority. A denied change stops the controller without escalation.
The new launcher barrier observes only its owned PID for at most three seconds.
It allows `/usr/bin/nice` as an unaccepted transient, then requires the exact
private artifact path, UID, parent, group, and observed nice 15. Each accepted
identity includes OS process start time. All three launches in this attempt
already showed their final executables on the first observation. The transient
branch was source-reviewed; this run did not exercise it.

Resource sampling now starts only after all three identities and the real
trusted connection readbacks pass. The initial startup sample that caused the
old race is gone. [controller-repair.patch](controller-repair.patch) records
the full controller change. The executed controller is retained without a
post-failure repair. Its SHA-256 is
`3ee701ac3f02b9013ca9d5a556dfc7c705eb7c143667ee984cee3818982112b6`.

## Private build and input bridge

The old reference's 456 source inputs differ from this parent only in
`AppEndToEnd.swift`, which now includes the separate caller-cancellation mode.
A new whole-app release build therefore used the current complete harness
plus [harness-overlay.patch](harness-overlay.patch). The overlay carries the
same finite idle method as the old attempt and preserves the current harness's
other modes. No production class changed. All other copied app inputs match
the current checkout byte for byte. The daemon was copied, with all its source
and dependency inputs unchanged; no Rust rebuild ran.

The first build passed: 36.814860 seconds measured by the controller, with
`Build complete! (33.20 secs.)` from Swift. It used two jobs, deployment target
15.0, Swift 6.4, and the exact copied Sparkle dependency cache with automatic
resolution disabled. Rust version verification returned 1.99.1. No dependency
or toolchain default was installed or changed. The private app bundle received
an ad-hoc signature; no signing key was used.

The private root is `/private/tmp/oe-idle-r2-c57ae290-qypkpvud`. The signed app's
SHA-256 is `3f7e1c10c3f81125d75b6ddc26d463f5a5e20c93c5a2c50f0da743e00d3d353a`.
The daemon's SHA-256 is
`350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
The overlay's SHA-256 is
`d053fa6e8e993d53321e584b5a55216e400050f0577341a7ee34aed9e2e46c24`.
[raw/preparation.json](raw/preparation.json), [raw/build.json](raw/build.json),
and [raw/ready.json](raw/ready.json) bind the build inputs and artifacts.
All 768 hashed historical dependency-cache files remained unchanged.

Build compiler inventories are separate in
[raw/build-compiler-samples.json](raw/build-compiler-samples.json), at a
0.5-second target interval. The owned build driver and its recorded direct
compiler children show nice 15. The all-compiler inventory also includes nice 0
processes whose complete ancestor chains were not captured. This record makes
no blanket priority claim for every compiler process. The before/after build
inventories are empty. Runtime ended before the readiness-gated compiler and
resource sampler ran; compiler activity during runtime is **UNMEASURED**.

## The single new runtime result

The runtime controller PID was 46926, UID 501, observed nice 0 before the
absolute change and nice 15 afterward. The accepted owned identities were:

- Peer PID/group 46929, parent 46926, exact private daemon path, nice 15.
- App PID/group 46932, parent 46926, exact private app path, nice 15.
- App daemon PID/group 46934, parent 46932, exact private daemon path, nice 15.

[raw/exec-barriers.json](raw/exec-barriers.json) includes `ps` states and both
`ps`/`getpriority` observations. The peer used literal `127.0.0.1:55021`; the
app's private host config used `127.0.0.1:55022`. Both came from fresh exclusive
ephemeral reservations. Private config existed before each daemon started.
Discovery was disabled and both daemons used the virtual platform. Inherited
`OPENERGY_`/`XDG_` settings were excluded before explicit fixture settings were
applied. Only those explicit settings were recorded.

The real AppModel connected to its supervised daemon, verified its safe Off
config, enabled private TLS, matched the peer's public pending fingerprint,
and sent one public approval. The next full readiness check failed after
3368.087667 ms from the idle branch's start. The app never emitted `ready`,
`steady_begin`, `steady_end`, `rejoined`, or `end`.
[raw/run/flow/report.json](raw/run/flow/report.json) and
[raw/run/flow/idle.jsonl](raw/run/flow/idle.jsonl) are the unmodified records.

The run made 22 traced app requests and one peer CLI readiness request: **23 of
the 128-request limit**. No public disconnect/rejoin or final readback ran.
The request and first failure records are
[raw/request-accounting.json](raw/request-accounting.json),
[raw/runtime-first-failure.txt](raw/runtime-first-failure.txt), and
[runtime.stderr](runtime.stderr). The latter also retains the cleanup assertion
that followed the first app failure.

The OS sampler's target was one second, with a batch-start monotonic timestamp,
Unix timestamp, sequential per-process reads, and recorded batch duration.
CPU counters would use libproc rusage v0 integer nanoseconds; their effective
OS resolution was not established. `ps` cumulative time displays hundredths of
a second. The recorded monotonic clock resolution is
0.00000004166666666666667 seconds. There are **zero** resource samples:
[raw/os-samples.json](raw/os-samples.json) is exactly `[]` plus a newline.
CPU, RSS, threads, FDs, and a one-second process-state series are UNVERIFIED.
No CPU-idle, resource-bound, memory-bound, or speedup claim follows.

## Cleanup and evidence limits

Owned process cleanup reached 4.591248417 seconds from runtime entry. All three
tracked PIDs and groups were gone. Both private IPC socket paths were absent.
The app's normal shutdown logged goodbye and daemon stop. The controller sent
SIGTERM to its owned peer group, which exited 0; no SIGKILL was observed.
The app daemon's separate exit code was not collected.

The first exclusive rebind of peer port 55021 failed with
`OSError: [Errno 48] Address already in use`. The executed controller replaced
that port's preceding `lsof` result with the bind-failure record. Its `lsof`
stdout and exit are therefore unavailable. The reason for the bind failure is
not established; this record does not label it TIME_WAIT or claim the port was
released. Port 55022 had empty `lsof` output/exit 1 and a successful fresh
exclusive bind. **Strict cleanup is failed**, and the peer-port release gate
remains UNVERIFIED. No later probe replaces this first result.

Both new private TLS directories were removed after metadata-only inventories
were saved. No key content or artifact binary is archived.
[raw/tls-removal.json](raw/tls-removal.json) records the removals.
[raw/cleanup.json](raw/cleanup.json) retains the failed cleanup result at
4.734243583 seconds, within the 60-second runtime limit.

[raw-records.json](raw-records.json) maps every copied raw file to its original
path and SHA-256, with no transformation. [results.json](results.json) binds
the source, controller, artifacts, outcomes, and open gates. The first local
overlay-extraction assertion also remains in
[setup-first-failure.json](setup-first-failure.json); it preceded all builds
and runtime. Syntax parsing, source review, and archive-integrity checks are
separate from the failed actual flow.

The commands executed once each were:

```sh
python3 docs/verification/macos-idle-resources-r2-2026-10-23/measure.py
python3 docs/verification/macos-idle-resources-r2-2026-10-23/measure.py --measure /private/tmp/oe-idle-r2-c57ae290-qypkpvud
```

No unit test, mock, desktop/UI automation, default IPC, installed app,
production config/TLS, TCC, login, service, account, firewall, DNS, forbidden
port fixture, Docker, global dependency, push, publication, or Cloudflare
change was used. The three provider-stopped workstreams were not resumed.

After the parent pause/resume, [resume-reconciliation.json](resume-reconciliation.json)
matched the completed build/runtime exec receipts and the exclusive attempt
guard. It found all six known controller/build/runtime PIDs and all three owned
runtime groups absent. It sent no signal and ran no build, runtime, or port
rebind probe. The first failed cleanup receipt remains unchanged.
