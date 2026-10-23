# Native Windows discovery and virtual control verification

PASS on source `ac5c45081a9e060392cb850eddd625f07b591bfe` on 2026-10-22 (America/Vancouver). The isolated checkout started clean at that exact commit. This report covers one real loopback discovery run and one real CLI/named-pipe run with the virtual daemon. It does not clear the original release or Windows privilege findings.

The already-running parent build completed at 2026-10-22T20:56:24.424668-07:00. It was not restarted or duplicated. Native AMD64 MSVC used pinned Rust 1.99.1, two build jobs, `BelowNormal` priority and session 0. Strict release/all-targets Clippy passed for the eight selected packages; daemon/platform release builds and both discovery example builds passed. Build arguments and timings are in the [JSON evidence](windows-discovery-recovery.json).

Source identity and frozen binaries:

- Archive SHA-256: `4ff5283c9f1b1d41cc3c3358479ffcff09188f0a3a2f5652568e74b1e35a6707`.
- Manifest SHA-256: `a72bbfa46fe883d15e42444cdd220fb6aaf7887099eedc538db7dd68326e06f4`.
- Build script SHA-256: `f6827acd5c377282c66a147e6339307cd7c9d8e33629aa4cf28a78f62fb87aed`.
- All 572 Windows source files matched the manifest before and after execution and stayed read-only. The archive contains the same 572 regular files plus 86 directories.
- 571 archived files match Git blobs byte for byte. `tools/windows/openergy-run.cmd` has the CRLF checkout conversion required by `*.cmd text eol=crlf`: Git blob 502 bytes, archive 511 bytes. Git's clean filter produces the exact committed blob `9cd6f37ce26dde4edcad6f808492171f8daf3462`. The initial exact-byte assertion failure is retained; no source or archive was changed.
- Fresh copies under `<WINDOWS_FIXTURE>/bin-native-3e1f16ce` were made read-only. Both original and frozen copies retained these exact hashes:

```text
openergy.exe           4101632 bytes  c5951c2c1e1d92f9114ae66a214f709ccf9162e9a295c374c3b112490bff28dc
discovery_recovery.exe  928256 bytes  0b821581fdc085f82b01bc081d2878f9de1c5d07824e6416413d650fe9dc228d
discovery.exe          1567232 bytes  f0a44136eddc47316cbe22b1ec70c30e591de68dba807362507e3fd1babdf26c
```

Here, `<BIN>` is `<WINDOWS_FIXTURE>/bin-native-3e1f16ce`, and `<RUNTIME>` is `<WINDOWS_FIXTURE>/native-verification-3e1f16ce`.

The discovery command was `<BIN>/discovery_recovery.exe probe <RUNTIME>/discovery-runtime 25092 25094`. It exited 0 in 21343 ms, below its 120-second limit, with empty stderr. It used a separate real publisher process and custom loopback multicast. All eight original checks passed:

```text
CHECK initial-resolution=true
CHECK real-upstream-channel-overflow=true
CHECK snapshot-cap=true
CHECK fresh-advertisement-after-overflow=true
CHECK updated-port-and-untrusted-hint=true
CHECK goodbye-removes-all-peers=true
CHECK publisher-exits-zero=true
CHECK worker-shutdown=true
BURST_SERVICES=800 SNAPSHOT_PEAK=64
RECOVERY_OBSERVATION_MS=0
```

The raw calibrating receiver closed at its 1,000-event bound during the 800-service burst. The probe then observed fresh-peer recovery, TXT/port changes with the invalid fingerprint hint discarded, goodbye cleanup and worker shutdown. `RECOVERY_OBSERVATION_MS=0` is a post-phase observation after the publisher already waited two seconds. It is not a network recovery latency measurement. `discovery.exe` was frozen and hash-checked but was not executed.

The unchanged `tools/windows/verify-control.ps1` ran through PowerShell 7.6.4 against the frozen `openergy.exe`, with `-Port 25241`. It exited 0 in 5881 ms, below its 180-second limit, with empty stderr and `WINDOWS_CONTROL_PASS`. Its 23 assertions covered CLI status/connect/disconnect, private config changes, duplicate-daemon rejection, protected kernel pipe DACL with one current-user allow entry, event/log subscriptions, fragmented frames, typed unknown-method errors, rejection of client 65, slot recovery, shutdown with an incomplete frame, the absent-daemon error and parent-watched shutdown. Config changes, including clipboard flags, applied only to the virtual fixture.

Both runs used session 0, reduced priority, private config/TLS/IPC paths and `OPENERGY_ALLOW_PORT_24800=0`. Child `TEMP` and `TMP` pointed inside `<RUNTIME>/temp`, under the protected current-user-only fixture ACL. The control script disabled discovery and used only virtual backends. No process required forced cleanup. The final audit found no fixture executables, no TCP listeners or UDP endpoints on the assigned ports, and no remaining control temp labs. A separate endpoint audit used `-ErrorAction Stop` and nonempty OS inventories to confirm these counts. Source/archive/manifest/script hashes and all six original/frozen binary hashes were unchanged. The global Rust default stayed `stable-x86_64-pc-windows-msvc`; the source directory's pinned override stayed 1.99.1.

The first read-only postflight audit failed because its PowerShell array expression concatenated two file paths. Original `native-postflight.stderr` and the script are retained. `native-postflight-v2.ps1` only adds parentheses to those array entries. The corrected audit passed without repeating runtime checks or changing their assertions. Other local inspection/formatting failures are retained in private `helper-failures.json` and summarized in the JSON evidence.

Private raw reports are retained under `%LOCALAPPDATA%/Temp/oe-win-discovery-905e1537e742/native-verification-3e1f16ce` on Windows. The local private evidence directory is `windows-native-20261022` beneath the parent recovery evidence root from `/tmp/oe-discovery-recovery-state.json`. Its `raw/` contains `preflight.json`, both `*-result.json` reports, captured stdout/stderr, publisher log, `postflight.json` and `endpoint-audit.json`; `original-build/` contains the original build records and three build logs. `private-evidence-sha256.json` records local raw-file hashes. Personal paths, SIDs and PIDs are omitted here; report hashes are in the JSON evidence.

This is one bounded native run, not a sustained soak or an independent Windows baseline comparison. Native desktop input/clipboard, restricted-token acceptance, positive Windows migration and trusted signed distribution remain gates. GitHub private reporting is deferred. Installed software, live sharing, production config/TLS/default IPC, port 24800, services/tasks/login, accounts/credentials/desktop/windowstation, firewall/network settings, Docker, repository settings, signing/publication and remote refs were not changed. Only these two new evidence files are committed.
