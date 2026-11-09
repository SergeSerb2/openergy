# Windows and Linux desktop GUIs

Design proposal, 2026-11-09. No implementation or desktop acceptance is implied.

## Decision and scope

Build two small native frontends over the existing daemon: Windows native Win32 controls in Rust (`windows` bindings), and Linux GTK4/libadwaita in Rust. Share IPC, application state, config edit operations, and layout geometry. Do not share widget code. Keep the existing SwiftUI macOS app.

Start with Windows. Its per-user install, supervisor, private pipe, and ownership checks already exist. That gives a smaller lifecycle gap than Linux, which has an X11 backend but no packaged systemd user unit. Windows also offers a direct cross-platform acceptance pair with the macOS app. This is a scope decision, not a claim that Windows desktop acceptance is complete.

V1 is an interactive, per-user keyboard/mouse/clipboard controller. It must support Off, Host, Join, secure pairing, host layout editing, settings, diagnostics, import, and explicit startup control. It must not include a Windows service, login-screen control, unattended machine-wide access, Wayland control, cloud accounts, remote administration, automatic trust, or a new network protocol. A GUI that renders under Wayland does not make the daemon support Wayland.

## Evidence and contract authority

Read these together; older prose alone is not an implementation guarantee:

- [UI contract](ui-contract.md): thin client, framing, full status snapshots, config patches, methods and events. Its process decision is specifically macOS.
- [Frozen IPC types](../../crates/ipc/src/lib.rs) and [daemon dispatch/status](../../crates/daemon/src/runtime.rs): actual methods, responses, events and failures. For example, `config.get` wraps `{config, derived_links}`; `config.set` accepts TOML text, not a config JSON object. `permissions.open_settings` currently returns `settings_unavailable`.
- [Config model](../../crates/config/src/lib.rs), [validation/merge patch](../../crates/config/src/model.rs), and [layout/import contract](../../crates/config/LAYOUT.md): the serialized array key is `screen`, not `screens`; patches replace whole arrays.
- macOS [window/navigation](../../apps/macos/Sources/Openergy/Views/MainWindow.swift), [settings](../../apps/macos/Sources/Openergy/Views/SettingsPanes.swift), [model/actions](../../apps/macos/Sources/Openergy/Model/AppModel.swift), [layout editor](../../apps/macos/Sources/Openergy/Views/LayoutEditor.swift), and [geometry](../../apps/macos/Sources/Openergy/Views/LayoutGeometry.swift): parity baseline, not native acceptance evidence.
- [Windows lifecycle](../../tools/windows/README.md), [ownership checks](../../tools/windows/LIFECYCLE-OWNERSHIP.md), [transport](../../crates/daemon/src/windows_pipe.rs), and [Linux backend](../../crates/platform-linux/src/lib.rs). Linux currently creates an X11 backend. There is no systemd unit to reuse in this checkout.
- [Setup](setup.md), [readiness](readiness.md), and [README footprint](../../README.md): native release gates and the measured macOS comparison. The published 9.1 MiB versus Synergy 524.4 MiB is a macOS bundle comparison, not a Windows/Linux result.

## Toolkit decision and footprint

The following are design assessments, not measured binary sizes. Measure release builds before promising a number.

| Option | Benefit | Cost and decision |
| --- | --- | --- |
| Windows Win32 native controls + Rust | Uses OS controls, no browser or .NET runtime; direct notification-area integration; shares Rust code | More manual layout, DPI, focus, dark-mode, and accessibility work. **Choose** for footprint. Use standard controls for forms; custom drawing only for layout. It will look like a Windows utility, not a Fluent replica. |
| WinUI 3, preferably C++ if reconsidered | Modern Windows composition and native control/accessibility foundation | Windows App SDK deployment adds a shared runtime dependency or a larger self-contained payload; C# can add another runtime. Tray still needs desktop integration. Reconsider if Win32 accessibility or DPI work proves too costly. |
| Win32 + WebView2 | Familiar web UI, useful DOM automation/accessibility | WebView processes, web assets, and runtime dependencies undermine the footprint goal; HTML controls need Windows styling. A small download can hide a large prerequisite. Reject for v1. |
| Tauri | One web UI, Rust backend, installer tooling | Uses WebView2 on Windows and WebKitGTK on Linux; still needs native lifecycle/tray integration. Accessible HTML is possible but canvas layout needs a second accessible interface. Reject for v1. |
| egui or iced | One Rust view codebase and a shared canvas | Custom rendering, fonts and renderer dependencies; OS conventions and assistive technology need explicit validation. Platform tray/packaging work remains. Do not assume smaller or accessible by default. Reject for v1; retain as a costed fallback. |
| GTK4/libadwaita + Rust | Standard GNOME controls, scalable layout, GTK accessibility, no web runtime | Distro GTK dependencies can dominate a clean install; GNOME appearance is not KDE-native. **Choose** for Linux. Accept a GNOME-style utility on KDE; do not maintain a second Qt UI in v1. |

Microsoft documents [shared versus self-contained Windows App SDK deployment](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/deployment-architecture). Tauri documents [WebView2 installer choices](https://v2.tauri.app/distribute/windows-installer/) and [Linux prerequisites](https://v2.tauri.app/start/prerequisites/). GTK documents its [accessibility interfaces](https://docs.gtk.org/gtk4/section-accessibility.html); libadwaita provides [GNOME application widgets](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/index.html). These support dependency and toolkit choices, not size estimates for Openergy.

Provisional engineering budgets: Windows installed app-owned GUI + daemon + lifecycle files below 25 MiB; Linux app-owned files below 25 MiB excluding shared distro libraries. These are targets to validate, not release claims. Record compressed download, installed app files, incremental dependencies on a clean OS, cold launch, idle CPU, and RSS for GUI plus daemon with window open and closed. Also record Windows supervisor PowerShell memory. It must not disappear from the comparison. If it dominates, propose a separate small Rust supervisor that preserves existing ownership semantics before replacing the scripts.

No bundled browser, video assets, icon font collection, or second daemon copy. A larger package needs an explicit budget review. Publish clean-install dependency totals beside app-owned totals. Do not compare a Linux package without GTK against Synergy's whole bundle.

## Native interaction design

Use one resizable settings window with navigation: Connection, Layout, Input, Clipboard, Security, System, Activity. Put persistent sharing state and Off / Host / Join at the top. Use OS fonts, standard spacing, native file pickers, keyboard access keys on Windows, and a GTK header bar/sidebar on Linux. At narrow widths, collapse navigation without hiding the status or Off control.

Windows: a notification-area icon opens a short menu: current state, Open Openergy, Off, Retry, Quit UI, Stop Openergy. Restore the icon after Explorer restarts. Closing the window leaves the notification-area UI running. Quit UI exits it; the supervised daemon may remain active. State this on first close and in System.

Linux: use a normal launcher/window and desktop notifications. Optional StatusNotifierItem integration can provide the same menu on supporting desktops. Do not require a tray extension on GNOME. Closing the window exits the frontend; show “Sharing continues in the background” when appropriate. Reopening always works from the launcher. A pending trust notification opens Security but never approves a peer.

The layout canvas has an equivalent list/inspector. Keyboard users select a screen, place it left/right/above/below a named screen, adjust offset, and remove it. Narrator and Orca must read screen name, position, neighbors, trust, errors, and pending actions. Use text plus icons, not color alone. Respect high contrast, text scaling, reduced motion, focus order, and Windows per-monitor DPI. Do not announce cursor coordinates or every status event to a screen reader.

### Feature parity

“Required” means required before a platform's v1 GUI release, not necessarily in its first PR.

| macOS feature | Windows v1 | Linux v1 | Contract / adaptation |
| --- | --- | --- | --- |
| Off / Host / Join, local name, remote address, retry/disconnect | Required | Required on X11 | `config.patch`, `connection.connect/disconnect/restart`; Off patches role to `off`, disconnect alone does not change saved role |
| Host listen address/port and nearby servers | Required | Required | `server.listen/port`, `discovery.list`; manual address always available |
| Host canvas, unplaced peers, inspector, online/active screen | Required | Required | `screen`, `derived_links`, status peer/local geometry; joiners see a read-only overview |
| TLS identity, pending approval, reject, trusted lists/revoke | Required | Required | `tls`, `pending_trust`, `trust.*`; no auto-approve |
| Input options, edge delay/double tap, dead corners, lock hotkey | Required | Required where backend supports them | Preserve `input.*`, `switch.*`, `hotkeys.lock_to_screen`; explain unsupported backend behavior instead of inventing capability |
| Clipboard enable, limit, recent transfer/blocked notice | Required | Required | `clipboard.enabled/max_bytes`, status and event; preserve `send/receive` even if not exposed |
| Accessibility/Input Monitoring permission guidance | Native equivalent | Native equivalent | Show existing permission grants; no macOS TCC instructions on other OSes |
| Activity tail, filtering/copy, bounded live logs; config location | Required | Required | `logs.tail/subscribe/unsubscribe`; open native file manager, redact before sharing diagnostics |
| Open at login | Separate daemon and UI preferences | Start sharing at graphical login | Windows limited logon task; Linux systemd user unit; not `SMAppService` |
| Update controls | Check/download signed installer | Package-manager guidance | No Sparkle port and no daemon IPC updater |
| Synergy import | Required with preview | Required with preview | `config.import_synergy`; retain settings not editable in UI |
| macOS click-numbering debug switch, Secure Input app guidance | Omit | Omit | Preserve `macos` config on edits; no irrelevant UI |

Settings are edited in small sections with pending/saving/error states. Apply numeric/text changes on an explicit Apply action; do not send invalid intermediate text. Keep drafts during unrelated status events. Serialize mutations, then use the accepted config and refresh derived links. Whole `screen` replacement must retain aliases, links, ranges, modifiers, OS hints and per-screen options. Rename the local canonical screen and all explicit link targets in one patch, as the macOS model does. With no config revision/CAS method today, detect external `config.changed` during an edit and require reload/review before replacing arrays; do not promise conflict-free concurrent writers.

## IPC and shared state

Create a small shared Rust desktop client layer, separate from daemon lifecycle and widgets. Reuse `crates/ipc` types and framing: four-byte big-endian length, UTF-8 JSON, request IDs, response result/error, and unsolicited events. Enforce the 16 MiB frame bound. Dispatch replies by ID; events may arrive between replies. Keep blocking pipe/socket work off the UI thread.

Connect to the explicit user endpoint, subscribe with `events.subscribe`, and load `status.get`, `config.get`, `trust.list`, and `discovery.list`. Render the latest full `status.changed` snapshot, coalesced by the daemon to at most 10 Hz. Refresh config/derived links and discovery on their respective change events. Handle peer connected/disconnected/untrusted, permissions changed, clipboard blocked and error events as hints to the same authoritative state. Subscribe to `log.line` only while Activity is visible; seed it with `logs.tail`, bound memory, and unsubscribe on close.

Distinguish “daemon unavailable” from a live daemon with `role=off` or `state=idle`. On transport loss, mark previous values stale, disable writes, reconnect with bounded backoff, resubscribe, and fetch fresh snapshots. The daemon's partial-frame read budget is ten seconds; a completed subscription may remain idle. Set bounded client call timeouts too. Never replay a timed-out config/trust action blindly: its side effect may already have happened. Refresh and reconcile first.

Use `daemon.version` for diagnostics and tested compatibility policy. There is no capability negotiation method today. Unknown fields should not break status rendering; unknown methods must yield a visible unsupported-version state, not silent success. Virtual control methods are harness-only, absent from product controls.

## Layout geometry without a third implementation

Use screen points in persisted rects, not canvas pixels, Windows physical pixels, or arbitrary “monitor tiles.” Apply one uniform view transform and preserve aspect ratios. `local_screen.displays` has origin, dimensions and scale; use the backend's units, not another DPI conversion on already normalized sizes. Remote peers expose only aggregate `screen.w/h`, not their display outlines; show an outline as unknown rather than fabricating a multi-display topology.

The daemon's `crates/config/src/layout.rs` remains the authority for actual crossing links. Facing edges within one point and perpendicular overlap greater than one point derive links. Source/target percentage ranges round independently, with empty rounded ranges omitted. Corner contact is not a link. Explicit full/ranged/empty links override derivation per edge. Synergy 3's expanded/probed geometry remains import-only.

Proposed shared Rust geometry library: extract the semantics of Swift `LayoutGeometry` and `CanvasTransform` into an OS-neutral module. Windows and GTK consume it directly; Swift adopts a narrow C ABI adapter in the geometry phase. Do not leave two permanent math implementations. Share contact/reachable segments, overlap detection, snapping, free placement, fitting and inverse transform. Keep widgets and pointer gestures local. Until Swift consumes it, treat the extraction as unfinished and compare both real editor flows before removing Swift math.

Preserve existing editor behavior: one-point contact tolerance; display-normalized edge reachability (0.001/0.999 bounds); nearest non-overlapping snap with at least 20% common span; top/bottom/center alignment within a supplied layout-unit threshold; new screens placed to the right. Keep deterministic candidate order for ties. The editor overlap helper tolerates up to one point, but config validation rejects any positive-area overlap: the final save must obey the stricter daemon rule. Normalize positions by subtracting minimum x/y and round rect values as macOS `saveLayout` does; validate the result through the real daemon.

Draw effective arrows from `derived_links`, not just visual contacts. Reachable contact highlights are a separate visual aid. Imported explicit links may differ from physical adjacency. Label them “Imported links”; moving a screen must not silently erase them. Offer an explicit “Use touching edges” action to remove selected overrides, explain the change, then apply. Preserve original imported units until a deliberate conversion. Never silently rescale them based on a newly connected peer.

No new geometry IPC is needed for commits. If unsaved link previews need daemon-authoritative derivation, propose a separate read-only preview method in that phase; it does not exist today. Do not patch live config on every drag just to get a preview.

## First run, trust, and recovery

1. Start Off. Show the computer name and choices “Share this computer's keyboard and mouse” (Host), “Use another computer's keyboard and mouse” (Join), and Import from Synergy. Validate names with daemon errors, including global alias uniqueness.
2. Host: choose the LAN listen address and port explicitly. Show a copyable address and “Waiting for a computer” while listening. Windows fresh installs currently use loopback port 24801, unlike config defaults of 0.0.0.0:24800. Keep these distinctions visible. Do not silently enable port 24800, open a firewall, or bind all interfaces.
3. Join: show discovery `{name, host, port, fingerprint_hint}` plus manual hostname/IP input with IPv6 support. Empty discovery says “No computers found. Enter an address.” Discovery hints and names are untrusted metadata, not approval evidence.
4. Pair: explain “Check that this code matches the code shown on the other computer.” Display the full SHA-256 fingerprint in grouped, readable text with Copy, name/address marked as reported, and an expiry countdown when supplied. The other machine's Security page shows its own `status.tls.fingerprint`. Compare through that screen or another trusted channel. Do not shorten the code used for approval or imply a name proves identity.
5. Approval is a deliberate “Codes match — Trust computer” action after comparison, never the default Enter action. Offer Reject and Cancel. Both roles may need approval because client trust is required by default. Use the fingerprint from the selected pending record, refresh immediately before approval, and disable approval if it expired or disappeared. Since `trust.approve` can also add a nonpending fingerprint, this is not an atomic expiry guarantee; a strict pending-only approval would need a new daemon contract. V1 must disclose/recover from a peer retry rather than pretend that guarantee exists.
6. Re-fetch trust/status after the reply. Do not show connected until status confirms it. Identity changes stop the flow and require comparison again. Revocation explains that current matching connections are cut off. Never solve trust errors by disabling TLS.

Surface errors in three places: concise text in status/tray, an actionable banner in the relevant pane, and details in Activity. Use `status.last_error.code/message/unix_ms`; show a previous error with its time if current state recovered. Show RPC action failures separately from daemon state. Present null geometry, latency, permissions or identity as unknown. For reconnects show `client.attempt/next_retry_ms`; for clipboard caps show peer, bytes and limit from `last_blocked`. Permission `not_required` is not missing permission.

Windows permission guidance must explain that ordinary user input cannot control elevated apps or the secure desktop. Do not request admin to “fix” it. Linux guidance distinguishes X11 authentication/extension failures from unsupported Wayland; an XWayland window alone is not full-desktop sharing. Existing permission fields do not advertise Wayland portal or Windows integrity capabilities; native session diagnostics may explain them but must not invent successful grants.

### Synergy import

Use the native folder picker and `config.import_synergy {path, apply:false}` to preview returned config and warnings. Show local name, role, remote/listen settings, layout and changed options. Require confirmation of the local screen for plain Synergy configs. Apply the same folder with `apply:true` only after confirmation; compare the returned config again, since files can change between calls. There is no immutable import token today. A strict “apply exactly this preview” flow needs a future contract extension, not a JSON argument to `config.set`.

Explain that import replaces settings and may start sharing for the imported role. Offer a saved prior configuration for recovery, without exporting private keys. Preserve aliases, modifier mappings, options and explicit ranges. Daemon imports are bounded and ignore license/account/telemetry fields. `ImportReport` can hold trusted-server candidates, but current IPC returns only config and warnings. Do not claim those candidates are imported or approved. Verify peers again. Importing settings never authorizes stopping Synergy services; show a separate conflict warning and migration guidance. Windows' protected migration helper remains an optional, separately authorized path.

## Process ownership and packaging

### Windows

Reuse the current limited-user lifecycle manager/supervisor. GUI launch first attaches to the expected pipe. If absent, validate the managed installation, ask the manager to Start, then await verified control readiness. Do not launch a second direct child alongside it. The supervisor passes its own parent PID to the daemon and owns crash restart, lifetime lock and five-rapid-failure cutoff. GUI crash/exit does not terminate the managed daemon. Do not add a second restart loop in the GUI.

Pipe paths derive from the selected socket path and user SID; use the existing client security rules, not a guessed global pipe name. Managed stop must retain the same-connected-pipe server PID/start-time/executable checks. An independently started daemon may be attached to, but label it externally managed: no automatic stop, replacement, update or task adoption. Occupied or unverified endpoints block managed startup. Never kill by executable name.

Keep three controls distinct: Off stops sharing but leaves control alive; Stop Openergy uses the managed Stop action and disables its logon task; Quit UI leaves managed sharing alone. The existing Start enables a registered task and Stop disables it. Therefore an independent “Start sharing at sign-in” switch while running requires a small lifecycle API addition that changes task enablement without stopping or starting the daemon. Do not implement a cosmetic toggle that silently inherits current Start behavior. Default autostart off, display actual task state, and verify failure recovery. A separate opt-in GUI startup entry may open the frontend minimized; it is not a daemon supervisor.

Ship a signed per-user installer first, with files in `%LOCALAPPDATA%\Programs\Openergy` and config/identity/trust/logs in `%APPDATA%\Openergy`, matching existing scripts. No elevation by default; the current scripts refuse elevated tokens. Offer an uninstall registration and Start menu shortcut. MSIX is deferred until task registration, data paths, update replacement and full-trust desktop behavior have real acceptance; do not assume it can wrap existing lifecycle scripts unchanged. A session-0 Windows service is not a suitable substitute for the interactive input daemon. Machine-wide deployment needs a separate design.

V1 update action opens/downloads a signed release installer over HTTPS; user initiates installation. The installer verifies publisher/version, stops only the owned supervisor/daemon, replaces the GUI/daemon/lifecycle payload as one version, preserves data, and restarts only if previously running. Handle locked files and rollback on failed startup. No silent updater in v1. An eventual MSIX channel can use package-managed updates; do not run both update owners.

### Linux

Add an unprivileged `openergy.service` user unit. Proposed settings: foreground daemon with explicit config/socket paths, `Restart=on-failure`, bounded restart delay/start limit, `PartOf=graphical-session.target`, and ordering after that target. Enable it for the graphical session only when requested; no lingering and no system/root service. The GUI uses user-manager D-Bus start/stop and reads unit failure details. `daemon.stop` is graceful control, while the user manager owns process supervision. A unit-owned daemon has no GUI `--parent-pid`. GUI exit does not stop it.

The unit requires the actual session's `DISPLAY` and, where needed, `XAUTHORITY`. Import only those required variables through a session startup helper before starting; never hard-code `:0` or copy a stale environment file. Desktop target integration varies: validate GNOME and KDE X11 sessions, and start the unit explicitly from the helper when the target does not pull it in. Stop at graphical logout and avoid a later SSH login reviving it. See systemd's [desktop integration guidance](https://github.com/systemd/systemd/blob/main/docs/DESKTOP_ENVIRONMENTS.md). V1 supports one active graphical session per user. Multi-session routing needs another design.

Use `$XDG_CONFIG_HOME/openergy/config.toml` (fallback `~/.config/openergy/config.toml`) and `$XDG_RUNTIME_DIR/openergy.sock` (existing fallback `~/.local/share/openergy/openergy.sock`). Keep private file/socket permissions and existing ownership checks. GUI and unit must choose identical explicit paths. If a daemon already answers outside the unit, show externally managed and do not enable a competing unit. On non-systemd desktops, provide a clearly labeled session-only GUI-owned child using `--parent-pid`, bounded restart, and an explicit warning that closing the GUI stops sharing; no autostart support promised there in v1.

Prioritize deb and rpm with distro GTK/libadwaita dependencies, desktop file, icon, and user unit. Installing packages may require the package manager's normal authorization; running Openergy never does. Use signed repository/package updates and let that manager own upgrades. System shows installed version and “Update with your software manager,” not a fake automatic-update switch. Verify active user-service restart after an upgrade; package scripts must not start arbitrary users' sessions.

Defer Flatpak: sandbox-to-host daemon IPC, host unit management and broad input access need an explicit boundary, not `flatpak-spawn --host` as a shortcut. Defer AppImage: bundled GTK/runtime size, desktop integration and updater ownership weaken the small-package story. A future AppImage channel needs signed atomic replacement and a stable executable path for the unit. Neither format adds Wayland control.

### Uninstall and authority

Windows uninstall stops only the validated owned installation, disables/removes its task and UI startup entry, removes program files and shortcuts, and preserves config, TLS identity, trust and logs by default, as current scripts do. Offer an explicit separate “Delete my Openergy data” choice with paths shown. Retain recovery state if stopping/restoring fails. Never touch Synergy's service during ordinary uninstall.

Linux uninstall removes packaged files. Disable/stop the current user's unit through the user session when possible; document cleanup for other logged-in users and stale user enablement links. Offer a user-run data removal action; package removal must not enumerate/delete home directories. Preserve identity/trust by default so reinstall does not cause unexpected identity changes. Remove session startup entries owned by Openergy. No GUI writes to system service configuration or grants itself input-device group access.

## Delivery and verification: real interfaces only

No unit tests, mocks, fake daemons or widget-only assertions. Virtual backends are useful real daemon modes for IPC and transport checks; they do not prove physical input or desktop permissions. Each phase records source/binary hashes, OS/session/toolkit versions, commands, screenshots or accessibility output, pass/fail markers and retained failure artifacts. Use private temp state and nonproduction ports; never mutate a user's active install to run checks.

### Phase 0 — first PR-sized milestone

**Exact milestone: “Windows desktop control contract gate: real named-pipe host/join and lifecycle flow.”** This PR adds one PowerShell-driven acceptance scenario and evidence, with no GUI, toolkit, installer rewrite or protocol changes. Run two temporary real `--platform virtual` daemons through the existing `-NoStartup` lifecycle path on permitted lab ports. Use real CLI/framed pipe requests to establish status, config, events and TLS host/join. Approve/reject/revoke real fingerprints; observe blocked/untrusted and connected transitions; patch a layout and read effective derived links; preview/apply a fixture import; subscribe/tail/unsubscribe logs; reject invalid config without changing persisted config; disconnect, restart, stop and confirm cleanup. Check subscription reconnection after restart and one partial-frame failure/recovery.

Reuse the ownership and control drivers rather than duplicating their checks: [verify-control.ps1](../../tools/windows/verify-control.ps1), [verify-lifecycle.ps1](../../tools/windows/verify-lifecycle.ps1), [verify-lifecycle-ownership.ps1](../../tools/windows/verify-lifecycle-ownership.ps1), and [verify-ipc-deadlines.ps1](../../tools/windows/verify-ipc-deadlines.ps1). Run in a non-admin Windows environment; no task registration, migration helper or native input needed. Acceptance: real persisted data/status/events agree, failures remain recoverable, no unrelated daemon stops, and owned processes/endpoints are gone. This proves the controller substrate before widget work. The new aggregate scenario is proposed work, not an existing test command or a result of this document.

### Phase 1 — Windows native vertical slice

Add the shared Rust client/state layer and native Connection/Security/System/Activity shell. Wire Off/Host/Join, real fingerprint approval, diagnostics and existing lifecycle. In a disposable interactive Windows VM, run an end-to-end driver through accessible controls (UI Automation through an automation runner, not desktop mouse/keyboard computer use), then assert daemon status and remote peer behavior. Include empty discovery, denied trust, bad remote, daemon crash, occupied pipe and UI restart. Check Narrator focus/readout, 100–200% scaling, high contrast and Explorer restart. Native GUI browser automation is not available merely because a browser exists; do not substitute a web replica.

### Phase 2 — geometry and complete Windows v1

Extract/adopt shared geometry, add the host canvas plus keyboard inspector and remaining settings. Drive actual editor saves into a real daemon and move input across the resulting peer edges. Use the documented 1512×982/2560×1440 partial overlap, L-shape, one-point gap, corner-only contact, normalized negative draft origins, multi-display local outlines, and explicit imported links. Verify saved ranges after reopen and real crossing boundaries; no isolated math assertions. Run macOS editor acceptance through its real app to prove the shared adapter preserves behavior. Add signed installer upgrade/rollback/uninstall, real limited logon task, reboot/login/logout and identity preservation in a disposable VM. Pair with a Mac and check keys/releases, pointer, clipboard both ways, sleep/reconnect, and elevated-target limitations.

### Phase 3 — Linux desktop and user lifecycle

Reuse the shared client/geometry with GTK widgets. Add the unit/session helper and deb/rpm packaging. First run real socket contract scripts with temporary virtual daemons (existing [daemon verification scripts](../../crates/daemon/scripts) provide examples; `verify_app.py` currently targets the macOS app and is not a Linux GUI runner). Then drive the actual GTK app through AT-SPI automation against real daemons in an isolated X11 desktop. Extend existing [Linux lab](../../tools/linux-dev/README.md) and `verify_linux.py` flows for layout/clipboard/input assertions. Verify empty discovery, import failure, missing X authority, unsupported Wayland guidance and daemon recovery.

Finish on disposable GNOME and KDE X11 sessions: logon unit environment, logout stop, startup off/on, no tray, multi-monitor scaling/hotplug, Orca, package upgrade/remove/reinstall, preserved fingerprints, and a Mac or Windows peer. Non-systemd fallback gets a separate child-lifetime flow. GTK rendering under Xvfb is not evidence for physical displays, compositor behavior or a real login.

### What CI can establish

Linux CI can build packages, run real Unix-socket/TLS/virtual-daemon flows, launch GTK under Xvfb, inspect accessibility and drive full UI-to-IPC flows, plus isolated X11 backend checks. Windows CI can build/sign when credentials are available, exercise real named pipes and virtual daemon/lifecycle drivers, and inspect installer contents. Interactive GUI automation requires a dedicated desktop runner; session-0 SSH or an ordinary hosted job cannot prove native capture, secure desktop limits, notification-area behavior, task startup or real clipboard interoperation.

Dedicated disposable interactive machines must cover logon, notifications, assistive technology, multiple monitors, firewall prompts, signed install/update/uninstall, and native cross-OS input/clipboard. Where automation cannot cover an OS dialog or assistive-technology behavior, record a human-operated acceptance checklist; do not use agent computer use, mocks or inferred passes. CI reports those gates as pending until evidence exists. This design-only change runs no GUI or runtime checks and changes no TypeScript package.
