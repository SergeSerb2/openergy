Independent full ordinary-helper SOURCE/DATA review. 2026-10-25, America/Vancouver.

**BOUNDED SOURCE CLEAR: 0 P1 / 0 P2. execution=false; runtime_acceptance=false.**
This verdict covers the two complete inert module proposals and their packet,
with the relevant frozen consumers. It grants no execution authority, runtime
approval, new module admission, or whole-controller/composition verdict.

Candidate `d405da38eabc4d1a7a008c2c267b4cebf6a1afe4` has parent
`4ea32a7538aa1b996e60fe7dd185497cbba75356` and tree
`36fd8d4af45458e236dab6a413d5953d35a46eba`. It adds only the 15-file
[proposal folder](../m1-ordinary-helper-signal-repair-2026-10-25-t3-552c8d93/README.md).
All 15 candidate/import/working packet blobs and modes agree. The review checkout
opened clean at import `64bcde791ea95472b796fab623519a124ee553d0`.
Its whole tree differs from the candidate because it also carries separate
review/proposal folders. No whole-tree equality is claimed.

The exact frozen sources, read in full before and after, are:

- common: 12781 bytes, SHA256
  `20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c`,
  from `ordinary-combined-recorder-diagnostic-2026-10-25/source/common.py.txt`;
  proposal 14797 bytes, SHA256
  `da670bf1bf1e135ddf2d8340d3c8cb1362ff2b1e9cc551659b4dfe9106461061`.
- port_diagnostic: 10096 bytes, SHA256
  `9d35bf1adbde9209562d9b4cfa211b75574a0fb1922dc0297761880c2d6d3985`,
  from the same frozen source folder; proposal 11022 bytes, SHA256
  `1b2d744d0c2f2a9334a392413ce10257d8e4a069eba1357844c3ab30af70627e`.

[data-evidence.json](data-evidence.json) binds all packet files, 14 checksum
entries, six strict unique-key JSON documents, 28 public input records, ten
historical modules, and 17 pins to actual Git bytes. Each module patch was
reconstructed in memory. Every context/deleted row, old/new position and hunk
count agrees; the result equals every final byte. The global patch also
reconstructs exactly. Patch SHA256 values are:

- common: `f30997f782e5b9c082e0b83229a2d70d4e8571a09bc0b1292579d9c38c66bb83`.
- port_diagnostic: `c888ecc51c284f99d68ad54428837cfc9fd12040ec0ec21316a9096f460b39cd`.
- globals: `0a03a8ad0caa70ec6431f16c668bab19a8e0bb9004a9717f24ef53711efaee3c`.

[declaration-bindings.json](declaration-bindings.json) covers all 27 physical
declarations, including classes, methods, nested `io_until.expired`, decorators
and final LF. Only `common.command` and `port_diagnostic.query` change. All 25
other declarations and every signature/default are exact. Common's full global
remainder is exact; port adds only `import signal`. Both complete function
transcripts agree with the actual source slices. No new entry point appears.

In common.command, lines294 and319-320 latch the original exception before
cleanup/finish. Lines298-301 take the retained child's actual poll, then an
immediate same-process monotonic sample, then allow one direct
`os.kill(process.pid, signal.SIGKILL)` only for None and sample < original
deadline. No extra OS/PID query or Popen signal wrapper intervenes. The actual
poll code remains in the retained Popen before the clock call. A late sample or
clock failure cannot erase it. Late None or failed sampling reaches no new
cleanup wait. The existing wait uses only `remaining(deadline)`.

Cap/nonzero failures at312-317 precede finish at326. Cleanup and later finish
failures use the existing sticky COVERAGE_ERRORS channel, capped at32 entries
and2048 characters. The outer handler at334-338 preserves the first error even
if io_until restoration fails. A finish exception with no prior error still
fails the command. All original Popen/communicate/wait call ASTs remain exact.
The post-communicate cap remains a posthoc check; it does not bound memory while
reading. No pipe-emission or hard-deadline guarantee is inferred.

In port query, lines153-158 retain actual signed exit/reap data before the next
clock. Lines160-167 use the same strict eligibility rule and original wait
remainder. The three new eligibility fields distinguish no returned poll, None,
and a real signed result. `kill_requested` records an attempt; `kill_returned`
records only successful syscall return. Neither means delivery or reap. A real
exit survives later clock/alarm failure through179-181. Unknown status keeps the
handle in the diagnostic-only map and cannot satisfy the unchanged completeness
predicate. Query errors precede cleanup errors. Capture, pipe-close, parser,
EOF/truncation handling and status suffix remain exact.

[context-evidence.json](context-evidence.json) binds the actual consumers and
17 relevant complete declarations. All five active recorded command callsites
retain three-item coverage tuples. The unchanged git_file wrapper has no caller
in this ten-module map. Recorder start is its constructor/intent plus spawn;
this frozen API has no `start` method. Sampling/budget/publication gaps remain
failures. Finish records actual signed returncode or None without querying a
PID; audit and controller/recovery consumers keep sticky errors and UNVERIFIED
coverage. No new sample, counter, reserve, ownership or product signal is added.

Controller SHA256 remains
`08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4`.
Its original FAILED bind is stored before diagnostics. Its sole query consumer
remains post-EADDRINUSE and uses `/usr/sbin/netstat -AanW -p tcp` with
`min(original port end - 0.05, current monotonic + 0.5)`. The original pre-bind
snapshot is unchanged. The diagnostic keeps its80/20 split, regular publication
capacity and fixed caps. No new prequery, bind retry, SO_REUSE, alternate port,
product action or cause claim appears. The bind still fails and throws.

The full COPY review `f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5`, its findings
and retained public subprocess excerpts were read and bound. The full prepare
ROOT-layout review and both combined/tuple reviews were also read. Their
verdicts remain attached to their own bytes. The retained POSIX library text
shows kill -> send_signal -> poll -> waitpid and possible skipped signalling.
It supports this source repair, but does not establish any installed target
library identity or executed branch. Original COPY HOLD_SOURCE findings are
not transferred or rewritten by this verdict.

Two data limits are explicit. The author lists def-only start lines for
io_until/byte_lock while hashing their decorator-inclusive bytes; this review
records exact physical ranges. The serialization for four author global
digests is unspecified and was not reproduced. Those opaque digests are
UNVERIFIED and unused as proof. Complete actual global bytes, known-format
independent hashes and exact patch reconstruction establish the source scope.
The supplied91 pass flags are author claims, not behavior evidence. Corrected
review-reader assumptions and one metadata-display syntax error are retained
in the evidence. No supplied source or acceptance predicate was changed.

The full candidate whitespace check remains **exit2** for literal blank context
rows in the exact patches. Excluding precisely those patches is **exit0**.
Raw results are retained. Required patch bytes were preserved.

All12 future materialization/review/authentication/runtime fields remain null.
New actual module paths/map, active import surface, full source checks,
controller/watchdog composition, complete private-prefix/ROOT pre-entry
authentication and their exact review remain **BLOCKED**. The old17 pins and
source checks do not admit either new hash. No ROOT module or alias was
materialized. Separate prepare, COPY/caller, collector/observer/assessor and
driver work receives no new clearance. Exclusive retained-child reaping is a
required source precondition; no concurrent waiter, SIGCHLD discard or external
reaper is admitted. Scheduling pauses, audit hooks, clock/OS stalls, delivery,
termination, actual reap and EOF remain unproved. Failed cleanup may leave an
unreaped helper; it grants no retry, replacement, reaper or time extension.

[preservation.json](preservation.json) binds opening tree
`76cd29c956e8fbf20484ca905be08a12ae19ddc7`, all4655 old entries and canonical
raw-working/index/tree digest
`fd548cc3cadba87f5569964435d8b2025f0236da82e4826ed228e63f3c4760a2`.
Every old entry is compared individually for Git/index blob and mode, actual
bytes/length/SHA256 and full permission mode. The original CRLF working form of
openergy-run.cmd is preserved separately from its normalized Git blob. The
producer's4566-entry parent preservation digest was also reproduced exactly.
No duplicate old inventory is committed. Only this new review folder is owned;
hooks/signing are disabled. Final commit and post-commit preservation/clean
status are reported outside these committed bytes to avoid self-reference.

Only public Git, file/text/hash, strict JSON and AST/literal DATA readers were
used. No supplied code/checker was executed, imported, evaluated or compiled to
executable code. No tests/mocks, builds, private/state reads, network/SSH/native/
runtime/process/port actions, children, PR or push occurred. No TypeScript
changed. End-to-end execution was excluded by this commission.

All history stays intact: strict SDK FAILED_UNVERIFIED; M1 app25/25/app0 with
controller1 FAILED_NOT_ACCEPTED and earlier23/25 FAILED; first post65431
EADDRINUSE cause UNKNOWN; missing original second bind, bundle-after, outcome
and archive; whole-flow NI15/runtime UNVERIFIED; stripping warning and unknown
full outer build duration/OS exit; Windows SSH255/noRoute with remote state
UNKNOWN; V-READER-STARTUP UNVERIFIED; updater CLOSED. This review supplies no
runtime, product, native/UI/performance or release acceptance.
