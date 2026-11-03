# SER-198: public update channel

Source: `e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`.
Issue: <https://linear.app/sergedev/issue/SER-198/verify-public-sparkle-feed-access-and-reconcile-update-channel-docs>.
The issue had no comments when read. Checks ran on 2026-11-03 at approximately
15:03–15:07 America/Vancouver (22:03–22:07 UTC), with the Mac refusal
record added at 15:11; individual requests retain UTC timestamps.

The public feed is reachable with ordinary curl and a Sparkle-style user agent.
It still has **zero release items**, although GitHub v0.1.0 is published.
The earlier 403 is reproducible with Python's default user agent. No universal
outage is established. SER-198 remains incomplete: the unchanged signed app
refused the isolated updater harness before making a request (exit 64), because
that harness requires a disposable bundle identity and private defaults domain.
See the [Mac refusal record](../macos-candidate-2026-11-03/updater-release-refusal.json). These GETs do not prove Sparkle
runtime, update installation, or native input acceptance.

## Real requests

All requests use HTTPS with normal certificate validation. There is no resolver
override, proxy override, redirect following, login, credential, or access-rule
change. Curl runs with a minimal environment (`PATH`, `HOME`, `TMPDIR`) and ignores
curlrc. No proxy environment variables were present in the Linux run.
`Openergy/0.1.0 Sparkle/2.10.0` is an emulated user agent, not a claim about the
exact user agent emitted by the signed app.

| Client and OS/build | Ordinary curl feed | Sparkle-style feed | Release items |
| --- | --- | --- | --- |
| Linux aarch64, kernel `7.2.12-orbstack-00394-gbecdd6130068` | 200 | 200 | 0 in both |
| Host macOS 27.2, `26B5112a`, arm64 | 200 | 200 | 0 in both |
| SSH `m1-dev`, macOS 27.0.1, `26A434`, arm64 | 200 | 200 | 0 in both |

All six feed responses were 756 bytes and had SHA-256
`c693230197b548eba00fd2ea25a7a696cef218d1e854410f0c68934bcc6dd2d7`.
The Linux channel-page GET returned 200. The nonexistent-object GET returned 404.
These Mac checks use command-line GETs only. They do not open an app, start a
sharing daemon, drive the desktop, or change preferences.

The real `tools/release/deploy_updates.py verify` command exited 1 with
`updates deployment: HTTP Error 403: Forbidden`, with both the inherited and
minimal environments. It wrote no success report. A controlled Python urllib
comparison, with the same URL and `Cache-Control: no-cache`, returned:

| User-Agent | HTTP | Feed bytes/hash |
| --- | --- | --- |
| `Python-urllib/3.12` | 403 | 17-byte denial; release count unknown |
| `curl/8.5.0` | 200 | Matches the source feed |
| `Openergy/0.1.0 Sparkle/2.10.0` | 200 | Matches the source feed |

The denial response identified `server: cloudflare`. This narrows the observed
behavior to the user-agent comparison. It does not identify a specific access
rule or prove which infrastructure component generated the denial. The verifier's
403 must not be interpreted as an empty feed or a failure for every app user.

## Configuration, source artifact and signature

`apps/macos/Support/Info.plist` sets:

- `SUFeedURL`: `https://openergy-updates.sergeserbinenko.com/appcast.xml`
- `SUPublicEDKey`: `T6P8IZ5N6ivIZD3SdPRTOKWImB81Lfj2PFQqH4TMZ6o=`

The source plist has version/build placeholders, not the packaged app's final
values. `AppUpdater.swift` validates the bundle configuration and returns its
validated feed through `feedURLString(for:)`, preventing a legacy preference from
replacing that URL. The real `OPENERGY_REQUIRE_UPDATES=1 python3
tools/release/sparkle.py config` command exited 0 (`Sparkle: configured`) with
source defaults and no update environment overrides. This validates source
configuration. The separate Mac agent's fresh
[artifact inspection](../macos-candidate-2026-11-03/artifact-inspection.json)
corroborates the published signed ZIP: app 0.1.0 build 108 embeds the same URL
and key, and its computed archive SHA-256 matches the GitHub digest below.
On M1 macOS 27.0.1 build 26A434, strict/deep code-signature verification, stapler
validation and Gatekeeper assessment returned success. A curl GET of that
bundle's configured URL returned 200, 756 bytes, the same feed hash and zero
items. Those checks do not execute the signed app's Sparkle client.

The live response matches `deployment/cloudflare/public/appcast.xml` exactly.
The file is a bootstrap RSS feed with no `item` or enclosure, so there is no
application payload URL or payload signature to verify on this channel.
Its trailing Sparkle signature records `length: 613` and:

```text
NA5gsOXfgnsQRpHmr8jtPANKF6a5Qa/eji1VyEyIlVOV9vqbCjy0SAWPkNrSzh8w32YfL73lrNsP4muqjZCIDw==
```

OpenSSL 3.0.13 verified the Ed25519 signature over the first 613 bytes against the
source plist's public key. Exit 0: `Signature Verified Successfully`. This is
an independent public-key check, not Sparkle execution or Apple code-signature
validation. The capture script constructs only a public DER key; no private key
is read or created. The full file hash above includes the signature trailer.

## Routing and publication review

Read-only source review found the dedicated R2 bucket `openergy-releases`, custom
domain `openergy-updates.sergeserbinenko.com`, and pinned Wrangler `4.152.0` in
`deployment/cloudflare/updates.json`. `deploy_updates.py bootstrap` uploads the
source feed/page, enables the custom domain with TLS 1.2 minimum, and preserves an
existing configured channel. It does not upload application archives. This is
source intent; no authenticated inspection of deployed DNS, WAF, Access, bucket
policy, or dashboard settings was performed. No credential was requested or used.
No configuration in this source review establishes the live denial rule.

`sparkle.py archive` creates a local signed universal ZIP, signed appcast and
`update.json`. It does not publish them. The draft GitHub workflow and
`release.py`/`trusted_release.py` create draft releases through separate gates;
they do not publish an R2 appcast. GitHub publication cannot demonstrate Sparkle
publication. Historical checks in
[the 2026-10-23 report](../sparkle-integration-2026-10-23/README.md) remain
historical; they are not fresh signed-app acceptance.

The unauthenticated GitHub release API returned 200, independently of the feed:

- Tag `v0.1.0`, name `Openergy 0.1.0 (alpha)`, `draft=false`, `prerelease=true`.
- Published `2026-11-02T18:27:22Z`; `target_commitish` is the source SHA above.
  This metadata field is not independent resolution of the tag object.
- macOS asset `Openergy-0.1.0-macos-universal.zip`, 7,780,962 bytes.
- GitHub-reported digest:
  `sha256:acd5c81113670edd34658d14ca489ce083758acc4df1ae5676b4308bcda6e9b0`.
- Release notes state Developer ID signing and notarization. They explicitly
  list update installation through Sparkle as unverified.

All eight asset metadata entries and the release notes are retained in
[results.json](results.json). No release archive was downloaded or executed by
this investigation. This investigation records the GitHub digest as metadata; the linked Mac
artifact inspection separately computed the matching archive hash.

## Reproduce and retained evidence

From the repository root, use fresh output paths under private temporary state:

```sh
umask 077
run_dir=$(mktemp -d /tmp/oe-update-channel.XXXXXX)
python3 docs/verification/update-channel-2026-11-03/capture.py \
  --output "$run_dir/results.json"
# Requires the authorized host bridge and existing BatchMode SSH to m1-dev.
python3 docs/verification/update-channel-2026-11-03/capture-clients.py \
  --output "$run_dir/macos-clients.json"
```

[capture.py](capture.py) performs real curl/urllib GETs, the real source config
and hosting-verifier commands, and public Ed25519 verification.
[capture-clients.py](capture-clients.py) performs real host/SSH GETs.
[results.json](results.json) records source hashes, commands, tool versions,
responses, signature result, verifier failures, and GitHub metadata.
[macos-clients.json](macos-clients.json) records the two Mac clients.
Only selected response headers, body sizes/hashes, and public release fields are
retained. Cookies, network addresses, raw denial pages, personal paths, account
IDs and credentials are omitted. No daemon roles, peer versions, IPC sockets or
TLS peer identities apply to these HTTPS-only checks. No port 24800 was used.

## Signed-release harness refusal

The Mac agent subsequently invoked the unchanged signed release's actual
`--updater-e2e` entrypoint in private output/home state. The retained
[refusal record](../macos-candidate-2026-11-03/updater-release-refusal.json)
shows exit 64 after 0.014 seconds, no `updater-result.json`, and an empty private
home. The harness stopped before Sparkle startup: the release bundle identifier
is `dev.openergy.app`, while the harness requires `dev.openergy.updater-e2e.*`.
The release also omits `SUDefaultsDomain`, while the harness requires an absolute
private `<config>/sparkle-preferences` domain. The app/output/home paths satisfied
the private-state setup reported by the Mac agent.

Changing those Info.plist fields would invalidate the release signature. No
bundle mutation, re-signing, rebuild or normal GUI launch was performed. This
is a real safety refusal, not a feed failure, and it proves no signed-app network
request. The Linux capture scripts above still perform HTTP checks only; this
separate execution belongs to the linked Mac evidence.

## Remaining completion criteria

1. The linked M1 artifact inspection now supplies the published app's feed/key,
   version/build, computed ZIP hash, code-signature checks and OS/build. The
   remaining check is to run that signed app's real updater in a separate
   account or test machine and record feed
   status/release count or Sparkle's actual error. The unchanged published bundle
   cannot use `--updater-e2e`: its `dev.openergy.app` identity and absent
   `SUDefaultsDomain` fail the disposable-fixture guard before Sparkle starts.
   Preserve that guard and the original signature. A disposable re-signed fixture can verify runtime
   behavior but cannot establish acceptance of the original signed artifact.
2. If that app or another legitimate client is denied, a release owner must
   inspect the deployed Cloudflare routing/access rules read-only and correlate
   the failing request. This report does not authorize rule changes or deploys.
3. For any future nonempty feed, retain its exact bytes, payload URL, byte count,
   checksum and EdDSA signature; verify against the bundle public key and the
   matching immutable archive. Download, installation and relaunch need their
   own isolated native acceptance. They did not run here.

No updater source fix was justified by these checks. Only documentation and this
new evidence directory changed. No unit tests, mocks, commits, pushes, PRs, Linear
updates, release uploads or child agents were created.
