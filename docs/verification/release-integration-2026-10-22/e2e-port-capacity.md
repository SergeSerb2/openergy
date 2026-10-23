# E2E port capacity — 2026-10-22

The runner now rejects an undersized port range before starting any scenario.
Its local default is **24980–24999**, which fits the current catalogue's largest
scenario. The lab default remains 24930–24939. Production port 24800 stays denied.

The preceding [DNS regression run](dns-cancellation-and-readiness.md) recorded
88/89 passed. P07 failed during setup with `no free port in configured range`.
The reason is now established in source: `P07-hostile-field-bounds` declares
13 local nodes, each reserves one port, and that run supplied only ten ports.
The catalogue has 119 scenarios; P07 is the only one above ten local nodes.
The maximum lab requirement is two ports.

Commit `91247ca` increases the default to twenty ports and checks each selected
scenario's local and lab capacity before executing the first scenario. Explicit
small ranges remain useful for smaller selections. `list` can still inspect a
scenario with a small range because it does not reserve or run nodes. The check
does not guarantee OS availability; unrelated listeners or a later bind race
can still produce a retained environment failure.

Real command-path checks at source
`d598e21b8ffb2fab4908ea802932f18b3ad300d9` passed:

- Ten-port P07 run: exit **2** with `needs 13 node ports`, no started scenario
  and no result report. It fails before launching a fixture daemon.
- P07 with the new default: **1/1 passed**, exit **0**.
- P07 with exactly thirteen ports, 25950–25962: **1/1 passed**, exit **0**.
- P07 `list` with ten ports: exit **0**.

Both positive runs used the frozen real virtual daemon from `07adb0a`, whose
hash is in the [machine record](e2e-port-capacity.json). No daemon product source
changed. Pinned Rust 1.99.1 release build, strict all-target Clippy for the runner
and workspace formatting passed with two low-priority jobs. All 235 tracked
source/scenario inputs stayed unchanged during verification. No fixture processes
or assigned listeners remained. No unit tests or mocks were added or run.

The original full run remains 88/89, exit 1; these focused passes do not rewrite
it. Native input/clipboard, the Windows refusal-control mismatch, original
security/provenance acceptance and trusted distribution remain open.
