# R6 native Windows build only: PASS

The fixed source `06b9a139ec855a6f697ded393fa4c2c894110613` passed one
native strict release Clippy invocation and one native release build on
2026-10-23. Clippy returned 0 in 18.688 seconds. The release build returned 0
in 47.248 seconds. There were no native compiler errors. No app, daemon or
example was executed, including with `--help`.

The exact scope was `openergy-net --lib --example dns_queue_cancel_c4806a3d`:

```text
cargo clippy --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-net --lib --example dns_queue_cancel_c4806a3d -- -D warnings
cargo build --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-net --lib --example dns_queue_cancel_c4806a3d
```

See [result.json](result.json), the actual
[Clippy receipt](evidence/downloaded/clippy-result.json),
[Clippy output](evidence/downloaded/clippy.stderr),
[build receipt](evidence/downloaded/release-result.json), and
[build output](evidence/downloaded/release.stderr).

[controller.py](controller.py) was derived from the R5 controller at
`4c98c28be71fe43a014ec59606b422ca89268e6f`. Every Git operation with source
pathspecs uses explicit `git -C REPO_ROOT` and the repository root as its
working directory. The R6 controller ran once, from 18:18:19 to 18:22:40
America/Vancouver. Its measured nice was 15, and its macOS deployment target
was 15.0. The fresh R6 guard remains present. No setup correction, resume,
compiler retry or watchdog stop occurred during the Windows workflow.
[workflow-bindings.json](workflow-bindings.json) binds the scripts. All seven
control/transfer receipts bind the same executed controller SHA-256.

[source-manifest.json](source-manifest.json) lists all 558 exported files.
Each archive member was compared byte-for-byte with its Git blob. No source
file, overlay or line ending was changed. The archive was 4,065,280 bytes,
with SHA-256
`6d11dc7edeb4ac4fd18058d2a56015c0e808f3aa7e61a2a483f17a1e993246a4`.
The requested module
`crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs` had SHA-256
`9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4`.
Windows checked each transferred file's length/SHA before extracting the
source. It checked all 558 extracted files before and after compilation.
The source files were read-only. See
[source-before.json](evidence/downloaded/source-before.json) and
[source-after.json](evidence/downloaded/source-after.json).

The workflow used existing OpenSSH authentication and fixed-file `scp -O`,
with one upload and one evidence download. Control commands used immediate
stdin EOF. Transfers had 90-second watchdogs. Each compiler step had a
600-second watchdog. A stop would require the exact owned PID, creation
identity and parent chain. All seven control/transfer commands returned 0;
none timed out or stopped a process.

[qualification.json](evidence/downloaded/qualification.json) records installed
Rust/Cargo 1.99.1 and native `x86_64-pc-windows-msvc`. MSVC 14.44.35207 and
Windows SDK 10.0.26100.0 paths applied only to the child environment. Cargo
used a private Cargo home with a verified copy of the existing 6,300-file
registry. Registry source files were read-only. Global Cargo configuration
and credentials were not copied. Jobs were 2 and dependency resolution was
locked/offline. Build scripts needed by Cargo ran as compiler work; no
product output was run.

The controller recorded 104 actual owned process observations: 3 Cargo,
1 cargo-clippy, 84 rustc, 7 clippy-driver, 3 cl, 1 link, and 5 Cargo build-script
processes. Every observed process was BelowNormal in session 0. The
[Clippy observations](evidence/downloaded/clippy-processes.json) and
[release observations](evidence/downloaded/release-processes.json) retain
PID, start time, parent PID, command line, path, priority and session.
There were 30 Clippy and 6 release observation races with short-lived
processes. These records prove the observed identities and priorities;
they do not claim every short-lived process was sampled.

[postflight.json](evidence/downloaded/postflight.json) passed. The protected
current-user-only R6 root had 9,971 checked ACL items and no invalid ACL or
reparse point. No owned build process remained. The before/after snapshots
of nine prior fixtures, original registry, pinned toolchain, eight selected
default-file states, sixteen tool binaries and live sharing process
identities were exactly equal. The 20,834,593-byte original snapshot SHA-256
was `12c9c06aa4f8fbded3e66cdc16fc197b66faa8a81eaeea6909b600de9b58b165`
both times. [shared-input-summary.json](shared-input-summary.json) retains
these bindings and concise tree counts. Full snapshots remain private.
The separate DNS qualification helper's roots/processes were not used or
changed.

The example artifact was 2,521,600 bytes with SHA-256
`862d20212522ccd4bba242d1651af9a8ca960e381a818205f07cf874aa53958d`.
The 44,900,352-byte evidence bundle had SHA-256
`f85dfb49cc0425bdfa7788cd7b109135bcb762d1b930ec092185254aebee2717`.
Both the evidence bundle and downloaded artifact were length/hash verified
before cleanup. The artifact remains private and was never executed or
committed. The final cleanup receipt in [result.json](result.json) confirms
removal of only the new R6 root, with no owned process left. This is build
fixture cleanup, not runtime cleanup acceptance.

R5 is preserved. Its entire 22-file tree, including private location
metadata, first failure, initial code, amended code and original guard,
remained byte-for-byte unchanged. See
[r5-preserved-before.json](r5-preserved-before.json) and
[r5-preservation-result.json](r5-preservation-result.json). Its original
`git archive` exit 128 occurred locally because `Cargo.toml` was interpreted
relative to the evidence directory. R5 performed zero SSH, transfer,
compiler or runtime actions. Its original guard SHA-256 remains
`38ddd5e5778429ce47a26e1be9eb67979961c5b73fcebc7915910ea024012579`.
R6 neither reran R5 nor reset that guard. The existing R4 source audit at
`8704f63` remains a separate source review.

One later **local evidence packaging check** returned 1 because two derived
SCP receipts still contained the known profile prefix with forward slashes.
This happened after native success, evidence download and remote cleanup.
[local-packaging-first-failure/](local-packaging-first-failure/) preserves the
first error, original checking code and its SHA-256 identity. The corrected
[package-evidence.py](package-evidence.py) masked those two derived copies
and updated [copy-ledger.json](copy-ledger.json). It verified existing bytes
locally and ran zero remote/compiler commands. The exact raw receipts remain
unchanged privately. [packaging-verification.json](packaging-verification.json)
records the successful corrected check. Large shared snapshots and the
duplicate source manifest are omitted from committed copies; their original
lengths/hashes remain in the ledger and summary.

[build-only-stop-audit.json](build-only-stop-audit.json) records the limited
scope. No product source, test, mock, installed app, production config/TLS,
default IPC, TCC, login item, service, account, firewall, DNS or toolchain
default was changed. No app socket, TCP 24800 or UDP 5353 fixture was used.
No desktop driving, delegation, Docker, installation, push, publication or
Cloudflare change occurred. Native compilation is now verified for this
fixed source and scope. Runtime, strict deadline, security/provenance,
publication and broader release acceptance remain outside this result.
