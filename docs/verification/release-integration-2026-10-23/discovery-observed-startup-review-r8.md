# Observed startup verifier review R8

The independent source review of `e6a9125be2fc59fbe92fef6ea351b222298bfc52`,
including parent `2253049d72f4ee5aa079045432c5014bb0c4be78`, found no concrete
P1/P2 product or verification defect. Both R7 objections are closed in source.
This result does not approve runtime or native Windows acceptance.

The verifier drains the old standard receiver after the failed-send outcome and
public command barrier. It then requires a new repeated search within a finite
deadline based on the observed search count. Unsupported coverage fails.

A mandatory held standard subscription must be accepted, committed, resolved and
active before Drop. Its automatic cleanup requires exactly one matching normal
`Disconnected` removal, followed by a public command barrier. A missing, duplicate,
wrong-kind, unknown or `Full` removal cannot pass. Explicit stop follows failure.
The worker emits cleanup traces after removal; explicit stop and replacement do
not emit those records. Initial-send traces remain separate from commit proof.

The review also checked the four required failed-replacement witnesses, finite
attempts, bounded observer storage, vendor inventory and reverse patch. Retained
tests, licenses, manifests, lockfile and notices are consistent. Disabled traces
add no formatting allocation. Enabled logging can affect timing.

The reviewer read immutable Git blobs and did not inspect the implementation
agent's mutable evidence. It ran no builds, binaries, probes, tests or traffic.
Matching baseline failure, missing-observer cleanup controls, retained 19/19 and
owned-shutdown 4/4 evidence still require acceptance. The original Windows startup
result remains FAIL, 11/14. Other release gates remain open.

The [full review record](discovery-observed-startup-review-r8.json) preserves the
task identity, model settings, source scope and limits.
