# Accepted-root source-history scan after Sparkle — 2026-10-23

Frozen source: `8c9f0abbdf4016e22738b5b5a51e7c35185e3596`. Tree: `9be8746e8092e2f032e1d9c69d207a497e7e3899`.
The cached Gitleaks **8.30.1** CLI ran each required full scan **once**:

- Non-merge patches: **exit 1**, **105 `generic-api-key` findings**, **274 commits**.
- Separate merge-parent patches: **exit 0**, **0 findings**, **11 merge commits**.

All **105 findings resolve** against their exact historical Git blobs, JSON
leaves and added patch lines. **104 are literal source SHA-256 records**;
**one is a public Ed25519 verification key**. No credential or uncertain
finding remains among these results. **Raw exit 1 remains a findings result.**
This is patch-history coverage, **not every historical full snapshot checked
secret-free or current security clearance**.

All **95 prior full location identities remain**. The ten new locations are:

- Nine source hashes in `docs/verification/sparkle-integration-2026-10-23/results.json`
  at evidence commit `c7c805a72ac5409f4dcfd465c9229c0f5e6fbc72`, lines
  159, 315, 320, 331, 357, 386, 392, 413 and 454. Each equals its exact Git
  source bytes at the manifest's explicit `source_commit`,
  `d647a01a7b83f79187d1a15c067a763647f01a31`.
- `docs/verification/sparkle-packaging-2026-10-23/results.json:80` at
  `c1863689fa44a2ef002df1c8fffc4fed0d6b76d4`, JSON leaf
  `builds.production_public_defaults.SUPublicEDKey`. It equals the tracked
  `apps/macos/Support/Info.plist` public bundle field at lines 49–50 in that
  revision. Its canonical base64 decodes to 32 public-key bytes. The updater
  reads that field at `AppUpdater.swift:191–194`. It is **not a SHA-256 source
  record**. No production signing/private key was accessed.

Checks include **116 recorded-source blob comparisons**, plus the cached
upstream archive member and revision. The first column diagnostic found all
reported ranges one column beyond blob ranges. The leading `+` on added
patch lines accounts exactly for all 105 ranges. The initial result remains
private; no scan was repeated and no raw location changed. Candidate proof
values are redacted only in the committed reports; exact values remain private.
The [machine record](source-history-sparkle-snapshot.json) contains all 105
sanitized locations, Git object IDs, source revisions, patch-line bindings,
commands, exits and artifact hashes.

The frozen history has **285 commits**, compared with 254 at the prior
`7ad8265f9237aad8d37672095c884f051b5e4c2f` snapshot. The delta is **31 non-merge commits** and no new merge.
All **274 non-merge commits emit patches**. All **22 merge-parent pairs** are
accounted for: **21 nonempty patches** match their direct parent comparisons,
allowing one log separator LF where recorded. Merge
`06b61287afd7f31aa3002d252389f2c2b17f73f5` and second parent
`f6401e98f0203cb0e13c954e482ba6b6444e70f6` have identical tree
`e03abf0b639e551235781d369b9e7a397b7b872d` and an empty diff. Parent pairs,
trees and direct patch hashes match the unchanged merge supplement. No
nonempty parent diff is missing. All exact commit sets appear in the JSON.

The retained binary and official archive were rehashed before use:

- Binary SHA-256: `ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`.
- Archive SHA-256: `b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
- Version: `8.30.1`, exit 0. No download, setup or default change.

Detector settings match the prior workflow: built-in default rules and
allowlists, 100 percent redaction, empty private ignore file, inline allow
comments ignored, archive depth 2, decode depth 5 and timeout 120 seconds.
No custom config, filter, scanner baseline, new allowlist or suppression.
Both scans ran at nice 15. The traced selectors were:

```sh
git log -p -U0 --no-merges --no-ext-diff --no-textconv 8c9f0abbdf4016e22738b5b5a51e7c35185e3596
git log -p -U0 --merges --diff-merges=separate --no-ext-diff --no-textconv 8c9f0abbdf4016e22738b5b5a51e7c35185e3596
```

Non-merge raw report SHA-256: `e28df5aeb49e42efe1ef21d1c4ece50ff7618faf2588ab314fda94385f1a25e1`.
Merge raw report SHA-256: `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`.
The exits were saved before parsing. Both scans took
1.454277 seconds total.
Raw reports, logs, traces, patches, initial diagnostics and exact proof values
remain outside the repository in a **0700 directory with 0600 files**.
The private locator and inventory hash are in the JSON. The old 95-finding
raw exit-1 report and all **130 prior private artifacts remain byte-for-byte
unchanged**, as do the prior reports and README at this accepted root.

The first report-helper construction failed on nested Python quoting before
any statement or report write ran. Its error is retained privately and in
the JSON failure record. Only the report writer was corrected; scans were
not repeated. Only these two new evidence files are added.

No build, unit test, mock, product runtime, network, setup, installed-app
operation, desktop automation, signing-key/credential access, environment dump,
default/configuration change, TCP 24800, UDP 5353, push, publication or
Cloudflare operation ran. No further delegation. Sibling refs/worktrees,
uncommitted work, unreachable objects, later commits and this evidence commit
are excluded. No `--all` scan. The provider-stopped original release-provenance/
security review, crafted mDNS expiry/NSEC/orphan experiment and IPC/Synergy3
fix were not resumed or reframed. No runtime or release gate is closed here.
