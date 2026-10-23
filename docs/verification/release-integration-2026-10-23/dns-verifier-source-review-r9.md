# DNS verifier source review R9 — 2026-10-23

R9 found two P2 verifier defects in immutable `8333a48` against `c7f6989`.
No concrete product P1/P2 was found. The candidate remains unaccepted. The
[full record](dns-verifier-source-review-r9.json) includes exact source lines.
This was source review only; no runtime execution or private evidence audit ran.

1. Base queue and lifecycle requests use `localhost` while their initial
   fixtures own only IPv4. A request can reach an unowned IPv6 endpoint before
   the later peer check. Own every captured loopback address used by these
   hostname flows before issuing requests, and check stale accepts on every
   owned family. If ownership is incomplete, keep that hostname case unverified
   and continue safe independent cases.
2. Base frame exchange returns on a matching BACK frame and discards its local
   event vector. Extra failures, connections or frames can escape later checks.
   Retain complete transcripts, require exact identities and cardinality, and
   check for extra frame bytes during actively driven quiet polling.

R8's unqualified-timeout verdict, same-socket TLS handoff and complete trace
correlation findings are closed in source. The early abort is also removed,
but complete ownership remains open under R9-1. Same-driver timeout recovery
retains its caller, TLS and pin, with complete event/frame checks. Caller and
observation deadlines remain separate, with timing uncertainty preserved.

The parent instructed the Windows task to start no further `8333a48` probe
commands and finish owned-fixture cleanup. Its final execution and cleanup
record is still pending. A separate implementation follow-up now owns the two
repairs. Earlier source and evidence remain preserved. The independent audit
of saved Mac records remains separate from source acceptance.
