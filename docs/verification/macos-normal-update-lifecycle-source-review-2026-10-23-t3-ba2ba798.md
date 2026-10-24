# Normal macOS update lifecycle: source review

Review ID: `oe-normal-update-lifecycle-source-r1-5a978373-20261023-t3-ba2ba798`.
Reviewed Openergy: `5a9783738c5b649844805e1614da7ea37cfbf2bb`.
Date: 2026-10-23, America/Vancouver.

**No concrete P1/P2 product or lifecycle defect found in this scope.** This is
source inspection, not independent Sparkle termination, replacement, or relaunch
proof. No app, daemon, fixture, installer, build, test, network request, desktop
action, registry operation, or secret access occurred. Only this document changed.
The stopped security/provenance, mDNS, and IPC/Synergy3 investigations remain stopped.

## Termination and child ownership

- `apps/macos/Sources/Openergy/App/OpenergyApp.swift:148-159` installs the normal
  application delegate. `applicationWillTerminate` calls `AppModel.shared.shutdown()`.
  Closing the last window does not quit the menu-bar app.
- `apps/macos/Sources/Openergy/Model/AppModel.swift:157-228` sets `running = false`,
  changes the session ID, cancels connect/action/log tasks, and disconnects IPC
  before `supervisor.stop()`. The connect error path checks running/cancellation
  before spawning. Disconnect callbacks return when the model is stopped.
- `apps/macos/Sources/Openergy/Daemon/DaemonSupervisor.swift:56-118,135-152` stores
  only the `Process` it spawned. Stop sets `stopping`, cancels scheduled restart,
  sends SIGTERM, and pumps the run loop in 50 ms steps until a 1.5 s deadline.
  If the child still runs, stop sends SIGKILL and clears its stored process.
  The restart task checks cancellation and `stopping`. Queued exit callbacks
  check `process === finished` before clearing state or scheduling restart.
- This is a nominal bounded wait, not confirmed child termination or a strict
  end-to-end deadline. It uses wall-clock `Date` and runs nested callbacks.
  The code ignores the SIGKILL result and does not wait afterward. The process
  identity check protects queued callbacks, not an independently observed exit.
  `supervisedDaemonPID == nil` therefore cannot serve as an exit receipt.
- `AppModel.swift:185-207` first tries the configured socket. A reachable external
  daemon is not adopted as an owned child. Shutdown disconnects from that daemon
  without terminating it. A future owned-child check must establish ownership
  before the update. An attached daemon surviving is not itself a defect.

## Daemon and IPC cleanup

- `crates/daemon/src/main.rs:193-234` validates the supplied parent PID and passes
  explicit config/socket paths to the daemon. The supervisor supplies its app PID
  (`DaemonSupervisor.swift:65-74`). This watches a PID, not an authenticated claim
  about the kernel parent relationship.
- `crates/daemon/src/lifetime.rs:99-122,171-187` registers a macOS kqueue
  `EVFILT_PROC/NOTE_EXIT` watch. It also checks `kill(pid, 0)` for `ESRCH`.
  `runtime.rs:1489-1508` installs signals and the parent watch before its main
  loop. Both SIGTERM and parent exit lead toward the shutdown path.
- `runtime.rs:1641-1663` shuts down the session, flushes networking for 250 ms,
  releases local input, restores cursor visibility, stops workers, and joins
  IPC/capture/clipboard threads. These joins and platform calls have no overall
  daemon-side deadline. The supervisor's force stop can bypass this cleanup.
- `crates/daemon/src/ipc_server.rs:53-138,159-232` wakes listener shutdown through
  a private socket pair, without connecting to a pathname that may now have a
  new owner. Binding holds an exclusive lock and refuses a live existing socket.
  Graceful removal requires the original socket type, device, and inode.
  SIGKILL skips that destructor. A later bind can remove a refused stale socket
  after acquiring the lock. A retained lock file alone does not prove a live owner.
- `apps/macos/Sources/Openergy/Daemon/DaemonClient.swift:23-51,111-125` invalidates
  connection generations and rejects stale socket callbacks. Disconnect fails
  pending calls. `IPCSocket.swift:58-85,108-163` bounds connect polling at 2 s,
  performs writes off the main actor, and uses `shutdown(SHUT_RDWR)` to wake I/O.
  The reader closes the descriptor under the write lock. Shutdown does not join
  that reader. A late detached connect is rejected after generation/cancellation
  checks. These paths do not supply an independent process-exit receipt.

The force-stop/relaunch overlap and kqueue fallback timing are runtime assertions
to check. This review did not establish a supported trigger that makes them a
P1/P2 product failure. It does not convert possible timing into a demonstrated bug.

## Pinned Sparkle sequence

`apps/macos/Package.resolved:5-10` pins Sparkle 2.10.0 to
`eef1a539a373c1f1a320624b1130fc5de7b2e100`, tree
`1973159ee5d6ae334396a84b3e00e924ac94ce7c`. The six cached files below matched
both `git show <pin>:<path>` from the existing local repository and the retained
`sparkle-cli-build-preparation-2026-10-23/raw/source-files.json.gz` inventory.
That inventory's SHA-256 remains
`5342ed8f91a16eafc114a4dd6b85a48ebe67a7d5d4c12c7770c154efbcebb629`.
No fetch or build occurred. Cache location: `/private/tmp/oe-sparkle-cli-build-yhdocbhw/source`.

- `Sparkle/SPUCoreBasedUpdateDriver.m:307-335` requests installation/relaunch and
  sends the optional will-relaunch notification. Openergy needs no custom
  will-relaunch delegate to reach its normal application termination hook.
- `Sparkle/SPUInstallerDriver.m:509-560` sends the installation/resume request.
  `Autoupdate/AppInstaller.m:392-418,749-823` observes the old app's termination,
  requests termination, and performs final replacement before requesting relaunch.
- `Sparkle/InstallerProgress/InstallerProgressAppController.m:196-260,292-348,372-415`
  matches running bundles by bundle ID and resolved path, sends an Apple quit
  event through `NSRunningApplication.terminate`, and observes app termination.
  Relaunch requires the target to have been initially alive. It opens the bundle
  through `NSWorkspace.openURL`, with no explicit old arguments or environment.
  Watching the app does not also watch its daemon.
- `InstallerLauncher/SUInstallerLauncher.m:56-100,257-285` submits/removes transient
  launchd jobs for the host ID. Both job dictionaries request `Nice = 0`.
  `Sparkle/SPULocalCacheDirectory.m:19-54` and `Autoupdate/AppInstaller.m:462-467`
  use the executing helper's user cache directory. A private parent home,
  defaults domain, or shell `nice 15` alone does not contain all helper effects.

Audited cache SHA-256 values, in the same file order as the six files above:

```text
SPUCoreBasedUpdateDriver.m       a0c2d0993514feaa438ff397bacbd7c29a74c0a050d3c43f604e38e66e21aa08
SPUInstallerDriver.m             e813d3e1cae7668ea88b64f9c5a980bf826a06910f99db68da7af99f466deac5
AppInstaller.m                   39369669a98181f81fa58ab7d409c8310fbe4b5ff046fb48bd11a39632195dd1
InstallerProgressAppController.m c6248006f3ab3548f9f1acd86a89074ce91597313b27e8d9fc8d70e4af06f71e
SUInstallerLauncher.m            d18fb924749ae8fd4b164fb20e9397c7aa548b795426c035e96cb69122dab256
SPULocalCacheDirectory.m         b14e2624031a95479a5e3d1c002a3ca0702c1502caa619ab99f0c6f206707cc8
```

## Prerequisites and falsifiable handoff assertions

The current normal entrypoint has no durable private bootstrap. With no mode
arguments, `OpenergyApp.swift:7-40` enters the normal app. The supervisor reads
private path/platform overrides from the environment, then uses application
support defaults when absent (`DaemonSupervisor.swift:18-47,65-74`). Shell-only
isolation is therefore insufficient for a no-argument Sparkle relaunch.

The separate bootstrap preparation is outside this review. Before any future
private check, both bundle versions must independently establish the same private
configuration before normal app/model/updater initialization. Missing or invalid
fixture data must fail before default startup. This document supplies acceptance
conditions and does not change or direct that implementation.

1. Record one unique fixture bundle ID shared by both versions, canonical target
   paths, versions, and executable/helper hashes. Use a separate unique driver ID
   if needed. Use newly owned fixtures. Do not launch old fixtures.
   Establish private defaults, logs, TLS, config, and socket paths before startup.
   `crates/daemon/src/adapter.rs:8-23` derives daemon TLS storage from the config
   parent. Require virtual platform, discovery disabled, and only uniquely owned
   literal loopback endpoints. Exclude TCP 24800 and UDP 5353.
2. Require containment for helper caches, transient jobs, priority, and automatic
   updater behavior before the future run. The normal updater requires HTTPS
   (`AppUpdater.swift:30-38,174-188`). Neither an HTTP information-only command
   nor an assumed inherited environment establishes a safe normal updater launch.
   Preserve production settings, installed apps, services, and live sharing.
3. Before termination, record the old app/child PID plus process start identity,
   actual parent relationship, executable path, private socket device/inode, and
   private config digest. Confirm the app owns the child, rather than merely
   attaching to a reachable daemon. Sparkle must observe this target alive.
4. Observe the actual Sparkle quit request and old app termination. Observe the
   old child exit independently. Require no additional old-app child or restart
   during termination. Retain timing and whether SIGTERM or forced stop occurred.
   A stop log, nil supervisor PID, CLI success, or old app exit alone is insufficient.
5. Observe replacement and Sparkle's no-argument relaunch at the target path.
   Require candidate version/hashes, a new app process identity, and a new child
   identity with the new app as parent. Require the same private config and socket
   paths. Reject attachment to the old daemon. Record socket ownership across
   replacement. Do not infer ownership from the unchanged pathname alone.
6. Observe readiness through the explicitly selected private IPC socket.
   `runtime.rs:291-296` exposes daemon PID, version, platform, uptime, and name.
   `AppModel.swift:185-196,261-283` subscribes and reads status/config before it
   reports connected. Require those responses from the new PID, virtual platform,
   and a unique persisted config marker, plus the unchanged config digest.
   These interfaces can supply evidence without querying the default/live app.
7. Preserve the first failed transcript and identities. Retain failure as failure.
   Record owned-process/helper exit and owned socket/cache/job cleanup separately.
   Any future parent-loss or forced-stop branch needs its own contained evidence.

Prior `043757a` IPC source review and the reported current caller 19/19 result are
context only. The reported `5c2ed82` clean stopped-target R2 proves only its ad hoc
temporary replacement scope. None proves this normal app lifecycle. This review
does not revalidate or extend those results. Standard update UI, native input and
clipboard release/recovery, live Mac/Windows sharing, Developer ID, notarization,
Gatekeeper, and publication remain separate native/release gates. No runtime run
or further review round is requested or authorized by this document.
