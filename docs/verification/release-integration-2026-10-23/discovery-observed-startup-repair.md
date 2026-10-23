# Discovery startup verification repair — 2026-10-23

The repaired real-process probe passes twice on Mac, **58/58 each time**.
The insertion-before-send baseline fails **42/58** with a working observer and
actual failed-start witnesses. Missing and ambiguous outcome evidence both
produce exit 1. Retained discovery passes **19/19**; actual owned shutdown passes
**4/4**. This is Mac verification, not native Windows acceptance.

The immutable source/provenance commit is
`2253049d72f4ee5aa079045432c5014bb0c4be78`, from clean parent
`542ab5df5549cb68991356d0dba69f3c7b470098`. No source change followed that commit.
The [structured report](discovery-observed-startup-repair.json) contains all
commands, checks, witnesses, limits and hashes. The parent owns independent
review and later native Windows acceptance. No push or other thread message was
used.

## Outcome contract and implementation

The former probe treated an immediate caller drop as a failed startup. The
required [R6 review](discovery-startup-contract-review.md) and
[two Windows diagnostics](../release-integration-2026-10-22/windows-discovery-startup-diagnosis-2026-10-23.md)
show why that inference is invalid. `send_cmd` queues before its wake notification;
the actual worker can accept startup while the caller still holds its receiver.

The only vendor change here is an opt-in `mdns_sd::startup` trace immediately
after the actual initial `try_send`. It reports service type, limited/standard
kind, and `Accepted`, `Disconnected` or `Full`. It uses the same result for the
existing branch. It changes no subscription state, ordering, capacity, lifetime,
cancellation or scheduling. The log macro does not evaluate formatting arguments
when disabled. There is no production request ID, observation map, worker pause,
rendezvous, hook, injected worker command or new thread.

The example enables only this log target in startup mode. Its bounded 1,024-record
observer records caller phases and actual send outcomes. Distinct starts have
unique service types. Same-type requests run one at a time between public
`get_metrics` command barriers. Every submitted request must have exactly one
matching initial result. A missing, duplicate or unsupported result returns an
error. A barrier proves commit completion; the result trace alone precedes commit.

Only `Disconnected` establishes the failed-start precondition. `Accepted` is a
separate lifecycle outcome. Neither a dirty snapshot, an open new view nor caller
drop establishes that precondition. The repair removes the replay workload as a
startup-ordering mechanism. It adds no sleep or replay work to force cancellation.
Polling waits only for public state, service data, lifecycle or cleanup evidence.

## Real cases and coverage

Each run uses the actual worker and a separate ordinary `ServiceInfo` publisher
child. The publisher has a normal stdin phase protocol. Reused ready-file names
are cleared before each request, so an old acknowledgement cannot finish a new
phase. No DNS bytes are crafted. Both sides use loopback interfaces on the
assigned private ports.

The first phase drops 32 distinct limited starts and keeps a live limited guard
on another type. All 32 outcomes must match, and at least one must actually fail
with `Disconnected`. The barrier count cannot include a failed request's state.
Accepted requests may have transient state until normal receiver-drop cleanup.
All state observations are retained. Cleanup must reach **exactly one** guard
state; its original handle must remain open and receive a fresh update.

- Final fixed run 1: **10 Disconnected, 22 Accepted**; barrier gauge **23**;
  normal cleanup reaches **1**; **58/58**, exit 0, 27,157 ms.
- Final fixed run 2: **11 Disconnected, 21 Accepted**; barrier gauge **22**;
  normal cleanup reaches **1**; **58/58**, exit 0, 26,438 ms.

Failed limited and standard replacements are checked separately against both
live limited and live standard subscriptions. Each combination has an explicit
budget of eight natural attempts. Both fixed runs obtained `Disconnected` on the
first attempt in all four combinations. Accepted attempts would be recorded as
accepted/drop lifecycle cases and would not satisfy failed-start coverage.
Missing any required failed-send witness makes the run fail.

For each actual failed replacement, the exact old handle must stay open, the
actual limited-state gauge must stay at its baseline, and that handle must receive
a fresh TXT update, goodbye removal and a returned peer. Standard old receivers
also receive a later repeated `SearchStarted`. The limited API hides its lifetime
events; its direct evidence is the same open view, exact state and fresh lifecycle
data. Private sender/state/retransmission fields are not inspected.

Each combination also retains accepted replacement handles through actual startup,
resolution and a command barrier. Standard handles consume `SearchStarted`.
The old handle must close; the new handle must resolve the current peer and have
the exact per-type limited-state count. A newer limited generation then replaces
it. Dropping both older handles must leave that newest view open, fresh and at the
exact gauge. Dropping the newest view must clean up its limited state normally.
A final held view is explicitly stopped and must close. Finally stopping the
independent guard must leave **exactly zero** limited states.

Immediate accepted drops have a three-second cleanup deadline, derived from the
actual 10..50 ms initial-query jitter, the next one-second retry and scheduling
slack. Held newest views have eight seconds, covering their observed 1/2/4-second
retry steps plus slack. Every poll and transient gauge remains in the raw output.
These are finite checks, not a guarantee that future scheduling will cover every
failed-start path. Lack of coverage remains failure.

The [explicit mapping of all 14 former startup checks](discovery-observed-startup-repair.json)
keeps their state, identity, freshness, replacement and cleanup properties.
The two replay-work checks are mapped to real command barriers and the unchanged
retained flow's real overflow check. Replay completion is no longer used as a
cancellation precondition. No identity check is skipped or lower final gauge
accepted.

## Red capability on the actual buggy baseline

The baseline is an archive of
`884ff876b98e38d7380e119c67de8acbb86f6331`. It has the exact final repaired example,
the existing actual-map metric patch, and the necessary initial-result trace.
The baseline trace labels kind from the already-installed limited map entry;
that read occurs only inside enabled trace formatting. Its original command
insertion/removal-before-send ordering is unchanged.

The [complete instrumentation patch](discovery-observed-startup/outputs/baseline-instrumentation.patch)
and [archive scope audit](discovery-observed-startup/outputs/baseline-source-scope.json)
show that only the probe and vendor observation/metric source differ from that
archive. There are no compatibility or setup errors.

The final baseline matches **all 53** submitted browse requests. Its 32 distinct
starts include **19 Disconnected and 13 Accepted**. The gauge is **33** at the
barrier, then **20** at 3,025 ms: the guard plus **19 extra map entries**. The exact
one-state assertion fails before explicit cleanup. Actual failed replacements
are also witnessed in all four combinations:

- Limited over limited: baseline **2 → 2**, but the old limited handle closes.
- Standard over limited: baseline **2 → 1**, although the initial standard send
  is `Disconnected`; the old limited view loses fresh/lifecycle delivery.
- Limited over standard: baseline **1 → 2**, and the old standard receiver closes.
- Standard over standard: baseline **1 → 1** and the old receiver survives; this
  older path already preserved the old standard sender on a failed initial send.

The baseline exits **1**, with **42/58** checks. Sixteen checks fail on retained
state or lost old-view behavior. It does not fail on missing observation evidence.
The failed-send witnesses, exact gauges, identity loss and later failed freshness
checks establish the probe's red capability. Explicit stops happen only after
the strict measurements, so later cases can remain independent. The baseline
still completes publisher exit, final exact-zero state and worker shutdown.

See [baseline stdout](discovery-observed-startup/outputs/baseline-startup-final.stdout),
[actual send trace](discovery-observed-startup/outputs/baseline-startup-final.stderr),
and the [independent saved-evidence inspection](discovery-observed-startup/outputs/independent-evidence-inspection.json).
An earlier baseline run also failed 42/58, with 18 actual failed distinct starts.
Its raw output and source/artifact identities are retained. It does not replace
the final-source baseline above.

## Evidence-negative real processes

The unmodified final executable runs with `OPENERGY_STARTUP_OBSERVE=0` for the
missing-observer control. Its real worker reaches a public metrics barrier and
holds one actual limited state, but the matcher has **zero** initial-send records.
It reports `matches=0`, exits **1**, and emits no acceptance summary. Its ordinary
publisher exits zero and the worker acknowledges shutdown.

For ambiguity, a standalone copy of the final example adds one extra **real held
public browse call** within the same first observation phase. Both actual sends
record `Accepted`; the unchanged matcher finds **two** records for the phase,
reports `matches=2`, and exits **1** without an acceptance summary. Both workers
and the publisher follow their normal cleanup path. No record is duplicated or
fabricated. The [three-line helper delta](discovery-observed-startup/outputs/ambiguity-helper.diff)
and [exact linked real-library hashes](discovery-observed-startup/outputs/ambiguity-link-inputs.json)
preserve its source and build identity. It changes no product source or library.

Both [missing](discovery-observed-startup/outputs/missing-observer.stderr) and
[ambiguous](discovery-observed-startup/outputs/ambiguous-observer.stderr) evidence
therefore fail through real processes. `Full` is also rejected by the matcher;
it was not artificially injected. The [read-only evidence checker](discovery-observed-startup/inspect-results.py)
was run on these actual saved outputs. No unit tests, mocks or altered input
fixtures were used.

## Retained flow, shutdown, checks and provenance

The retained `probe` function body remains byte-identical to the parent. Its
separate 800-advertisement process flow passes **19/19**, including real upstream
overflow, bounded selected delivery, retained updates/removals, paused view
replacement, recovery, final goodbyes and shutdown. The [output](discovery-observed-startup/outputs/retained-final.stdout)
preserves all 19 names and values.

Both owned-shutdown probe sources are unchanged. Their actual process observer
passes normal drop, normal shutdown, exhausted drop and exhausted shutdown.
Each child remains alive while its owned thread count returns **2 → 1** and all
owned UDP descriptors disappear. Cleanup took **46, 45, 826 and 831 ms**. These
are actual OS observations, not a sentinel acknowledgement substitute. All
[before/live/after records](discovery-observed-startup/owned-shutdown/report.json)
are retained.

All final scoped checks pass with Rust **1.99.1**, deployment target **15.0**,
nice **15**, jobs **2**, and private targets/TMPDIR:

- Locked offline build of both real examples, and a separate locked offline
  baseline build.
- Strict Clippy: `-p openergy-net -p mdns-sd --lib --example discovery_recovery
  --example discovery_owned_shutdown --no-deps -j 2 -- -D warnings`.
- Example `rustfmt --check`; the vendor's new observation block also matches
  rustfmt output without reformatting preserved upstream source.
- Original archive/source verification and existing notices `--check`.
- Scoped whitespace check and exact source invariants.

The vendor delta against the parent is only the initial-result trace. Removing
that exact observation block restores the parent file byte for byte. The full
upstream patch and distinct modified/local hashes were regenerated. Archive
checksum, original file hashes, licenses, existing tests, Cargo manifests,
lockfile and toolchain file remain unchanged. Registry cache sources were not
edited. No upstream or unit tests were added or run.

Selected SHA-256 values:

```text
final probe source  77822d87411e035548f225a1964fa7954d5b1ae9a8a0ce25111815e661acf18d
final vendor source 0b1dcbd7b9e214d5d6ee5ad59e75818c72ee7761d7bf648c422a4a56029177d5
final probe binary  6f42357cc236d59438969b7c6aa9cd7c1303a2c2e8ec20edeafd9d37b728d28f
baseline binary     3669168889c28ca5ad3fbacf466a4f074498e1f4876052c92a18102c82ab6a53
upstream archive    8562654710f72a9cbce4b0d5b7c366a2f94330ba1ce3740f15e932ea2d37b161
```

[All source/artifact hashes](discovery-observed-startup/source-artifact-hashes.json)
include the negative helper, actual linked libraries and shutdown executable.
[Original source identities](discovery-observed-startup/outputs/original-source-identities.json)
retain full commits, blobs and SHA-256 values for `884ff876`, `da668c3`, `e4f48ba`
and `542ab5d` probes/vendor sources. One earlier fixed exploratory run passed
58/58; its raw output and exact source delta/artifact identity remain, separate
from the two final-source runs.

## Commands, preservation and cleanup

The JSON's `commands` array contains exact argv, environment, exit and duration
for every final build/check/process. `<PRIVATE_ROOT>`, `<WORKTREE>`, `<CARGO_HOME>`
and `<USER_HOME>` replace only path roots in copied command/report text.
The [copy manifest](discovery-observed-startup/copy-manifest.json) preserves original
and copied hashes and the exact transform. Checks, outcomes, timings and gauges
are unchanged. The [workflow runner](discovery-observed-startup/workflow/execute.py)
and helper diffs retain exact source bytes.

The main reproduction commands, with new private directories, are:

```sh
export PATH=/Users/serge/.cargo/bin:$PATH
export MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$TASK_ROOT/target-fixed" TMPDIR="$TASK_ROOT/tmp"
nice -n 15 cargo +1.99.1 build --locked --offline -p openergy-net \
  --example discovery_recovery --example discovery_owned_shutdown -j 2
nice -n 15 "$CARGO_TARGET_DIR/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/new-startup" 25090 25094
nice -n 15 "$CARGO_TARGET_DIR/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/another-new-startup" 25092 25096
OPENERGY_STARTUP_OBSERVE=0 nice -n 15 \
  "$CARGO_TARGET_DIR/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/new-missing-observer" 25090 25094
nice -n 15 "$CARGO_TARGET_DIR/debug/examples/discovery_recovery" \
  probe "$TASK_ROOT/new-retained" 25086 25087
nice -n 15 python3 crates/net/examples/discovery_owned_shutdown.py \
  "$CARGO_TARGET_DIR/debug/examples/discovery_owned_shutdown" \
  "$TASK_ROOT/new-owned-shutdown" 25098 25099
```

For the baseline, archive `884ff876` into a separate new private directory, apply
the saved complete instrumentation patch there, and copy the final example into
that archive. Use a separate private Cargo target and the recorded locked offline
build command. The saved baseline source audit verifies every archive file and
finds only these two permitted changed inputs. Reproduce ambiguity from the final
example plus its saved three-line delta and the recorded standalone pinned
`rustc` command, using the exact actual fixed-library inputs. No registry edits
or dependency changes are needed.

All **302** historical evidence files in the scoped old evidence directories and
R6 record are byte-identical to `542ab5d`; their
[manifest](discovery-observed-startup/outputs/preserved-old-evidence-sha256.json)
includes the original failures. **Original native Windows startup remains FAIL
11/14 and unaccepted.** Its original unlogged interleaving remains unknown.
Old Mac results and both Windows diagnostics are retained as historical evidence;
they are not relabelled by this repair.

Private originals, build trees and binaries remain under
`/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-discovery-observed-20261023-han0t49b`.
The root mode is `0700`. Postflight found no fixture process, assigned UDP
endpoint, assigned TCP listener or private TMP entry. Normal flows and both
negative controls completed normal publisher/worker cleanup; no process timeout
or forced cleanup occurred. Passive report/runtime files and artifacts remain
for review.

## Limits and exclusions

The gauge measures actual limited map entries. It does not measure total heap,
standard-map size, query bounds or private identities. Limited lifetime events
are hidden by the public API. Their behavior is verified through the original
view and actual state/data/lifecycle, not private-field inspection. Held newest
limited-drop cleanup is measured; no direct standard-map-size claim is made.
The observer has a finite memory/formatting cost and can affect natural scheduling;
the probe classifies the observed result and never treats that schedule as fixed.

Whole-cache/expiry/timer/query/variable-byte bounds, slow DNS, native
input/clipboard, positive Windows migration and original security/provenance/
trusted distribution remain open. The stopped IPC Synergy3 task, older
expiry/NSEC/orphan-record experiment and original blocked provenance review were
not continued, reviewed, rerouted or integrated.

No Windows execution, unit tests, mocks, computer use, new child agents, other
thread contact, installed/live daemon, sharing/backend/input/clipboard/desktop,
TCC/login/services/tasks/elevation/accounts/credentials/firewall/network/global
resource/toolchain-default/Docker/signing/publication/GitHub/push/merge change was
used. Private advertisement/listener ports stayed within 25080..25099. Existing
OS-assigned loopback command-wake sockets retain their upstream behavior. Ports
24800 and 5353 were not used. TypeScript validation is not applicable to this
scoped Rust/evidence change.
