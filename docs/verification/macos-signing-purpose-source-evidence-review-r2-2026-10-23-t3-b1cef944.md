# macOS signing purpose: independent R2 source and saved-evidence review

Review `oe-signing-purpose-source-evidence-r2-88c33628-t3-b1cef944-20261023`
used GPT-6.1-sol, xhigh effort, default service tier, with no further delegation.
Candidate: `88c33628d6df1f6ac0bdc11c7cc955c55ae74eaf`.
Parent: `4d9de8985a844ffed316631fc1c9939d335014b8`, confirmed as the candidate's
direct parent. **No concrete P1/P2 finding in this bounded review.** The original
selector/timestamp P2 in `b7cabba67219f553e307c3d62128816c0298d0b0` is addressed
by the source change. This disposition adds no independent signing execution
or distribution acceptance evidence.

The review read immutable Git objects and existing local evidence, including
private originals where needed. It did not run the project CLI, collectors,
signing tools, compilers, app, daemon or installer. All source line numbers below
refer to the candidate, not the review checkout. Scope is the two signing tools,
their maintained operator documentation, and
`docs/verification/macos-signing-purpose-2026-10-23/{README.md,results.json}`.

`tools/release/sparkle.py:201-227` resolves the identity and purpose in one
function. Direct `sign` calls it at line 231, before inspecting or signing bundle
objects. The new `signing-policy` dispatch calls the same function at lines
408-410 and prints only the selected purpose and flags. That path does not call
the subprocess helper, query identities, inspect a bundle, read a signing key or
contact a timestamp service. Main parsing and imports introduce no such action.
Policy errors reach the existing exception handler at lines 424-425 and exit 1.

`apps/macos/scripts/bundle.sh:6,14-18` uses `set -euo pipefail` and invokes that
real policy command before update configuration or the version command. Output
creation begins at lines 26-31; Cargo metadata and builds follow at lines 41-59.
The final sign command at line 95 inherits the same signing environment. No
intervening command changes or clears the identity, opt-in or purpose. The shell
quotes the interpreter, tool and bundle paths. The identity is read from the
environment in Python and passed as one argument after `--sign` through the
existing `subprocess.run` argument list (`sparkle.py:31-38,235-240`). There is no
shell evaluation of selector text and no new command-injection path.

The policy matches the revised contract:

- Unset identity and explicit `-`, with purpose unset, retain `--options 0
  --timestamp=none`. An explicitly empty identity now fails as documented.
- Nonempty `Developer ID Application:` names default to distribution. Nonempty
  `Apple Development:` names default to development. Conflicting explicit
  purposes fail. Matching purposes are accepted.
- Hash, preference and other ambiguous substring selectors require the exact
  explicit purpose `distribution` or `development`. The selector is passed
  unchanged. This confirms operator intent; it does not identify the resolved
  certificate or its usability.
- Every distribution path selects named `runtime` and `--timestamp`. Every
  development path selects named `runtime` and `--timestamp=none`. A failed
  signing subprocess raises immediately; no untimestamped retry exists.
- Empty or unknown purposes, purpose whitespace, purpose with ad-hoc signing,
  missing named opt-in, empty selectors, surrounding selector whitespace,
  ASCII control characters and bare known prefixes fail before mutation.

This closes the original silent untimestamped fallback for supported ambiguous
selectors without making local development depend on the timestamp service.
`docs/macos-updates.md:81-134` and `docs/releasing.md:145-166` explain the intent
setting, prefix defaults, malformed inputs, network requirement, read-only CLI,
real key use in signing, and separate trust gates. The universal example is now
explicitly ad-hoc. The documented default requires the purpose to be unset;
an inherited purpose with identity `-` is deliberately rejected.

Static comparisons with the parent confirm that `run`, `nested_code`, `binaries`,
`verify_app` and `embed` are unchanged. The four final signing/verification
statements are unchanged apart from line positions. The five Sparkle objects
are signed inside out, then the daemon with `dev.openergy.daemon`, then the
outer app. Nested identifier/entitlement preservation remains in place. Named
signing retains Hardened Runtime. No library-validation exception was added or
accepted by the existing verifier. Early policy failure is mutation-free; this
does not claim transactional rollback after a later real signing failure.

The saved evidence was checked against its underlying data, not only its result
flags. The committed results SHA-256 is
`a62e7cf8fd692866bb18b68fe1f4c0d365b3d5d1412d8150f2e3d669761e5816`.
The complete private ledger is
`/private/tmp/oe-sign-purpose-v2-20261023-jf_s2gyj/results.json`, SHA-256
`4fc233c59ea627f2106770e898fd14112cadec6850200b79f965a2cd63c647a2`.
All 14 public path/size/hash bindings matched their retained files. All 259
raw/auxiliary file bindings in that private ledger also matched. Its 112 command
labels are unique. The four final source hashes match the candidate Git blobs.
The collection hashes for both executable signing tools match those same blobs:

- `tools/release/sparkle.py`:
  `86b6470a7ea6fda264fdd35977ba85070dba48199e22f2aa1e6ad840c0444b9b`.
- `apps/macos/scripts/bundle.sh`:
  `19a1f2b58b6c59c1103a9bd3424d4a8cf0feb37edb678b5001a21ab6b4692aa2`.

Only `docs/macos-updates.md` differs between collection-time and final source
hashes. The results explicitly record that documentation change. A source hash
identifies the script used in the saved commands; it does not prove compilation
of the cached app or daemon from that source.

For all 16 cases, the saved plain direct CLI and bundler records each show exit 1,
the expected diagnostic and matching explicit synthetic policy settings. These
are 32 real early rejections. Each supplementary direct execution trace has no
executed line in the subprocess helper body. Every traced source line matches
the candidate. Each bundler trace ends at its first `signing-policy` invocation,
before config, version, `mkdir`, Cargo, Swift or codesign. The trace wrapper's
exit 0 is not counted as a real CLI rejection. The ten saved read-only policy
commands have exit 0 and the exact reported policy output. No real identity was
resolved in those checks.

Both failure fixture manifests contain 168 entries and match before/after.
Their entire app content equals the 167-entry cached app manifest. Read-only
hashing also confirmed that the retained failure trees still match those saved
manifests, with the requested DIST, Cargo and Swift destinations absent. The
positive before/after manifests each contain 167 entries and equal the cached
whole-app manifest. Its canonical tree digest is
`595db0302e64209f999a1a14406a414f8f0646a5f7d5e4df48a7433e5f2d9811`.
The retained cached Sparkle artifact has 221 entries and digest
`e6514b979561752fe2dca8547a833a2561502f95315644643f0cc17e38636e88`.
Both cached inputs and the positive copy still match these saved identities.
The identity definition covers file bytes/modes, directory modes and symlink
targets. It does not claim extended-attribute equality. The cached framework's
plist reports Sparkle 2.10.0.

The ledger contains exactly one `real-adhoc-sign` record: exit 0, 1.1854 seconds,
through the real direct CLI on a complete private universal app copy. The public
sign record is the same private record with process observations filtered to the
five codesign samples. Those samples show actual nice 15. Their Python parent
observations show nice 0, consistent with the recorded neutral launcher and the
single nice increment in `run`. Sampling does not measure every signing process.

The saved `real-verify-cli` and deep strict all-architecture verification records
show exit 0. All seven signed objects, including the outer app, have separate
strict all-architecture verification records. For each object, saved arm64 and
x86_64 detail output agrees with the reported identifier, `Signature=adhoc`,
flags `0x2(adhoc)`, no Team ID and no secure timestamp. All 14 saved entitlement
dictionaries match their corresponding cached-app baselines. None contains the
library-validation exception. These are audited saved results, not fresh
codesign checks by this reviewer.

All collector failures remain failures:

- The original collection at
  `/private/tmp/oe-sign-purpose-20261023-zrbwjygj` stopped after three commands.
  Its direct trace wrapper returned 0 where the collector required 1. The saved
  trace reaches the policy rejection and `sys.exit`; it is not a plain CLI exit
  receipt. The next collector changed the method to plain CLI exits plus separate
  boundary traces in a new fixture. The original controller, result and raw
  outputs remain present.
- The priority-label snapshot contains 61 commands and the successful signing
  record. Its filter expected `codesign` while the observed label was
  `(codesign)`. The continuation corrects classification of those existing
  samples. Every one of those 61 command records is unchanged in the final
  ledger; there is no second signing record.
- The entitlement-assumption snapshot contains 75 commands and stops at
  `Downloader entitlements lost`. All 75 records are unchanged in the final
  ledger. The continuation had wrongly required nonempty sandbox/network keys.
  Four retained legacy-format baseline reads and six explicit XML reads show
  empty Downloader dictionaries in the original cached app, cached artifact and
  positive copy on both architectures. The finalizer reuses existing commands
  after checking their exact arguments/environment, then compares actual input
  and output dictionaries and completes remaining read-only checks.

The original collector, corrected collector, continuation, finalizer, both later
failure snapshots and first failed results are hash-bound and were read as data,
never executed. **Preservation of nonempty Downloader entitlements remains
UNVERIFIED.** Matching empty inputs and outputs does not supply that proof.
The deprecated entitlement-output warning also remains in the saved stderr.

The immutable prerequisite report at
`9b857c4b6b557320e8bb7a1e969466ab404e4a4f` records Developer ID Application 0,
Apple Development 1 and no project-declared notary profile. Those are historical
counts. No query was repeated and the available Apple Development key was not
used. Positive named signing, actual hash/preference resolution, key access,
secure timestamps, timestamp-service failure, notarization, Gatekeeper and
trusted distribution remain unverified. No-fallback behavior is source evidence.
The cached app contains older app/daemon binaries; this is not a current-source
app build, runtime check, Sparkle load check or installed-update acceptance run.

The prior review and packaging history remain intact. The historical dyld exit
-6 from ad-hoc Hardened Runtime and the later options `0` fix are not changed or
reproduced. Package review `6ab225e`, title correction `d647a01`, final review
`2d89aef`, and prerequisite help/permission-check failures remain historical.
This review supplies no broader publisher or release-provenance/security audit.

Only this unique review document was added. Checks consisted of immutable Git
reads, static source comparisons, JSON/raw-record inspection, read-only hashes
of retained private inputs, and document/diff/scope checks. No unit tests, mocks,
project CLI, build, signing, app/daemon/installer execution, new fixture, network,
identity query, keychain operation, key/secret/token read, environment dump,
desktop driving, push, publication or Cloudflare change occurred. No stopped
release-provenance/security, crafted mDNS expiry/NSEC/orphan, or IPC/Synergy3
budget task was resumed. Live Mac/Windows sharing, installed apps, production
config/TLS/default IPC, TCC, login/services/accounts, firewall, DNS and toolchain
defaults were untouched. No TCP 24800 or UDP 5353 operation occurred. No
TypeScript package changed; compilation/typechecking is outside the explicit
source-and-saved-evidence-only review scope.
