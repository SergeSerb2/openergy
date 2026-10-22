# Final resource, boundary, and runtime hardening

This follow-up covers the fifth audit pass and its independent review/fix
cycles through round seven. It supplements the earlier [integration record](README.md).
The [machine record](round5.json) retains source-specific outcomes and hashes.
These checks do not authorize a public launch or change installed software.

## Fixed behavior

- Config and imported trust inputs must be regular files. Unix nonblocking
  opens close the metadata-to-FIFO replacement race. Bounded reads reject
  growth beyond 16 MiB for daemon TOML and 1 MiB for each trust list.
- Runtime saves and offline Synergy imports share the startup TOML limit.
  They serialize once and refuse oversized output before touching the
  destination. Failed live changes preserve disk, running config, peers,
  virtual input delivery, and restart behavior. JSON-only inspection and
  Synergy-format export keep their separate contracts.
- Trust imports take one directory lock and preflight both serialized lists
  before writing either. A maximum-size compact list can still revoke a pin:
  revocation may retain the already-supported bare SHA-256 representation.
  This does not promise atomic commits across two files or a lock deadline.
- Unrelated client settings preserve clipboard transfers. Live aliases adopt
  their new canonical names and merges retain one owner. Double-tap edge
  timing now requires physical retreat and return. Large dead-corner values
  use wide arithmetic with both positive and negative display origins.
- Transport budgets charge packet allocation capacity and metadata, including
  queue-growth headroom. The Bulk wire window and reserved Control budget
  remain in force. X11 image conversion checks decoded and encoded sizes
  before allocating expanded pixels or replacing the clipboard selection.
- Draft validation reads embedded archive provenance without extraction and
  requires one regular root `BUILD.json` no larger than 1 MiB, equal to its
  sidecar. Case aliases and nested or duplicate manifests are refused.
- Forced-teardown verification now observes the actual running Docker lab,
  validates ownership labels, and confirms removal by its exact container ID.
  It fails closed when Docker cannot establish cleanup.

## Integration and review evidence

The full release verification passed **117/117** external scenarios, including
all **30** Barrier/Deskflow Docker cases, with no exclusions or cleanup failures.
Its functional source was `a18f2ca`; notice/prose refresh `fe44328` changed no
runtime code. This run preceded the final config-save, compact-revoke, and
offline-import corrections. Its daemon hash was not captured before the next
build replaced the output, so the machine record does not invent one.

The final package uses clean source `655e59f`, including all runtime fixes.
Pinned Rust 1.99.1 workspace/all-targets release Clippy with warnings denied,
formatting, notice freshness, the locked release build, and Swift release build
passed. Staged and freshly extracted daemon CLI/virtual IPC checks and extracted
code-signature verification passed. The newly extracted app passed **25/25**
headless app-to-daemon steps and **7/7** startup safety refusals, including owned
process cleanup. These checks do not render or drive the native desktop.

The fixed copy of that extracted daemon passed **88/88** standard virtual
scenarios, with all 30 Docker cases explicitly excluded and no cleanup failures.
Its two 60-second idle samples measured 0.0666% and 0.0500% process CPU, with
sampled RSS peaks of 9,420,800 and 7,946,240 bytes. The 1,000-move external
IPC-to-receiver observation had p99 **1.234 ms**. Other work ran concurrently;
this is a local observation, not native input or LAN latency acceptance.

The final offline fix also passed a separate **27-scenario** independent review:
**67 CLI calls**, six private virtual daemon starts, and all 112 recorded
Rust/Cargo inputs matching the reused verified binary. Outputs of 16,777,215 and
16,777,216 bytes imported and started; 16,777,217 bytes were refused. Existing
files stayed identical; missing files, parents and temporary outputs were not
created. The review found no concrete P1/P2 defect or material regression.
The [config policy record](../2026-10-22-config-size-policy.md) retains the
implementation checks and binary/source hashes.

Earlier independent follow-ups reproduced and closed the archive case-alias
bypass, runtime save overflow, and compact trust revoke failure. The round-six
review then found the offline import bypass; the round-seven review above
checked its fix and the relevant writer/caller paths. Scoped transport/X11
review found no remaining P1/P2 issue and repeated real TCP, Xvfb and archive
CLI paths. These are bounded reviews, not a claim that the program has no bugs.

The machine record distinguishes the final virtual run from the full run,
platform checks, and focused boundary replays. One previous 88-case attempt
passed 87 cases: the new one-second allocation check inherited a 0.1% idle CPU
threshold, where one clock tick measured 0.939%. Its RAM check passed. That
short check now has an explicit CPU allowance; the separate two 60-second idle
CPU checks keep the 0.1% gate. The corrected case passed on replay. See the
[short-window record](short-resource-window.md). An earlier run was deliberately
cancelled when another build could replace its binary; it is not counted.

The [Windows record](../../../tools/windows/ROUND5-1ef54fc.md) covers an
immutable `1ef54fc` snapshot: pinned MSVC release/strict Clippy, actual private
pipe and kernel ACL checks, virtual TLS/clipboard, runtime config rollback, and
compact revoke/import bounds. A separate copy with exactly the three
`ccf9847` source files passed pinned build/Clippy and normal/oversized offline
imports. Both 514-file inventories and Cargo.lock stayed unchanged; no owned
process or assigned-port listener remained. Its explicit composite source is
recorded without treating it as a full later-parent archive.

The [Linux record](../../../crates/platform-linux/FINAL-R5-VERIFICATION-2026-10-22.md)
covers the same immutable base and separate three-file overlay on ARM64
Docker/Xvfb. The base passed pinned workspace release/strict Clippy, four CLI
bounds checks, ten focused external scenarios, and real clipboard/INCR/pixel
checks. The overlay passed pinned build/Clippy and nine real import CLI checks.
Both inventories and lockfile stayed unchanged; owned containers were removed.
The supported x86_64 Linux distribution artifact was not built by this ARM64
verification.

## Measured resource changes

These pairs use the same workload before and after each fix. They are
case-specific observations, not general product performance guarantees.

| Workload | Before | Fixed | Meaning |
| --- | ---: | ---: | --- |
| Six-second tiny-reply TCP flood, maximum sampled daemon RSS | 134,627,328 B | 20,611,072 B | Packet metadata is now charged to queue budgets. Sampling is not kernel peak RSS. |
| Oversized indexed DIB conversion, kernel child peak RSS | 276,992,000 B | 18,833,408 B | The fixed backend refuses the oversized image and preserves the prior selection. |
| Stationary double-tap edge, process CPU over 20 seconds | 0.3491% | 0.0499% | The fixed timer no longer renews the hit while stationary; one sample per version. |

Transport/image raw sanitized records, exact sources, and reproduction commands
are in the [resource audit](../../../crates/net/examples/r5-evidence/README.md).
The stationary-edge baseline/fixed daemon hashes and sample windows are in
`round5.json`. The normal 60-second idle and 1,000-move IPC-to-receiver
observations remain in each source-specific suite record. They do not measure
physical input-to-screen latency, nor show a latency speedup over Synergy.

The final local ARM64 archive is **3,612,577 compressed bytes**, with
**6,324,424 regular app-file bytes** across nine files. This is about 98.85% fewer
regular app-file bytes than the previously inspected 548,821,397-byte Synergy
3.7.0-beta bundle. File bytes omit filesystem allocation overhead. The systems
were not run under matched native workloads, so no comparative CPU, RAM or
latency speedup is claimed.

Archive SHA-256:
`1e7f2f8912ae6cf33b59c12f2f5ccc89f58326c0989c6e1b0913f7e418fc85d1`.
It is ad-hoc signed, unnotarized and unpublished. Its embedded manifest records
clean source `655e59f69c490841dc2cd2c8128eca51545c15f7`. Later evidence-only commits
do not change that artifact's provenance.

## Remaining launch gates

Native macOS/Windows input, permission recovery, interactive clipboard, keyboard
layouts, scaling, fresh install and login behavior still need acceptance on
isolated machines. Signed/notarized distribution, Intel Mac and x86_64 Linux
artifacts, actual Windows service restoration, and a verified private security
reporting route remain open. Historical personal checkout paths remain in
reachable commits; no history rewrite was performed. No live connection,
installed app, user clipboard, service, login item, TCC setting, or port 24800
was used in this pass. No unit tests or mocks were added.

## Final source checks

Python AST validation covered 26 scripts, shell syntax covered nine, actionlint
passed, and local Markdown links and changed-file personal-path checks passed.
The notice inventory matches Cargo.lock. No external package versions changed
in this pass; the earlier cargo-deny advisory, ban, license and source-policy
result remains source-scoped, with no new advisory exception. The new direct
Unix libc edge reuses the already-resolved package. Final secret-scan counts
are recorded separately in `round5.json`.
