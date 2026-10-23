# Windows DNS verifier contract review R5

The independent source and saved-evidence review of `5248180` found two P2
verification defects. It established no product P1/P2 defect. Windows DNS
acceptance remains open. Original failed reports remain unchanged.

1. A direct OS control requires `ConnectionRefused` within 500 ms before the
   probe runs its five-second NetDriver refusal matrix and later lifecycle cases.
   Recorded Windows controls time out in that short window, while separate longer
   controls and actual API requests return refusal within the API budget. The
   short OS observation does not define the product deadline contract. Fixture
   qualification, short timing observations and actual API assertions must be
   separate; merely increasing the old timeout is insufficient.
2. The original refusal predicate and supplement can accept a refusal observed
   during six seconds without enforcing the caller's five-second deadline. The
   supplement logs first-event time but omits it from its pass predicate. A 5.5 s
   event could therefore pass. Recorded supplemental events all arrive before
   five seconds, so this is a source-supported oracle gap, not a demonstrated
   runtime deadline violation.

The proposed repair preserves the ten exact typed-refusal cases and observes
through six seconds for duplicate or late events. It must separately enforce the
caller deadline, record command/poll timing brackets, and add real short-deadline
and fallback-spanning deadline cases. An earlier legitimate refusal does not
count as timeout coverage. Missing preconditions remain unverified. All eight
queue batches, frame exchanges, stale-result checks and lifecycle controls stay.

The original control sorts/deduplicates addresses, while the product preserves
resolver order and one absolute deadline across sequential fallback. Actual
family/attempt observations are needed to establish the fallback deadline case.
No arbitrary grace period, global network/DNS setting change or retry-to-green
is authorized by this finding.

The reviewer matched source blobs and saved output hashes. It ran no builds,
probes, tests, Windows commands or network traffic. The [full review record](windows-dns-contract-review-r5.json)
preserves the exact budgets, findings, suggested real-interface checks and limits.
