# DNS verifier source review R8 — 2026-10-23

R8 found four P2 verification defects in immutable candidate `082baaf` against
`c7f6989`. No concrete product P1/P2 was found. The candidate is not accepted.
The [full record](dns-verifier-source-review-r8.json) contains the exact source
references. This was source review only; no build or real flow ran.

1. A failed exclusivity check exits before independent queue, lifecycle and
   timeout/recovery flows. Record the rejected fixture and keep affected gates
   unverified while continuing flows with safe owned fixtures.
2. A valid Timeout on an unqualified refusal fixture becomes FAIL.
   Preserve the transcript and classify that missing refusal coverage as
   UNVERIFIED. Independently wrong events, duplicates and deadline violations
   must still fail.
3. The macOS TLS handoff drops its reservation before rebinding.
   Keep an unconnected bound reservation and call `listen` on that same socket.
4. TLS qualification accepts a matching readiness record without a unique,
   complete attempt/readiness pair. Missing connection or deadline fields can
   also compare equal. Require present fields, exact cardinality and complete
   connection, address and deadline correlation before granting coverage.

Same-driver recovery calls now exist for both timeout drivers. They check exact
bidirectional frames and extra events. The production change remains four trace
sites and ten added lines; the reviewer found no concrete behavior or resource
leak defect there.

The findings were delivered to the running implementation task. A later
unreviewed candidate already existed; it receives no acceptance from this
review. The implementation must address remaining findings, exercise the real
Mac path through unavailable fixture qualification, preserve all earlier
reports and identify a complete candidate for fresh review. Native and runtime
gates remain unaccepted.
