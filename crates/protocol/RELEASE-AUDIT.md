# Protocol release audit — 2026-10-21

Scope: `crates/protocol`, with new `P` scenarios in the external E2E runner.
Parent commit: `06b6128`. Platform: macOS arm64, Rust 1.99.1.
The installed application, live settings, native input and native clipboard
were not used. All daemons used the runner's private config/socket/TLS paths
and virtual backend. Local ports were 25000–25019; lab ports were 25020–25039.
The runner and dependency manifests/lockfile were not changed.

## Fixed issues

1. **Impossible frame lengths kept connections open.** After the hello, a
   positive payload length below four cannot contain a message code. The
   decoder waited for that body before rejecting it. Initial hello lengths
   below 11, or between 12 and 14, likewise cannot form either hello shape.
   Reject these lengths from the prefix, before waiting for or copying body
   bytes. Empty frames already failed without a body; the new check keeps
   that behavior. P01 and P02 require closure within 500 ms without sending
   a body or EOF. Both fail on the parent and pass with the fix.
2. **Holding a key from Barrier disconnected the client.** The decoder
   required DKRP's language string for every dialect. Real Barrier 2.4
   advertises wire 1.6 and sends repeats without that string. The decoder
   now remembers the first hello version and accepts an absent language
   field for wire 1.1–1.7. It also accepts the complete modern suffix at those
   versions, as used by some current Synergy proxy writers. Wire 1.8 and
   later still require the suffix; a partial suffix always fails. No public
   message types or encoder signatures changed. Real Barrier held-key P03
   and raw fragmented/coalesced P04 fail on the parent and pass with the fix.

The legacy wire distinction is also described in the upstream
[Deskflow protocol reference](https://github.com/deskflow/deskflow/blob/master/docs/dev/protocol_reference.md).
The real pinned lab peers, rather than that moving source, supplied runtime
evidence: Barrier 2.4 / wire 1.6 and Deskflow 1.27 / wire 1.8.

## Audit coverage

- Checked packet/string/vector bounds, checked slicing, option-count parity,
  count-to-byte checks, signed scalar encoding, output reservation and atomic
  encoder validation. No additional concrete arithmetic bug was found.
- P07 drives 12 independent hostile connections: excessive/odd/truncated
  option counts, trailing option data, long/invalid UTF-8 language data,
  overflowing decimal clipboard totals, nonempty end chunks, excessive
  chunk lengths, trailing fixed-message bytes, and NUL/invalid UTF-8 names.
  Each connection closes; the daemon remains available.
- P04 checks mixed-case hello echo, fragmented hello bytes, coalesced frames,
  both legacy repeat shapes and the following release/keepalive. P05/P06
  check strict modern/partial-language rejection and held-input release.
- P03/P08 hold a key through real third-party Xvfb servers and require
  down, repeat, up and a connected client afterward.
- P09 sends real DCLP transfers through sockets and IPC clipboard readback.
  It checks Unicode and embedded NUL text, HTML, binary bytes, unknown format
  skipping, malformed container counts/lengths/UTF-8/trailing bytes, retention
  of the previous clipboard after invalid updates, and a valid empty update.
- Existing S8, S9 and S10 cover text in both directions, CRLF conversion,
  an 8 MiB DIB hash with input interleaving, and whole-transfer size refusal.
  Protocol's clipboard helper functions were reviewed statically; the
  daemon uses its own `session::clipboard` container implementation.

## Exact validation

From the worktree root, with Rust available on PATH:

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/oe-protocol-target nice -n 15 cargo build --release --locked -p openergy-daemon -p openergy-e2e
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/oe-protocol-target nice -n 15 cargo clippy --release --locked -p openergy-protocol --all-targets -- -D warnings
cargo fmt --check -p openergy-protocol
git diff --check

nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'P*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-final.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'S13-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-framing.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'S13-flood' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-final-flood.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'S8-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-text.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'S9-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-dib.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'S10-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-clipboard-cap.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'A2-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-negotiation.json
nice -n 15 /tmp/oe-protocol-target/release/openergy-e2e run --scenario 'A10-*' --port-range 25000-25019 --lab-port-range 25020-25039 --keep --report /tmp/oe-protocol-existing-unsupported.json
```

Build, Clippy, formatting and whitespace checks pass. P01–P09 pass **9/9**.
All **11 distinct existing focused scenarios** passed across the listed runs.
The first S13 batch was **5/6**: flood failed during raw send with
`raw send absolute deadline expired`, before its watchdog assertion. A
separate run passed on both the parent binary and the fixed binary. This
does not establish repeatable flood timing under load.

The unchanged parent was built from a `git archive HEAD` copy in a private
temporary directory, using `CARGO_BUILD_JOBS=2`, `nice -n 15` and
`CARGO_TARGET_DIR=/tmp/oe-protocol-baseline-target`. The same external runner
selected it with `--openergy /tmp/oe-protocol-baseline-target/release/openergy`.
`/tmp/oe-protocol-parent-comparison.json` records the Barrier/P04/hello
failures. `/tmp/oe-protocol-parent-short-frame.json` records the final P02
three-byte-prefix failure. An earlier P02 version checked an empty frame
and passed on the parent; that was replaced with the actual regression.
`/tmp/oe-protocol-parent-flood.json` records its passing flood run. Reports
retain the private runtime artifact paths. No unit tests or mocks were added
or run. There is no TypeScript in this scope.

## Boundaries and cross-scope follow-up

- The encoder still emits the current Synergy DKRP format. This change
  establishes reception of legacy repeats; it does not claim that every
  older receiver accepts our outgoing repeat format. Encoder version
  selection would need coordination with the transport/session owners.
- The daemon's separate session clipboard parser lacks the protocol helper's
  duplicate-format check. This was observed in source, not established as a
  native-platform failure. The session owner should assess consistent
  rejection before clipboard backend dispatch.
- The full integrated suite belongs to the parent. Native Synergy/Windows
  runs, native clipboard rendering, protocol 1.0–1.5 compatibility and public
  release readiness were not established by this audit.
