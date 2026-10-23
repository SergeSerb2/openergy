# Integrated checkpoint virtual baseline — 2026-10-23

The exact integrated checkpoint `ca618418dc8c4d93b8fd04eeda3fefa01003b8e7`
passes **89/89** existing real-process scenarios. The runner exits zero.
All scenario cleanup-failure arrays are empty. **Thirty Docker scenarios are
explicitly excluded**, with names/reasons in the [full record](virtual-baseline.json).
The Docker socket was absent; no Docker service was started or changed.

This checkpoint includes bounded discovery and its cancelled-startup fix, plus
the E2E port-capacity correction. It predates the separate IPC candidate. The
stopped Synergy 3 task's partial changes were not integrated or executed here.

## Exact build and run

A locked, offline release build of `openergy-daemon` and `openergy-e2e` passed
in 18.034 s with Rust 1.99.1, deployment target 15.0, nice 15 and two jobs.
Fresh copies of both binaries were frozen read-only in a new private directory.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| openergy | 3,725,472 | `eae3201b0e793c3533afe385a1f6c3d5eaa9992eff3a4cbd4d1a433134768367` |
| openergy-e2e | 1,049,904 | `fa2a0b7069f54d73dcad590154311247b0b1364b63c924a710b87ef084f6a8bb` |

The existing runner used all non-Docker scenarios, numeric loopback ports
24980–24999, separate lab range 24930–24939, virtual backends, and private
config/control/TLS/state paths. Discovery stayed disabled in these daemon
fixtures. Dedicated discovery probes keep their own evidence scope.

The full suite finished in 210.489 s. It exercised the real CLI, Unix control
sockets, raw TCP peers, plaintext/TLS paths, simulated input/clipboard protocol,
configuration and failure/recovery cases described by its scenario requirements.
No unit tests, mocks or computer-use checks ran. These are virtual-platform
protocol checks; they do not capture or inject live desktop input or clipboard.

All 250 recorded Rust/Cargo/toolchain/scenario inputs and both frozen artifacts
kept their hashes. The record includes every scenario result, exclusion, command,
cleanup array and stderr tail. Its copied report changes only private path
prefixes; the raw report hash and originals are retained. The `--keep` option
preserves owned fixture evidence. Postflight found no fixture executable or
TCP listener on the assigned local range. No unrelated process was stopped.

## Earlier failure and remaining scope

The previous 88/89 run remains an exit-1 result at its own source. Its P07
setup required 13 ports but was given ten. The corrected runner's default range
and explicit range here contain twenty ports; P07 now passes within the full
89-scenario run. No threshold or scenario assertion was weakened.

This is a new baseline, not acceptance of the unintegrated IPC candidate or its
blocked import-budget fix. Native Windows, Docker/X11 interoperation, native
input/clipboard, positive Windows migration, original independent security/
provenance and trusted signed/notarized distribution remain separate gates.
No speedup, whole-cache bound, long-duration RSS stability or broad security
clearance is inferred from this passing suite.
