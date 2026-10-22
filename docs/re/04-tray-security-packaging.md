# Openergy RE 04 — tray, security, diagnostics, and macOS packaging

Target: `/Applications/Synergy.app`, **3.7.0-beta**, arm64. Observed on **2026-10-21**, macOS **27.2**, America/Vancouver (UTC−07:00).

This is a clean-room behavior report. **Verified static** means the claim follows from installed bytes, metadata, or source supplied by the user. **Verified passive** means the claim follows from an existing process, file, or log. **Inference** and **hypothesis** identify conclusions that still need a controlled experiment. No Synergy executable was launched, attached to, signalled, restarted, or driven through its UI. No input was injected. No connections to peer computers or external services were made.

## Evidence and method

The working directory is `/tmp/openergy-re/aux/`. The four target executables, relevant static metadata, updater executables, and the three requested workaround source files were copied there. Configuration values, TLS material, identities, and raw user logs were not copied into this report. Log analysis retained only safe category names, counts, timestamps, and line references.

Evidence references used below:

| Reference | Source and reproducible observation |
| --- | --- |
| **T** | Copy `synergy-tray`; `otool -L/-l/-tvV/-ov`, `nm -nm`, `strings -a`; retained as `synergy-tray.{libs,load,disassembly,objc,symbols,strings}.txt`. Addresses below are unslid Mach-O virtual addresses, image base `0x100000000`. |
| **S** | Copy `synergy-security`; equivalent static outputs; REA/Ghidra `security-check.json`, function `0x100001340`. |
| **D** | Copy `synergy-diagnostics`; equivalent static outputs; REA/Ghidra `diagnostics-entry-task.json` for the logger/setup entry `0x10000e9dc`, which calls collector `0x10000f90c`. Literal offsets below are file offsets, not runtime addresses. |
| **P** | Original bundle `Info.plist`, helper and framework plists; `codesign -dvv --entitlements -`; safe output retained in `signing-summary.json`; original bundle strict signature verification. |
| **L** | `~/Library/Logs/Synergy/synergy.jsonl`, **first 733 records only**, ending `2026-10-21T20:58:41.549Z`; `log-effective-categories.json`, `log-categories.json`, `startup-events.json`; the two adjacent launch-agent `.log` files. Line numbers refer to this fixed prefix. Later live additions are excluded. |
| **R** | Passive `ps -axo pid,ppid,rss,%cpu,etime,command`, `lsof`, `launchctl print`, file inventories, and byte counts; `process-snapshot.json`, `process-samples.json`, `footprint.json`, `tray-stdio-types.txt`. Retained process summaries omit command arguments and endpoint addresses. |
| **F** | Copied `README.md`, `SynergyFocusFix.swift`, `SynergyFocusFix-v2.swift`; source names below identify functions directly. The originals are in `~/Documents/Codex/2026-06-29/the/outputs/SynergyFocusFix/`. |
| **SDK** | Installed Apple `MacOSX27.0.sdk` headers: CoreGraphics `CGEvent.h`, `CGEventTypes.h`, `CGEventSource.h`; HIServices `AXUIElement.h`; AppKit `NSRunningApplication.h`; `sys/sysctl.h`. Reference documentation was read locally. |

**REA use.** Ran `npm_config_offline=true npx -y rea-agents@latest --help` and `doctor`: cached REA **6.7.0**, Node **22.23.1**. Doctor initially reported no configured native provider. Ghidra **12.1.4** was available under `/opt/homebrew/Cellar/ghidra/12.1.4/libexec`; it was selected with per-command environment variables, without running setup or changing agent configuration. Used `inspect-macho`, `inspect-artifact`, `inspect-native-dispatch-metadata`, `decode-interface-builder`, `inspect-asset-catalog`, and static `function` analysis. No capture, debugging, instrumentation, or native UI scenario tool was used.

REA decoded Electron's copied `MainMenu.nib` into **13 objects and one connection**. It contains the generic Electron `AtomApplication` setup, not the Synergy tray menu. No compiled `Assets.car` was found in the original bundle inventory or the copied metadata subset. The tray embeds its PNGs in the executable. Dispatch inspection found no statically declared Objective-C or Swift classes in the Rust tray; this does not exclude classes registered at runtime. The native metadata reader also reported incomplete legacy dyld fixup coverage. [T, P; `tray-macho.json`, `security-artifact.json`, `tray-dispatch.json`, `electron-nib.json`, `asset-catalog.json`]

## A. synergy-tray

### Implementation and menu

**Verified static:** this is a Rust executable using `tao`, `tray-icon`, `muda`, `objc2-app-kit`, and AppKit. Embedded source paths identify `apps/tray/src/main.rs`, `tray/menu.rs`, and `listeners/{menu,stdin}.rs`. AppKit imports include status-bar and menu selectors. It does not link Electron or Qt. The linked-framework list and undefined-symbol inventory show no socket/connect networking API used by the tray. [T: imports; source-path strings]

The exact menu labels recovered from construction and update calls are:

| Construction order | Exact title / item | Internal identifier | Action sent to parent |
| --- | --- | --- | --- |
| 1 | `Show config` | `show_config` | `CONFIG` plus newline |
| 2 | `Resume` when paused; `Pause` when running | `start_stop` | `START` for Resume; `STOP` for Pause, plus newline |
| 3 | `Restart` | `restart` | `RESTART` plus newline |
| 4 | Predefined separator | — | — |
| 5 | `Exit` | `quit` | `EXIT` plus newline |

The constructor creates the four named items and inserts one predefined item before Exit. The separator selector is present in the AppKit selector set. No additional Synergy-specific menu titles or submenu construction were identified in this path. The tooltip text is `Synergy 3`. These are static findings; the live menu was not opened. [T: constructor `0x100007ac0–0x100007bd0`; literal file offsets `0xf8c42` (Show config), `0x121b8e` (Resume), `0x120fa6` (Pause), `0xf8c62` (Restart), `0x123f78` (Exit); `separatorItem`; tooltip construction near `0x100007cf4`]

### IPC and status icons

**Verified static:** the parent communicates with a newline-delimited standard-input listener. The listener strips LF and optional CR, recognizes three exact case-sensitive commands, and forwards an event to the tray's main event loop. Unknown input reaches an `unrecognized event` diagnostic. EOF or a read failure also sends an exit event. [T: stdin listener `0x10000a844` onward, particularly `0x10000abac–0x10000accc`, line-end handling `0x10000ac7c–0x10000acb4`, exit path `0x10000ae30`]

| Parent → tray | Visible result |
| --- | --- |
| `SHOW_START\n` | Set the action title to `Resume`; install the paused icon. |
| `SHOW_STOP\n` | Set the action title to `Pause`; install the running icon. |
| `EXIT\n` | Exit the tray event loop. |

The two embedded images are **64×64 PNGs**: paused image at file offset `0xf7877`, 2,577 bytes; running image at `0xf8288`, 2,297 bytes. Visual inspection of the extracted copies shows a circular Synergy mark with central pause bars in the first image, and the running mark without those bars in the second. The paused branch uses the first image and the running branch uses the second. There is no separate discovered connected/disconnected icon command: the tray learns the parent's start/stop control state, rather than independently measuring peer connectivity. [T: state branches `0x100008378–0x100008404`, `0x100008500–0x100008570`; extracted `tray-embedded-{1,2}.png`]

**Verified passive:** the tray's parent PID is the live service PID. Descriptors 0, 1, and 2 are connected Unix descriptors (`lsof` reports `tunix`), consistent with subprocess standard streams implemented with socket pairs. The tray has no observed Internet socket. This is not a tray-owned HTTP, WebSocket, or named Unix-socket protocol. Descriptor payloads were not read. [R: process hierarchy; `tray-stdio-types.txt`; passive socket inventory]

### Actions and opening the UI

The tray emits `READY\n` when ready, `CONFIG\n` for configuration requests, and `EXITED\n` on its event-loop exit path. Its left-click event branch also emits `CONFIG\n`. The start/stop menu handler reads the current action title: `Pause` produces `STOP`, otherwise it produces `START`. Restart and Exit emit the distinct messages listed above. [T: `0x100008364`, `0x100008430`, `0x100008590`, `0x100008618–0x1000086c0`; constants near file offsets `0xf8bdb`, `0xf8be1`, `0xf8be8`, `0xf8c38`, `0xf8c3d`, `0x123a48`]

**Verified boundary; inferred service behavior:** this tray requests the UI through `CONFIG`; no direct Synergy UI launch call or executable path was found in its action handler. The parent must decide how to open or reveal the Electron UI. The exact service-side spawn/reuse mechanism belongs to the service investigation and is not established here. Similarly, `Exit` requests parent action; it does not prove that the service, core, and tray have all stopped. [T: action handler; R: parent relationship]

## B. synergy-security

**Verified static:** this is a small Rust **Accessibility trust probe**, not a certificate generator or Keychain helper. Its application logic calls **`AXIsProcessTrusted()`**, without options. The complete recovered function has no argument parsing. The entry-point path reaches that function through the Rust runtime. [S: REA/Ghidra `FUN_100001340`; trust call `0x1000013c4`; Mach-O entry offset `0x17d0`]

The exact normal-result contract is:

| AX result | Standard output | Process exit status |
| --- | --- | --- |
| Trusted | `authorized\n` | `0` |
| Not trusted | `denied\n` | `1` |

The strings are at file offsets `0x321ff` and `0x321f8`. The branches print the result and enter the runtime exit routine (`0x10002abc0`, eventually `_exit`). This is plain text and exit status, not JSON. It is the contract available to the service; the exact service-side interpretation of output versus exit status was not inspected in this scope. [S: `0x1000013c4–0x1000013f8`; print routine `0x10002bb7c`; exit chain `0x10002abc0–0x10002ac44`]

No import was found for `AXIsProcessTrustedWithOptions`, `CGPreflightListenEventAccess`, `CGRequestListenEventAccess`, `CGPreflightPostEventAccess`, `IOHIDCheckAccess`, Screen Recording preflight/request APIs, or Keychain APIs. There is no dynamic-library lookup API in the undefined imports that would suggest a second, late-loaded permission implementation. **Conclusion for this executable:** it neither requests those permissions nor distinguishes them. AX trust is a check of the current process, as defined by Apple's header; it is not a direct permission query for an arbitrary other PID. The service's use of this result as a core/bundle permission indicator requires separate responsibility-chain analysis. [S: imports and full application function; SDK: `AXUIElement.h:55–74`]

`AXIsProcessTrusted()` has no prompt option. No help/version CLI branch was found, so the copy was **not run**, even with `--help` or `--version`. A denied trust check would still run its normal behavior with those arguments. The log's startup message `no os access permission, skipping core config` supports the existence of permission gating, but does not establish the helper's current result or the core's full TCC authorization. [S; L: line 9]

## C. synergy-diagnostics

**Verified static:** this is a Rust support-report collector, with embedded tool-version text **`0.1.0`**. It assembles a ZIP report, gathers installation/service checks, reads local settings and logs, and includes system and recent crash information. Embedded source paths include `diagnostic/{binaries,ping,db,report}.rs` and `macos/diagnostics/service.rs`. ZIP, HTTP/TLS, system-information, and process-launch support are linked into the executable. [D: imports; file offsets `0x289fb2` onward; source-path strings]

The shipped archive manifest specifies:

| Archive area | Collected material |
| --- | --- |
| `config/` | `db.json`, `local.json`, `synergy.conf`, `synergy-core.conf`, `synergyCert.pem`, `trusted-servers`, `UISetting.json`, and the system login-agent plist when available. |
| `logs/` | Merged `synergy-unified-<unixtime>.jsonl`, a human-readable merged `.log`, and per-source logs. Listed sources include service, core, tray, security, GUI backend, and GUI frontend. |
| `system/` | `diagnostic.txt`, `versions.json`, and `system-info.txt` for installation checks, versions, OS/kernel/architecture/session details. |
| `crash-report/` | Recent crash reports, when present. |
| Root | A `README.txt` describing the archive and its sensitivity. |

These are manifest/collector intentions; no report was generated to prove every optional file's inclusion on this machine. The manifest says TLS private-key material is removed while the certificate remains. Static PEM-boundary strings support a filtering step, but the exact filter and treatment of all other sensitive settings were not validated. In particular, retaining configuration, trusted-server material, logs, and crash reports does **not** make an archive safe for public sharing. No real certificate, fingerprint, license, or report content is reproduced here. [D: embedded archive paths around `0x28a158`; report manifest starts around `0x28a22e`; PEM-boundary markers near `0x28a820`]

The collector has explicit loopback targets **`http://127.0.0.1:24802/ping`** and **`http://127.0.0.1:24803/v1/settings`**. It looks for the settings-response fragment `"permissionStatus":"Authorized"` and reports an authorization problem otherwise. It also contains `launchctl` and service-plist/PID failure messages. Thus it can consult the running service and inspect launchd state; it is not merely a file zipper. Neither endpoint was contacted for this investigation. [D: file offsets `0x289fb2`, `0x28a82a` and nearby strings]

Output strings identify progress text, a generated report path, completion text, and a “Found problems” summary. Static path code resolves the home directory through `HOME` or the user database, then constructs a `Documents` path. The filename formatter is **`synergy-diagnostic-<hostname>-<time-value>.zip`**: it obtains the hostname with `sysctl(CTL_KERN, KERN_HOSTNAME)` and falls back to `unknown`; it computes the other field from date/time components as an epoch-seconds value. Thus even the archive filename can disclose identity. **Not fully established:** how the Documents path is used as the final parent/current directory, collision behavior, or output-path overrides. [D: home lookup `0x100045fe8`; Documents construction `0x10000fe00–0x10000fe3c`; time/name formatting `0x10000ff60–0x10001013c`; hostname reader `0x1001925fc`; format literal at file offset `0x32f6c8`; SDK: `sys/sysctl.h` defines CTL_KERN=1 and KERN_HOSTNAME=10]

**Upload behavior:** no Symless support-upload endpoint or explicit upload action was identified in the inspected application strings. The two application-specific HTTP URLs found are local service checks. HTTP/TLS libraries and a default configuration field named `send-report` are present; neither is proof that this program uploads the archive. Local ZIP generation is established by the manifest/output code evidence. Automatic upload, caller-side upload, and telemetry behavior remain open. No runtime network experiment was performed. [D: URL inventory, manifest, report-output strings]

**CLI limit:** no validated safe help/version-only path was established. The executable was not run. Passing `--help` to an unverified collector could start collection, subprocesses, or service requests, so there is no observed help output or exit-status contract in this report. [D: static-only inspection]

## D. Packaging, signing, launchd, and persistence

### Electron launcher and helpers

**Verified static:** the 52,816-byte `Contents/MacOS/Synergy` is a thin launcher linked to Electron Framework. Its startup checks the run-as-Node fuse and `ELECTRON_RUN_AS_NODE`, then chooses `ElectronMain` or `ElectronInitializeICUandStartNode`. It also repairs missing standard descriptors using `/dev/null`. This does not mean the installed fuse permits Node mode; fuse values were not decoded here. Electron Framework's plist reports **42.4.0**. [P; copied `Synergy.disassembly.txt`, `0x100000460–0x1000005b8`; framework plist]

The outer bundle identifier is `com.symless.synergy`, version `3.7.0-beta`, minimum system version `12`, principal class `AtomApplication`, main nib `MainMenu`, and category `public.app-category.utilities`. It declares generic camera, microphone/audio-capture, and Bluetooth usage descriptions. These declarations do not show that keyboard/mouse sharing invokes those permissions. ATS permits arbitrary loads and local networking, with loopback exceptions. The app also records packaged-ASAR integrity metadata. [P: outer `Info.plist`]

Four helper apps are packaged: `Synergy Helper`, `Synergy Helper (GPU)`, `Synergy Helper (Renderer)`, and `Synergy Helper (Plugin)`. All are arm64, have `LSUIElement=true`, set `MallocNanoZone=0`, and allow automatic graphics switching. Their bundle identifiers append `.helper`, `.helper.GPU`, `.helper.Renderer`, or `.helper.Plugin` to the main identifier. Their bundle version is `3.7.0-beta`. None was running in the passive samples. [P: four helper plists/signatures; R]

### Signing and permissions

All six `Contents/MacOS` executables and all four Electron helper apps are signed by **Symless Ltd** under Apple's Developer ID chain. Each inspected code directory has hardened-runtime flag **`0x10000(runtime)`**. The single displayed entitlement in each case is **`com.apple.security.cs.allow-jit = true`**. No App Sandbox entitlement or TCC bypass entitlement was displayed. The native tray/security/diagnostics/core signatures report runtime version 14.5; the service reports 15.0; the Electron launcher/helpers report 26.2. These are signing metadata, not the host OS or proof of source/compiler identity. [P: `signing-summary.json` and original signature display]

`codesign --verify --strict` passed for the three copied native target helpers. The copied bare Electron launcher cannot validate its original bundle-bound Info.plist from an arbitrary directory; the **original app bundle** passed strict verification and satisfied its designated requirement. This is signature integrity evidence, not a fresh notarization/Gatekeeper assessment. No trust or quarantine settings were changed. [P: verification output]

The entitlement list itself cannot explain how the injector receives Accessibility/Input Monitoring authorization. The AX helper checks its own trust, and the live core demonstrably runs, but the TCC database and process responsibility chain were not read. No permission was requested or changed. A stable signed identity is an architectural requirement to investigate for Openergy, not a demonstrated explanation of this install's grant. [S, P, R; inference explicitly limited]

### LaunchAgents and login-screen support

The exact meaningful contents of `~/Library/LaunchAgents/com.symless.synergy3.plist` are:

| Key | Value |
| --- | --- |
| `Label` | `com.symless.synergy3` |
| `ProgramArguments` | `/Applications/Synergy.app/Contents/MacOS/synergy-service`, `-d` |
| `KeepAlive` | Dictionary: `SuccessfulExit = false` |
| `WorkingDirectory` | `/Applications/Synergy.app/Contents/MacOS/` |
| `StandardOutPath` | `/~/Library/Logs/Synergy/com.symless.synergy3.log` |
| `StandardErrorPath` | `/~/Library/Logs/Synergy/com.symless.synergy3_error.log` |

There is no explicit `RunAtLoad` key in this file. Passive `launchctl print gui/<uid>/com.symless.synergy3` showed it loaded and running, **runs=1**, with no prior exit in the current loaded job. Its service parent is launchd; the service parents the tray and core. `SuccessfulExit=false` is the unsuccessful-exit restart condition, not an unconditional promise to restart after a clean quit. [P: copied plist; R: launchctl/process hierarchy]

No `~/Library/LaunchAgents/com.symless.synergy-agent.plist` exists. Instead, `/Library/LaunchAgents/com.symless.synergy-agent.plist` is a **root-owned dangling symlink** to `/Library/Application Support/Synergy/com.symless.synergy-agent.plist`. The destination file is absent. Its intended contents, including any `LoginWindow` session constraint, therefore cannot be recovered from that live file. The diagnostics collector explicitly knows this system plist, and its embedded default schema has `local_system.run_on_login_screen = false`; this is a shipped default, **not the user's setting**. [P/R: `lstat`, `readlink`, existence check; D: archive manifest and default schema]

`/Library/PrivilegedHelperTools/synergy-core` is also root-owned, but it is a **symlink to the ordinary app-bundle core**, not a separate privileged helper executable. The target mode is `0755`, without setuid/setgid bits. No Synergy-named LaunchDaemon was found. Queries for the two observed Synergy labels in the system launchd domain, and for `synergy-agent` in the GUI domain, found no such service. These bounded checks do not exclude an unrecognized helper label or an on-demand login-window setup. Current login-screen functionality was not exercised and is not established. [R: directory inventory, symlink target/mode, launchctl exit 113]

### The apparent temporary core path

**Verified passive, correcting an assumption:** `ps` identifies the active core as `$TMPDIR/synergy-core` and displays its client role. However, `lstat($TMPDIR/synergy-core)` identifies a **Unix socket**, not an executable. `lsof -a -p <core-pid> -d txt` identifies the mapped executable as **`/Applications/Synergy.app/Contents/MacOS/synergy-core`**, together with bundle Qt libraries. A separate `$TMPDIR/synergy-service.sock` is also present. No temporary regular file named `synergy-core` was found in that inventory. [R: process title, temp-node types, core text mappings]

This observation does **not** support a currently running copy of the core at that temporary path. A changed process title/launch argument or another launch indirection is a plausible explanation. Why the displayed path differs, whether another mode copies a core, and whether this affects TCC attribution remain open; the service/core startup code must resolve it. Do not implement temporary-copy behavior based on `ps` alone. [R; hypothesis]

### Updates and removal leftovers

`Resources/app-update.yml` names the GitHub provider, owner `symless`, repository `synergy-blacksheep`, and cache directory name `synergy-updater`. Squirrel.framework, ShipIt, Mantle, and ReactiveObjC are bundled. Squirrel's native interfaces describe downloading/preparing updates, signature verification, a launchd ShipIt job, privileged installation, cleanup, relaunch, and rollback. ShipIt includes checks for canonical target paths, symbolic-link traversal, matching the containing installer bundle under elevation, and multiple running app instances. These are bundled capabilities, not proof that the active service uses them or that an update has run. [P: copied `app-update.yml`; `Squirrel.strings.txt`, `ShipIt.strings.txt`]

Potential Squirrel artifacts include `.ShipIt` jobs, `ShipItState.plist`, `ShipIt_stdout.log`, and `ShipIt_stderr.log`. No `com.symless.synergy.ShipIt` cache directory or `synergy-updater` cache directory was present at the checked user-cache locations. No Synergy/Symless package receipt matched `pkgutil --pkgs`. The update UI, feed authentication, download policy, service/core restart sequencing, and uninstall workflow were not executed. [P/R: framework strings and bounded path/receipt inventory]

Actual paths that an uninstall/migration plan must account for are the user LaunchAgent, both system symlinks, `~/Library/Preferences/Synergy`, `~/Library/Application Support/Synergy` (Electron storage/cache/crash/telemetry directories), `~/Library/Synergy/Synergy.conf`, `~/Library/Logs/Synergy`, `/Library/Application Support/Synergy/tls`, and the two observed temporary socket names. The dangling login-agent link is already stale. These are existing persistence locations, not proof that Synergy's uninstaller leaves every one behind. The user's independent FocusFix bundle, agent, and logs are separate user work and must be preserved. No removal was attempted. [R: filesystem inventory; F: helper LaunchAgent]

## E. Real user log analysis

### Coverage and severity defect

The fixed JSONL sample contains **733 records**, **331 service** and **402 core**, from **10:31:02.908Z to 20:58:41.549Z on 2026-10-21**. This is **03:31:02.908–13:58:41.549 PDT**, about 10 hours 28 minutes. The launch-agent stdout file was 572 bytes and stderr was empty at inspection. No other `.log` files were in this directory. [L; initial file inventory]

The outer JSON levels are **728 INFO, 5 WARNING, 0 ERROR**. Core text messages carry another level inside `msg`, for example a bracketed local timestamp followed by `WARNING:`. Using that inner level for core messages yields **504 INFO, 225 WARNING, 4 ERROR**. Thus a filter that trusts only JSON `level` misses **220 warnings and all four errors** in this sample. Inner core timestamps are local time; outer `ts` values are UTC. Counts below use the outer UTC timestamp and the effective inner severity. [L: all first 733 records; `log-effective-categories.json`]

### Warning and error categories

| Effective severity | Category | Count | First → last UTC | Evidence lines |
| --- | --- | ---: | --- | --- |
| Warning | Network unreachable | 116 | 10:31:05.437 → 10:33:13.862 | 11–371; 115 core messages plus one service `ENETUNREACH` |
| Warning | `cursor may not be visible` | 79 | 16:15:58.159 → 20:58:30.182 | 378–732 |
| Warning | Connection timeout | 13 | 17:00:02.290 → 17:01:11.168 | 396–442 |
| Warning | Server not responding | 1 | 17:01:27.168 | 453 |
| Warning | `not sending clipboard data, exceeds limit: 4000 KB` | 10 | 17:02:56.323 → 20:54:03.909 | 474–729 |
| Warning | Core IPC socket closed | 1 | 17:18:47.113 | 486 |
| Warning | Unexpected core exit / auto restart | 1 | 17:18:47.115 | 488 |
| Warning | Core rejected because that client name is already connected | 2 | 18:13:40.275 → 18:13:41.339 | 547, 559 |
| Warning | Service `AlreadyConnected` state | 2 | 18:13:40.276 → 18:13:41.338 | 548, 557 |
| Error | TLS system-call failure | 2 | 16:59:58.991 → 18:13:39.192 | 392, 537 |
| Error | Core duplicate-client rejection | 2 | 18:13:40.275 → 18:13:41.338 | 546, 556 |

All table values are from L. The two duplicate-client incidents each produce a core error, a core warning, and a service warning; those six rows of log output are **two incidents**, not six independent connection failures. The 4000 KB messages establish skipped clipboard sends, but do not identify clipboard contents or prove ten unique clipboard payloads. The cursor warning establishes a detection/reporting problem, not 79 verified episodes of an invisible pointer. No payloads, peer identities, or endpoint values are included.

### Timeline and implications for Openergy

At **10:31Z**, service startup took about 2.5 seconds to its success message. Permission gating initially skipped core configuration. The service then started a core, stopped it, and started another within roughly one second; IPC became connected at 10:31:13.701Z. Network failures occupied the first few minutes. **Do not interpret the later first Connected record at 16:15:58.199Z as a continuously measured six-hour outage:** the log does not prove host wake/network availability throughout that interval. [L: lines 1–30, 371, 380; R: one service launchd run]

At **16:59:58Z–17:02:21Z**, a TLS failure was followed by a reconnect cluster, 13 timeouts, one unresponsive-server warning, and further reconnects. At **17:18:47Z**, the core IPC closed and the service automatically restarted the core; Connected returned at 17:18:48.128Z, about one second later. At **18:13:39Z**, another TLS failure preceded two duplicate-client refusals; Connected returned at 18:13:42.510Z. [L: lines 392–473, 486–505, 537–567]

The service logged **136 one-second retry notices**, **eight Connected transitions**, **five Disconnected transitions**, and **three core-process starts** in the fixed sample. The last two are service events, not independently measured network sessions or three launchd restarts. Most later warnings concern cursor visibility during ordinary screen-entry/exit activity, with clipboard-limit warnings recurring near the end. [L: exact marker counts in `log-categories.json`; category table]

The small launch-agent stdout log identifies a production build, an empty parsed `.env` result, INFO logging to `synergy.jsonl`, a skipped production Node-version check with Node **v24.16.0**, and successful daemon startup. It does not contain an error trace. This is startup-wrapper evidence, not a complete launch profile. [L: `com.symless.synergy3.log`; empty `_error.log`]

**Design implications (inference):** preserve the real level in structured logs; group retries into incidents; expose the clipboard-size policy and skipped-send state; distinguish local injector health from peer connection state; and make reconnect failures explainable without publishing identities. The logs do not measure the focus bug or input latency. [L; inference]

## F. Focus bug and the user's workaround

### What is established

The user's README reports that on macOS 27, Synergy content-area clicks can operate controls without activating an inactive application, while title-bar clicks activate it. This report did **not** reproduce that defect because a reproduction would require injected input and desktop interaction. The workaround sources establish what the repair attempts, not that a particular CoreGraphics field is the root cause. [F: README and both source files]

**Version 1:** creates a **listen-only**, head-insert **session event tap** for left, right, and other mouse-down events. It enumerates on-screen non-desktop CG windows in front-to-back order, skips transparent/tiny/self windows, selects a normal-layer window at the event location, calls `NSRunningApplication.activate`, then sets the application's focused AX window and performs AXRaise. Its AX match is based on bounds containing the point with a 12-unit tolerance. It returns the original event and does not create or repost a click. [F: `watchedEvents`, `windowUnderPointer`, `raiseAccessibilityWindow`, `focusWindowUnderPointer`, callback, `CGEvent.tapCreate`]

Version 1 acts on **all** matching mouse-downs, including physical mouse input, and uses `activateIgnoringOtherApps`. It can therefore alter ordinary focus behavior. The popup guard is narrower than the README's simple description: it blocks focus behind only qualifying nonzero-layer windows owned by the frontmost app, no more than 1,200 units in either dimension and 900,000 in area. It is a geometric heuristic, not true system hit testing. Apple's current SDK says `activateIgnoringOtherApps` has no effect from macOS 14 onward. [F: `blocksFocusBehind`, activation call; SDK: `NSRunningApplication.h:25–27`]

**Version 2:** limits action to left mouse-downs whose `eventSourceUnixProcessID` resolves through `proc_pidpath` to executable basename `synergy-core`. It excludes a 12-unit resize halo/border, a 44-unit title-bar area, and the already frontmost normal window. It retains the bounded overlay guard. It picks the AX window whose position/size has the smallest sum of absolute coordinate differences, accepts it only when that sum is at most 80, activates the app if needed with `options: []`, and focuses/raises the window. [F: `isSynergyCoreEvent`, `windowUnderPointer`, `boundsDistance`, `activateAndRaise`]

Version 2 schedules activation on `DispatchQueue.main.async` so the tap callback can return the original click first. This reduces the chance of raising a window before the click reaches its target; it does **not** prove WindowServer has finished delivery before the queued activation runs. It also re-enables taps disabled by timeout/user input and keeps debug title/owner logging disabled unless `SYNERGY_FOCUS_FIX_DEBUG=1`. [F: callback, `installedEventTap`, `debugLogging`]

The user LaunchAgent is loaded and running. It launches the bundled `SynergyFocusFix.app/Contents/MacOS/SynergyFocusFix`, uses associated bundle identifier `com.example.SynergyFocusFix`, `RunAtLoad=true`, `KeepAlive=true`, `LimitLoadToSessionType=Aqua`, and `ProcessType=Interactive`. Its stdout/stderr go to separate FocusFix logs. This supports the README's stable-bundle TCC-identity intent. **The inspected source version was not matched to the installed executable**, so running-agent evidence is not proof that version 2 is installed. [F: README; R: helper plist and launchctl]

### Limits

Neither version repairs the original event's source, routing metadata, button state, click count, or down/up pairing. A delayed app activation cannot recreate native first-click semantics. Version 2 skips right/middle click, title bars, borders, and the frontmost normal window by design. Geometric matching can fail with sheets, full-screen/Spaces, unusual window layers, stale bounds, overlapping windows, non-AX apps, or activation denial. The PID-basename check can miss a renamed/different injector. Source requests AX trust; tap creation can still fail, and the README also instructs the user to grant Input Monitoring when required. No Screen Recording request is implemented in these sources. [F: filtering, matching, tap creation, permission code]

### Injector correction: verified API facts versus hypotheses

The root correction should be in the **injector's global event construction/routing**, with AX activation kept as a narrow fallback if experiments prove it necessary. The exact macOS 27 defect is **not established by this scope**. In particular, the workaround does not show whether Synergy currently uses a HID or session post, whether it leaves click state at zero, or whether it supplies window metadata. [F; hypothesis boundary]

1. **Use a persistent, explicit `CGEventSource`.** Apple defines separate combined-session, HID-system, and private state tables. Its header recommends private state for remote-control programs that need state independent of other processes. A login-session injector may instead need combined-session semantics. **Hypothesis to test:** source-state selection or a reused event carrying stale state contributes to this failure. Do not claim that `kCGEventSourceStateHIDSystemState` alone guarantees activation; source state and posting location are different choices. [SDK: `CGEventSource.h:18–57`]
2. **Construct real global mouse events and post through WindowServer.** Use `CGEventCreateMouseEvent` with the correct down/up/drag type and global position. Investigate `CGEventPost(kCGHIDEventTap, …)` as the global injection path and compare it with the existing session path in an isolated session. App-directed `CGEventPostToPid` is an explicit routing API and is not a substitute for a normal global click when activation must follow native hit testing. **Unverified:** switching taps alone fixes macOS 27. [SDK: `CGEvent.h:44–69, 347–374`; `CGEventTypes.h:399–405`]
3. **Maintain coherent click metadata.** Set/verify `kCGMouseEventClickState` as 1 for a single click and 2/3 for subsequent clicks, with native timing/distance rules; maintain matching down/up event numbers, button number/state, location, modifiers, timestamps, and drag sequence. Do not replay a mouse-move object as a mouse-down or carry an old target into a new click. These are API-defined event fields and sound injector requirements; a missing click state is only a candidate cause here. [SDK: `CGEventTypes.h:139–164`; `CGEvent.h` mouse constructor]
4. **Check window annotations rather than inventing them.** Public fields `kCGMouseEventWindowUnderMousePointer` (**91**) and `kCGMouseEventWindowUnderMousePointerThatCanHandleThisEvent` (**92**) exist. A controlled comparison should determine whether the normal hardware path populates them and whether Synergy's synthetic event differs. If explicit stamping is needed and validated, use the actual hit-tested CG window number in the correct coordinate space, including overlay behavior. Do not fill both fields with an arbitrary normal-window ID, a process ID, or an undocumented numeric “window” field. Apple's headers do not promise that setting these fields activates an app; this remains a macOS 27 hypothesis. [SDK: `CGEventTypes.h:376–378`; F: overlay-related workaround limits]
5. **Use `NSRunningApplication.activate(options: [])`/AX focus only for a proven fallback.** Activation is a request and may fail; Apple does not guarantee immediate activation. Respect the clicked window, popup/sheet ownership, Spaces, and native first-mouse policy. Calling activation before delivery can retarget the click; calling it afterward can race subsequent keys. Avoid a permanent all-click focus-stealing observer. Version 2 illustrates useful restrictions but does not remove the race. [SDK: `NSRunningApplication.h:128–148`; F: both callbacks]

**Required future proof, not performed here:** in an isolated permitted session, compare a physical click with the injector's event source/post location/click/window fields; check inactive content, same-app background windows, first-click action policy, double-click, drag, right click, menus/sheets, multiple displays and Spaces; verify subsequent typing reaches the intended key window. This research added no implementation or tests and did not drive the user's desktop. [Inference from SDK/F and the task's live-input constraints]

## G. Resource baseline and launch-time work

### Live memory and CPU

Three passive `ps` samples, five seconds apart, were taken at **14:03:27–14:03:38 PDT**. All observed Synergy processes are listed below. Values are **KiB RSS**, not unique physical memory or a launch peak. `ps %cpu` is the OS-reported rounded measure, not an instrumented latency or load benchmark. No workload was generated. [R: `process-samples.json`]

| Process | PID / parent | RSS range KiB | Approx. MiB | `%CPU` in each sample |
| --- | --- | ---: | ---: | ---: |
| `synergy-service` | 1229 / launchd | 47,568–47,648 | 46.45–46.53 | 0.0 |
| `synergy-tray` | 1400 / service | 46,208 | 45.13 | 0.0 |
| `synergy-core` | 76958 / service | 53,264–53,936 | 52.02–52.67 | 0.0 |
| **Synergy total** | **3 processes** | **147,040–147,792** | **143.59–144.33** | **0.0 summed** |

Earlier, at 13:59:17 PDT, the same process set summed to **118,832 KiB (116.05 MiB)**. RSS can vary with paging and normal activity; these observations are not a leak measurement. The service/tray had been running for about 10.5 hours and the current core for about 3.75 hours at the three-sample capture. No Electron GUI/helper, security, diagnostics, or updater process was present. A foreground-UI baseline remains unmeasured. Research-tool processes were excluded using their executable identity, not by a broad match for the word “synergy.” [R: `process-snapshot.json`, `process-samples.json`]

The user's separate FocusFix process was also running: **12,208 KiB RSS (11.92 MiB)** and **0.0% CPU** in a separate passive sample. It is not part of Synergy's packaged process total, but it is real extra user cost associated with the workaround. [R: `ps` for the loaded helper PID]

### Installed disk footprint

The app contains **548,821,397 bytes (523.40 MiB)** of regular-file content when framework symlink aliases are counted once. Allocated file blocks total **549,822,464 bytes (524.35 MiB)**. `du -sk` reports **536,936 KiB (524.35 MiB)** including filesystem directory accounting. [R: `footprint.json`; regular-file inventory and `du`]

| Component | Regular bytes | Approx. MiB |
| --- | ---: | ---: |
| All Frameworks | 340,961,326 | 325.17 |
| All MacOS executables | 140,202,048 | 133.71 |
| All Resources | 67,636,244 | 64.50 |
| Metadata/signature remainder | 21,779 | 0.02 |
| `synergy-service` | 127,473,952 | 121.57 |
| `synergy-core` | 7,003,696 | 6.68 |
| `synergy-diagnostics` | 3,962,128 | 3.78 |
| `synergy-tray` | 1,404,560 | 1.34 |
| `synergy-security` | 304,896 | 0.29 |
| Electron launcher | 52,816 | 0.05 |
| Electron Framework | 283,675,605 | 270.53 |
| QtGui / QtWidgets / QtCore | 18,011,477 / 12,762,269 / 12,717,175 | 17.18 / 12.17 / 12.13 |
| QtPlugins / QtNetwork / QtDBus / QtSvg | 5,897,184 / 3,458,445 / 1,476,391 / 1,224,133 | 5.62 / 3.30 / 1.41 / 1.17 |
| `app.asar` | 67,474,000 | 64.35 |

Component rows overlap the three directory totals; do not add both sets. Framework byte counts include their metadata/resources, not only their main Mach-O. The four Electron helper bundles together occupy about **0.86 MiB**. Qt plists report **6.11.1**. [R: inventory; P: framework plists]

At a later bounded inventory, regular-file content outside the app measured about **11.5 KB** in user Preferences/Synergy, **159.7 KB** in user Application Support/Synergy, **65 bytes** in the legacy `~/Library/Synergy` directory, and **96.2 KB** in Synergy logs; these are live, changing logical sizes. System Application Support/Synergy contained a tiny TLS directory. Symlinked helper paths add no second core binary. No backup snapshots, APFS shared extents, or other users' data were measured. [R: bounded path totals]

### Launch work visible without restarting

The observed login session loads the service through launchd; the service owns a separate Rust/AppKit tray and a Qt core. Core mappings include QtCore, QtGui, QtWidgets, QtNetwork, QtDBus, the Cocoa platform plugin, and macOS style plugin. Thus “no Electron UI open” still means a service plus two native GUI/event stacks. The tray decodes embedded images and creates its menu/event loop and stdin thread. The Electron launcher can add the Electron framework and helper processes when its UI is opened, but that cost was not measured. [R: parent/mapping inventory; T: startup path; P: launcher/framework metadata]

Historical startup logs show environment/log initialization, service startup, permission gating, discovery, core startup/restart, IPC connection, and network retry work. They show roughly **2.5 seconds** from the initial service version record to “service started successfully,” and **10.8 seconds** to first core IPC connection in this boot. Those are wall-clock log intervals under that day's conditions, not cold-launch CPU, allocation, or keyboard-readiness benchmarks. No launch instrumentation was permitted. [L: lines 1, 5, 8–10, 15–30]

**Baseline for Openergy (inference):** compare a native always-on implementation against approximately **144 MiB summed RSS with the GUI closed**, a **523 MiB app content footprint**, the 45 MiB tray alone, and the extra 12 MiB user workaround. Measure UI-open cost, actual input latency, wake/reconnect behavior, and launch readiness separately before claiming a speed improvement. [R, L; limits above]

## Open questions

1. Which exact service handler receives `CONFIG`, and does it reuse or spawn the Electron UI? How does it decide when to send `SHOW_START` versus `SHOW_STOP`?
2. Is the tray's predefined item definitively a separator on the current Rust/AppKit path, and does a user click open the menu or take the left-click CONFIG branch under the installed tray-icon settings? Static labels/branches are recovered; no menu interaction was performed.
3. What process responsibility/TCC identity does the service's security subprocess inherit, and how is the core authorized for event posting/listening? The helper's AX result alone cannot answer this.
4. What installed/installer code writes the missing system login-agent plist, what are its exact `LoginWindow` constraints, and is it removed intentionally when login-screen support is disabled?
5. Why does the active core expose a temporary socket path as its process name while mapping the bundle executable? Does any other launch mode actually copy a core, and why?
6. What are diagnostics' exact CLI/default output directory and filtering rules? Is report upload performed by a caller, telemetry component, or neither? Inspect collection and caller code without generating a report from private data.
7. Which event source, posting tap, click-state fields, and window annotations does the current injector use? None of the proposed macOS 27 root causes has been reproduced or proven here.
8. Does the installed FocusFix binary match the supplied v2 source? How often do its window-matching/activation requests fail under normal use?
9. How does the updater coordinate replacing the bundle with a still-running service/core? Is Squirrel active or simply packaged beside another update path?
10. What are visible-UI RSS, true physical footprint, active CPU, input/clipboard latency, wake latency, and cold-login readiness? Passive samples and log intervals cannot answer those questions.

Only this Markdown report was added to the repository. No implementation, tests, configuration changes, launchd mutations, or git mutations were made.
