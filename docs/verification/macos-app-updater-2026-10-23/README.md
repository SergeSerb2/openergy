# macOS app updater implementation evidence

Scope: Swift app/UI changes based on `41ad7c70fbc95d22c2e11c7f10e0790ea7369067`.
No manifest, lockfile, production plist, bundle script, daemon, DNS or release files changed.
The live app was not launched, stopped, installed or replaced. No GUI automation, unit
tests, mocks, native input, service registration, Keychain access or public feed requests
were used. This is app source and updater-command evidence, not release acceptance.

`AppUpdater.shared` is inert until `AppDelegate.applicationDidFinishLaunching` calls
`startAfterNormalLaunch()`. Snapshot, daemon E2E, updater E2E, login-command, demo and
offscreen arguments are excluded from that path. Snapshot/E2E views receive no updater.
Existing `AppModel.shutdown()` remains the termination path for Sparkle's normal relaunch.

The retained `SPUStandardUpdaterController` uses Sparkle's standard user driver. Its
underlying throwing `SPUUpdater.start()` lets configuration errors appear in Settings
without a configuration-error dialog. Sparkle owns scheduling, approval, signature
validation, installation and relaunch. No automatic preference is reset on normal launch.
The UI observes `canCheckForUpdates`, `sessionInProgress`,
`automaticallyChecksForUpdates`, `automaticallyDownloadsUpdates`, and
`allowsAutomaticUpdates` through KVO on the main actor. Settings setters use the actual
Sparkle properties. App menu, menu-bar panel and Advanced settings share the same instance.
Advanced settings remain accessible if the daemon is unavailable.

Production configuration requires both a strict HTTPS `SUFeedURL` and a canonical base64
`SUPublicEDKey` that decodes to 32 bytes, from `Bundle.main`. Userinfo, fragments, bad ports
and other schemes are rejected. The feed delegate returns that validated URL so an old
Sparkle feed preference cannot replace it. Runtime `OPENERGY_UPDATE_*` values are ignored.
No production feed, public key, default key or fallback URL is embedded in these sources.

## Build and real commands

Verified on macOS 27.2 arm64 with Apple Swift 6.4, language mode 6, deployment target 15.
The whole app compiled in a private `/tmp/openergy-app-updater.1Z27M1/package` copy.
Only that copy's manifest added `.package(url: "https://github.com/sparkle-project/Sparkle",
exact: "2.10.0")` and the Sparkle target product. Resolution selected revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`. The checked-out manifest and lockfile were not edited.

```sh
MACOSX_DEPLOYMENT_TARGET=15.0 nice -n 15 swift build \
  --package-path /tmp/openergy-app-updater.1Z27M1/package \
  --jobs 2 --disable-automatic-resolution
```

Full initial compile passed in 20.32 s. The final incremental compile passed in 1.60 s
without warnings. Each disposable app copied the resulting executable and complete
`Sparkle.framework` into `Contents/Frameworks`, added the
`@executable_path/../Frameworks` runpath with `install_name_tool`, and received an ad-hoc
signature. The first staging attempt omitted that runpath and dyld rejected the bundle;
all recorded successful runs used the corrected staging. This does not validate the
separately owned production packaging script.

A finite Python `ThreadingHTTPServer` bound only to `127.0.0.1` served the real appcasts.
Version 1 matched the test bundle's `CFBundleVersion=1`; version 2 was newer. A temporary
CryptoKit `Curve25519.Signing.PrivateKey` supplied the public key and seed file. Sparkle
2.10.0 `sign_update --ed-key-file` signed a fixture-only ZIP and both XML feeds. The ZIP
contained a text marker, not an app. The server observed no ZIP request. Signing fixture
metadata does not prove that the app validated an archive signature.

`outcomes.json` retains exact primary command JSON, request paths, rejection summaries,
and the deliberate deadline failure. Current, candidate, HTTP 503, malformed XML, and
eight unavailable configurations behaved as expected. A stalled response produced
`deadline-exceeded`, exit 1, after 1.004 s with a one-second deadline. Adversarial runtime
`OPENERGY_UPDATE_FEED_URL` and `OPENERGY_UPDATE_PUBLIC_KEY` values were set during this pass;
the configured local feed and bundled fixture key still controlled the checks.

## Updater-only command contract

This branch runs before `AppModel.shared`, daemon/native initialization and SwiftUI startup.
It uses the same `AppUpdater`/standard controller and real Sparkle delegates. It makes
`checkForUpdateInformation()` probes. `shouldProceedWithUpdate` records the chosen
candidate and throws the documented veto before presentation, download or installation.
The outcome is explicitly `candidate-observed-installation-not-exercised`.

Create owned mode-0700 output and config directories under `/tmp` or the user's temporary
directory. Place the private `.app` inside output. Give it a unique bundle identifier
starting with `dev.openergy.updater-e2e.`, an absolute
`SUDefaultsDomain=<config-dir>/sparkle-preferences`, and a fresh preference file path.
Set `CFFIXED_USER_HOME` before process launch. The command verifies all of these paths.
The defaults domain's existing parent directory is resolved before comparison, so
`/tmp` and `/private/tmp` aliases work. A matching private feed and public key must be baked
into this temporary bundle. Its temporary ATS exception was only
`NSAppTransportSecurity = { NSAllowsLocalNetworking = true; }`.

```sh
CFFIXED_USER_HOME="$CONFIG" "$OUTPUT/Openergy.app/Contents/MacOS/Openergy" \
  --updater-e2e "$OUTPUT" --updater-config "$CONFIG" \
  --updater-feed "http://127.0.0.1:$PORT/current.xml" \
  --updater-expect current --updater-timeout 15
```

`--updater-expect` accepts `current`, `candidate`, `failure`, or `unconfigured`.
The fixture URL must use a literal `127.0.0.1` or `::1` host, with no userinfo or fragment.
Only this command allows loopback HTTP. For configured cases the argument must match
the baked feed exactly. For `unconfigured`, the bundle must fail strict production
configuration validation; the same production configuration/start pipeline then runs
and must create no controller or request. This does not run the normal GUI lifecycle.

The timeout defaults to 15 s and accepts 1–30 s. A monotonic deadline bounds the run.
Output is one bounded `updater-result.json` written atomically in output and one status
line. Exit 0 means the expected result occurred; exit 1 means a mismatch/deadline or
output error; exit 64 means invalid arguments or unsafe paths. An existing result file
is rejected. Candidate version text is capped at 256 characters.

The private command disables scheduled checks, automatic downloads and profiling with
volatile argument-domain preferences. It also suppresses the permission prompt through
the real updater delegate. Actual Sparkle writes go to the private absolute defaults
domain and are synchronized before exit. `CFFIXED_USER_HOME` alone was insufficient on
this macOS version: an earlier run created nine disposable fixture-domain preference
files in the user's Preferences directory. Those exact owned files were removed. The
final run wrote `config/sparkle-preferences.plist`; zero fixture-domain files remained
in the user Preferences directory.

Remaining proof: combined packaging, visual UI, saved preference changes through the UI,
production feed/key setup, signed archive validation, installation, daemon teardown
during a real update, and relaunch. No installed-app or signed release claim follows
from these command results.

API references: [programmatic setup](https://sparkle-project.org/documentation/programmatic-setup/),
[SPUUpdater](https://sparkle-project.org/documentation/api-reference/Classes/SPUUpdater.html),
[SPUUpdaterDelegate](https://sparkle-project.org/documentation/api-reference/Protocols/SPUUpdaterDelegate.html).
