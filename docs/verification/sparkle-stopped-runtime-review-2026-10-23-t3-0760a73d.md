# Independent source and saved-evidence review

No actionable P1/P2 findings in the reviewed commit.

This review audits source and saved records. It does not independently reproduce
the CLI cases. No fixture controller, cleanup controller, application, updater,
server, build, signature tool, registry query, or desktop operation ran during
this review. The audit used Git reads and read-only content/hash comparisons.
Only this review document was added.

## Exact scope

- Reviewed commit: `5c2ed82a3cd772cc8c14e7b5b472d0b717fec26a`.
- Its verified direct source parent: `a121376fe6c1715fa99d72559336ca3b3a73b439`.
- Isolated review checkout base: `8c9f0abbdf4016e22738b5b5a51e7c35185e3596`.
- Reviewed evidence directory: `docs/verification/sparkle-stopped-runtime-2026-10-23/`.
- Evidence ID: `d8001bf18a06eae043e04a76ac0ca899ebe3fce975a1f89b835b0a65b2e57ccb`.
- Manifest SHA-256: `15a5cf8a943e7a31d881a7a1b9af75d0fcfa14a83ae1c114aba039f0c5427b54`.
- Approved operation-v2 SHA-256: `2846f3adafe9b1f39092bb0e98e672dd66c9ff7f89715c110c282723d89d3cef`.
- Approval supplied in the task and saved records: 2026-10-23 17:16:55 PDT.

The commit adds evidence and controller snapshots only. This review does not
reopen the stopped release-provenance/security, mDNS, or IPC/Synergy3 work.
The preserved proposal's historical pending-approval fields do not override the
later explicit user approval.

## Audited bindings and checks

All 139 manifest entries have the recorded byte count and SHA-256 in Git. Each
Git copy also matches its exact saved private original byte for byte. The raw
file set matches the manifest without omissions or duplicate paths. Recomputing
the canonical ordered array of file/bytes/sha256 entries, with sorted JSON keys
and compact separators, gives the exact evidence ID above.

All 12 referenced preparation records match their Git hashes and saved private
original hashes. The two gzip copies decompress to their originals. All 18
preparation-directory files listed in `raw/final-audit.json` retain their hashes
from the source parent. These checks preserve the earlier preparation and review
records rather than rewriting their history.

The documented controller is byte-identical to `raw/controller-cli-once.py`.
The documented and raw cleanup controllers are also byte-identical. The frozen
first controller, correction record, traceback, and failed preflight result
remain present. Source order places that first xattr failure before the server
bind and CLI launch. The correction changes xattr reporting, case directories,
and exclusive start markers; it leaves the approved command arrays unchanged.

The saved 98-entry prepared-driver inventory and 112-entry immutable original
CLI inventory match their retained private artifacts. Their main executable
hashes and the prepared framework executable hash match the preparation record.
Both case preflights perform these comparisons before startup, rather than
merely writing success flags. All six case old/candidate/round-trip trees also
match the earlier preparation trees, with 167 entries each.

The two enclosure signatures decode to 64 bytes and differ in one byte. The
remaining enclosure attributes use the same archive URL and length. Saved
preparation command receipts record acceptance of the correct archive signature,
rejection of the changed signature, and successful verification of both signed
feeds. These are audited receipts; this review did not run the verifier or read
any private key.

Each saved CLI command exactly matches its approved operation-v2 command. The
controller supplies the private configuration environment before startup, binds
only literal `127.0.0.1:25600` without address reuse, and uses exclusive case
directories and CLI-start markers. The saved records support one CLI operation
per case, after the separate first preflight failure. No case was rerun by this
review.

## Audited outcomes

The negative server receipt records one full 7,492,869-byte archive GET with
SHA-256 `a4bf7a1386fd870fed820a169a005fd6deb99adfa4546c836a3550b6b7308183`.
The actual saved CLI stderr reaches 100 percent download and reports signature
validation error `4005 (SUSparkleErrorDomain)`. The raw exit is 1. Comparing the
saved before/after trees independently confirms unchanged old content, modes,
and symlink targets. The old build is 1 and its executable hash is
`36243ed8c67e31fe261f699d10d439e0d22d32cb34b60ed51d90253b558ba5e6`.
Raw negative result SHA-256:
`cefd1e0d939ef7a29fee1084acccee998d179e9cc48e24a0f55795bd54ac003e`.

The positive saved safety decision follows the negative result and cleanup.
The fresh positive preflight records no fixture processes, jobs, or cache roots.
Its server receipt records the same complete archive. CLI stderr records
`Installation Finished.` and raw exit 0. The saved tree at the old path equals
the full prepared candidate tree. Build 2 and executable hash
`7250e62280f864ba58888a3924b89296cdf6204a96146b6783cfd52827c50d5e`
agree with the tree and metadata records. All seven individual nested/outer
checks and the deep check record exit 0, no timeout, and the actual
`--all-architectures --strict` arguments.
Raw positive result SHA-256:
`ba32573fa3e83d1e24b56b8d5f3967de58ee0bd8bf0eedfa9111d064f6cc925e`.

Cleanup source requires absent saved baselines, no fixture processes/jobs, exact
allowlisted cache roots, matching fixture names, and current UID. It preserves
each root's inode and inventory before removal. The saved receipts record three
exact roots removed per case and absent roots/processes/jobs afterward. The final
saved PID and port reads agree with the reported cleanup boundary. No current
process, port, service, or registry state was queried for this review.

## Limits retained

The 17 negative and 18 positive process samples show CLI nice 20 and nice 15,
respectively. The negative priority deviation remains explicit. The nominal
100 ms sampling and job snapshots can miss brief activity. Direct saved unified
log emitters identify the real fixture helper PIDs; they do not establish a
complete execution audit. The controller's target-stop branch was not observed
to run, and the absence of a sampled target is not absolute no-execution proof.

The retained Sparkle source hash for `Autoupdate/SUPlainInstaller.m` is
`d5191e8b7684df756383c2122dcfcaaa011d5bae237b2ddbf9fa05986c8f5b55`.
The progress controller matches its saved source inventory hash
`c6248006f3ab3548f9f1acd86a89074ce91597313b27e8d9fc8d70e4af06f71e`;
its relaunch branch requires `_applicationInitiallyAlive`. This is source
corroboration only.

Saved LaunchServices lines identify the private driver and its Updater.app.
Positive unified-log lines 48 and 51-55 retain the metadata denial, XProtect
error 2 and unresolved Swift 6.2 rpath, and completed candidate scan receipts.
They do not prove a successful trust assessment. Cleanup does not claim to undo
shared LaunchServices or Gatekeeper state. Four private configuration/storage
files remain as recorded evidence.

Tree equality excludes extended attributes and records root ownership/timestamps
separately. The original preparation transcript gap and unaccepted earlier CLI
build remain explicit limits. Nothing in this commit closes Developer ID,
notarization, target relaunch, running-target updates, native UI, daemon handoff,
or production updater acceptance.

The review transcript retains two read-only inspection errors: an initial JSON
parse rejected the unified-log header, and a guessed source filename did not
exist. Subsequent reads handled the unchanged header and used the exact source
path from the saved inventory. Neither error started a fixture operation or
changed any saved record.
