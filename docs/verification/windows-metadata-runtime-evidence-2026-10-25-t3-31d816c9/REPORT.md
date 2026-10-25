# Saved metadata-reader runtime evidence audit — 2026-10-25

**Result: the saved local four-case CLI evidence is consistent with the reviewed source and required outcomes. The separate Windows metadata attempt remains FAILED_UNVERIFIED.** Its saved exit chain is outer reader 1, inner reader 1, SSH 255. No metadata was obtained.

This is a saved-data audit. It adds no execution or remote acceptance. The external collector clock and timing limits below remain open.

**Scope and revision binding.** The exact parent is `22e411d1f1a4514ce6598227806ffa1d41a2347e`. This report owns only its new folder. Candidate `891a443fa29513bc9747e74f6981b8aaebd000e2`, imported at `bd0319df37abcbccfce302e435ff4f125c3b2e96`, has 11 byte-identical package files at the original revision, import, and parent. Review `04b4329698065307f16153f0472670e86a46dda9`, imported at `781c61e945b5e38aabbfea9a1ae4237afe7920eb`, has the same equality for its three files. Its bounded SOURCE/DATA CLEAR verdict remains a source/data verdict.

The private labels map to the two exact roots authorized in the task brief. Absolute private paths are omitted. `private-cli` has 25 regular files and 311,947 bytes. `private-query` has 17 regular files and 177,326 bytes. The total is 42 files and 489,273 bytes. Every root component and entry was checked without following symlinks. No symlink or special file was accepted. All private reads, including bounded rereads and the final preservation check, total 7,339,095 bytes, below 64 MiB. No private file was written.

[The index](evidence-index.json) publishes only private labels, relative file names, byte sizes, and SHA-256 hashes. It includes the original guards and captures. [The audit facts](audit.json) retain sanitized exit, clock, deadline, and hash-check results. No raw argv, selector, endpoint, SSH alias, PID, process command line, selector hash, or private config is published.

**Frozen source.** All five saved source copies equal the reviewed candidate bytes and the supplied source hashes:

- Reader, 22,658 bytes: `15d2a6275d70135857db059bdf1982f5071dd819c3c73ea3ea27e08cd9802bc4`.
- Driver, 12,426 bytes: `59ba17b43e4414998cc171b897f173a8282c7277908a23636e1d086e8a4f9b59`.
- Query, 4,586 bytes: `3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`.

The exact source package is [the exit-deadline repair](../windows-metadata-exit-deadline-route-repair-2026-10-25/README.md). The bound independent review is [the source/data review](../windows-metadata-exit-deadline-route-review-2026-10-25-9902f5e5/REVIEW.md). Supplied source was read as text only. No supplied code or AST was executed or imported.

**Local full CLI, one recorded invocation.** `invocation.json` binds the frozen driver, reader hash, source copies, and output directory. `driver-result.json` separately records actual driver exit **0**, no outer timeout, and elapsed **0.48449754099965503 seconds**. This exit record is distinct from `flow/result.json` and its saved pass status. The saved driver invocation and all four fixed child commands match the reviewed driver. Each parent receipt hash matches the actual 65,536-byte receipt. Every stream hash and byte count matches the raw file.

The four case outcomes are:

- `success`: reader 0, child 0, receipt COMPLETE. Stdout has 7 bytes; stderr has 0. Both streams have EOF and no overflow.
- `nonzero`: reader 1, child 1, receipt INCOMPLETE. Both streams are empty with EOF and no overflow. First error is null, secondary errors are zero, and no kill was called.
- `partial-timeout`: reader 1, child -9, receipt INCOMPLETE. The first error is TimeoutError in capture. The reader called local kill. The exact 7-byte prefix survives; both streams have EOF and no overflow.
- `output-cap`: reader 1, child -13, receipt INCOMPLETE. The first error is OverflowError in capture. Stdout preserves exactly 128 bytes of the required prefix, reads 129, and leaves 1 observed byte unpreserved. Its overflow flag is true. Neither stream has EOF. No local kill was called. This is the required actual SIGPIPE settlement branch.

All four saved `accepted` flags are true. The audit also checked the raw data, expected exits, bindings, budgets, caps, and deadlines independently of those flags. The expected failure receipts remain INCOMPLETE.

For each case, the driver records exit observation after its actual wait and before receipt reads. The [driver text](../windows-metadata-exit-deadline-route-repair-2026-10-25/verify-cli.py.txt) establishes this order at lines 98–118 and the common gate at lines 120–155. The saved bounds are finite. Reader starts fall within the one-second startup allowance. Work and absolute bounds equal start plus the exact declared budgets. Invocation deadlines equal the earlier of the whole-flow bound and the case bound.

The parent-observed reader exit times are strictly before both relevant bounds:

- `success`: observation 10444.511580791; reader absolute 10447.503758083; invocation 10448.469385.
- `nonzero`: observation 10444.554776458; reader absolute 10447.54656575; invocation 10448.512102291.
- `partial-timeout`: observation 10444.852743041; reader absolute 10445.839326; invocation 10446.805295208.
- `output-cap`: observation 10444.907817875; reader absolute 10447.899147125; invocation 10448.853881708.

The driver flow starts at 10444.469270958 and has a 28-second deadline of 10472.469270958. The outer invocation starts at 10444.427308625. Adding its saved elapsed duration gives **10444.911806166**, after the last case and before the flow deadline. This value is **derived**, not a separately saved exit timestamp. The external collector implementation and its own clock descriptor are absent from the allowed roots. Thus the absolute outer comparison depends on the stated same-host monotonic contract. The actual saved exit 0 and duration remain evidence; they do not establish an authenticated outer watchdog or hard OS preemption.

**Clock descriptors.** The driver, all four case readers, and both nested query readers have exactly equal full saved descriptors, including the private resolved interpreter field. The published fields are `time.monotonic`, `mach_absolute_time()`, `darwin`, CPython `[3, 15, 0]`, resolution `4.166666666666666e-08` seconds, monotonic true, and adjustable false. The local case readers are direct children launched by the frozen driver with its interpreter. The nested query commands use the same saved interpreter token, isolated mode, and no-bytecode mode.

These are saved descriptors, not a fresh installation check or clock attestation. Neither external collector retains a separate clock descriptor or observer source here. Do not extend the comparison to another Mac, M1, a remote Windows clock, or wall-clock time.

**New metadata attempt, one recorded invocation.** The execution-start record binds the reviewed reader/query hashes, the R8 review commit, both raw command JSON hashes, and the raw local `driver-result.json` hash. The inner reader command is bound by `outer-command.json`. The outer reader invocation is bound by `execution-start.json`. Both reference the saved reviewed reader copy and exact caps and budgets. The saved public transport prefix and encoded query payload match the frozen source. No transport, selector, or command value is copied here.

The outer receipt records the actual inner reader exit **1**. The inner receipt records the actual SSH child exit **255**. Both record local child start and exit, no kill, null first error, zero secondary errors, receipt INCOMPLETE, and metadata_complete false. The final operation record independently retains actual outer CLI exit **1**, no timeout, no unreaped outer child, and **FAILED_UNVERIFIED**.

The original inner stderr has **61 bytes**. Its actual bytes and receipt hash agree. The sanitized transport diagnostic is **“No route to host” (address PRIVATE)**. Inner stdout has **0 bytes**, so it contains **zero metadata records**. Both inner streams reached EOF, with no overflow and no observed bytes lost. The outer capture streams are empty with EOF and no overflow. Complete raw capture and local exit observation do not make the metadata operation successful.

The parent starts at 11485.52213375. It observes outer exit at **11485.61810675**, before its absolute deadline **11525.52213375**. The same observation is also before the inner absolute deadline **11520.595519083** and outer reader absolute deadline **11525.563035666**. The inner work completion observation, 11485.60458625, precedes its work deadline 11515.595519083. The outer work completion observation, 11485.609915875, precedes its work deadline 11521.563035666. The audit uses these exit and completion observations; earlier receipt elapsed values and pre-write Booleans cannot replace them.

The final record retains elapsed **0.0959893330000341 seconds**. Its saved parent start-to-exit timestamp difference is **0.09597299999950337 seconds**. The duration is about **16.333 microseconds longer**. The exact reason is not established because collector source is absent. Both values remain visible; neither is rewritten.

The first parent assessment error remains **AssertionError**. No retained traceback or collector implementation identifies its exact failed assertion. This assessment error does not erase the separately evidenced **SSH transport failure**. Final classification stays **FAILED_UNVERIFIED**.

**Preservation and proof limits.** One guard and one receipt set per root support the recorded one-time local CLI run and one-time metadata attempt. They cannot prove the absence of unrecorded activity. The audit made no new attempt, retry, remote probe, query, process inspection, product/build/signing action, credential access, test, mock, fixture, or child agent. It used only bounded file/JSON/hash checks and static Git/source reads.

The saved preparation rejection remains REJECTED. The disabled preparation snapshot still has execution=false and PENDING_R8. Those earlier states are preserved alongside the later execution record. The bound historical review still retains the original local driver FAILED/INCOMPLETE and old metadata TIMED_OUT_UNVERIFIED outcomes. Historical private roots were not opened, and historical reports were not rewritten. The final private inventory and both guard hashes equal the audit inventory.

No Windows metadata, installed versions, effective limits, native parser/runtime, stop behavior, remote query exit/current state, or remote cleanup is proved. Remote state stays UNVERIFIED and remote_cleanup_proved stays false. Saved native stop requests and product/build command counts remain zero. Hard wall-clock enforcement remains false; the external envelope requirement remains true. Storage/close faults, transactional receipt durability, hard OS preemption, nonempty-stderr success, competing caps, M1 execution, and other transport variants remain outside this evidence.

Only this new folder is committed, with hooks and signing disabled. Existing source, historical FAILED/UNVERIFIED results, private originals, and guards remain intact. No TypeScript changed; compilation and runtime checks are outside the explicit saved-data scope.
