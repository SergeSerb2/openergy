# Receive fairness and connection churn — 2026-10-22

## Reproduction and change

The prior daemon, SHA-256
`4ba143a7f99f03e2a0c5a844b6c40589ac10dc0c8543d6ca3f3f0341effc3091`,
kept queue memory bounded during the existing non-reading-peer stress command.
However, one read turn could decode 32,768 eight-byte keepalive frames before
returning to the daemon owner. Processing their replies delayed public IPC.
The initial memory-only run sampled a 301 ms control reply; an explicit
100 ms regression gate reproduced the delay in later runs.

Runtime fix: `ebf30ee01999ee4dc7849b52bc057b60a6bf0625`.
Each connection now yields its receive turn after accumulating at least 1,024
events, finishing the current read of up to 16 KiB. This is a yield threshold,
not an exact event-count cap. The existing 256 KiB wire/plaintext byte budget
still bounds work when messages are large. Buffered decoder/TLS data and the
read-ready state stay intact for the next turn.

The change does not alter frame order, trust, queue accounting, write selection,
or flood/slow-peer policy. No worker, dependency, persistent cache or additional
connection storage was added. The release daemon stays **3,707,520 bytes**;
its new SHA-256 is
`683de1fc9c64c062dc98482975f3f8dc05542426af36716a3a4fb299c406b5a4`.

## Paired real-process flood checks

The [existing queue command](../../../crates/net/examples/r5_queue_memory.py)
now has an optional `--max-ipc-ms` gate and explicit TLS-version modes. A real
virtual client receives keepalives from a real non-reading loopback peer. Every
case must still reach more than 100,000 outgoing events and a visible queue-full
condition. The original 64 MiB sampled RSS-growth gate remains in force.

The final commands used the same script and Python 3.14, sequentially without
a concurrent build. Both TLS modes require and verify the client's certificate
against its generated private fixture certificate; the daemon pins the peer.
The negotiated protocol is recorded before shutdown.

| Transport | Before maximum sampled IPC reply, ms | After, ms | Before 100 ms gate | After gate |
| --- | ---: | ---: | --- | --- |
| Plaintext | 225.518 | 13.924 | FAIL | PASS |
| Mutual TLS 1.2 | 109.568 | 18.087 | FAIL | PASS |
| Mutual TLS 1.3 | 62.442 | 15.916 | PASS | PASS |

All six runs reached the stress condition and passed the original memory gate.
After the fix, sampled RSS growth was 7,421,952 / 7,684,096 / 7,553,024 bytes
for plaintext / TLS 1.2 / TLS 1.3. These are sampled finite-workload observations,
not kernel high-water measurements, whole-daemon memory limits, CPU results,
physical input latency, or a comparison with Synergy.

Earlier plaintext probes sampled maximum replies of 301.345, 645.052 and
233.579 ms. They remain exploratory results in the machine record; the table
uses the final common-script matrix. The first TLS 1.3 summary queried the
version after socket shutdown and lost that metadata; it is excluded from the
matrix and was rerun after fixing the reporter. The system Python's LibreSSL
2.8.3 cannot run TLS 1.3, so the final matrix uses the installed Python 3.14 SSL
build. No Python dependency or lockfile was changed.

## Full connection cap under churn

The new [real daemon/CLI command](../../../crates/daemon/scripts/verify_connection_churn.py)
adopts all 16 peers through complete protocol handshakes. One real virtual
client exchanges input while 15 raw peers stay connected. A 17th connection
must close before any protocol byte. The four cases use 0, 4, 16 and 32
connector workers, with 60 independent status probes and 60 input-delivery
probes per case.

All four cases passed the declared 100 ms p99 budget. The three pressure cases
observed **76,984** actual overflow closures, with no unexpected admission or
socket error. Every case preserved all 16 peers, exact config bytes, active
screen and held key. The final key release reached the client. All owned
processes and IPC endpoints were cleaned up.

This cohort uses plaintext. It does not claim a universal bound under every
possible connection or message flood. The listener accept loop was inspected
and exercised but did not reproduce the suspected starvation at these loads;
no listener change was made. The receive-fairness fix addresses the separate
reproduced tiny-message workload above.

## Affected-flow verification

- The real TLS priority pair transferred **10,485,760 bytes**, delivered all
  **321** control/input probes, and observed at most one bulk frame between
  input frames. Both processes exited zero.
- The guardrails CLI passed input-budget refusal with the peer preserved,
  reserved-Control slow-peer closure, a 120 ms handshake deadline, original
  cross-thread waker reuse, and the unchanged five-second flood watchdog.
  The latter closed after **2,906,112** actual framed messages.
- Both existing lifecycle cycles passed, including delivery after stopping
  the listener and cleanup across TLS disablement.
- The retained trust-directory CLI passed **4/4** real mutual-TLS scenarios
  with the new transport: revocation/admission, both directory read failures,
  reverse recovery and rejected preparation.
- Locked release daemon/runner/example builds, formatting, strict release
  workspace/all-targets Clippy, Python AST and four real CLI refusal checks
  passed.

The first legacy guardrail run failed with `no SlowPeer close`. Its old probe
sent only Input frames, which the existing high-water check already refuses
before crossing the watchdog threshold. The corrected probe, committed as
`974a17e`, asserts those refusals and peer preservation first, then deliberately
uses the reserved Control allowance to reach the unchanged watchdog. The
original failed output is preserved; no runtime threshold was weakened.

The final full virtual suite passed **89/89**, with zero cleanup failures.
**30 Docker cases were excluded**, not counted as passes: the Docker socket
was absent and the engine was not started. The S15 idle check measured
server/client CPU at 0.050%/0.017% over separate 60-second windows, with peak
RSS of 9,437,184 bytes for each. Its 1,000-move plaintext virtual-loopback
sample had p99 **0.905 ms**. These idle and loopback measurements are separate
from the flood workload above.

A clean fresh local clone matched all **113** tracked Rust/Cargo/toolchain
hashes and both affected script hashes. Relative to the prior source record,
the two changed Rust inputs are the reactor and the guardrails example;
only the reactor is part of the daemon runtime.

## Reproduce and limits

```sh
CARGO_BUILD_JOBS=2 MACOSX_DEPLOYMENT_TARGET=15.0 nice -n15 \
  cargo build --locked --release -p openergy-daemon -p openergy-e2e
nice -n15 python3 crates/net/examples/r5_queue_memory.py \
  --openergy target/release/openergy --port 26200 \
  --source SOURCE_COMMIT --max-ipc-ms 100 --report /tmp/receive-plain.json
nice -n15 python3 crates/net/examples/r5_queue_memory.py \
  --openergy target/release/openergy --port 26200 \
  --source SOURCE_COMMIT --max-ipc-ms 100 --tls-version 1.3 \
  --report /tmp/receive-tls13.json
nice -n15 python3 crates/daemon/scripts/verify_connection_churn.py \
  --openergy target/release/openergy --port-range 31510-31517 \
  --report /tmp/connection-churn.json
```

Use a Python SSL build supporting the selected TLS version. All final checks
used macOS 27.2 ARM64, Rust 1.99.1, minimum macOS 15.0 and private temporary
state. Builds used two Cargo jobs at reduced priority. The stress peer used
port 26200, churn used 31510–31517, and the example peers used 24973–24977 and
25090–25093. All traffic stayed on loopback. The full virtual runner used
31600–31699 and reserved 31700–31799 for its excluded Docker lab.

The [machine record](receive-fairness.json) preserves exact source/artifact
hashes, reports, output and original failed observations. No unit tests or
mocks were added. Installed software, live sharing, native input/clipboard,
permissions, services, tasks and port 24800 were preserved. No new Linux,
Windows, native or signed/notarized distribution acceptance is claimed.
Those release gates and the two original security review threads remain open.
GitHub private reporting setup remains deferred.
