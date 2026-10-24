# macOS signing and notarization prerequisites

This read-only prerequisite check found **zero Developer ID Application identities**
in the current default keychain search context. It found **no project-declared
notarization profile identifier** in the tracked source/configuration/documentation
scope below. Notarization credential availability is **UNVERIFIED**. No identity or
credential was obtained, created, imported, unlocked, or used.

Source snapshot: `72e0233a1fe55b2c50fb231de2e0354877a0de22`.
Public tool queries started at **2026-10-23 17:54:44 PDT (UTC-07:00)**.
[summary.json](summary.json) records redacted counts, command receipts, public tool
hashes, source hashes, and the declaration-search scope.

## Prerequisite table

| Prerequisite | Result | Evidence and remaining blocker |
| --- | --- | --- |
| Developer ID Application identity available | **NO: 0 returned** | One allowed `security find-identity -v -p codesigning` call exited 0. Valid identity classes: `DeveloperIDApplication=0`, `AppleDevelopment=1`, `other=0`; total 1. This covers the current default search context only. Other keychains, accounts, private-key usability, and future availability were not checked. |
| Correct app/helper Hardened Runtime source options | **PRESENT, source only** | Named signing uses `--force --options runtime`; the daemon gets `--identifier dev.openergy.daemon`. The outer app uses its `dev.openergy.app` bundle identifier. Actual Developer ID signatures, runtime flags, entitlements, and timestamps are unverified. |
| Correct framework/nested-code source options | **PRESENT, source only** | Explicit inside-out signing covers Sparkle's Autoupdate, Updater.app, Downloader.xpc, Installer.xpc, and framework before the daemon and outer app. Nested signing preserves `identifier,entitlements`. Runtime is applied to each named-signing invocation; library code does not itself require executable entitlements. Actual nested signatures and entitlement contents were not inspected. |
| Secure timestamp selection | **CONDITIONAL** | `--timestamp` is selected only if the configured identity string starts with `Developer ID Application:`. Every other string selects `--timestamp=none`, including an identity hash or alias. A future operator must account for this exact source constraint. No suitable identity is available in this query. |
| Declared notarization profile reference | **NOT DECLARED in inspected scope** | Case-insensitive search of 823 tracked project source/config/doc files returned no `notarytool`, `keychain-profile`/`keychain_profile`, or notarization-profile reference. No profile name was guessed. Credential existence, validity, team access, and noninteractive use remain **UNVERIFIED**. |
| Signing/notary tools | **HELP/VERSION ONLY** | `codesign --help` returned usage with exit 2; no retry. `notarytool` version is `1.1.3 (42)`. Its general/submit help exited 0 and describes `--keychain-profile`. Tool presence does not prove authentication, signing ability, or notarization acceptance. |
| Sign / notarize / staple / assess exact final artifact | **BLOCKED / NOT RUN** | Missing publicly listed Developer ID Application identity; no declared notarization profile; no distribution-signed artifact or notarization result inspected. This task authorized no build, signing, upload, stapling, assessment, or app execution. |

Apple requires a Developer ID identity for direct distribution, a secure timestamp,
and Hardened Runtime for main executables. It recommends signing each code item
inside out and applying entitlements only to executables that need them. Apple
Development does not satisfy the Developer ID requirement. See Apple's
[distribution signing guide](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac/)
and [notarization prerequisites](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).

## Exact source declarations

The inspection read settings and code. It did not execute any project release,
bundling, Sparkle, build, or verification command.

| Code item / setting | Declared behavior | Source pointer |
| --- | --- | --- |
| Named identity selection | `OPENERGY_SIGN_IDENTITY`; non-ad-hoc selection requires `OPENERGY_ALLOW_SIGNING=1` | [`bundle.sh`](../../../apps/macos/scripts/bundle.sh), lines 23-27; [`sparkle.py`](../../../tools/release/sparkle.py), lines 201-209 |
| Sparkle inner executables | `Versions/B/Autoupdate`, `Versions/B/Updater.app`, `Versions/B/XPCServices/Downloader.xpc`, `Versions/B/XPCServices/Installer.xpc` | `sparkle.py`, lines 137-143 and 210-212 |
| Sparkle.framework | Signed after its inner code, with preserved identifier/entitlements | `sparkle.py`, lines 137-143 and 210-212 |
| Rust daemon helper | `Contents/Helpers/openergy`, named-signing runtime/timestamp selection, explicit identifier | `sparkle.py`, lines 209 and 213 |
| Outer Swift app | Signed last; bundle identifier is `dev.openergy.app` | `sparkle.py`, line 214; [`Info.plist`](../../../apps/macos/Support/Info.plist), lines 13-14 |
| Signature/source checks | Individual `--all-architectures --strict` checks, then `--deep --strict` verification; rejects a host entitlement containing `com.apple.security.cs.disable-library-validation` | `sparkle.py`, lines 155-187. No `--deep` signing is declared. These checks do not separately assert each final runtime flag, secure timestamp, or absence of `get-task-allow`. |
| Local ad-hoc mode | Default identity `-`; `--options 0 --timestamp=none` | `sparkle.py`, lines 201-209; [`macOS update docs`](../../macos-updates.md), lines 81-92 |
| Native archive packaging | `release.py package` explicitly supplies `OPENERGY_SIGN_IDENTITY="-"` and `UNIVERSAL="0"` to the bundler | [`release.py`](../../../tools/release/release.py), lines 363-367. This is not the universal Developer ID candidate path. |
| Toolchain/deployment declaration | Rust `1.99.1`; Swift tools/language 6; macOS 15; two build jobs | [`rust-toolchain.toml`](../../../rust-toolchain.toml), lines 1-4; [`Package.swift`](../../../apps/macos/Package.swift), lines 1-21; `Info.plist`, lines 27-28; `bundle.sh`, lines 11 and 18-19. Installed compiler/build suitability was not tested. |

The source preserves Downloader's existing sandbox/network entitlements instead
of inventing a replacement. No new entitlement file was added. The upstream
framework's actual preserved entitlements, and the absence of
`com.apple.security.get-task-allow` in a future final artifact, still need review.
No existing build or installed app was inspected for signing state.

The declaration search used Git's tracked path list, not an environment dump or
credential store. It included `apps/`, `tools/`, `docs/`, `.github/`, `scripts/`,
root `README.md` and `Makefile`, with script, Markdown, TOML, YAML, JSON, plist,
Swift and Xcode configuration extensions. `.env*` files were excluded. The path
list digest and exact expression are in `summary.json`. `.github/` files were
searched only for profile declarations; publisher behavior was not reviewed.
There is no claim about profiles outside this declared project scope.

## Future sign / notarize / staple / assess prerequisites

These are conditional follow-up requirements, **not commands run or authorization
to perform them**. The zero identity count remains a blocker; this task does not
obtain an identity or request one from the user.

| Future stage | Precise prerequisite and proof needed | Current blocker / limit |
| --- | --- | --- |
| Prepare a fresh candidate | A separately authorized build must use Rust 1.99.1, macOS deployment 15, two jobs, pinned/locked dependencies and offline caches where possible. Universal update candidates need arm64 and x86_64 across all seven executable paths in `sparkle.py:binaries`. Record actual nice 15 without nested increments. | No build was authorized or run. Existing `release.py:run`, bundler build commands, and `sparkle.py:run` add `nice -n 15`; wrapping them in another nice-15 launcher can accumulate to 20. A future launch plan must resolve and observe this. |
| Sign exact nested code | A suitable, already available Developer ID Application identity and separately authorized private-key use; explicit named-signing opt-in; each inner executable signed before its containing framework/app; secure timestamps on all code; Hardened Runtime on main executables; only required entitlements, no enabled `get-task-allow`, and retained library validation. A secure timestamp requires network access. | Public query returned 0 Developer ID Application identities. No key usability or signing proof. Identity-hash/alias selection currently chooses `--timestamp=none`; that selection would need a separately authorized adjustment or a confirmed full Developer ID Application identity string. |
| Verify before submission | On the exact fresh candidate, verify every nested item and all architectures with `codesign --verify --all-architectures --strict`, then deep-verify the outer app. Inspect each executable's signature/runtime/timestamp/identifier/entitlements separately. | No final candidate was supplied or assessed. The source's integrity checks alone do not prove Developer ID trust or notarization readiness. |
| Submit for notarization | A project-declared profile identifier backed by already provisioned, authorized, noninteractive credentials for the correct team; approved upload of the exact signed candidate in an accepted container. The documented command shape is `notarytool submit <archive> --keychain-profile <declared-profile> --wait`. Keep the submission ID, final **Accepted** status and complete log, including warnings. | No project profile reference exists in this scope; credential availability is UNVERIFIED. No submit, info, history, wait or log request was made. No placeholder was passed to a tool. A prompt or required keychain write/unlock is a stop condition. |
| Staple | Accepted notarization for the same signed content, then `stapler staple` and `stapler validate` on the app. A ZIP cannot be stapled; staple its app and recreate the final ZIP. Preserve nested symlinks, modes and signatures. | No accepted submission/ticket. Neither stapler nor an artifact was executed. |
| Assess final distribution | Check the final extracted/stapled app with `syspolicy_check distribution <app>` and `spctl --assess --type execute --verbose=4 <app>` under normal policy. Preserve exact output and final hashes. A successful signature-integrity check is separate from policy acceptance. | No assessment was authorized or run. Tool presence/command syntax is not a Gatekeeper pass. Normal system policy stays unchanged. |
| Bind the final Sparkle payload | After stapling, create the final universal ZIP, then bind its final bytes/length/signature in the Sparkle feed. Recheck the final extracted app. Keep update-signing key use and any upload/publication under separate authorization. | Existing ad-hoc fixture signatures do not cover a future Developer ID/stapled candidate. No key was read and no archive/feed/publication was changed. |
| Manual UI / updater acceptance | A separately authorized environment and plan must prove the intended UI, update, daemon handoff and relaunch behavior while preserving live sharing and host permissions. | This prerequisite check supplies no UI, relaunch, installed-update, or production acceptance evidence. |

Apple documents the submit/log/staple sequence and the need to rebuild ZIPs after
stapling in [Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow).
The local submit help confirms profile syntax and possible credential/unlock
prompts. Apple distinguishes signature integrity from distribution-policy checks
in [TN3161](https://developer.apple.com/documentation/technotes/tn3161-inside-code-signing-certificates).
See also Apple's [notarization issue guide](https://developer.apple.com/documentation/security/resolving-common-notarization-issues)
for runtime, entitlement, timestamp, and assessment requirements, and
[packaging guide](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution)
for preserving final distribution content.

## Existing Sparkle evidence stays separate

The stopped ad-hoc cases in `5c2ed82a3cd772cc8c14e7b5b472d0b717fec26a` and the
[R2 source/saved-evidence review](../sparkle-stopped-runtime-review-2026-10-23-t3-0760a73d.md)
remain historical evidence. The [case report](../sparkle-stopped-runtime-2026-10-23/README.md)
records negative signature rejection with unchanged old content/modes/symlinks,
and positive replacement with the exact prepared candidate tree. Neither case
was reproduced here. R2 reported no actionable P1/P2 finding; it added no
independent execution evidence.

The first preflight failed because Apple Python lacked `os.listxattr`; it started
no server or CLI. The negative case observed nice 20 from nested wrappers; the
positive case observed nice 15. Neither CLI case was retried. The original report
retains XProtect/rpath and metadata-denial diagnostics, possible persistent
LaunchServices/Gatekeeper state, sampling limits, no extended-attribute equality,
the unaccepted earlier CLI build, and the original preparation transcript gap.
This report changes none of those records or limits.

Those cases and R2 do not prove Developer ID signing, notarization, manual UI,
running-target updates, daemon handoff, production updater behavior, or relaunch
acceptance. This check does not reopen the stopped release-provenance/security,
crafted mDNS expiry/NSEC/orphan, or IPC/Synergy3 budget tasks.

## Read-only execution and privacy boundary

The six public tool commands are recorded once each in `summary.json`. Their
observed process priority was nice **15**, set directly without a `nice` wrapper.
All used closed stdin and bounded timeouts. No timeout occurred. The codesign
help exit 2 is preserved as usage output, not converted into a successful signing
check. No prompt was answered and no retry was made.

Raw identity output, including its subject and identity fingerprint, stays only
in the uniquely owned `/private/tmp/oe-macos-signing-prereq-20261023-*` evidence
directory outside Git (directory mode **0700**, files **0600**). Git contains only
redacted identity counts/classes, not subjects, identity fingerprints,
certificates, credentials, or a digest of the raw identity output. Tool SHA-256
values identify public tool executables only.

The first static document check caught auxiliary evidence files at mode 0644
inside the private 0700 directory. Raw identity output was already 0600. Only
those owned auxiliary files were changed to 0600. The failed assertion and
correction remain recorded in `summary.json`; no public tool query was repeated.

No build, signing, app/installer/updater execution, keychain configuration/unlock,
key/certificate creation/import, private-key/password/token read, `.env` read,
credential/profile enumeration, account API request, environment dump, network
fixture, unit test, mock, desktop driving, push, publication, or Cloudflare change
occurred. No live Mac/Windows sharing, installed app, production config/TLS/IPC,
TCC, login item, service, account, firewall, DNS, or toolchain default was changed.
No TCP 24800 or UDP 5353 operation occurred. No further delegation occurred.

Only this README and the redacted `summary.json` were added. Verification covers
the allowed public query, help/version output, source declarations, JSON/data
consistency and Git scope. Builds, runtime/E2E tests and TypeScript compilation
are outside this explicitly documentation-only, no-build task.
