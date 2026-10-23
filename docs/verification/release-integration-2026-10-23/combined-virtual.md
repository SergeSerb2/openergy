# Combined release virtual verification — 2026-10-23

**PASS: 89/89 existing non-Docker scenarios, zero cleanup failures.**
Thirty Docker scenarios remain explicitly excluded. This run uses a fresh
combined release binary from `f7b10e7550aa11200385b06b69f6dcb2328dd9b5`.
Runtime source matches reviewed candidate `e6a9125` exactly. R8 source review
is separate from this real CLI/IPC/protocol run.

The pinned Rust 1.99.1 locked/offline build passed with deployment target 15.0,
nice 15 and two jobs. The suite then ran against frozen executable copies with
private configs, TLS directories, sockets and assigned loopback ranges
`24980-24999` and `24930-24939`. The existing runner applies virtual/private
configuration before daemon startup and disables discovery. It exercises real
CLI, Unix IPC, TCP, TLS, session and virtual clipboard/input protocol paths.
It does not exercise native input or the discovery startup lifecycle.

The build took 27.212 seconds. The suite took 210.351 seconds. These wall times
are observations of this run, not a performance comparison. All 248 recorded
source/scenario/vendor inputs and both frozen artifacts stayed unchanged.
Postflight found no assigned listener or owned process. Kept private fixtures
and logs remain available through the state pointer in the structured report.

| Frozen artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `openergy` | 3,725,472 | `0fc946a8fdb45d666e86739242a719212cafe0fcabc7bc48557e6af4d36608cd` |
| `openergy-e2e` | 1,049,904 | `fa2a0b7069f54d73dcad590154311247b0b1364b63c924a710b87ef084f6a8bb` |

The runner hash matches the earlier checkpoint runner. The new daemon has a
different hash and the same byte size. Equal file size does not establish equal
CPU, memory, latency or long-term behavior. Earlier idle measurements remain
pinned to the earlier checkpoint binary.

[Structured evidence](combined-virtual.json) records exact command arguments
with path aliases, build settings, hashes, all 89 scenario verdicts and the 30
explicit exclusions. Raw stdout/stderr, the complete report, exact input manifest
and execution helper remain in the private artifact directory with recorded hashes.
No unit tests, mocks, computer use or installed/live sharing state changes ran.

Final discovery evidence audit and revised native Windows startup acceptance
remain pending. The original Windows startup 11/14 failure remains unchanged.
Native input/clipboard, positive Windows migration, stopped IPC/Synergy3 work,
whole-cache/expiry/query/byte bounds, slow DNS, original security/provenance and
trusted distribution remain separate open gates. No push or publication occurred.
