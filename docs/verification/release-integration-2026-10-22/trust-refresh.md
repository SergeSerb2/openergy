# Trust refresh under held peers — 2026-10-22

## Reproduction

Baseline source: `2592e3906f9845ab45269dc70f22c7fa1871567b`.
An unapproved TLS peer triggers trust polling every 500 ms. Each pass reloaded
the driver's store, then opened and parsed a store for each eligible connection.
At the 16-connection cap, this repeated the same legal large-file work for one
trusted peer and 15 held peers.

The real-process regression uses 16,000 extra compact pins per role, within the
existing 1 MiB file policy. It sends 120 input probes and 120 independent status
queries at 40 ms intervals. Server CPU comes from the kernel process-time counter
over the complete probe window. These are workload CPU measurements, not idle
CPU. The declared local budgets are below 5% of one core and below 100 ms p99 for
both probe types.

The baseline passed both controls: large files without held peers, and 15 held
peers with small files. Only the combined large-store cohort failed, due to CPU.
Its latency remained within the 100 ms budget. Approval and failure semantics
were correct before the change.

## Change

Code commit: `e1798698320ebf1fb3b45e4005e1abc5c767666a`.
Each refresh now reuses a fresh store snapshot for all connections that read the
same TLS directory. The driver's existing store is still reloaded on every
pass. Other directories are processed one at a time and dropped before reading
the next, so a pass does not retain a full store for each connection directory.
Only small per-connection decisions remain until application.

Read errors are explicit decisions for peers requiring stored pins; they never
use the driver's last good store. Explicit pin and disabled-client-trust guards
run before stored-pin lookup. Initial TLS handshake checks and connection-info
reads keep their existing behavior. No file-mtime cache, worker or dependency
was added, and no trust decision is reused across refresh passes.

## Paired real-interface results

The committed [cohort command](../../../crates/daemon/scripts/verify_trust_refresh.py)
starts actual daemons, real mutual TLS connections and public IPC:

```sh
python3 crates/daemon/scripts/verify_trust_refresh.py \
  --openergy target/release/openergy --port-range 31200-31279 \
  --report /tmp/trust-refresh.json
```

| Cohort | Before CPU, % core | After CPU, % core | Before input p99, ms | After input p99, ms |
| --- | ---: | ---: | ---: | ---: |
| Large store; no held peers | 0.948 | 0.948 | 0.733 | 0.688 |
| Small store; 15 held peers | 1.143 | 1.136 | 0.737 | 1.762 |
| Large store; 15 held peers | 15.350 | 2.653 | 66.398 | 6.438 |

The large cohort's status p99 decreased from 66.070 to 3.243 ms. Its measurement
windows were about 5.86 seconds before and 5.28 seconds after. The small-store
input p99 did not improve in this finite run. No general real-time guarantee or
physical input-to-screen latency is claimed.

The baseline passed **3/4 cases**; the fixed binary passed **4/4**. The same script
verified all of these conditions on both binaries:

- Pending peers, exact trust-file bytes, configuration and a held key stay intact
  during the measured workload.
- A correct-role file update admits the chosen held peer on a later refresh. A
  wrong-role pin leaves another peer held.
- A real invalid-UTF-8 trust file closes peers requiring stored trust and releases
  held input. Restoring the file allows both authorized peers to reconnect.
- With actual refresh errors, an explicitly pinned client and a server with
  disabled client-pin requirements retain their connections and input. A later
  valid-store mutation succeeds.
- Every owned daemon and socket is cleaned up.

The final binary also passed the **9/9 trust-scaling**, **6/6 trust-lock/concurrency**,
**20/20 identity-file** and **15/15 daemon/CLI** regressions. The locked release
build, strict workspace/all-targets release Clippy and formatting passed with
Rust 1.99.1 and minimum macOS 15. The daemon grew from 3,690,816 to 3,707,520 bytes:
16,704 bytes, about 0.45%.

Exact final-suite results, binary/script hashes, initial exploratory measurements
and 112 tracked Rust/Cargo/toolchain file hashes are in the
[machine record](trust-refresh.json).

The final full virtual suite passed **89/89** with zero cleanup failures. **30
Docker scenarios were explicitly excluded**, not counted as passes; the Docker
socket was unavailable. The separate S15 idle check measured server/client CPU
at 0.033%/0.017% over 60-second windows, peak RSS of 9,469,952/9,388,032 bytes,
and p99 0.977 ms for 1,000 plaintext virtual-loopback moves. These idle and
loopback observations are separate from the TLS cohort workload above.

## Scope and remaining checks

Runtime measurements used macOS 27.2 ARM64, private temporary state, the virtual
backend, assigned loopback ports and production-port denial. Baseline and fixed
cohort commands ran sequentially without a concurrent build. Builds used two
Cargo jobs at reduced priority. No unit tests or mocks were added.

This runtime matrix uses one TLS directory per daemon. Its original review
covered retained connections from other directories only in source. The later
[retained-directory integration record](trust-directory-reconfiguration.md)
now exercises direct `NetDriver::reconfigure` with real TLS peers, directory-
specific revocation/read failures, recovery and continued frame delivery.
That is a separate run; it does not change this matrix's scope. The harness's
Linux process-time counter was not executed here; no new Linux or Windows
runtime acceptance is claimed. Filesystem operations remain synchronous.

Installed software, live sharing, native input/clipboard, permissions, login
items, services, tasks and port 24800 were preserved. Native, security-review
and distribution gates remain open. GitHub private reporting remains deferred.
