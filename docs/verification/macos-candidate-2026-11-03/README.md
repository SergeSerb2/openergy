# Downloaded macOS alpha verification, 2026-11-03

The published **0.1.0 alpha, build 108** passed signature, notarization-ticket,
stapling and Gatekeeper checks on an Apple M1 running macOS **27.0.1, build
26A434, arm64**. Its unchanged packaged app passed the existing headless
`AppEndToEnd` flow: **25/25 steps, exit 0, 3.671 seconds**. The bundled daemon
also passed a separate real CLI/IPC/TLS smoke check with fresh identities.
No normal app launch, desktop control, native input/clipboard, login change or
TCC permission request was performed.

These are fresh checks of the published alpha. They do not close the full
stable-candidate or native-desktop acceptance issues. In particular, the signed
release's updater harness **refuses before Sparkle startup**, so this run does
not prove an actual signed-app feed request or a signed update installation.

## Artifact and source

- Repository source inspected: `e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`.
- Published release: [v0.1.0](https://github.com/SergeSerb2/openergy/releases/tag/v0.1.0).
- Annotated tag object: `eef40ced98e8ae8357997b1a110c63a1cca62419`, pointing to
  that source commit. The tag itself is unsigned.
- Archive: `Openergy-0.1.0-macos-universal.zip`, 7,780,962 bytes.
- Archive SHA-256:
  `acd5c81113670edd34658d14ca489ce083758acc4df1ae5676b4308bcda6e9b0`.
  Both the downloaded `.sha256` file and GitHub asset digest agree.
- App executable SHA-256:
  `b8649a96645c5d9dd95c886856159a3b35d414dc753eb0a606890ef742d15277`.
- Bundled daemon SHA-256:
  `4735c0666cd26b6b6a137129c0f4636256c456fbd1a8c3309ff20d1290284797`.
- App and daemon contain both `x86_64` and `arm64` slices. Only arm64 ran.
- The bundle declares macOS 15.0 as its minimum. macOS 15 was not tested.
- Embedded `LICENSE-MIT`, `LICENSE-APACHE` and `THIRD_PARTY_NOTICES.md` match
  the inspected source byte for byte. The separate Sparkle license is present.

The release/tag identifies the intended source. The macOS ZIP has no embedded
build manifest with a source SHA. This check does not independently establish
reproducible binary-to-source provenance. No replacement app was built or signed.
`Package.swift` defines the `Openergy` executable, targets macOS 15 and pins
Sparkle 2.10.0. No SwiftPM build or unit test was needed for the downloaded app.

## Executed checks

[artifact-inspection.json](artifact-inspection.json) records the bundle plist,
all regular-file hashes and exact command results. The app, bundled daemon and
Sparkle framework pass strict `codesign` verification. Deep verification also
passes for the nested Sparkle helpers. The app and daemon use Developer ID
Application `Serge Serbinenko (78A5P57U23)` with the hardened-runtime flag.
`xcrun stapler validate -v` succeeds, including an Apple ticket lookup.
`spctl --assess --type execute --verbose=4` returns exit 0 and
`source=Notarized Developer ID`.

`curl` downloaded the ZIP directly into a new owned mode-0700 temporary
directory on the M1; `ditto -x -k` extracted it there. Neither the ZIP nor the
extracted app has `com.apple.quarantine`. This is an observed limitation of
this download path, not a simulated quarantined first launch. No quarantine
attribute was added or removed, and the app was not installed into Applications.
The initial inspection script ran the checksum command from the wrong working
directory; it was corrected to use the archive directory and rerun successfully.

[app-flow.json](app-flow.json) records the existing packaged `--e2e` command,
its environment, all 25 step results, app/peer logs and cleanup. The app used
`OPENERGY_E2E_NO_RENDER=1`, `OPENERGY_DISABLE_DISCOVERY=1`, explicit config,
TLS-directory and socket paths, and `--platform virtual`. Its initial config
directory did not exist. A real owned virtual peer listened on
`127.0.0.1:25760`; the app's Host flow used `127.0.0.1:25761`. TLS was **off**
in this existing app harness. Checks covered:

- Safe off-role startup, Join, real peer connection, disconnect and rejoin.
- Clipboard configuration toggles, layout/link preservation and live logs.
- Invalid-layout rollback and ordered Join/Off actions.
- A stopped owned daemon during a large IPC write, followed by bounded failure.
- Killing the app's owned daemon and observing automatic supervisor recovery.
- Import warnings and the empty-import error path.

A separate invocation with an existing config directory correctly refused with
exit 64 before starting the model. This confirms the packaged harness's private
state guard. It did not test UI rendering or app-process crash/relaunch.

[tls-smoke.json](tls-smoke.json) records the bundled CLI and two real virtual
daemons on `127.0.0.1:25762`. TLS and client trust were explicitly enabled. Two
fresh private certificates were created with `fingerprint --dir ... --synergy`.
The client held the unknown server; the server then held the unknown client.
Each pending fingerprint matched the other node's offline fingerprint. Live
`trust.approve` calls allowed mutually trusted TLS connections. An all-zero
explicit pin produced `tls_pin_mismatch` without a peer or plaintext fallback.
The correct explicit pin connected. `trust.revoke` then disconnected the client,
removed its stored server pin, and made the peer require approval again.

The TLS check includes 11 real status/ledger checkpoints. Before approval, its
virtual ledger contained no input event and held no key or button; lifecycle
`release_all` records are permitted. It did not inject input to test post-revoke
delivery cutoff, rotate a certificate, or drive the app's trust UI. Two initial
controller errors are retained, not counted as product failures or passes:
[fingerprint-format comparison](tls-smoke-initial-controller-error.json) and
[empty-ledger assumption](tls-smoke-initial-ledger-error.json). The final
controller normalizes the CLI's v2 format and IPC's colon format and distinguishes
lifecycle release records from input events. Each retry used fresh private state;
each attempt stopped its own daemons cleanly.

[postflight.json](postflight.json) confirms all 96 bundle regular files are
unchanged after these checks, deep strict signature verification still passes,
no private process remains, and none of ports 25760–25762 has a listener.
Port 24800, installed sharing services and live account state were not used.

## Update-channel observation and signed-app blocker

The unchanged signed app embeds:

```text
SUFeedURL=https://openergy-updates.sergeserbinenko.com/appcast.xml
SUPublicEDKey=T6P8IZ5N6ivIZD3SdPRTOKWImB81Lfj2PFQqH4TMZ6o=
```

A real M1 `curl` GET of that bundle-configured URL returned **HTTP 200**,
756 bytes, SHA-256
`c693230197b548eba00fd2ea25a7a696cef218d1e854410f0c68934bcc6dd2d7`,
and **zero release items**. This is a command-line observation, not a Sparkle
request. The [channel record](../update-channel-2026-11-03/README.md) covers
other user agents, feed-signature verification and deployment limits.

[updater-release-refusal.json](updater-release-refusal.json) records an actual
invocation of the unchanged downloaded app:

```sh
CFFIXED_USER_HOME="$private_run/updater-private-home" \
  "$private_run/extracted/Openergy.app/Contents/MacOS/Openergy" \
  --updater-e2e "$private_run" \
  --updater-config "$private_run/updater-private-home" \
  --updater-feed https://openergy-updates.sergeserbinenko.com/appcast.xml \
  --updater-expect current --updater-timeout 15
```

The run also supplied explicit private daemon/config/socket/TLS paths, virtual
daemon arguments, disabled discovery and the port-24800 guard. The private home
existed with mode 0700 and the signed bundle was inside the private output root.
The command returned **64 in 0.014 seconds**, with no updater result or file in
the private home. The preflight requires a bundle identifier beginning with
`dev.openergy.updater-e2e.` and `SUDefaultsDomain` pointing to the private
`sparkle-preferences` domain. The release's identifier is `dev.openergy.app`
and `SUDefaultsDomain` is absent. These guards fail before `AppUpdater` or
Sparkle starts. Changing `Info.plist` would invalidate the release signature;
that substitution was not performed. Normal GUI startup was not permitted.
Even a prepared private updater fixture stops before archive download and
installation, so it cannot prove the full signed-update criterion.

## Remaining issue criteria

| Issue | Evidence added | Still open |
| --- | --- | --- |
| SER-194 | Download/checksum, universal slices, Developer ID, notarization/stapling and Gatekeeper on M1; headless Host/Join | Quarantined first-user install on fresh accounts, GUI onboarding, Intel execution and minimum-supported-OS coverage |
| SER-196 | Owned bundled-daemon crash and recovery in the actual app harness | App-process crash/relaunch, single-instance/persisted role/config/trust acceptance, login enable/disable and logout/login or reboot |
| SER-197 | Real release signature/notarization checks; precise updater-harness refusal | Signed old/new download, authentication, install/relaunch, cancellation/interruption, preserved state/login and post-update connection; no signed update pair was supplied |
| SER-198 | Signed bundle URL/key, M1 command-line HTTP 200 and empty feed; unchanged signed harness refusal | Actual signed-app Sparkle request under private state remains blocked by the harness identity/defaults-domain guard |
| SER-203 | Bundled-daemon mutual TLS approval, fingerprint comparison, wrong pin and live revocation | App/native first-use trust UI, certificate rotation, and injected-input cutoff acceptance; no native trust criterion is claimed |
| SER-212 | Published packaged macOS app flow passes 25/25 | Complete exact-candidate build/audit/real-process/Docker/Windows matrix belongs to separate evidence; native gates remain open |
| SER-213 | Published Mac archive checksum, notices, signatures/notarization/stapling/Gatekeeper, private IPC/TLS smoke | No embedded Mac source manifest; stable final-artifact provenance and other-platform/native acceptance remain separate |

Native permission denial/revocation, login integration, physical input, native
clipboard, focus, sleep/wake and desktop acceptance remain untested. Access to
a Mac over SSH does not make those acceptance criteria pass.

## Reproduction and retained files

Read `CONTRIBUTING.md`, `docs/plan/readiness.md` and the harness safety checks
first. On the test Mac, create a fresh private directory with the prefix
`/private/tmp/oe-candidate-20261103.`, download the release ZIP and its checksum,
change to that directory for `shasum -a 256 -c`, then extract with
`ditto -x -k ... extracted`. The retained command controllers take that private
directory as their only argument:

```sh
python3 inspect-artifact.py "$private_run"
python3 run-app-flow.py "$private_run"
python3 run-tls-smoke.py "$private_run"
```

The app-flow script requires a new `app-flow` subdirectory and refuses reuse.
The TLS script creates a fresh subdirectory on each invocation. Reserve the
listed loopback ports before running concurrently with another worker. These
are external real-process verification commands, not unit tests or mocks.

Reports replace the private run path with `<private-run>`, the account home with
`<test-home>` and Apple's per-user temporary ticket path with `<system-temp>`.
Private keys, state directories and app archives remain outside the repository.
`results.json` records source/asset metadata, notice comparisons and hashes of
all retained evidence. No Linear update, commit, push, PR or aggregate-document
edit was made by this verifier.
