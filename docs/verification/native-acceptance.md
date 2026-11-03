# Native acceptance matrix

Snapshot: **2026-11-03, America/Vancouver**. Maintained for [SER-205] and
[readiness refresh SER-204][SER-204]. This matrix is an execution and evidence
index. **It grants no native desktop acceptance.**

The published baseline is `v0.1.0`, source
`e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`. The exact archive names, sizes and
SHA-256 values are in [readiness](../plan/readiness.md#published-alpha-and-source).
A later candidate needs its own source, build and artifact fields; do not
inherit a pass merely because the version string is unchanged.

## States and evidence rules

| State | Meaning |
| --- | --- |
| **Pass** | The stated flow ran on the recorded source/artifact and environment. A build, metadata or virtual pass is limited to that scope. |
| **Failed** | A flow ran and the actual result violated a stated expectation. Retain the reproduction and link a bug; do not turn it into blocked. |
| **Blocked** | A prerequisite, environment, permission or artifact is missing, so the required flow cannot complete. Record what will unblock it. |
| **Skipped** | The flow was deliberately excluded, with a scope reason. It remains unverified; skipping a required gate needs an explicit support-scope decision. |

The rows below are acceptance results, not Linear workflow statuses. No native
row is passed. A missing desktop is blocked, not a confirmed product bug.
A partial supporting pass never replaces the row's full acceptance result.
There are no newly observed native input failures in this documentation-only
run. Isolated supporting checks ran separately; the updater harness stopped at
its safety guard before the required signed-app flow could start.

Follow [CONTRIBUTING.md](../../CONTRIBUTING.md): separate recoverable test
machines/accounts, local recovery keyboard/mouse, owned processes only, private
config/socket/TLS/log paths, explicit unused test ports and
`OPENERGY_ALLOW_PORT_24800=0`. Do not control the user's desktop or change live
sharing services, TCC, login items, clipboard or trust. Remote shell access to
macOS/Windows permits isolated checks, not live desktop input acceptance.

## Required run record

Create a dated report under `docs/verification/<area>-YYYY-MM-DD/README.md`.
Link it in this matrix and the [index](README.md). Split rows by actual
OS/build, architecture and role; a universal archive does not prove both CPUs.
Every row must link to the report section containing these fields:

| Field | Required content |
| --- | --- |
| Identity | Linear issue, case ID, operator, date/time/timezone; exact source SHA, dirty state, version/build number and archive/binary SHA-256. |
| Platform | OS version **and build**, architecture, backend, interactive/session-0/Xvfb context, hardware and local recovery path. |
| Role and peer | Host/client on both ends, peer product/version/source/artifact, network topology and assigned ports. |
| Input and display | Keyboard layouts/input methods, relevant key/remap/lock state, monitor bounds/origins, resolution/scaling/rotation and hotplug topology. Use explicit N/A where unrelated. |
| Security and install | TLS mode, compared-pin approval/rotation state (no private keys), signature/notarization/Gatekeeper state, install method/quarantine, permission grants and login setting. |
| Update | Old/new versions/builds/hashes, configured feed, HTTP status/item count, archive authentication, install/relaunch and post-update daemon state; N/A for non-update cases. |
| Procedure | Exact commands or human steps, harmless input/clipboard fixtures, expected result, actual result, failure/empty/recovery case and cleanup. |
| Outcome | Pass/failed/blocked/skipped **per case**, reason, remaining criteria, redacted raw logs/report/media links and related bug IDs. |

No real clipboard content, credentials, TLS private keys, personal hostnames or
unrelated input may enter retained evidence. Attach only redacted diagnostics.
Use [TEST-MAC](../../crates/platform-macos/TEST-MAC.md),
[focus lab](../../tools/focus-lab/README.md),
[Windows lifecycle](../../tools/windows/README.md),
[Linux lab](../../tools/linux-dev/README.md), and
[external E2E](../../tools/e2e/README.md) as starting points, not as proof that a
gate ran.

## Environment coverage to instantiate

These are **required test profiles**, not machines tested by this report.
Exact builds, layouts and scaling must be filled in by the operator.

| Profile | OS/build and architecture coverage | Roles and coverage |
| --- | --- | --- |
| Mac | macOS 15+ support floor; affected macOS 27.2 build for focus; arm64 and Intel installation coverage. Exact builds pending. Historical 27.0.1 is supporting focus evidence only. | Host and client; English/French/German; mixed Retina/non-Retina, offsets/negative origins; native pasteboard, signing, TCC, login and updates. |
| Windows | Exact Windows version/build pending for interactive acceptance, x86-64 published package. Headless checks on build 26340 are a separate scope if recorded. | Host and client; ordinary/elevated apps and UAC; per-app layouts/AltGr; mixed DPI; native clipboard; per-user install/logon/migration. |
| Physical X11 | Exact distribution/kernel/X server/desktop/build pending; x86-64 published package. Record other architectures as separate source builds. | Host and client; layouts/AltGr/keymap restore; real RandR/scaling/suspend; clipboard manager and INCR; no Wayland support. |

## Supporting evidence, separate from native acceptance

| Scope / related tasks | State | Date, evidence and limit |
| --- | --- | --- |
| Published source, archive inventory and byte hashes; [SER-204], [SER-213] | Pass (read-only provenance subset) | 2026-11-03: [release/tag/assets and three downloaded checksum matches](../plan/readiness.md#published-alpha-and-source). No signature execution or install claimed. |
| Native builds + virtual daemon scenarios; [SER-212], [SER-177] | Pass (CI scope) | 2026-11-02: [CI #41497579334](https://github.com/SergeSerb2/openergy/actions/runs/41497579334), source above; Unix 93/93, Windows smoke. S25 remap passed on Unix. Logs reviewed 2026-11-03. |
| Docker scenarios in those CI jobs; [SER-201], [SER-212] | Skipped | 30 excluded per Unix run under the default virtual suite. Full Docker interop remains required separately. |
| Native architecture packaging; [SER-213] | Pass (draft workflow scope) | 2026-11-02: [#41497602883](https://github.com/SergeSerb2/openergy/actions/runs/41497602883), four targets, extracted virtual IPC smoke. Draft macOS packages are ad-hoc, not final universal signature acceptance. |
| Unchanged published Mac app; [SER-194], [SER-196], [SER-212], [SER-213] | Pass (isolated artifact/app scope) | 2026-11-03: [M1 record](macos-candidate-2026-11-03/README.md), macOS 27.0.1 build 26A434 arm64, app 0.1.0 build 108; signature/staple/Gatekeeper and 25/25 virtual app checks. No quarantine, first-user install or native input proof. |
| Public feed GET and signature; [SER-198] | Pass (HTTP/public signature subset) | 2026-11-03: [channel record](update-channel-2026-11-03/README.md); Linux and two Macs, curl/Sparkle-style GETs return 200 with zero releases; bootstrap signature verified. No signed-app Sparkle execution. |
| Python hosting verifier; [SER-198] | Failed (client-specific HTTP check) | [Channel record](update-channel-2026-11-03/README.md): actual verifier exits 1 on HTTP 403; controlled user-agent comparison reproduces denial. This does not establish a universal outage. |
| Original signed app's updater harness; [SER-198], [SER-197] | Blocked | [M1 refusal](macos-candidate-2026-11-03/updater-release-refusal.json): exit 64 before Sparkle startup; production bundle ID and absent SUDefaultsDomain fail the disposable-fixture guard. Do not modify/re-sign the release and call it original-artifact acceptance. |
| Published Windows package; [SER-199], [SER-200], [SER-212], [SER-213] | Pass (private command/IPC/TLS subset) | 2026-11-03: [Windows record](windows-candidate-2026-11-03/README.md), NT 10.0.26340.0 x64, administrator SSH/session 0; archive/manifest/notices and unsigned state checked; 23 control, 19 deadlines, pipe identity, negative migration checks and approved mutual-TLS virtual pair passed. |
| Windows limited-user lifecycle/migration and Synergy; [SER-199], [SER-200], [SER-201] | Blocked | [Windows limits](windows-candidate-2026-11-03/README.md#open-acceptance-criteria): lifecycle guards refuse the administrator token, medium-token children/broker cannot run successfully in this session, and documented Synergy installation is absent. No guard bypass or installed-service change. |
| Linux virtual and Docker peer catalogue; [SER-201], [SER-212] | Pass (repeat and separate lab runs) | [Linux record](linux-candidate-2026-11-03/README.md): repeat 93/93 plus Docker 30/30 cover 123 cases; initial 91/93 retained. Build/audit, supplementary, Xvfb and teardown checks passed. Synergy 1.21.1 and native desktops remain open. |
| Older native focus, app flow, Xvfb, Windows private checks and peers | Pass (recorded historical scopes only) | [Historical record index](README.md#historical-records). Each report applies only to its own source, OS and setup. In particular, legacy focus passed on 27.0.1; affected-build acceptance stays blocked. |

Candidate validation and public-channel reports are collected in the
[current report index](README.md#current-candidate-records). Read their actual
results before updating a row. Isolated macOS/Windows command checks and
Linux/Xvfb/lab checks cannot close native desktop rows.

## Native input, layout, clipboard and recovery

All rows use the required fields above. “Mac”, “Windows” and “Physical X11”
refer to the required profiles, not a completed run. The final column records
the present blocker/supporting evidence; a completed result must replace it
with a dated report link.

| Task | Required environment / role | Real flow and recovery criteria | State | Evidence / blocker |
| --- | --- | --- | --- | --- |
| [SER-172] | Mac host | Keyboard, pointer, every button, drag and wheel across the edge; local cursor return; repeat after restart. | **Blocked** | Separate recoverable Mac desktop and physical input are not assigned to this run. |
| [SER-173] | Mac host/client | Hold Shift, a normal key and each button across owned-peer loss; reconnect; repeat with a stalled peer. | **Blocked** | OS pressed-state cleanup needs native desktop observation. |
| [SER-174] | Mac host/client | Sleep/wake both roles and reboot owned host; capture, screen size, clipboard and timed reconnect recover. | **Blocked** | Desktop sleep/reboot acceptance is not authorized on the live sharing setup. |
| [SER-175] | Mac host/client | Retina/non-Retina, negative origins, unequal heights and offset edges; hotplug/rearrange while connected. | **Blocked** | No recoverable mixed-display native test topology is available. |
| [SER-176] | Mac client (+ Mac host) | English/French/German; dead keys, accents, Alt/Option, keypad, repeat, layout changes and supplementary Unicode; balanced key-up. | **Blocked** | Native layouts and real application text/shortcuts have not been exercised. |
| [SER-177] | Mac client (+ real host) | Ctrl→Command, identity and none mappings; hold modifiers across crossings and local return; verify shortcuts and balanced releases. | **Blocked** | Virtual S25 is supporting evidence only; see [current source](../../crates/session/src/common.rs). |
| [SER-178] | Mac host/client | Text, HTML and images both ways in real apps; exact pixels/content; empty/unsupported/busy/over-limit and direction-off cases. | **Blocked** | No native pasteboard access or real-app copy/paste was performed. |
| [SER-179] | Affected Mac client | macOS 27.2 affected build: fresh legacy and numbered processes, S1–S7 (S7 first); active/key/order, menus, drag and first action. | **Blocked** | [27.0.1 lab](focus-lab-macos-27.0.1/README.md) did not reproduce legacy failure; daily use does not close this gate. |
| [SER-180] | Mac client | Right/other buttons, physical-click interleaving, drag, double-click and reconnect; standard/status menus and popovers without click-through. | **Blocked** | Native application/menu routing has not been exercised. |
| [SER-181] | Mac host/client | Enter/leave Secure Input and lock; controlled event-tap disable; visible status, local control and recovery without leaked input. | **Blocked** | Requires recoverable native desktop and explicit permission/lifecycle work. |
| [SER-182] | Windows host → Mac client | Interactive hooks/raw mouse: physical capture, relative motion, wheel/buttons, local return and hook recovery. | **Blocked** | Session-0/virtual checks and daily-use reports do not establish desktop acceptance. |
| [SER-183] | Windows client ← real host | Ordinary apps: scan/extended keys, shortcuts, motion/drag/wheel; stop/restart/disconnect releases all input. | **Blocked** | No interactive client-injection acceptance has run. |
| [SER-184] | Windows host/client | Ordinary/elevated apps and enter/leave UAC: no error flood or coordinate leakage; recover; document secure-desktop restrictions. | **Blocked** | Headless access cannot prove UIPI/UAC desktop behavior. |
| [SER-185] | Windows host/client | Multiple scales, negative origins, offsets, hotplug; pointer mapping, clip/visibility restoration and packaged DPI behavior. | **Blocked** | No controlled mixed-DPI interactive topology is assigned. |
| [SER-186] | Windows host/client | English/French/German, AltGr/dead keys, shortcuts/keypad and per-app layout changes; balanced key-up and visible unsupported-key errors. | **Blocked** | Physical layouts and application behavior have not been checked. |
| [SER-187] | Windows ↔ Mac | Real apps: Unicode text, HTML, DIB/DIBV5; busy clipboard, empty/unsupported/oversize data and own-write echo suppression. | **Blocked** | Private IPC checks do not exercise real desktop clipboard ownership. |
| [SER-188] | Windows host/client | Held keys/buttons across peer loss, owned stop, sleep/wake and reconnect; cursor state restored, no duplicate hooks. | **Blocked** | Native hooks, cursor and pressed-state cleanup require desktop acceptance. |
| [SER-189] | Physical X11 host/client | Physical keyboard/mouse in real apps; capture/return, drag and scroll; an app rejecting X SendEvent. | **Blocked** | Isolated Xvfb is not a physical X11 desktop. |
| [SER-190] | Physical X11 host/client | English/French/German, AltGr/dead/repeat/lock keys; disconnect/restart restores spare and prior external mappings. | **Blocked** | Physical keymap restoration has not been observed. |
| [SER-191] | Physical X11 host/client | Multiple monitors/offsets, RandR, suspend/resume and abrupt owned-process loss; grabs/cursor/keys/reconnect recover. | **Blocked** | Physical displays, suspend and desktop inhibition remain unrun. |
| [SER-192] | Physical X11 ↔ peer | Real apps/clipboard manager: text/HTML/images and large INCR; owner change/exit, unsupported/oversize recovery; PRIMARY stays local. | **Blocked** | Xvfb results cannot close real desktop selection/manager behavior. |
| [SER-193] | Windows ↔ Mac and X11 | Non-sensitive real-app DIB/DIBV5/PNG fixtures; compare decoded pixels; explicit unsupported rejection preserves prior clipboard. | **Blocked** | Real Windows fixtures and cross-desktop transfers have not been collected. |

## Install, updates and compatibility

| Task | Required environment / role | Real flow and recovery criteria | State | Evidence / blocker |
| --- | --- | --- | --- | --- |
| [SER-194] | Mac arm64 AND Intel, fresh accounts | Exact downloaded archive: quarantine, Developer ID, notarization, staple and Gatekeeper; first Host/Join; minimum supported OS coverage. | **Blocked** | [M1 signatures/Gatekeeper pass](macos-candidate-2026-11-03/README.md), but curl/ditto left no quarantine. Fresh-user flow, Intel and support-floor acceptance remain open. |
| [SER-195] | Mac host/client, fresh permissions | Start denied; grant Accessibility/Input Monitoring; live status and sharing; revoke/regrant/relaunch and signing/build change. | **Blocked** | No native permission changes or sharing were performed on a disposable desktop. |
| [SER-196] | Mac host/client, disposable login | Enable/disable login startup; logout/login or reboot; owned app/daemon crash; single instance and config/trust/role retained. | **Blocked** | [25/25 headless app flow](macos-candidate-2026-11-03/README.md) includes owned-daemon SIGKILL recovery; login-item/session transitions remain unrun. |
| [SER-197] | Mac old → new signed artifacts | Check/download/authenticate/install/relaunch; preserve config/trust/login; bundled daemon reconnect; cancel/interrupted-download recovery. | **Blocked** | Requires signed old/new artifacts and separate update-install state; feed/HTTP success is insufficient. |
| [SER-198] | Signed Mac app + independent external client | Configured appcast: HTTP status, item count and signature/source; inspect access rules on legitimate denial; distinguish GitHub from feed publication. | **Blocked** | [HTTP clients read a signed empty feed](update-channel-2026-11-03/README.md); Python verifier fails 403. [Unchanged signed app refuses private harness](macos-candidate-2026-11-03/updater-release-refusal.json), so signed-app feed access is not proven. |
| [SER-199] | Windows interactive test user | Per-user install/path/pipe ownership; start/stop/restart, real logon startup, uninstall removes only owned entries/processes/hooks. | **Blocked** | [Lifecycle guards refuse the administrator SSH token](windows-candidate-2026-11-03/README.md#open-acceptance-criteria); limited-user lifecycle, real logon and uninstall remain unrun. |
| [SER-200] | Windows disposable Synergy setup | Preflight → pending → commit → rollback; trust/readiness, startup restoration and practical failure recovery. | **Blocked** | [Negative checks ran](windows-candidate-2026-11-03/README.md); medium-token children/broker could not complete in session 0. Interactive pending/commit/rollback and startup restoration remain unrun. |
| [SER-201] | All claimed platforms, both roles | Deskflow 1.27, Barrier 2.4, Synergy core 1.21.1: TLS approval, wrong pin, reconnect, input and clipboard; lab-only plaintext. | **Blocked** | [Current Docker cases passed 30/30](linux-candidate-2026-11-03/README.md): Deskflow 1.27.0/Barrier 2.4.0, both roles. Synergy 1.21.1 and full native peer acceptance remain open. |
| [SER-202] | Input Leap, both roles (optional) | Select exact version; TLS approval/wrong pin, input and clipboard; publish observed limits per platform. | **Skipped** | Optional compatibility expansion is not adopted into current support; no Input Leap compatibility claim. |
| [SER-203] | Mac app host/client, fresh identities | No sharing before compared-pin approval; reject wrong pin, rotate identity and revoke live peer; cutoff without silent retrust. | **Blocked** | Virtual trust checks do not close fresh-user app/desktop acceptance. |

## Final candidate and release decision

| Task | Required environment / role | Completion criteria | State | Evidence / blocker |
| --- | --- | --- | --- | --- |
| [SER-211] | Claimed native platforms, host/client | Controlled idle, active input, large clipboard and stalled-peer load; sustained CPU/RSS/latency and stated baseline. | **Blocked** | No controlled native benchmark on exact artifacts; old field metrics remain historical. |
| [SER-212] | All candidate targets | Pinned format/lint/locked build, dependency/license/notice/secret audit; full real-process/Docker, Windows private checks and macOS app flow. | **Blocked** | [Linux repeat 93/93 + Docker 30/30](linux-candidate-2026-11-03/README.md), Mac app 25/25 and Windows private subsets passed; initial failures/scanner findings retained. Complete per-target build/audit and candidate acceptance remain open. |
| [SER-213] | All final download targets | Archive checksum/manifests/source/notices; macOS signing/notarization/staple/Gatekeeper; extracted private CLI/TLS/IPC smoke. | **Blocked** | [Mac](macos-candidate-2026-11-03/README.md), [Windows](windows-candidate-2026-11-03/README.md) and [Linux](linux-candidate-2026-11-03/README.md) archive/isolated smoke subsets passed. Mac ZIP lacks source BUILD.json/provenance sidecar; stable final-artifact gates remain open. |
| [SER-214] | Maintainer, all claimed scope | Review required rows and follow-up bugs; approve precise support/recovery claims or record explicit scope deferrals before publication. | **Blocked** | Native and install/update gates remain open; no release approval is implied. |

[SER-204] owns current readiness; [SER-205] owns this matrix and failure triage.
Dependency reviews SER-206–SER-210 are tracked in the
[complete Linear task index](https://linear.app/sergedev/document/complete-task-index-milestones-and-backlog-a96a1d5ad7d3).
They are review decisions, not executed acceptance flows. Optional future
features SER-215–SER-222 are outside current scope unless explicitly adopted;
Wayland, file transfer, PRIMARY sharing and Windows/Linux GUIs/updaters stay
unsupported. Input Leap is an explicit optional skipped row, not presumed
compatible.

## Triage and next execution order

1. Start with clean Mac installs and permission cases ([SER-194], [SER-195]).
   Then run affected-build focus and recent held-key/display/remap cases
   ([SER-179], [SER-173], [SER-175], [SER-177]).
2. Confirm public update-channel access ([SER-198]); use separately signed
   old/new artifacts for complete update installation ([SER-197]).
3. Complete the remaining per-platform native rows, interactive lifecycle and
   fresh trust flow. Retest peers ([SER-201]) on the selected candidate.
4. Run full exact-candidate checks ([SER-212]) and final download/signature
   acceptance ([SER-213]); review all required gates under [SER-214].

For every actual failure, stop only the owned fixture, preserve redacted
diagnostics, and record expected vs actual behavior with the exact reproduction
and source/artifact/OS tuple. Search Linear for a duplicate before filing an
actionable bug. Link that bug in the failed row and the originating task. A
retest gets a new dated result; keep the original failure and its evidence.
The parent/maintainer performs Linear updates for this run. No bug is filed
solely because a flow is blocked.

A skipped required case needs a named support-scope exception approved at the
release decision. Keep Windows/Linux desktop support experimental if their
native gates are deferred. Green build/virtual results, a fixed source path,
or publication of an alpha never substitutes for that decision.

[SER-172]: https://linear.app/sergedev/issue/SER-172/macos-verify-native-host-capture-and-screen-return
[SER-173]: https://linear.app/sergedev/issue/SER-173/macos-verify-held-key-and-button-cleanup-on-disconnect
[SER-174]: https://linear.app/sergedev/issue/SER-174/macos-verify-sleep-wake-and-host-reboot-recovery
[SER-175]: https://linear.app/sergedev/issue/SER-175/macos-verify-mixed-scale-displays-offsets-and-hotplug
[SER-176]: https://linear.app/sergedev/issue/SER-176/macos-verify-keyboard-layouts-dead-keys-and-unicode
[SER-177]: https://linear.app/sergedev/issue/SER-177/macos-verify-modifier-remaps-across-screen-crossings
[SER-178]: https://linear.app/sergedev/issue/SER-178/macos-verify-native-clipboard-text-html-and-images
[SER-179]: https://linear.app/sergedev/issue/SER-179/macos-272-prove-numbered-click-focus-against-legacy-mode
[SER-180]: https://linear.app/sergedev/issue/SER-180/macos-extend-focus-acceptance-to-rightother-buttons-and-menus
[SER-181]: https://linear.app/sergedev/issue/SER-181/macos-verify-secure-input-lock-and-event-tap-recovery
[SER-182]: https://linear.app/sergedev/issue/SER-182/windows-verify-interactive-host-hooks-and-raw-mouse-capture
[SER-183]: https://linear.app/sergedev/issue/SER-183/windows-verify-client-input-injection-through-real-applications
[SER-184]: https://linear.app/sergedev/issue/SER-184/windows-document-and-verify-uac-and-elevated-app-behavior
[SER-185]: https://linear.app/sergedev/issue/SER-185/windows-verify-mixed-dpi-monitor-hotplug-and-cursor-recovery
[SER-186]: https://linear.app/sergedev/issue/SER-186/windows-verify-altgr-dead-keys-and-per-app-keyboard-layouts
[SER-187]: https://linear.app/sergedev/issue/SER-187/windows-verify-clipboard-ownership-contention-and-rich-formats
[SER-188]: https://linear.app/sergedev/issue/SER-188/windows-verify-held-input-sleepwake-and-forced-disconnect-recovery
[SER-189]: https://linear.app/sergedev/issue/SER-189/linux-x11-verify-physical-desktop-capture-and-injection
[SER-190]: https://linear.app/sergedev/issue/SER-190/linux-x11-verify-non-us-layouts-and-keymap-restoration
[SER-191]: https://linear.app/sergedev/issue/SER-191/linux-x11-verify-display-changes-sleep-and-process-loss-recovery
[SER-192]: https://linear.app/sergedev/issue/SER-192/linux-x11-verify-real-desktop-clipboard-and-large-incr-transfers
[SER-193]: https://linear.app/sergedev/issue/SER-193/verify-real-windows-image-variants-across-all-clipboard-backends
[SER-194]: https://linear.app/sergedev/issue/SER-194/macos-verify-a-clean-downloaded-install-on-apple-silicon-and-intel
[SER-195]: https://linear.app/sergedev/issue/SER-195/macos-verify-permission-deny-grant-revoke-and-regrant-flows
[SER-196]: https://linear.app/sergedev/issue/SER-196/macos-verify-open-at-login-and-appdaemon-crash-recovery
[SER-197]: https://linear.app/sergedev/issue/SER-197/macos-prove-a-complete-signed-sparkle-update-and-relaunch
[SER-198]: https://linear.app/sergedev/issue/SER-198/verify-public-sparkle-feed-access-and-reconcile-update-channel-docs
[SER-199]: https://linear.app/sergedev/issue/SER-199/windows-verify-per-user-install-logon-stop-and-uninstall
[SER-200]: https://linear.app/sergedev/issue/SER-200/windows-verify-synergy-migration-commit-and-rollback-interactively
[SER-201]: https://linear.app/sergedev/issue/SER-201/retest-supported-deskflow-barrier-and-synergy-peers-on-the-release
[SER-202]: https://linear.app/sergedev/issue/SER-202/establish-an-input-leap-compatibility-matrix
[SER-203]: https://linear.app/sergedev/issue/SER-203/verify-first-use-trust-wrong-pins-and-live-revocation-in-the-app
[SER-204]: https://linear.app/sergedev/issue/SER-204/refresh-readiness-and-verification-records-for-the-published-alpha
[SER-205]: https://linear.app/sergedev/issue/SER-205/maintain-a-native-acceptance-matrix-and-triage-real-flow-failures
[SER-211]: https://linear.app/sergedev/issue/SER-211/measure-sustained-native-cpu-memory-and-input-latency
[SER-212]: https://linear.app/sergedev/issue/SER-212/run-the-full-exact-candidate-build-audit-and-real-process-suite
[SER-213]: https://linear.app/sergedev/issue/SER-213/verify-final-artifact-provenance-signatures-and-download-smoke-checks
[SER-214]: https://linear.app/sergedev/issue/SER-214/review-stable-release-gates-and-make-the-release-decision
