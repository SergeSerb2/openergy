# Release preparation integration — 2026-10-22

This record covers the public-source preparation branch. It does not approve a
production deployment, repository visibility change, or binary publication.
The repository remains private and no release was published during this work.

All daemon checks used private temporary config, TLS and control paths, the
virtual backend, and assigned loopback ports. Docker peers used their own Xvfb
display. No installed Openergy/Synergy process, live input connection, native
clipboard, TCC setting, login item, or production port was used. No unit tests
or mocks were added. Native backend probes ran only inside isolated Linux Xvfb.

## Final hardening follow-up

The [DNS cancellation and connection-readiness follow-up](dns-cancellation-and-readiness.md)
verifies obsolete-job recovery and fixes a reproduced false `Connected` event
after address fallback. Earlier failed reports remain intact.

The [source-history scan refresh](source-history-refresh.md) preserves the raw
64-finding Gitleaks result and verifies every reported value against historical
source blobs. It adds no scanner exception or history rewrite.

The [owned discovery shutdown follow-up](discovery-owned-shutdown.md) verifies
the integrated worker-leak fix through four live-child cases, including private
descriptor exhaustion. It keeps retained replay and DNS refusal failures open.

The [initial discovery recovery](discovery-recovery-initial.md) and
[Windows discovery/control evidence](windows-discovery-recovery.md) retain their
exact intermediate source scope. Retained-cache replay and worker shutdown
acceptance gaps remain explicit; these records do not approve final recovery.

The [dependency-policy refresh](dependency-policy-refresh.md) checks current
RustSec data, all-features and five platform graphs, registry yank/checksum
metadata and notice freshness without changing dependencies or runtime code.

The [Windows pipe deadline follow-up](windows-ipc-deadlines.md) verifies all
19 request/idle/refusal/shutdown checks against real Windows named pipes. The
retained pre-deadline control fails exactly the six timeout/recovery cases.

The [current Windows transport follow-up](windows-transport-current.md) records
a fresh pinned native MSVC build and real pipe, ACL, TCP/TLS and virtual
clipboard/bounds checks. It preserves the failed observation-window probe,
the stricter portable refusal check, exact source scope and native acceptance gates.

The [receive-fairness follow-up](receive-fairness.md) records delayed control
replies under a tiny-frame flood, the receive-event yield threshold, paired
plaintext/mutual-TLS checks and the real 16-peer connection-churn matrix.

The [retained trust-directory follow-up](trust-directory-reconfiguration.md)
checks direct transport reconfiguration with real mutual-TLS child peers.
It covers directory-specific revocation, admission, read failures and recovery,
failed preparation, and continued frame delivery through the original handle.

The [trust-refresh cohort follow-up](trust-refresh.md) records repeated reads at
the connection cap, shared fresh snapshots, and real approval, role, read-failure
and bypass controls. It distinguishes workload CPU from idle measurements.

The [trust-list scaling follow-up](trust-scaling.md) records slow legal large-list
operations, preserved first-seen order and file policies, lower-allocation
fingerprint formatting, and the measured speed, memory and executable-size tradeoff.

The [identity-file resource follow-up](identity-file-bounds.md) records bounded
PEM reads, exact-limit controls, generated-output preflight, and real CLI/startup/
live TLS refusal and recovery. It preserves the original memory measurements.

The [identity-file contention follow-up](identity-lock-contention.md) records a
second reproduced lock stall, prompt startup/live TLS refusal, preserved active
input and connections, and refusal of an incomplete identity without key replacement.

The [trust-file contention follow-up](trust-lock-contention.md) records a
reproduced session-thread stall, nonblocking live updates and preserved offline
serialization. It covers five live routes and concurrent CLI/identity writers.

The [live pin revocation follow-up](trust-revocation-cutoff.md) records input
accepted after a successful revoke reply, the immediate-disconnect fix,
the new A13 real-TLS regression, and stored-trust and persistence-failure controls.

The [incomplete IPC request follow-up](ipc-read-deadlines.md) records the resumed
hardening pass, a reproduced client-slot exhaustion failure, absolute request
read deadlines, and real CLI/socket regressions. It also records the maintainer's
decision to defer GitHub private reporting setup while keeping native acceptance
as a release gate. Earlier reporting prerequisites below are historical.

The [Linux x86-64 and Python 3.11 follow-up](linux-amd64-python311.md) records a
real packaging compatibility failure and fix, the extracted local Linux
artifact, and 30 focused scenarios plus 15 direct CLI checks under AMD64
emulation. It does not establish physical desktop acceptance.

The [idle IPC and Linux parent follow-up](idle-wake-and-parent.md) records the
later idle CPU improvement, Mac ARM64 and Intel/Rosetta regression checks,
Linux zombie-parent and live-worker fixes, three further source reviews, and
the clean `6b83420` local archive. Original failures and finite-run limits remain
in its [machine record](idle-wake-and-parent.json).

The [final hardening record](round5.md) covers bounded config/trust I/O, safe
save/import limits, queue and image allocation bounds, live alias and clipboard
recovery, and embedded archive provenance. It records the later 117-case full
run, exact-source package and platform checks, independent review responses,
and measured resource changes. The earlier checks below retain their original
source scope; they are not silently relabeled as final-build results.

The later [sustained workload and memory control](sustained-memory.md) records
180 transfers, 90 cancellations and nine reconnects, then isolates diagnostic
history polling as a source of resident allocator pages. Its finite-run limits
remain explicit.

The later [trusted release boundary](trusted-release-boundary.md) separates
default-branch validation and draft creation from candidate builds. Its real CLI
checks passed; hosted execution and remote draft creation were not attempted.
The [Windows migration follow-up](../../../tools/windows/MIGRATION-PRIVILEGE-BOUNDARY.md)
separates per-user commands from a protected privileged helper. Real rejection,
ACL and occupied-pipe checks passed. A successful limited-user handshake and
service/UI recovery remain unverified because the restricted PowerShell process
could not start in the SSH session. These source fixes do not close the native
acceptance gates below.

The [late review and archive record](late-review.md) records the Windows
privilege split, five later correctness findings and their code responses, and
the clean `e10e4a4` macOS archive. It also records the unavailable independent
release-review verdict. Passing refusal checks does not close positive Windows
migration or native acceptance.

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
| Integrated Windows | Native x86_64 MSVC Rust 1.99.1 release build, strict all-targets Clippy across eight packages, real pipe/CLI control and identity/trust kernel ACL checks passed. Virtual TLS/input/clipboard/clear and lifecycle ownership checks passed. The original lint failure and its exact two-file follow-up are preserved in the [Windows records](../../../tools/windows/PINNED-CLIPPY-FOLLOWUP.md). |
| macOS archive | Clean source `7a1e077`: local ARM64 archive, **3,609,660 compressed bytes**, **6,324,408 app-content bytes**. Staged and freshly extracted daemon CLI/virtual IPC smoke and extracted code-signature verification passed. Ad-hoc signed, unnotarized, unpublished. SHA-256 is recorded in checks.json. |
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

The Windows lifecycle fix authenticates the supervised child's executable,
PID and start time, then verifies the kernel server PID on the connected pipe
before sending control requests. Migration validates the complete installation
before service changes and attempts restoration even if Openergy stop fails.
Actual service restoration remains untested; the owned virtual failure cases
passed on both PowerShell 7 and Windows PowerShell 5.1.

A fourth independent source review checked these late fixes and found no
remaining concrete P1/P2 defects in scope. The Mac supervisor now preserves
the daemon's native-only default-port policy; its rebuilt headless app repeated
all 25 steps and seven safety refusals successfully. The pinned Windows lint
warning was fixed without suppression and rechecked with the real compiler.

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
