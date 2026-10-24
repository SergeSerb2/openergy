# Independent review of the failed isolated package attempt

Date: 2026-10-23, America/Vancouver. Review worktree: `t3-60b539d2`,
branch `t3/audit-isolated-package-failure`.
Review started at `3183a9ed0de957dd471c6e1e8a76d1331f190d22`.

**No new concrete P1/P2 finding. The package attempt remains FAILED.** The
known insufficient preflight remains an open verification failure. This review
does not close the original P2 cache-preservation breach.

## Exact inputs and review boundary

Evidence commit: `3dcf8b2f7ccb1fa91d5fce43cdc4fd917c33dd03`.
Evidence directory: `docs/verification/release-priority-isolated-2026-10-23-t3-8c51bae6`.
Paths beginning with `raw/` below refer to that directory at this exact commit.
Line numbers refer to decoded text, including text stored as gzip.

Build/source base: `b713d7fbe480a1a3ecadd737ae5de58256169c66`.
Production source: `d3b30e1719dae5c885ded57cc7523b92d9686adc`.
Original failed preservation evidence: `d2ab17d4e053437d9f7568b0f6dbf96727bfd6c9`,
under `docs/verification/release-priority-2026-10-23`.

I read these inputs from Git objects with `git show`/`git cat-file`; I did not
check out either target commit. I also read and hash-checked the exact private
original files named in the new 64-entry ledger and the prior 25-entry ledger.
These were file reads only. All earlier records remain unchanged.

The review covered the controller, policy, observation hook, both saved setup
controllers, both collectors, first setup/audit errors, preflights, actual
failure streams, traces, export ledger, private originals, inventories, source
and dependency bindings, process samples, cleanup, port redactions, postflight,
and reported scope. Read-only calculations checked their consistency. No saved
controller, project CLI/helper, compiler, sandbox, daemon, or application ran.
There was no API/network/port/process inspection or mutation, credential access,
desktop interaction, test or mock creation, delegation, signing, or publication.
The separate corrected Swift recipe and all three provider-stopped tasks were
left alone. This is source and saved-evidence review, not independent runtime
reproduction or release clearance.

## Known preflight failure, distinct from new findings

`raw/controller.py:250-279` checked policy parsing, inherited NI15, and a private
Unix-socket setup exchange. It did not evaluate an actual Swift manifest under
the outer policy. Thus `preflight-complete.json` did not establish that the
native package could pass manifest evaluation.

The actual package stderr records denied Darwin temporary-directory
`xcrun_db` cache writes and `sandbox-exec: sandbox_apply: Operation not permitted`.
Rust completed its release build in a reported 44.40 seconds, with the retained
nonfatal linker/cache and `rust-objcopy` diagnostics. The Swift package then
failed during manifest evaluation, before whole-app compilation. This agrees
with the stated preflight gap in `raw/failure-limits.json.gz`, the README, and
the actual streams. It is a known verification setup failure; this command
failure does not establish a new product defect. No corrected recipe is
qualified by this review.

The earlier setup errors also remain visible. The first controller failed on
the read-only private copied Git alternates file. The next policy failed its
parser check because the address used numeric loopback instead of `localhost`.
The saved controller/policy differences match those limited setup repairs and
the later added observations, guards, and checks. Both setup failures preceded
the package call. The first source-binding collector later failed on the clean
Windows command file's required CRLF conversion. Its replacement adds only
that explicit conversion and records both hashes. The first collector and
error remain retained. The controller's final success assertion failed at
line 486 after it had saved the failed package and postflight records; that
assertion is not presented as a package success or a second attempt.

## Failure and stages not reached

The package result is exit **1**, **47.33682537500863 seconds**, `timeout: false`.
Its stdout/stderr hashes match the decoded saved streams and the current
retained originals. `raw/package-start.json.gz`, the plan, the trace, and the
result agree on the command and root process identity.

`raw/controller.py:359-408` has one package `Popen` site and no retry loop. It
creates `package-once.guard` with exclusive creation before that call. The
guard's retained bytes match its export. The preparation/preflight entry points
also refuse an existing guard. The outer watchdog is 900 seconds; process
snapshot commands have a five-second timeout and the final wait has a
ten-second timeout. This saved attempt ended normally before the watchdog.
The guard was not reset in the reviewed record.

The trace contains one `release.py package` observer start. Other observer
starts are `sparkle.py signing-policy`, `sparkle.py config`, and `release.py
version`; these do not constitute another package call. The observation hook
uses `sys.setprofile` at `raw/observe/sitecustomize.py:35-77`. It records real
call/return state. It does not assign replacement functions or change returned
results. The execution hashes match the exact controller, hook, policy, plan,
and preflight exports/private originals. The failed bundle command raises from
`subprocess.run(check=True)`, so its profile event has no completed return
object; the actual package result and streams retain the failure status.

The source order supports the reported stopping point:
`apps/macos/scripts/bundle.sh:48-60` builds the daemon before the Swift app and
stops on the failing Swift build. Assembly and signing follow at lines 62-96.
At the source base, `tools/release/release.py:366-373` reaches staged smoke only
after successful bundling. Archive generation, extraction, extracted smoke,
signature verification, and final artifact copy occur later.

`raw/full-smoke.json.gz` contains no smoke returns, smoke commands, or daemon
starts. Its only cleanup record is the package temporary directory.
`raw/artifacts.json.gz` is empty and `raw/artifact-bindings.json.gz` is `{}`.
The saved failure limits bind a compiled arm64 daemon of 4,492,776 bytes, SHA-256
`580370618c9a041f5b5860f5898738be519e022c0b7c1eb4567ce7b2b9c97d0b`,
as unexecuted. There is no accepted app/archive or new archive/source binding.
Whole-app compilation, bundle assembly, signing, ZIP roundtrip, staged and
extracted virtual IPC flows, and artifact acceptance remain **not reached**.

## Isolation, source, and preservation

The controller seeds only `artifacts/sparkle/Sparkle`, `checkouts/Sparkle`, and
`repositories/Sparkle-09d89c53` into new Swift state. Its Cargo home receives
the registry only; the Cargo target starts empty. It does not copy the old
Swift output/build databases, module cache, or Cargo outputs. HOME, temporary,
cache, config, scratch, target, and output paths are private. The environment
selects Rust 1.99.1, jobs 2, offline Cargo, native arm64, and deployment target
15.0. The source template also sets 15.0. Signing-intent secrets/identities are
absent from the explicit environment; package source selects ordinary ad hoc
intent, but signing was not reached.

The generated workspace state uses the committed Sparkle 2.10.0 pin
`eef1a539a373c1f1a320624b1130fc5de7b2e100` and the declared artifact checksum.
The exact seed-copy inventories match the corresponding old-cache inventory
subtrees. Only the private checkout's `.git/config` and Git alternates file
change before the build baseline, consistent with redirecting the origin and
object store to the private repository. Before/after private seed inventories
then match exactly. The saved pinned revision and clean source status agree
with the dependency checks. Private `.git/index` byte churn is explicitly
permitted, but no such difference occurred in this attempt.

The saved policy denies filesystem writes by default and denies network
operations, with declared exceptions for the private root, `/dev/null`, the
package/smoke temporary-path patterns, and the selected loopback/Unix-socket
fixtures. It does not deny host reads. The rejected Darwin cache writes remain
visible. This is a policy/source assessment, not an independent sandbox test
or a complete filesystem/network activity trace.

I recomputed these saved-inventory comparisons:

- All 1,309 old Swift and 1,159 old Cargo byte/link entries match before/after
  and the prior after-recovery inventory.
- All 1,725 Swift and 1,378 Cargo selected metadata entries, including roots
  and directories, match this attempt's baseline. Fields are mode, kind, uid,
  gid, link count, flags, nanosecond modification time, and symlink target.
  ACLs, extended attributes, access/change times, and old prebreach metadata
  equality remain unproved. This does not establish full host restoration.
- The complete tracked non-verification source set has exactly 714 inputs.
  All before/after hashes agree. I independently checked every source binding
  against the production Git blob. There are 713 identity mappings and one
  declared LF-to-CRLF mapping for `tools/windows/openergy-run.cmd`, consistent
  with `.gitattributes`. No source binding is missing or mismatched.
- Production source to build base changes only verification records. Build
  base to evidence commit adds only the 67 files in this isolated evidence
  directory. The prior evidence folder has no change across that latter diff.
- All 25 prior gzip exports and current private originals match their original
  ledger sizes/hashes. Their before/after ledger and complete folder inventories
  match. The complete folder inventory also matches the pinned Git objects.
  Original raw blobs are unchanged from `d2ab17d4e053437d9f7568b0f6dbf96727bfd6c9`.

The original run still has exactly **114 removed Swift byte/link entries**,
with no added/changed Swift entries and no Cargo difference. Its command exit
0 remains separate from its overall FAILED preservation result. Later byte/link
recovery and this attempt's new metadata baseline do not erase that original
failure or recover unknown original metadata.

## Priority, cleanup, export privacy, and limits

Recalculation of `raw/process-samples.json.gz` gives **621 samples**, **3,330
observations**, and **100 observed owned PIDs**. Both recorded effective and
`ps` nice values are always **15**. All recorded groups are 9624. PID/start-time/
group identities and the parent chains support attribution in the saved
samples. The source launcher sets and verifies absolute NI15 before `execvp`;
it does not add a relative nice increment on nested calls.

The samples include 55 Rust compiler PIDs, 7 `cc` PIDs, 16 clang PIDs, and one
`ld` PID. They also include Swift driver processes, but no Swift frontend,
codesign, or packaged daemon. The coverage summary matches the raw counts.
Coverage spans 0.060517290999996476 to 47.30142820799665 seconds; the maximum
sample gap is 0.10476466700492892 seconds. This proves the recorded observations
only. It does not prove priority or ownership for missed short processes or
all intervals.

Cleanup source checks attributed PID/start/group identities before individual
signals. Its retained result has no actions and no remaining sampled identity;
elapsed time is 0.0003872499946737662 seconds against the stated 60-second
budget. The saved postflight reports the owned package temporary path absent.
Both port records show all ten first exclusive binds of 65420-65429 passed.
The controller holds each first bind, uses no reuse option, saves preceding
state, and has no port retry. No new bind or live process query ran in this
review. Incomplete `lsof` output and sampling limits preclude claims of complete
host-state restoration or universal descendant cleanup.

I verified all **64** ledger exports, their payloads, and retained private
originals by size and SHA-256. There are no duplicate, missing, or unledgered
exports; README, ledger, and export-audit are the three declared additional
files. The 62 exact records preserve original bytes. Every gzip header uses
mtime 0. For the two port records, I independently reproduced the declared
JSON transformation byte for byte: only `lsof_before.stderr` changes. The
unfiltered private originals remain intact and hash-bound. The unrelated full
mount path and original warning text do not appear elsewhere in the 67-file
export set, including decoded gzip. Owned evidence/source/toolchain paths are
intentionally present. Review of the contents and a limited credential-format
scan found no exposed credential payload; this is not a universal secret scan
or an audit of unrelated host files.

The known preflight failure remains visible, and all later acceptance claims
remain withheld. No additional concrete P1/P2 issue was found within this exact
source and saved-evidence scope.
