# Openergy mdns-sd delivery patch

This is the exact published mdns-sd 0.21.6 source, with a local patch in
`src/service_daemon.rs` and its public export in `src/lib.rs`.

- Upstream: https://github.com/keepsimple1/mdns-sd
- Published revision: `1a5cdb6ebb7299ee74dda75099461a2921b6e740`
- Archive: https://static.crates.io/crates/mdns-sd/mdns-sd-0.21.6.crate
- Original archive SHA-256:
  `8562654710f72a9cbce4b0d5b7c366a2f94330ba1ce3740f15e932ea2d37b161`
- Licenses: unchanged `LICENSE-MIT` and `LICENSE-APACHE`.

`UPSTREAM.json` records **original** file hashes separately from **modified**
file hashes. It is provenance metadata, not a Cargo package checksum. The root
lockfile identifies a path dependency and does not attach the original archive
checksum to modified files. No `.cargo-checksum.json` is shipped.

The published `.cargo_vcs_info.json` and separate `Cargo.lock` are excluded.
Their revision/archive identity is recorded here instead. All other published
files, including existing tests and examples, remain. No upstream unit tests
were added, changed or run. No generated binaries or local registry metadata
are included. `OPENERGY.patch` is the complete code delta against that archive.

## Delivery behavior

The opt-in `ServiceDaemon::browse_limited(type, limit)` accepts limits 1..=64.
It keeps the latest resolved state of at most that many selected peers on the
existing worker, behind a mutex. Existing peers keep their slots until removal.
New resolved peers can fill empty slots; there is no unselected waiting list or
automatic refill. Repeated updates coalesce. `take_snapshot` returns a complete
view only after a change; callers must replace their view, so paused consumers
observe removals. The wrapper drops a closed view and clears its old peers.

A capacity-one channel carries initial subscription identity/lifetime, not
service-state events. There is no delivery queue, forwarding thread, wake timer,
worker recreation or increased event capacity. The normal upstream browse API
keeps its original capacity and overflow behavior.

Initial limited replay walks the existing PTR records one at a time and selects
at most 64 valid resolved services. It does not allocate an alias list or stop
at incomplete records before reaching ready services. The walk can still scan
the full existing cache; its CPU work is not a constant-time cache bound.

The same real-process probe exposed a shared-host goodbye defect: unregistering
one ordinary ServiceInfo sent a zero-TTL address even while other local services
used that address. The patch preserves only addresses still owned by another
registration on that interface. Full worker cleanup still withdraws addresses.
This lets selected peers remain valid when their sibling service disappears.
It does not keep a stale snapshot or ignore a peer's own PTR/SRV/TXT goodbye.

## Explicit limits

This is a delivery-state bound, not a total-memory bound. DNS records, timers,
resolver bookkeeping, per-packet temporary work, and upstream pending queries
retain their upstream policies. Each selected service still has variable-sized
DNS data. No DNS cache/expiry algorithm is changed. Crafted expiry testing was
excluded and remains runtime unverified. No native or distribution gate closes.
The parent must independently review this local delta before acceptance.

## Verify source

Run from the repository root:

```sh
python3 vendor/mdns-sd/verify-source.py /private/path/mdns-sd-0.21.6.crate
```

The optional archive argument also checks the downloaded original archive and
all retained original file hashes. Without it, the check verifies every local
file against the distinct original/modified hashes and rejects extra files.
