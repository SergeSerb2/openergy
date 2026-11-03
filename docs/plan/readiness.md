# Readiness

Status date: 2026-11-03 (America/Vancouver). Openergy `0.1.0` is a
**published alpha**, not a stable desktop release. Keep another way to control
each computer while you try it.

## Published alpha and source

[v0.1.0](https://github.com/SergeSerb2/openergy/releases/tag/v0.1.0) was published
on 2026-11-02 at 10:27:22 America/Vancouver (18:27:22 UTC). The source is
`e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`. GitHub's annotated tag object
`eef40ced98e8ae8357997b1a110c63a1cca62419` peels to that commit. The release
is marked prerelease.

On 2026-11-03, read-only GitHub checks confirmed the tag, release metadata,
workflow results and eight release assets. All three archives were downloaded
to private temporary storage with `gh release download`. Their SHA-256 values
matched the downloaded checksum sidecars and GitHub asset digests:

| Published archive | Bytes | SHA-256 |
| --- | ---: | --- |
| [macOS universal ZIP](https://github.com/SergeSerb2/openergy/releases/download/v0.1.0/Openergy-0.1.0-macos-universal.zip) | 7,780,962 | `acd5c81113670edd34658d14ca489ce083758acc4df1ae5676b4308bcda6e9b0` |
| [Windows x86-64 unsigned ZIP](https://github.com/SergeSerb2/openergy/releases/download/v0.1.0/openergy-0.1.0-x86_64-pc-windows-msvc-unsigned.zip) | 2,051,082 | `abd5d4f80f1141e5893881d82bdfa0dd5f93f49a8d3a43a863801f8bb9cd53b9` |
| [Linux x86-64 unsigned tar.gz](https://github.com/SergeSerb2/openergy/releases/download/v0.1.0/openergy-0.1.0-x86_64-unknown-linux-gnu-unsigned.tar.gz) | 2,299,135 | `e866b3a5475fe9d41464fc69e5a2d805b24454122476f6ece888a34ae08321e9` |

Each archive has a `.sha256` asset at the same release. The Windows and Linux
JSON sidecars also record the exact source above, `dirty: false`, Rust 1.99.1,
third-party notices present, unsigned state, and CLI/virtual IPC smoke scope.
The release notes describe the published macOS download as Developer ID signed
and notarized. Checking its bytes on Linux does **not** verify that signature,
notarization, Gatekeeper, a clean install, or an installed update. The draft
workflow's ad-hoc per-architecture macOS archives are distinct from the final
signed universal download.

Recheck the existing records without starting a workflow:

```sh
gh release view v0.1.0 --repo SergeSerb2/openergy \
  --json tagName,targetCommitish,publishedAt,isPrerelease,url,assets
gh api repos/SergeSerb2/openergy/git/ref/tags/v0.1.0
gh api repos/SergeSerb2/openergy/git/tags/eef40ced98e8ae8357997b1a110c63a1cca62419
gh run view 41497579334 --repo SergeSerb2/openergy --json headSha,status,conclusion,jobs
gh run view 41497602883 --repo SergeSerb2/openergy --json headSha,status,conclusion,jobs
```

## Latest verification run

Release-source CI ran on **2026-11-02**, at the exact source above. The metadata
and logs were inspected on **2026-11-03**. This is not a local rerun of CI.

| Evidence | Observed result | Scope and exclusions |
| --- | --- | --- |
| [Native verification #41497579334](https://github.com/SergeSerb2/openergy/actions/runs/41497579334) | All three jobs succeeded: macOS `macos-15` arm64, Linux `ubuntu-24.04` x86-64, Windows `windows-2025` x86-64. | Locked native builds, formatting, strict Clippy, notices and real CLI/virtual IPC checks. Linux and macOS each report **93/93 passed; 30 excluded**. Windows reports `suite=smoke`. |
| [Manual draft release #41497602883](https://github.com/SergeSerb2/openergy/actions/runs/41497602883) | Validation, all four builds, and draft job succeeded. | macOS arm64 and Intel, Windows x86-64, Linux x86-64: exact-tag checks, verification, archive build and extracted virtual IPC smoke. Each Unix job reports **93/93 passed; 30 excluded**. Windows runs smoke. The last job creates a draft only. |

The [verification workflow](../../.github/workflows/verify.yml) calls
[release verification](../releasing.md) with `--e2e-slowdown 3`. It excludes
Docker interop by default. It compiles the Swift app on macOS but does not run
packaged app E2E, native desktop input, clipboard, TCC, login or an update
install. The [draft workflow](../../.github/workflows/draft-release.yml) also
uses isolated virtual runtime checks. These green jobs do not close the
[acceptance matrix](../verification/native-acceptance.md).

The README's footprint numbers come from the historical October 25 field
measurements below. They were not remeasured by November 2 CI.

## Current behavior and evidence boundaries

- Modifier remaps apply to modifier key down/repeat/up packets **and** modifier
  masks; mapping to `none` drops that modifier. See
  [session source](../../crates/session/src/common.rs) and
  [current option limits](../../crates/config/V1-LIMITS.md). November 2 CI
  passed `S25-modifier-key-remap` on Unix virtual backends. Native shortcuts
  and held-modifier crossings remain open under
  [SER-177](https://linear.app/sergedev/issue/SER-177/macos-verify-modifier-remaps-across-screen-crossings).
- The [changelog](../../CHANGELOG.md) records shipped changes for held-key
  crossings, touching-display geometry, final releases to stalled peers,
  permission-grant detection and reconnect. Source and virtual results do not
  prove their native desktop edge cases.
- Sparkle uses the HTTPS appcast in
  [Info.plist](../../apps/macos/Support/Info.plist) and validates the bundle feed
  and Ed25519 key in
  [AppUpdater.swift](../../apps/macos/Sources/Openergy/App/AppUpdater.swift).
  The October 23 signed empty-feed record is historical. A published GitHub
  release does not establish a readable or populated Sparkle feed. See
  [channel guidance](../macos-updates.md) and the
  [install/update rows](../verification/native-acceptance.md#install-updates-and-compatibility)
  for the separate public-access and full install/relaunch gates.
- The [daemon report](../../crates/daemon/VERIFICATION.md) adds current context
  above its superseded modifier-remap limit. Its old counts and statements
  remain historical evidence.

New candidate reports belong in the [verification index](../verification/README.md).
Each must state its source, artifact, environment and result. A report in
progress supplies no pass.

## November 3 candidate follow-up

- The [M1 candidate record](../verification/macos-candidate-2026-11-03/README.md)
  checks the unchanged published app, version 0.1.0 build 108, on macOS 27.0.1
  build 26A434, arm64. Signature, stapled-ticket and Gatekeeper checks passed;
  the headless virtual app flow passed 25/25. Curl/ditto did not attach
  quarantine, so this does not prove a fresh downloaded user install. Native
  input, permissions, login and affected-build focus remain open.
- The [channel record](../verification/update-channel-2026-11-03/README.md)
  reports HTTP 200 and zero releases from curl and Sparkle-style GETs on Linux
  and two Macs, with a verified signed empty bootstrap feed. The Python
  verifier's 403 is client-specific. The unchanged signed app's private updater
  harness refuses with exit 64 before Sparkle startup: its production bundle
  identifier and absent `SUDefaultsDomain` do not meet the disposable-fixture
  guard. No signed-app feed request or installed update is established.
- The [Windows candidate record](../verification/windows-candidate-2026-11-03/README.md)
  checks the published unsigned x86-64 package on Windows build 26340 through
  session-0 SSH: 23 control checks, 19 IPC deadline checks, pipe identity and
  an approved mutual-TLS virtual pair passed. Lifecycle suites correctly
  refused the administrator token. Limited-token migration and Synergy peer
  checks remain blocked; no interactive desktop or logon gate is closed.
- The [Linux candidate record](../verification/linux-candidate-2026-11-03/README.md)
  reports a strict repeat of **93/93** virtual scenarios on Linux and **30/30**
  Docker peer cases using a macOS virtual runner. Together these separate runs
  cover all 123 catalogue cases. The initial Linux run was 91/93; A7 lock
  observation and S15 latency failures remain recorded, followed by passing
  focused retries and the full repeat. Pinned build, format, strict Clippy,
  notices, cargo-deny, supplementary checks, Xvfb and teardown passed. Secret
  scans exited 1; all 93 findings per scan were reviewed as hash fields.
  This is not physical X11 or native input-latency acceptance.
- Published Windows and Linux packages passed extracted private TLS/IPC smoke
  (Linux x86-64 under emulation). The Mac ZIP has no source `BUILD.json` or
  provenance sidecar. Release/tag metadata links its intended source, but does
  not independently establish binary-to-source provenance. SER-213 remains open.

## Historical pre-release verification — 2026-10-25

The former readiness snapshot reported the following results on its recorded
builds and machines. They are retained for context, not attributed to the
November 2 release artifacts. No check below was rerun for this docs refresh.

- Rust: `cargo fmt --check`, strict workspace Clippy (`-D warnings`), locked
  release build, `cargo deny check`, and the notice check passed.
- Real-process suite: 119/119 scenarios on an Apple M5 Pro (macOS 27.2),
  including the Deskflow and Barrier Docker cases. Linux checks passed in Docker
  (89/89 without the Docker-in-Docker cases) and in an isolated Xvfb server.
- macOS app: a Developer ID signed, notarized, and stapled build was accepted
  by Gatekeeper on this Mac and, as a quarantined download, on a second Mac
  (macOS 27.0.1). There it passed the 25-check headless app flow.
- Windows, Rust 1.99.1 MSVC: release build and strict Clippy passed.
  Private-instance scripts passed: control (23 checks), IPC deadlines (19/19),
  lifecycle (15), lifecycle ownership (11), and pipe identity.
- Footprint: arm64 app bundle 9,276 KiB on disk; daemon alone 8.47 MiB RSS and
  0.0% CPU over 60 s idle. The installed app and daemon used 64.5 MiB together
  as a connected client. See the [comparison](../../README.md#openergy-vs-synergy).
- Fixed in this run: the menu-bar app redrew a hidden animation in the client
  role and used 21–29% CPU. With the fix it uses 0.0% in the same scenario.

## Historical records

Each record covers the commit and machine it ran on. Resource numbers depend
on the machine, build, and workload.

| Area | What was checked | Record |
| --- | --- | --- |
| Daemon and CLI | Real processes over local IPC and loopback TCP/TLS with the virtual backend: input, clipboard, trust, restart, recovery. | [round5](../verification/release-integration-2026-10-22/round5.md), [daemon](../../crates/daemon/VERIFICATION.md) |
| macOS app | Packaged app and bundled daemon: 25/25 headless checks against a virtual peer, including reconnect and crash recovery. | [app flow](../verification/macos-app-current-flow-2026-10-23/README.md) |
| macOS updates | Sparkle checks against local test feeds and the live, empty, signed feed (2026-10-23). | [Sparkle](../verification/sparkle-integration-2026-10-23/README.md) |
| macOS focus | Numbered clicks pass all seven focus scenarios on macOS 27.0.1. On macOS 27.2, daily use as a client of a Windows host since 2026-10-21, with clicks focusing windows. | [focus lab](../verification/focus-lab-macos-27.0.1/README.md) |
| Linux X11 | Capture, injection, and clipboard in an isolated Xvfb server on ARM64. The x86-64 package was checked under emulation (2026-10-22). | [X11](../../crates/platform-linux/VERIFICATION.md), [x86-64](../verification/release-integration-2026-10-22/linux-amd64-python311.md) |
| Windows | Daemon and CLI over private named pipes, with virtual TLS peers in both roles. As a host, daily use since 2026-10-21 with a Mac client: keyboard, mouse, and clipboard. | [Windows](../verification/windows-daemon-2026-10-21/README.md) |
| Other peers | Deskflow 1.27 and Barrier 2.4 in a Docker lab, both roles, TLS and plaintext. Synergy core 1.21.1 on Windows, both roles, TLS and clipboard (2026-10-21). | [interop lab](../../tools/interop-lab/README.md), [Synergy](../verification/windows-synergy-1.21.1/README.md) |
| Licenses and secrets | cargo-deny, notice generation, and secret scans of the tree and full history; no real secrets found (2026-10-25). | [audit](../verification/public-release-audit-2026-10-21/README.md) |

## Not yet verified

The [native acceptance matrix](../verification/native-acceptance.md) links every
native, install, update, compatibility and final-candidate task. No native
desktop acceptance is newly marked passed in this refresh. Windows and physical
X11 checks need recoverable interactive desktops; headless or session-0 checks
cannot close them.

- Native input on test desktops: held keys across a disconnect, sleep and
  wake, mixed display scaling, and keyboard layouts.
- A controlled side-by-side focus test on macOS 27.2, where the bug is seen.
- Windows as a client: injection, clipboard, elevated apps, and UAC.
- A first install by a new user: granting and revoking permissions, and the
  login item.
- Downloading, installing, and relaunching an update through Sparkle.
- Input Leap and other Synergy versions.
- A lab benchmark against Synergy on the same workload. The README comparison
  is a field measurement on one Mac.

## Known limits

- macOS is the main platform. Windows and Linux have no graphical app or
  automatic updates. Wayland is not supported.
- No file transfer or drag-and-drop. Clipboard formats differ by backend and
  peer. The 64 MiB clipboard limit is local; a peer can accept less. X11
  PRIMARY selection is not shared.
- Secure Input, screen lock, UAC, and OS input policies can pause sharing.
- Config and IPC formats can change before a stable release. More limits are in
  [V1-LIMITS.md](../../crates/config/V1-LIMITS.md).

## Release gates

A stable release needs all of these on the exact candidate commit and artifacts.
See the [matrix](../verification/native-acceptance.md) for task links and evidence fields:

1. Build, format, lint, dependency and license checks, and the real-process
   scenario suite pass.
2. Native input and clipboard checks pass on a separate test machine or
   account that has another input path. Virtual checks do not count.
3. A fresh macOS install passes: signing, notarization, permissions, login
   item, and recovery.
4. The focus check passes on an affected macOS build.
5. Interactive Windows checks pass before Windows is called desktop-ready.
   Real X11 desktop checks pass before broad Linux support is claimed.
6. Third-party peers are retested, with versions, roles, TLS, and trust
   settings recorded, including a wrong-pin case.
7. A complete signed Sparkle download/install/relaunch and post-update connection.
8. Review of every blocked/failed/skipped result and an explicit maintainer
   decision under [SER-214](https://linear.app/sergedev/issue/SER-214/review-stable-release-gates-and-make-the-release-decision).
   A support-scope deferral stays visible and never becomes a test pass.

Alpha releases list every check that did not run. Report security problems
through [SECURITY.md](../../SECURITY.md). The maintainer steps are in
[releasing](../releasing.md).
