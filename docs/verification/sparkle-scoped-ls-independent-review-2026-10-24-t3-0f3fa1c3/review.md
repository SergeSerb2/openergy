Independent source review of the frozen scoped LaunchServices repair

Reviewed on 2026-10-24, America/Vancouver. **No concrete actionable P1/P2 source
finding was found.** All five earlier P2 repairs remain present. This result
comes from independent source and saved-evidence review. The earlier R6 verdict
was context, not a premise for this result. This document does not clear a
concrete operation for approval, make the old operation runnable, or grant
execution or runtime acceptance. No executable review or approval receipt was
created.

The exact held commit is `a7492bce9b22af9339846b2011b5099f92dafe95`, imported as
`4d4e4f77aad3c010cbde9c31cb217d362d688dc7`. The reviewed folder is
`docs/verification/sparkle-scoped-ls-source-repair-2026-10-23-t3-2d3b20d4/`,
called `SOURCE` below. `C` means its `controller-DO-NOT-RUN.py.txt`; `H` means its
`launchservices-DO-NOT-RUN.py.txt`; `P` means its `operation-DO-NOT-RUN.json`.
All source line references below refer to these exact held files.

Exact independently recomputed SHA-256 identities:

- Controller: `2a25d273ebf5b79783de44b7d39c38e4916b7475a2ba4901b092691506826175`.
- Helper: `b4672e2c65d604ab793247083283ac140056923a3341ee331a2bdba893e03644`.
- Proposal: `589cee62e7a0446f19d55d4e29060e6770bcd86a16f5043ce69a3e8765911fff`.
- Freeze: `c48e635d57611b7a6231141c10de44f120e1d29c9d2f98b47bd9369b7ff45487`.

Read the full README, controller, helper, proposal data, API rationale,
responses, static-reader sources/results, freeze manifests, saved reader errors
and failure evidence. Read the complete copied R2, R4 and R6 reviews, all earlier
READMEs/responses and all controller/proposal patches. The earlier controllers
were read as complete byte and syntax data and reviewed through the complete
patch chain against the current controller. Duplicate inputs were checked for
byte equality. Opaque source-hash records were parsed and compared as data;
their product behavior was not reviewed or executed by this task.

**Bounded integrity checks.** An independently authored inline reader used only
standard file, Git-object, hash, strict JSON, AST and text-patch operations. It
was not copied from, imported from, or run through any supplied checker. Its
source-file inputs were capped at 2 MiB each, its Git batch input at 1 MiB and
its captured Git batch output at 64 MiB. No AST was evaluated. These checks
completed without a failed assertion:

- Read all 60 regular files, 1,217,403 bytes, with 50 distinct contents. All 60
  match the held commit. The folder tree object is
  `f1f5781322bcf1bfdb15ac631aa2681a568a9fbf` at the held commit, imported commit
  and review base. Git records mode 100644 throughout; no symlink was found.
- Recomputed the exact path sets and every size, mode and hash in the 59-entry
  current freeze, 35-entry copied cleared freeze and 16-entry earlier freeze.
  Each excludes only its own root manifest. The enclosing tree binds it.
- All 36 cleared-source files match `95c59a27be0f9c95ed207eac4180193a493bb5f2`.
  All 17 earlier repair files match `82cb25b6775d69fe1a312bdc1f26486aba0bf287`.
  The four direct original inputs match preparation `9490a0e` and R2 `48c41fb`.
  Full R4 and R6 copies match `cf8127d` and `6ad28c7` respectively.
- All 18 original failure-export files in this checkout match
  `1cc4d03113d8d78be84ad0fbc161610860455f35`; all five selected failure copies
  also match that commit. This is committed-evidence preservation, not current
  private artifact retention or live guard-state verification.
- Strictly parsed all 22 JSON files, rejecting duplicate keys and nonfinite
  values. Parsed all 10 Python/text source copies as AST data, including the
  supplied checkers. Parsing is not a behavior or Python 3.15 execution check.
- Recomputed and materialized all three new unified patches in memory against
  the exact inputs. All resulting bytes match. The four historical patches
  also recompute and materialize exactly. No patch was applied to source files.
- Controller AST comparison permits only the module warning text and
  `os_evidence` to differ from cleared source. All other 39 functions/classes
  and all remaining top-level nodes are equal. Existing job/listener capture
  statements and the four capture call sites are preserved.
- Exactly four prior proposal fields change: `controller`,
  `missing_prerequisites`, `proof_limits`, `readonly_pre_post_evidence`.
  Only `launchservices_capture` and `scoped_LS_correction` are added. Every
  other prior field, including the full limits object, is equal. All original
  limit values also remain equal through the earlier proposals.
- All 451 source-hash records are equal across the proposal generations. All
  902 Git-blob hash comparisons at preparation `9490a0e` and declared source
  root `ad280644a96905d9831825bb4f87d1388dbdca73` match. The six original
  evidence hashes and original evidence-manifest hash also match Git blobs.
  This does not revalidate compiled artifacts or import the later stop fix.
- Helper literal IDs and nine reporting roots exactly match the proposal and
  the existing fixture/cache roots. Controller and helper hash bindings match.
  Approval/runtime/runnable/reuse flags remain false as declared.
- Read the seven named local Apple SDK headers and two text linker stubs as
  primary source data. All nine whole-file sizes/hashes and all 38 frozen
  excerpts match `api/sdk-evidence.json`. No framework was loaded, symbol
  resolved, toolchain queried, or native API called.

**Prior finding disposition.** These are preservation checks of the actual
source mechanisms, not inferred runtime passes:

- R2 R1, missed old-daemon restart accounting: preserved at C:235-338,
  C:340-353, C:369-397, C:994-1003 and C:1035-1065. The same persistent ledger
  starts before launch and remains active during IPC and OS capture. Counts
  do not reset at readiness. PID/UID/start identifies one process across the
  permitted exec. The initial parent fixes the old role. A second child of
  that parent fails instead of using the candidate allowance. Old app and
  every recorded old child still require independent exit observations.
- R2 R2, waits beyond absolute deadlines: preserved at C:92-104, C:120-146,
  C:504-534, C:578-609, C:681-718 and C:740-899. IPC recomputes one exchange's
  remaining time before connect/send/each fragment; HTTP recomputes before
  each receive and header/body send. Independent recovery keeps the 180-second
  cutoff and 60-second reserve within 240, cancellation, bounded I/O and reader
  handling, and the separate terminal timer. The 10/15-second recovery delays
  remain. Native scheduling and completion time were not measured.
- R2 R3, failure cleanup quitting the old app: preserved at C:644-666 and
  C:1057-1067. The sole `normal_quit` call is C:1067 in main, after explicit
  NEW build/launch/app/child validation. It checks current identity and build/
  image hash and consumes the single quit allowance before native dispatch.
  Recovery has no normal-quit call. There are exactly two `launch_once` call
  sites, C:1012 and C:1028, for initial entry and updater respectively.
- R4-1, stale ENTRY replay after same-PID IMAGE exec: preserved at C:213-223,
  C:249-295, C:314-321 and C:369-397. The existing RLock spans full identity
  acquisition and immediate merge, including the parent read. The two raw-read
  sites are C:215 and C:282; the sole `observe` call is C:218. Bulk scans and
  stored ready rows do not replay snapshots. Forward exec still saves evidence;
  reverse or other executable transitions still fail.
- R4-2, late observed helper omitted from completed recovery: preserved at
  C:249-264, C:323-329, C:400-474 and C:810-891. Recorded scope is broader than
  permission to signal. Incomplete reads/overflow fail scope completeness.
  Cache-phase live observations latch a removal block. Each destructive step
  has a locked scope check and descriptor-relative no-follow traversal. Final
  fresh scope reconciliation follows main/reader quiescence and observer join,
  before result, `RECOVERY_DONE` and timer disarm at C:1084-1088. Recorded live
  or unconfirmed helpers force errors, failed cleanup and retained state.
  Unknown helpers gain no signal or deletion authority.

**Native API and publication review.** At C:537-552 the replacement uses one
direct child through unchanged `ro()` at each existing phase. That child uses
the same interpreter with `-I -S -B`. Both source texts require Python 3.15.
The same five-second absolute command deadline covers startup and both native
queries together. Timeout kills the direct reader and attempts bounded drain/
reap; it cannot become an absence result. There is no broad registry dump,
retry, fallback, registration or unregister call in the candidate capture path.

The declarations at H:38-85 match the inspected SDK's macOS LP64 ABI:
pointer results/arguments use `c_void_p`, CFIndex uses signed long, CFTypeID
uses unsigned long, Boolean uses unsigned byte, encoding uses UInt32 and
CFRange has two CFIndex fields passed by value. The LS error parameter is a
pointer to a CFError pointer. The framework install names match the SDK text
stubs. This establishes source consistency, not framework availability at runtime.

At H:88-159 the identifier, copied result array, returned error and copied path
string each have matching release paths. Array elements, URL strings/base,
error domain and exported OSStatus constant are borrowed and are not separately
released. The result array stays alive while its URLs are read. Runtime type
checks precede typed access. Only NULL plus an OSStatus-domain error code
`-10814` is classified absent; empty arrays remain distinct. Missing or other
errors, simultaneous array/error, conversion failure and excess counts fail.

At H:88-125 and H:179-216 arguments and the fixed output envelope are checked
before native queries. Counts must be 0 through 16 before URL indexing.
Strings must fit 4,096 UTF-16 units before conversion into a fixed 4,096-byte
buffer, with complete conversion and no embedded NUL. The declared envelope is
811,008 bytes, below the 1,048,576-byte publication cap. Final JSON size is
checked before it reaches the saved output pipe. No API-derived count or size
controls an unbounded Python allocation or result loop.

At H:102-125 each bounded original URL string is hashed. File-reference URLs
are classified before path conversion. Base/relative, nonlocal, noncanonical
and outside-root results publish no raw URL/path. A path is published only
after absolute canonical lexical and component-boundary root checks. No path
is resolved or used to grant ownership. At H:162-176 and H:185 the child saves
the JSON pipe and redirects its native stdout/stderr before framework loading.
Structured failures use fixed codes and bounded numeric error metadata, not
native descriptions or exception text. This addresses native diagnostic output
through those descriptors; it does not assert absence of OS logging or shared
service effects. Hashing also does not prevent guessing a known outside path.

**Closed failure and unresolved gates.** The original approved one-shot remains
failed and closed. The byte-exact saved traceback and result show exit 1 at
PREPRODUCT `main:972` -> `os_evidence(preflight):539` -> `ro:511`. The unchanged
five-second deadline supplied a timeout of `4.99583949999942` seconds. This was
the first `lsregister -dump` preflight capture, before target/daemon/updater/
installer startup, HTTP bind/request, IPC, relaunch or normal quit. Seven saved
preflight records and the control-flow stop support those zero operation
counts. They do not establish host-wide process absence or independent reader
PID/exit cleanup. No retry occurred in this review.

The frozen marker expired at **2026-10-24 00:56:25.164527 PDT**, before this
review. The unchanged preflight gate also needs 240 seconds remaining. The user
reports the `/tmp` checkpoint and several exact private fixture/raw artifacts
missing. No current per-path retention inventory was supplied or obtained here.
The 21:11 selected-file postflight is a historical observation; it cannot prove
retention now. The exact missing private artifacts, live guard state and any
viable prospective operation therefore remain unresolved. Nothing was probed,
recovered, renewed, moved, reset or rebound.

The 21:01:00 PDT approval and its R6 receipt cover only the closed invocation.
The 17:16:55 stopped-case approval is also inapplicable. Exact private artifact
availability, a separately reviewed viable concrete operation, its own explicit
user consent and any required separate review receipt are absent. This source
document supplies none of those execution gates. Retention alone could not
reopen the expired one-shot. Old evidence, nonempty raw directories and terminal
guards must remain intact. No retry, marker renewal, approval reuse, new fixture,
new key/signing, alternate port or environment repair follows from this result.

The API returns metadata and has no caller-set native allocation or duration
bound. The count/string/JSON caps bound subsequent processing and publication;
they do not prove native peak memory or five-second success. URL metadata cannot
prove actual exec, NSWorkspace relaunch delivery, registration timestamps,
complete nested-helper inventory, Gatekeeper/XProtect outcome, registration
deletion or full host restoration. Process samples can miss brief processes
and cannot exclude future kernel events after the final scan. Product log and
OS-output checks remain sampled or post-capture. These are explicit proof limits,
not actionable source defects under the stated contract.

Bundle.main and private environment at the protected load boundary, both
no-argument entries, real model/virtual Off daemon readiness, independent old
app/all-old-child exits, actual Sparkle replacement/relaunch, new readiness,
final quit, helper/cache cleanup, priorities and deadlines remain **UNVERIFIED**
at runtime. Runtime qualification is separate and was not authorized. Standard
visual/manual AppUpdater, native input/clipboard, Developer ID, notarization,
trusted release and publication acceptance also remain open. The later daemon
stop fix is not attributed to the frozen `ad28064` artifacts.

Only this new review document is owned by the task, in isolated checkout
`/Users/serge/.t3/worktrees/openergy/t3-0f3fa1c3`, branch
`t3/sparkle-ls-source-review`, initially clean at parent base
`133a44cbdfec90fec0ad9f34eab0923b588c454c`. No prior source, evidence or guard
was edited. Initial combined output was truncated and a `wc` read included
directory operands; bounded follow-up reads covered the omitted text. No
source-check failure was hidden. The quick memory lookup supplied only the
general rule to keep commit-specific source proof separate from runtime proof;
all task facts above were verified from the held data.

No supplied controller/helper/checker/module was imported, executed or evaluated.
No unit test, mock, simulation, compiler/build, fixture preparation, signing,
key/credential access, product/preflight/process/OS/port/job/log/registry query,
network access, desktop driving, remote command or delegation occurred. Local
SDK text reads were the only API prerequisite inspection. No installed app,
live sharing, production config/TLS/default IPC, TCC, login/service/task/account,
firewall, global DNS or toolchain default changed. No TCP24800/UDP5353 action,
Docker, push or publication occurred. Crafted mDNS, original provenance/security,
IPC/Synergy3 import-budget and isolated Swift-build gates remain stopped.
Source review is not runtime acceptance.
