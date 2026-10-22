# Public documentation verification — 2026-10-21

This is a documentation-scope check, not an integrated release acceptance run.
Engine baseline: `06b61287afd7f31aa3002d252389f2c2b17f73f5`. Documentation changes
do not alter the engine. Environment: Apple silicon macOS, Rust 1.99.1. All
runtime checks used the virtual backend, private temporary config/socket/TLS
paths, loopback addresses, and local ports from 25400–25419. The external
runner's lab range was 25420–25439; no Docker node was selected.

## Build and external scenarios

```sh
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked --bin openergy --bin openergy-e2e
target/release/openergy-e2e run --scenario 'S0-*' \
  --port-range 25400-25419 --lab-port-range 25420-25439 \
  --report /tmp/oe-public-docs-smoke.json
```

The release build exited 0. The following scenarios were each run separately
with the same ranges and distinct report files:

| Scenario | Result | Behavior checked |
| --- | --- | --- |
| S0-smoke | PASS | Real daemon control and lifecycle. |
| S6-trust-approval | PASS | Unknown identity held, approval persisted, reconnect. |
| S7-wrong-pin | PASS | Wrong pin rejected without plaintext fallback. |
| S14-import-roundtrip | PASS | Import/export through the real CLI. |
| S19-config-patch | PASS | Live config apply, invalid patch rollback, persistence. |
| S21-fingerprint-import | PASS | Synergy-format fingerprint and trust import. |
| R1-control-files | PASS | Control errors, files/modes, restart, subscriptions. |
| R2-lifecycle | PASS | Owned process lifecycle and clean daemon stop. |

The runner also rejected a selection matching no scenarios with exit 2 and
`no scenarios match; nothing ran`. No empty selection counted as a pass.

## Setup command path

A temporary command driver exercised the actual compiled CLI and two daemons.
It read the parent-owned example server/client configs and copied them into
separate private directories. It replaced the documentation/LAN addresses and
port with `127.0.0.1:25400`, used explicit `--config`, `--socket`, and `--dir`
arguments, and disabled discovery. It did not edit the original examples.

All of these checks passed:

- Empty offline trust list and missing-daemon status error.
- Local fingerprint creation and offline approval stored in both role lists.
- An unapproved server remained pending before connected state.
- The **exact shell snippet** for `ctl trust.approve` in [setup.md](setup.md),
  including its JSON quoting, succeeded. It stored a server pin in the client's
  server list only. Both peers then reported connected, TLS enabled, and trusted.
- An invalid clipboard-limit config patch failed without changing config bytes.
- The documented disconnect and connect commands recovered the connection.
- Import/export with explicit input/output paths and synthetic repo fixtures.
- Both owned virtual daemons stopped with exit 0. Unix identity files had mode
  `0600`. The temporary state and keys were removed after the run.

## Document checks and integration dependencies

Markdown fence balance and every `sh` example's shell syntax were checked.
The issue forms, template config, and Dependabot YAML parsed successfully;
issue-form IDs and field types were checked. `git diff --check` passed.

Local Markdown links were checked for existing targets. Five integration-owned
targets were verified in their owners' worktrees and must be present after
integration: `docs/releasing.md`, `examples/server.toml`, `examples/client.toml`,
`THIRD_PARTY_NOTICES.md`, and `tools/windows/README.md`. These files are owned
and committed separately; this documentation change does not supply them.

## Limits and follow-up

No native app was launched. No desktop input, clipboard, permission request,
login task, installed app, live service, release publication, or visibility
change was involved. Linux/Windows command examples were source-reviewed;
they were not executed on those systems in this documentation check. The
parent must rerun links and integrated validation after merging all owners.

Source review found a Linux default-path inconsistency: the daemon fallback
ignores custom `XDG_CONFIG_HOME`, while offline commands honor it. The setup
guide uses explicit paths. The daemon help and `crates/config/V1-LIMITS.md`
also mention CLI forms (`--apply`, `connect HOST`) not supported by the current
offline import/control dispatch. Current setup instructions use the working
commands. These source/help corrections belong to their owners.

A verified private vulnerability reporting channel remains a public-launch
requirement. Native Windows desktop acceptance and an automated legacy/native
focus comparison on an affected macOS build remain separate release gates.
