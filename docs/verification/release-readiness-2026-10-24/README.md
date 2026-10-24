# Active hardening round — 2026-10-24

The user resumed work at 09:53:14 PDT. The goal remains incomplete.
One independent source review remains active. No new runtime has started.

| Work | Current supported result | Remaining gate |
| --- | --- | --- |
| Ordinary controller | R9 passed independent R10 review: no actionable P1/P2; prior repairs preserved. | Compiled app, signed ordinary bundle, icon and raw workspace state remain unavailable at checked named paths. |
| Windows adapter | Both earlier containment/recovery findings are addressed. The later missing-PID observation defect has a one-line direct API repair in `762240a`. | Fresh independent R10 review is running. |
| Ordinary retained data | Matching daemon, six receipts and bindings: 21 private data files, 4,850,035 bytes. | Complete ordinary input set remains INCOMPLETE. |
| Pinned Sparkle data | All 124 historical records match: 115 files, 27,348,967 bytes, and 9 symlinks; exact official archive checksum matches both pins. | Data identity only; no architecture, signing, loading, build or runtime acceptance. |
| Windows fixture facts | Bounded search exhausted 160 retrievals across six threads. Zero of nine bindings qualified. | No runnable baseline selection exists. Exact historical paths remain critical missing facts. |
| Isolated Mac | Connected local/direct/cloud sandbox inventory each returned zero rows. | No suitable existing isolated build-diagnosis environment was established. |
| Running Sparkle operation | Scoped LS source review clear; earlier one-shot failed before product launch. | Expired marker, closed consent and missing artifacts keep that old operation closed. |

[Ordinary source review](../ordinary-controller-R9-review-2026-10-24-t3-3f0af888.md)
records the bounded source result. The
[Windows R8 review](../windows-adapter-p2-review-2026-10-24-81ddd0c1/REVIEW.md)
explains the cmdlet exception mismatch; the
[new repair](../windows-missing-pid-repair-r9-2026-10-24/README.md) preserves all
other source bytes and budgets. Its one-hunk change awaits independent review.
Own data-reader count and staging-whitespace errors are preserved; neither was
a native attempt. The 178 published inventory members and all 179 historical
folder files remain exact.

[Pinned dependency recovery](../sparkle-pinned-input-recovery-2026-10-24-bdfbf824/README.md)
retains exact public bytes in the durable private state parent. The archive is
10,193,895 bytes, SHA-256
`17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
The private receipt SHA-256 is
`7dce50e63988399ba6b062383286b6d74c7b8a501bb3defdb06b023ee1ce6c8a`.
Actual own-worker NI15 was recorded. One metadata fetch and one archive request
completed retention in 95.50 seconds. This restores data, not a historical
workspace file or compiled app.

[Windows fact recovery](../windows-fixture-fact-recovery-2026-10-24-a25e3605/README.md)
keeps partial facts distinct from a valid selection. It created no
baseline-selection.json and made no host connection. No guessed path, reduced
selection or current scan substitutes for the missing historical bindings.

All old runtime failures remain unchanged: ordinary acceptance FAILED / NOT
ACCEPTED, 23/25; native Windows Off stop UNVERIFIED. Source review and data
retention do not establish runtime acceptance. No new build, signing, native
command, product launch or runtime occurred. No machine was created or started.
The stopped experiments, build/trace gates and expired one-shot stay stopped.
No push or publication occurred.

[Durable continuation state](continuation.json) retains exact task IDs, commits,
private data directory labels and hashes without transport values or raw logs.

