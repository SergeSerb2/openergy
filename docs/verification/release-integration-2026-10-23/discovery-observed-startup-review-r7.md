# Observed startup verifier review R7

The independent source review of candidate `2253049d72f4ee5aa079045432c5014bb0c4be78`
found no concrete P1/P2 product defect. It found two P2 verification defects.
The candidate is not integrated or accepted. This review ran no binaries.

1. The standard receiver is drained before the replacement call. A buffered
   `SearchStarted` emitted before the actual failed send can then satisfy the
   repeated-search check. Drain after the failed-send outcome and command barrier,
   then require a new event within a finite deadline covering the retry interval.
2. The accepted-drop check reads only `limited-view-states`. That gauge cannot
   prove cleanup of a standard subscription, and the probe does not require an
   accepted active-standard-drop witness. Require that lifecycle and observe its
   actual automatic cleanup before explicit stop. Missing coverage must fail.

The startup trace observes the real send result. A public command barrier follows
accepted sends. Distinct starts and four replacement combinations remain separate.
Unknown, missing, duplicate and full-channel outcomes fail. Trace storage is bounded.
The reviewer found no worker-control hook or product behavior change in the trace
delta, and matched the recorded vendor hashes and reverse patch.

The parent checked both findings against the immutable candidate source and sent
them to the active implementation task. A newer source commit, matching real-flow
evidence and a fresh independent review are required. Earlier reports stay intact.

The original Windows startup result stays **FAIL, 11/14**. Retained discovery
19/19 and virtual control 23/23 remain separate results. Native input/clipboard,
positive Windows migration, slow DNS, whole-cache/expiry/timer/query/byte bounds,
original security/provenance and trusted distribution gates remain open.

The [full review record](discovery-observed-startup-review-r7.json) preserves the
task identity, requested model settings, findings and source-only limits.
