Native Windows daemon build passed. The virtual Off stop path remains UNVERIFIED.
The one runtime attempt failed in this harness before healthy IPC or daemon.stop.
No runtime retry or product edit followed.

Source: `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a`. This source contains production
change `2485c142` and saved review `63915871`. The complete relevant export contains
558 files from the workspace manifests, crates, tools/e2e, and vendor. Every member
matches its immutable Git blob and the local source bytes. Windows checks each
file's bytes, SHA-256, read-only state, CRLF count, and LF count before compilation,
after compilation, and after the runtime attempt. There is no source overlay or
line-ending conversion. See [source-manifest.json](source-manifest.json) and the
three source receipts under [evidence/downloaded/](evidence/downloaded/).

Exactly one native strict release Clippy command and one conditional release build
ran, with installed Rust/Cargo 1.99.1, jobs 2, native x86_64-pc-windows-msvc,
locked/offline resolution, MSVC 14.44.35207, and Windows SDK 10.0.26100.0:

```text
cargo clippy --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-daemon --bin openergy -- -D warnings
cargo build --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-daemon --bin openergy
```

Clippy returned 0 in 20.854 seconds. The release build returned 0 in 83.494 seconds.
See [clippy-result.json](evidence/downloaded/clippy-result.json),
[clippy.stderr](evidence/downloaded/clippy.stderr),
[release-result.json](evidence/downloaded/release-result.json), and
[release.stderr](evidence/downloaded/release.stderr). Each Cargo root has a retained
process handle, creation/start identity, parent PID, and actual BelowNormal/session-0
observation. The 126 sampled process records contain 3 Cargo, 1 cargo-clippy,
9 clippy-driver, 101 rustc, 4 link, 5 build-script, and 3 cl observations. All are
BelowNormal in session 0. There are 24 Clippy and 9 release observation races.
Parent-chain checking uses PID plus creation time and retains observed PID reuse.
Every sampled chain reaches its Cargo root. All four sampled linkers belong to
Clippy compiler work. No release linker was sampled. Final-linker priority is
UNVERIFIED; priority is not inferred for any unsampled process.

The AMD64 PE32+ artifact is 4,139,520 bytes, SHA-256
`4984ef1222bbcd551dbebdda9db2fc623ff42a3560c5261806f09ad8f3dbda2f`.
Its hash was checked before runtime and after recovery, and its downloaded bytes
match the build receipt. The artifact remains private. The build receipt's
`artifact.executed=false` describes the build stage only; the runtime did execute
version, absent-daemon status, and the virtual daemon from this artifact.

The one finite runtime attempt passed actual binary version (exit 0) and
absent-daemon status (expected exit 1). It then launched the foreground virtual
role-Off daemon. Every product process was created suspended with BelowNormal
priority, sampled in session 0 with its real parent identity, and resumed only
after the controller retained its handles. The controller PID and lifetime were
established before passing --parent-pid. All product config, profile, data, cache,
and temp paths were private before launch. Discovery was disabled, autoconnect was
false, and TLS and clipboard were disabled. The literal configuration used
127.0.0.1:59367. Initial exclusive IPv4 and IPv6 loopback binds succeeded without
listen or connect calls. The named pipe was fresh and private.

The readiness reader then called .NET File.ReadAllText on the active daemon log.
Windows rejected that open because its sharing mode conflicted with the writer.
This is a harness failure; it is not evidence that daemon.stop failed. See
[runtime-first-failure.stderr](evidence/downloaded/runtime-first-failure.stderr),
[runtime-result.json](evidence/downloaded/runtime-result.json), and
[runtime-daemon.log](evidence/downloaded/runtime-daemon.log). No healthy public
status/config, invalid-method request, or daemon.stop was sent. There is no stop
acknowledgement, successful wire request/response identity, normal OS exit code,
or Windows stop timing to report. The running-Off TCP inventory was not reached. Before/after exclusive binds do
not prove that no listener existed while the failed attempt was running. The
private transparent IPC observer was
prepared but no exchange ran. It was intended to forward real CLI frames unchanged;
it provides no runtime acceptance in this attempt.

Bounds were 15 seconds for readiness, 15 seconds per CLI call, and 10 seconds for
normal daemon exit from stop request forwarding. The overall remote workflow was
bounded to 2,400 seconds from the first setup, including the corrected setup.
The normal-stop deadline never began because zero stop requests were sent.
Recovery followed the log-reader error before the readiness ceiling. The raw
recovery field `only_after_declared_deadline=true` is a generic harness label,
not proof of a normal-stop deadline. No one-second contract is claimed.
The controller used the retained process object plus exact executable path,
creation identity, and parent PID to kill only its owned daemon. Its recovery exit
code was not saved. This forced recovery is not normal-exit acceptance. Explicit
handle-close calls and the runtime controller's exit are recorded; no handle-table
or external SSH-job audit was performed. No job or process group was created or
modified by the workflow. No cross-platform timing comparison or speedup is claimed.

The [final cleanup audit](final-owned-cleanup-audit.json) independently confirms
that all four recorded runtime identities are gone, both private named pipes are
absent, and the fresh root is removed. Exclusive IPv4 and IPv6 binds on 59367
succeeded again, without listen/connect. It made no application request and did
not relaunch a product. The main cleanup ran only after the private evidence
archive, every evidence member, and the artifact were downloaded and hash-verified.
The private final bundle is 46,836,736 bytes, SHA-256
`3247d9f23cd5fb078c7c048de7f5ae48e4d71614ab4cd5dcc5a39658d7ea6908`.

[postflight.json](evidence/downloaded/postflight.json) passed. The fresh protected,
current-user-owned root had 10,055 ACL items and no invalid ACL/reparse entry.
The saved before/after bytes match for the nine prior Windows fixture trees,
original registry, pinned toolchain, eight selected default-file states, sixteen
tool binaries, and the one live-sharing identity. These are the established
bounded snapshot fields; this is not an exhaustive host-settings audit. The fresh
Cargo home contained a verified registry-only copy. No Cargo credentials or global
configuration were copied. All build environment changes were child-local.
Local R6/R5/original runtime evidence and the complete first failed setup folder
and private originals remain unchanged. See the shared summaries and
[local-prior-preservation.json](local-prior-preservation.json).

Preserved errors and corrections:

- [preflight-local-errors.json](preflight-local-errors.json) records the unmatched
  local search glob and two failed local transport-signature lookups. No SSH,
  compiler, or product action ran during those failures. The exact configured R6
  transport was then matched against its saved command signature. No key contents
  or endpoint details were printed.
- [setup-failure-01/](setup-failure-01/) preserves the first native setup attempt,
  which failed after writing qualification.json because the ancestor helper reached
  a process without a CIM creation date. It ran zero compiler or runtime commands.
  Its root was archived and removed. [setup-correction.json](setup-correction.json)
  binds the corrected helper, new root, and fresh guard. The old guard was not reset.
  Unavailable ancestor boundaries are now recorded, without inferred ownership.
  The first saved controller_exit field was 0 although its final shell exit was 1;
  both the original field and an explicit observed-exit receipt are preserved.
- The one runtime log-reader failure is preserved unchanged. Runtime was not retried.
- Two cleanup-only audit failures are preserved in evidence/raw: Windows PowerShell
  5.1 rejected device-path enumeration; the PowerShell 7 correction then found that
  normal cleanup had already removed its input root. The final cleanup audit used
  the immutable downloaded identities and passed. No product or compiler ran in
  these audit corrections.
- [packaging-first-failure/](packaging-first-failure/) preserves the failed local
  packaging check that used PID alone. The corrected local check uses PID plus
  creation time. It ran no remote, compiler, or product command.

[copy-ledger.json](copy-ledger.json) binds the private originals and every derived
copy. Known transport/profile/private-root prefixes and nested EncodedCommand
wrappers are masked in derived copies, with BOM removal and preserved line endings.
No raw original was edited. Full shared snapshots, the duplicate source manifest,
archives, and executable remain private. [packaging-verification.json](packaging-verification.json)
and [artifacts.json](artifacts.json) bind the final local package.

Only this new evidence folder is owned. No production source, unit test, mock,
installed app, live sharing, production config/TLS/default IPC, TCC, login item,
service, task, account, firewall, global DNS, or toolchain default was changed.
There was no desktop driving, native input, live-sharing traffic, TCP 24800,
UDP 5353, browse/DNS/discovery probe, optional suite, install, Docker, signing-key
access, delegation, push, or publication. This result closes native compilation
for this exact source and binary scope only. Virtual Off stop, hardware, migration,
Windows UI, strict DNS/discovery, trusted release, security/provenance, and wider
release gates remain unverified. The three provider-stopped tasks were not resumed.
