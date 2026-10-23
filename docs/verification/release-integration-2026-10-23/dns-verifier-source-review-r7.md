# DNS verifier source review R7 — 2026-10-23

R7 found two P2 verification defects in candidate `7a56d17`. It found no
concrete product P1/P2. The candidate is not accepted. This review used immutable
Git blobs and did not run a build or real flow. The [full record](dns-verifier-source-review-r7.json)
preserves its exact references and scope.

1. The probe binds each refusal address and immediately drops its listener.
   A later attempt can reach another process that acquired the port.
   `EXCLUSIVE_BIND=PASS` therefore proves only an earlier bind.
   Keep exclusively bound non-listening sockets throughout qualification and
   the relevant cases. Hand the TLS port to its owned listener only when needed.
2. Each new pending-timeout phase ends without a healthy connection and exact
   frame exchange on the same driver. A driver that emits the correct timeout
   but cannot reconnect could pass. Require subsequent healthy use of each
   affected driver, with checks for stale and duplicate events during recovery.

The two R5 timing predicates are repaired in source. The eight queue/frame
batches and existing lifecycle flow remain. The production change adds four
trace sites; the reviewer found no new Unix/Windows conditional-compilation
issue.

The findings were sent to the existing implementation task. Its current
reports remain preserved. Revised source needs a fresh independent review
before acceptance. Runtime deadlines, fallback coverage, same-driver recovery,
the actual `615c16e1` negative control and native Windows acceptance remain
unverified.
