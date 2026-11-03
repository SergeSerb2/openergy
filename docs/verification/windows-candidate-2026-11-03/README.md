# Windows downloaded-candidate verification — 2026-11-03

Source: `e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`, tag `v0.1.0`.
Scope: Windows portions of SER-199, SER-200, SER-201, SER-212 and SER-213.
This record does not close any native desktop acceptance gate.

The runner downloaded the published Windows ZIP, checksum and JSON sidecars with
`gh release download v0.1.0 --repo SergeSerb2/openergy --pattern '*windows*'`.
It transferred the ZIP to a new private Windows temporary directory and expanded
it there. Every daemon used that extracted executable, explicit private config
and socket paths, `--platform virtual`, disabled discovery, and an explicit denial
of port 24800. All checks ran through key-authenticated SSH in session 0. No
installed service, task, login entry, native clipboard or desktop input was used.

Environment: Windows NT 10.0.26340.0, x64, PowerShell 7.6.4; the SSH token was an
administrator token. PowerShell 5.1 also ran the migration negative checks.
See [environment and cleanup](environment-and-cleanup.json).

## Results

| Check | Result | Evidence |
| --- | --- | --- |
| ZIP SHA-256 vs sidecar | Pass: `abd5d4f80f1141e5893881d82bdfa0dd5f93f49a8d3a43a863801f8bb9cd53b9` | [Provenance](artifact-provenance.json) |
| Root BUILD.json vs JSON sidecar | Pass; exact source SHA, clean source, target x86_64-pc-windows-msvc, Rust 1.99.1 | [Provenance](artifact-provenance.json) |
| Notices, licenses, packaged PowerShell/CMD scripts | Pass: byte-identical to source commit except the expected CMD line-ending conversion; hashes retained | [Inventory](artifact-provenance.json) |
| Executable version and Authenticode | `openergy 0.1.0`; `NotSigned`, as published | [Provenance](artifact-provenance.json) |
| Public CLI/control/pipe behavior | Pass: 23 checks, including DACL, duplicate daemon refusal, subscriptions, fragmented frames, saturation recovery and parent exit | [Control](control.log) |
| IPC deadlines | Pass: all 19 checks; clean shutdown and endpoint removal | [JSON](ipc-deadlines.json), [log](ipc-deadlines.log) |
| Pipe client identification | Pass: identification level 2, no full token impersonation | [Log](pipe-identity.log) |
| Migration privilege boundary | Pass for 35 negative/read-only checks; limited-token checks blocked | [Log](migration-boundary.log) |
| Lifecycle and ownership suites | Blocked by required non-administrator guard; each exits 1 before installation | [Lifecycle](lifecycle.error.log), [ownership](lifecycle-ownership.error.log) |
| Extracted Openergy server and client | Pass: mutual approved fingerprints, connected TLSv1_3/trusted status, disconnect to idle, reconnect, clean exit 0 for both | [Log](tls-pair.log), [driver](tls-pair-driver.py) |
| Synergy 1.21.1 both roles | Blocked: no Synergy installation at documented Program Files locations and no matching installed service on the reachable host | See limits below |
| Cleanup | Pass: zero processes with an executable under the run path; protected ACL fixture removed; private run directory, config, TLS keys and logs removed after evidence collection | [Cleanup](environment-and-cleanup.json) |

`openergy-run.cmd` is the one byte-level source exception: `.gitattributes`
requires CRLF, so its 502-byte LF Git blob becomes 511 packaged bytes.
Reconstructing that conversion gives the retained package hash, and normalizing
CRLF back to LF gives the Git blob exactly. The inventory keeps
`matches_source: false` for the byte comparison and records the normalized
match separately. All other compared script/license/notice bytes match.

Extracted executable SHA-256:
`69ad2c7ffd946bc205b48b7942ad2e64993861089a54ecc6bc536124aec619d9`.
It stayed unchanged through all checks. [Command results](command-results.json)
record the suite exit codes and elapsed times.

## Reproduction and driver changes

The shipping scripts were unchanged. Only private copies of verification drivers
received port substitutions. [The exact patch](driver-port-remap.patch) maps
25240–25259 constants into 26240–26259, and migration constants 26520–26539 into
26320–26339. The provenance inventory records original and modified driver hashes.
The deadline report's script hash is the modified private driver's hash.

With `$r` set to a new private run directory, `$exe` to its extracted
`artifact\openergy.exe`, and `TEMP`/`TMP` set to `$r`, the commands were:

```powershell
$env:OPENERGY_ALLOW_PORT_24800='0'
& "$r\drivers\verify-control.ps1" -BinaryPath $exe -Port 26241
& "$r\drivers\verify-ipc-deadlines.ps1" -BinaryPath $exe -ReportPath "$r\deadlines.json"
& "$r\drivers\verify-pipe-identity.ps1" -Exe $exe
& "$r\drivers\verify-migration-boundary.ps1" -BinaryPath $exe -Port 26330 -Lab "$r\migration"
& "$r\drivers\verify-lifecycle.ps1" -BinaryPath $exe -Port 26240
& "$r\drivers\verify-lifecycle-ownership.ps1" -BinaryPath $exe
```

The TLS driver ran with `python.exe`, beside the private `artifact` directory.
It creates fresh config and TLS state under `tls-pair-4`, uses loopback port
26270, and stops only its own children. Its final run passed. Earlier harness
attempts used an entire fingerprint command response as a fingerprint, an invalid
IPC method, and then an unsupported `--raw` flag on a direct CLI command. Those
were corrected in the retained driver. They are not reported as product failures.

## Open acceptance criteria

- **SER-199:** limited-user install/start/stop/restart/uninstall and occupied-pipe
  ownership checks still need a usable limited token. The elevated SSH token is
  correctly refused. Logon startup, task behavior, hook removal and actual desktop
  lifecycle acceptance remain open. No account or desktop ACL was changed.
- **SER-200:** both PowerShell engines' restricted medium children exited
  `0xc0000142` before user code in session 0. Kernel migration pipes denied the SSH
  Network SID. Therefore successful medium-token broker binding and per-user
  lifecycle did not run. The script reports its limits explicitly. Pending
  migration, commit, rollback, service/startup restoration and failure recovery
  need separate interactive acceptance. No service migration was attempted.
- **SER-201:** no third-party peer was started. The checkout has the older
  `tools/windows-lab/lab.ps1`, not a `PRIVATE-INSTANCE` driver. Its setup/trust
  commands can touch shared Synergy paths, so it was not used. The connected
  Windows host had no documented Synergy core or service. Deskflow/Barrier/Synergy both-role compatibility, wrong-pin refusal,
  input and clipboard acceptance remain open. The Openergy pair is only a package
  TLS smoke check.
- **SER-212:** this record covers the downloaded Windows package and real-process
  Windows checks only. It does not establish pinned-toolchain build/audit, Docker
  interop, packaged macOS app or native desktop results.
- **SER-213:** Windows archive provenance and extracted virtual/TLS/IPC smoke are
  verified. Windows remains unsigned. No macOS signing/notarization/Gatekeeper
  claim is made.

Evidence replaces user paths with placeholders. It contains no private keys,
credentials, clipboard data, input events from a desktop, or machine addresses.
No Linear update, commit, push or PR was performed.
