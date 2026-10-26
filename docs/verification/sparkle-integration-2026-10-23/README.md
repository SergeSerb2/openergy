# Sparkle and Cloudflare integration

Source `d647a01a7b83f79187d1a15c067a763647f01a31` includes Sparkle 2.10.0,
normal app lifecycle, three Check for Updates controls, Advanced update settings,
complete framework packaging, signed universal update archives and the dedicated
Cloudflare channel. The earlier native archive was built from clean `6ab225e`.
The only subsequent source change corrects the generated feed title before signing.

The live feed is <https://openergy-updates.sergeserbinenko.com/appcast.xml>.
The page is <https://openergy-updates.sergeserbinenko.com/index.html>.
The dedicated R2 bucket is `openergy-releases`. The public feed is signed and
contains **zero application releases**.
No application archive was uploaded.

## Real command results

| Flow | Result |
| --- | --- |
| Whole native `release.py package`, clean `6ab225e` | Exit 0, 35.178 s. Built bundle, preserved ZIP links/modes, checked staged and extracted virtual daemon IPC, verified extracted app signatures. |
| Final universal bundle, `d647a01` | Exit 0, 33.852 s. Both architectures, full framework/helpers/XPC, nested signatures and deployment target checks passed. |
| Final signed universal archive | Exit 0, 2.428 s. Official file-only signing, archive and feed verification against bundled public key, extraction/tree comparison and nested signature checks passed. |
| Loopback empty feed | Exit 0, real Sparkle reports no update. |
| Loopback signed candidate feed | Exit 0, real Sparkle selects build 2; delegate stops before download. |
| Loopback HTTP 503 | Exit 0 for expected failure; real feed retrieval fails. |
| Loopback malformed XML | Exit 0 for expected failure; real parser fails. |
| Unconfigured private bundle | Exit 0, updater never starts, zero checks. |
| Live Cloudflare URL | Exit 0, real Sparkle loads feed and reports no update. No resolver override in this app command. |
| Public hosting CLI | Exit 0; HTTPS 200 RSS, title and content type, empty channel, channel page 200 and missing object 404. |
| Formatting and notice freshness | Exit 0. Full upstream Sparkle license and exact dependency records retained. |

The local server received exactly four feed requests and no archive request.
All six final updater commands use disposable app copies, unique fixture bundle
IDs, private mode-0700 config directories, absolute `SUDefaultsDomain` paths and
`CFFIXED_USER_HOME`. They use the real standard Sparkle controller and delegates.
No AppModel, sharing daemon or normal app scene starts in these commands.
The whole-package checks separately start only their private virtual daemon.
The final postflight found no owned app/daemon process, assigned listener,
fixture preference outside its private config or remaining package staging directory.

The final update ZIP is 7,492,715 bytes, SHA-256
`06aca9d4652a3fa39c72917c645562075021cda37023e62afa3f3042c776c547`.
It remains private. Its generated signed feed has the exact `Openergy Updates`
title expected by the hosting verifier. The live empty feed SHA-256 is
`c693230197b548eba00fd2ea25a7a696cef218d1e854410f0c68934bcc6dd2d7`.
Signing uses the dedicated key outside Git. No key material was copied into evidence
or the repository, and no Keychain key was created or changed.

## Preserved failures and review response

The first combined universal bundle compiled and verified its code signatures,
but its real executable aborted with dyld exit -6: ad-hoc code under Hardened
Runtime could not load the framework because it lacked a matching Team ID.
The final local ad-hoc mode explicitly uses signing options `0`. Named identities
retain Hardened Runtime. No library-validation exception entitlement or global
security setting was added. The original failure and intermediate artifacts remain.

The first private check of the public feed returned Sparkle error 2001. Later
diagnostic and final real app checks passed. Python's ordinary URL lookup still
returns `[Errno 8] nodename nor servname provided, or not known`. Its cause is
not established. The separate hosting CLI uses a request-specific address checked
against public Cloudflare DNS, retaining hostname, SNI and certificate validation.
This does not change system DNS. It is separate from the successful normal-URL
Sparkle command.

A fresh private headless Chrome profile checked the channel page and feed DOM.
That browser used only a process-specific resolver mapping, with certificate
validation enabled. This proves these browser paths, not the native settings UI.

Source review found two issues. An extra `sessionInProgress` condition blocked
manual refocusing of an active update window; the fix follows Sparkle's
`canCheckForUpdates` directly. The feed generator used `Openergy updates` while
the hosting check expects `Openergy Updates`; `d647a01` corrects it before
signing, and the final archive flow and candidate check passed with that feed.
A final source and evidence review found no further issues. These were source
reviews, not independent runtime reproduction.

## Evidence and limits

[results.json](results.json) binds 456 source files, four retained binary/archive
artifacts and 26 byte-exact copied records. The raw commands, logs, result JSON,
archive manifest, signed generated feed and controller scripts are in `raw/`.
The original private artifacts are not in the repository. Failed transcripts
were not rewritten.

This proves local bundle loading, real feed checks/candidate selection, local
EdDSA archive/feed validation and live empty-feed hosting. It does not prove the
standard visual UI, signed archive download through Sparkle, installation,
daemon handoff or relaunch. Developer ID signing, notarization, Gatekeeper,
native input/clipboard, Windows migration/DNS/discovery acceptance, original
security/provenance clearance and trusted distribution remain separate gates.
The source-history scan predates these changes. No application release was
published, installed or advertised by the empty feed.
