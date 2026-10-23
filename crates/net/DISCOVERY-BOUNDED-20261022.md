# Bounded selected discovery delivery — implementation candidate

The isolated checkout was clean at parent `54aa50b9eafc51f6eef2589c100c509eaac7e415`.
Its changes after runtime source `ac5c45081a9e060392cb850eddd625f07b591bfe` are four
source-history evidence documents only. Required cherry-pick
`330d83f2f168644035c4a99ef3484675214d0e65` became local `5d0391a` before new work.
No reactor source changed. The parent must independently review this candidate.

## Source and dependency change

The official [published API](https://docs.rs/mdns-sd/0.21.6/mdns_sd/struct.ServiceDaemon.html)
and [upstream main at a90c138](https://github.com/keepsimple1/mdns-sd/blob/a90c1382f45949338fe93eada953bd7b760cf538/src/service_daemon.rs)
were checked before patching. Both expose `browse` and `browse_cache`, with the
fixed event channel and synchronous replay; neither exposes a service limit or
paced replay. The crates.io JSON API request returned HTTP 403; the docs.rs
latest API identified 0.21.6. No dependency upgrade was made.

The repository now patches exactly locked mdns-sd 0.21.6 through a path crate.
The downloaded published archive matched the old lockfile's SHA-256:
`8562654710f72a9cbce4b0d5b7c366a2f94330ba1ce3740f15e932ea2d37b161`.
Its published VCS revision is `1a5cdb6ebb7299ee74dda75099461a2921b6e740`.
Both licenses and all existing test/example sources are retained. Original and
modified hashes are separate. There is no package checksum that presents the
modified tree as the original archive. No Cargo registry source was patched.

Candidate `884ff876` had a 332-line delta across two upstream source files.
The [current exact upstream delta](../../vendor/mdns-sd/OPENERGY.patch) also
includes the [cancelled-startup follow-up](DISCOVERY-LIMITED-STARTUP-20261022.md). The [patch record](../../vendor/mdns-sd/OPENERGY-PATCH.md)
and `verify-source.py` explain and verify the source inventory.

`browse_limited(type, 64)` uses a shared map of at most 64 selected resolved
services on the existing worker. Selected peers retain their slots until
removal; later resolved peers can fill empty slots. Updates replace the same
peer's state. There is no unselected waiting list. A capacity-one channel holds
initial subscription identity/lifetime only. It does not transport service
updates. The wrapper replaces its complete view after each change, including
removals that occur while its caller is paused. No forwarding thread, extra
worker, event-capacity increase or polling timer was added.

Limited initial replay reads one PTR alias at a time, skips incomplete services,
and selects at most 64 ready services. It does not allocate a namespace-sized
alias backlog. It can scan the full existing cache; this is not constant-time
work. The original standard browse API still overflows, as the retained controls
show. Openergy uses the new opt-in path for initial browse and retries.

The ordinary probe also exposed a separate shared-host goodbye defect in the
publisher: removing one service withdrew an address still used by its sibling
registrations. That invalidated the selected survivors and let random peers
occupy their slots. The narrow dependency correction preserves only addresses
still owned by another local registration on that interface. Each removed
service's PTR/SRV/TXT goodbye is still sent. Full worker cleanup retains its
original address-withdrawal behavior. The probe uses the patched library for
both publisher and consumer; this does not repair incorrect goodbyes emitted
by external, unpatched publishers.

## Real-process results

Runtime: macOS 27.2 (26B5112a), Rust 1.99.1, deployment target 15.0, nice 15,
two build jobs. No unit tests, mocks, crafted packets, native input backend,
installed service or live daemon were used.

The fresh baseline (`baseline-probe.log`, exit 1) reproduced the original 10/13
result. Standard retained replay twice drained exactly 1,000 events and closed.
The selected live update appeared but did not remain fresh. All three retained
namespace gates failed. Existing failures were not overwritten.

Two intermediate patched runs also failed and remain in the evidence:

- `patched-probe-1.log`: original retained checks passed, but the new peer after
  first-peer removal was absent.
- `patched-probe-2.log`: the new peer remained absent, and limited replay selected
  zero ready services because the first 64 PTR records were incomplete.

Those failures led to the shared-host goodbye correction and ready-service
replay selection. `patched-probe-3.log` then passed the expanded checks. The
final probe adds paused-update and paused-goodbye checks; its raw result is
`patched-probe-final.log` (exit 0), 19/19 checks. An independent repeat
(`patched-probe-repeat.log`, exit 0) also passes 19/19:

- The independent standard API control still overflows at 1,000 events; limited
  replay on that same retained worker reaches 64 and stays open.
- The wrapper stays useful while 800 ordinary burst registrations remain live.
  The chosen set stays stable. A selected peer's TXT name, port and untrusted
  fingerprint hint update correctly and remain fresh for two seconds.
- That selected peer's goodbye removes it without regression. A replacement is
  registered first, so 800 burst services remain live through that removal.
- The original first peer then disappears. A new peer advertised after its
  goodbye appears, and the other selected peers remain stable.
- While the caller pauses, two updates coalesce to the latest state; a later
  goodbye is absent on the next full view.
- The original clear/recovery, changed-port/hint, final empty-view and publisher
  exit controls pass. All 13 emitted snapshots respect one-second spacing and
  the peak view is 64.

`patched-shutdown-final/report.json` passes all four actual owned-lifetime
cases: normal Drop, normal advertised shutdown, descriptor-exhausted Drop and
advertised shutdown. Each returns from two threads/five UDP descriptors to one
thread/no UDP descriptors while its private child stays alive. Cleanup took
51, 51, 834 and 821 ms. The explicit exhausted shutdown still reports the real
`Too many open files (os error 24)` wake error. These checks observe the owned
worker and do not substitute a sentinel acknowledgement for cleanup.

At ten seconds into the retained run, the probe process had three threads
(main plus two mDNS workers), ten UDP descriptors, and 10,576 KiB RSS. The
publisher is a separate process and is not included in that point sample.
The final `/usr/bin/time -l` result reports 11,796,480 bytes maximum resident
set, 41.21 seconds elapsed, 3.17 user seconds and 0.36 system seconds. These
are debug-probe measurements, not a speedup or a whole-process memory bound.

## Validation and notices

The locked offline build, strict Clippy for openergy-net and mdns-sd, scoped
Rust formatting, exact archive/source integrity check and notice `--check`
passed. Vendor formatting uses upstream width 100; wrapper formatting uses the
repository configuration. Existing tests were neither modified nor executed.
The product/evidence diff passes whitespace checks. Whole-vendor diff checks
report two original trailing-space lines and blank context lines in the exact
patch file; these are retained to preserve original source and patch bytes.

`tools/dependency-audit` is absent from this starting checkout. The existing
notice generator skipped every path crate. It now skips workspace members but
includes this exact approved vendor path, records local modification and upstream
identity, and rejects unknown third-party paths. Regenerated inventory and
THIRD_PARTY_NOTICES retain mdns-sd's license. No fresh advisory or platform-wide
policy acceptance is claimed. An initial notice-generator implementation error
is preserved in `notices-generate.log`; the corrected generation and check pass.

Reproduce from the repository root with the pinned Cargo bin directory on PATH:

```sh
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 cargo +1.99.1 build \
  --locked --offline -p openergy-net --example discovery_recovery \
  --example discovery_owned_shutdown -j 2
nice -n 15 target/debug/examples/discovery_recovery probe NEW_PRIVATE_DIR 25090 25091
nice -n 15 python3 crates/net/examples/discovery_owned_shutdown.py \
  target/debug/examples/discovery_owned_shutdown OTHER_NEW_PRIVATE_DIR 25094 25095
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 nice -n 15 cargo +1.99.1 clippy \
  --locked --offline -p openergy-net -p mdns-sd --lib \
  --example discovery_recovery --example discovery_owned_shutdown --no-deps -j 2 -- -D warnings
nice -n 15 python3 vendor/mdns-sd/verify-source.py ORIGINAL_CRATE_ARCHIVE
nice -n 15 python3 docs/verification/public-release-audit-2026-10-21/generate-notices.py --check
```

## Remaining boundaries

This bounds delivery state by peer count. It does **not** establish a total DNS
cache, record-byte, timer, pending-query, or temporary packet-work bound. The
mutex can briefly hold the worker while a caller clones up to 64 variable-sized
services. There is no automatic admission guarantee for an arbitrary new peer
when all slots are occupied. Standard event consumers retain upstream overflow
behavior. These limits are explicit, not repaired by stale snapshots or retries.

Expiry runtime remains excluded, provider-stopped and unverified. No NSEC or
orphan-record expiry runtime work was attempted. Native acceptance, positive
Windows migration, provenance/security and trusted distribution gates remain
open. No push, publish, PR/settings change, thread contact or agent delegation
occurred.

All ordinary listeners used private loopback UDP 25090/25094; advertised service
ports were 25091/25092/25095. Parent-reserved 25098/25099 were untouched. Upstream
command-wake sockets retain their existing OS-assigned loopback ports. No 24800,
5353, global limits, DNS, firewall, Docker, native input, clipboard, credentials,
TLS identity, login/TCC or desktop state was changed.

New [raw evidence](examples/discovery-bounded-evidence-20261022/) retains result
values and removes personal host paths/usernames and trailing whitespace. Exact
private originals remain under `/tmp/oe-bounded-discovery-20261022-5a69/`.
The required cherry-pick's earlier raw evidence remains separate and unchanged.
