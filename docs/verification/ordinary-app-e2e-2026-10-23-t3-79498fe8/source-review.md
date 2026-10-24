# Source gates read before the single attempt

The complete `docs/verification/macos-app-current-flow-2026-10-23/README.md`,
`results.json`, and all six files in its `raw/` directory were read. Its
`raw/command.py` is the retained controller; there is no separate `controller/`
directory in that record. That prior 25/25 run is historical evidence only.
The current `OpenergyApp.swift`, complete `AppEndToEnd.swift`, supervisor,
model startup, updater guard, IPC bounds and daemon platform selection were
also read. No prior runtime outcome is treated as a fresh reproduction.

`OpenergyApp.swift:8` calls private dispatch first. In
`PrivateUpdateFixture.swift:23`, nil `Bundle.main.infoDictionary` refuses;
an available ordinary dictionary without fixture metadata returns through
the ordinary branch. The prepared plist was a complete normal dictionary,
with `dev.openergy.app` and executable `Openergy`. This one run reached
`AppEndToEnd`; it does not exercise a malformed/nil plist or private startup.

`OpenergyApp.swift:28-36` dispatches `--e2e` and returns before
`OpenergyApp.main()`. `AppEndToEnd.swift:37-40` validates private paths,
literal loopback ports and virtual arguments, then calls `initializeConfig`.
Only later, at lines 52-61, does it construct and start the real `AppModel`.
The path checks reject an existing config directory. Lines 457-478 create
that directory with exclusive `mkdir(0700)` and write the known Off config
without overwrite, mode 0600. Its client remote is empty and autoconnect false.
The external controller did not insert config callbacks or substitute commands.

For no rendering, lines 44-45 bypass even `NSApplication.shared` setup;
line 261 runs `CFRunLoopRun`, and lines 495-500 return before rendering.
The separate render-enabled branch sets activation policy `.prohibited`.
This run did not enter that branch, start NSApplication's normal loop or
create the normal SwiftUI scenes. These are source dispatch gates, not a
runtime audit of all linked framework behavior or host preference effects.

`AppModel.swift:29-30,64` disables system integration for `--e2e`, so its
normal login-item refresh does not run. `AppUpdater.swift:30-37` excludes
`--e2e` from normal updater startup; normal AppDelegate launch is not reached.
No updater verification or running private fixture flag was supplied.

`DaemonSupervisor.swift:43-48` resolves the bundled helper because no
`OPENERGY_DAEMON` override was supplied. Lines 65-74,93 run that helper with
the app parent PID, explicit private config/socket and virtual arguments,
passing the complete private app environment. The supervisor retains its
actual Process child PID for the routine's stop/continue/kill/restart case.
The external controller's additional group assumption was wrong and caused
the recorded failure; the default product routine was unchanged.

`adapter.rs:50-62` selects `virtual_backend::create` for the explicit virtual
platform. Native macOS construction is in a separate match arm. At lines
16-23, mDNS is absent for virtual mode and also disabled by the environment.
The runtime's default live-port enablement applies only to named native
platforms and only without an explicit denial. This run supplied
`OPENERGY_ALLOW_PORT_24800=0` before launch. No global DNS or native backend
was needed for the real private peer.

`IPCSocket.swift:31` caps a frame at 16 MiB. `DaemonClient.call` defaults
to 10-second requests. The default routine retains its own per-step waits.
Its stopped-write case uses the existing 1,000,000-byte padding and 300 ms
request timeout, with the existing one-second responsiveness and two-second
request bound. Its crash/recovery case retains the three-second invalidation
and fifteen-second recovery waits. No import-budget fixture or other stopped
task was enabled. The ordinary legacy and empty imports are unchanged.

Every source pointer above is bound to the exact root by the app and daemon
input inventories. This source review supports execution preflight. It does
not cure the failed runtime controller or qualify the full ordinary flow.
