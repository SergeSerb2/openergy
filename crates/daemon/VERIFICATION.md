# WP6 verification — 2026-10-21

All checks use real executables, local IPC, real TCP/TLS sockets, or real
Deskflow/Barrier processes in Docker. No unit tests, mocks, or test doubles were
added. Source references are the project contracts and documented wire facts.

## Build gates

macOS ARM64, Rust 1.99.1:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --workspace
cargo clippy --release --workspace --all-targets -- -D warnings
```

```text
Finished `release` profile [optimized]
Finished `release` profile [optimized]
```

Both commands exit 0 with zero warnings. `SwitchCfg` derives `Default`.
Native clipboard backends return typed limit errors so the daemon publishes
`clipboard.blocked`, including aggregate size/limit, without fetching oversized
payloads where the OS offers length metadata. The Linux check exercises that
path with a real X11 selection.

The Linux workspace also builds and passes the same clippy command in the
isolated `oe-linux-dev-wp8:rust-1.99` image. macOS-only example entry points are
cfg-gated so Linux all-targets checks compile.

## Full catalogue

```sh
LAB_NAME=oe-lab-wp6 LAB_PORT_RANGE=24950-24959 \
OPENERGY_ALLOW_PORT_24800=0 target/release/openergy-e2e run \
  --port-range 24980-24983 --lab-port-range 24950-24958 --keep \
  --report /tmp/oe-wp6-e2e-final.json
```

```text
61/61 passed; JSON report: /tmp/oe-wp6-e2e-final.json
```

| Scenario | Result | Time (ms) |
| --- | --- | ---: |
| I1-barrier-plain | PASS | 31688 |
| I1-barrier-tls | PASS | 32534 |
| I1-deskflow-plain | PASS | 31790 |
| I1-deskflow-tls | PASS | 32430 |
| I2-barrier-plain | PASS | 2622 |
| I2-barrier-tls | PASS | 3158 |
| I2-barrier-wrong-pin | PASS | 1924 |
| I2-deskflow-plain | PASS | 2775 |
| I2-deskflow-tls | PASS | 3301 |
| I2-deskflow-wrong-pin | PASS | 3725 |
| I3-barrier-plain | PASS | 4954 |
| I3-barrier-tls | PASS | 5064 |
| I3-deskflow-plain | PASS | 4522 |
| I3-deskflow-tls | PASS | 4873 |
| I4-barrier-plain | PASS | 3550 |
| I4-barrier-tls | PASS | 4008 |
| I4-deskflow-plain | PASS | 3620 |
| I4-deskflow-tls | PASS | 3668 |
| I5-barrier-export | PASS | 1968 |
| I5-deskflow-export | PASS | 2086 |
| L1-barrier-hello | PASS | 1558 |
| L1-deskflow-hello | PASS | 1632 |
| L2-barrier-clipboard | PASS | 3621 |
| L2-deskflow-clipboard | PASS | 3642 |
| L3-barrier-pair | PASS | 4914 |
| L3-deskflow-pair | PASS | 5306 |
| L4-barrier-raw-client | PASS | 1419 |
| L4-deskflow-raw-client | PASS | 1556 |
| R1-control-files | PASS | 1185 |
| R2-lifecycle | PASS | 1127 |
| S0-smoke | PASS | 384 |
| S1-connect | PASS | 686 |
| S10-clipboard-cap | PASS | 2046 |
| S11-heartbeat | PASS | 2392 |
| S12-display-change | PASS | 601 |
| S13-flood | PASS | 8457 |
| S13-garbage | PASS | 612 |
| S13-oversized | PASS | 604 |
| S13-truncated-hello | PASS | 311 |
| S13-truncated | PASS | 630 |
| S13-unknown | PASS | 1104 |
| S14-import-roundtrip | PASS | 724 |
| S15-resources-latency | PASS | 121810 |
| S16-permissions | PASS | 606 |
| S17-healthy | PASS | 887 |
| S17-same-fingerprint | PASS | 1117 |
| S17-stale | PASS | 1584 |
| S18-click-numbering | PASS | 632 |
| S19-config-patch | PASS | 696 |
| S2-edge-mapping | PASS | 615 |
| S2-wire-order | PASS | 302 |
| S20-parent-exit | PASS | 759 |
| S20-parent-off | PASS | 530 |
| S21-fingerprint-import | PASS | 1180 |
| S3-input-order | PASS | 598 |
| S4-disconnect-release | PASS | 852 |
| S5-disconnect-release | PASS | 943 |
| S6-trust-approval | PASS | 1645 |
| S7-wrong-pin | PASS | 731 |
| S8-text-clipboard | PASS | 1199 |
| S9-large-dib | PASS | 2419 |

The catalogue checks the real IPC/config/status shapes, handshake/wire ordering,
layout mapping, input ordering, releases after death/disconnect/permission loss,
clipboard contents and hashes, cap events, hostile framing/floods, trust holds and
pins, replacement rules, click numbering, lifecycle, import/export and file modes.
The I-series covers Deskflow 1.27.0 and Barrier 2.4.0, both roles, TLS on/off.
The L-series independently checks the Docker lab with real peers and raw sockets.

Corrections made during integration:

- Input follows the observed CINN transition in lab scenarios; one xdotool chain
  could click before the peer completed switching.
- Touching equal-height screens offset by half a height derive half-to-half
  ranges. Source y=810 maps to destination y=270, as physical geometry requires.
- CLSZ is measured in KiB. A 512-byte fixture rounded to zero and disabled
  sharing; the peer-cap case now advertises 1 KiB and sends a larger container.
- I3 explicitly configures a 4 MiB cap. Barrier does not advertise Synergy's CLSZ
  option. Peer-limit negotiation is independently checked in S10.
- S7 supplies an explicit wrong fingerprint. An unrelated global trust-file
  entry is not a host-specific pin and correctly produces a TOFU hold instead.
- CLI import requires a confined output path; fingerprint/trust accept
  `--config`, which selects the same identity directory as the daemon.
- Barrier exports omit unsupported Synergy options and warn about the lost
  clipboard limit/lock/modifier controls. Both real peers accept ranged links.

## Real-process drivers

```sh
python3 crates/daemon/scripts/verify_virtual.py --port-base 24986
python3 crates/daemon/scripts/verify_security_lifetime.py
python3 crates/daemon/scripts/verify_memory.py
python3 crates/daemon/scripts/verify_latency.py
python3 crates/daemon/scripts/verify_errors.py
python3 crates/daemon/scripts/verify_interop.py
python3 crates/daemon/scripts/verify_app.py
```

Final driver artifacts:

| Driver | Result | Artifact |
| --- | --- | --- |
| Virtual S1–S19 coverage | PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-oc13i17a/report.json` |
| TLS/replacement/parent exit | PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-security-28jbbwn5/report.json` |
| Idle + retained memory | PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-memory-5beo91r2/report.json` |
| Timestamped move latency | PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-latency-f_gm2zmu/report.json` |
| Connector errors | PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-errors-tqacfidr/report.json` |
| Docker input interop | 8/8 PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-interop-8pg6q4uh/report.json` |
| Updated app E2E | 14/14 PASS | `/var/folders/s6/_07cr4w13vsg_myjkv9ldpqc0000gn/T/oe-wp6-app-9x95_f8n/output/report.json` |

Trimmed real output:

```text
PASS S9 8 MiB DIB with typing: typing_before_complete=True
PASS S11 heartbeat death: SIGSTOP server -> timeout and release
PASS S13 hostile framing: unknown frame skipped after hello
PASS S17 healthy incumbent EBSY: incumbent stayed connected
PASS S17 stale incumbent replacement: before 3-heartbeat death deadline
PASS S20 parent exit watcher: child_exit=0 peer_goodbye=True socket_removed=True
```

Connector failures now use stable codes and plain sentences. Real socket/OS
resolver checks passed:

```text
connect_refused: No server is listening at 127.0.0.1:24989.
dns_failed: Can't find host openergy-not-a-host.invalid.
connect_timeout: No answer from 127.0.0.1:24987.
tls_pin_mismatch: The server's identity changed — check its fingerprint.
```

The refusal check covers Darwin SO_ERROR handling. TCP_NODELAY is set after an
outbound connection completes. No peer_addr/repeated connect masks a refused
socket. Unreachable OS errors map separately to `host_unreachable`; that path
was not forced with a non-loopback target on this workstation.

## Memory and latency

Idle daemons without a large clipboard use about 10 MB RSS each. A separate
fresh pair measured:

| Node | Baseline RSS | RSS after 8 MiB transfer | Bound: baseline + 2 × payload |
| --- | ---: | ---: | ---: |
| Server | 10,010,624 | 18,595,840 | 26,787,840 |
| Client | 9,830,400 | 19,152,896 | 26,607,616 |

The current clipboard remains available. Reassembly and outgoing transfer blobs
are dropped after completion; the server reads the platform clipboard again when
needed later. macOS daemon allocations of at least 1 MiB bypass malloc's retained
large-block cache and unmap on drop. Input events continue to use fixed state.

The 1,000-move same-host timestamp sample measured:

```text
capture_to_sink_p50_ms: 0.027
capture_to_sink_p99_ms: 0.435
IPC-observed p50_ms:   0.092
IPC-observed p99_ms:   1.6985
```

Capture and sink timestamps exclude the observer's extra IPC requests. They use
the shared host wall clock and assume it does not step during the short sample.
The final catalogue also measured the following. Each idle window lasts 60
seconds; its limits are RSS <30,000,000 bytes and CPU <0.1%.

```json
[
  {
    "cpu_percent": 0.09992209136987198,
    "kind": "resources",
    "rss_peak_bytes": 9371648,
    "window_ms": 60046
  },
  {
    "cpu_percent": 0.09987754319554687,
    "kind": "resources",
    "rss_peak_bytes": 9273344,
    "window_ms": 60073
  },
  {
    "kind": "capture_to_recorded_upper_bound",
    "max_ms": 4.605042,
    "p50_ms": 0.714125,
    "p95_ms": 2.6211659999999997,
    "p99_ms": 3.7015,
    "samples": 1000
  }
]
```

The catalogue's latency includes additional observer IPC and has a 5 ms p99 gate.

## App and native platform checks

The app was rebuilt from its existing sources:

```sh
cd apps/macos && swift build
```

```text
Build complete! (1.96 secs.)
```

The app driver creates isolated configs/sockets, starts a virtual server at
127.0.0.1:24984, then runs the app's own offscreen E2E entry point:

```sh
OPENERGY_SOCKET=<private-app-socket> OPENERGY_CONFIG=<private-config.toml> \
OPENERGY_DAEMON="$PWD/target/release/openergy" \
OPENERGY_DAEMON_ARGS="--platform virtual" OPENERGY_ALLOW_PORT_24800=0 \
apps/macos/.build/debug/Openergy --e2e <private-output> \
  --join 127.0.0.1:24984 --host-port 24985
```

```text
PASS join ... state=connected
PASS role server ... state=listening
PASS layout keeps imported fields
PASS derived links arrive
PASS rename keeps layout entry
PASS live log lines
E2E PASSED (14 steps)
```

The safe native constructor check does not start capture or observers:

```sh
cargo run --release -p openergy-platform-macos --example mac_probe -- create
```

```text
backend=macos permissions=...; capture not started
invalid lines_per_notch rejected before construction
create_with_notices backend=macos; shared Backend type; notice queue empty; observers not started
```

Linux verification uses a read-only source mount, disposable `/work`, Xvfb :99,
and no host display, device, clipboard or published port. Run with the existing
linux-dev image, target and Cargo caches:

```sh
docker run --rm --name oe-linux-dev-wp6-final --network bridge \
  -e OPENERGY_DISABLE_DISCOVERY=1 -v "$PWD:/src:ro" \
  -v oe-linux-dev-wp8-target:/target -v oe-linux-dev-wp8-cargo:/cargo \
  --tmpfs /work:exec,size=512m oe-linux-dev-wp8:rust-1.99 bash -c \
  'cargo build --release --workspace && cargo clippy --release --workspace --all-targets -- -D warnings && ln -s /target /work/target && OPENERGY_X11_PROBE_ISOLATED=1 python3 crates/daemon/scripts/verify_linux.py'
```

```text
PASS Linux daemon: X11 backend, DSOP, native edge capture, key/click forwarding, clipboard both directions
```

## Safety, cleanup and limits

All host input E2E uses the virtual backend. Native macOS input posting, event
taps, cursor changes, pasteboard writes and permission prompts were not run.
No test used port 24800 or a LAN peer. Docker peers ran only in owned lab
containers; copied lab scripts isolated state, set LAB_NAME/LAB_PORT_RANGE,
and ran `lab.sh down`. Driver cleanup resumes/stops only children it created.

Windows named-pipe IPC remains unavailable in the daemon; its non-Unix entry
returns a typed error. Windows runtime/E2E is not claimed. Native Mac capture,
TCC revocation, sleep/lock/wake, and activation behavior remain manual checks on
an isolated or explicitly authorized workstation. No contract-change request is
needed for the completed macOS/Linux integration.
