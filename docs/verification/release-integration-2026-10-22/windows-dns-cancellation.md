# Native Windows DNS cancellation and connection readiness verification

## Later supplementary diagnosis

The [separate supplementary run](windows-dns-supplement.md) measures the OS
refusal window and exercises the driver refusal and later lifecycle contracts
independently. It preserves both original exit-1 results in this record and does not
grant overall Windows acceptance.

FAIL for the unchanged full and refusal-only probes on source `07adb0a3cdc4cff1aa77ce31dbe078a48db56bd5`, on 2026-10-22 (America/Vancouver). Both exit 1 at the direct OS refusal prerequisite. The eight queue batches and their frame checks pass. This is a partial native result, not an overall Windows pass.

The checkout started clean at that exact commit. The Git archive and a per-file manifest were transferred through the existing `sergecode-windows` key-auth alias with `BatchMode=yes` and `ConnectTimeout=8`. A new protected current-user-only fixture was created under the current user's temporary directory. Source, target, frozen binaries, runtime, TEMP and TMP stayed inside it. Previous frozen fixtures were not written. Execution used native AMD64 MSVC in session 0 with two build jobs and `BelowNormal` priority.

The prior [Windows discovery/control report](windows-discovery-recovery.md) covers `ac5c45081a9e060392cb850eddd625f07b591bfe`. That build predates the TCP changes and was not reused or relabeled. The parent's Mac probe and source reviews are supplied context, not independently reproduced results in this report.

Source identity:

- Archive SHA-256: `23dfa9257d79c286cda94bf1381b60981ddb082a8edb24c0ba1ed54bc06c203f`.
- Transferred per-file manifest SHA-256: `e3f31a782b3165c049ff7cbc360da894f599c7cd51b1cf68b9dc007bd1681e3c`.
- Unchanged example `crates/net/examples/dns_queue_cancel_c4806a3d.rs` SHA-256: `a7dbab22249c7c65443c6943d04ab3c078d9d3f02a97016f5596fd0f6e6d4a2f`.
- All 658 archived regular files match the manifest. Of these, 657 match Git blobs byte for byte. `tools/windows/openergy-run.cmd` has only the expected CRLF conversion under `*.cmd text eol=crlf`: 502 Git bytes become 511 archive bytes. `git hash-object --stdin --path=tools/windows/openergy-run.cmd` cleans the archive bytes to the exact committed blob `9cd6f37ce26dde4edcad6f808492171f8daf3462`.
- All 658 Windows source files matched before build, after build and after both probes. They remained read-only. There were no runtime source edits or changes to classification or timing checks.

Pinned Rust was `rustc 1.99.1 (58cf8f9c3 2026-10-13)` and Cargo was `cargo 1.99.1 (5f94df478 2026-08-27)`. The existing toolchain binaries were selected through process-local environment variables and PATH. No toolchain was installed or made the global default. The global default remained `stable-x86_64-pc-windows-msvc`.

All three fresh build steps passed with exit 0:

- Strict release Clippy: 30,729 ms, `--locked --release --target x86_64-pc-windows-msvc --jobs 2 --all-targets -- -D warnings` for `openergy-daemon`, `openergy-platform-windows`, `openergy-net`, `openergy-session`, `openergy-platform`, `openergy-protocol`, `openergy-config` and `openergy-ipc`.
- Release build of those eight packages: 85,097 ms.
- Release build of `openergy-net --example dns_queue_cancel_c4806a3d`: 34,844 ms.

The [JSON evidence](windows-dns-cancellation.json) contains exact argument arrays, timestamps, build logs, tool hashes, seven library artifact hashes and original/frozen executable hashes. `openergy.exe` was built and frozen but never executed. Both copies of each executable kept these hashes:

```text
openergy.exe                      4102656 bytes e0e88ac730943d63eb235d4bc2885152c379f0be03c0389e013db868eaf1ce38
dns_queue_cancel_c4806a3d.exe       2392064 bytes 4a1e7cb4293e94d627308a4bab01e97962fadfb1d78169b2507ad57074cf4127
```

The unchanged example ran once in each mode, against two new absolute private runtime directories:

```text
<WINDOWS_FIXTURE>/bin/dns_queue_cancel_c4806a3d.exe <WINDOWS_FIXTURE>/runtime/full
<WINDOWS_FIXTURE>/bin/dns_queue_cancel_c4806a3d.exe <WINDOWS_FIXTURE>/runtime/refusal-only --refusal-only
```

Full mode ran from 21:36:30 to 21:36:49 local time, exited 1 in 19,316 ms and did not time out. It passed all eight 16-job batches: two Shutdown batches, two DisconnectAll batches and four one-nanosecond deadline batches. Each batch rejected the 17th pending connection, retained exactly one obsolete-job failure per port, emitted no extra late failures, accepted no stale TCP connections and completed a real frame round trip to `127.0.0.1:25116`. The four cancellation batches each observed the expected DNS-budget rejection before completion accounting released capacity. There were 128 obsolete-job failures and eight successful frame round trips.

The example checked exact outbound bytes `0000000450415353` (length 4 plus `PASS`) and a real inbound `NetEvent::Frame` carrying `BACK` after sending `000000044241434b`. Healthy connection times were 2,020,416–2,053,729 microseconds. Frame round trips were 2,020,565–2,053,825 microseconds, within the unchanged three-second deadline. The real embedded-NUL hostname case then produced one typed DNS failure.

The first refusal control failed before the NetDriver refusal matrix. Both modes retained this exact stderr:

```text
dns_queue_cancel_c4806a3d: expected direct OS refusal at 127.0.0.1:25117: Err(Error { kind: TimedOut, message: "connection timed out" })
```

The source binds and releases the candidate address, then requires `std::io::ErrorKind::ConnectionRefused` from a direct `TcpStream::connect_timeout` with a 500 ms bound. Windows instead returned `TimedOut`. Refusal-only mode ran from 21:36:49 to 21:36:50 local time and exited 1 in 578 ms with the same first failure. Neither mode reached the ten typed NetDriver refusal checks, the direct IPv6 refusal control or the second refusal port. This native prerequisite/timing mismatch does not establish a wrong NetDriver refusal classification. No timeout or classification was weakened, and neither mode was rerun after failure.

Both unchanged probe runs logged actual Rust localhost ordering as `[[::1]:25117, 127.0.0.1:25117]`. The independent OS preflight reported the same IPv6-first order. The source preserves resolver order. All eight healthy localhost connections reached IPv4-only listeners and exchanged frames. This supports that IPv6-to-IPv4 fallback was exercised; it is an inference from the real run, not a trace of each queue lookup and socket attempt. No hosts, DNS, resolver or network setting was changed to force ordering.

IPv6 was not a missing prerequisite: Windows reported IPv6 support and a `::1:25119` listener bind succeeded and was released during preflight. Positive IPv6 NetDriver frames remain unverified because full mode stopped first. The later inbound listener, original handle/waker, reconfigure, invalid-reconfigure recovery and driver-drop contracts also remain unverified. The final `RESULT` marker was not reached.

The first read-only tool query failed with SSH exit 1 and PowerShell `ExpectedExpression` because the remote shell expanded a quoted command. The corrected query used EncodedCommand. The first fixture setup transport timed out after 30 seconds; the local helper exited 1 and the remote exit is unknown. Recovery found exactly one new protected fixture with empty source/target and no durable preflight result, so it was reused. The replacement read-only preflight passed. The first detached build launch returned SSH exit 0, but its process disappeared without source extraction, target files, logs or a build result; its build exit is unknown. The replacement kept SSH attached, completed one actual build and returned exit 0. These helper failures, the first runtime failures, exact exits and raw hashes remain in the JSON/private evidence. No build was duplicated while active.

Postflight passed at 21:37:03 local time. All source and executable hashes were unchanged. The protected fixture and all 2,743 audited items had only the current-user allow entry. TEMP was empty. A fixture-path-scoped process query found no remaining fixture executables. Successful, nonempty OS endpoint inventories found no TCP listeners or UDP endpoints on 25100–25119. Both probe processes exited on their own; no forced cleanup or unrelated process termination was used. The private source, binaries, scripts, logs and original errors remain available under `<WINDOWS_FIXTURE>`; the downloaded private raw bundle has SHA-256 `aa5fe95529819521eaac6723497716d03c4a28abce0c1330ef88a3782ac61344`.

Active OS DNS timing, cancellation of an executing OS lookup and deterministic slow-DNS gates remain unverified. The one-nanosecond deadline batches are not slow-DNS measurements. The provider-stopped cache-expiry crafted test and original release-provenance review were not rerouted or run. This result cannot clear native input/clipboard, positive Windows migration, privilege/provenance or trusted distribution gates.

No installed/native daemon/backend, live sharing, production config/TLS/credentials, default IPC, port 24800, mDNS 5353, desktop/windowstation/TCC/login, accounts, services, tasks, elevation, firewall, network settings, toolchain defaults, previous frozen fixtures, Docker, signing/publication, GitHub posts, push/merge or other-agent messages were changed. No unit tests or mocks ran. Only these scoped evidence files are committed.
