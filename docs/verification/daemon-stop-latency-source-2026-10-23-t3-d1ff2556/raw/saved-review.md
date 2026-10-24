**Independent source and saved-evidence review: bounded no findings**

Reviewed commit `b75dfa1b543e4bf6b47858581d4d2cf19aa777e6`, whose direct parent is
`71a40b7c2a5cf22cd0bf18a8b602f9ead51de681`. No concrete P1/P2 finding was
established within this review. The saved overall result remains **FAILED**:
the public peer stop exceeded the controller's one-second wait, and the first
exclusive rebind of its port failed. The twelve passing app checks do not make
the controller run or strict cleanup pass.

This review used Git blobs, saved JSON/text, in-memory patch reconstruction,
hash comparisons, and local Apple SDK headers. It did not import or execute the
controller, build software, launch an app or daemon, inspect live processes,
query the timebase, make IPC/network/port calls, drive a desktop, or read keys,
tokens, secrets, or environment dumps. No unit tests, mocks, or delegation ran.
Only this unique review document is a write target. The three provider-stopped
tasks remain outside scope. All runtime statements below describe audited saved
evidence; none is a fresh reproduction.

Paths below are relative to
`b75dfa1:docs/verification/macos-idle-resources-r3-2026-10-23/` unless a commit
or source path is stated. This report's checkout does not contain the candidate
directory; the candidate was reviewed directly from its Git object.

The archive and source bindings are consistent within their stated scope:

- The full README and manifest were read. The candidate adds 63 files only in
  its private evidence directory. All 62 manifest entries match both SHA-256
  and byte length. The sole unlisted file is the manifest itself.
- All 43 entries in `raw-records.json` match their committed bytes, sizes, and
  `transform: none`. Each original path maps exactly from `raw/` to the unique
  `/private/tmp/oe-idle-r3-f553ae64-nnze4977/` fixture. These are archive checks;
  this review did not reopen the original temporary trees or binaries.
- All 456 `raw/preparation.json` source hashes match parent `71a40b7`. The
  reference-results hash also matches. Its two declared differences are the
  whole-app harness and standalone `dns_queue_cancel_c4806a3d.rs` example.
  This checks the declared input set, rather than independently reconstructing
  a Rust binary's complete build provenance.
- Applying the addition-only harness overlay in memory to the parent yields
  SHA-256 `ba756646d69b20be2a0cf80fa5663fd3d5da9929da1207eedc09c6047705c056`,
  matching the copied harness and archive receipt. The other 25 copied app
  inputs match the parent exactly. Production classes and cache logic are not
  patched by this candidate.
- The archived controller hashes to
  `170e07ed165606e07b9823677a1cdadea59fe25b33e146d8d01685d0c50150c6`.
  The overlay hashes to
  `d2db3e4f7ae6a0c41d7d5d03cf30b1be13d6f8af9bc0931a3d4a21fcc9e55cb2`.
  Both match readiness, pre-runtime gates, runtime entry, and results; the
  overlay also matches preparation. `controller-repair.patch` is exactly the zero-context diff from
  the r2 controller at `907ab5b` to this controller. Python syntax was parsed
  without loading its module or calling libproc.
- Saved readiness binds daemon SHA-256
  `350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572` and signed
  private app SHA-256
  `1f9351253b0d5c6649da3ea4d9cf8b9faa0fc4b259608e88b9170cf25e86f8f7`.
  The compiled executable has a different recorded hash before the private
  ad-hoc signing step. Binary bytes are not part of this archive, so this
  review verifies the recorded binding, not the binaries themselves.

Historical preservation was checked against the named Git objects. All 18 r1
file hashes match `e31213ef8edf4979a2487a692114071b97c4c200`; those files also
remain byte-exact in the candidate tree. All 51 r2 hashes match
`907ab5bdcb13da846c4b171cd90e40da77487807`. The r2 directory is not included in
the candidate tree; its binding is to that separate historical commit.
`reconciliation.json` records 1,257 and 1,273 historical private-file hashes
(2,530 total), eight absent prior PIDs, five absent groups, and both terminal
run-directory guards. `raw/postflight.json` reports no historical mismatch and
768 unchanged dependency-cache files. These before/after private-file and cache
claims were audited as saved receipts, not remeasured now.

Both historical failures retain their exact limits. R1 sampled the app while
its executable was still `/usr/bin/nice`, before readiness, and failed. R2
accepted the final executables and nice 15, then the app exited 1 at its full
readiness check. Its OS-sample archive is empty. The `connected/peers=1` detail
does not prove its other readiness conditions. The r3 source establishes that
the configured-name comparison could not pass: at parent
`crates/session/src/client.rs:94`, `ServerConnected` creates a peer named
`server`. R3 keeps that role-label check and moves the configured server-name
binding to real private server IPC. Neither old attempt becomes a baseline.

The saved new build is one whole-app release build, exit 0, elapsed
27.84935275 seconds. Its arguments use two Swift jobs and disable automatic
resolution/update, with private source, scratch, cache, config and security
paths. Saved environment metadata sets deployment target 15.0, Rust 1.99.1,
jobs 2, and Cargo offline. The saved Rust version check returns 1.99.1; the
daemon was copied and no Rust rebuild is claimed. There are 51 build compiler
inventories, with three to five compiler processes globally. The build driver
45018 and recorded owned compiler descendants show nice 15. That is evidence
for the observed processes, not a universal compiler-priority guarantee.

Runtime ownership and connection checks bind these exact identities:

| Role | PID / process group | Parent PID | Executable under the private root |
| --- | --- | --- | --- |
| App | 51094 / 51094 | 51066 | `Openergy.app/Contents/MacOS/Openergy` |
| App daemon | 51096 / 51096 | 51094 | `openergy-daemon` |
| Server peer | 51089 / 51089 | 51066 | `openergy-daemon` |

All 35 sampled identities have UID 501, effective nice 15 from both `ps` and
`getpriority`, and stable executable, parent, group, start-time string and
`proc_start_abstime`. Both controllers record an absolute priority change from
0 to 15, with no `nice` wrappers. All three exec barriers accepted the exact
executables on their first observation; the transient-launcher branch was not
exercised in this saved r3 flow.

The peer is `idle-peer-f553ae64-r3`, PID 51089, listening only on
`127.0.0.1:65202`. The app's separate configured server address is
`127.0.0.1:65203`; it runs as a client here. Both daemons report platform
`virtual`. The controller writes the peer config before launching it. The
parent harness creates its safe off-role config before `model.start()` can
spawn the app daemon. Explicit private config/TLS/Unix-socket paths, discovery
disablement, and the virtual adapter keep this flow separate from production
defaults. The saved readiness socket inventory shows only the owned loopback
listener and connection, with no TCP 24800 or UDP 5353 entry.

The exact approved server fingerprint is
`B1:D3:58:C0:94:3E:EA:0A:7B:29:44:9C:54:A2:19:70:39:2B:55:EC:91:E8:BD:3A:39:5E:94:AA:81:7B:8D:E7`.
Before idle and after rejoin, the independent private IPC records each include
status/config/trust responses from both endpoints, plus the app snapshot.
All 32 recorded controller conditions are true in each record. Separate
evaluation of the raw identity/TLS/config/trust fields agrees: configured and
reported peer name, exact address, supervised daemon PID, platform, roles,
enabled TLS, server fingerprint, app trust membership, and empty pending/error
state match. The reverse peer fingerprint matches the app daemon's identity.
The fixture server sets `require_client_trust=false`; its client peer is
reported `trusted=false`. This proves the intended client trust of the server
and the reverse fingerprint binding, not mutual trust approval.

All twelve saved app checks pass, and the app exits 0. Ten measurement-specific
check snapshots each retain twenty healthy-condition values. Some conditions
are correctly false during TLS setup, pending trust, and deliberate
disconnect; the check-specific predicates, rather than all twenty values,
define those steps. The failure-only app diagnostic branch compiled according
to the saved build but was not exercised by a failed app check.

The app event interval is exactly 31.519038541 seconds from `steady_begin` to
`steady_end`, for a requested 30-second sleep. The peer record, including
`sinceUnixMs=1792803855932`, is unchanged at these endpoints. Config, trust,
supervised daemon PID and public peer identity also match. Normal production
session heartbeats/event processing remain active; there is no harness callback
injection or repeated request during the sleep. The app trace has 41 requests;
the controller has 19 CLI attempts, including three failed diagnostics after
shutdown, for a conservative 60/128. Method counts agree with
`raw/request-accounting.json`: one trust approval, two connects, one disconnect.
There is no app request between the steady markers, and the controller source
makes no IPC call in its steady sampling branch. The public disconnect reaches
idle with no peers, and one rejoin obtains the same trusted server identity.

The 35 OS batches comprise one trusted-ready batch, 33 steady batches
(`steady_begin`, 31 periodic samples, `steady_end`), and one rejoined batch.
Their reads are sequential, not simultaneous. Steady batch-start gaps range
from 0.533826916 to 1.021502667 seconds, including the separately triggered end
batch. Batch durations range from 0.042849708 to 0.339472125 seconds. The
controller's steady batch-start bracket is 31.53494925 seconds; it is not the
same timestamp boundary as the app's event interval.

Exact RSS ranges recomputed from all 33 steady batches are:

| Process | RSS bytes, minimum–maximum | RSS MiB, minimum–maximum | Threads | FDs | Saved OS state |
| --- | --- | --- | --- | --- | --- |
| App 51094 | 13,369,344–13,451,264 | 12.75–12.828125 | 3–6 | 5 | `SNs` |
| App daemon 51096 | 10,584,064–10,633,216 | 10.09375–10.140625 | 5 | 13 | `SN` |
| Peer 51089 | 10,502,144–10,567,680 | 10.015625–10.078125 | 5 | 13 | `SNs` |

For all 35 batches, including readiness and rejoin, the RSS ranges are
13,287,424–13,631,488 bytes for the app, 10,584,064–10,715,136 for its daemon,
and 10,485,760–10,665,984 for the peer. The per-batch sum during steady is
34,471,936–34,652,160 bytes (32.875–33.046875 MiB). This sum can count shared
resident pages more than once and uses sequential readings; it is not a unique
physical-memory total. These are finite sampled ranges, not lifetime peaks,
resource bounds, leak exclusions, or normal rendered UI memory.

RSS has a direct primary unit basis. In the local Apple SDK,
`usr/include/sys/proc_info.h:124-142` defines the sampler's `proc_taskinfo`
layout and labels `pti_resident_size` as resident memory size in **bytes**.
Lines 717 and 726 define LISTFDS=1 and TASKINFO=4; lines 672-675 define the FD
record. `sys/resource.h:202-214` matches the rusage v0 field order and widths.
For every saved process reading, the archived RSS equals both
`task_info.resident_size` and `rusage.resident_size`. MiB above means bytes
divided by 1,048,576. The sampler's thread and FD counts also match the saved
task fields and FD-array lengths.

CPU needs a narrower conclusion. The raw `cpu_cumulative_ns` field equals
`rusage.user_time + rusage.system_time` in every reading and remains monotonic.
Its numeric values were preserved despite the incorrect inherited unit label.
The steady sample endpoints and exact arithmetic are:

| Process | Raw cumulative begin → end | Raw delta | Conditional derived nanoseconds, raw delta × 125/3 |
| --- | --- | --- | --- |
| App | 606,276 → 742,938 | 136,662 | 5,694,250 |
| App daemon | 284,923 → 635,372 | 350,449 | 43,806,125 / 3 |
| Peer | 225,894 → 382,009 | 156,115 | 19,514,375 / 3 |

The rational arithmetic in `results.json` is correct for the saved post-run
timebase ratio 125/3. However, the local SDK's `rusage_info_v0` declaration does
not specify CPU-time units. `counter-unit-annotation.json` cites Apple's
upstream `fill_task_rusage -> task_power_info_locked -> rm_time_mach` route,
but contains links and an account of the inspection, not the Apple code bytes
or a pinned source revision. No network access was authorized for this review,
and no local copy of that implementation was read. Thus this review does not
independently prove that the deployed kernel returned Mach ticks through this
API. The derived nanoseconds above are **conditional**, not accepted as proven
CPU measurements. Neither the post-run ratio nor integer field widths establish
the deployed counter's effective resolution or kernel version. Keep the raw
counters authoritative until the primary implementation route is established.

Two unrelated Swift processes, PIDs 23475 and 44785, occur at nice 0 in every
one of the 35 runtime compiler inventories. The compiler's cumulative CPU time
changes during the run. Neither the saved data nor this review supports a
CPU-idle, zero-CPU, optimization, speedup, native UI, or full-release claim.
`OPENERGY_E2E_NO_RENDER=1` bypasses the normal SwiftUI app scene; the measured
app process includes the real AppModel/IPC/supervisor and private harness.

The failures and cleanup stay separate from those successful app checks:

- `raw/runtime-first-failure.txt` records `peer.wait(timeout=1)` at controller
  line 571 after `daemon.stop` returned success. The three later peer reads
  fail and remain in the CLI receipts and `failure-readbacks.json`. They are
  diagnostic attempts, not a successful shutdown or a repeated flow.
- At parent `crates/daemon/src/runtime.rs:1239`, `daemon.stop` sets the stop
  flag and returns `{"stopping":true}`. The current iteration can continue to
  its poll, whose timeout is clamped to at most 2,000 ms at line 1634. Shutdown
  then has a 250 ms flush and worker joins at lines 1641-1660. The reply has no
  one-second process-exit guarantee. This explains why the private oracle is
  stricter than that source contract; it does not establish the actual reason
  for this timeout or turn it into a product P2.
- The controller records one SIGTERM to owned peer group 51089 and no SIGKILL.
  The peer and app return 0. All three PIDs and groups are recorded absent;
  both private IPC paths are absent. The app daemon's separate exit code was
  not collected. Source `stop_group` at lines 279-294 skips an already-exited
  direct child before signalling its group, unlike r2. It can therefore skip
  that group's signal if descendants remain. The independent group/PID checks
  still prevent a cleanup pass in that case; all groups are absent here.
  This saved attempt does not prove general orphan-group recovery.
- The first exclusive bind of peer port 65202 fails with Errno 48 even though
  `lsof` returns exit 1 and empty stdout/stderr. Its filtered contemporaneous
  `netstat` receipt has two matching TIME_WAIT rows. These records do not prove
  causation. Port 65203 passes its first exclusive bind. There is one bind
  attempt per port, no retry, and cleanup remains failed.
- Metadata-only TLS inventories record removal of both generated TLS trees.
  The cleanup elapsed time is 37.906356208999995 seconds, within 60 seconds.
  The controller exits 1. None of the absence/removal receipts erase its first
  shutdown failure or failed strict port-release predicate.

The candidate also preserves its read-only archive-validation failure (wrong
relative-path base), followed by the corrected archive check, and its failed
historical r1 priority-receipt lookup. This review had two read-only extraction
errors: requesting the separate r2 directory from the candidate gave
`fatal: path ... exists on disk, but not in 'b75dfa1'`; looking for
`since_unix_ms` in an app-encoded snapshot gave `KeyError: 'since_unix_ms'`.
The corrected checks used r2's named commit and the app's `sinceUnixMs` field.
These were evidence-reader corrections only. No fixture, build, runtime, bind,
or failed saved transcript was retried, replaced, or edited.

Local SDK references used the prefix
`/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/`.
The inspected header SHA-256 values are
`e427fa96b348537b21552b9de71e01039410bcad2cedee5584c5e5fddafd70fc`
for `usr/include/sys/proc_info.h`,
`1067178025c5fd88c9a1b8206a07c2695ef94dd471fe2783f06c512995382961`
for `usr/include/sys/resource.h`, and
`d690e304ea595ca216ed51e20aa9747e0b3bcb07446d996abe0080a231c7065f`
for `usr/include/mach/mach_time.h`. These establish which local declarations
were read, not deployed-kernel provenance.
