# WP4 transport verification

Verified on macOS, 2026-10-21, Rust `1.99.1 (58cf8f9c3 2026-10-13)`.
Only `crates/net/` is part of this change. No unit tests, mocks, platform input,
event taps, cursor changes, pasteboard writes, TCC requests, native Synergy
processes, production-port sockets, or LAN connections were used.

## Driver contract and integration notes

The binding `NetConfig`, `MdnsConfig`, `NetDriver`, `ConnInfo`, `NetHandle`, and
all requested methods are implemented. `NetDriver` owns the sole transport
`mio::Poll`; the daemon calls `poll(timeout, out)` on its io thread. Commands
from other threads use the bounded handle and the same `Arc<mio::Waker>`.
The public waker also lets unrelated daemon work interrupt that poll.

- `Shutdown` immediately closes the listener and all connections, emits
  `Disconnected` for each, and keeps the driver, handle, and waker usable.
  A cancelled outbound attempt that had not yet produced a result also emits
  `ConnectFailed`. `StopListening` keeps existing connections.
- `Close` flushes pending Control frames and TLS close-notify for at most
  2 s, retrying a blocked socket every 10 ms. Send/flush CBYE before
  `Shutdown` when a graceful protocol exit is needed.
- `reconfigure` changes future connections. Existing connections retain their
  original TLS connection, frame limit, trust path, and hold deadline.
- `Connect` produces one initial Connected, UntrustedPeer, or ConnectFailed.
  Held approval later produces Connected; rejection/expiry produces Disconnected.
  A held socket is removed from mio after its TLS flight drains. Its application
  reader/writer remains gated; only a 500 ms trust-file timer and expiry run.
- Trust files are sets of allowed certificate identities, without hostname
  bindings. `ConnectPinned { host, port, fingerprint }` expresses an expected
  identity and rejects a changed certificate in the TLS verifier. An unknown
  identity on ordinary `Connect` enters HELD, including when other peers are
  already trusted. No failure changes the connection to plaintext.
- `approve_held(fp, out)` persists the correct server/client role and resumes all
  matching holds. `trust_store_mut()` mutations are checked on the next poll;
  `ReloadTrust` also applies revocations to live trusted connections. While a
  hold exists, external file changes are read within 500 ms. Offline CLI approval
  has no role context, so `trust approve` adds both roles. Live reject uses IPC;
  the offline `trust reject` command gives a clear IPC instruction.
- Input absolute moves coalesce only at the unsent Input queue tail. Discrete
  messages break a coalescing run; partly transmitted frames are never replaced.
  Control precedes other queues during the protocol handshake. Sending/receiving
  DSOP ends that phase. Frame diagnostic mode starts with this phase complete.
- Bulk frames must fit **32768 bytes including their u32 prefix and message
  fields**. Thus a DCLP Data payload can contain at most **32750 bytes**.
  Larger Bulk frames return `NetError::InvalidFrame`; the caller must chunk them.
  Bulk has a 256 KiB per-peer admission window; retry `QueueFull` after polling.
  The global mailbox/queued-byte budget is 8 MiB, with 256 command slots.
  More than 4 MiB of queued Input/Control marks SlowPeer, clears queued
  Input/Bulk, flushes Control for one heartbeat (default 3 s), then closes.
- Connect/DNS/TLS has a default 10 s deadline; the TLS hold defaults to 30 s.
  OS name resolution uses one lazy bounded worker so getaddrinfo cannot block
  the io thread. Numeric connections create no resolver thread. IPv4/IPv6
  resolved candidates are tried within the original deadline.
- A default **100 ms first-ClientHello timer** works around the inherited
  Deskflow/Synergy accept readiness race. The lab's existing `smoke.py` also
  documents and waits for this race. Immediate first flights stalled against
  fresh Deskflow launches; two fresh runs passed after this change. This timer
  does not sleep the io thread and does not affect established input latency.
  `set_client_hello_delay(Duration::ZERO)` is available for known fast peers.
- TLS uses ring, TLS 1.2/1.3, no roots/ALPN/PKIX validation, and real handshake
  signature checks. The client config always supplies its certificate when
  requested, including authenticated Deskflow security level 2.
- Identity files are exactly `NetConfig.tls_dir/{cert,key}.pem`, both 0600 on
  Unix. The CLI's `--dir` is the parent config directory, so it appends `tls/`.
  `TrustStore::load` takes the TLS directory itself. Files are fsynced and renamed
  atomically; writer directory locks prevent lost concurrent updates. Partial
  or corrupt identities produce errors rather than silent regeneration.
- Added diagnostic surfaces are `TransportMode::Frames`, `SendFrame`, and
  `NetEvent::Frame`. Normal mode always uses frozen `protocol::encode`/`Decoder`,
  including the first hello/hello-back frame. Frame commands are refused in
  normal mode. Other additive commands: Approve, Reject, ReloadTrust,
  ConnectPinned, StopListening. Other additive events: Error, ListenFailed,
  ConnectFailed. `tls_cipher(conn)` supplies negotiated cipher diagnostics.

## Build gates

From the repository root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt -p openergy-net -- --check
cargo build --release
cargo build --release -p openergy-net --examples
cargo clippy --release -p openergy-net --all-targets --no-deps -- -D warnings
```

All pass with zero warnings. The required command was also run exactly:

```sh
cargo clippy --release -p openergy-net -- -D warnings
```

It stops in the unchanged dependency `crates/config/src/lib.rs:188`:

```text
error: this `impl` can be derived
188 | impl Default for SwitchCfg {
= note: `-D clippy::derivable-impls` implied by `-D warnings`
error: could not compile `openergy-config` (lib)
```

No file outside WP4 was changed to suppress it. `--no-deps` checks all WP4
library/example targets cleanly. Cargo refreshes the workspace lock during
builds; the generated root lockfile change is not committed because the work
package owns only `crates/net/`. The integrator must refresh the shared lock.

## CLI and real sockets: S6, S7, S21

```sh
target/release/openergy fingerprint --dir /tmp/oe-wp4/a --synergy
stat -f '%Lp' /tmp/oe-wp4/a/tls/key.pem
target/release/openergy fingerprint --dir /tmp/oe-wp4/a --synergy
python3 crates/net/examples/verify_transport.py
```

The fingerprints matched `^v2:sha256:[0-9a-f]{64}$`, were identical on repeat,
and the key mode was `600`. The retained script drives two **real** `tls_peer`
processes, the real `openergy` CLI, and a transparent TCP capture relay:

```text
S21 fingerprint regex=PASS mode=0600 stable=PASS
TLS trusted two-way frames=PASS
S6 HELD no frames; API approve -> two-way frames=PASS
server-side HELD -> API approve; only trusted-clients updated=PASS
HELD trust file reload -> two-way frames=PASS
S7 wrong pin refused; captured_tls_bytes=898 plaintext_hello=absent
HELD deadline -> close with no frame=PASS
explicit plaintext two-way frames=PASS
hostile oversized prefix -> BadFrame=PASS
trust import/dedup/SHA1 ignored/invalid CLI=PASS
```

Captured traffic is encrypted TLS, not a plaintext fallback. Artifacts include
both process logs and `wrong-pin.tls` in the printed temporary directory. One
run is `<temporary-directory>/oe-wp4-<run-id>`.
No private keys or application input/clipboard content appear in these logs.
The real CLI also returns status 2 for unknown fingerprint/trust flags and a
missing `--dir` argument, with no identity-file creation on those paths.

## Backpressure, deadline, and cross-thread wake

```sh
target/release/examples/guardrails /tmp/oe-wp4/guardrails
```

This uses a real stalled TCP reader, a real listener that never completes TLS,
an IPv4 loopback listener, a separate command producer thread,
and a real control-frame flood sender. Representative output:

```text
REFUSED exactly_one_ConnectFailed=PASS
SLOW_PEER queued_nonbulk_over_4MiB close_ms=3001 reason=SlowPeer
HANDSHAKE_DEADLINE close_ms=120 exactly_one_ConnectFailed=PASS
CROSS_THREAD_WAKER wake_ms=41 reused_driver=PASS
FLOOD valid_frames=3346432 close_ms=5012 reason=Flood
```

## Live role/TLS reset with the original driver and handle

```sh
target/release/examples/lifecycle /tmp/oe-wp4/lifecycle
```

One driver listens with TLS, exchanges frames with a real child process,
stops listening while the child still receives CNOP, resets all transport,
reconfigures TLS off, then repeats on a different port:

```text
CYCLE tls=true port=24974 bidirectional_frames=PASS StopListening_keeps_peer=PASS Shutdown_disconnected=1
CYCLE tls=false port=24975 bidirectional_frames=PASS StopListening_keeps_peer=PASS Shutdown_disconnected=1
REUSE same_driver=PASS same_waker=PASS original_handle=PASS TLS_on_to_off=PASS
```

## Priority under a 10 MiB transfer

`python3 crates/net/examples/verify_examples.py` creates the identities, seeds
trust with the real CLI, waits for listener readiness, and runs all the
priority/mDNS/lifecycle/guardrail examples. To run the priority pair separately,
pre-seed each directory with the other identity, then run:

```sh
target/release/examples/priority_peer server /tmp/oe-wp4/a 24973 &
server_pid=$!
target/release/examples/priority_peer client /tmp/oe-wp4/b 24973
wait "$server_pid"
```

The sender streams 10 MiB through the bounded Bulk window while queuing an
Input frame on every iteration. The receiver checks actual wire frame order
and echoes the Input diagnostic timestamp:

```text
SENT bulk_bytes=10485760 input_frames=321 echo_samples=321 p99_echo_us=455
RECEIVED bulk_bytes=10485760 input_frames=321 max_bulk_frames_between_input=1 max_slice=32768
```

This is a transport scheduling measurement, not an OS input latency claim.

## mDNS on loopback only

```sh
target/release/examples/discovery browse /tmp/oe-wp4/a &
browse_pid=$!
target/release/examples/discovery advertise /tmp/oe-wp4/a
wait "$browse_pid"
```

Two processes use UDP 24979. All interfaces are disabled before loopback IPv4
is enabled, and before any browse or registration command. Output:

```text
ADVERTISE _openergy._tcp.local. name=wp4-loopback ver=1.8 fp=a0654fb05f3a3702
RESOLVED name=wp4-loopback host=openergy-a0654fb05f3a3702.local. port=24970 fp=Some("a0654fb05f3a3702")
```

Production `MdnsConfig` uses standard UDP 5353. `loopback_only=false` enables
LAN advertisement/browsing on mdns-sd's default interfaces, excluding Apple
P2P interfaces by its default policy. Discovery never connects or trusts;
`fp` is only a display hint. mdns-sd owns its internal worker thread. Changed
peer snapshots are emitted at most once per second. `mdns=None` disables it.
No test service was advertised on a LAN interface.

## Real Deskflow 1.27 TLS interop, both directions

```sh
export LAB_NAME=oe-lab-wp4 LAB_PORT_RANGE=24920-24929
python3 crates/net/examples/verify_interop.py
```

The script uses the unmodified lab and always calls `lab.sh down` in `finally`.
It starts the real Deskflow server with TLS and our client fingerprint, then
runs our client with the lab server fingerprint in trusted-servers. It then
reverses roles: our loopback server trusts the lab client certificate and the
lab client pins our server certificate. No input/clipboard messages are sent.
Trimmed real output from two consecutive fresh runs:

```text
OUR_CLIENT ...
CONNECTED conn=2 tls=true version=Some("TLSv1_3") cipher=Some("TLS13_AES_256_GCM_SHA384") elapsed_ms=104
FRAME conn=2 bytes=11 elapsed_ms=116
HELLO 0000000b53796e6572677900010008
fingerprint matches trusted fingerprint
WRONG_PIN ...
CONNECT_FAILED TlsFailure: unexpected error: certificate fingerprint mismatch delivered=0
OUR_SERVER ...
CONNECTED conn=2 tls=true version=Some("TLSv1_3") cipher=Some("TLS13_AES_256_GCM_SHA384") elapsed_ms=345
FRAME conn=2 bytes=18 elapsed_ms=356
openssl cipher: TLS_AES_256_GCM_SHA384 TLSv1.3
DESKFLOW_TLS_BOTH_DIRECTIONS=PASS artifacts=/tmp/oe-wp4/interop-final
```

The 18-byte frame is Deskflow's Synergy 1.8 hello-back with name `lab`.
The lab server's successful pin check plus application hello proves that our
client supplied its certificate, rather than merely receiving a server cert.
Logs are retained under `/tmp/oe-wp4/interop-final`.

TLS 1.2 was also exercised with the real LibreSSL 3.3.6 CLI, with peer `b`'s
certificate/key and a CAfile containing only peer `a`'s public certificate:

```sh
target/release/examples/tls_peer server --dir /tmp/oe-wp4/a --addr 127.0.0.1:24970 --seconds 2
openssl s_client -tls1_2 -quiet -connect 127.0.0.1:24970 \
  -cert /tmp/oe-wp4/b/tls/cert.pem -key /tmp/oe-wp4/b/tls/key.pem \
  -CAfile /tmp/oe-wp4/a/tls/cert.pem
```

A framed hello-back was supplied on stdin. Real output:

```text
OpenSSL received hello= True
CONNECTED conn=2 tls=true version=Some("TLSv1_2") cipher=Some("TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256")
FRAME conn=2 bytes=18
```

## Remaining integration gates

The protocol crate in this checkout still returns `NotImplemented` from
`encode`/`Decoder::feed`. As the work package permits, these transport checks
use explicit **frame mode**, not a substitute codec. Normal `Message` flows,
message-level absolute-move coalescing, and daemon IPC trust approval need WP1
and WP6 integration before they can be exercised end to end. No other crate
or frozen public signature needs to change for those paths.

The final lab state is stopped (`oe-lab-wp4`, `running=false`). Only assigned
TCP ports 24920-24929 and 24970-24979, plus loopback UDP 24979 for discovery,
were used. The production environment override was never enabled.
