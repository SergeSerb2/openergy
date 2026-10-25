# One M1 ordinary app flow: 25/25 app checks; overall FAILED

The parent ran one already-authorized headless ordinary flow against the fresh
M1 bundle. The app reported all 25 unchanged checks passed, and its retained OS
exit was 0. The overall attempt remains **FAILED / NOT ACCEPTED**.
The first strict postflight bind on loopback port65431 returned EADDRINUSE,
errno48. Its saved pre-bind scoped netstat rows are empty. The cause is unproved.
No second bind, alternate port, SO_REUSE option or flow retry ran.

The controller's actual OS exit was 1 after 6.7644 seconds. That is one failed
controller invocation duration, not a performance distribution, UI startup
measurement or comparison with Synergy. The second postflight port65432 check,
normal reserved-file rename archival, final bundle-after record and original
final outcome were not reached. No missing final gate is filled as a pass.

The collected AppModel/IPC checks cover connection, safe Off startup, real peer
join/disconnect/rejoin, settings, layouts and imported fields, live logs,
invalid-layout rollback, stalled IPC responsiveness, intentional owned daemon
crash/recovery, and import warning/empty cases. Clipboard checks change model
settings; they do not prove physical clipboard transfer. Rendering and the
native input backend were disabled.

Saved cleanup reports record all four direct OS exits as 0: app, peer,
peer-status and peer-stop. The collector sealed the intent/handle/exit sets.
The watchdog recovery report records six known identities as ESRCH, private
sockets absent, no unresolved candidates, no overflow or lost handoff, and no
recovery TERM/KILL deliveries. The watchdog itself exited 0 and its independent
absence check passed. These records support observed owned-process cleanup;
they do not turn the strict port failure into acceptance or prove its cause.
Full priority/identity coverage and all source contracts are under independent
saved-data audit. Do not infer universal containment or host restoration.

Preparation ran once and returned OS exit0. The source admission was cleared by
[the independent R2 review](../m1-ordinary-admission-review-2026-10-24-t3-af8da74b/REVIEW.md).
The [artifact assessment](../m1-ordinary-data-source-assessment-2026-10-24-t3-a104ad9f/ASSESSMENT.md)
supports the immutable source, Python prefix, arm64 interfaces and ad-hoc code
objects. Source and copied app inventories matched; no rebuild, re-copy or
re-signing ran. One parent data-transfer error before any write and one parent
verbose-message-order comparison failure are preserved. The corrected metadata
comparison kept all exit codes and content, allowing only order changes in
parallel verifier progress lines. It completed the remaining 41 checks without
repeating the first nine. All 50 copy metadata commands returned 0 before the
single product flow. No prior failed checker record was rewritten.

After failure, a separate read-only retention pass archived 100 original regular
data files plus its own manifest. It made no new process/port query or bind.
The 182,473-byte archive SHA-256 is
`535ba1f87258d8e94f584c427db890fa599acb6b6e06cfbfb0d60dea3dbb9f24`.
All 101 members and 100 source file bindings were verified locally as inert data.
That separate pass also found the copied full bundle tree unchanged. It does
not replace the original missing final bundle-after or archival gate. Raw paths,
logs, process observations, reports and qualification receipts stay private.

[result.json](result.json), [app-checks.json](app-checks.json), and the
[private evidence index](private-evidence-index.json) retain the scoped results
and hashes. The old ordinary run stays FAILED / NOT ACCEPTED23/25. The old
isolation failures and their unknown cause remain unchanged. Windows native
stop remains unverified; its new source fixes have no native acceptance. The
expired Sparkle one-shot stays closed. Native UI/input, trusted signing,
notarization and matched performance comparison remain open.
