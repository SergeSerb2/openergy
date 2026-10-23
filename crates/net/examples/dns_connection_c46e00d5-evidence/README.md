# DNS cancellation and connection-readiness follow-up

The confirmed product bug is fixed in `crates/net/src/reactor.rs`. The final
expanded real-interface probe exits **0**. The candidate's byte-identical original
full probe also exits **0** against the fixed library. The original candidate's
baseline and patched failure reports remain unchanged and both still record
exit **1**. No failed result was replaced with a passing result.

## Source and failure

The worktree started clean at parent `54aa50b9eafc51f6eef2589c100c509eaac7e415`.
Its descendants after runtime source `ac5c45081a9e060392cb850eddd625f07b591bfe`
change evidence files only. Candidate `a8a40f45cf66948bf1cda1f8b859de81f13f0c94`
was cherry-picked first as `615c16e`. Its DNS cancellation flags, deadlines, single
worker, 16-job bound, and active-lookup completion accounting are preserved.

The [original full reproduction](raw/original.log) exits 1. It reports a false
`Connected` event for `localhost:25117`, then `Disconnected { reason: ClosedByPeer }`.
The [smaller refusal matrix](raw/matrix-before.log) also exits 1: direct numeric
IPv4 and IPv6 emit typed refusal, while all three hostname attempts fail the
control. Exclusive standard-library socket binds and direct OS connects confirm
both loopback addresses are free and refuse connections with errno **61**.

Temporary [readiness tracing](raw/trace.log) shows the cause. An IPv6 refused
socket receives writable/closed readiness and `SO_ERROR=ECONNREFUSED`. Address
fallback replaces it with IPv4 under the same connection token. The next event
for that token is readable/closed. On the new socket, `take_error` returns
`Ok(None)`, while `peer_addr` returns errno **57**, `NotConnected`. The old code
sets `connecting=false` from empty `SO_ERROR` alone and announces success.
This agrees with Mio 1.2.4's `TcpStream::connect` source documentation: check
`peer_addr` after `take_error` before using the stream.

The fix requires `peer_addr` success as well. `NotConnected` and Unix
`EINPROGRESS` retain the pending connection and its existing deadline. Other
errors use the existing address fallback and typed failure path. It adds no
thread, dependency, retained history, sleep, or resolver interposition. Temporary
trace code is absent from the final product source.

## Real interface validation

The [final full report](raw/full-final.log) exits 0 with:

- Eight batches of 16 localhost jobs: two Shutdown, two DisconnectAll, and four
  one-nanosecond deadline batches. Each rejects request 17, emits one failure per
  obsolete request, recovers the DNS budget, and opens no stale TCP connection.
- A healthy IPv4 listener at 25116 and real bidirectional length-prefixed frames
  after each batch. The OS resolves localhost IPv6 first, so these runs also
  exercise refusal-to-IPv4 fallback and verify the final peer address.
- Ten typed-refusal requests across ports 25117 and 25119: numeric IPv4, numeric
  IPv6, and three hostname attempts per port. All return exactly one typed
  `connect_refused` event and no `Connected` event. Both addresses pass exclusive
  bind and direct OS refusal checks at both ports. The connection deadline is
  five seconds and the event wait is bounded to six seconds, as in the intent of
  existing control `db0d0bd`; classification was not relaxed to timeout.
- Healthy direct IPv6 and hostname connections to the IPv6 listener, with real
  bidirectional frames. The same driver, original handle and waker survive
  resets, reconfiguration, rejected configuration, and further connections.
- A single classified embedded-NUL resolver-input failure and handle rejection
  after driver drop. This is not a public-DNS NXDOMAIN test.

The [unchanged original probe report](raw/original-probe-after.log) exits 0 too.
Its source was extracted from the candidate git object and compiled directly
against the fixed Cargo-built library. Its SHA-256 is identical to the candidate
probe hash. This controls for changes to the expanded probe's assumptions.
Intermediate one-port [before](raw/matrix-before.log), [trace](raw/trace.log), and
[after](raw/matrix-after.log) reports are retained as intermediate evidence.

Pinned Rust 1.99.1 scoped [build](raw/final-build.log), strict
[Clippy](raw/final-clippy.log), rustfmt check, and whitespace check all exit 0.
The commands, source/artifact hashes, exit codes and unchanged historical report
hashes are in [source-and-validation.json](source-and-validation.json).
No unit tests or mocks were added or run.

## Repeat and boundaries

Run from this checkout with fresh private fixture paths and free loopback TCP
25100 through 25119. IPv6 loopback must be available. The minimal mode uses only
25117 and 25119; the full mode uses the complete range.

```sh
umask 077
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=1.99.1 \
  nice -n 15 "$HOME/.cargo/bin/cargo" +1.99.1 build --locked --offline -j 2 \
  -p openergy-net --lib --example dns_queue_cancel_c4806a3d
nice -n 15 target/debug/examples/dns_queue_cancel_c4806a3d /tmp/NEW-PRIVATE-FULL-DIR
nice -n 15 target/debug/examples/dns_queue_cancel_c4806a3d /tmp/NEW-PRIVATE-MATRIX-DIR --refusal-only
MACOSX_DEPLOYMENT_TARGET=15.0 CARGO_BUILD_JOBS=2 RUSTUP_TOOLCHAIN=1.99.1 \
  nice -n 15 "$HOME/.cargo/bin/cargo" +1.99.1 clippy --locked --offline -j 2 \
  -p openergy-net --lib --example dns_queue_cancel_c4806a3d -- -D warnings
nice -n 15 "$HOME/.cargo/bin/rustfmt" +1.99.1 --check --edition 2021 \
  crates/net/src/reactor.rs crates/net/examples/dns_queue_cancel_c4806a3d.rs
git diff --check
```

Only private fixtures and loopback TCP sockets were used. Discovery and TLS were
disabled. No daemon or native backend ran. No live sharing, installed binaries,
services, configuration, credentials, identities, input, clipboard, desktop,
login/TCC, global limits, DNS, firewall, Docker, port 24800 or mDNS 5353 changed.
Discovery, Cargo, dependency and vendor source files were not edited.

**Slow active OS DNS lookup and deterministic slow-DNS queue timing remain
unverified. Cache-expiry runtime remains unverified.** No crafted upstream
expiry test was run or taken over. Native acceptance, positive Windows migration,
provenance/security and trusted distribution gates remain open. Private GitHub
reporting remains deferred; no push, publication, merge or repository settings
change occurred.

Personal Cargo paths in historical README/metadata were changed to `$HOME` only.
The original baseline and patched failure reports are byte-identical to the
candidate. An initial [evidence path audit](raw/evidence-audit-initial.log) failed
because inherited compiler logs still held personal worktree prefixes. Those
prefixes and new build-log prefixes were replaced with `<WORKTREE>` only; numeric
result values, errors and exits are retained. Exact original candidate evidence
remains in the private fixture directory outside git. The corrected audit passes.
Local timing samples are observations, not performance guarantees.
