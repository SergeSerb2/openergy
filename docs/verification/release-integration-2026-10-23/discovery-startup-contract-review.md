# Discovery startup contract review — 2026-10-23

A fresh independent source/evidence review found no concrete P1/P2 product
defect in the scoped discovery lifecycle. It found one **P2 verification defect**:
the startup probe does not establish that the target initial sends failed.
The [complete review](discovery-startup-contract-review.json) preserves code
references, a falsifiable ordering check, clean areas and remaining limits.

The reviewed checkout is `4deb2d2`. Its product, vendor and probe source is
identical to `e4f48ba`. The reviewer ran no binaries, builds or traffic and made
no edits. This review does not replace native runtime acceptance.

Later evidence: the [two-run native diagnostic](../release-integration-2026-10-22/windows-discovery-startup-diagnosis-2026-10-23.md)
demonstrated the possible ordering in a matched reproduction. The original
untraced failure keeps its result and its precise interleaving remains unknown.

## Missing precondition

The probe queues sixteen cache replays, drops thirty-two distinct limited
handles, then immediately drops limited and standard replacements of the live
guard. It requires the old guard to survive. Completed replay overflows prove
that replay work finished; they do not prove it still delayed these last sends.

`send_cmd` queues before creating/sending its wake notification. The real worker
can process a replacement while its receiver still exists inside the caller's
API call. A returned handle's immediate drop does not prove a failed initial
`SearchStarted` send.

The product commits replacement only after a successful initial send. It
preserves the old sender, shared state and retransmissions when that send fails.
After a successful start, dropping the new receiver uses normal disconnect
cleanup. That is the documented contract.

One source-compatible ordering lets the cancelled limited replacement fail its
send and preserve the old guard, then lets the following standard replacement
succeed before caller drop. That standard replacement legitimately removes the
old limited state. Later disconnect cleanup removes the new subscription. The
limited-state gauge can therefore become zero and the old handle can close.
This accounts for all three observed false checks without proving a product bug.
It is a possible ordering, not an observed Windows event trace.

## Acceptance and next proof

**Windows startup remains FAIL, 11/14.** The original output, frozen inputs and
assertions are unchanged. Source inference does not establish the cause of that
run. An ordered record of receiver-drop completion, initial-send outcomes,
replacement commit and cleanup identity is needed.

A successful target send followed by replacement supports the probe-race
explanation. Both target sends returning `Disconnected`, followed by old-view
loss, would contradict it and require product investigation. Initial dirty
snapshots, an open view, or a later metrics barrier alone cannot prove this
ordering.

The separate native diagnostic is collecting the narrowest available evidence.
It may not turn missing ordering evidence into a pass. The review's opt-in
worker-rendezvous proposal is a proposal only; it has not been implemented or
approved for the current diagnostic. Adding sleeps or replay work, accepting a
lower state gauge, or skipping identity checks would not fix the precondition.

The review found failed-send ordering, sender-generation checks, bounded map
delivery, snapshot replacement, wrapper closure handling and direct Exit
ownership sound within its source scope. Native input/clipboard, positive
Windows migration, whole-cache/expiry/timer/query/byte bounds, slow DNS, original
security/provenance and trusted distribution remain open. The stopped IPC task
was not resumed or replaced by this review.
