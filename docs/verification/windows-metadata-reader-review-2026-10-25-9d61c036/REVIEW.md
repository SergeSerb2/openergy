# Frozen metadata reader and local CLI driver review

Result: **three P2 findings; no P1 found within this scope**. This is a source/data review. It is not approval, an execution receipt, or Windows qualification. No supplied program was imported or executed by this reviewer. No artifact was repaired or integrated.

The reader scope is candidate `78759877392af3e41bb4d51fb5f40470c33ff6f9`, imported without byte changes at review base `f9ea16761c0333ac8c965d2aee441e508a8b390d`. Its owned folder is `docs/verification/windows-metadata-reader-repair-2026-10-25-588e28c3`. The separate driver scope is `aa474ae00703abd2f39e3d1bc5c4432ba468a222`, folder `docs/verification/metadata-reader-local-cli-2026-10-25`. Revised drivers are outside this verdict.

## P2 R1: Recheck the work deadline after the final exit observation

Source: `metadata_reader.py.txt:283-316`, especially the final `observe()` at line 316; capture loop at lines 361-362; completion predicate at lines 420-433.

`pump()` checks its deadline before selecting, but it can return after that deadline. It does not check again after `select()` and the final `observe()`. If both pipes have reached EOF while the child is still running, the selector has no registered streams. A last poll can start just before `work_end`, resume just after it, and observe exit 0. The capture loop then ends with no first error. Finalization checks only the larger absolute deadline, so it can mark this late work COMPLETE and return CLI exit 0 using the cleanup/finalization allocation.

This is a static control-flow finding. The saved four-case flow does not exercise this timing boundary. It does not require a claim that an OS call can be interrupted.

Fix: check `work_end` after the capture loop, inside the capture try block, before accepting normal completion. Record the completion observation time if it is needed as evidence. Also check the pump deadline after its last observation. A late observation must retain the actual local return code but set the first capture TimeoutError and keep INCOMPLETE. Keep the existing statement that hard OS preemption requires an external envelope.

## P2 D1: Accept the actual SIGPIPE exit after cap-triggered pipe closure

Source: `verify-cli.py:103-106`. Related reader source: `metadata_reader.py.txt:363-367` and `379-386`.

The output-cap predicate requires `local_kill_called=true` and child return code `-SIGKILL`. The reader first closes pipes when it observes the overflow sentinel. The real `yes` child can terminate with SIGPIPE before settlement calls `kill()`. In that case, the reader correctly preserves `-13` and does not claim a kill. The frozen driver rejects that valid settlement result.

The parent's saved flow confirms this defect against the exact reviewed hashes. The driver exited 1 and `flow/result.json` remains INCOMPLETE. Success, nonzero, and partial-timeout cases were accepted. The cap case retained 128 stdout bytes, observed 129, recorded one byte not preserved and `OverflowError/capture`, returned reader exit 1, observed child exit `-13`, and recorded `local_kill_called=false`. Its stderr was empty. Both raw stream hashes match their receipt entries. This evidence supports the cap/settlement behavior for this invocation; it does not make the failed driver flow pass.

Fix: check the first OverflowError in capture, exact bounded raw prefix and receipt accounting, reader failure, and an observed actual child exit. Permit the supported settlement outcomes, including SIGPIPE without a kill and SIGKILL after a kill, with consistent kill fields. Do not require a kill after an already observed exit. Preserve this failed flow and review any revised driver separately.

## P2 D2: Reject a flow that finishes after its outer work deadline

Source: `verify-cli.py:53-54`, `73`, and `82-111`.

The 28-second elapsed check runs only before each case. The wait receives a timeout, but after it returns the driver reads and checks three files without another outer deadline check. The last case can therefore start within budget, resume late during the wait or a file read, then reach `LOCAL_CLI_CASES_PASSED` after 28 seconds. Recording `ended_monotonic` and final elapsed time does not reject that outcome. The code also does not compare the actual observed reader exit time to the per-wait outer deadline. The saved flow finished well inside the envelope and does not demonstrate this defect.

Fix: retain absolute outer flow and per-invocation deadlines. Recheck them after the reader exit is observed and after evidence reads, before marking the case accepted or marking the flow passed. Keep timeout enforcement and late-result rejection distinct: a stuck OS call may be uninterruptible, but an observed late return must still fail. Define any interpreter-startup allowance explicitly; do not silently treat the reader's internal budget as an independently enforced outer budget.

## Repairs and remaining source assessment

- Directory fsync now closes its descriptor in `finally` at reader lines 331-335.
- Overflow and partial-write paths close pipes before further capture can create a gap. Settlement closes full/broken pipes and has a separate bounded local-exit poll loop at lines 387-398. D1 concerns the driver, not a need to replace the reader's exact observed exit with SIGKILL.
- Raw reads use at most 4,096 bytes, bounded by the remaining per-stream and aggregate allowance. At the cap, only one sentinel byte is observed and not stored. Partial writes update the retained byte count and hash. Failure keeps INCOMPLETE. The fixed 65,536-byte INCOMPLETE receipt is written before launch.
- Admission binds the exact private command bytes, fixed SSH argv, adjacent query hash, and closed command-layer data. Null effective limits remain UNVERIFIED. Policy ceilings do not become installed limits or route qualification.
- Completion requires both EOFs, actual local exit 0 and no first error. Metadata mode additionally requires empty stderr and eight validated records. R1 is the missing terminal work-deadline condition. A receipt alone still needs actual CLI exit and external time evidence.
- The PowerShell text emits begin, six file records, and complete. The six requested paths match the Python constants. Length, version fields, full SHA-256 and runtimeconfig content remain in the file record shape. Hashing is streamed with file/aggregate caps; config buffering and JSON output have explicit caps. Native PowerShell syntax and behavior were not parsed or run here. Concurrent installed-file mutation and unsupported text remain proof limits.
- The reader always keeps remote exit/current state UNVERIFIED and remote cleanup false. No end record, local kill, or local CLI result supplies native build, product, stop, route, installed-version, or effective-limit acceptance.

## Data bindings and saved evidence

`bindings.json` binds all ten reader package files, all three frozen driver files, 30 public inputs, all 179 historical files, and the two old private hashes as preserved public data. Each public/historical member matches its recorded hash, Git blob, original parent `b221d9807cbc4c3db5a30f2114399ce20942dcf1`, candidate, imported base, and this checkout. The old private files were not reopened; this review confirms preservation of their public hash bindings, not their current private storage bytes.

Both SHA256SUMS files and all eight candidate manifest entries match. Five candidate JSON files parse. Both Python texts parse as AST data. Full-index binary Git diffs are retained as `.patch.txt` data and their added-file bodies were independently reconstructed and compared to immutable blobs. No patch was applied to the checkout or index.

The reader SHA-256 is `e7fb7a8fde0f63246796d5209f85571d2de0418ab96536797101561bec39b380`; query SHA-256 is `3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`; frozen driver SHA-256 is `8090e3dce24bbaf5b557f2ee219ab08ba20ba71477ad71f4583390b266622915`. Static native-line arithmetic gives 12,299 UTF-16 units including NUL. This is a size calculation, not native acceptance.

The immutable driver README remains PREPARED; NOT RUN. The later parent flow is separate saved evidence, read only under the user's exact allowlist. Its bindings cover `driver-result.json`, `invocation.json`, `flow/result.json`, and each case's receipt/stdout/stderr. No private raw file or full invocation was copied into this report. The flow remains FAILED/INCOMPLETE; no retry or revised checker was reviewed.

Coverage stays narrow. The saved local commands do not exercise nonempty stderr, competing stream/aggregate caps, disk faults, preparation/close failures, terminal deadline races, the Windows response parser, or native command execution. No test, mock, fixture, helper, process/port probe, preparation, build, signing, network, credential access, private write, or child delegation ran in this review. All review writes are confined to this new folder. TypeScript checks are outside this documentation/source-data scope.

The original metadata result stays TIMED_OUT_UNVERIFIED; original Windows FAILED/UNVERIFIED, historical preservation UNVERIFIED, installed versions/effective limits UNVERIFIED, and zero native daemon.stop requests stay unchanged. The saved local evidence does not alter those gates. This report issues no approval and no executable receipt.
