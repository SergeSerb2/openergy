# R5 Windows build-only transport controller: UNVERIFIED

The single workflow invocation failed during local archive preparation.
**No SSH command, file transfer, Windows fixture creation, Clippy invocation,
build invocation, or example execution occurred. Native Windows compilation
remains UNVERIFIED.** The workflow was not rerun.

Requested source: `06b9a139ec855a6f697ded393fa4c2c894110613`.
Starting checkout: `38fdc455cb831b05265764a878099715771bafe7`.
The requested module is
`crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs`.
Its Git blob is `91c84ffada825055e0163a1fd53e7d94f8c5d1ee` and its freshly
read Git-object SHA-256 is
`9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4`.
This Git-object check is not an exported-archive or remote-source binding.
No fresh source archive or all-file source manifest was produced.

## First failure and preserved receipts

The local controller ran once at `2026-10-23T17:52:31-07:00`, with measured
nice 15 and `MACOSX_DEPLOYMENT_TARGET=15.0`. Its first subprocess was:

```text
git archive --format=tar 06b9a139ec855a6f697ded393fa4c2c894110613 -- Cargo.toml Cargo.lock rust-toolchain.toml rustfmt.toml crates tools/e2e vendor
```

The controller set that subprocess's working directory to this evidence
directory. Git interpreted the source pathspecs relative to that directory.
Git returned **128** with:

```text
fatal: pathspec 'Cargo.toml' did not match any files
```

The controller caught the exception, saved its first traceback and failed
result, then exited **1**. The failed result remains unchanged in
[result.json](result.json). The initial controller, result, copy ledger,
guard, and exact displayed command transcript are preserved in
[preparation-failure-01/](preparation-failure-01/). The initial controller's
SHA-256 is
`6bee4146edb02afb44f0bb40192381920bc68eb485aa0b88f78a53747a40e2e4`.
The transcript was copied from the first tool result; it is not a newly
captured subprocess stream. The private traceback remains unchanged.

A later static amendment makes the controller resolve the checkout root
before its Git pathspec commands. Only Python AST parsing was performed on
that amended controller. It was **not executed**, and it has no Windows
validation. The original one-attempt guard remains in place. Neither that
guard nor the failed result was reset or replaced.

## Transport repair code, not transport proof

[controller.py](controller.py) replaces the prior stdin/Base64 payload route
with the historical native Windows workflow's fixed-file OpenSSH SCP
protocol (`scp -O`). Authentication stays with normal OpenSSH. The caller
supplies the existing SSH alias and private historical fixture-state path;
the controller does not read keys, tokens, or SSH configuration contents.

The code creates one fresh protected current-user-only Temp directory. It
refuses any existing directory with that name and refuses the old failed
root. It freezes source bytes and a schema-tagged all-file manifest locally,
checks exported bytes against Git blobs, records actual line endings, and
refuses conversions, links, overlays, or unexpected source modes. The upload
contains fixed files, not a shell stdin payload. All SSH control commands
use `DEVNULL` for immediate stdin EOF. SCP must return its completion status
before the next control command. Remote file lengths and SHA-256 hashes must
match before source extraction or compilation. The upload has a 90-second
watchdog and no resend or alternate-transport branch.

[qualify.ps1](qualify.ps1) validates the transferred schema, source commit,
module SHA, archive SHA/length, and every extracted file. It selects the
installed native Rust 1.99.1 MSVC toolchain and a byte-checked private copy of
the existing registry. Credentials and global Cargo configuration are not
copied. Source files and copied registry source files become read-only.

[build.ps1](build.ps1) defines one strict release Clippy command and one
release build, scoped to `openergy-net --lib --example
dns_queue_cancel_c4806a3d`, with `--locked --offline --target
x86_64-pc-windows-msvc --jobs 2`. Clippy adds `-- -D warnings`. The build is
skipped after a nonzero Clippy result. MSVC 14.44.35207 and Windows SDK
10.0.26100.0 paths apply only to the child environment. The controller
samples actual owned Cargo/compiler/linker process identities, command
lines, session IDs and priorities. Version queries apply only to Rust tools.
The example executable is never invoked, including with `--help`.

Each compiler step has a 600-second watchdog. Stops require the process ID,
creation time, and the owned parent chain. An outer stage watchdog uses the
exact unique encoded control command and its identified descendants. There
is no identity-wide kill. No watchdog or stop ran in this attempt.

[common.ps1](common.ps1) and [postflight.ps1](postflight.ps1) define before/
after byte, read-only and ACL checks for prior fixtures, the original
registry and pinned toolchain; selected default files and native tools; and
live sharing process identities. They also define source requalification,
actual compiler/linker observation gates, artifact hashing, protected ACL
checks, and a build-only process audit. A fixed-file evidence download must
match its remote byte count/SHA before removal of only the new owned root.
All these remote steps remain **unexecuted and unverified**.

## Reconciliation and boundaries

After the parent resume message, [resume-reconciliation.json](resume-reconciliation.json)
checked the completed exit-1 receipt, unchanged one-attempt guard, private
raw-file inventory, and exact local controller/transfer process inventory.
No live exec session was returned by the original command. No owned local
controller or transfer process remained. The only private raw file was the
first local traceback; no setup receipt existed. Control flow stopped before
its first `remote()` call, so there was no remote PID or fixture to clean.
No case was resumed, duplicated, resent, or retried.

[source-and-prior-object-bindings.json](source-and-prior-object-bindings.json)
binds the requested module and all **70** original evidence files in commit
`52cbf8c3c41b15789500ce7be620d203006630a9`. Those Git objects were only read.
The prior stdin setup/controller was inspected and never executed. Its
underlying transport failure remains unknown; this attempt adds no diagnosis
of that old failure. The old empty root was not recreated.

[build-only-stop-audit.json](build-only-stop-audit.json) records zero remote
actions, compiler invocations, runtime invocations and stopped processes.
The selected source and production libraries were not edited. No native
backend/input/clipboard, desktop action, installed app, service, account,
registration, firewall, DNS, TLS store, production config, default IPC,
toolchain default, global PATH, TCP 24800 or UDP 5353 was used or changed.
No tests, mocks, delegation, Docker, installation, push or publication ran.

Exact private originals remain local. [copy-ledger.json](copy-ledger.json)
binds the committed traceback copy to its private original. The copy removes
only any listed private prefixes/BOM; it preserves line endings. No binary,
archive, key or token is committed. [artifacts.json](artifacts.json) binds
every committed file in this directory except itself and the later Git
commit identity.

The required fresh native Windows compile is still missing. Source review,
old runtime evidence, final runtime cleanup, strict deadline acceptance,
security/provenance and broader release gates remain separate and unchanged.
