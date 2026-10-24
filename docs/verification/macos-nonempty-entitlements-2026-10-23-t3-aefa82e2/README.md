# Downloader nonempty-entitlement file flow

**INCOMPLETE: required retained inputs, raw evidence and signed output are now
absent.** The real native file flow passed before this loss, but this report
cannot support the requested independent saved-evidence acceptance. Do not mark
the remaining case verified from the surviving tool transcript alone.

Source: `dd545e52cdc2e5b29a4e5e1604c76a177f9b0e23`, worktree `t3-aefa82e2`.
Production source and prior tracked files remain unchanged. Only this new
folder is owned by this task. [results.json](results.json) gives source hashes,
last observed artifact identities, limits and the missing prerequisite.

## Frozen sequence and observed execution

[freeze.json](freeze.json) was saved before signing. It binds the original
24,746-byte collector (`6630f1d0…`) and command plan (`d8c718e0…`). The full
signer matches priority repair `d3b30e17`; the signing-purpose repair is integrated
through `e749c22`, with independent review through `0506444`. Priority review
`2092db0d` was read as an immutable Git object. No production code was changed.

One fresh mode-0700 root held the complete disposable bundle:
`/private/tmp/oe-nonempty-20261023-t3-aefa82e2-yc0e4rvh`.
Preflight matched the exact prior app, Sparkle artifact and positive fixture to
all 167, 221 and 167 saved inventory entries. The whole-app copy matched every
file hash/mode, directory mode and link target. All seven binaries contained
arm64 and x86_64. No scratch/build database was copied.

Read-only codesign, file, lipo and Info.plist metadata independently identified
Downloader's bundle and executable. Both slices had identifier
`org.sparkle-project.DownloaderService` and empty dictionaries. One setup-sign
on the copied Downloader bundle set exactly these deliberate fixture values:

```xml
<key>com.apple.security.app-sandbox</key><true/>
<key>com.apple.security.network.client</key><true/>
```

These are fixture inputs, not a claim about Apple's original entitlements.
The setup used the established identifier, `--sign - --options 0
--timestamp=none --entitlements FIXTURE_PLIST`, through the current absolute
priority launcher. Native codesign signed both universal slices.

Actual command arrays survive in the frozen plan and
[tool-observations.json](tool-observations.json). The saved receipts were read
before the private files became absent:

- One setup-sign: exit 0, 0.673608 seconds.
- One current public `python3.11 tools/release/sparkle.py sign --app COPIED_APP
  --arch arm64 --arch x86_64`: exit 0, 4.496076 seconds.
- One public `sparkle.py verify` with the same app and architectures: exit 0,
  5.739117 seconds.
- Separate strict all-architecture checks of all seven signed objects: seven
  exits 0. Deep strict all-architecture outer verification: exit 0.

The public sign used an explicit allowlisted environment with
`OPENERGY_SIGN_IDENTITY=-`; purpose and named-signing opt-in were absent. It ran
the unchanged inside-out path for Sparkle's five objects, daemon and outer app.
No tool was substituted. Neither signing command nor public verify was retried.
Commands had 30-second signing/verify/copy or 10-second metadata limits, 1 MiB
per captured stream, a 180-second collector wall budget, file/tree/root limits,
exclusive one-shot guards and retained direct process handles.

## Comparisons observed before loss

Both arm64 and x86_64 output dictionaries exactly equaled the full two-key
fixture dictionary, with Boolean `true` values. Both identifiers exactly matched
the fixture input. Bundle and executable metadata agreed. All seven output
binaries remained universal. The 14 object/slice comparisons found ad-hoc
signatures, flags `0x2(adhoc)`, no Team ID or secure timestamp, and no added
`com.apple.security.cs.disable-library-validation` entitlement.

The setup changed only Downloader's executable. Its SHA-256 changed from
`a2ac3e14…` to `c348da23…`; the full public sign preserved that fixture hash.
The public sign changed only the Sparkle and app executables and their two
CodeResources files. Non-code bytes, all modes and framework links matched.
Tree identities were `595db030…` before setup, `b1e02c07…` after setup, and
`73f6a7c1…` after full signing. Full hashes survive in the observations.
Postflight reported the original app/artifact/old positive fixture and selected
prior evidence bindings unchanged. Those paths cannot now be re-read.

## Collector failure, priority and retention failure

The first collector exited 1 after all signing/verify receipts succeeded. Its
optional `os.killpg(pid, 0)` liveness probe raised
`PermissionError: [Errno 1] Operation not permitted` after the first final
metadata command had been waited. That metadata exit receipt was not saved;
it is not counted as an exit 0. The original failure ledger, raw streams and
collector were preserved at the time. A separate read-only finalizer omitted
that optional probe, collected only codesign display/lipo metadata and remaining
inventories, and exited 0. It did not sign or rerun either public CLI.

All three owned collectors set absolute NI 15 and read back 15. Public sign and
verify parents were observed at NI 15. One actual verification codesign sample
was PID 1283, parent 1161, start `Fri Oct 23 21:05:13 2026`, NI 15. The parent
also had NI 15. **No signing codesign process was identified in the samples.**
Do not extend priority coverage to those unsampled processes.

Completed direct handles were waited. The first 44 recorded group probes
reported absence; the failing metadata probe leaves its group status unknown.
No cleanup signal was sent. Filesystem cleanup only removed owned empty
`tmp`/`cache` directories. No root or old-fixture deletion command was issued.
The cause of the later disappearance is not established.

Before the raw export, the owned root returned `FileNotFoundError`. Exact-path
checks at 21:19:44 PDT also found the prior ledger, source app, source Sparkle
artifact and prior positive fixture absent. [missing-retention.json](missing-retention.json)
keeps that failure and the failed export tool output. No broad scan, rebuild,
substitution, restoration, new fixture or signing retry followed.

The surviving observations are explicitly transcribed tool results, **not
original raw streams or full manifests**. [evidence-bindings.json](evidence-bindings.json)
hash-binds this bounded surviving text. The required complete raw/hash export
and retained signed fixture could not be delivered. Access to the exact bound
bytes and originals is the missing prerequisite for independent review.

No build, package manager, unit test, mock, network, keychain query, named-key or
credential access, launch/install, desktop driving, delegation, push or
publication occurred. Live sharing and host settings were not changed. No TCP
24800/UDP 5353 operation or stopped task was resumed. Runtime, arbitrary or
original production entitlement preservation, current-source compilation,
named-key signing, timestamps, notarization, Gatekeeper and trusted distribution
remain unverified. Earlier failures and empty-entitlement evidence are unchanged.
