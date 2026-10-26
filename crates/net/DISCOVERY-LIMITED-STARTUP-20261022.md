# Cancelled limited-browse startup — implementation follow-up

Starting source was clean `884ff876b98e38d7380e119c67de8acbb86f6331` in the
existing isolated worktree. No parent IPC, Windows or reactor files changed.
This is an implementation candidate for a fresh independent review.

## Reproduced P2 and fix

The independent review identified this sequence: `Command::BrowseLimited`
removed the old subscription and installed its shared state before the initial
`SearchStarted` send. A receiver dropped while the command was queued caused
that send to fail. The new state stayed in `limited_browses` without a sender or
retransmission that could later notice cancellation. A cancelled replacement
also destroyed the prior subscription's identity. The standard-browse command
similarly removed an old limited map entry before checking the new receiver.

The worker now attempts the initial send before any subscription mutation.
Only a successful send replaces the old sender, shared state and retransmission
generation. A failed send returns with the old subscription intact; the new
command's state then drops normally. Both standard and limited commands use this
ordering. The original private helper signature remains available for unchanged
upstream test sources. There is no forwarding thread or new queue.

`get_metrics()["limited-view-states"]` reads the actual `limited_browses.len()`
on the daemon worker. It measures retained map entries, including any orphans.
It does not use Arc reference counts and does not count states still in queued
commands. The metrics command after the queued work is a command-order barrier.
The retained state is measured again after 300 ms to distinguish transient
startup/disconnect handling from a persistent orphan.

## Ordinary public-API process probe

The existing `discovery_recovery` executable has a separate `startup` mode.
It uses a separate publisher child with 800 ordinary `ServiceInfo` burst
registrations. A guard service on another valid type supplies a live handle.
The worker receives sixteen ordinary cache replays, then 32 distinct valid
limited browses whose returned handles are immediately dropped. All replay
channels are bounded by the unchanged upstream 1,000-event capacity. The command
workload stays below the existing 100-command queue capacity.

The probe verifies all sixteen replay channels actually overflowed and closed.
It drains the command path through real `get_metrics`, then checks that only the
live guard state remains. Cancelled same-type limited and standard starts must
preserve that handle and deliver a later real TXT update. Successful replacements
must close the old generation, resolve on the new generation, and keep exactly
one limited state (or zero for a standard replacement). A second ordinary replay
batch checks cancellation while a standard receiver is live, including its later
real update. Final `stop_browse` must close the final view and leave zero worker
map entries. The publisher exits zero and the worker acknowledges shutdown.

No callbacks, injected worker commands, fabricated state counters, unit tests,
mocks, crafted DNS bytes or isolated Arc checks are used.

## Raw baseline and fixed results

[Evidence directory](examples/discovery-startup-evidence-20261022/) contains the
raw reports, source/binary hashes, baseline metric-only patch and exit statuses.
Earlier candidate evidence directories remain unchanged.

The baseline adds only the real metric to `884ff876`'s vendor source. Its metric
instrumentation patch and compiled source hashes are preserved. No startup fix
was in that binary. `baseline-startup.log` exits 1 with 5/12 checks passed:

```text
CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows=16 before=1 after_barrier=32 after_settle=32 live_closed=true
CHECK cancelled-distinct-types-retain-no-state=false
CHECK cancelled-replacements-preserve-identity=false
CHECK surviving-handle-receives-live-update=false
CHECK final-stop-clears-worker-state=false
```

All 32 cancelled keys retained their worker-owned state after the live guard had
been lost. Stopping the guard left those 32 entries. The probe still completed
publisher cleanup and actual worker shutdown. This failure is preserved.

`fixed-startup-1.log` passes the initial 12/12 checks. The final probe adds the
two standard-receiver cancellation/update checks. `fixed-startup-final.log`
passes all 14/14 checks; `fixed-startup-repeat.log` also passes 14/14:

```text
CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows=16 before=1 after_barrier=1 after_settle=1 live_closed=false
```

The final worker-map counts are `1 -> 1 -> 1 -> 1 -> 0 -> 0 -> 1 -> 0`:
initial live guard, cancellation barrier, settled cancellation, successful
limited replacement, standard replacement, cancelled standard replacements,
standard-to-limited replacement, final stop. No cancelled distinct type remains.
The identity/update checks pass for both live limited and standard receivers.

The original retained mode passes 19/19 on the final source. All four actual
owned-shutdown cases also pass; cleanup took 52, 52, 823 and 837 ms. Their raw
results are recorded beside the startup logs.
The shutdown check observes each private child still alive after its owned
worker thread and UDP descriptors are gone; a sentinel acknowledgement alone
is not substituted for that check.

## Validation and provenance

Runtime remains Rust 1.99.1, deployment target 15.0, nice 15, two build jobs.
Locked offline builds, strict Clippy for openergy-net and mdns-sd, scoped Rust
formatting, archive/source verification and the existing notice check pass.
The notice inventory remains accurate without a license/manifest/lock change:
the same locally modified 0.21.6 path crate and unchanged license bytes apply.

The complete [current upstream delta](../../vendor/mdns-sd/OPENERGY.patch) is
389 lines across two upstream source files. `UPSTREAM.json` updates the distinct
modified-file and local patch/document hashes. The original archive checksum
and original file hashes remain unchanged. Both licenses and all existing tests
remain byte-identical to the original archive. No unit tests were added or run.
Whole-vendor whitespace exceptions remain the preserved original source lines
and exact patch context documented in the earlier candidate report. The baseline
instrumentation patch also keeps its required unified-diff context whitespace.

Reproduce with the pinned Cargo bin directory on PATH:

```sh
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 cargo +1.99.1 build \
  --locked --offline -p openergy-net --example discovery_recovery \
  --example discovery_owned_shutdown -j 2
nice -n 15 target/debug/examples/discovery_recovery startup NEW_PRIVATE_DIR 25090 25091
nice -n 15 target/debug/examples/discovery_recovery probe OTHER_NEW_PRIVATE_DIR 25086 25087
nice -n 15 python3 crates/net/examples/discovery_owned_shutdown.py \
  target/debug/examples/discovery_owned_shutdown THIRD_NEW_PRIVATE_DIR 25094 25095
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 cargo +1.99.1 clippy \
  --locked --offline -p openergy-net -p mdns-sd --lib --example discovery_recovery \
  --example discovery_owned_shutdown --no-deps -j 2 -- -D warnings
nice -n 15 python3 vendor/mdns-sd/verify-source.py ORIGINAL_CRATE_ARCHIVE
nice -n 15 python3 tools/release/notices/generate-notices.py --check
```

## Limits and safety

This closes the reproduced cancelled-startup retention path in the local
candidate. A cancellation after a successful initial send uses the existing
subscriber-disconnect cleanup path. The metric measures map entries, not total
heap usage. This adds no whole-cache, timer, resolver-work, performance or
release-acceptance claim. The parent will request independent review.

Listeners and advertised service ports stayed within private loopback
25080..25097. Existing OS-assigned loopback command-wake sockets are unchanged.
No 24800/5353 traffic, live daemon, native input, clipboard, installed state,
services, TLS identities, credentials, TCC, accounts, global limits, DNS,
firewall, Docker, push or publication were used or changed. The shutdown probe's
existing descriptor limit applies only to its own private child. No other
thread was contacted and no additional agent was launched.

Provider-stopped expiry/NSEC/orphan-record testing and the original blocked
release-provenance review were not rerouted or attempted. They remain unverified.
Native, positive Windows migration and trusted-distribution gates remain open.
New copied raw evidence removes personal host paths/usernames and trailing
whitespace while preserving result values. Private originals remain under
`/tmp/oe-limited-startup-20261022-5a69/`.
