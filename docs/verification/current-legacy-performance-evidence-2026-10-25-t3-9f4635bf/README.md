# Current and legacy performance evidence audit

2026-10-25, America/Vancouver. Parent: `22e411d1f1a4514ce6598227806ffa1d41a2347e`.
Owner: `t3-9f4635bf`. **READ-ONLY EVIDENCE AUDIT; no new measurements.**

There are **zero qualified current-versus-legacy runtime metric pairs** in the
bounded tracked evidence inspected here. RAM, CPU, input latency, launch,
clipboard, reconnect and endurance improvements remain unmeasured against the
reference app. Do not draw improvement bars or a completion percentage from
these records. Missing values are null, not zero.

## Exact candidates and reference

- The audit target is parent `22e411d`. Its workspace version is `0.1.0`.
  The retained ordinary M1 app is `0.1.0`, build `1`, from product source
  `ed7d96bf53da1a539fa2acde3b7133613221fd4b`. Every tracked change between that
  source and the parent is under `docs/` (320 paths). This proves tracked
  non-documentation source continuity, not a new build or runtime result.
- The M1 record reports arm64, macOS `27.0.1 (26A434)`, Xcode `27.1 (27A9275)`,
  Swift `6.4`, SDK `27.0`, Rust `1.99.1`, two jobs and deployment target `15.0`.
  App SHA-256: `094d02b3831ee6565ae1628407756ad48053a0268f2a489c65bbde28b768d775`.
  Daemon/peer SHA-256: `17065c334290b34c7a1b69483f409655e072eb0505ff2a0cc0bf6ed5f77a7bb2`.
  Those are saved public record bindings, not hashes of private files read now.
  No hardware model beyond the record's M1 label was established here.
- The reference is **Synergy 3.7.0-beta**, arm64, inspected on macOS `27.2` on
  2026-10-21. Its core is **1.21.1-beta**, with wire version **1.8**. The pristine
  core SHA-256 is `616e13a00b9981bbff0fea25f73a162c410e86cc21eac78eee198f4513e9bd2b`.
  Product, core and wire versions are different identifiers. The tracked reports
  do not bind a complete reference bundle hash or hardware model to the samples.
  No installed reference or private research scratch directory was read.

## What the numbers support

The M1 build worker took **101.050310709 s** after preparation through inventory.
Dependency resolution took 9.432977791 s; bundling took 87.455429375 s. These are
build observations, **not app launch latency**. App payload was 10,050,709 regular
file bytes in 95 files. Synergy's recorded payload was 548,821,397 regular file
bytes. These inventories describe different packages and features. They support
descriptive payload labels only, not runtime efficiency or a current reduction
percentage. The old 6,324,472-byte Openergy package in the existing comparison
belongs to `6b83420`, not this M1 bundle.

The M1 ordinary headless flow reported **25/25 app checks and app exit 0**.
Its controller exited 1 after 6.764437458 s at the first strict port rebind,
`EADDRINUSE_ERRNO48`. Overall status stays **FAILED_NOT_ACCEPTED**. The duration
is neither a startup measurement nor a performance distribution. Whole-flow
NI15, the second post-bind and original final bundle/outcome/archival gates stay
missing. The older ordinary run remains **FAILED / NOT ACCEPTED, 23/25**.
The later 2026-10-25 readiness and next-operation packages report no new product
flow; their matching files and source reviews do not close these runtime gates.

The historical ARM64 S15 run at `4a65290` used two virtual daemons, plaintext
same-host loopback and no GUI. Its 1,000-event IPC-to-receiver observation had
p99 **0.936791 ms**. Server/client CPU windows were about 60 s, with sampled peak
RSS **9,469,952 / 9,420,800 bytes**. The daemon hash matches the old `6b83420`
package. Synergy's three passive native connected-session snapshots, five
seconds apart with GUI closed, instead report core RSS **53,264–53,936 KiB** and
three-process summed RSS **147,040–147,792 KiB**. Its displayed 0.0% CPU values
are rounded passive readings. These different process sets, workloads, methods
and artifacts cannot form a performance pair. A summed RSS is not unique memory.

The historical connected r3 headless app observation retained 33 steady samples
over **31.519038541 s**, with virtual TLS peers and compiler overlap in all 35
runtime sample batches. It has useful app/daemon/peer RSS ranges, but overall
status remains **FAILED** for normal peer-stop timeout and strict first rebind.
Raw CPU fields incorrectly said nanoseconds; their actual units are Mach ticks.
The saved correction uses a 125/3 timebase. No CPU percentage is derived here.
The two earlier idle attempts remain **UNVERIFIED**, with zero resource samples.

The only stop-fix pair included here is historical Openergy versus Openergy:
**1254.038 ms before / 252.767 ms after**, request to independently observed
normal exit. Each side has one sample. The real public CLI/daemon path, role Off,
virtual backend, TLS enabled, discovery disabled and shared `CLOCK_MONOTONIC`
instrumentation match; the private configured ports differ. Both used the same
CLI trace overlay. This supports that narrow stop-fix observation only. It is
not a Synergy comparison, whole-app improvement, distribution, one-second exit
contract or connected-server result. Both stop acknowledgements and the separate
reply-to-exit values are retained in the data.

Favorable historical trust-list timings are retained together with the costs:
the small-list median rose **10.785625 → 11.643292 ms**; large-list peak RSS rose
**9,551,872 → 9,781,248 bytes**; executable size rose
**3,674,000 → 3,690,816 bytes**. No-op import RSS fell and overlapping-import RSS
was unchanged. The old finite sustained workload passed its declared gates but
still grew by **4,194,304 / 4,440,064 bytes** in late server/client medians, with
positive fitted trends. Full diagnostic polling had additional RSS costs.
These are older Openergy lab/control records, not legacy-app performance proof.
Original Rosetta CPU failures remain visible as separate platform observations.

Historical event delivery records also have exact bounds: daemon event timestamp
to model-log polling is not physical input-to-screen latency. Focus-lab files
named `legacy` use Openergy with numbering off or a lab Synergy-style injector;
they do not measure the shipped Synergy app. The Windows Synergy interoperability
record uses session 0, SSH tunnels and a virtual Mac backend; it supplies no
matched desktop performance comparison. No compressed focus-lab archive needed
to be opened to establish those scope limits.

## Data and visualization contract

- [comparison.json](comparison.json) contains identities, units, sample counts,
  measured values, exact source selectors, scoped comparisons, failed statuses
  and missing gates. `qualified_current_legacy_runtime_pairs` is empty.
- [sources.json](sources.json) binds each selected tracked source to the parent
  commit, Git blob ID, SHA-256, byte length and last content-change commit.
  JSON selectors point into those immutable bytes. Markdown line selectors
  identify report-only evidence. A raw report hash without tracked raw bytes
  remains a binding, not independent inspection of that private raw file.
- [static-audit.json](static-audit.json) records data-only integrity checks and
  output hashes. It is not a runtime acceptance receipt.

Use separate panels for historical virtual observations, reference snapshots,
payload inventories, correctness/build status and historical Openergy repair
pairs. Keep units and status visible. Do not combine platforms, sum independent
RSS maxima, convert rounded 0.0 CPU to zero work, relabel a lab injector as the
reference app, or relabel an old binary as a new measurement. A source-equivalent
artifact remains a saved measurement under its original host and workload.

## Next minimal real comparison

First close the current ordinary flow's existing acceptance gates with the
reviewed controller. Do not reinterpret the prior failed bind or retry an expired
operation. The current audit authorizes no runtime action.

For a first limited performance pair, then bind exact current and shipped
reference app/daemon bytes on **one common Mac and OS**, with the same real peer,
role, TLS/discovery settings, GUI state and polling. Predeclare one finite
connected-idle workload and a duration (for example 60 s), process inclusion,
whole-stack RSS and kernel CPU-time definitions, sample period, instrument
overhead, cleanup and failure rules. Collect one bounded pass per app through
its real user path, with raw timestamps and independent process-exit observation.
Keep both outcomes even if either fails; do not substitute another host, port,
virtual backend or later attempt. One pair supports only those observations.
Any median/p95/p99 or distribution claim needs a separately declared sample plan.
Input, clipboard, launch, reconnect and endurance each need their own matched
operation and measurement boundary. None follows from idle or build time.

This audit used tracked file/Git/JSON/hash reads and new data transformations
only. No supplied source was imported or executed. No private runtime state,
network, process/port query, product, build, test, mock, signing, browser, desktop
or child agent ran. Only this new folder changed. No TypeScript changed, so no
TypeScript build/check was relevant to this docs/data-only audit. Commit hooks
and signing are disabled for the audit commit.
