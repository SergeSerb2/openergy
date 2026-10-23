# Mac DNS evidence review R10 — 2026-10-23

R10 found one P2 reporting defect in evidence `4f506a9` for source
`8333a48`. No concrete product P1/P2 was established. This saved-evidence audit
did not execute builds, probes, helpers or traffic. The [full record](dns-verifier-evidence-review-r10.json)
preserves its scope and findings.

The historical summary says both earlier runs contain six hostname false
`Connected` observations. The `7a56d17` run contains **four**; its other two
hostname cases report typed refusal. The `40b1d12` run contains **six**.
The parent repeated the raw-line and derived-event counts and matched all
derived events to their raw logs. The implementation task received a narrow
summary correction request. Raw transcripts and exits must stay unchanged.
Neither historical run satisfies the final strict negative-control gate.

The audit matched all 132 source hashes and final binary/library identities.
Final normal and conflict commands retain exit 2 and durations of 104.971 and
74.406 seconds. Both recorded child niceness 15 from a priority-0 controller.
Saved build records retain pinned Rust 1.99.1, two jobs, deployment target 15.0
and locked/offline operation.

Normal records ten refusal matrix requests; conflict records six safe requests
and four skipped requests after actual IPv6 reservation conflict. Both record
eight 16-job queue batches, frames and lifecycle markers. All six final
same-driver timeout recovery exchanges have exact frame and event records.
These literal results do not close the separate [R9 source defects](dns-verifier-source-review-r9.md)
in base address ownership and complete event handling.

All 22 timing cases match their derived brackets. Strict TCP/TLS receipt
intervals overlap deadline bounds, and fallback reaches only one IPv6 attempt.
Refusal, strict timing, fallback and the matching new negative control remain
UNVERIFIED.

All 212 ledger entries match the 213-file archive; 190 private originals match
their hashes and declared transforms. The 48 historical DNS files, 34 pre-R7
raw records, 114-input baseline manifests and 13 compiled-source patch sections
reconcile. Immediate cleanup audits preserve two address-in-use results per
command; a later audit records 40/40 successful binds. Empty OS inventories
supply no broader proof. Private permissions and recorded identity removal
remain consistent. No Windows or full candidate acceptance follows.
