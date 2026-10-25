# One fresh M1 bundle operation — inert source and plan

Date: 2026-10-24, America/Vancouver. Worktree: `t3-c52ad306`.
Root source: `ed7d96bf53da1a539fa2acde3b7133613221fd4b`.
**Execution is false. No controller, helper, compiler, build, signing command,
product code, or remote command was run for this proposal.**

This is a new baseline on a separate Apple Silicon test Mac. Parent reports
arm64, macOS 27.0.1 (26A434), Xcode 27.1 (27A9275), first-launch status 0,
Swift 6.4 (`swiftlang-6.4.0.34.1 clang-2100.3.34.1`), SDK 27.0, a logged-in
desktop and about 373 GiB free. Parent alone verified the direct local SSH
hostname and the physical ED25519 fingerprint
`SHA256:C/dJQyPnK0M75B8PGa6s8lJJkubk43qHNOYvqdfnx8Q`.
These are supplied facts, not observations made by this sub-agent. No hostname,
account, connection command or key is stored here.

Only Apple's Python 3.9.6 is reported on that Mac. This operation requires a
parent-supplied portable Python 3.11+ prefix. It must have a concrete executable
path and a full prefix digest. Do not substitute `/usr/bin/python3`, a guessed
Homebrew path, a rustup shim, or an unverified toolchain installation.

## Exact input bindings

`source-bindings.json` binds all **702 tracked files outside `docs/`** at
`ed7d96b`, including their bytes and Git executable modes. Parent's final
`source-workspace-r3.tar` is **5,365,760 bytes**, SHA-256
`e28b00daeaf5de62ab12f1e0165a5ba1b815d22ca9139a74b54f599e20ada277`.
Its `source-workspace-r3-receipt.json` is SHA-256
`7ec86206f0ad7a0931451e46c0e056e6dea07223cd60e4e05d4037f1b4f5c0bb`.
The 702 regular files total 4,773,386 bytes. Bind their M1 paths in the template.
The old 435-file archive is **not build-complete** and is
not an extraction input for this operation. No supplement is used in the final
recipe. The controller changes no Cargo manifest, lockfile or dependency version.

The parent build data receipt is SHA-256
`96079ccbb6a0352141b5865cd5df4dd64b212b7f0f9694222ab7d2c3c9621e54`.
It is named `build-input-receipt.json` in the reported M1 staging directory
(the original local name is `receipt.json`). This receipt still binds the
Rust/Cargo data below. Its old source entry does not qualify the new source
snapshot. The receipt and these original dependency archives remain read-only:

- `rust-toolchain.tar.gz`: 223,257,861 bytes;
  `c981c4cbc442451fc6d300f4f34540b20032a67f26bce6ec8d97454cb18ffb3c`.
  Prefix `rust/`; 225 regular files, 639,160,693 bytes. Parent retained the
  existing Rust 1.99.1 aarch64 toolchain as data. Installation is not assumed.
- `cargo-seed.tar.gz`: 22,988,930 bytes;
  `178fb8a0fcb8298faaebacff7a6cd7d671d8644af126bdda205ec3b20fbad48b`.
  Prefix `cargo-home/`; 210 files, including 106 checksum-verified registry
  crate archives and their sparse index/config data. No Cargo target or old
  compiled output is included. Cargo expands these data into the new root.

Parent's final prerequisite update also reports one fresh private Rust/Cargo
prefix extraction: all 225 Rust and 210 Cargo seed files were validated, and
owner execution was enabled only on the 20 bound Rust executable paths.
Direct `rustc --version --verbose` and `cargo --version` returned 0 with empty
stderr at an NI15 bootstrap: Rust/Cargo 1.99.1, Rust commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM 23.1.1, host
`aarch64-apple-darwin`. No global install or defaults changed. This recipe
copies the same immutable prefix data from its hash-bound archive into the
new root and checks its direct versions there. It copies no Cargo build state
from the qualified prerequisite root.

Parent's first local data-reader failures remain historical failures: Python
3.9 lacked tomllib, and a shared parent directory was mode 0755 rather than the
assumed 0700. Parent corrected data preparation with explicit Python 3.15 and
a new private 0700 child before retaining this snapshot. The shared parent's
permissions were unchanged. This proposal neither resets nor reruns those steps.

The first `source.tar` was 3,215,360 bytes, SHA-256
`17c8f514257903d1c8c6503077c62a449405cbc19e2fdfcbdb004e309853b745`,
with only 435 files. Parent's real `release.py version` qualification on its
first extracted source tree **FAILED before build**:
`release: [Errno 2] No such file or directory: '<private-source>/tools/e2e/Cargo.toml'`.
The helper reads every Cargo workspace member's manifest. The source selection
also omitted the local `[patch.crates-io]` path `vendor/mdns-sd`. Parent preserves
the original archive, first source tree, first stderr and failed CLI receipt.
No app, compiler build or signing action followed that failure. The corrected
archive is a new source-data snapshot at the same root commit, not a claim
that the failed qualification passed. This task does not run that helper.
Parent also retained source R2 as **FAILED**: normal Git archive applied a
`.cmd` CRLF attribute transform, so it failed exact raw-blob equality. R3 uses
raw Git blobs and preserves that predicate. It does not rewrite R2 evidence.
Parent independently hash-checked all 702 R3 files on the M1. Its real private
CPython `release.py version` on that source returned 0, stdout `0.1.0`, empty
stderr, in 0.157396 seconds with an NI15 bootstrap. That proves only this
prerequisite helper result. It does not qualify a build or hashlib.file_digest.

Parent's later prerequisite update selects CPython 3.13.9 arm64 from Astral's
immutable 20251014 release. Its candidate is Git commit
`83334d4235c91f4dddb55dd43eed2d1eabdc82a1`, folder
`docs/verification/m1-python-prerequisite-data-2026-10-24-t3-27c54a9f/`.
The 16,526,691-byte archive is SHA-256
`931db8f735e18700d4eab9ee39dbbd0b4c114d7d039dd2707b2d932ded039698`;
the receipt is `ea5aaf52ae45e21dc9b8bcb9b18fcfcf1a2045f459701c3213cbc0065fbdb270`,
and the inert extractor is
`bab38f8ed1bf4ab1d15843fc8ad3872c553e6665faf5eb97e432261f51f9b027`.
`source-bindings.json` binds these three staged data files by their M1 names.
The controller only hashes them; it does not run the extractor or Python
archive content. Parent's later update reports extractor source clearance at
`00f5792793b27d604689d2e827015e9e650bec6f` and one successful real M1 data
extraction, followed by the successful R3 version-helper call above. Its
`bin/python3.13` is mode0700 and SHA-256
`d81701904d7a58f6abb3ab339c1d38d48cd24a39b37645cf7338c07a1e8e70e6`.
This is parent evidence, not work run by this sub-agent. The prefix path and
complete prefix digest stay placeholders for concrete machine binding.
The controller uses streaming SHA-256, without depending on file_digest.

Parent's staging report `662812e` records seven exact prerequisite data files
on the M1, each mode0600 with one link, under a fresh 0700 directory. The
receiver and its marker are separate parent-owned staging data; this recipe
does not touch them. The first quoting failure stays preserved in its own
root. The corrected NI15 data transfer took 3.732603 seconds. It was not a
product retry or a build operation. The final R3 archive and receipt are
separately staged in a fresh parent-owned M1 root. Bind those paths without
using its already qualified source-data tree as a build workspace. This
operation extracts another fresh source copy.
No remote read or transfer was performed by this sub-agent.

The source snapshot is the selected 702 files, not a complete repository or
checkout: `docs/` and `.git` are absent. `bundle.sh` calls `release.py version`,
whose untagged metadata path needs no Git repository. **Do not replace this
operation with `release.py
package`**: that adds Git provenance, notice generation, archives and real IPC
smoke flows outside this one-bundle operation.

## Parent binding, before any execution

The parent must copy and concretely bind the recipe and input data on the M1.
`operation.template.json` leaves execution false and every machine path as a
placeholder. Bind its prerequisite directory, corrected source archive/receipt paths,
new root, Python prefix/executable,
Python prefix digest, developer directory, and one absolute UTC deadline.
The developer-directory binding must select
`/Applications/Xcode.app/Contents/Developer` for this baseline. The root must
not exist and must have a canonical, owned 0700 parent. The controller changes
no existing directory mode. Do not use or empty an old verification root.
This is concrete machine/data binding, not a new human approval condition for
the ordinary build the user already authorized. Parent owns the later review
and execution; this implementation sub-agent stays source-only.

The parent must verify the Python distribution as data **before** using it to
interpret this source. A program cannot authenticate its own interpreter
before startup. The controller then checks the complete prefix digest again,
requires Python 3.11+ with `-I -B`, and binds `sys.executable` to that prefix.
The digest format is documented in `prefix_hash`: sorted compact ASCII JSON
records with LF delimiters; it includes directory/file modes, file lengths
and SHA-256 values, and internal symlink text. It rejects escaping links,
special files, more than 100,000 records or eight GiB of regular bytes.
Retain the concrete operation JSON and its SHA-256 independently. Its recipe
hashes must still match this package. No approval or executable receipt is
created by this source-only proposal. Enabling a bound copy is parent work.

The future invocation shape is the bound Python executable with arguments
`-I -B controller.py BOUND_OPERATION.json`. It is shown as a recipe, not run
here. The controller refuses execution false before creating a root or
changing process priority. A failed or used root stays spent. There is no
retry, reset, alternate recipe or second build in this operation.

## Finite operation

1. Validate the bound recipe, time window and new root. Set and read back
   **absolute NI15** before forking the operation keeper or starting tools.
   Make the new root 0700. Require at least 32 GiB free. Save the bound plan,
   hash and converted monotonic deadline inside it.
2. In one new owned session, verify the data receipt, corrected source archive,
   Cargo seed and full Rust archive hashes. Verify the Python
   prefix. Extract exactly the receipt-bound regular records, without tar
   links, devices or overwrites. Verify all 702 source files, reject extra
   initial files, and require no `.git`, target, Swift scratch or workspace
   state in source. Build state starts empty. No old build DB, output,
   workspace-state, module cache or full scratch tree is copied. Parse Cargo
   TOML as data before any compiler or Cargo command: require every workspace
   member manifest, each release.py configuration input, and the recursive
   local path dependency/patch/replace manifest closure to exist in the bound
   source. Keep every local path inside the new source root. Include target
   dependencies and workspace dependencies. Save this explicit closure receipt.
3. Create private HOME, temp, Cargo target, Swift scratch/cache/config/security,
   module cache, XDG paths and DIST under that root. Use the copied real
   `rust/bin/cargo`, `rustc` and `rustdoc`, not rustup. Verify Rust 1.99.1,
   aarch64 host and Cargo 1.99.1. Bind Xcode per process with DEVELOPER_DIR;
   verify OS build, Xcode, first-launch status, Swift version and SDK through
   bounded metadata commands. Keep jobs 2 and deployment 15.0.
4. Run **one preparatory SwiftPM resolution**, at NI15, maximum 240 seconds:
   `swift package --package-path <root>/repo/apps/macos --scratch-path
   <root>/swift --jobs 2 --force-resolved-versions resolve`.
   The explicit `swift-entry.sh` adds only private cache/config/security
   paths and disables Keychain/netrc lookup. It keeps the normal SwiftPM
   engine and normal sandbox; it adds no build-system or isolation opt-in.
   Fresh private SwiftPM state fetches the official Sparkle source at
   `eef1a539a373c1f1a320624b1130fc5de7b2e100`, version 2.10.0, and its official
   binary data. SwiftPM must validate binary checksum
   `17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
   Then require the exact checkout revision and that checksum in its manifest.
   Verify the original source files, including Package.resolved, remain exact.
   No describe, manifest probe, build preflight or fallback is added.
5. Run **one** `/bin/bash <root>/repo/apps/macos/scripts/bundle.sh`, maximum
   900 seconds. Keep the script byte-exact. Its existing interfaces supply
   `PYTHON`, `DIST`, `SWIFT_SCRATCH_PATH`, `UNIVERSAL=0`, build number 1 and
   the standard Cargo environment. The normal source chooses arm64 from the
   verified host; Rust uses locked `aarch64-apple-darwin`. It builds the daemon
   once, builds the app once, queries `swift build --show-bin-path`, and runs
   the existing Swift icon script. The bin-path query and icon command are
   normal bundle steps, not extra app compilation attempts. Do not add a
   separate Cargo, Swift build, Clippy, test or package command.
6. The same normal bundle invocation assembles, strips and **ad-hoc signs**
   nested code and the app, verifies signatures/architectures/linkage, then
   executes only the bundled daemon's isolated **`version`** command. No
   named signing identity, timestamp service, updater, app launch, daemon
   start or IPC scenario is added. Update configuration is explicitly disabled.
   The `version` paths are generated by bundle.sh under its own private DIST
   staging directory. This is future product-code execution inherent in the
   interface, not an action performed during this source task.
7. Check all bound source bytes, the copied Rust records, original Cargo seed
   records and Python prefix again. Record bounded output file hashes. Release
   the session keeper only after the worker succeeds and the observed owned
   group contains the keeper alone. Otherwise stop. Preserve the new root,
   first failure and available logs. Normal bundle.sh may remove only its own
   transient staging directory through its existing EXIT trap.

The public dependency fetch in step 4 is intentional. Cargo is offline and
locked. Git's system/global config, credential helper, hooks and prompts are
disabled per process; SSH is disabled. There is no dependency credential,
private key or signing identity input. The recipe does not impose an OS-level
network allowlist or claim capture of every internal networking retry by
SwiftPM/Git. There is one controller invocation of resolve and one of bundle;
failure ends this operation without changing flags or fetching replacements.
Neither controller nor bundle opens a test listener. No operation targets
TCP24800, UDP5353, a live app, production config or an existing connection.

## Bounds and identity-safe recovery

The parent supplies a future **absolute UTC deadline at most 1,800 seconds
away**. The controller converts it once to the same host monotonic clock.
All input checking, extraction, resolution, building and postflight share
that deadline. Work stops 15 seconds before it: five seconds for TERM grace,
then ten seconds for observation and reporting. Each metadata command has
10–15 seconds; resolve has 240 seconds and bundle has 900 seconds, all inside
the total. No stage renews the total deadline. A per-command timeout leaves
its child alive for the common TERM/grace/KILL path; it does not use Python's
subprocess.run timeout-and-immediate-kill behavior.

The operation keeper is a directly forked session leader. It stays alive
after the worker outcome until the controller releases it. Before opening
the start gate, the controller binds PID, UID, PGID and kernel start seconds
and microseconds with macOS libproc. It samples only that owned process group
at a nominal 250 ms interval and refuses an observed priority other than 15,
an unreadable identity or a full 2,048-PID inventory. The parent requests
recovery over the unique lifeline pipe. The keeper signals its **own current
group**, retaining its identity until the last signal. It sends TERM, waits
five monotonic seconds, then sends KILL. The parent does not signal a reused
PID or an unanchored numeric process group. A separate keeper thread uses
the same deadline and pipe EOF to recover after parent loss. It keeps the
leader alive during the grace period. There is no process-name kill or
signal to another session, Xcode service, installed app or connection.

Kernel start identities also bind final checks of previously observed PIDs.
An uncertain final inventory or surviving sampled identity is a failed
recovery check. Unseen or session-escaping descendants and system Xcode/XPC
services remain **UNVERIFIED**; group observation is not universal process
containment. Samples do not prove the priority of short unsampled processes
or unrelated services. Kernel calls, filesystem I/O and scheduling can delay
observation; the deadline is an enforced stop target, not a hard real-time
guarantee. A missing terminal receipt is failure, never presumed success.

Combined command stdout/stderr is capped at 16 MiB; excess triggers stop and
retains the first prefix plus a dropped-byte count. Process samples are
capped at eight MiB. Metadata captures are limited to 16 KiB for acceptance.
The owned root has a sampled 24 GiB / 200,000-entry ceiling, normally checked
every two seconds. Each inherited regular-file output has a two GiB kernel
RLIMIT_FSIZE, and core dumps are disabled. The total root ceiling is a sampled
logical-size bound, not a hard volume quota: temporary growth, individual
metadata capture growth before inspection, tool-internal captured buffers,
and outside-root OS caches are not covered by that ceiling. Bounds can fail;
their failures remain intact. The controller never deletes the new root.
The unchanged Sparkle helper retains its own 600-second per-tool timeouts and
captured-output behavior. The five-second grace above is the controller's
operation recovery policy; it does not replace internal helper timeouts.
Compiler and stripping warnings remain verbatim in the retained output.
No warning or historical toolchain failure is declared repaired by this plan.

## OS state and historical limits

Private HOME, TMPDIR and cache paths do not redirect every Darwin Foundation,
xcrun, SwiftPM or Xcode service write. This fresh baseline permits normal
tooling behavior, including OS-managed temporary/replacement directories,
xcrun caches and normal Xcode/build-service caches. There is **no outer
filesystem-denial profile**, no global cache deletion, cache relocation,
defaults edit, xcode-select change, account change, TCC action or attempted
OS-state restoration. No universal shared-OS-state preservation or restoration
claim is made. Only the new root and its new process session are owned.
The source/seed inputs are read-only; old verification data are not cleanup
targets and are not reopened as workspaces.

This operation does not reuse or relax the old isolated run's filesystem
predicates. That run and its failure remain separate. The held source review
and causal research are prior failed evidence only:

- [Held Swift isolation evidence](../release-swift-isolation-2026-10-23-t3-bc17f311/README.md).
- [Held isolated source review](../release-swift-isolation-review-2026-10-23-t3-73b2eeaf.md).
- [Atomic-write causal research](../release-swiftpm-atomic-write-research-2026-10-23-t3-57d3a189.md).
- [Release-readiness state](../release-readiness-2026-10-24/README.md) and
  [continuation](../release-readiness-2026-10-24/continuation.json).

The OS differs from the old run. Even a successful bundle here would not prove
the output-file-map failure's cause, repair the old diagnostic, restore its
failed preservation predicates, reopen an expired update guard, or establish
native input, updater, signing trust, release or full-product acceptance.
The causal diagnostic remains stopped and its cause remains **unproved**.
Existing **FAILED / UNVERIFIED** results remain unchanged.

## Static validation for this source task

Read CONTRIBUTING.md, the current release-readiness README and continuation,
bundle.sh, priority.py, release.py and sparkle.py. Read the held isolation
plan/review and causal research only as historical evidence. Inspect source,
JSON, archive data, hashes, AST and diff. Do not import or execute this
controller, its entry script, project helpers or supplied product source.
Do not run tests, mocks, simulations, compilers, GUI automation, remote
commands, codesign or packaging. Runtime verification is intentionally
**NOT RUN** under the task's source-only limit. `static-checks.json` records
the permitted checks. `SHA256SUMS` binds this package. Commit only this folder,
with Git hooks and commit signing disabled. No push or publication is included.
