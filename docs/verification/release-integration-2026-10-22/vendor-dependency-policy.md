# Vendored discovery dependency-policy check — 2026-10-22

The exact discovery candidate `da668c3aac3fc2c74354d136733d4db8464a6144`
passed cargo-deny after a fresh database/index fetch. The combined graph and
five platform graphs report zero errors in advisories, bans, licenses and
sources. Advisory warnings are also zero. These are dependency-policy checks.
They do not establish platform build or runtime acceptance.

The immutable private snapshot contained 741 files. All hashes were unchanged
at the end. The integrated source `dddf9fef012f9d56ea6435be36bd5c74a08087d7`
has identical policy, Cargo and modified vendor inputs. The companion
[JSON record](vendor-dependency-policy.json) contains exact commands, timestamps,
source/tool hashes, raw-output hashes and all warning details.

## Results

| Graph | Packages | Advisory warnings | Duplicate warnings | Unused license allowances |
| --- | ---: | ---: | ---: | ---: |
| aarch64-apple-darwin | 73 | 0 | 1 | 4 |
| aarch64-unknown-linux-gnu | 79 | 0 | 1 | 3 |
| combined | 93 | 0 | 2 | 3 |
| x86_64-apple-darwin | 73 | 0 | 1 | 4 |
| x86_64-pc-windows-msvc | 75 | 0 | 1 | 4 |
| x86_64-unknown-linux-gnu | 79 | 0 | 1 | 3 |

The combined duplicate warnings concern `windows-sys` 0.52.0/0.61.2 and
`winnow` 0.7.15/1.0.4. Each filtered graph reports only `winnow`. Unused
allowances are BSD-2-Clause, CC0-1.0 and Zlib; macOS and Windows graphs also
report the unused Apache-2.0 WITH LLVM-exception allowance. No policy was
weakened and no dependency version or lockfile changed during the check.

## Inputs and freshness

The pinned tool is cargo-deny 0.20.2. Its executable hash was checked against
the previously downloaded official release artifact. Commands used the pinned
Rust 1.99.1 environment, nice 15 and two jobs. The database/index fetch and
all six scans exited zero at 21:46:46–21:46:50 America/Vancouver on October 22.

The RustSec checkout is clean at
`640bf5a9f06e66d8721e8772766531fe77c8630d`. A direct official Git remote HEAD
check after the scans returned the same revision. The checked-in database
commit is dated `2026-10-22T12:49:47+02:00`. Raw output remains in the private
policy evidence directory recorded in the parent handoff.

The vendor archive checksum, complete inventory, unchanged licenses and existing
tests, exact local patch and candidate source were independently reviewed.
The integration check also reruns the source verifier and notice generator.
That source evidence is separate from the advisory database result.

## Remaining limits

Target-filtered graphs do not compile or run Windows, Linux or macOS binaries.
Cargo-deny's index fetch and cached-index checks do not replace a fresh direct
HTTPS observation of every registry version. This step collected no new full
registry yank/checksum inventory. The vendored patch needs its own source and
runtime evidence; a clean advisory result does not approve its implementation.

Whole-cache, expiry, timer, query-work and variable-record-byte bounds remain
open. Native input/clipboard, positive Windows migration, original independent
security/provenance acceptance and trusted signed/notarized distribution remain
release gates. Private reporting setup remains deferred at the user's request.
