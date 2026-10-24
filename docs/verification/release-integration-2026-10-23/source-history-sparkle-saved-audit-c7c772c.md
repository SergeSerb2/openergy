# Saved source-history evidence audit — 2026-10-23

No concrete P1/P2 finding in this independent, read-only saved-evidence/source
audit. The reported classifications and coverage bind to the retained artifacts
and exact Git objects. This does not close a runtime, release or security gate.

Reviewed commit: `c7c772c2dd07b814489e5eda90f5b7036e62fbb3`.
Its direct parent and frozen scan input:
`8c9f0abbdf4016e22738b5b5a51e7c35185e3596`, tree
`9be8746e8092e2f032e1d9c69d207a497e7e3899`.
The reviewed commit adds only `source-history-sparkle-snapshot.md` and `.json`.
This audit starts from `5a9783738c5b649844805e1614da7ea37cfbf2bb` and adds only
this document. That later root, the reviewed evidence commit and this audit
commit are outside the frozen scan coverage.

Verified from saved evidence and source:

- The retained Gitleaks executable matches its recorded SHA-256 and the binary
  member of the cached archive. Saved version output is `8.30.1`, exit **0**.
  The runner saves each exit before parsing and guards against an existing run.
  Each retained trace has exactly one pinned patch-history command. Full CLI
  arguments match the committed JSON, including 100% redaction, the empty
  private ignore file, ignored inline allow comments, archive/decode limits
  2/5 and the 120-second timeout. No custom config, scanner baseline or new
  allowlist appears in those commands or the frozen root configuration.
- The non-merge report retains literal **exit 1 / 105 findings**. The separate
  merge-parent report retains **exit 0 / 0 findings**. Both frozen Git patch
  streams were reproduced byte-for-byte without invoking Gitleaks. Exact sets
  contain 285 reachable commits: 274 non-merge and 11 merge commits. All 274
  non-merge commits emit patches. The prior 254-commit set is retained; all 31
  added commits are non-merge.
- All 22 direct merge-parent pairs match Git and the prior merge supplement.
  The 21 nonempty sections equal their direct diffs, allowing only the recorded
  separator LF. The remaining pair is merge
  `06b61287afd7f31aa3002d252389f2c2b17f73f5`, second parent
  `f6401e98f0203cb0e13c954e482ba6b6444e70f6`: both trees are
  `e03abf0b639e551235781d369b9e7a397b7b872d`, with an empty direct diff.
- Every one of the 105 full raw locations matches the committed JSON, exact
  historical evidence blob, unique JSON leaf and added patch line. All column
  ranges include the leading patch `+`. The complete staged finding list
  equals the private triage after proof-value redaction. All 95 prior full
  location identities remain. Independent checks confirm **104 source SHA-256
  records**, with 116 explicit-source blob comparisons, plus the cached
  upstream member/revision check. **Zero reported locations remain unresolved.**
- The other finding is public verification material at
  `c1863689fa44a2ef002df1c8fffc4fed0d6b76d4`,
  `sparkle-packaging-2026-10-23/results.json:80`, leaf
  `builds.production_public_defaults.SUPublicEDKey`. It equals that revision's
  tracked `Info.plist` field at lines 49–50. `AppUpdater.swift:191–194` consumes
  the field as an Ed25519 update public key. In the same revision,
  `tools/release/sparkle.py:262,305–311` places those bytes in the public half of
  a verification-only container and invokes verification. Classification rests
  on this exact public field and public verification use; 32-byte encoding
  alone would also fit a private seed. No private seed or signing key was read.
- All 71 entries in the retained artifact manifest match their hashes; the
  manifest accounts for every file except itself. All 130 prior private
  artifacts and 441 prior committed evidence files match their saved hashes.
  The old 95-finding raw report still has exit 1. Retained evidence permissions
  are 0700 for its directory and 0600 for its files. Original report-writer and
  column-diagnostic first failures remain intact.

Evidence identities (SHA-256 unless marked Git blob):

- Reviewed JSON Git blob: `93d3465b27359bd5fd85f74c46c69fa7fff7c355`.
- Reviewed Markdown Git blob: `4f73abd7024d2acee45361196625fe0357d4bcd8`.
- Gitleaks binary: `ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`.
- Cached archive: `b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
- Non-merge raw report: `e28df5aeb49e42efe1ef21d1c4ece50ff7618faf2588ab314fda94385f1a25e1`.
- Merge raw report: `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`.
- Retained artifact manifest: `b775040ab8855997bd4ee89a02b37e3c8aa38e240924013c32d097622b896aad`.
- Independent audit script: `4b943c940ee36b4f06bc1ee064fa3254c00ad5c62e7cc80584da664cca1aed9e`.
- Independent audit summary: `3f59df0e372985a790ac7f78f974319034780994c2a5d6a2315958169be6c8a8`.

The private audit locator is
`/tmp/oe-sparkle-sourcehistory-audit-20261023-state.json`. It identifies the
script, checks, summary, stdout/stderr and literal exit **0**, run at nice 15.
An earlier inspection helper assumed a missing `triage.findings` field in an
older report and stopped with `KeyError: 'findings'`. Its first-failure record
and this thread's transcript are retained. The schema was inspected; no scan
was retried or report changed.

Limits: these are two retained CLI executions, not independent scanner runtime
proof or proof against unrecorded activity. Cached tool hashes were verified;
publisher authenticity was not refreshed over the network. Default detector
rules and allowlists apply to selected patches. These results do not establish
zero findings, secret-free historical snapshots, absence of other credentials
or vulnerabilities, or current security clearance. Later source, sibling refs,
uncommitted work and unreachable objects remain outside coverage.

No rescan, build, unit test, mock, historical code execution, product runtime,
network/remote operation, desktop automation, installation, live-sharing or
production-default change, private-key/token access, environment dump, port
traffic, push, publication or Cloudflare operation ran. The provider-stopped
original security/provenance review, crafted mDNS expiry/NSEC/orphan experiment
and IPC/Synergy3 budget fix were not resumed or reframed. No delegation.
