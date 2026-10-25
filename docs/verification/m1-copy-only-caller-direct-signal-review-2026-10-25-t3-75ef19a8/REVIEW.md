# Independent full COPY-only caller source review

2026-10-25, America/Vancouver. **SOURCE CLEAR, bounded to the exact new alias
and its source/data contracts. No concrete P1/P2 finding.** execution=false;
runtime_acceptance=false. This verdict supplies no execution authority,
materialization, parent composition, runtime approval or new human gate.

Candidate `23cb52f54cf9ef288500d4b09606d42303ff414f`, imported as
`b6d19d36f3672cce7e9735b371e03ae82e485426`, adds only the 20-file folder
`m1-copy-only-caller-direct-signal-2026-10-25-t3-56c87ccf`. Its folder tree is
`4e8152613cb59d01826cc29433e5e28d88a0f2ec`. All 20 original/import/current
entries agree in blob, mode100644, raw bytes and working0644 mode. All 19
SHA256SUMS entries match. [inputs.json](inputs.json) binds every packet file,
42 declared frozen inputs and two further reviewer-selected inputs. No
author pass flag was used as evidence of correctness.

The reviewed [complete source](../m1-copy-only-caller-direct-signal-2026-10-25-t3-56c87ccf/metadata-copy-caller-direct-signal.py.txt)
is 47,491 bytes / 796 lines, SHA256
`43415cd864f2e163eb053ec9a6999ce849a477cb4bc980a00591ec3ddb9ba0a1`.
It is a distinct future fixed leaf, `metadata-copy-caller-direct-signal.py.txt`.
The original R9 caller remains 46,908 bytes, SHA256
`01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`.
No old caller, saved result, guard, review or status was replaced.

I read the full source, every changed and unchanged body, module/entry blocks,
all 20 packet files, complete before/after excerpts, exact patches, CLI,
schemas, PARENT-DELTA and false/null plan. The seven-hunk source patch has
SHA256 `804ea49e3bc88305e48a42dfb16b4d3610206d9075e2881b347d5bc943f1ab63`.
Independent forward and reverse reconstruction matches every new and old
source byte; the independently generated zero-context diff also matches.
No patch was applied to a supplied file.

[observations.json](observations.json) binds all 36 declarations, including
both classes, Writer methods, decorators and the nested alarm handler. Only
`collect` and `main` change. All other 34 declarations are byte exact.
All remaining module nodes, imports and the os._exit entry are byte exact;
the header and new COPY_CALLER_LEAF constant are the only global changes.
The global patch also reconstructs forward and backward. Both full changed
function excerpts equal their actual physical source slices.

The complete historical reviews were read and rebound:

- R10 `1c0e4bcdc2dddcb64a3277db34d5ccd5e872fab2`, report SHA256
  `ad868577561145dcba9e74db7138e5118544e721ca3629f5c0c4f1808203c496`.
  Its narrower historical R9 SOURCE CLEAR remains attached to its old bytes.
- Original full COPY review `f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5`,
  report SHA256 `33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f`.
  Its HOLD_SOURCE/two P2 verdict remains historical on the old COPY source.
- COPY repair `c82cc12b8a2d709d8ce1e384545475159b24d2a9`, complete source
  120,718 bytes, SHA256
  `94efc9b4de615485827e471c3e3f31250b28ea64f35228955b91154e4ba2a22a`,
  its README and full parent contract, and independent R4
  `b56dcdc450d2152c06bff384f2760af2a82ceff0`, report SHA256
  `79873cafbbca9068e262dc5738686bb2495d3146a4f5a7dd73171188b60e64eb`.
  R4 closes the two COPY repair findings SOURCE_ONLY; it does not compose this alias.

**Direct signal and failure evidence.** Source412-534 retains the original
command10 ceiling, phase-1s publication reserve, command-0.5s work end,
nonblocking combined1MiB reads plus one detection byte, actual EOFs, known
drops and unknown unread data. The retained Popen handle enters HANDLES before
collection. The final actual poll is478; the next call samples CLOCK6 at480.
A real signed exit stays in the handle before that sample, and the small
finally482-484 retains reaped=True even if the sample raises. There is no
coercion, inferred exit or signal for a returned real code.

For None,486 requires the completed sample strictly before BOTH original
command and phase ends. Equality or a late sample skips the signal and records
timeout/loss. Only overflow or timed_out passes487. The one signal call490 is
`os.kill(proc.pid, signal.SIGKILL)`, using only that retained unreaped child's
PID. Between the sample and call are local gates and assignments. No hidden
Popen.kill wrapper, extra poll, PID search, OS query, group target or retry is
present. The full collector retains its existing two explicit poll and two
wait sites; the latter include the pre-existing EOF-without-exit work wait.
The final wait494 still uses remaining(deadline), with no renewed allowance.

kill_syscall_returned starts false at424 and becomes true only after the
call returns at491. It means syscall return only. ESRCH and other exceptions
leave it false, keep kill_attempted true, and retain the original error/loss
path496-498. No fallback poll/wait follows that exception. Final actual_exit
comes from the retained returncode at520. An exception before assembly can
leave collector_status null; no complete failure record is promised.

The frozen public library excerpts show the old kill/send_signal/poll chain
and os's POSIX export. They were read as data. No target library identity,
C implementation, audit policy, kernel result or signal was qualified.
Trusted POSIX behavior, exclusive retained-child ownership and absence of
another/automatic reaper remain assumptions. Scheduling after the sample and
OS stalls remain possible. Source ordering proves no hard deadline or reap.

**Entry, old data and complete body assessment.** main677-678 first requires
exactly five argv values, mode copy and the fixed new leaf. SOURCE fails
before canonical root work or clock sampling. Root/cwd/owner/mode/device,
canonical ASCII start and original start+120 admission remain. The original
SOURCE branches and SYSTEM bootstrap exception remain unreachable through
main. New COPY full results use v3 at691; saved SOURCE still requires exact
v2 in fresh_source620. Both fresh_source and generic complete are byte exact;
complete reads no kill_delivered key.

All unchanged bodies were assessed directly. Stop/require and the clock,
remaining/until/expired/common_deadline group preserve typed darwin6 ns,
original alarms/postchecks, conservative local conversion and no new interval.
canonical/read/sha/bound/encoded/sync_file preserve bounded pinned data and
publication barriers. Writer and all four methods preserve the existing
POSTLEDGER common writer, critical=False, regular precharge, sticky first
COMMAND/comparator causes and separate publication errors. There is no new
INITIAL writer, ledger repair, reset, refund or borrowed reserve.

startup/normalize_startup/native_environment preserve the original ten-key
COPY map and native environment. pin_inputs/expand preserve 17 pins and50
fixed vectors. parent_reference/inspect_completed_source/inspect_parent_helper
preserve same-ROOT saved refs, completed-file provenance and the nineteen-key
authentication interface. import_common/copy_bundle keep pinned sibling import,
prepared root/tree namespace and owner/mode/device checks. The parent still
must authenticate the full2006-entry/46,504,040-byte prefix, executable and ten
modules BEFORE private entry; the alias's later inspection cannot do that act.
The separate SYSTEM helper10 remains inside COPY120 and each child command10.

perform/normalize preserve raw stdout/stderr/status publication before the
comparator, first failed COMMAND before writes, partial/durable states and
both saved-basis and fresh-SOURCE comparisons. Only -verify sorts full strict
UTF8 splitlines, with duplicates and empty lines retained. fresh_source keeps
the original exact v2/50 ordered native/ref checks. direct_children performs
no new poll or wait. emit_summary retains the exact returned full-result ref,
closed five-key success.v1 receipt, actual <=512-byte/FIFO/PIPE_BUF checks,
one nonblocking exact-count write and failed full fallback. main retains first
causes, result-publication errors, pre-exit full data, one summary and os._exit.
Actual caller0/reap/both EOF/no errors before the SAME end remains external.

Independent public data checks confirm every17 pin, ten source-check and
materialization module identities, and all50 exact vectors expanded from the
recipe (seven objects by seven verbs, then deep verify). COPY remains exactly
copy-files-before, native50, copy-files-after:52 observations and208 durable
raw/status/comparator references on full success. No probe, repeat, completion
pass or extra native vector is added. Caps remain file8MiB/total32MiB/stream1MiB,
product16MiB/cleanup4MiB/accounting64KiB. Additional key bytes consume existing
capacity; no maximum-output-fit or publication guarantee follows.

The public saved validation and current parent's complete literal bindings
agree on SOURCE327813/SHA
`7482a637159e0389c5cd4c505778919bb49354437bc9feb1cf008d9db395712d`,
all53 recorded zero exits/no KILL,212 refs and six boundaries. The raw manifest
canonical SHA is `2baea069155b633485b1e8aed8fe72f3bf746f67de83a84f07a2daf0648fe4ee`.
The unchanged complete/fresh_source interface is compatible with those old
successful records. This checks public identities and recorded facts only;
no private original or raw record was reopened and SOURCE was not repeated.

**Schemas and future limits.** The new child-status schema's core24 keys equal
the actual collect dict. Published status adds both stream refs; retained
status permits either ref to be absent after partial publication. Direct-child
status can be null. Every failure-capable actual_exit/eligibility_exit domain
is signed integer/null without an unsigned minimum. Clock/count/byte domains
remain nonnegative; PID is positive/null. Success intersects actual and
eligibility exit with0, real reap/EOF and both signal flags false. No synthetic
receipt, validator execution or mock was used. Scalar domains do not prove
causal ordering, real exits or all semantic relations; future parent checks
must retain nulls, signed codes, losses and every snapshot/publication state.

Current COPY parent497/732/918/969/1140 still authenticates and launches the
old leaf, closes over kill_delivered, and requires v2. Its own direct-signal
repair does not make it accept this alias. No composition clearance is given.
The old compact contract's v2 wording remains historical; the exact future
consumer/contracts must explicitly bind NEW COPY v3 while keeping OLD SOURCE v2.

PARENT-DELTA correctly leaves materialization and exact composition BLOCKED.
Actual prepare source1127-1166 requires exactly260 initial names, then264 with
four durable outputs. The alias cannot be placed before prepare or between
those gates. Only after real preparation may a separately reviewed SYSTEM
materializer publish this new leaf in the SAME ROOT via reviewed
common.write_bytes(..., critical=False), existing lock/ledger and original
budget. It must preserve exclusive final publication, nofollow regular0600/
nlink1/owner/device/linked identity, fsync/readback, actual size/hash and errors.
No overwrite of the old leaf, extra root, private self-publication or reset is
permitted. Common's charge/write bodies were read directly, as were the full
operator/receipt contracts and schemas. No new materializer implementation or
actual publication receipt exists here.

Future review must bind exact alias/source review, materializer and parent,
then cover the new fixed vector, separate alias pins, cache/auth gates, v3 and
all duplicated failure status domains, full52/208 actual comparisons, actual
exit/EOF, compact/full receipt and operator/artifact/outer D4 boundaries. The
plan retains execution=false, zero invocation counts and all future slots null.
This source review fills none of those slots. Other frozen common.command,
port_diagnostic and old SOURCE signal paths keep their separate verdicts.

**Preservation and limits.** Opening checkout was clean at
`64bcde791ea95472b796fab623519a124ee553d0`, tree
`76cd29c956e8fbf20484ca905be08a12ae19ddc7`. [baseline.json](baseline.json) and
[preservation.json](preservation.json) bind4655 old entries and canonical digest
`aa589018d8e86069919ae915b0868b5cee89ef27d2239845b579d9893efa3d48`.
Every entry was checked against HEAD and index mode/blob/stage0, actual file
kind, full permission mode and raw working bytes. The one existing Windows
CRLF conversion is explicit. Closing checks repeat each comparison and the
whole digest; there is no duplicate large inventory. Only this NEW review
folder is owned. The folder-only commit disables hooks/signing; its identity
and post-commit clean check are reported separately to avoid self-reference.

Only reviewer-authored public Git/text/hash/strict JSON/AST-data readers ran.
No supplied source/checker execution, import, eval or executable compilation;
no tests/mocks, build, private/state read, network/SSH/native/runtime action,
target process/port action, child agent, PR or push occurred. No TypeScript
changed. The explicit SOURCE/DATA scope excludes CLI/E2E execution. No actual
startup, signal/reap/EOF, clock-rate/OS-stall, crash durability, D4, whole-flow
NI15, native COPY/signature/ABI, UI, product or release acceptance is proven.

Preserve SDK FAILED_UNVERIFIED helper1/SSH1; M1 app25/25 app0/controller1
FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431 EADDRINUSE cause
UNKNOWN; whole-flow NI15/runtime UNVERIFIED; missing original second post-bind,
bundle-after/outcome/archive; stripping warning and unknown complete outer
build duration/exit; Windows SSH255/noRoute FAILED_UNVERIFIED and remote state
UNKNOWN; V-READER-STARTUP UNVERIFIED; all failed evidence/guards; updater CLOSED.
