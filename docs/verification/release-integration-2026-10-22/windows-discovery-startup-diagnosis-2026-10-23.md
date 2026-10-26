# Windows discovery startup diagnosis — 2026-10-23

**The matched Windows failure demonstrates a verification assumption: an immediate receiver drop is not necessarily a pre-start cancellation.** In the second diagnostic, the old limited view closed while the new receiver was still held, and the standard replacement delivered `SearchStarted` before its receiver was dropped. Both replacements therefore started successfully before those drops. The same three assertions then failed, with the settled gauge at zero. No product defect is established by this bounded diagnosis.

The original [64e0005 acceptance record](windows-bounded-discovery-2026-10-23.md) remains **startup FAIL 11/14, retained PASS 19/19, control PASS 23/23**. Its exact `before=1 after_barrier=0 after_settle=0 live_closed=true` output is unchanged. The original had no startup trace, so its precise interleaving cannot be reconstructed retrospectively. The new matched reproduction has a different intermediate gauge; that difference is retained below. Nothing here relabels the original failure as a pass.

[Structured evidence](windows-discovery-startup-diagnosis-2026-10-23.json) contains the exact source bodies, commands, input/tool/library/helper hashes, full check lists, ordered records, first failures, and cleanup. The original-fixture file manifest covers all 2,579 original files. Diagnostic source and exact diffs and archived workflow scripts are supplementary evidence, not product changes or revised acceptance probes.

## Source contract and question

The frozen source is `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`. A local Git comparison confirmed that parent root `988e7837f212fcbde355e8e2f7452e82faf5a412` has the same crates/vendor/probe/Cargo/toolchain source.

- `service_daemon.rs:443–469` queues a command before creating and sending its UDP wake signal. API return does not acknowledge startup; the worker can already process the command before the caller receives its handle.
- `service_daemon.rs:4225–4254` attempts the initial `SearchStarted` send first. A failed send returns without touching the old subscription. A successful send removes the old subscription and its retransmissions, then installs the replacement. Same-type replacement is also the public `browse_limited` contract at lines 524–528.
- `service_daemon.rs:1574–1595` removes both sender/state and the matching retransmissions, and checks live sender identity with `same_channel`. The repeating browse branch checks this identity before pushing an event. Normal disconnected-sender cleanup can later remove a successfully started subscription after its caller drops the receiver.
- `LimitedBrowse` keeps its lifetime receiver private. It exposes snapshots and closed state, not `SearchStarted` or a startup/commit acknowledgement. An initially dirty snapshot or `new.is_closed()==false` does not prove startup.
- `discovery_recovery.rs:593–617` queues 16 retained cache replays, drops 32 distinct limited handles, then immediately drops the two GUARD replacements. Its comment calls them failed replacements and unconditionally requires the prior identity and exact gauge 1. Replay completion at a later metric barrier does not prove the two GUARD startup sends failed.

The ranked hypotheses were post-start replacement, committing despite a receiver already dropped before startup, and cleanup from an older same-type generation. No cause was selected from source plausibility alone.

## Two real diagnostic runs

Only two new ordinary public-API startup runs were used. Both used a separate real publisher child, 800 ordinary advertisements, the original 16/32 workload, capacity 64, all original check bodies and timing calls. A programmatic source comparison checked those bodies and waits byte for byte. The helper added only `log::Log` capture, caller observations, and one nonblocking `try_recv` on the standard replacement before its drop. It did not expose private fields or change any worker/vendor source. There were no worker pauses, rendezvous, test hooks, injected commands, resource tricks or assertion changes.

| Run | Observation mode and observed schedule | Checks | Exit | Wrapper duration |
| --- | --- | ---: | ---: | ---: |
| 1 | Immediate trace output; pre-start cancellation | 14/14 | 0 | 13,970 ms |
| 2 | Buffered guard trace; post-start cancellation | 11/14 | 1 | 16,870 ms |

These are instrumented diagnostic outcomes, not new acceptance results. Trace output changes observation cost. The first helper wrote selected existing logs immediately; the second buffered records and flushed them after the flow to reduce that cost. No workload was paced or delayed to force an outcome. No third run or soak was used.

In run 1, the limited receiver drop completed at 9,118,173,400 ns and its worker command began at 9,120,767,200 ns. The standard drop completed at 9,118,556,000 ns and its command began at 9,120,840,300 ns. Both subsequent existing debug records reported `Failed to send SearchStarted(...): sending on a closed channel`. All 32 distinct types also recorded initial-send failures. The actual metrics were:

```text
CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows=16 before=1 after_barrier=1 after_settle=1 live_closed=false
```

The old view remained live and received its later update. This is direct observed pre-start failure and preservation, not an assumption based on queued work. Run 1 trace.

Run 2 supplied the matched failure. The following times are monotonic nanoseconds from the parent process's diagnostic clock. They are observation boundaries, not network latency or exact unlogged send timestamps.

| Recorded time (ns) | Event |
| ---: | --- |
| 9,187,689,200 | Before the limited replacement call, old `live_closed=false`. |
| 9,187,738,400 | Worker enters the non-repeating limited GUARD replacement. |
| 9,187,936,900 | After API return and before drop, caller records old `live_closed=true` while it still holds the new receiver. |
| 9,187,939,000 | Caller records completion of the new limited receiver drop. |
| 9,187,952,900 | Worker enters the non-repeating standard GUARD replacement. |
| 9,188,165,800 | Caller records `first_event=Ok(SearchStarted(...))` on the new standard receiver. |
| 9,188,170,000 | Caller records completion of the standard receiver drop. |
| 9,241,744,300 | Next GUARD repeating-command entry, after old-view loss and both drops. |

The standard `SearchStarted` reception directly proves startup before drop. For the limited receiver, this is a source-backed commit inference: the observed old view closed while the replacement receiver still existed, and the initial-send-success branch performs that removal. The helper does not log the precise limited send result/time directly. No older repeating GUARD command occurs between the observed open old handle and its closure. Thus an old retransmission does not explain that earlier loss in this trace. Run 2 trace.

Run 2 retained the exact original failed check names:

```text
CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows=16 before=1 after_barrier=6 after_settle=0 live_closed=true
CHECK cancelled-distinct-types-retain-no-state=false
CHECK cancelled-replacements-preserve-identity=false
CHECK surviving-handle-receives-live-update=false
```

The intermediate six is a real observed gauge and was not replaced with the original zero. It did not persist after settle. Neither run is evidence of 32 orphan states. Successful startup followed by ordinary receiver-drop cleanup permits the settled zero and old-view loss; the unchanged probe incorrectly applies its pre-start preservation assertions to that schedule. The first run's two failed startup sends followed by preservation give the opposite observed case. Both failed sends followed by old-view loss would require product investigation; that sequence was not observed.

## Smallest next observational step, not implemented

No product fix or acceptance rewrite is proposed as completed work. The smallest additional observation would be a diagnostic-only log at the exact initial `try_send` result, before map mutation, with service type, limited/standard kind, per-request/channel identity, actual result and monotonic time. Match it to caller drop boundaries. This needs no worker rendezvous, pause, command injection or timing control. Parent must decide and assign any further change.

A strict future oracle must distinguish the actual outcomes, rather than increase queued work:

1. Only a recorded `Disconnected` initial send establishes pre-start cancellation. For that case keep the exact old identity, gauge 1, later old-view update and zero extra cancelled state requirements. Lost identity or orphan state stays red.
2. Treat an accepted start as a separate contract. In a separately assigned positive-start case, keep the new handle alive until startup/resolution is observed. Require the old handle closed, the new one live/fresh and exactly one limited state. Then require cleanup after dropping it and require a newer live generation to survive older work.
3. Unobserved or unmatched startup outcomes mean unverified coverage, never a pass. Current ordinary APIs cannot guarantee that an immediate drop precedes worker startup. A deterministic outcome classifier does not force the scheduler to produce pre-start cancellation. No original gauge check should be relaxed or identity check skipped.

## Hashes, failures and cleanup

Both helpers linked existing frozen rlibs directly. The selected direct mdns-sd artifact is `libmdns_sd-d2bc7df45f7006c8.rlib`, SHA-256 `732f64bc9cb3eae88671f2902dbcb845e3dd473205feea8f52418ef2bab21cc4`. The direct log artifact is `liblog-79610fd8c621607d.rlib`, SHA-256 `8d521a94243ae876206b907c3f8628fb268cc95f7c882c3ec030462eed1b8614`. The JSON records the exact Cargo example/dependency fingerprint match and every original file hash. No library rebuild or old binary execution was needed.

Existing native MSVC Rust 1.99.1 compiled each standalone helper with `-D warnings`, release optimization, panic abort, thin LTO and the original native dependency search paths. Both compiler invocations exited 0; one ran at a time with session 0, `BelowNormal` and jobs 2. No package build or Clippy validation was repeated.

| Helper | Compile duration | Source SHA-256 | Executable SHA-256 |
| --- | ---: | --- | --- |
| `startup_diagnostic.exe` | 10,173 ms | `577b8bf0f0045bd624e0393b714f5e1ae66a659d2749ace535b8a6fdb3624a6b` | `d60ab87455295bd68a26b37eba8c444c03c01026bdf5d50a76045b97f40a75b1` |
| `startup_diagnostic_v2.exe` | 10,091 ms | `d9c7a18a0339545657f303318c8583cb44ae7a0b5efd63dcc9b073f731d1956d` | `0649c8736946bc001be0cfb165b1c2788d80cf42c20e4ac3587e38599f51d52d` |

The two commands were sequential, each with a 120-second limit:

```text
<DIAGNOSTIC_FIXTURE>/bin/startup_diagnostic.exe startup <DIAGNOSTIC_FIXTURE>/runtime/run1 25090 25094
<DIAGNOSTIC_FIXTURE>/bin/startup_diagnostic_v2.exe startup <DIAGNOSTIC_FIXTURE>/runtime/run2 25092 25096
```

The first two preflight attempts failed before compilation: the first found both Clippy and linked-release example fingerprints; the second selected a Clippy metadata-only entry through its hash-suffixed `.d`. Their scripts, errors and hashes remain private and are summarized in the JSON. The third preflight selected the unique fingerprint whose recorded mdns/log dependencies resolve to actual frozen rlibs. Only the two successful standalone compiler invocations and two runtime runs followed. No unknown transport exit was converted to success.

Postflight confirmed all 2,579 old-fixture files retained their byte lengths, hashes and original read-only flags, including all 801 source files, all 1,679 artifacts and original evidence. The old fixture was accessed read-only. The new protected fixture allowed only the current user; helper sources and executables were frozen read-only. TEMP/TMP stayed inside it. No timeout or forced cleanup occurred. The final audit found no diagnostic or old-fixture process, assigned TCP listener/UDP endpoint, or private TEMP entry. Tool hashes, Rustup/default settings and existing Cargo config hashes stayed unchanged. The live sharing process kept its identity.

Private originals and the exact Windows locator are under `/tmp/oe-win-startup-diagnosis-20261023-kgI8j2`; `state.json` locates the Windows fixture and `private-evidence-sha256.json` indexes raw inputs, traces and failures. The downloaded raw tar hash is `9bf021173b8a7a056638feee9dec3b5231aad47cb42523cff770182ac7ffc352`. Copied reports redact personal path roots, host/user/SID and process IDs and normalize report text to UTF-8/LF. Original/copy hashes and transforms are recorded; helper and workflow source bytes remain exact.

Native input/clipboard, positive Windows migration, whole-cache/expiry/timer/query/record-byte bounds, slow DNS, original security/provenance/privilege and trusted distribution remain open. No later combined IPC or owned-worker observer acceptance is claimed. The sibling provider-stopped IPC Synergy3 task, original expiry/NSEC/orphan-record experiment and original provenance review were not continued or rerouted. Installed sharing/backend/state/TLS/default IPC/24800/5353, desktop/windowstation/TCC/login, services/tasks/elevation/accounts/credentials, firewall/network/global settings, toolchain defaults, Docker/publication/GitHub/remote refs were untouched. No unit tests, mocks, computer use, other-thread messages or child agents were used.
