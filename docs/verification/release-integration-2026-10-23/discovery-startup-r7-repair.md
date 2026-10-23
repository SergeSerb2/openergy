# Discovery verification after R7 — 2026-10-23

Final source is **`e6a9125be2fc59fbe92fef6ea351b222298bfc52`**.
It supersedes `2253049d72f4ee5aa079045432c5014bb0c4be78`.
The newer source was built, checked and committed before the matching real runs.
No source changed afterward. Fresh independent review must follow this revision.

Mac verification passes **61/61 twice**. The matched insertion-before-send
baseline fails **45/61 on actual retention/identity behavior** with working
observation. Missing initial evidence, ambiguous initial evidence and missing
normal-cleanup evidence all fail. Retained flow passes **19/19**, and actual owned
shutdown passes **4/4**. **Native Windows startup remains FAIL 11/14 and unaccepted.**
No Windows execution or fresh independent review pass is claimed here.

The [structured report](discovery-startup-r7-repair.json) contains every command,
exit, check, ordered witness, source/artifact hash, old-check mapping and limit.
The [saved-evidence checker](discovery-startup-r7/inspect-results.py) was run on the
actual private outputs and their copied counterparts. It uses no synthetic input.

## The two R7 verification gaps

R7 found no concrete P1/P2 product defect in `2253049`. It found two verification
gaps, both repaired here.

First, draining the old standard receiver before replacement allowed a queued
pre-failure `SearchStarted` to satisfy the later retained-repeat assertion. The
probe now establishes the actual `Disconnected` initial result and completes its
public command barrier **before draining the exact old receiver**. It then waits
for a fresh public repeat. The receiver counts actual initial/repeated start
events. That count establishes the next 1/2/4/... second retry interval, with two
seconds of scheduling slack. Counts above the finite established range fail
coverage. No worker pause, timing control or extra replay work was added.

Both fixed runs recorded the required order for limited and standard failed
replacements against a standard receiver:

```text
actual initial Disconnected
public command barrier completes
old standard receiver drained
fresh SearchStarted received
```

The recorded waits used an observed start count of 2 and a three-second deadline.
The fresh events arrived after the drain in both cases. The report preserves
their exact monotonic timestamps, rather than inferring freshness from API return.

Second, the limited-state metric cannot show removal of a standard subscription,
and an immediate accepted standard drop might never occur naturally. The probe
now always creates a separate **held active standard subscription** on
`_oe-drop-std._tcp.local.`. It requires actual initial acceptance, public command
barriers, `SearchStarted`, resolution and a fresh repeat while still held. Initial
acceptance alone is not a commit acknowledgement.

After dropping that receiver, the probe requires exactly one actual completed
normal removal with the matching service type, standard kind and `Disconnected`
reason. It follows that observation with another real command barrier. No
explicit stop or replacement is submitted before this check. A missing witness
or missing held-standard case fails coverage. The existing immediate accepted-drop
branches also require normal removal evidence; standard cleanup can no longer
pass from `limited-view-states == 1` alone.

## Narrow observation source

Relative to `2253049`, the vendor code adds only three `log::trace!` calls after
the existing completed normal-removal calls in `push_service`:

- Limited receiver disconnected.
- Standard event channel full.
- Standard receiver disconnected.

The `mdns_sd::cleanup` target records actual type, kind, reason and completed
removal. Explicit stop and replacement do not emit this normal-cleanup trace.
The required drop case accepts only one `Disconnected` record. Missing,
duplicate, unknown or `Full` cleanup evidence fails. It cannot be satisfied by a
later explicit cleanup.

There is no production state, scheduling, lifetime, cancellation, API or capacity
change. Disabled logging adds no formatting/allocation. The existing exact
initial-send trace remains. Both targets use the executable's bounded 1,024-record
observer. Matching uses distinct types or serialized same-type phases and normal
public metrics barriers. No production request ID, map, hook or rendezvous was
added. The exact upstream patch, modified hashes and provenance were regenerated.

The [source invariant record](discovery-startup-r7/outputs/source-invariants.json)
proves that removing those three trace macros restores the `2253049` vendor file
byte for byte. The existing embedded upstream test module also matches the crate
archive byte for byte. Licenses, original archive checksum, original hashes,
existing tests, Cargo manifests, lockfile and toolchain file remain unchanged.
Registry sources were not edited. No unit tests were added or run.

## Matching real results

Both final fixed runs match all **54** submitted browse requests to actual initial
results. They preserve the independent live guard, exact final gauges, old/new
identity, fresh updates, goodbye/return lifecycle and generation-drop checks.

- Fixed run 1: **61/61**, exit 0, 29,705 ms. The 32 distinct drops recorded
  **13 Disconnected / 19 Accepted**. Barrier gauge **20** cleaned to exactly **1**.
- Fixed run 2: **61/61**, exit 0, 29,602 ms. The 32 distinct drops recorded
  **15 Disconnected / 17 Accepted**. Barrier gauge **18** cleaned to exactly **1**.
- Each fixed run obtained actual `Disconnected` on the first attempt in all four
  replacement combinations. The maximum remains eight natural attempts per case.
  Accepted starts do not satisfy failed-start coverage.
- Each fixed held-standard drop recorded one completed normal standard
  `Disconnected` removal, then a command barrier, with no explicit stop for that
  type. Removal followed drop by about 1.99 seconds in these two observations.

All transient state samples remain in raw output. The distinct phase still
requires a real failed-send witness and exact one-state cleanup. Every failed
replacement retains exact baseline state and original handle identity, then
receives fresh data. Final limited state must be exactly zero. A future run that
misses a required outcome or cleanup path fails; natural scheduling is not forced.

The matched baseline uses original
`884ff876b98e38d7380e119c67de8acbb86f6331`, the exact final probe, the existing real
map metric and necessary initial/normal-removal observation. An explicit
observation-only command-kind argument labels limited/standard starts without a
map lookup. Original insertion/removal-before-initial-send ordering stays intact.
The [full patch](discovery-startup-r7/outputs/baseline-instrumentation.patch) and
[archive scope audit](discovery-startup-r7/outputs/baseline-scope.json) preserve
that boundary; only the example and instrumented vendor source differ.

Baseline observation matches all **54** requests, including **16 Disconnected /
16 Accepted** distinct starts and failed sends in all four replacement cases.
Its gauge is **33** at the barrier and **17** at 3,001 ms: the guard plus 16 extra
entries. Actual failed replacements also lose old limited/standard behavior.
It fails **16 retention/identity/data checks**, for **45/61**, exit 1. Its required
held-standard normal-cleanup case passes, so this is not missing cleanup evidence,
a missing initial log, an incompatible helper or an unrelated setup failure.
Explicit stops occur only after the strict failing measurements. Publisher exit,
final exact-zero state and actual shutdown still complete.

See [baseline stdout](discovery-startup-r7/outputs/baseline-startup.stdout),
[baseline trace](discovery-startup-r7/outputs/baseline-startup.stderr),
[fixed run 1](discovery-startup-r7/outputs/fixed-startup-1.stderr),
[fixed run 2](discovery-startup-r7/outputs/fixed-startup-2.stderr), and
[ordered inspection](discovery-startup-r7/outputs/evidence-inspection.json).

## Real negative controls

The final executable with `OPENERGY_STARTUP_OBSERVE=0` reaches its real worker
barrier but has zero initial-send records. It rejects `matches=0`, exits 1 and
emits no acceptance summary. Its publisher exits zero and worker shutdown is
acknowledged.

With only `OPENERGY_STARTUP_CLEANUP_OBSERVE=0`, initial observation still works and
the required standard subscription starts, resolves and remains active. After
drop, the gauge is **1**, but the normal-removal witness is absent. Both the
automatic-removal and accepted-standard-drop coverage checks fail: **59/61,
exit 1**. The later explicit stop occurs only after that failed check and cannot
mask it. This actual process proves that a small limited gauge cannot make
standard cleanup pass.

For ambiguity, a standalone exact final example adds one extra actual held public
browse in its first phase. Both actual sends are accepted. The unchanged matcher
rejects `matches=2`, exits 1 and completes normal publisher/worker cleanup. No
records are fabricated. Its [minimal source delta](discovery-startup-r7/outputs/ambiguity-helper.diff),
standalone compiler command and [actual linked library hashes](discovery-startup-r7/outputs/ambiguity-link-inputs.json)
are preserved. Unknown/duplicate/Full cleanup results are rejected by source
cardinality and exact reason matching; no synthetic Full event was injected or
claimed as runtime coverage.

## Retained flow and actual owned shutdown

The retained `probe` function body remains byte-identical to `542ab5d`. Its actual
800-advertisement separate-publisher flow passes **19/19**, including overflow,
bounded selected delivery, retained and paused updates/removals, recovery, goodbye
and shutdown. [Raw retained output](discovery-startup-r7/outputs/retained.stdout)
keeps all 19 check names.

Both owned-shutdown probe sources remain unchanged. All four actual cases pass:
normal drop, normal shutdown, exhausted drop and exhausted shutdown. Each child
stays alive while its owned worker thread disappears (**2 → 1** total threads)
and its UDP descriptors return to the empty baseline. Cleanup took **60, 50, 857
and 860 ms**. The [OS observation records](discovery-startup-r7/owned-shutdown/report.json)
preserve baseline/live/after data. A sentinel acknowledgement is not substituted
for the actual lifetime observation.

## Checks, commands and hashes

Before committing final source, these scoped checks passed with Rust **1.99.1**,
deployment target **15.0**, nice **15**, jobs **2**, private targets/TMPDIR and
locked offline builds:

- Both real examples built successfully.
- Strict scoped Clippy for `openergy-net` and `mdns-sd`, both examples and libraries,
  with `--no-deps -j 2 -- -D warnings`.
- Example format check and the new vendor trace blocks against rustfmt output.
- Original crate archive/source verification and existing notices `--check`.
- Source invariants, exact historical-byte preservation and scoped whitespace.

The JSON `commands` array records exact argv, environment, result and duration
for the final builds, baseline, controls and real flows. Representative calls are:

```sh
PATH=/Users/serge/.cargo/bin:$PATH
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR="$TASK_ROOT/target-fixed" TMPDIR="$TASK_ROOT/tmp" \
  nice -n 15 cargo +1.99.1 build --locked --offline -p openergy-net \
  --example discovery_recovery --example discovery_owned_shutdown -j 2
nice -n 15 "$TASK_ROOT/target-fixed/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/new-startup" 25090 25094
OPENERGY_STARTUP_OBSERVE=0 nice -n 15 \
  "$TASK_ROOT/target-fixed/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/new-missing-initial" 25090 25094
OPENERGY_STARTUP_CLEANUP_OBSERVE=0 nice -n 15 \
  "$TASK_ROOT/target-fixed/debug/examples/discovery_recovery" \
  startup "$TASK_ROOT/new-missing-cleanup" 25092 25096
nice -n 15 "$TASK_ROOT/target-fixed/debug/examples/discovery_recovery" \
  probe "$TASK_ROOT/new-retained" 25086 25087
nice -n 15 python3 crates/net/examples/discovery_owned_shutdown.py \
  "$TASK_ROOT/target-fixed/debug/examples/discovery_owned_shutdown" \
  "$TASK_ROOT/new-owned-shutdown" 25098 25099
```

Each runtime/report directory must be new. Archive `884ff876` into a separate
private directory for the baseline, apply its complete saved instrumentation
patch, copy the final example and use the recorded separate locked offline build.
Reconstruct the ambiguity helper from the exact final example and saved delta;
its recorded standalone compiler command links the actual fixed libraries.

Selected SHA-256 values:

```text
final probe source  eb7460e8e98527f9e2d1ed9e8081cb46c996d7324045da98043ff599db14b100
final vendor source 7cd5be1cfa354dc35f95777348486adaec7d1afba2801e78a062f3132aa65853
final probe binary  6bb2949b752dd2c88232c8aff9be57da4cbba52f7a92c741d4aebd565a02c6ef
baseline binary     33e0cd33f7a80852b4865b06fa4c18afd9cdf7e484b4faef892b2af66c4e73bd
upstream archive    8562654710f72a9cbce4b0d5b7c366a2f94330ba1ce3740f15e932ea2d37b161
```

The [full hash inventory](discovery-startup-r7/source-artifact-hashes.json) includes
the normal-removal provenance delta, shutdown binary and ambiguity helper.
[Source identities](discovery-startup-r7/outputs/source-identities.json) preserve
original commits/blobs/hashes and both `2253049` and `e6a9125` revisions.

## Preservation, cleanup and limits

All **302 original evidence files** and **97 superseded `2253049` report/evidence
files** remain byte-identical. Their manifests are attached. The older
[2253049 report](discovery-observed-startup-repair.md) keeps its original historical
assessment unchanged; its pass wording is **superseded**, not final acceptance.
Original Windows **FAIL 11/14**, both diagnostics, original probe identities and
all previous failures remain untouched. The original unlogged Windows schedule
is still unknown.

New copied output changes only private/worktree/user path roots. Check values,
timestamps, event order, gauges and failures remain. The [copy manifest](discovery-startup-r7/copy-manifest.json)
records original/copied hashes and transforms. Patch/helper/workflow source bytes
remain exact. Raw process output and unified-diff context whitespace are retained.

Private originals and frozen artifact copies remain under
`/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-discovery-observed-r7-20261023-e8wuchni`.
Its root mode is **0700**. Final postflight found no fixture processes, assigned
UDP endpoints, assigned TCP listeners or private TMP files. All real flows and
negative controls completed normal cleanup. No timeout or forced cleanup occurred.
Passive runtime files, reports and build artifacts remain for review.

The metric measures actual limited map entries, not standard-map size or total
memory. Standard normal removal is now directly observed at its completed normal
path and followed by a command barrier. Limited lifetime events remain private;
its same public view, exact map count, openness and fresh lifecycle data are the
available evidence. The observer has finite cost and can affect natural scheduling;
the classifier accepts no schedule assumption as proof.

Fresh independent review and later native Windows acceptance remain the parent's
gates. Whole-cache/expiry/timer/query/variable-byte bounds, slow DNS, native
input/clipboard, positive Windows migration, original security/provenance and
trusted distribution remain open. No stopped IPC/Synergy3 task, older expiry/NSEC/
orphan-record experiment or original blocked provenance review was continued,
reviewed, rerouted or integrated.

No units, mocks, computer use, child agents, other-thread messages, installed/live
daemon, native sharing/backend/input/clipboard/desktop/TCC/login/service/task/
elevation/account/credential/firewall/network/global resource/toolchain-default/
Docker/signing/publication/GitHub/push/merge changes occurred. Ordinary advertised
service and mDNS listener ports stayed in **25080..25099**. Existing OS-assigned
loopback command-wake sockets retain upstream behavior. Ports **24800/5353** were
not used. TypeScript validation is not applicable to this Rust/evidence scope.
