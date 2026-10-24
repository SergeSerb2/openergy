# Stopped-target Sparkle CLI runtime evidence

Both approved CLI cases ran once against the prepared private fixture. The
negative case completed the archive download, reported a signature validation
error, and kept build 1's exact content/mode/symlink tree. The positive case
completed the archive download and replaced the old path with the exact build 2
candidate tree. Nested and deep signatures passed for all architectures.
No target launch or relaunch was observed. This is local ad-hoc fixture evidence,
with the observation limits below. It is not Developer ID or notarization acceptance.

Source parent: `a121376fe6c1715fa99d72559336ca3b3a73b439`.
Sparkle source: `eef1a539a373c1f1a320624b1130fc5de7b2e100` (2.10.0).
The user approved the reviewed v2 fixture effects at **2026-10-23 17:16:55 PDT**.
The exact approved proposal SHA-256 is
`2846f3adafe9b1f39092bb0e98e672dd66c9ff7f89715c110c282723d89d3cef`.
The original preparation, reviews, proposal, and results remain unchanged.
Their historical pending-approval fields were not rewritten.

The fixture root is `/private/tmp/oe-stopped-update-llil939f`; its host ID is
`dev.openergy.updater-fixture.5b4a5cb7d61d444eaf4a3d673e1df4a5`.
The controller and private logs remain at
`/private/tmp/oe-stopped-runtime-20261023-wXymLm`.
Both commands used the reviewed driver, arguments, literal loopback URLs, and
private `CFFIXED_USER_HOME`/`TMPDIR` before startup. Each server exclusively bound
`127.0.0.1:25600` without address reuse. Neither timeout fired.

Observed cases
---------------

- **Negative:** started 17:26:22.792898 PDT, elapsed 0.528681 seconds, raw exit 1.
  The server sent all **7,492,869** archive bytes; the CLI reported 100% downloaded.
  It then reported `4005 (SUSparkleErrorDomain)` and
  `The update is improperly signed and could not be validated.`
  Build 1's full prepared content/mode/symlink tree stayed exact. Its executable
  SHA-256 stayed `36243ed8c67e31fe261f699d10d439e0d22d32cb34b60ed51d90253b558ba5e6`.
- **Positive:** started 17:27:49.290392 PDT, elapsed 0.527425 seconds, raw exit 0.
  The server again sent the complete archive; the CLI reported
  `Installation Finished.` The stopped old path now has build 2, its exact
  candidate tree, and executable SHA-256
  `7250e62280f864ba58888a3924b89296cdf6204a96146b6783cfd52827c50d5e`.
  Seven individual nested/outer signature checks and one deep check passed with
  `--all-architectures --strict`.

Both downloads used archive SHA-256
`a4bf7a1386fd870fed820a169a005fd6deb99adfa4546c836a3550b6b7308183`.
Before each CLI operation, all preparation hashes, old/candidate/round-trip trees,
the prepared driver tree, and the immutable original CLI tree matched. The
negative result and cache cleanup passed the recorded positive safety gate.

Host effects and cleanup
------------------------

Unified logs directly identify the real Autoupdate/Updater helpers: PIDs
21638/21639 for negative and 26922/26923 for positive. Periodic job snapshots
missed their transient job labels. Expected exact labels, sampled state, helper
identities, and final PID absence are retained. No job removal command was needed.

Each case created three exact fixture cache roots. Their saved baselines showed
absence. Cleanup recorded their identities and trees, then removed only those
roots after fixture processes and jobs were absent. Four private preferences/HTTP
storage files remain inside fixture `config` as evidence. Final reads show no
owned helper PIDs, fixture jobs, or listener on port 25600. The installed app and
daemon remained visible as PIDs 1040/1060; neither was controlled.

The filtered LaunchServices readback was empty before the negative run. Later
readbacks contain the private driver and its nested Updater.app paths. The
immediate positive readback did not contain the installed target path. Replacement
changed its root timestamp. Positive unified logs record the staged candidate's
Gatekeeper/XProtect scan, `scan finished`, and raw `GK evaluateScanResult: 2`.
They also preserve `XProtectError` code 2 with an unresolved Xcode Swift 6.2
Sparkle rpath, plus a metadata denial for `mds`. These diagnostics were not repaired
or hidden. No standalone gktool PID or exit was sampled. No trust outcome is
inferred from the scan. Normal scanning, notification, and validation stayed enabled.
No identity-wide unregister or policy reversal ran. Shared LaunchServices and
Gatekeeper state may persist; full restoration is not claimed.

Preserved failures and limits
-----------------------------

The first controller preflight failed because Apple `/usr/bin/python3` has no
`os.listxattr`. It bound no server and launched no CLI. Its exact controller,
traceback, command receipts, and result remain under `raw/negative/` and
`raw/controller-before-negative.py`. The correction recorded the unsupported
xattr API and used separate one-shot CLI directories. No app artifact or reviewed
command changed. Neither CLI case was retried.

The negative CLI ran at observed nice **20**, because outer and inner nice 15
wrappers accumulated. This reduced its priority further. The positive CLI used
one wrapper and was observed at nice **15**. The first result was not repeated.

Tree equality covers file content, modes, and symlink targets. Extended attributes
were not compared. Ownership and root timestamps were recorded separately.
Process sampling used nominal 100 ms intervals plus command time; it can miss
short-lived processes. The controller never launched the target, observed no
target process, and recorded no breach. The pinned stopped-target relaunch
condition is separate source corroboration, not a kernel execution audit.

No target relaunch, running-target update, daemon handoff, production updater,
Developer ID, or notarization acceptance is established. No unit tests, mocks,
GUI driving, Rust/Xcode/TypeScript build, production key access, installed-app
change, push, or publication occurred. The original unaccepted CLI build and its
prior host-limit breach remain unchanged and unaccepted. The earlier preparation
controller's missing original transcript remains a documented preservation limit.

Evidence binding
----------------

[results.json](results.json) records outcomes and limits. [manifest.json](manifest.json)
binds 139 byte-exact private evidence files, including complete CLI stdout/stderr,
server bytes, process/job/cache snapshots, OS receipts, preflights, and cleanup.
No application executable, application archive, or private key is committed.

Evidence ID:
`d8001bf18a06eae043e04a76ac0ca899ebe3fce975a1f89b835b0a65b2e57ccb`.
Its canonical digest definition is in `results.json`. Raw case-result SHA-256:

- Negative: `cefd1e0d939ef7a29fee1084acccee998d179e9cc48e24a0f55795bd54ac003e`.
- Positive: `ba32573fa3e83d1e24b56b8d5f3967de58ee0bd8bf0eedfa9111d064f6cc925e`.
