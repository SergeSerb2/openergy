# Independent full R7 prepare-parent source review

2026-10-25, America/Vancouver. SOURCE/SAVED-DATA ONLY. execution=false.

**BOUNDED_SOURCE_CLEAR: 0 P1 / 0 P2 in this exact R7 prepare-parent source
packet. FP-P2-3 closes SOURCE_ONLY.** FP-P2-1 and FP-P2-2 remain OPEN. The old
attempt remains FAILED_UNVERIFIED. All runtime and operator gates remain held.
R7 cannot execute as-is with the known missing semantic requirements data.
This report supplies no execution authority, new approval gate or retry.

Candidate is `44a6f8dfc7b8cb9597caec7b2fa34e98677cb705`; import and opening HEAD
are `5a587faa701da00d6e16f2e5ff9f2f43f553caf5`. The reviewed folder is
`m1-prepare-parent-observed-layout-r7-2026-10-25-t3-00056e6d`.
Candidate, import and actual working packet agree entry by entry in bytes and
modes. Both commits add only its 18 files. Their packet subtree is
`b99294d93f99df0ce3449a1e8397ba59b7c52ed1`: 18 files / 229400 bytes.
All 16 manifest payload records and all 17 SHA256SUMS rows match.
Canonical packet-entry digest is
`912019b96a4023a34dda74534324d193622f92568e4f60128dbef5ee888b5545`.
Its encoding is the sorted relative path/mode/bytes/sha256 record list, compact
sorted-key ASCII JSON plus LF. This is this reader's encoding, not a substitute
for a differently shaped historical digest.

The complete [R7 source](../m1-prepare-parent-observed-layout-r7-2026-10-25-t3-00056e6d/prepare-parent.py.txt)
is 110497 bytes / 1943 lines, SHA256
`8fb377d4e9d0f1fc45a261bf26b8b2df06111fa780e8d3932d9fbc3029e4d67c`.
The exact 2114-byte, two-hunk patch has SHA256
`432e651d52ab33f650e13c61a094167ff066adcf137eb7d2f594c0b0b18948dd`.
Original R5 `02f94afbe3e1001b8d3e8fc4d79976475485f661` retains 109444 bytes,
SHA256 `b3fbd793a0709c7857e34efca667a081a0486accfd0f3ad13a8068484b7eb8d9`.
Full historical R6 `63b6a3b564d6a557f60e0e2b0e4a1943ffa2b19f` retains report
SHA256 `dc8e986c6a3e914b65a13686fc9f1a7e15702e8548e545075d6967f340541c4e`.
The latest failed-data audit is
`b90044c48727e7d0e96e685b1b6c1d08cff81111`, import
`e3611068e1e2ad3766e79eff87be6f38c7f4c531`, report SHA256
`604fec5c9c11b6ca34706d8752c8db5332b9c78163af8338aabf291fef485d7f`.
Those full reports were read and remain historical. Their full packet identities
and this review's original-commit public inputs are in [input-bindings.json](input-bindings.json).

**FP-P2-3: CLOSED_SOURCE_ONLY.** R7 lines 925–937 introduce exactly the literal
ROOT_PARENT_ONLY tuple, literal ROOT_OBSERVED_DATA map and revised
ROOT_INITIAL_NAMES expression. COMPLETED7 and MANIFEST_HASH remain exact full
parent provenance. The two excluded remote names are
`source-files-once.guard` and `source-files-parent-result.json`. They remain
external parent originals that require full actual parent inspection.

This distinction follows actual complete reader bodies. R7 inspect_remote
1063–1097, the SOURCE launcher's inspect_remote, and both materializers'
inspect_completed each read the same five remote originals. The publisher's
actual public receipt says remote_completed_files_authenticated=5. The SOURCE
parent_record puts all seven references in data; that does not publish its two
local originals to ROOT. R7 inspect_remote and inspect_frozen remain byte exact.
The existing full-seven external inspection duty in PARENT_DUTIES and bind_config
is unchanged. No private parent original was opened here.

The revised disjoint groups have 20+5+212+6+8+7+2 names, exactly 260. Applying
the saved diagnostic's two missing and two extra names to the old set produces
exactly this new set. The saved count 260 and total 1242294 bytes remain saved
observations. Scalar agreement does not prove a complete current body manifest.
The older R6 inference from seven-name declaration agreement is superseded only
for this remote-set provenance. The historical report itself is unchanged.

The two additions are passive, nonimportable observed DATA with exact policies:

- `coverage-policy.json`: 1864 bytes, SHA256
  `35516fa08202a4ccd26ece557daecc5bf4e1e8f8e690f18b59d5cc70c3d30d36`.
  The saved raw file equals the original frozen policy byte for byte.
- `:-`: 67 bytes, SHA256
  `d55daee618ec400cc662eb3fb480e13b7ea93ff24c00b69bd4b2e9c871d48c22`.
  Its exact content is the designated-requirement comment with CDHash
  `07f9138c67bb59ebdc8c04536fc0c9cb108b0fb0`, followed by ONE LF.
  It is untrusted failed evidence. It supplies no seven-object body proof.

Both raw files were reopened only in public failed packet
`6dea2532eeae763150d7efb6701632f8ebeb9163`. Its full 60-file packet and 59
checksum rows match. No creator trace is invented. No deletion, rename,
overwrite, guard reset, ROOT repair, alternate path, pattern admission or retry
is introduced or authorized.

The complete changed gate is R7 lines 1138–1182. It enumerates all entries and
requires full set equality. Each invocation then checks each observed DATA
leaf's current stamp against its own enumerated seen identity, calls existing
read_leaf with the exact size/hash, and checks the same seen identity again.
The call is inside the loop after complete set equality, so BOTH leaves must
be read on every successful gate. read_leaf retains bounded reads, exact length
and SHA256, nofollow open, regular/owner0600/nlink1 metadata, and stable open and
linked stamps. The enumeration supplies same-device enforcement and the stamp
rebinding connects that enforcement to the byte read. The full linked root
stamp is checked last. A changed identity or body fails admission.

All original owner0700 root, flat regular0600 leaves, nlink1, same-filesystem,
nofollow, complete name set, name/count/file/aggregate caps and stamp checks
remain. Only the ten prescribed pinned .py names are admitted. Directories,
packages, extensions, bytecode, cache entries and unknown siblings still fail.
This depends on external exclusive ROOT ownership through exit; it is not a
hostile same-user or continuous-layout proof.

main repeats the initial gate at 1820 before frozen reads, growth or helper.
import_common repeats it at 1505 after pin/cache checks and before module entry.
main repeats it at 1864 before primary_attempted. Later admission is exactly
264: initial 260 plus OUTPUTS[:4]. The publication map must contain exactly
those four, each already durable with a retained raw reference; each is read
and hashed. Future ledger, prepared and output leaves cannot pass this gate.

**Full source and contract coverage.** I read all 45 physical declarations,
including nested functions and decorators, every global, entry block and main
guard, the full new source and all 18 packet files. Long literal data were
decoded completely. [declaration-review.json](declaration-review.json) binds
every full declaration range and hash. Only closed_root_layout changes; all 44
other declarations are byte exact. All 41 original top-level assignments are
exact except ROOT_INITIAL_NAMES; exactly two literal assignments are added.
Both complete before/after transcripts agree with actual source slices.

The patch reader checks every hunk header, declared old/new position, context
byte, line prefix and hunk length with no fuzz. Full forward reconstruction
equals R7; full reverse reconstruction equals R5. Independently restoring only
the complete changed body and global block reproduces every R5 byte. Thus
imports, entry, guard, separators and all bytes outside those two blocks remain
exact. No supplied source was patched or executed to make these comparisons.

The full R6 report, PARENT-CONTRACT, SIGNAL-CONTRACT, LAYOUT-DERIVATION and
limitations were read. The full upstream operator-plan and receipt-contract,
closed artifact/run receipt schema, next-operation map, helper, preparer and
existing common writer paths were inspected as public text/data. The copied
R7 config schema/template, child-status schema, CLI contract and PARENT-CONTRACT
are exact R5 copies. Their old identities and relative references remain
historical and resolve at the original R5 folder. R7's explicit scope describes
the corrected set. No CLI, startup selector, configuration, schema or authority
change is hidden in the new source.

All 17 original pins match their complete actual public bytes. All ten module
lengths/hashes match the original source-checks and materialization. All 50
actual vectors match a complete literal expansion of the frozen recipe. All
212 raw length/hash/name rows match historical SOURCE validation and the full
53-label expansion, totaling 213334 bytes. The canonical raw-manifest SHA256
remains `2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee`.
SOURCE_RESULT remains 327813 bytes /
`7482a637159e0389c5cd4c505778919bb49354437bc9feb1cf008d9db395712d`.
This rechecks public reference data; it does not claim to reopen the other 180
raw bodies or requalify the target runtime. All source/ABI/build/artifact
identities, full eight-file bindings, limits and startup's ten keys remain exact.

The original actual caps are INITIAL_CAP=12517376, ENTRY_CAP=4096 and
RAW_CAP=1048576. FILE_CAP=8388608, TOTAL_CAP=33554432, CONSOLE_CAP=512,
STATUS_CAP=16384 and RECEIPT_CAP=262144 remain exact. INITIAL_CAP derives from
32MiB minus 16MiB product, 4MiB cleanup and 64KiB accounting reserves.
The recovery design R1 proposed tighter caps; R7 does not implement that proposal.
The 512-byte cap is console output, not the initial entry cap.

The original clock and budget paths remain intact: remote128/work126/reap126.5,
helper at most10, unchanged prepare120, and external local135=130+2+3.
The fresh launch sample follows alarm setup and must leave original120.5s before
the unchanged work end. No helper, writer or failure renews an end. Common
conversion samples local before shared, discards the gap plus1ms and rounds
down. No cross-host origin, startup2s bound or new enclosure proof is inferred.

collect retains real poll/wait exits, including negative codes, before the next
clock. Actual None plus its immediate clock must precede BOTH original ends.
Only overflow/timeout can reach the sole direct os.kill of proc.pid from the
retained unreaped handle. There is no intervening OS query, second waiter,
PID lookup or group authority. kill_syscall_returned means successful syscall
return only. ESRCH and other errors remain errors; no delivery, reap or EOF is
fabricated. The closed status schema matches the actual constructor and later
retained_bytes field, with signed integer/null exit domains. First failure,
bounded prefixes, drop/unknown facts, close paths and failure-before-writer
ordering remain intact. OS scheduling/stalls, delivery and actual reap remain
unverified. Separate frozen dependency signal branches retain their old limits.

The guard and bounded helper raw/status reservation precede the first write.
The SYSTEM helper must authenticate the complete 2006-entry/46504040-byte
private prefix, executable and ten modules, with actual0/reap/both EOF/noerrors,
before common import or any private-Python entry. Pins and the exact264 gate
repeat before the irreversible sole primary attempt. startup and the exact
helper/private argv remain unchanged. No INITIAL writer is reachable after
that boundary. Post-child publication needs a real reaped exit and the existing
valid prepare ledger/lock, then uses unchanged common.write_bytes with locked
precharge, temporary/final charging, the same reserves and original end.
There is no fallback ledger, writer, refund, reset or retry.

Prepared data still require actual0/reap/EOF/noerrors and complete stable new
source/tree/environment/limits bindings. The copied tree, signatures,
entitlements and native qualification remain later gates. Full receipt,
barriers, root close, one bounded nonblocking FIFO write and own OS exit remain
separate facts. Receipt text cannot prove its own later barriers or exit.
The external parent still owes real SSH/collector exits, complete retention,
D4 before the original A, causal enclosure and all operator predicates.

**Held requirements and composition gates.** I read the complete latest
failed-data audit and rechecked all 28 public requirements files. Each saved
command still has actual0/reap/EOF and the unsafe `--requirements :-` vector.
All seven stdout bodies are empty; stderr contains one Executable line each.
The primary public codesign manual was read as text. Its requirements display
argument names an output destination; a dash selects stdout. This is public
manual evidence, not qualification of a target executable. FP-P2-1's unreserved
native-write risk and FP-P2-2's missing seven-body semantic evidence remain OPEN.
The historical 53 statuses / 212 references / 50 raw comparisons stay intact.
The single observed `:-` comment cannot fill six other missing bodies or all
seven semantic checks. Inherited COPY vectors cannot run as-is or outside R9.

The new requirements supplement, basis, COPYv4 and corrected preparation
semantic gate remain pending. No source clearance is extended to local-path
component `b94f1c7cba63a15f86aa6dc2f7301158e6bf5b1d`: its new observer/assessor
paths have no active clearance, and its future driver binding is not committed
in that component. Exact local composition/render/reviews and actual future
operation/ref slots stay null. The null CONFIG fails bind_config before ROOT
access. This packet's inert finite plan retains zero actual invocations.

Standing scope `86784a09b70aa662420477c69328935bca1cfb71` covers routine repair
work and creates no new routine confirmation gate. It does not make SOURCE_CLEAR
execution authority. The burned local guard and all old evidence remain
immutable. Fresh names do not authorize another operation. No target access
occurred here.

All historical states remain: strict SDK helper1/SSH1 FAILED_UNVERIFIED; M1
app25/25 with app0/controller1 FAILED_NOT_ACCEPTED and earlier23/25 FAILED;
first post65431 EADDRINUSE cause UNKNOWN; whole-flow NI15/runtime UNVERIFIED;
missing original second post-bind/bundle-after/outcome/archive; stripping
warning and unknown full outer-build duration/OS exit; Windows SSH255/noRoute
FAILED_UNVERIFIED with remote state UNKNOWN; V-READER-STARTUP UNVERIFIED; all
failed guards; updater CLOSED. No runtime, COPY, native, artifact/operator,
continuous NI15, durability, UI/performance, product or release acceptance is
supplied by this review.

**Method and preservation.** Only reviewer-authored public Git/text/strict JSON/
hash/AST/literal data readers ran. JSON rejects duplicate keys and nonfinite
numbers. ASTs were data only. No supplied source/checker execution, import,
eval or executable compilation occurred. No tests/mocks/builds, private/state
reads, network/SSH/native/process/port/GUI actions, children, PR or push occurred.
The explicit commission excludes real CLI/E2E verification. No TypeScript changed.
[source-data-evidence.json](source-data-evidence.json) records 3032 successful
data comparisons and 392 exact public Git bindings; those are source/data
comparisons, not behavior or runtime tests. [reader-notes.json](reader-notes.json)
records the one corrected reviewer data-path assumption. No input was changed.

Opening checkout was clean. One inventory in [preservation.json](preservation.json)
contains all 5223 old entries: 4788 files, 435 directories, no symlinks,
78010223 working bytes, and 4787 original index paths. Every old entry's actual
bytes, full permission mode and type, every old logical index mode/blob/stage,
and the exact old path set were independently rechecked. Original whole index
bytes also match before staging. Existing Windows CRLF bytes remain untouched.
Closing canonical working-inventory SHA256 is
`f3ea19f02a8221573b477a49d7f6b06265c249c6ee2e47add1e237919bbb0186`.
Old index projection SHA256 is
`da624159cd8c5d01b947f66d1751eb494ee75533cf2de171940154aab42ddc01`.
Only this one NEW review folder is committed with hooks and signing disabled.
The actual commit, report hash, post-commit per-entry checks and clean status
are reported outside the packet to avoid self-reference.
