# Discovery replay and owned shutdown: partial fix, open replay blocker

Starting checkout: clean `ac5c45081a9e060392cb850eddd625f07b591bfe`, branch
`t3/discovery-replay-shutdown`, isolated worktree `t3-8828aae9`. The named parent
source matched exactly before edits. Runtime: macOS 27.2 (26B5112a), Rust
`1.99.1 (58cf8f9c3 2026-10-13)`, deployment target 15.0. Builds used `nice -n 15`
and two jobs. No unit tests or mocks were added or run.

## Result and source change

`Discovery::shutdown` now submits upstream `Exit` directly. It no longer submits
`unregister` and `stop_browse` first. Upstream already unregisters advertisements
and stops browsing inside `Zeroconf::cleanup` when processing `Exit`.

The old sequence returned early when a command's temporary wake-socket allocation
failed. That command had already entered the upstream command queue, but `Exit`
had not. The upstream worker retains its own command sender, so dropping the
wrapper did not close the worker. The new sequence puts `Exit` in that queue
before the wake-socket allocation can fail. Errors remain visible to callers.

This is a submission API, not a worker join. A successful return does not prove
that the worker has exited. Conversely, an upstream wake error can follow a
successful enqueue. Queue-full rejection of `Exit` remains an upstream API
boundary; this change does not claim a reserved shutdown queue slot or an
unconditional shutdown guarantee.

No retained-replay workaround was applied. `ac5c450` is **not** complete for
discovery recovery in a large retained namespace.

## Real retained namespace

`discovery_recovery` keeps 800 ordinary `ServiceInfo` advertisements alive in a
separate publisher process. Its caller resumes 20 ms polling with a 256-event
drain limit. After a two-second recovery allowance, the probe checks useful
snapshots and their one-second emission spacing. It selects an actual displayed
burst peer immediately before changing that peer's TXT name, port and hint.
Freshness must persist for two seconds. It then removes that same peer. A
replacement advertisement keeps the burst namespace at 800 during removal;
the probe does not require admission of that replacement or display of all 800.
Removal only passes if the preceding live-update freshness check passed.

The first two failure reports used unchanged `ac5c450` discovery source. They
are retained without overwriting them. The final probe adds strict freshness,
snapshot-spacing and stop-after-overflow observations. The shutdown source fix
does not change browse/replay behavior.

Final raw result (`fixed-retained-final.log`, exit status 1):

```text
RETAINED_REPLAY attempt=1 drained=1000 disconnected=true elapsed_ms=91
RETAINED_REPLAY attempt=2 drained=1000 disconnected=true elapsed_ms=95
RETAINED_NAMESPACE burst=800 selected=burst-0003 full_polls=12 empty_polls=160 snapshots=6 update_seen=true update_stays_fresh=false removal_seen=true removal_regressed=false
CHECK snapshot-cap=true
CHECK snapshot-emission-spacing=true
CHECK retained-namespace-stays-useful=false
CHECK retained-selected-peer-live-update=false
CHECK retained-selected-peer-goodbye=false
```

The sentinel's retained cache had 801 PTR records, 499 SRV records and 499 TXT
records. After `stop_browse`, these counts were unchanged. The selected update
did appear briefly; accepting a single fresh-looking or nonempty snapshot would
have hidden the failure. The removal flag was observed, but a valid live-update
prerequisite was absent, so the final removal check correctly fails.

All seven original functional checks still pass: initial resolution, real
overflow, display cap, advertisement after clearing the burst, changed port and
untrusted hint handling, goodbye removal after clearing, and publisher exit.
The original eighth `worker-shutdown=true` statement has been corrected into
`discovery-shutdown-submitted` and `sentinel-worker-shutdown-ack`. Both pass.
They describe different workers and do not assert wrapper-owned worker lifetime.
Including the new checks, the final result is 10/13, with all three retained
namespace checks failed. The first failure reports' less strict removal check
must not be used to close the later freshness/removal gate.

## Actual owned worker and socket lifetime

`discovery_owned_shutdown.py` drives a real private Rust child over pipes. It
records `/bin/ps -M` and `/usr/sbin/lsof` output before initialization, after
initialization, and after Drop/shutdown and descriptor release. The child remains
alive during the last inspection. Only that child's soft `RLIMIT_NOFILE` is
temporarily lowered to 128; it holds `/dev/null` descriptors until `EMFILE`.
No global or parent limit changes. Advertised shutdown cases first resolve the
child's own advertisement through real loopback discovery.

Baseline `ac5c450` (`baseline-shutdown-1/report.json`, exit status 1):

- Normal Drop: cleaned, 47 ms.
- Descriptor-exhausted Drop: still two threads and five UDP descriptors after
  7,139 ms, with the child alive.
- Descriptor-exhausted explicit shutdown followed by Drop: same leak after
  7,053 ms. The real error was `Too many open files (os error 24)`.

Fixed source, two independent runs (`fixed-shutdown-1.log` and
`fixed-shutdown-2/report.json`, both exit status 0):

- Both runs passed all four cases: normal Drop, normal advertised shutdown,
  descriptor-exhausted Drop and descriptor-exhausted advertised shutdown.
- In the final run, cleanup took 47, 46, 804 and 807 ms respectively.
- Every case returned from two threads/five UDP descriptors to the original
  one thread/no UDP sockets while its child stayed alive.
- Explicit shutdown under descriptor exhaustion still returned its wake error;
  the separately observed owned worker nevertheless exited after handling the
  queued `Exit`. No sentinel acknowledgement is used for this proof.

The raw `ps`/`lsof` snapshots and child output are committed beside the baseline
and final JSON reports. The baseline probe had three cases and did not require
resolution of the child advertisement. Both fixed runs include the fourth case
and require that resolution.

## Exact upstream API blocker and proposal for parent integration

Read-only source: registry `mdns-sd-0.21.6/src/service_daemon.rs`:

- Lines 85-89: private `EVENT_CHANNEL_CAPACITY = 1000`.
- Lines 450-478: both `browse` and `browse_cache` create that fixed bounded
  channel. Neither takes a replay budget, selected-service limit or predicate.
- Lines 2991-3055: `query_cache_for_service` walks retained PTR aliases and sends
  `ServiceFound` plus a valid `ServiceResolved` synchronously.
- Lines 4070-4101: the initial browse command calls that replay before returning
  to the worker loop. A caller cannot pace it through its own next poll.
- Lines 1531-1550: `push_service` removes the subscription when `try_send` is full.
- Lines 4323-4340: `stop_browse` clears service cache only if an active subscription
  still exists. Overflow already removed it, so stopping the closed browse does
  not clear its retained cache. The real metrics above confirm this case.
- Lines 361-389 and 401-428: the worker owns a cloned command sender; enqueue
  precedes wake-socket creation. Lines 1591-1664 and 1747-1753 show cleanup on
  `Exit`, including unregister and browse shutdown.

The wrapper's 64-peer display cap applies after delivery and cannot bound replay
or cache ownership inside the dependency. Increasing the caller drain limit,
using `browse_cache`, or retrying the same cache cannot repair this ordering.
An extra forwarding thread would add resources and still race synchronous replay.
An unbounded event channel would weaken the resource boundary. Worker recreation
or interface cycling would discard cache and disturb discovery/advertising;
neither is implemented or claimed as a robust remedy.

Proposed parent decision: obtain an upstream API that supports bounded replay
and explicit cache ownership, then rerun this same retained-namespace probe.
One design is an opt-in 64-service view with stable selected peers, coalesced
latest state per selected service, and bounded pending removals. Its initial
Found/Resolved replay fits within 129 events. Large unselected namespaces must
not displace a selected peer's updates or removal. Record/cache and pending-query
budgets must also account for unselected PTR/SRV/TXT/address data. The current
public API cannot express that policy.

A narrower upstream alternative is resumable, budgeted replay on the existing
worker, with a bounded pending cursor and an explicit overflow/resync contract.
It needs real large-namespace update/removal and resource measurements before it
can be accepted. Making `stop_browse` purge retained state even after a closed
subscription is useful cleanup, but does not alone bound the next live burst.
No dependency, manifest, lockfile, registry file or notices were changed here.

Upstream expiry runtime verification remains a separate open gate. No expiry
example is included in this change. An exploratory expiry invocation had already
completed before the 21:10:59 scope update arrived; its uncommitted source was
removed immediately after that update, and its prior raw output remains only in
`<private>/cache-expiry-1.log`. It is excluded from
this task's evidence and does not close that gate. No reroute or further expiry
runtime work followed the update.

## Reproduce and validation

All multicast listener ports below are custom 25080..25099, on loopback only.
The dependency's private command-wake sockets use OS-assigned loopback ports.
No traffic was sent to 5353 or 24800. Service ports in advertisements also use
the assigned range. No native input, clipboard, installed service, real config,
credentials, firewall, Docker or distribution state was changed.

```sh
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 \
  cargo +1.99.1 build --locked -p openergy-net \
  --example discovery_recovery --example discovery_owned_shutdown -j 2
nice -n 15 target/debug/examples/discovery_recovery probe NEW_DIR 25080 25081
nice -n 15 python3 crates/net/examples/discovery_owned_shutdown.py \
  target/debug/examples/discovery_owned_shutdown OTHER_NEW_DIR 25083 25084
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 \
  cargo +1.99.1 clippy --locked -p openergy-net --lib \
  --example discovery_recovery --example discovery_owned_shutdown \
  --no-deps -j 2 -- -D warnings
nice -n 15 /Users/serge/.cargo/bin/rustfmt +1.99.1 --edition 2021 --check \
  crates/net/src/discovery.rs crates/net/examples/discovery_recovery.rs \
  crates/net/examples/discovery_owned_shutdown.rs
```

The scoped build, strict Clippy and formatting checks passed. The retained replay
command intentionally still exits 1; owned shutdown exits 0. This does not replace
the parent's previously preserved Mac/Windows or virtual-control evidence, and
does not close release provenance, Windows privilege, native behavior, positive
Windows migration or trusted distribution gates.

Raw committed reports: [discovery-replay-evidence-20261022](examples/discovery-replay-evidence-20261022/).
Full original runtime directories remain at
`<private>/`.

The public copies replace the local username and absolute fixture paths, and
remove trailing whitespace from `ps` output. Result values are unchanged.
[Normalization hashes](examples/discovery-replay-evidence-20261022/public-copy-normalization.json)
identify both copies. The original bytes remain local.
