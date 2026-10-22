# Release preparation integration — 2026-10-22

This record covers the public-source preparation branch. It does not approve a
production deployment, repository visibility change, or binary publication.
The repository remains private and no release was published during this work.

All daemon checks used private temporary config, TLS and control paths, the
virtual backend, and assigned loopback ports. Docker peers used their own Xvfb
display. No installed Openergy/Synergy process, live input connection, native
clipboard, TCC setting, login item, or production port was used. No unit tests
or mocks were added. Native backend probes ran only inside isolated Linux Xvfb.

## Integrated checks

The sanitized [machine record](checks.json) contains scenario names, outcomes,
durations, cleanup results and measurements. It omits raw input, clipboard data,
addresses, local paths and process logs.

| Check | Source and result |
| --- | --- |
| Full Mac release verification | `4f45290`: locked workspace build, formatting, strict Clippy, Swift release build, real CLI smoke and **106/106 external scenarios**, zero exclusions/cleanup failures. Includes all 30 Barrier/Deskflow Docker scenarios. |
| Network ordering follow-up | `b36e840`: rebuilt release daemon; **77/77 virtual scenarios**, including N2 queued key release/reentry. The 30 Docker cases were explicitly excluded in this separate run. |
| Final clipboard format-loss follow-up | `29fa8ba`: formatting, strict release workspace Clippy and locked release build passed; **23/23 Q scenarios** and **2/2 Barrier/Deskflow clipboard interoperability scenarios** passed. This includes all four new wire/local unsupported-only regressions. |
| Direct daemon/CLI failure paths | `b36e840`: **15/15**, including socket file/symlink preservation, endpoint replacement/shutdown, private logs, framing and response caps, argument errors, connect HOST, pinned-connect rollback and revocation. |
| Offline identity/trust commands | `b36e840`: private permissions, stable identity under 24 concurrent creators, concurrent trust updates, incomplete/corrupt identity refusal, non-destructive import and temporary-file collision handling passed. |
| Headless Mac app | Current app code through `05204a7`, repeated with daemon `29fa8ba`: **25/25 model-to-real-daemon steps**, **7/7 startup safety refusals**, owned process/socket cleanup and private-directory checks. Rendering, native UI and TCC were not exercised. |
| Integrated Linux | `b36e840`: ARM64 Rust 1.99.1 locked workspace build and strict Clippy, real CLI smoke, **9/9 focused daemon scenarios**, Xvfb clipboard and recovery probes. [Full Linux record](../../../crates/platform-linux/INTEGRATED-VERIFICATION-2026-10-22.md). |
| Dependency policy | cargo-deny 0.20.2: advisories, bans, licenses and sources passed. Remaining warnings concern duplicate transitive versions and unused allowed license names. The unmaintained PEM parser was removed; no advisory exception was added. |
| Privacy and source checks | Gitleaks 8.30.1 scanned 122 content-bearing history commits with zero leaks. Changed-file personal-path scan and local Markdown link check found none. Python AST checks covered 24 scripts; shell syntax covered nine scripts; actionlint passed with declared optional local-worker labels. No TypeScript files exist. |

The full run preceded the network ordering follow-up and later focused review
fixes. It is evidence for its recorded revision, not a claim that every later
commit ran the complete suite. Subsequent affected-flow results are recorded
in the subsystem reports and the final follow-up record below.

## Review and fix rounds

The first independent source review found pin revocation, clipboard policy,
restart recovery, geometry, failed readback, identity creation, rename/import,
size-limit and CLI issues. The second round found unsafe initial E2E config,
aggregate held-modifier and genuine clipboard-clear defects. Their fixes were
integrated before the full run.

The third round reproduced unsupported-only clipboard containers becoming
destructive clears, and Windows lifecycle endpoint ownership and migration
rollback problems. The session follow-up also covered unsupported-only local
PNG data. See the [format-loss regression record](../../../crates/session/FORMAT-LOSS-AUDIT.md).
The network ordering follow-up separately reproduced old safety key releases
being overtaken by new input and added N2 to the standard scenario catalogue.

## Evidence limits and launch gates

The measured virtual daemon RSS was below 10 MB, idle CPU below 0.1%, and the
1,000-event external IPC-to-receiver observation had p99 below 1.2 ms in these
two local runs. Other build/check workloads ran concurrently. These are scoped
lab observations, not physical input-to-screen latency or a comparative product
benchmark. The exact measurements remain in the machine record.

Historical privacy findings still apply: earlier reachable commits contain
personal checkout paths. Current changed evidence was scrubbed, but no history
rewrite was performed. A zero-leak secret scan does not erase that history.

Native macOS/Windows desktop input, permission/recovery, mixed scaling,
keyboard layout, login startup, fresh install, notarization/Gatekeeper and
interactive Windows clipboard acceptance remain open. Linux verification was
ARM64 Xvfb; x86_64 Linux archive/distribution acceptance remains unverified.
The Intel Mac distribution target was not built here. A private vulnerability
reporting channel must be enabled and verified, or a monitored private contact
published, before public launch. See [readiness](../../plan/readiness.md),
[security](../../../SECURITY.md), and [releasing](../../releasing.md).
