# Explicit macOS signing purpose

Parent `4d9de8985a844ffed316631fc1c9939d335014b8`. This change addresses the
single P2 in immutable review `b7cabba67219f553e307c3d62128816c0298d0b0`,
`docs/verification/macos-signing-timestamp-source-review-2026-10-23-t3-fe292237.md`.
Parent fresh R2 review remains pending. No independent review or distribution
acceptance is claimed.

`tools/release/sparkle.py` now resolves signing purpose in one read-only function.
Both `signing-policy` and the real `sign` command use it. The bundler calls
`signing-policy` before update configuration, version reads, output creation or
compilation. Hashes, preferences and ambiguous name substrings require
`OPENERGY_SIGN_PURPOSE=distribution|development`. Distribution selects
`--timestamp` for every selector form and has no untimestamped fallback.
Development selects `--timestamp=none`. Full `Developer ID Application:` and
`Apple Development:` names retain their distribution and local development
defaults. Conflicts and malformed settings fail early. Default ad-hoc signing
remains offline with options `0`; named signing retains `runtime`. The existing
inside-out signing order, helper identifier, entitlement preservation and
library-validation checks are unchanged. The operator contract and examples
are in [macos-updates.md](../../macos-updates.md#bundle-configuration) and
[releasing.md](../../releasing.md).

## Real CLI evidence

[results.json](results.json) binds the current source, private command ledger,
cached inputs, fixture manifests and preserved collector failures with SHA-256.
The complete retained ledger is
`/private/tmp/oe-sign-purpose-v2-20261023-jf_s2gyj/results.json`.
Its raw command records contain exact arguments, explicit fixture settings,
exits, stdout and stderr. Only synthetic selectors appear in policy checks;
none names an available identity. No named signing operation ran.

- Python syntax compilation and `bash -n`: exit 0. No bytecode cache was written.
- Sixteen configuration failures through each real entry point: **32 exits 1**.
  Cases cover missing purpose for hash/preference/substring, empty/unknown/
  whitespace purpose, known-prefix conflicts, purpose with ad-hoc signing,
  missing named opt-in, empty/whitespace/control selectors and empty known-name
  prefixes. Both complete copied fixture trees stayed unchanged after each case.
  `DIST`, Cargo output and Swift scratch destinations stayed absent.
- Plain `sparkle.py sign` supplied the direct exits and diagnostics. Separate
  Python execution traces show that the subprocess helper body was never
  entered. Actual bundler `bash -x` traces stop at its first `signing-policy`
  command, before configuration, version, `mkdir`, Cargo, Swift or codesign.
  These are real commands, without compiler or codesign replacements.
- Ten successful read-only `signing-policy` commands cover default/explicit
  ad-hoc, both known full-name defaults, and each synthetic hash/preference/
  substring selector under both explicit purposes. They prove policy selection
  only; they do not resolve identities or prove signing-key access.
- One real default ad-hoc `sparkle.py sign --app OWNED_APP --arch arm64 --arch
  x86_64`: **exit 0**, 1.1854 s. It used a complete private copy of the cached
  universal app, including the daemon and full Sparkle 2.10.0 framework, helpers,
  XPC services and links. The app and daemon were not built, installed or launched.
- The real `sparkle.py verify` command and final `codesign --verify
  --all-architectures --deep --strict` both exited 0. All seven objects separately
  passed strict all-architecture verification. Their arm64 and x86_64 signature
  details show `Signature=adhoc`, flags `0x2(adhoc)`, no Team ID and no secure
  timestamp. Entitlement dictionaries match the cached whole-app inputs on both
  architectures; no library-validation exception is present.
- The signer launcher was observed at nice 0. Five sampled codesign processes
  ran at **actual nice 15**. The existing helper supplies the sole nice increment.
  All copied file bytes/modes and links matched the cached app before signing;
  the final tree also matched. Complete cached app and artifact identities stayed
  unchanged. This is an offline cached-input signing check, not a current-source
  compilation or app-runtime check.

## Preserved collection failures and input limit

The initial controller stopped because `python -m trace` swallowed the CLI's
`SystemExit`, returning 0 where the collector required 1. Its original controller,
logs, fixture and results remain at
`/private/tmp/oe-sign-purpose-20261023-zrbwjygj`. The corrected collection used
plain CLI exits and separate traces in a new private fixture. This was a changed
collection method, not an unchanged retry to obtain a pass.

The second collection reached a successful real signing command, then stopped
because its process filter compared `codesign` with macOS's `(codesign)` label.
Existing observations already showed nice 15. A continuation corrected that
classification without signing again. It then stopped at an incorrect collector
assumption that Downloader had nonempty sandbox/network entitlements.

Read-only inspection found **empty Downloader entitlement dictionaries** in the
original cached app, the cached Sparkle artifact and the new ad-hoc copy, on both
architectures. The current `--entitlements - --xml` form confirmed the deprecated
`--entitlements :-` output. Final collection compared every input/output
entitlement dictionary instead of asserting keys absent from the input. It reused
completed successful command records and ran only remaining read-only checks.
The two failed snapshots and all three collector sources remain unchanged and
are hash-bound in `results.json`. **Preservation of nonempty Downloader
entitlements is not demonstrated.** The signing implementation was not changed
in response to these collector failures or the inherited input condition.

The immutable prerequisite report at
`9b857c4b6b557320e8bb7a1e969466ab404e4a4f` historically found Developer ID 0,
Apple Development 1 and no declared notary profile. No identity query was repeated.
Positive Developer ID/development-key signing, actual selector resolution, secure
timestamps and timestamp-service failure, notarization, Gatekeeper and trusted
distribution remain **UNVERIFIED**. The no-fallback behavior is source evidence;
no timestamp network request was made. Prior packaging reviews `6ab225e`,
`d647a01` and `2d89aef`, the original ad-hoc runtime failure, and prerequisite
failures remain historical records and do not prove named signing.

No unit tests, mocks, fake tools, delegation, desktop driving, Rust/Swift/Xcode
build, daemon/app/installer execution, listener, identity preference, keychain or
private-key operation, production Ed25519 seed read, notarization, push,
publication or Cloudflare change occurred. No stopped task was resumed. Live
sharing, installed apps, production config/TLS/IPC, TCC, login/services/accounts,
firewall, DNS and toolchain defaults were preserved. No TCP 24800 or UDP 5353
operation occurred. Only the two signing tools, two maintained docs and this
new evidence directory changed.
