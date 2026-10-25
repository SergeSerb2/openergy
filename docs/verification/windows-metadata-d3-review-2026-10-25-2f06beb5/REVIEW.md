# Metadata driver D3 repair: independent source/data review

Result: **D3 addressed in source; one remaining P2 (D4); no P1 found**.
This is a bounded source/data verdict. It gives no approval, executable receipt,
or runtime acceptance. No repair was applied.

Review base: `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`.
D3 repair: `93cd2e6634535d3d22464eaa9eb42d2ccf2ab3e9`.
The full prior reports at `ef9a635feebc4e03b36b68392c60165457ffe478`
and `f9aaf96ba4ef2b48c44367edba9bc6c7242e0376` were read and reused
with their proof limits. The reader's prior SOURCE CLEAR with limitations is
retained; this finding concerns driver acceptance of a failed reader outcome.

## P2 D4: Nonzero capture can accept a terminal checkpoint timeout

Driver: `docs/verification/metadata-reader-local-cli-2026-10-25/verify-cli.py`
lines 71-72, 99-114 and 132-146. Reader:
`docs/verification/windows-metadata-terminal-deadline-repair-2026-10-25/metadata_reader.py.txt`
lines 244-261 and 436-454.

The strengthened predicate rejects a timeout present in the saved receipt.
It still cannot distinguish ordinary child exit 1 from a timeout that occurs
after the final receipt bytes were written. A concrete path in the exact source is:

1. `false` exits 1 within the work deadline. Both streams reach EOF, remain empty,
   and have no overflow or discarded bytes. No kill or reader error occurs.
2. Finalization writes an INCOMPLETE receipt with `first_error=null` and
   `secondary_error_count=0`. Its final `checkpoint()` writes all receipt bytes
   before the reader's three-second absolute deadline.
3. The receipt fsync at reader line 260, or scheduling around it, returns after
   that deadline. Line 261 raises TimeoutError. The finalize handler records the
   error in memory. Its recovery checkpoint immediately fails the deadline check
   at line 245, so the already-written receipt keeps the earlier null error.
4. The reader correctly returns 1. If it exits before the driver's four-second
   invocation deadline, the driver accepts its actual exit 1, actual child exit 1,
   saved null error, both EOFs and exact empty-stream bindings. The remaining
   cases can finish within 28 seconds and produce `LOCAL_CLI_CASES_PASSED`.

This path needs no receipt alteration, invented exit, disk corruption or remote
action. It is a static control-flow finding, not a reproduced timing result.
The receipt's elapsed value and `deadline_observation_before_final_write` can
both predate the timeout; checking that Boolean alone does not close this path.

Fix: make normal nonzero capture distinguishable from failure after the last
saved checkpoint. For this timeout path, bind the reader's absolute deadline and
compare the independently observed reader exit against it. Keep the separate
startup/outer envelope, actual exits and all D3 byte/error checks. An unobservable
terminal result must stay UNVERIFIED or fail the case. Expected reader exit 1
plus an earlier receipt is insufficient. Broader storage/close faults retain
the prior report's limits; this finding specifically concerns terminal timeout.

## D3 and inherited findings

- **D3 capture timeout: addressed in source.** Lines 110-114 require null first
  error, zero secondary errors, no kill, both EOFs, no overflow and no discarded
  bytes. Common checks still require reader exit 1, observed child exit 1 and
  exact empty raw files with matching hashes. Both named stream entries must
  exist at lines 103-106, so an empty mapping cannot bypass `all(...)`.
  The bound reader keeps capture errors sticky. D4 occurs after serialization.
- **R1: prior source response preserved.** Reader bytes match the reviewed
  deadline repair exactly. Its two terminal work checks remain unchanged.
- **D1: prior source response preserved.** The exact cap prefix/accounting and
  actual collected SIGPIPE or SIGKILL settlement checks remain unchanged.
- **D2: prior source response preserved within its outer-envelope scope.**
  Checks after wait/read/validation, before flow success and at CLI return remain
  unchanged. D4 can occur inside that larger envelope. Actual driver exit 0 and
  independent external exit-time evidence remain required for any future run.

## Bindings and retained limits

The sole driver source delta from `97cc8f9` is the D3 predicate replacement.
Repair `93cd2e6` changes only that driver, its README and its checksum list.
All 29 prior scope bindings match their original immutable blobs. At this base,
26 are unchanged and those three are the only changed files. The five files
of the full `ef9a635` review are byte-identical. The reviewed files also match
the parent worktree. All five relevant checksum lists match. Bindings record
the exact full-index binary diff hash and current public file identities.

Current reader SHA-256:
`8f55fbb7c930989f0ad7327536eb61a1330eb741c108992784f37b54af68086c`.
Query SHA-256:
`3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`.
Current driver SHA-256:
`8f4f95f0178e0f249fa3b44804d23d10090822c6d30c3ca92e7fb2e1930a642f`.

The 179 historical Git blob identities are unchanged; their contents were not
reread. No private file was opened. The unchanged public results/index and full
prior review retain the original 50-file audit, with no fresh private-byte claim.
The initial SIGPIPE driver flow stays FAILED/INCOMPLETE with actual exit 1.
The later saved four-case pass belongs to its earlier exact source hashes.
Both historical nonzero summaries have `first_error=null`; neither is evidence
for the current driver or D4. No unpublished parent run is adopted here.

No supplied source/module/AST, helper or native parser was imported or executed.
No tests, mocks, fixtures, runtime/remote/network actions, OS/process/port queries,
private writes, builds, signing, product/credential actions or child agents ran.
Checks used only file/Git/JSON/hash/patch data. Writes are confined to this new
review folder; the report commit disables hooks and signing. No TypeScript
changed; execution and compiler checks are outside this explicit review scope.

Current-driver runtime remains NOT RUN / UNVERIFIED. Native PowerShell parsing,
Windows runtime/native stop, installed versions/effective limits and historical
preservation stay UNVERIFIED. The old metadata result stays TIMED_OUT_UNVERIFIED;
all original FAILED/UNVERIFIED outcomes and zero native daemon.stop requests are
preserved. Null effective limits and policy ceilings establish no qualification.
This review makes
no retry, recovery, default/config change, full NI15 or matched-performance claim.
