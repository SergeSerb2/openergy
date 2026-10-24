# Independent source and saved-evidence review

Date: 2026-10-23, America/Vancouver. Review worktree:
`t3/review-release-priority` (`t3-ab89755e`).

Source reviewed: `d3b30e1719dae5c885ded57cc7523b92d9686adc`.
Evidence reviewed: `d2ab17d4e053437d9f7568b0f6dbf96727bfd6c9`, especially
`docs/verification/release-priority-2026-10-23`.

**Overall verification: FAILED.** No new actionable P1/P2 source defect was
found in the five-file priority repair. The known cache-preservation breach
remains an actionable P2 verification failure. Recovery does not close or erase
that failure. This review audits source and saved evidence; it is not an
independent runtime reproduction or release clearance.

## Retained P2: copied build state crossed the old-cache boundary

The frozen `raw/controller.py.gz` copies the complete old `swift` and `cargo`
trees with `/bin/cp -cR`, then passes the new Swift tree as
`SWIFT_SCRATCH_PATH`. The copied build database retains absolute paths into
the old scratch tree. `raw/package.stderr.gz` records stale-file warnings for
those old paths outside the new allowed roots. The first
`raw/postflight.json.gz` still reports `cached_inputs_unchanged: false`.

I recomputed the differences from the saved inventories. Exactly 114 old
Swift file/link entries were removed, all under `out/Products/Release` in the
Sparkle framework and helper dSYM outputs. There were no added or changed
Swift entries in this inventory. The old Cargo inventory had no differences.
These results exactly match `raw/cache-differences.json.gz`.

The affected boundary is the verification controller's old-cache read-only
contract, not a newly changed production cache policy in the priority repair.
A copied scratch directory is not sufficient isolation when build state holds
absolute output paths. Before another authorized package run, use fresh build
state and seed only the required pinned dependency artifacts/checkouts. Do not
replay this controller. No new run was made for this review.

The failure and its limits are explicit in [README.md](README.md) and
[results.json](results.json). Neither command exit 0 nor later restoration
turns this run into a preservation pass.

## Complete source-path assessment

The source commit changes exactly `tools/release/priority.py`,
`tools/release/release.py`, `tools/release/sparkle.py`,
`apps/macos/scripts/bundle.sh`, and `docs/releasing.md`. Those files are
unchanged between the reviewed source and evidence commits.

- `priority.py:7-17` requires the `-- COMMAND` interface. It reads this
  process's effective nice value, sets absolute 15 when needed, reads it again,
  and reaches `execvp` only after the second read returns 15. A caller already
  at 15 takes the same verification path without another relative increment.
  A permission or executable-start error produces a nonzero exit. A failed
  priority operation cannot fall through to the payload.
- `execvp` receives the original argument vector. It searches the inherited
  `PATH` for a command without a slash and uses the given path otherwise.
  There is no shell evaluation or command-string interpolation. Spaces and
  metacharacters remain argument bytes. Environment, working directory and
  standard streams pass through. The successful exec replaces the launcher
  in the same PID, so daemon wait/terminate/kill operations still address the
  directly started process. Payload exit status and signals reach the caller
  without a persistent Python wrapper. This does not prove descendant cleanup.
- `release.py:40-48` wraps every `run` payload only on Unix, using the current
  Python interpreter and the checkout's launcher. The existing `cwd`, `env`,
  capture, binary/text, timeout and `check` arguments remain intact.
  `check=True` failures reach the main CLI's existing exception handler;
  `check=False` callers still inspect the returned status. Smoke first requires
  a successful version read. Its deliberate exit-1 cases cannot hide a general
  inability to start the launcher. The separate daemon `Popen` at lines
  167-172 uses the same launcher; readiness and clean-stop checks still detect
  early failure. The owned-process `finally` block is unchanged.
- `sparkle.py:31-39` uses the same interpreter/launcher for each external
  command. Nonzero status becomes `ValueError`, then the CLI exits nonzero.
  The sensitive-command branch still suppresses tool output. No key value is
  added to launcher arguments or diagnostics by this change. Existing timeout
  handling and signing-command arguments are unchanged.
- `bundle.sh:13` quotes the interpreter and launcher paths and forwards
  `"$@"`. Its Cargo build loop, Swift build, Swift bin-path query, icon script
  and staged daemon version read now use `owned`. Both native architectures
  and both iterations of the universal branch therefore use absolute 15.
  The bin-path and version assignments preserve command-substitution failure
  status; `set -euo pipefail` stops the script. Temporary environment assignments
  on the version call reach the function's child, preserving private config,
  IPC, TLS and discovery restrictions. Signing-intent validation still occurs
  before compilation or destination creation.
- `release.py` retains both `os.name != "nt"` guards. Windows starts its
  payloads directly, as before, and its package path calls neither the macOS
  bundle nor Sparkle helper. `sparkle.py` remains a macOS tool; this review
  does not claim new Windows support for it. Linux keeps its existing build,
  smoke and tar paths with the Unix launcher applied.

This is a policy for the wrapped payloads. It does not renice the caller shell
or Python process. Direct bundle filesystem/metadata utilities retain their
existing invocation paths. Under the saved package run they inherit the
controller's 15. The source does not establish an invariant for programs that
later change their own priority, or for processes missed by sampling.

The diff leaves signing intent, named-identity opt-in, purpose/timestamp
selection, inside-out signing, entitlement preservation, sensitive diagnostics,
feed/public-key checks, archive/extraction/provenance logic, and production
application/daemon source unchanged. The package path still explicitly selects
ad-hoc signing and native architecture. These are source conclusions, not fresh
signing, Windows, universal-build, GUI or production-runtime acceptance.

## Evidence integrity and bounded observations

I used read-only standard-library inspection and local Git reads. No project
CLI, launcher, compiler, signing command, app or daemon was executed. No socket,
port, network, credential, desktop or system-setting operation was performed.
No unit tests, mocks, source edits or further delegation were used.

All 25 ledger paths are unique and cover all 25 files in `raw/`. Each compressed
length and SHA-256 matches `copy-ledger.json`; each gzip header has zero mtime.
Each decompressed length and SHA-256 also matches. All 25 private originals
remain present and byte-identical to their decompressed copies. The package,
first postflight, additional postflight, recovery and artifact sections in
`results.json` match their raw records. The package stdout/stderr hashes match
the package-result record. I left all existing evidence unchanged.

All 704 saved source-input hashes match the named source commit. The controller
excludes `docs/verification/` from this list, so this is a bounded source-input
check, not a whole-filesystem or full-metadata invariant. Its saved postflight
also reports those inputs unchanged after packaging.

The saved package result is exit 0, 138.112162 seconds, without timeout. Its
stdout contains two successful real daemon smoke completions. Source inspection
maps these to staging and archive extraction: version, absent-daemon error,
off-role virtual IPC status, invalid-method error and clean stop. It also shows
the final extracted-app signature check. Those are audited prior results; this
review did not reproduce IPC or cryptographic signature verification.

The plan records Rust 1.99.1, jobs 2, deployment target 15.0, private build/home
and module-cache paths, offline Cargo and ad-hoc signing. The selected smoke
range is 65310-65319, with discovery disabled and production port use refused
by the smoke source. This supports the stated isolated fixture design. It does
not establish a complete host-state or network audit.

Recounting the process record gives 716 samples, 205 distinct owned PIDs and
3,763 process observations, all at NI 15. Command observation counts match the
additional postflight. Samples run from 0.062337 to 137.996464 seconds; the
controller itself first sets and verifies 15. This is evidence for nested
launches from NI 15. Short-lived children and intervals between observations
remain unsampled. Saved checks report no sampled PID still present, absent
package staging, and the first exclusive loopback rebind of port 65310. They do
not prove that every descendant exited or that all possible host state recovered.

The saved direct-bundle failure exits 1 with the ad-hoc/distribution-purpose
conflict, empty stdout and an absent requested destination. Source places that
failure before compilation/signing. The saved NI 20 launcher attempt exits 1
with `Permission denied` and empty stdout. Its command asks to execute
`/usr/bin/true`; the reviewed source excludes that exec after the priority error.
This is a source-supported payload exclusion, not a separately recorded exec
trace. No new NI 20 process was started here. The saved `release.py check`
exit 0 verifies workspace/toolchain declarations, not a fresh compiler run.

All three private output artifacts still match their recorded sizes and
SHA-256 values. Reading the ZIP without extraction confirms 112 entries, valid
CRC data, an embedded manifest equal to the sidecar, and an exact checksum
sidecar. The ZIP remains 4,668,089 bytes with SHA-256
`88077210e4333bcd2e9051695305f746d95a19a907a8e34d43ae534c2965ec76`.
The manifest names the exact source, clean recorded checkout, arm64, build 3,
ad-hoc signing and `notarized: false`. Its Info.plist retains the source's public
feed/key pair, enabled automatic checks and macOS 15.0 minimum. No private key
was read. Artifact identity and saved signature success do not prove publisher
trust, Gatekeeper acceptance, GUI behavior, updater relaunch, native input or
production runtime.

## Recovery corroboration and remaining limits

The saved pre-run and copied inventories are equal: 1,309 Swift entries and
1,159 Cargo entries. After the run, Swift has 1,195 entries and Cargo still has
1,159. The candidate-match and restored-entry lists each contain exactly the
same 114 unique paths as the removal set, with no missing candidate recorded.

Read-only checks of the retained private clone confirm exact pre-run bytes or
link targets for all 114 recovery candidates. Checks of the old scratch tree
confirm the same for all 114 restored entries. I also recomputed the complete
current old Swift/Cargo inventories: both equal the saved pre-run inventories
and the saved after-recovery record. This corroborates the bounded recovered
outcome. I did not perform or repeat any restoration. The retained records do
not include a per-operation restoration trace; they support the candidate/path
sets and final byte/link equality, not every historical filesystem operation.

The inventory schema records regular-file size/hash and symlink target. It
does not record modes, owner/group, ACLs, extended attributes, flags, directory
contents as metadata objects, or original times. Full metadata equality remains
**UNVERIFIED**. The original failed inventory and diagnostics remain intact.
No package retry was made. **Overall verification remains FAILED.**

The three provider-stopped tasks remain out of scope and were not resumed:
crafted mDNS expiry/NSEC/orphan work, the original release-provenance/security
review, and IPC/Synergy 3 import-budget work. This commit owns only this new
review document.
