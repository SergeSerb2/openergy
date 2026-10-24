# Named signing timestamp source review

Review `oe-signing-timestamp-source-r1-45c0e44-t3-fe292237-20261023` used
GPT-6.1-sol, xhigh effort, default service tier, with no further delegation.
Parent: `45c0e44de620fd222d1d7d22e198bc6ca4790858`. Scope: identity selection and
timestamp policy in `tools/release/sparkle.py`, `apps/macos/scripts/bundle.sh`,
and `docs/macos-updates.md`. **One P2; no P1. Source only.**

The prerequisite report was read from immutable commit
`9b857c4b6b557320e8bb7a1e969466ab404e4a4f`, paths
`docs/verification/macos-signing-prerequisites-2026-10-23/{README.md,summary.json}`.
Those files were not imported or changed. Its three scoped source files match
this parent. Its recorded counts are historical: Developer ID Application 0,
Apple Development 1; no project-declared notary profile; credential/private-key
usability and positive Developer ID signing unverified. No query was repeated.

## Contract and finding

The maintained [signing contract](../macos-updates.md#bundle-configuration),
lines 81-92, defines local ad-hoc signing and explicitly selected named signing.
[releasing.md](../releasing.md), lines 140-155, permits an operator with a
suitable identity to select it with `OPENERGY_ALLOW_SIGNING=1` and
`OPENERGY_SIGN_IDENTITY`. Neither document limits selectors to a full certificate
common name or designates hashes/preferences as development-only.

[`bundle.sh`](../../apps/macos/scripts/bundle.sh), lines 23-27, checks the
non-ad-hoc opt-in but imposes no selector syntax. Lines 95-98 call the real
Sparkle signing CLI with the inherited environment.
[`sparkle.py`](../../tools/release/sparkle.py), lines 201-214, accepts that
selection and passes it unchanged to `codesign --sign`.

**P2 — Select timestamp policy independently of the raw identity prefix**
(`tools/release/sparkle.py:205`, relevant span 202-209).
Trigger: an opted-in operator selects a usable Developer ID Application identity
by its actual 40-hex SHA-1 certificate selector, an existing identity preference,
or an unambiguous common-name substring without `Developer ID Application:` at
the start. These are real `codesign` selector forms. The script accepts them,
retains `--options runtime`, but sets `--timestamp=none` for every nested code
object, the daemon, and the outer app. Selecting the same certificate by its full
Developer ID Application name instead requests `--timestamp`.

This disables the secure timestamp even when all other signing prerequisites
succeed. The resulting signature can pass integrity verification but lacks the
secure timestamp required for notarization; the operator must re-sign before
distribution. `verify_app`, lines 182-187, verifies integrity and the host's
library-validation entitlement, without requiring a timestamp. A separate
notarization gate does not correct the signing arguments. This is a supported
selector mismatch, not a finding against intentional development or ad-hoc mode.
No actual signing success, notarization rejection, or OS trust was reproduced.

Apple's installed public manual, `/usr/share/man/man1/codesign.1`, confirms:
lines 602-626 describe identity preferences, common-name substring matching and
40-hex SHA-1 selectors; lines 399-410 describe Apple's timestamp server, failure
when it cannot provide a timestamp, and explicit disabling with `none`.
Manual SHA-256:
`fec462ff82f5e09e4f59422774a1099192430598368196bcd1a3d51ad0b872ed`.
The secure-timestamp notarization requirement is carried from the prerequisite
report's README lines 26-31 and 73-74, with its
[Apple primary reference](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).
No web page was fetched in this no-network review.

## Smallest recommended change

Keep `-` as `--options 0 --timestamp=none`. Keep named signing at `--options
runtime`, with the existing inside-out order and entitlement preservation.
Keep the full Developer ID Application name path at `--timestamp`.

Replace the silent no-timestamp fallback for ambiguous named selectors with an
explicit signing-purpose choice: distribution requires `--timestamp` regardless
of selector spelling; explicit development may use `--timestamp=none`. Reject an
ambiguous selector without that choice before compilation or signing, in both
the bundler preflight and direct signing entry point. Document this small contract
addition and the timestamp service's network requirement. It needs no identity
discovery or new dependency. Do not change all named local builds to request a
network timestamp. Do not add a library-validation exception or weaken runtime.

## Future real CLI validation plan — not executed

1. Use only fresh, owned bundle copies, already provisioned authorized identities,
   and separately authorized private-key use and timestamp-network access. Do not
   use the existing Apple Development identity, create certificates, enumerate
   accounts/keychains, install, or launch an app. The recorded missing Developer
   ID identity blocks the positive cases until a later authorized environment
   supplies one. If a fresh build is needed, use Rust 1.99.1, deployment 15,
   jobs 2 and locked/offline caches where possible, with discovery disabled and
   private config/IPC/TLS set before any daemon version read.
2. Exercise the real `apps/macos/scripts/bundle.sh` preflight and
   `python3.11 tools/release/sparkle.py sign --app OWNED_APP --arch arm64 --arch
   x86_64` entry point. First require rejection of an ambiguous selector with no
   purpose and an unchanged fixture tree. No mock or fake `codesign` is valid.
3. On separate identical prepared bundles, select the same authorized Developer
   ID Application certificate by full name and its actual SHA-1 selector, both
   in distribution mode. Include an existing authorized preference if available;
   do not create one. Require secure timestamps and runtime on all seven code
   objects from `nested_code(app) + [daemon, app]`, and verify every architecture
   with real `codesign --verify --all-architectures --strict`, then deep-verify the
   app. Inspect signatures with real `codesign -d --verbose=4 --all-architectures`
   and entitlements per architecture. Compare resolved certificate identity,
   timestamp presence, runtime, identifiers and preserved entitlements. No
   library-validation exception may be introduced. Integrity alone is insufficient.
4. Exercise default ad-hoc mode with network access absent: require options 0,
   no secure timestamp and valid nested integrity. Verify an explicitly selected
   development mode only with a separately authorized development identity; it
   must not request a network timestamp. Preserve first failures without blind
   retries. A timestamp-service failure in distribution mode must not fall back
   to an untimestamped success.
5. Observe actual workload priority 15. The current helper adds `nice -n 15`;
   use a neutral-priority launcher for that path and no outer nice increment.
   Resolve any inherited-priority conflict before execution. Record exact CLI
   exits, arguments, final tree hashes and per-object signature results with
   private identity values redacted. This plan supplies no notarization,
   Gatekeeper, installed-update, relaunch or production acceptance claim.

## Prior review and execution limits

[Prior review ledger](sparkle-integration-2026-10-23/reviews.md), lines 12-25,
records packaging R2 at `6ab225e`, the title fix in `d647a01`, and final R3 at
`2d89aef`. This review leaves their findings and evidence unchanged. In
particular, [the original dyld failure and fix](sparkle-integration-2026-10-23/README.md#preserved-failures-and-review-response),
lines 48-55, remain: ad-hoc Hardened Runtime failed to load Sparkle; options 0
fixed that local mode while named identities retained runtime. No library-
validation exception was added. None of those records proves named signing.
The prerequisite report's help exit 2 and initial auxiliary-permission check
failure remain recorded; this review does not convert them into passes.

Only this document was added. Verification was source, Git and local manual
inspection plus document/diff checks. No build, project CLI execution, signing,
identity query, keychain/credential/private-key read, network, runtime, test,
mock, desktop driving, push, publication or Cloudflare action occurred. The three
provider-stopped tasks were not reopened. Live sharing, installed apps and all
production/host settings remain untouched. No TCP 24800 or UDP 5353 operation
occurred. No TypeScript package changed; its build/typecheck is outside this
explicit source-review-only task.
