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
failed at 11/14. A separate ordering diagnosis is running; the original failed
probe and its frozen inputs remain unchanged.

The [fresh startup contract review](discovery-startup-contract-review.md) found
a P2 verification defect: the probe does not prove that its target startup
sends failed before replacement. It found no concrete product defect in that
scope. The possible ordering still needs a native trace; acceptance stays failed.

The [bounded virtual idle observation](virtual-idle-resources.md) records two
30-second phases and the actual stop delay on the checkpoint binary. It does
not supply native performance or long-term memory acceptance.

The [source-history scan](source-history-refresh.md) retains raw exit 1 with
84 findings. All 84 are verified source-file hash records, with none unresolved.
The parent independently repeated 96 source-blob comparisons. Its patch-history
scope excludes proof of merge-only resolution coverage and later commits.
It is not a zero-finding or broad secret-free result.
