# Isolated native package attempt: FAILED

Date: 2026-10-23, America/Vancouver. Owned worktree: `t3-8c51bae6`.
Start/build HEAD: `b713d7fbe480a1a3ecadd737ae5de58256169c66`.
Production priority source remains exactly
`d3b30e1719dae5c885ded57cc7523b92d9686adc`; changes between these commits are
verification evidence only. The integrated independent source/saved-evidence
review was read. It is separate from this new runtime attempt.

**Exactly one new real `release.py package` invocation ran. It exited 1 in
47.336825 seconds, without timeout. No retry ran. No package artifact is
accepted.** The daemon compiled from an empty Cargo target directory. Swift
manifest evaluation then failed under the enclosing filesystem/network sandbox:

```text
swift: error: couldn't create cache file '<Darwin temporary directory>/xcrun_db-xFzIFMiR' (errno=Operation not permitted)
sandbox-exec: sandbox_apply: Operation not permitted
```

The exact first command output and paths are in `raw/package.stderr.gz` and
`raw/package.stdout.gz`. The enclosing policy denied the attempted write outside
the private root. Swift's own nested sandbox also failed. The setup preflight
had validated policy parsing and private Unix IPC, but did not evaluate a real
Swift manifest under that policy. This was an insufficient preflight. The
package guard remains intact; the command was not repeated or repaired after
compilation started. Nonfatal Rust linker/`rust-objcopy` diagnostics are retained
as well. No toolchain setting was changed to address them.

Whole-app compilation, bundle assembly, nested signatures, archive creation,
ZIP roundtrip, and both staged/extracted daemon smoke flows were **not reached**.
`raw/full-smoke.json.gz` records zero product smoke completions. There is no new
archive, BUILD sidecar, or archive/source binding. The compiled arm64 daemon is
hash-bound in `raw/failure-limits.json.gz`; it was not executed. This attempt
does not supply release clearance or independent IPC reproduction.

## Cache isolation and preservation

The new [controller](raw/controller.py) copied only the pinned Sparkle
`artifacts/sparkle/Sparkle`, `checkouts/Sparkle`, and
`repositories/Sparkle-09d89c53` trees. It read the compiled-only R3 bootstrap
controller as source data; it did not execute or import it or the failed old
package controller. It copied no old Swift `out`, `.buildSystem*`, module cache,
native build database, or Cargo output. Cargo home contains a registry copy
only; no credentials, global config, or user binaries were copied.

HOME, CFFIXED_USER_HOME, temporary/module/cache/config paths, Cargo target, and
Swift scratch/build state were fresh and private. The controller generated new
workspace state and a private artifact mapping, checked the committed
`Package.resolved` pin, and redirected the private checkout's Git origin and
object alternates to the private repository copy. It validated paths before the
package command. The immutable historical artifact/checkouts/repositories and
workspace-state read inputs are declared in `raw/scope.json.gz`. The sandbox
policy is retained in [package.sb](raw/package.sb).

Both complete original byte/link inventories match before and after: **1,309
Swift entries and 1,159 Cargo entries**. They also match the retained prior
after-recovery inventory. The new selected metadata baseline covers 1,725 Swift
and 1,378 Cargo file/link/directory objects, including each root. Modes, kind,
owner/group, link counts, flags, modification times, and symlink targets match.
This is equality against this attempt's baseline. It does **not** establish
full metadata equality with the old prebreach state, or cover ACLs, extended
attributes, access times, or change times.

Private dependency source/artifact checks use pinned Git revision, source
status, and payload hashes. Private copied `.git/index` byte churn is explicitly
permitted and is not a dependency integrity condition. No private seed
difference was observed in this failed attempt.

All **714 tracked source inputs** match their new before/after hashes. The
production commit binding also checks their Git blobs. One Windows `.cmd` has
the repository's required LF-to-CRLF checkout conversion; both hashes and the
declared conversion are retained. The priority implementation files match the
production commit byte for byte. No production source changed.

All **25 prior gzip records and private originals** match their original ledger
hashes before and after. The complete old evidence folder also matches. No old
guard was reset, old cache was written/restored, or old record was changed.
The [original overall FAILED result](../release-priority-2026-10-23/README.md),
its 114-removal preservation breach, and its metadata limit remain unchanged.

## Priority, cleanup, and limits

The controller set and verified absolute NI15 before starting the real command.
Across **621 samples, 3,330 observations, and 100 observed owned PIDs**, all
observed nice values were 15. Records include actual Rust compilers, clang/cc,
and an `ld` linker, with PID, microsecond start identity, parent, and process
group attribution. No Swift frontend, codesign, or packaged daemon was observed.
Samples cover 0.060517–47.301428 seconds; the largest gap is 0.104765 seconds.
Short processes and unsampled intervals remain unproven.

Cleanup checked only attributed PID/start/group identities. No signal was
needed; no sampled owned identity remained. The cleanup check took 0.000387
seconds, within its 60-second budget. The watchdog was 900 seconds. The owned
package temporary directory is absent. The first exclusive loopback binds of
65420–65429 passed both before and after the command, without reuse options or
retry. The saved state was captured before each first bind attempt. `lsof`
warned that an unrelated mounted filesystem could not be inspected; its output
was incomplete. The exclusive bind results supply the port check. These checks
do not prove complete host-state or universal descendant cleanup.

The environment explicitly selected Rust 1.99.1, jobs 2, offline Cargo, deployment
target 15.0, native arm64, and required production bundled feed/key defaults.
Signing-intent variables were unset; the package's ordinary ad hoc policy was
reported before the failure. Signing never ran. No GUI/input/clipboard,
installed app, target update/relaunch, installer, signing key/keychain,
notarization, timestamp service, internet fixture, production IPC/config/TLS,
live sharing, TCP24800/UDP5353, DNS/service/global setting, Docker, push, or
publication action was performed. The three provider-stopped tasks were not
resumed. No unit tests, mocks, computer use, or delegation were used.

## Retained records

[results.json](results.json) records the failed attempt and preservation checks.
[copy-ledger.json](copy-ledger.json) binds **64 exports** to retained private
originals under `/private/tmp/oe-priority-isolated-t3-8c51bae6-nfye7e16`.
Gzip copies use mtime 0. All copies preserve exact original bytes except the two
port records, whose exports redact an unrelated private mount path from the
`lsof` warning. Their unmodified private originals remain retained and
hash-bound; the transformation is explicit in the ledger. No source evidence
was substituted or failure output overwritten.

Two precompiler setup failures are retained with the then-current controllers:
the read-only private Git alternates file and the Seatbelt `localhost` syntax
requirement. Both were repaired before the package command. A later read-only
source-binding audit first assumed Git LF bytes equaled CRLF checkout bytes;
that error, its first collector, and its exact limited correction are retained.
The controller's postflight success assertion also failed after it recorded the
package failure and preservation results; `raw/first-error-run.txt.gz` retains it.

The controller, observation hook, policy, and preflight hashes in
`raw/execution-bindings.json.gz` identify what actually ran. The observation hook
used `sys.setprofile` to record real call/return results without replacing any
function or result. The controller is frozen evidence, **not a replay recipe**.
A separately authorized attempt must first resolve and validate the private
Xcode-cache and nested SwiftPM sandbox constraints. No proposed repair is
validated by this run.
