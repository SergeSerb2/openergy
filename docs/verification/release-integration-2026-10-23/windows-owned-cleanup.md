# Native Windows owned discovery cleanup — 2026-10-23

**PASS: normal explicit shutdown and normal Drop both removed the actual Discovery-owned mDNS worker and all helper-owned UDP endpoints while the helper process remained alive.** Each case ran once. This accepts only these two normal resource-cleanup cases on exact library source `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`.

[Structured evidence](windows-owned-cleanup.json) records source/tool/library/binary hashes, exact commands, native query return codes, all snapshots and control transcripts, preserved workflow errors and cleanup. [Exact helper/native observer source](windows-owned-cleanup/source), [workflow](windows-owned-cleanup/workflow), and [sanitized case outputs](windows-owned-cleanup/outputs) are new supplementary evidence. No product or original probe source changed.

| Normal case | Verdict | Threads: baseline → live → cleaned | Owned UDP: baseline → live → cleaned | First observed absence after cleanup command |
| --- | --- | --- | --- | ---: |
| shutdown | PASS | 1 → 2 → 1 | 0 → 3 → 0 | 12.6786 ms |
| drop | PASS | 1 → 2 → 1 | 0 → 3 → 0 | 10.0609 ms |

The fixed cleanup deadline was 10,000 ms, with a 50 ms poll interval. Both cases met the condition in the first completed OS poll. The times include command delivery and observation cost; they are upper bounds from observations, not exact worker teardown latencies or a performance benchmark. Each helper then answered `ping`, passed a second OS absence snapshot, and exited normally with code 0 only after the observer sent `exit`.

## Why this observes the owned resources

The helper calls the real public `openergy_net::discovery::Discovery::new(Loopback, port)`, advertises one ordinary private service and polls until that service resolves through the same discovery object. It starts no daemon, native backend, TLS or production IPC. Its control commands are private stdin/stdout commands.

A `log::Log` callback at the existing frozen `mdns_sd::service_daemon` `exec_command` trace site, source line 4058, calls genuine Win32 `GetCurrentThreadId`. The observer matches that exact native ID to the helper's OS thread inventory. It differs from the genuine main thread ID and has the native description `mDNS_daemon`. The source names and runs that worker in `ServiceDaemon::daemon_thread`. The callback changes no worker source, command, state or scheduling rule. It uses no fabricated worker counter or private-field access.

The separate PowerShell observer refreshes the same process ID, executable path and start time before and after each resource snapshot. `System.Diagnostics.Process.Threads` must return a nonempty native inventory. The UDP observer calls `GetExtendedUdpTable` for IPv4 and IPv6 with `UDP_TABLE_OWNER_PID`. Every native data query returned 0. The combined global UDP inventory was nonempty in every snapshot, with 90–95 rows across the two cases. The initial size-query code 122 is expected buffer-size negotiation, not a successful empty table. Invalid lengths, native errors, or repeated buffer growth would fail the observation rather than become zero owned endpoints.

Both fresh baselines were taken before any Discovery initialization or network warm-up. Each had only the main thread and zero owned UDP endpoints. After initialization, the OS showed the main thread, the log-bound mDNS worker, two custom-port wildcard UDP bindings and the existing OS-assigned loopback command-wake endpoint. The worker was positively identified; the observer did not assume all process threads were mDNS workers. No extra persistent Winsock/framework thread appeared at the recorded snapshots. Short-lived threads between snapshots are not excluded.

For explicit shutdown, the helper retains `Some(Discovery)` after calling public `shutdown` and throughout the absence observation and later ping. Its destructor runs only after the later `exit` command, after resource cleanup has already been proved. For Drop, the helper drops the initialized object without first calling explicit shutdown. In both cases it remains alive, blocked on its later control command, with only its main thread left and no owned UDP endpoint.

The helper's `CLEANUP_RETURNED` line is only a timing boundary. Public shutdown success means submission, not a join. No acknowledgement or whole-process exit supplies either verdict. [Shutdown case](windows-owned-cleanup/outputs/shutdown-case.json) and [Drop case](windows-owned-cleanup/outputs/drop-case.json) retain the native before/after membership, nonempty-query controls and post-absence liveness proof.

## Exact inputs and commands

The parent snapshot `30f1c1cfc16d74a3dac2ee5d90bba391fb7c5bbf` has an empty product/lifecycle diff against frozen e4f48ba for crates, vendor, Cargo files and toolchain. The separate outcome-verifier change was not integrated or run.

No package rebuild was needed. The standalone helper links the original frozen `libopenergy_net.rlib`, SHA-256 `c36d0a889628e3bfd84a60818df38ea1e3bb3ff0e78f5fd87d53249f64ef03ff`. Its root Cargo fingerprint and hashed rlib were matched by exact file hash; the direct dependency fingerprints then selected:

- `libmdns_sd-97b021b8bc89fed1.rlib`: `db0a51cd4838e5451002577319b2a69153068d2b2c04a2040a296d5b31ba0a68`.
- `liblog-0d1942c19c6f2e24.rlib`: `1113b4f6e2a7565a8713619fed2aa771bc725c8bd051390bbbb9dc5cb58d50f5`.

The source archive SHA-256 remains `21a551059f2dbbd7fc694e668512efb99805b33029490f424156c92c762ccf08`. All 801 original project source files and 1,679 original artifacts matched their recorded hashes and read-only flags before use. Postflight verified all 2,579 files in that fixture and all 73 files in the prior startup-diagnostic fixture, including their evidence, with no byte/hash/read-only-flag change. Both old roots retained their protected current-user-only ACLs. The old fixtures were accessed read-only.

Existing native MSVC Rust 1.99.1 compiled the helper with `-D warnings`, release optimization, panic abort, thin LTO, one codegen unit and the exact frozen native/dependency search paths. It exited 0 in 8,347 ms. One compiler ran, with session 0, `BelowNormal` and `CARGO_BUILD_JOBS=2`. The C# OS observer compiled with warnings as errors and optimization; it exited 0 in 536 ms. Exact arguments, compiler/PowerShell/Roslyn hashes, OS API DLL hashes and unchanged installed MSVC tool inventory hashes are in the JSON.

Rust helper source SHA-256: `fb7bceb8ced3f9c3ecf8a3ab146b612ca42910ede1b4845fa2905ccdd311ef81`.
Native observer source SHA-256: `23d64792b1c619a6e846d4ee4f22e53de82b470dc3395102390b828ff222c67d`.

| Frozen artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `owned_cleanup.exe` | 845,824 | `bf73cb35fbd2ac298592646aba0749cc2d3bca0a584bdbc40c45246e3b7723b4` |
| `owned_cleanup.pdb` | 1,478,656 | `f0197088952b92b1ea631fa8018f5df411d366875825275540e427ca6ebc8c48` |
| `OwnedNativeObserver.dll` | 5,120 | `0885c68c7868cd811565ea96ac3703b0f3bd477a4bbfa197f5d7cdf51c74a08e` |

The observer ran these commands sequentially against the frozen helper:

```text
<OWNED_FIXTURE>/bin/owned_cleanup.exe shutdown 25090 25094
<OWNED_FIXTURE>/bin/owned_cleanup.exe drop 25092 25096
```

For each: `READY` → OS baseline → `init` → real publication resolves → live OS worker/socket snapshot → `cleanup` → bounded native absence poll → `ping` → second absence snapshot while alive → `exit` → normal exit 0. Only custom loopback traffic on the assigned ports and the library's existing OS-assigned command-wake socket was used. No 5353 or 24800 endpoint was used.

## Preserved errors and final cleanup

The initial build/report SSH transport timed out after 90 seconds, with unknown remote exit. Rust and C# compiler exit-0 results were already durable, and their binaries were read-only. The remaining generic FileInfo metadata serialization/freeze-report stage had not produced its output; the exact transport cause is not claimed as proved. The original build script and raw transport failure remain intact. A metadata-only finalizer verified the same sources and binaries and wrote explicit file records. No compilation was repeated.

An audit of the timed-out private build process then hit a CIM-to-`Get-Process` race: the identified process was already gone when it was opened. That audit exited 1 before its Kill call; its local wrapper also failed to parse the empty stdout as JSON. Both errors and hashes are retained. A later exact-command process audit confirmed zero remaining original build processes. No forced stop was confirmed. Neither normal lifecycle case, native resource query, optional thread metadata query nor helper exit failed, and neither case was retried.

Final audit at 11:02:03 America/Vancouver found no new or old fixture executable, no assigned TCP listener/UDP endpoint and empty private TEMP. The new fixture and 44 audited items allowed only the current user, with no reparse point. Helper/workflow sources and all three new binary artifacts were read-only. Tool hashes, Rustup settings/default toolchain and existing Cargo config hashes stayed unchanged. The original live sharing process kept its process identity; no desktop sharing behavior was exercised.

Raw originals, private native IDs, exact paths and private command inputs are retained under `/tmp/oe-win-owned-cleanup-20261023-6tnNR7`; `state.json` locates Windows and `private-evidence-sha256.json` indexes them. The downloaded raw bundle hash is `f8e10d8047045c5794f22099c520b835443b600e55cd18d4e7193afbdafd784e`. Public copies use stable process/thread role aliases, alias native start addresses and the command-wake port, redact personal paths/host/user/SID and nested private command encodings, and normalize report text to UTF-8/LF. Original/copy hashes and transforms are recorded. Helper and workflow source bytes remain exact.

Only these two normal Windows owned-worker/socket cleanup cases are accepted. No resource-exhaustion, stress, abnormal cleanup or soak case ran. Startup 11/14 remains failed pending the separate verifier repair; retained 19/19 and control 23/23 are preserved and were not rerun. Whole-cache/expiry/timer/query/record-byte bounds, slow DNS, native input/clipboard, positive Windows migration and original security/provenance/privilege/trusted distribution remain open. The stopped IPC Synergy3 task, original expiry/NSEC/orphan-record experiment and blocked provenance review were not continued or rerouted. No installed/live sharing/backend/state/config/TLS/default IPC, desktop/windowstation/TCC/login/services/tasks/elevation/accounts/credentials, firewall/network/global limits/toolchain defaults, Docker/signing/publication/GitHub/remote refs changed. No unit tests, mocks, computer use, other-thread messages or agents were used. GitHub private reporting remains deferred.
