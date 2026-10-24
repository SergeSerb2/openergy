# Resumed hardening — 2026-10-23

Work resumed from clean checkpoint `e4f48ba` with GPT-6.1-sol agents using xhigh
reasoning and standard service mode. The [October 22 handoff](../release-integration-2026-10-22/pause-handoff.md)
retains its historical checkpoint scope.

The [fresh dependency-policy observation](dependency-policy.md) passes all six
graphs and checks 106 registry versions. Runtime acceptance remains separate.

The [virtual checkpoint baseline](virtual-baseline.md) passes all 89 existing
non-Docker scenarios with zero cleanup failures. Thirty Docker cases remain
explicitly excluded.

Provider safety review stopped the Synergy 3 import-budget task for possible
cybersecurity risk. Its partial uncommitted source and reports remain in the
isolated child worktree; they are not integrated or accepted. The
[stopped-task record](ipc-task-stopped.json) preserves status, input/report
hashes and cleanup observations. No replacement task was started for that work.

The [native Windows run](../release-integration-2026-10-22/windows-bounded-discovery-2026-10-23.md)
passes retained discovery 19/19 and virtual control 23/23. Startup remains
failed at 11/14. The original failed probe and its frozen inputs remain unchanged.

The [fresh startup contract review](discovery-startup-contract-review.md) found
a P2 verification defect: the probe does not prove that its target startup
sends failed before replacement. It found no concrete product defect in that
scope. The later [native diagnostic](../release-integration-2026-10-22/windows-discovery-startup-diagnosis-2026-10-23.md)
observed both failed-start preservation and accepted replacements before caller
drop. Its second run reproduced the three failed checks. The parent checked the
committed trace ordering and copied output hashes. The original untraced run's
precise interleaving stays unknown, and its acceptance result stays failed.

A separate implementation task repaired the verification with actual startup
outcomes and distinct failed-start/accepted-start cases. It preserves identity,
state and cleanup checks and supplies a failing baseline. Native acceptance
retains its separate gate.

The [R7 source review](discovery-observed-startup-review-r7.md) of its first
candidate found two remaining verification gaps: a pre-failure search event could
pass the repeat check, and the limited-state gauge did not prove standard-browse
drop cleanup. That first candidate was superseded. A separate Windows observation
checks actual worker/socket removal after normal shutdown and Drop while each
private helper stays alive.

The [fresh R8 source review](discovery-observed-startup-review-r8.md) found no
concrete P1/P2 product or verification defect in revised candidate `e6a9125`.
Both R7 objections are closed in source. Native Windows startup acceptance
remains open. The reviewed source is now
integrated locally as `7f27cbb` and `6da26a1`; product/probe source matches
`e6a9125` exactly.
The [fresh combined release run](combined-virtual.md) passes all 89 existing
non-Docker scenarios with zero cleanup failures. All 248 recorded inputs and
both frozen artifacts stay unchanged, and no owned process or assigned listener
remains. Thirty Docker cases stay explicitly excluded.

The [final Mac discovery evidence](discovery-startup-r7-repair.md), from `8e891f1`,
is integrated locally. The [independent R9 audit](discovery-final-evidence-review-r9.md)
found no concrete P1/P2 source or final-evidence defect. It supports startup
61/61 twice, a working-observer baseline failure of 45/61, retained recovery 19/19
and actual owned shutdown 4/4. It audited saved records and did not independently
execute binaries. Three superseded private metadata originals changed from old
raw hashes; their committed copies remain intact and final acceptance does not
depend on them. Windows startup and broader release gates remain open.

The separate [native Windows cleanup observation](windows-owned-cleanup.md)
passes normal explicit shutdown and Drop on the frozen `e4f48ba` library. The
identified mDNS worker and all owned UDP endpoints disappear while each helper
stays alive and answers a later ping. The parent matched the saved snapshots,
96 private evidence-file hashes, eight exact helper/workflow copies and frozen
library/source records. This closes only those two normal cleanup cases.
The [revised native Windows run](windows-observed-startup.md) on `e6a9125`
records strict startup failures of 105/106 and 113/115 because required failed-send
witnesses are missing. Retained discovery passes 19/19; missing-observer controls
fail as required. The [independent R10 audit](windows-observed-startup-review-r10.md)
supports those saved results and finds no concrete P1/P2 source or evidence
defect. It does not supply independent runtime execution. Both startup gates
stay unverified, and witnesses cannot be pooled across runs. The original
11/14 failure stays preserved.

The [bounded virtual idle observation](virtual-idle-resources.md) records two
30-second phases and the actual stop delay on the checkpoint binary. It does
not supply native performance or long-term memory acceptance.

The [Windows DNS contract review](windows-dns-contract-review-r5.md) found two
P2 verification defects: a short OS control blocks the actual API cases, and
the refusal verdict does not enforce the caller deadline independently of the
observation window. No product P1/P2 was established. An isolated verifier repair
is in progress. It must keep the original failures and add real deadline
coverage before native acceptance.

The [DNS verifier R7 source review](dns-verifier-source-review-r7.md) confirms
the R5 timing repairs but finds two remaining verification gaps in `7a56d17`:
refusal port ownership ends too early, and the new pending-timeout cases omit
healthy reuse of the same driver. That candidate is not accepted. The existing
implementation task is addressing both findings and preserving its reports.

The [DNS verifier R8 review](dns-verifier-source-review-r8.md) of `082baaf`
finds four remaining verification defects: qualification failure stops
independent flows, unqualified refusal timeouts become failures, macOS TLS
handoff releases ownership, and incomplete or duplicate traces can grant
coverage. Same-driver recovery calls are now present. No product P1/P2 was
found. The candidate remains unaccepted while the implementation addresses
the findings and preserves unavailable Mac coverage as UNVERIFIED.

The [DNS verifier R9 review](dns-verifier-source-review-r9.md) of `8333a48`
finds two more verification gaps: base hostname flows do not own every target
family, and base frame exchange can discard extra public events. Most R8
objections close in source, but this candidate remains unaccepted. The parent
instructed the Windows task to stop further probe commands and complete cleanup.
A new implementation follow-up addresses both gaps; saved Mac evidence remains
under a separate audit. The later [R11 integration](dns-verifier-r11-integration.md)
records the exact reviewed source now integrated here and its open runtime gates.

The [Mac DNS evidence R10 audit](dns-verifier-evidence-review-r10.md) supports
the saved source/artifact bindings and literal exit-2 results, but finds one
historical count error: the earlier two runs contain four and six false
`Connected` observations, rather than six each. The parent repeated both counts
and sent a narrow summary correction request. Raw transcripts stay unchanged.
Neither run closes the strict negative-control gate, and R9's source defects
still prevent candidate acceptance.

The [Windows DNS closeout](windows-dns-repaired-8333a48.md) records native
release and strict Clippy success for unaccepted `8333a48`. Neither runtime
command started: both are NOT RUN, with no exit or check count. No TLS identity
or conflict socket was created. Saved cleanup and invariance audits pass.
The parent matched 109 source-input bindings and 25 public-copy hashes.
This build-only evidence grants no DNS runtime acceptance. The wrapper's
session/priority were observed; cargo/rustc child priority was not sampled.

The later [native final verifier run](windows-dns-final-8281eb7.md) uses the
R11-reviewed source. Both full commands exit 2. Each passes 10/10 qualified
refusal cases and three same-caller recoveries. Normal queue/lifecycle coverage
passes; the conflict run skips eight unsafe batches. Strict timing and full
acceptance remain unverified. The [R13 audit](windows-dns-final-review-r13.md)
finds no concrete P1/P2 in those saved records and adds no runtime reproduction.

The [matching historical Windows control](windows-dns-historical-refusal-615c16e1.md)
also passes all ten refusal cases and exits 0. It captures no historical fault,
so it does not close the negative-control gate. The
[R15 saved-evidence audit](windows-dns-historical-review-r15.md) found no concrete
P1/P2 and confirms that limit. The new
[macOS fixture qualification](macos-owned-refusal-qualification.md) rules out
listener shutdown as an owned-refusal method; its successful TCP connections
are ordinary OS behavior, not false-Connected product events.

The later [combined-binary idle observation](combined-idle-resources.md) measures
the exact new release artifact. Its role-off idle RSS is 8.265625 MiB and reaches
8.609375 MiB during 30 status CLI calls. Displayed CPU-counter deltas are 0.00 s
at 0.01-second resolution; this does not prove zero CPU work or a speedup. Actual
stop exit takes 1.276 seconds. Config/artifact hashes and cleanup checks pass.

The [source-history scan](source-history-refresh.md) retains raw exit 1 with
84 findings. All 84 are verified source-file hash records, with none unresolved.
The parent independently repeated 96 source-blob comparisons. Its patch-history
scope excludes proof of merge-only resolution coverage and later commits.
It is not a zero-finding or broad secret-free result.

The [merge-diff supplement](source-history-merge-supplement.md) closes that
specific gap at the same pinned head. All 11 merges are covered by 21 emitted
parent diffs and one identical-tree parent. The incremental scan exits 0 with
zero findings. The parent independently matched every direct diff and the full
patch-stream hash. The primary exit 1 and 84 findings remain unchanged. This is
patch-history evidence, separate from product/provenance review; it does not
scan stopped sibling refs or establish every-snapshot secret absence.

The [current-parent history snapshot](source-history-current-snapshot.md) repeats
both scans at pinned `7ad8265`. Its non-merge scan retains raw exit 1 with 95
findings; all are verified source-file SHA-256 records, with none unresolved.
The merge scan exits 0 with zero findings. The parent repeated all 95 location
bindings, 107 source-blob comparisons, 22 merge-parent diff checks and 60 private
artifact hashes. This snapshot covers 254 reachable commits and excludes later
work, sibling refs and every-snapshot secret-free claims.

The later [Sparkle-era history snapshot](source-history-sparkle-snapshot.md)
covers pinned `8c9f0ab`, including Sparkle integration. Its non-merge scan retains
exit 1 with 105 findings: 104 source-file SHA-256 records and one public
verification key, with none unresolved. The merge-parent scan exits 0 with zero
findings and accounts for all 22 parent pairs. The
[independent audit](source-history-sparkle-saved-audit-c7c772c.md) finds no concrete
P1/P2 and verifies every classification and location. It adds no scanner run.
This frozen patch-history scope excludes later commits and does not establish
security clearance or secret-free historical snapshots.

The [current packaged macOS app flow](../macos-app-current-flow-2026-10-23/README.md)
passes 25/25 model/IPC checks with private virtual daemons. The separate
[bounded event measurement](../macos-event-delivery-2026-10-23/README.md) passes
12/12 paced checks and has a clean saved-evidence audit. It establishes neither
a queue bound nor a speedup. Native UI/input/clipboard acceptance stays separate.

The separate [caller cancellation flow](../macos-caller-cancellation-2026-10-23/README.md)
passes 19/19 checks: three pending callers return cancellation before daemon
resume, and the same client receives later healthy replies after the original
deadlines. The [independent audit](../macos-caller-cancellation-review-2026-10-23-24ad7903.md)
finds no concrete P1/P2 and matches all 17 artifact hashes and 421 source hashes.
It adds no runtime execution. Exact nice 15, private waiter/queue counts and an
independent per-PID cleanup check remain unverified. The default 25-step flow
was not rerun by this command.

The [first connected-idle measurement](../macos-idle-resources-2026-10-23/README.md)
failed in its launcher before app readiness. Its successful build and completed
cleanup do not supply a resource baseline. The unchanged failure and scheduling
deviation are preserved. The [second attempt](../macos-idle-resources-r2-2026-10-23/README.md)
fixed launch identity and priority, but failed trusted readiness and strict port
cleanup. The [third attempt](../macos-idle-resources-r3-2026-10-23/README.md)
corrected the private peer-name verifier and passed all twelve app checks,
including 31.519 seconds connected, disconnect and rejoin. Across 33 steady
samples, app RSS was 12.75–12.828125 MiB and its daemon RSS was
10.09375–10.140625 MiB. This headless model flow does not measure rendered UI.
The overall result remains **FAILED**: peer stop exceeded the private one-second
wait and the first exclusive port rebind failed. The [R4 audit](../macos-idle-resources-r3-source-evidence-review-2026-10-23-t3-4021ec78.md)
found no concrete P1/P2. CPU conversion remains conditional; compiler interference,
finite sampling and failed cleanup prevent performance or release acceptance.
The [shutdown source diagnosis](../daemon-stop-latency-source-2026-10-23-t3-d1ff2556/README.md)
establishes an avoidable poll after public stop acknowledgement. Saved timings
do not prove its sole contribution to the historical timeout. A narrow repair
and bounded real-command verification are in progress; the 250 ms shutdown
flush and joins must remain intact.

[Sparkle integration](../sparkle-integration-2026-10-23/README.md) and the signed
empty Cloudflare feed are implemented and checked. The later
[CLI build attempt](../sparkle-cli-build-preparation-2026-10-23/README.md) exceeded
host limits through Xcode registration/assessment tasks; its original artifact
remains unaccepted and persistent state is unknown. A
[prepared private stopped-target fixture](../sparkle-stopped-fixtures-2026-10-23/README.md)
has a separate file-only resource-seal repair and a clean corrected-scope review.
The user approved that scope at 17:16:55 PDT. The later
[stopped-target runtime](../sparkle-stopped-runtime-2026-10-23/README.md) downloaded
the full archive in both cases: the altered signature failed with error 4005 and
left build 1 unchanged; the valid signature installed the exact build 2 tree and
passed nested signature checks. The parent matched all 139 copied raw-record
hashes. The [independent audit](../sparkle-stopped-runtime-review-2026-10-23-t3-0760a73d.md)
also matches all 139 private originals and finds no actionable P1/P2. It adds no
runtime execution. The first preflight failure,
negative-case nice-20 deviation, XProtect diagnostics, and process-sampling
limits remain recorded. No target launch/relaunch was observed. This does not
close running-target handoff, native UI, Developer ID, notarization, or full
shared-OS-state restoration.

The [idle-TLS R2 audit](idle-tls-deadline-review-r2.md) found a Windows compilation
defect in the new opt-in probe at `9f6625c`: it selects the Unix listener wrapper
without a platform guard. The three-line fix in `06b9a13` is integrated here as
`2c31b26`. The [R4 audit](idle-tls-deadline-review-r4-364663a9.md) finds no
actionable P1/P2 and closes that defect in source. Fresh macOS release Clippy,
build and format checks pass; the first Windows transfer stopped before any
compiler ran. A later [R5 local setup failure](idle-tls-windows-build-r5/README.md)
also precedes remote work and remains preserved. The corrected
[R6 native Windows workflow](idle-tls-windows-build-r6/README.md) passes strict
release Clippy and release build on the exact fixed source. Its
[independent audit](idle-tls-windows-build-r6-review-f7a7f33b.md) finds no actionable
P1/P2 and adds no compiler or runtime execution.
The saved first-eligible-poll timeout and same-caller recovery checks pass, but
the original command remains **FAIL / exit 1** at immediate cleanup rebind.
Later TIME_WAIT records do not identify that failure's cause. The fixed source
has no new runtime result.

The [normal update-lifecycle source review](../macos-normal-update-lifecycle-source-review-2026-10-23-t3-ba2ba798.md)
finds no concrete P1/P2. It requires independent observation of old-child exit
and new private-config readiness before handoff acceptance. A supervisor stop
log or cleared PID alone proves neither. No-argument fixture isolation remains
under source/build preparation. Its [R2 review](../private-update-bootstrap-review-2026-10-23-t3-0a934f43.md)
found two P2 defects: unreadable bundle metadata can reach normal startup, and
the first app image enters Foundation before private-home setup. That candidate
is not integrated. A separate source/build repair is in progress; no running-target
update or relaunch has run.

The [signing prerequisite check](../macos-signing-prerequisites-2026-10-23/README.md)
records no Developer ID Application identity and no declared notary profile.
The [selector review](../macos-signing-timestamp-source-review-2026-10-23-t3-fe292237.md)
found that ambiguous Developer ID selectors could silently omit secure timestamps.
The [signing-purpose repair](../macos-signing-purpose-2026-10-23/README.md)
now requires explicit intent for ambiguous selectors and timestamps for distribution.
Its [independent review](../macos-signing-purpose-source-evidence-review-r2-2026-10-23-t3-b1cef944.md)
closes that P2 in source. Saved real ad-hoc signing and rejection flows pass;
named signing, timestamp service, notarization and nonempty Downloader entitlement
preservation remain unverified. No signing credential was used.

The later [absolute-priority repair and package run](../release-priority-2026-10-23/README.md)
builds the combined arm64 app/daemon, packages and extracts it, verifies signatures,
and passes both real virtual-IPC smoke flows. All 205 observed child processes
have nice 15. The overall verification remains **FAILED**: copied Swift build
state removed 114 old scratch-cache output entries. Their bytes and symlinks were
restored from exact retained copies, and the first failure remains preserved.
Full metadata restoration is unverified. No packaging retry or GUI launch ran.
The [independent priority review](../release-priority-2026-10-23/independent-review-t3-ab89755e.md)
finds no new source P1/P2 and corroborates recovery. The controller's cache
preservation P2 remains open; a fresh-build-state verification is being prepared.

The [slow-DNS prerequisite note](../slow-dns-fixture-prerequisites-2026-10-23-a3f15192.md)
identifies a possible Windows application-DNS API. The [settings-only helper](../windows-app-dns-qualification-prep-2026-10-23/report.md)
passed a native build and ABI checks, but all four execution proof gates remain
closed. Its [source/saved-build review](../windows-app-dns-review-2a7756f-2026-10-23-f9e74c74.md)
found no concrete P1/P2 and grants no runtime qualification. No helper execution,
DNS API call, DNS bind or query ran. Resolver confinement and reset lifetime
remain unverified.

The [Windows witness design](windows-failed-replacement-workflow-design.md)
establishes a public-API ordering limit: the worker may send before the caller
receives its new browse handle. More finite retries cannot guarantee a failed
initial send. Both saved Windows startup gates remain failed/unverified; the
conditional preparation/submission API design is not implemented.

The [finite release checklist](../release-readiness-2026-10-23/goal-status-1720.md)
separates implementation, saved real flows, native behavior, OS trust and
publication. Its roughly 70% planning estimate is explicitly unmeasured; the
checklist is not a numerical completion score.
