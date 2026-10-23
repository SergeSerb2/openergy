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
under a separate audit. No DNS candidate has been integrated here.

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
