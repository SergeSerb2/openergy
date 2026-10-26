# Clipboard format-loss audit — 2026-10-22

This follow-up is limited to clipboard format-loss ambiguity in
`crates/session/src/clipboard.rs`.

## Change

Previously, a nonzero-count wire container containing only unknown format IDs
was decoded into an empty `ClipboardData`. Both established receiving roles
then treated it as a true clear and removed existing clipboard text.
Likewise, the public virtual clipboard accepts `FormatKind::Png`, whose
`wire_id()` is `None`. Marshalling PNG-only local data filtered every format
out and sent a count-zero container, clearing the peer's clipboard.

Two guards now reject these cases with the existing `ClipboardError::Malformed`:

- Unmarshal rejects nonzero-count containers that contain no supported formats.
- Marshal rejects nonempty local format lists if filtering removes every format.

Unknown formats remain skippable alongside supported formats. A supported
zero-byte text format remains valid. A genuinely empty format list/count-zero
container still clears the clipboard. Transfer, policy, and availability
contracts are unchanged.

## Real-process evidence

The private fixture is an exact `git archive` of parent
`b36e840ea40b0c455db41afa6617d376b3b31898`. The baseline was built before the
source change. Only the fixed `clipboard.rs` was overlaid for the fixed build;
new TOML scenarios were supplied through the external runner's scenario-dir
option. No other implementation source was changed. Hash comparisons verified
that the fixture's root manifest and lockfile stayed identical to the parent.

Four new cases failed before the fix and passed afterward:

- `Q20-client-unsupported-only-clipboard` and
  `Q20-server-unsupported-only-clipboard` send the actual established-wire
  container `00000001000000630000000378797a` through DCLP Start/Data/End. They
  assert preservation of existing receiver text, then verify supported plus
  unknown formats, supported zero-byte text plus unknown data, and a genuine
  count-zero clear.
- `Q21-client-unsupported-only-local-clipboard` and
  `Q21-server-unsupported-only-local-clipboard` use the public virtual IPC path
  with PNG-only data. They assert that peer text survives, then verify text plus
  PNG and a genuine local clear.

The fixed focused run passed **10/10** scenarios: these four new cases, Q18
true clears in both directions, Q19 unavailable-read privacy, Q14/Q15 duplicate
format rejection, Q13 read-error retry, and A5 unavailable clipboard readback.
Both baseline and fixed E2E runs used debug binaries over real IPC/TCP sockets.
Release compilation, strict scoped Clippy, formatting, and diff checks passed.

Every daemon used explicit private config, socket, and TLS paths through the
external E2E runner, with `--platform virtual`. Listen/destination ports were
limited to `26040–26059`; the reserved lab range was `26060–26079`. No Docker,
native input/clipboard, installed application, live configuration, or production
port was used. No unit tests or mocks were added.

The sanitized [machine evidence](FORMAT-LOSS-AUDIT.json) records artifact hashes,
all four baseline failures, and all ten fixed results. Raw reports and retained
failure artifacts remain local in the private audit directory.

## Commands

`OE_FORMAT_AUDIT` below denotes the retained private source directory. Build,
E2E, and Clippy commands ran there; `new-scenarios` contained the four new files,
and `focused-scenarios` contained the ten cases listed above. Formatting and git
diff checks ran in the owned worktree.

```sh
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --locked -p openergy-daemon -p openergy-e2e
nice -n 15 target/debug/openergy-e2e run --scenario-dir "$OE_FORMAT_AUDIT/new-scenarios" --port-range 26040-26059 --lab-port-range 26060-26079 --report "$OE_FORMAT_AUDIT/before.json"
# Overlay the fixed owned clipboard.rs, then rebuild with the first command.
nice -n 15 target/debug/openergy-e2e run --scenario-dir "$OE_FORMAT_AUDIT/focused-scenarios" --port-range 26040-26059 --lab-port-range 26060-26079 --report "$OE_FORMAT_AUDIT/after.json"
CARGO_BUILD_JOBS=2 nice -n 15 cargo clippy --locked --no-deps -p openergy-session -- -D warnings
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked -p openergy-daemon -p openergy-e2e
# From the owned worktree:
rustfmt --edition 2021 --check crates/session/src/clipboard.rs
git diff --check
```

This focused verification does not assert native platform behavior or replace
the parent's full integrated suite. Release binaries were compiled; the E2E
results above are from the debug binaries.
