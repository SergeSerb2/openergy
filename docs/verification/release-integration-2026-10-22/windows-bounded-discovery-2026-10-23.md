# Native Windows bounded discovery — 2026-10-23

**FAIL: cancelled-startup acceptance on exact clean parent `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`.** The unchanged startup probe passed 11/14 checks. Retained discovery passed 19/19, and virtual control passed 23/23. The later passes do not clear the startup failure. No product source, probe assertion, capacity or timing changed. Each mode ran once.

[Structured evidence](windows-bounded-discovery-2026-10-23.json) includes exact command arguments, UTC and America/Vancouver timestamps, tool and binary hashes, all check results, original failure records and private raw-file hashes. Source manifest covers all 801 Git archive files. Artifact manifest covers all 1,679 built and frozen artifact files. Archived workflow records the diagnostic scripts, including the first failed versions.

| Native mode | Passing checks | Exit | Wrapper duration |
| --- | ---: | ---: | ---: |
| startup | 11/14 | 1 | 16,720 ms |
| retained | 19/19 | 0 | 41,471 ms |
| control | 23/23 | 0 | 6,196 ms |

## Startup failure

The real publisher retained 800 ordinary advertisements. The probe queued the unchanged 16 replay commands and 32 distinct cancelled limited starts, then tested the existing limited and standard subscription identities. Its exact observation was:

```text
CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows=16 before=1 after_barrier=0 after_settle=0 live_closed=true
CHECK cancelled-distinct-types-retain-no-state=false
CHECK cancelled-replacements-preserve-identity=false
CHECK surviving-handle-receives-live-update=false
```

The previous live limited subscription closed. Its actual worker-state gauge fell from 1 to 0 instead of retaining the one existing state. This is state loss; this run did not observe 32 orphan states. The other 11 startup checks passed, including the later successful replacements, final stop cleanup, publisher exit and worker acknowledgement. The probe exited 1 with `discovery_recovery: limited startup checks failed`. Full stdout and stderr are copied from the retained originals.

This one unchanged native run blocks the startup gate. It does not establish whether the cause is product behavior or a scheduling assumption in the probe. No further diagnosis, retry or product fix was attempted. Earlier macOS startup success remains a separate observation on its recorded source.

## Retained delivery and virtual control

The retained flow used the separate ordinary publisher child and custom loopback multicast. All 19 checks passed while the 800-service namespace stayed registered. Limited replay remained open at 64 selected entries. Selected-peer updates and goodbyes stayed visible, surviving selections stayed stable, paused updates coalesced to the latest view, and paused removal replaced the view. The full output records `SNAPSHOTS count=13 peak=64 too_close=false`, ordinary upstream overflow at 1,000 events, both retained standard replays disconnected at 1,000 drained events, and later fresh-peer recovery. Full retained output.

The unchanged `tools/windows/verify-control.ps1`, SHA-256 `264d6a71e54de650498f6dcf46b806acaf5cc20f2022222082372cf614ab8aeb`, ran through existing PowerShell 7.6.4 against the frozen virtual daemon on 25241 with discovery disabled. All 23 assertions passed and printed `WINDOWS_CONTROL_PASS`. They cover real CLI/named-pipe commands, private config, duplicate daemon refusal, protected current-user-only pipe DACL, subscriptions, fragmented frames, typed unknown-method failure, rejection of client 65, slot recovery, incomplete-frame shutdown, absent-daemon error and parent-watched shutdown. Config and clipboard flags applied only to the virtual fixture. Full control output.

This control pass covers the old IPC implementation at this exact parent. It does not check the parent's separate later IPC fix. That combined source will require new acceptance.

## Exact source, builds and frozen inputs

The T3-isolated worktree started clean at the requested full parent. One new current-user-only ACL-protected TEMP fixture held fresh `source`, `target`, `bin`, `runtime`, `temp` and `logs` directories. No previous source or binary fixture was reused or written. Child TEMP and TMP stayed under the new fixture. Existing key-auth SSH used `BatchMode=yes` and `ConnectTimeout=8`; PowerShell commands used UTF-16LE EncodedCommand or protected script paths.

`git archive` SHA-256 is `21a551059f2dbbd7fc694e668512efb99805b33029490f424156c92c762ccf08`. The per-file source manifest SHA-256 is `53cf3242666110a06275cd01eb5b01495bf7be65a48ff3066da08bdd8682e556`. All 801 regular files were matched against the exact Git tree. Of these, 800 match clean blobs byte for byte. `tools/windows/openergy-run.cmd` has the required CRLF checkout conversion: 502-byte Git blob, 511-byte archive file, exact Git clean-filter blob `9cd6f37ce26dde4edcad6f808492171f8daf3462`. The check was not weakened. All extracted source files matched before the build, after the build and after runtime, and stayed read-only.

The native AMD64 MSVC build used existing Rust 1.99.1, `--locked --offline --release --target x86_64-pc-windows-msvc --jobs 2`, session 0 and `BelowNormal` priority. Before compilation, no cargo/rustc/Clippy build process or assigned endpoint was present. Observed cargo/rustc processes also used session 0 and `BelowNormal`. Three cargo steps ran sequentially and each exited 0:

- Strict `clippy --all-targets -- -D warnings` for net/config/session/protocol/ipc/platform/platform-windows/daemon: 32,657 ms.
- Release build for those same eight packages: 64,952 ms.
- Release build of `discovery_recovery` and `discovery`: 34,483 ms.

`discovery.exe` was frozen and checked, but not executed. The macOS owned-worker observer was not built as an acceptance executable or run. All-target Clippy only checks each target's Windows configuration.

| Frozen binary | Bytes | SHA-256 |
| --- | ---: | --- |
| `discovery_recovery.exe` | 1,001,472 | `3bf1ae71aefc054a654fd362081c9d6055ca78dfb2072c0301c99f41b5b38f7f` |
| `discovery.exe` | 1,589,248 | `ee74706c01147f095f03b4869c3e3ea4cb8d8fee0b92e4cd2b720f3c32a3195a` |
| `openergy.exe` | 4,124,672 | `f1fada70f322a29df2d7a05762f456ca378dcad380ede927dd72a0d7e5768b4b` |

Seven selected project rlibs and both observed mdns-sd rlib variants were copied into the private frozen library directory. All original/frozen binary and library pairs have matching sizes and hashes. Every file in target and bin was made read-only. All 1,679 files retained their exact sizes and hashes after runtime. The JSON includes the nine library hashes, Cargo fingerprints, pinned Rust tool hashes, installed native-tool inventory hashes and exact script hashes. Pinned Rust and defaults were captured before cargo; the supplemental native-tool snapshot began during the release build and was rechecked after runtime. The extra cache-checksum scan returned zero entries and does not grant cache provenance acceptance.

## Commands, first failures and cleanup

Here `<BIN>` means `<WINDOWS_FIXTURE>/bin`; `<RUNTIME>` means `<WINDOWS_FIXTURE>/runtime`. These three runs were sequential and each had a 120-second limit:

```text
<BIN>/discovery_recovery.exe startup <RUNTIME>/startup 25090 25094
<BIN>/discovery_recovery.exe probe <RUNTIME>/retained 25092 25096
pwsh.exe -NoLogo -NoProfile -NonInteractive -File <WINDOWS_FIXTURE>/source/tools/windows/verify-control.ps1 -BinaryPath <BIN>/openergy.exe -Port 25241
```

The first build helper exited 1 after all cargo steps passed because its freeze inventory expected one mdns-sd rlib and found two. The original script and stderr are retained. `freeze-after-build.ps1` only inventories and freezes the actual artifacts; it does not rerun cargo. The first runtime helper had a missing closing brace and exited 1 before any probe started. `run-v2.ps1` adds that one brace. PowerShell parser checks passed, and the absent runtime-start record was checked before the first real launch. These helper failures remain separate from the real startup exit-1 result. All transport exits were observed; none is recorded as unknown. The aggregate runtime helper also exited 1 to preserve the startup failure.

No runtime timed out. No forced cleanup was needed. Before and after each mode, and at final retention audit, the private executable and assigned-endpoint inventories were empty. Final audit at 09:49:29 America/Vancouver confirmed no fixture executable, no TCP listener or UDP endpoint on 25080–25099 or 25241, and empty private TEMP. The protected fixture had 2,982 current-user-only ACL items and no reparse point at that snapshot. The original live sharing process retained its process identity. Pinned tool hashes, Rustup settings, global toolchain defaults and existing Cargo config hashes stayed unchanged.

Private raw originals, the exact source archive, sources, target artifacts, frozen binaries/libraries and all failures remain on Windows under `%LOCALAPPDATA%/Temp/oe-win-bounded-e4f48ba-<RANDOM>`. The exact locator and downloaded originals are under private local `/tmp/oe-win-bounded-20261023-MI4BOy`; `state.json` locates Windows and `private-evidence-sha256.json` indexes the originals. The raw evidence tar SHA-256 is `269cc9d05b5b9c01b6a4e110b3887cbd54604a6841d9478d5d06608d128ee7ad` and was checked after download. Original bytes were not sanitized or overwritten. Copied reports replace personal path roots, hostname, username and SID with documented placeholders, omit process IDs/unrelated machine inventories, and normalize text to UTF-8/LF. The JSON records original and copied hashes and the exact transforms. Archived scripts and the source manifest keep their original bytes. The copied artifact manifest uses repository LF newlines; its original Windows CRLF hash and copied LF hash are both recorded. The staging warning is retained in the private originals.

## Acceptance limits

Startup acceptance remains **FAIL**. Retained delivery and virtual control passed only within the scope above. Final process/endpoint absence does not observe owned worker/socket cleanup while the process stays alive. Submission and sentinel acknowledgement do not replace the separate macOS observer acceptance.

The delivery-state cap of 64 does not bound the whole cache, expiry, timers, query work or variable bytes. No automatic cached-service refill is guaranteed. `RECOVERY_OBSERVATION_MS=0` is a post-phase observation after publisher waits, not network latency. These are bounded probe durations, not a soak or performance benchmark.

Native input/clipboard, positive interactive Windows migration, restricted privilege, the original provenance/security review, trusted signed distribution, whole-cache/expiry/timer/query/byte bounds and slow DNS remain open. The forbidden provider-stopped expiry/NSEC/orphan-record experiment was not run or redirected. DNS builds and refusal supplements were not rerun or relabelled. Installed sharing/backends, production config/TLS/default IPC and 24800/5353, desktop/windowstation/TCC/login, services/tasks/elevation/accounts/credentials, firewall/network/global resources, Docker, signing/publication, GitHub posts and remote refs were untouched. No unit tests, mocks, computer use, other-thread messages or subagents were used. GitHub private reporting remains deferred.
