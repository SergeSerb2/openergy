# Repaired M1 sampler binding — execution false

Sampler source R3 is committed as
`7f20c9cdd97c1dc313f007b879babc726acbb914`. It addresses the sole P2 in the
[R2 review](../m1-bundle-controller-review-2026-10-24-t3-eaede70a/REVIEW.md).
The [exact repair](../m1-sampler-source-repair-r3-2026-10-24/README.md) is now
under fresh independent R4 review. No controller or build ran.

The parent copied this exact recipe and a new disabled operation to another
fresh private M1 directory. A separate data reader confirmed all four files'
bytes, hashes, ownership, single links and mode0600, and the directory's
mode0700. It confirmed that the new planned build root does not exist.
The original staged R1 recipe remains disabled and unchanged.

The prepared operation uses the same qualified Python prefix and immutable
source/Rust/Cargo inputs. Its controller hash is
`cd5dc620e6adbec09798e600a99f55c0368b8eeff10d1117e1f7793ca1baf4ee`.
The full Python prefix digest remains the previously bound
`c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67`;
the controller must recheck it before work and after building. It was not
represented as a new live prefix scan in this binding step.

Only the common absolute UTC deadline and execution flag remain to be filled
immediately before any source-cleared, already-authorized ordinary invocation.
All limits and source checks remain unchanged. No previous operation was
started, reset or retried. No old failed run or Sparkle consent was reopened.

[binding.json](binding.json) retains hashes, private data labels and pending
task IDs. Full machine paths remain in the dedicated private state tree.
Source review is still pending; runtime, bundle and release acceptance are
UNVERIFIED. The Windows command-length repair is a separate active task.
