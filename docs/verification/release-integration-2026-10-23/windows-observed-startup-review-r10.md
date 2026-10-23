# Windows discovery evidence review R10 — 2026-10-23

R10 found no concrete P1/P2 source or evidence defect. Both strict Windows
startup gates remain **FAIL/unverified**. Missing race observations do not
establish a product defect.

The review checked immutable evidence `1d46512` and exact source `e6a9125`.
It audited saved records and did not independently execute the native binary.
The [native report](windows-observed-startup.md) and [full review record](windows-observed-startup-review-r10.json)
retain the exact scope and hashes.

| Run | Literal result | Missing required witness |
| --- | --- | --- |
| Startup 1 | Exit 1; 105/106 | Limited over limited |
| Startup 2 | Exit 1; 113/115 | Standard over limited; limited over standard |
| Retained 800 | Exit 0; 19/19 | None in its separate scope |

Each missing replacement case exhausted eight attempts with eight Accepted and
zero Disconnected initial sends. Its consequent preservation, fresh-data,
removal, repeat and successor checks therefore remain unexercised in that run.
Witnesses from separate runs cannot close either gate. All behavior assertions
that ran in the positive startup runs passed.

Both runs observed the mandatory active standard-browse Drop through exactly
one completed normal Disconnected removal and a later public command barrier,
before any explicit stop. Witnessed failed standard replacements had fresh
repeats after the failed-send barrier and receiver drain.

The missing-startup-observer control failed before checks with zero matching
observations and recorded normal cleanup. The missing-cleanup-observer control
failed 95/124: 24 accepted-drop checks, two mandatory Drop checks and three
separate natural coverage checks failed. A limited-state gauge of one did not
make missing removal evidence pass.

The reviewer reconciled 863 archived source files: 862 byte-exact files and one
declared CMD CRLF conversion. It also matched 188 indexed private-file hashes,
121 raw-bundle contents, 17 copied outputs and six workflow copies. All five
invocations bind to the same frozen executable. Saved MSVC Rust 1.99.1 release
and strict Clippy results pass under session 0, BelowNormal and two jobs.

The saved records preserve prior fixtures, registry/compiler files, ACLs,
defaults and live sharing identity. Inspection and wrapper errors remain
recorded. Parent integration separately checks the reviewed evidence copies
and current runtime source; it does not count as a native rerun.

Ambiguous, Full and capacity rejection remain source-only in this Windows
round. Shutdown acknowledgement and post-exit inventories do not prove worker
disappearance while a helper stays alive on `e6a9125`. Earlier-source OS proof
does not transfer. Native input/clipboard, positive migration, slow DNS,
cache/expiry/timer/query/byte bounds, security/provenance and trusted
distribution gates remain open.
