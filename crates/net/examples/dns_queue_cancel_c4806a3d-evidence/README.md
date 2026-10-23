# DNS queue cancellation and deadline evidence

Starting source: clean isolated checkout on `t3/cancel-stale-dns-lookups`, exactly
`ac5c45081a9e060392cb850eddd625f07b591bfe`. The only product change is in
`crates/net/src/reactor.rs`. The probe uses public `NetDriver`/`NetHandle` commands,
the real standard-library resolver, real TCP listeners, and bidirectional frames.
It uses IPv4 loopback TCP 25100 through 25118, with discovery and TLS disabled.
Every run requires a new absolute directory. It does not use a native backend,
input, clipboard, production configuration, or default IPC.

## Source finding and change

The baseline clears `pending` on Shutdown/DisconnectAll and at a connect deadline,
but its single resolver worker still calls `to_socket_addrs` for each queued job.
The real interface confirms that 16 obsolete localhost jobs can retain the DNS
budget and reject a healthy hostname until completions are processed.

Each accepted job now shares one bounded cancellation flag with its pending owner
and carries the same connect deadline. Shutdown, DisconnectAll, timeout removal,
and driver drop set the flag. The worker checks the flag and deadline immediately
before entering the OS resolver. Every consumed job takes the existing single
completion path, including skipped jobs. Only completion processing decrements
`dns_outstanding`; cancellation does not free an active lookup's slot. Existing
pending-ID checks suppress late connections and duplicate public failures.
There are still at most 16 outstanding jobs, one lazy worker, no new dependency,
and no retained cancellation history. An OS call already entered cannot be stopped.

## Canonical raw reports

- [Baseline](raw/baseline-report.log): archived `ac5c450`, after an explicit net
  library rebuild; final probe source.
- [Patched](raw/patched-report.log): patched checkout, after an explicit net
  library rebuild; identical probe source.
- [Source hashes and validation](source-and-validation.json).
- [Final strict Clippy](raw/validation-clippy.log).
- [Baseline forced rebuild](raw/baseline-forced-build.log) and
  [patched rebuild](raw/patched-report-build.log).

Both canonical runs exercised eight batches of 16 actual localhost jobs: two
Shutdown batches, two DisconnectAll batches, and four one-nanosecond deadline
batches. Every batch rejected the seventeenth request, emitted exactly one failure
per obsolete request, recovered to the real healthy listener, exchanged frames,
and opened no stale TCP connection. Cancellation retained outstanding slots until
completion accounting. The same driver, original handle, and waker survived
DisconnectAll, Shutdown, valid reconfiguration, and a rejected reconfiguration.
The probe also verified numeric-loopback recovery and a rejected handle after drop.

The four baseline deadline batches each rejected two recovery admissions; their
recovery admission times were 156, 182, 160, and 137 microseconds. The four patched
deadline batches rejected none and each admitted recovery in 2 microseconds.
Actual frame roundtrips were 3,221–3,519 microseconds before and 1,630–1,734 after.
These are local observations, not a deterministic slow-DNS benchmark or a general
latency guarantee. Cancellation admission did not consistently improve: the active
lookup and completion accounting still apply. All eight roundtrip samples and
admission counts are in each raw report.

## Failed controls and limits

**Both canonical probe commands exit 1.** The real refused-localhost control on
the confirmed free TCP port 25117 reports `Connected` then `Disconnected` with
`ClosedByPeer`, rather than a typed `connect_refused` failure. It fails on the
original source and patched source. The probe preserves the events and continues
other checks, then returns failure. This result is unresolved; this DNS change
does not fix it. The embedded-NUL hostname control does emit a single classified
`dns_failed` event through the real API. That control exercises resolver input
rejection, not a public DNS NXDOMAIN response.

The first probe versions assumed `Connected` meant an immediate nonblocking
listener accept would succeed. Their real `WouldBlock` failures are preserved in
`raw/baseline-1.log` and `raw/baseline-2.log`. The final probe waits within a fixed
deadline and requires a real frame roundtrip. The initial strict-Clippy failure
and correction are preserved in `raw/clippy.log` and `raw/validation-clippy.log`.

`raw/baseline-final.log` is **excluded from comparisons**. That archived-source
attempt reused the shared Cargo target without forcing the older library's source
mtime to invalidate cached artifacts. Its binary's baseline provenance was not
established. The archived reactor's contents were then checked against the named
git object, its source timestamp was refreshed, and the net library was explicitly
rebuilt. `raw/baseline-forced.log` and the canonical `raw/baseline-report.log`
follow that rebuild. The patched net library was explicitly rebuilt afterward.
All preliminary raw reports and build failures remain in `raw/`.

No deterministic slow OS lookup was available under the private-loopback-only
constraint. No resolver was mocked or interposed, and no product sleep was added.
The probe does not measure a stalled active OS lookup, prove its cancellation,
or validate public-DNS negative responses. It uses the permitted real localhost
queue/admission/deadline evidence. The active-slot accounting guarantee also rests
on inspection of the unchanged single-completion/decrement path. No Windows,
native desktop, clipboard, TLS distribution, or release acceptance is claimed.
The original release-provenance and Windows privilege findings remain open;
private reporting, native acceptance, Windows migration, and trusted distribution
remain external gates. No PR, push, merge, publication, or live service change ran.

## Repeat the scoped check

```sh
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=1.99.1 \
  nice -n 15 /Users/serge/.cargo/bin/cargo +1.99.1 build --locked --offline -j 2 \
  -p openergy-net --example dns_queue_cancel_c4806a3d
nice -n 15 target/debug/examples/dns_queue_cancel_c4806a3d /tmp/NEW-UNUSED-DNS-PROBE-DIR
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=1.99.1 \
  nice -n 15 /Users/serge/.cargo/bin/cargo +1.99.1 clippy --locked --offline -j 2 \
  -p openergy-net --lib --example dns_queue_cancel_c4806a3d -- -D warnings
```

The actual baseline and patched invocation directories were
`/tmp/openergy-dns-c4806a3d-baseline-report` and
`/tmp/openergy-dns-c4806a3d-patched-report`. Builds used Rust 1.99.1, two jobs,
nice 15, deployment target 15.0, locked dependencies, and offline mode.
The build, strict Clippy, scoped rustfmt check, and whitespace check pass.
The real refusal control remains a reported failure.
