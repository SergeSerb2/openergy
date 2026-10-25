# Metadata terminal deadline repair: independent source/data review

Result: **one P2 driver finding (D3); no P1 found within this scope**.
The repaired reader is **source CLEAR with limitations**. Prior findings R1,
D1 and D2 are addressed in source within their stated scope. The current driver
still permits a false pass in its nonzero-child case. This report gives no
approval, executable receipt, or runtime acceptance.

Review base: `c884126196f0215094896f29649af4b895befd72`.
Repair: `97cc8f9930d0548c2d0c4d6b1a3ec56a18e14283`.
Original reader: `78759877392af3e41bb4d51fb5f40470c33ff6f9`, imported at
`f9ea16761c0333ac8c965d2aee441e508a8b390d`.
The full prior review is `f9aaf96ba4ef2b48c44367edba9bc6c7242e0376`, imported
at `626e2bc2a2aa29fbc9952c637388896b565523d5`. Its complete report and bindings
were read as data. No candidate source, driver, supplied helper, or native parser
was imported or executed in this review. No repair was applied.

## P2 D3: Reject a reader timeout in the nonzero-child case

Location: `docs/verification/metadata-reader-local-cli-2026-10-25/verify-cli.py:109-110`.
The common checks are at lines 91-106; case acceptance and flow success are at
lines 128-142.

The nonzero case requires reader exit 1, an observed child exit 1, INCOMPLETE,
empty raw output, and matching stream hashes. It does not check `first_error`.
Both an ordinary child failure and a reader capture timeout therefore satisfy
the predicate. The new external deadline checks do not distinguish them.

There is a concrete source path through the exact current reader. The real
`false` child can close both streams and exit 1 while the reader is delayed at
its final poll. If that poll resumes after the two-second work deadline, the
reader retains the actual child exit 1 and the new `before(deadline)` at reader
line 317 raises TimeoutError. Lines 366-367 keep that first capture error.
Finalization retains INCOMPLETE and the reader returns 1. If it returns before
the driver's four-second invocation deadline, the driver accepts the case:
both raw streams can be empty and all byte/hash checks can pass. The remaining
cases can then complete within 28 seconds and the driver can return 0 with
`LOCAL_CLI_CASES_PASSED` despite the reader timeout.

This is a static control-flow finding. No timing experiment, mock, changed
receipt, or simulation was run. The two retained historical nonzero receipts
have `first_error=null`; neither demonstrates this defect, and their recorded
results remain unchanged.

Fix the nonzero-case predicate so it requires ordinary nonzero-child capture,
including no reader error, complete EOF capture, no overflow or discarded bytes,
and no local kill. Keep the exact reader/child exit and raw-byte checks. In
particular, `first_error` must be null; a capture TimeoutError must fail this
case even when the expected reader exit is already 1. This finding concerns
the driver. The reader correctly reports the timeout on this path.

## Disposition of all prior findings

- **R1, terminal reader work deadline: addressed in source.** The pump checks
  its deadline after its final exit poll at reader line 317. The capture block
  records the completion observation and checks `work_end` again at lines
  364-365. Both checks remain inside the capture try block. A late result sets
  the first capture TimeoutError while preserving any actual child return code.
  The larger cleanup/finalization allocation cannot make that result COMPLETE.
- **D1, SIGPIPE settlement: addressed in source.** Driver lines 119-127 accept
  an actually collected SIGPIPE or SIGKILL with a recorded kill call. The common
  check still requires observed local exit. The cap case requires the exact
  128-byte prefix, 129 bytes read, one byte not preserved, OverflowError, empty
  stderr, and matching raw hashes. It does not invent SIGKILL after SIGPIPE.
  A kill call alone never supplies the collected exit. SIGPIPE can also win a
  race with a kill attempt; preserving that actual exit is correct.
- **D2, late outer driver result: addressed in source.** The driver sets one
  absolute 28-second flow deadline. Each case uses the earlier of that deadline
  and its reader-declared total plus an explicit one-second interpreter startup
  allowance. Checks follow the wait, evidence reads and validation, precede
  flow success, and gate CLI return after writing the result. A result file is
  insufficient without actual driver exit 0 and the external exit-time check.
  D3 above is a separate missing failure predicate inside that outer envelope.

These checks reject observed late results. They do not interrupt a stalled OS
call or prove the actual process-exit time. A file may retain an earlier pass
status if its final write returns late. The CLI return gate and the independent
external observation remain required; no saved receipt alone is acceptance.

## Reader, query, limits and privacy

The repaired reader materializes byte-for-byte from the bound original reader
and `reader-deadline.patch`: two hunks, three added lines, zero removed lines.
This comparison reconstructs patch bytes as data and does not apply or run the
patch. All other reader source is unchanged. The 87-line PowerShell query is
byte-identical to the original package.

Reader SHA-256:
`8f55fbb7c930989f0ad7327536eb61a1330eb741c108992784f37b54af68086c`.
Query SHA-256:
`3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`.
Current driver SHA-256:
`5f174d77b138d1b3b9a27d6cd6810d24cfb87c32c1f70dc079091d1d41c4bd27`.

The source retains bounded independent stdout/stderr capture and the aggregate
cap. Reads use at most 4,096 bytes. A full cap admits only the one-byte overflow
sentinel, records it as not preserved, and keeps failure. Partial writes update
the saved prefix count/hash. A gap closes pipes before further capture. Exit
collection remains separate after drain failure, so overflow cannot replace
an actual local exit with a claimed kill. Unobserved local exit remains null.
Directory descriptor closure and the prior separate settlement poll remain.

Preparation persists a fixed 65,536-byte INCOMPLETE receipt before spawn. Raw
files remain private, exclusive mode-0600 files beneath a caller-owned private
directory. Receipts omit argv and exception text. Raw data remains private.
Finalization and actual CLI exit still gate success. Disk faults, close faults,
host crashes and same-user path races remain limits on the saved evidence.

Metadata mode still checks the exact query and fixed SSH argv, six ordered file
records, lengths, version fields, full SHA-256 and runtimeconfig text, with
begin/complete records and empty stderr. The query retains the same six fixed
installed paths, streaming file hashes, file/aggregate/read/output caps and
cooperative deadline checks. No retry, recovery, default, remote cleanup or
qualification behavior changed. A null effective limit stays UNVERIFIED; policy
ceilings do not establish installed limits or route coverage.

Hard OS preemption remains false and an external execution envelope remains
required. Installed files can change during collection; the query is not an
atomic snapshot or provenance proof. Native PowerShell syntax and behavior
were not parsed or exercised. Remote exit/current state stays UNVERIFIED and
remote cleanup stays false for every local outcome.

## Data audit and retained observations

`bindings.json` binds 29 public scope files: the ten original reader package
files, seven deadline-repair files, five current driver/evidence files and seven
prior-review files. Each matches its immutable Git blob, the review base, this
checkout and the parent worktree at inspection. The current reader/driver scope
is unchanged between `97cc8f9` and the review base. The complete full-index binary
repair diff from `626e2bc` is retained as inert `.patch.txt` data. All four scope
checksum lists and the eight original manifest entries match.

The staged whitespace check reports four lines in the verbatim diff archive:
blank context markers and a nested patch marker. Those bytes are retained to
preserve the exact diff. The report, JSON bindings and checksum list have no
whitespace findings.

The 179 historical bindings retain the exact Git blob identities recorded by
the prior review, at its original parent, candidate, imported base and this
review base. Their contents were not reread or reconstructed. The prior review
and bindings are byte-exact, so their earlier full audit is retained with its
original limits. Thirty bound public input files were rehashed. The two old
private failure/command files were not reopened; their prior public hash
bindings remain preserved data.

All 50 files in the two explicitly allowed private local-flow roots match the
public index's lengths, modes and SHA-256 values: 25 files per flow, 606,626 bytes
combined. Source copies, exact command-byte receipt bindings, raw stream
counts/hashes and accounting, receipt hashes where the driver recorded them,
actual-exit records and public result summaries agree. No raw path or argv is
copied into this review; the bindings repeat public relative index labels only.

The first flow remains FAILED/INCOMPLETE with actual driver exit 1. Its final
cap case collected SIGPIPE without a kill and failed the old driver predicate.
The corrected `02ca76e` flow records four accepted cases and actual driver exit
0. Its collected child exits are 0, 1, -9 and -13. Both flows used reader
`e7fb7a8fde0f63246796d5209f85571d2de0418ab96536797101561bec39b380`;
their driver hashes are respectively
`8090e3dce24bbaf5b557f2ee219ab08ba20ba71477ad71f4583390b266622915`
and `2013c4939c8191480634277efbb028d70cf35b7fcc28a03c7a12371c7fd4bece`.
The public results/index remain byte-identical to publication commit
`0175b076a04c0b79181b8f070e4c9b53d7eb0418`. These are retained historical
observations, not executions or acceptance of the current deadline revision.

## Audit limits and unchanged gates

No new test, unit test, simulation, mock, fixture, helper, native parser, runtime,
network/remote action, OS/process/port query, credential read, private write,
build, signing, product launch, child agent, integration, push or publication
was performed. Only inert source/data reads, file/JSON/hash/Git/patch-data checks
and writes in this one new review folder were used. The report commit disables
hooks and signing. No TypeScript changed; compiler/build checks are outside the
explicit source/data-only scope.

Current reader and driver execution remains **NOT RUN / UNVERIFIED**. There is
no new terminal-race timing evidence, nonempty-stderr coverage, competing-cap
coverage, disk/preparation/close-fault evidence, or Windows parser/runtime proof.
This review does not convert the saved local flow into any of those claims.

The old metadata failure remains TIMED_OUT_UNVERIFIED. Original Windows
FAILED/UNVERIFIED, historical preservation UNVERIFIED, installed versions,
effective limits, native PowerShell, runtime and native stop all stay
UNVERIFIED. Zero native daemon.stop requests and all old results remain unchanged.
