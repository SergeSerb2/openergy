# Independent source/data review of the context tuple repair

2026-10-25, America/Vancouver. **SOURCE/DATA DELTA CLEAR: no new P1/P2.**
Prior finding **P2-1 is CLOSED IN SOURCE** for candidate
`2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`. Runtime remains **UNVERIFIED**.
This report grants no approval, executable receipt, or runtime acceptance.
It adds no human approval gate.

Reviewer checkout: `/Users/serge/.t3/worktrees/openergy/t3-4d328640`, initially
clean at the candidate. Only this new review folder is owned. The candidate's
parent is `32810df980ef1c18f4b6384f40d5235058359511`. Its entire delta adds
the 11 files in `ordinary-combined-context-tuple-repair-2026-10-25`.

**Prior finding and response.** The full review at
`c138853c78227e2e8ed8251cfd612ba3d3f5f678`, in
`../ordinary-combined-recorder-diagnostic-review-2026-10-25-t3-392ae27f/REVIEW.md`,
found that the composition's two-item coverage tuple caused argument binding
to fail before the first helper or bind. The parent acknowledged the omission.
The repair restores exactly:

`coverage=(self.runtime, 'main' if label == 'pre' else 'postflight', self.spec)`

The corrected controller's line 261 adds only `, self.spec` (11 bytes).
Strict in-memory reconstruction of `restore-context.patch` gives the complete
corrected controller: one removed line and one added line. The complete old
and new ASTs differ only at this keyword. The new keyword, including `self.spec`,
is AST-exact to the actual controller at priority commit
`43446949beac4dd445dfce980da9cd7f9a869ceb`; this comparison did not use a
README claim or a normalized tuple that could hide the missing value.

`common.command` still calls `CommandCoverage(*coverage, argv, deadline)`
(`common.py.txt:268-272`). The unchanged constructor requires
`runtime, actor, spec, argv, deadline` (`coverage.py.txt:34`). Three tuple
values plus the two explicit arguments now satisfy that contract. All five
recorded `command` call sites have three-item tuples. This closes the static
argument-count defect for both phase labels. It does not prove helper success.

**Full prior review reused with its limits.** I read the complete prior report
and checked its report, bindings and checksum bytes against its original
commit, imported commit `32810df`, this checkout and the parent checkout.
The prior review folder tree is `006edd3155812f9dca40e6a3775710674f9f0ac4`.
The priority, standalone diagnostic and composition package trees still equal
their original/imported trees. None of the prior bindings' 254 records for
195 distinct paths changed between the composition and this candidate.
The old archive was not reopened or audited again.

The prior review covered the full priority acquisition/control/helper changes,
the standalone diagnostic and their composition. Its other source conclusions
and proof limits carry forward through exact unchanged bytes. These include
R9 `e34401c`, M1 admission `02c9b17`, and the protected R4/R6/R8 repairs:
identity and wrapper authority, uncertainty handoff, actual exit ledgers,
app-first cleanup, fresh single-PID signal checks, TERM/KILL grace and settlement,
fallback/teardown, fixed deadlines and shared evidence budgets. The new tuple
does not change those bodies, product source, launch vectors, predicates,
delays, hidden switches, retry/reuse rules, ports, deadlines, or reserves.
The standalone review `90a221b` retains its original bounded source verdict.

**Independent data checks.** All checks below passed without source execution:

- All 11 repair files match candidate Git blobs and parent checkout bytes;
  all JSON parses and all 10 checksum entries match the exact file set.
- All ten materialization sources resolve to the declared paths with unique
  destinations. Their hashes, lengths and line counts match source-checks.
  All ten texts parse as AST data. The nine base module hashes match the prior
  review and immutable composition. The eight other priority modules and the
  standalone diagnostic module also match their original commits.
- The corrected controller differs from priority only in `Controller.port`
  and the declared diagnostic imports. Its full delta from composition is
  the single argument above. No runtime modules were materialized.
- Copied limits, coverage policy and disabled operation template are byte-exact.
  The extra build-source binding matches its immutable Git input and hash.
  The future plan only adds composition/prior-review inputs and updates its
  pending-review label. It and every step retain `execution=false`;
  `new_human_approval_gate=false` and the artifact receipt remains null.
- Diagnostic caps remain exact: 0.5-second query within the old port deadline,
  0.05-second publication share, 262144 output bytes, 64 selected rows,
  1024 bytes per row, 4096 retained stderr bytes and 98304 encoded record bytes.
  The failure-only diagnostic still cannot grant acceptance or change cause.

Old controller SHA-256:
`25348029134e10e7d587a5018d081cb3c7bfbe2025e257a0dd05beff6ba15e79`.
Corrected controller SHA-256:
`08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4`.
Repair package tree: `020ce328cb56828a601c54d50c2f81940a063e63`.
The full-index binary candidate diff is 63785 bytes, SHA-256
`68d0eedb0ad9074d1e0f24c738352d2c3156d8d616546da956103512a89676bf`.
All module/file hashes and check bindings are in [bindings.json](bindings.json).
SHA256SUMS binds this report and bindings; the review commit binds all three.

The authoritative parent advanced during review from the candidate to separate
metadata-evidence commit `3c52f5320c7ad8f434f06fadbbd4520565dd0b9b` and was clean
at the recorded check. All reviewed file bytes still match this fixed candidate.
Earlier metadata-reader work in the wider ancestry is outside this review.
Bindings retain two initial reader `FAILED_ASSERTION` results: an overbroad
additions-only history assumption and an assumption that the live parent HEAD
would stay fixed. Correcting those review assumptions did not relax any
candidate source, hash, policy, or acceptance check.

**All previous failure and proof limits remain.** Saved M1 remains
**FAILED_NOT_ACCEPTED**: app25/25, actual app exit0, controller exit1, first
strict post-bind65431 **EADDRINUSE FAILED**, cause **UNKNOWN**. Scoped known-
identity cleanup remains closed; it does not fill the second bind, original
archival, bundle-after or final outcome gates. Priority coverage remains
incomplete. No new full flow ran. Runtime, ultra-short stage/helper observation,
continuous NI15, target-M1 diagnostic flags/layout/visibility and the port cause
remain UNVERIFIED or UNKNOWN as recorded. All earlier ordinary/Windows FAILED
and UNVERIFIED outcomes remain unchanged. Whole-app/native UI/input, Windows,
updates, release, performance, universal containment and host restoration stay
UNVERIFIED; expired Sparkle scope stays closed.

Only standard-library file/Git/JSON/hash/patch-data and AST-data checks ran.
AST inspection included literal constant data. No supplied code/module/checker
or AST was executed or imported. No
tests/mocks, helper/native parser, runtime, remote/network or live OS/process/
port query, private write, fixture/build/sign/product/credential action, or
child agent was used. No TypeScript changed. Hooks and commit signing are
disabled for the review commit.
