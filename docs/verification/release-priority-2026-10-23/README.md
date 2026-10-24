# Absolute release-command priority and real packaging

Source: `d3b30e1719dae5c885ded57cc7523b92d9686adc`. One actual native
`release.py package` invocation exited 0 in **138.112 seconds**. It built the
arm64 app and daemon, embedded Sparkle, signed ad hoc, checked the staged daemon
through real virtual IPC, extracted the ZIP, repeated those IPC checks, and
verified extracted signatures. The overall verification remains **FAILED**:
the copied Swift build database removed 114 entries from the old scratch cache.
Recovery is recorded below. This archive is not accepted as a clean release run.

The source repair replaces nested relative `nice -n 15` calls with one small
Unix command launcher. It sets and reads absolute nice 15 before `execvp`.
Already-correct callers retain 15. A caller that cannot set 15 refuses before
executing the command. `release.py`, `sparkle.py` and the bundle's compiler,
icon and version commands use this path. Windows retains its existing branch.
Signing intent, inside-out signing, feed/key checks and archive contracts are
unchanged. The ordinary GUI app never launched.

Across 716 process samples, all **205 observed owned process IDs** had nice 15.
Samples include actual Rust compilers, Swift frontends, linkers, a codesign
process and packaged daemon commands. Short processes can escape sampling;
this does not establish the priority of every process ever created. The
controller itself sets absolute 15, so this run exercises nested calls from a
caller already at the required priority. No unrelated priority changed.

Separate real command checks passed:

- `release.py check`: exit 0, workspace/compiler metadata consistent.
- Direct bundle with ad-hoc identity and distribution purpose: exit 1 before
  its requested destination exists. No compiler or signing command starts.
- Launcher called from nice 20: exit 1, `Permission denied` when setting 15.
  The requested `/usr/bin/true` payload does not execute on this source path.

The package preserves the normal production update configuration. It uses
fresh private output, copied Cargo/Swift inputs, private Cargo registry/home,
HOME/CFFIXED_USER_HOME, temporary and module-cache paths, offline Cargo,
Rust 1.99.1, jobs 2 and macOS deployment target 15. The SDK is 27.0; Swift is
6.4. The two package smoke flows use private off-role virtual daemons and real
Unix IPC, including absent-daemon and invalid-method errors, status and clean
stop. No native input, clipboard, GUI, installer, relaunch, Developer ID key,
timestamp service, notarization, publication or Cloudflare operation ran.
The three provider-stopped tasks remain untouched.

All 704 tracked source-input hashes were unchanged after the command. The
embedded `BUILD.json` equals its sidecar and reports this exact clean source
commit, arm64, build 3, ad-hoc signing and `notarized: false`. The ZIP has 112
entries and is 4,668,089 bytes, SHA-256
`88077210e4333bcd2e9051695305f746d95a19a907a8e34d43ae534c2965ec76`.
The archive and three artifact records stay private under
`/private/tmp/oe-priority-package-sr3z_6fy/output/`.
The package staging directory is absent, no sampled owned process ID remains,
and the first exclusive loopback bind of assigned port 65310 passes. These
checks do not supply a universal descendant or host-state cleanup proof.

## Preserved cache failure and bounded recovery

The verification controller cloned the whole earlier Swift scratch directory.
That copied build state still contained absolute paths under
`oe-sparkle-combined-20261023-a_es9dei/swift/out/Products/Release`.
Swift's saved warnings identify stale files outside its new allowed roots.
After packaging, **114 prior file/link entries were absent**: the old output
Sparkle.framework and helper dSYM contents. Cargo's old inventory was unchanged.
The original pinned dependency artifact, checkouts and source were unaffected
by the measured differences. This violated the old-cache read-only boundary.

The first `postflight.json`, cache-after inventory, differences and stderr
remain unchanged and record this failure. Packaging was not repeated. Each
missing entry had an exact retained match in the private clone. Recovery
verified that match first, refused existing destinations, restored only those
114 entries, then recomputed the complete old Swift/Cargo file-byte and
symlink inventories. They match the pre-run inventories after recovery.
Directory times and complete original file metadata were not recorded, so
full metadata restoration is **UNVERIFIED**. Restoring bytes does not turn the
first preservation failure into a pass.

The frozen controller is evidence, **not a safe replay recipe**. Future builds
must seed only pinned immutable dependency artifacts/checkouts and use fresh
build state. Copying build databases or output directories can retain absolute
write targets. No additional package command was run here.

Two earlier read-only discovery errors are also retained in the conversation:
the first controller lookup requested a nonexistent `raw/setup.py`; an earlier
repo search returned too much output. Neither started or repeated a build.

## Evidence

[results.json](results.json) separates command success, failed preservation,
recovery, process observations and artifact identities. [copy-ledger.json](copy-ledger.json)
binds 25 private originals to deterministic gzip copies in `raw/`. Decompression
returns their exact bytes. It includes the executed controller, source inputs,
plan, first command output, process samples, all three cache inventories,
recovery, error cases and postflight. No private key, token or credential was
read or copied. The source change has no unit tests or mocks; verification uses
real commands and the packaged daemon's caller interface. Native/trusted
release gates and independent source/evidence review remain separate.
