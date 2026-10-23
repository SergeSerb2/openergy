# Dependency-policy refresh — 2026-10-23

The exact checkpoint source `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`
passes the unchanged dependency policy against refreshed advisory data.
The combined all-features graph and five explicit platform graphs each report
zero advisory errors or warnings. Advisories, bans, licenses and sources all
exit zero. The notice freshness check also passes.

This uses actual cargo-deny, Cargo metadata and HTTPS responses from the official
sparse index. It does not compile or execute Openergy. The separate IPC candidate
and its remaining import-budget fix are outside this source scope.
[Exact commands, observations and hashes](dependency-policy.json).

## Freshness and integrity

The existing verified cargo-deny 0.20.2 executable was rehashed before use.
Its official release archive and executable hashes match the previous record.
The tool was not installed globally. Rust 1.99.1, nice 15 and two jobs were
selected only for these commands. No policy, dependency version, lockfile or
notice inventory was changed.

Database/index fetch and all six checks exited zero. The clean RustSec checkout
is `84d943a6deeb8cc6c6f632531aaf4f1c62275239`; a direct official Git remote HEAD
check returned the same revision after the scans. The observation is
09:43:07 America/Vancouver on October 23. The record contains UTC timestamps.

Fresh HTTPS responses covered **103 registry crate names and all 106 locked
registry package versions** in all-features Cargo metadata. No observed version
was yanked. Every returned checksum matches Cargo.lock. The JSON retains request
URLs, HTTP freshness headers, response hashes and every version result. This
compares registry metadata; it is not a new download and rehash of every archive.

The local patched `mdns-sd` is a path package. It is excluded from the registry
count and retains separate archive/patch/source provenance. Earlier registry
counts included its registry form. Graph counts are also a different inventory.

All 801 files in the immutable source snapshot kept their hashes. Cargo manifests,
lockfile, policy, vendor provenance and notices are separately identified in the
record. No Openergy source or production state was written by these commands.

## Graph results

| Graph | Packages | Advisory findings | Duplicate warnings | Unused license allowances |
| --- | ---: | ---: | ---: | ---: |
| Combined | 93 | 0 | 2 | 3 |
| aarch64-apple-darwin | 73 | 0 | 1 | 4 |
| x86_64-apple-darwin | 73 | 0 | 1 | 4 |
| x86_64-unknown-linux-gnu | 79 | 0 | 1 | 3 |
| x86_64-pc-windows-msvc | 75 | 0 | 1 | 4 |
| aarch64-unknown-linux-gnu | 79 | 0 | 1 | 3 |

Combined duplicate warnings concern `windows-sys` 0.52.0/0.61.2 and
`winnow` 0.7.15/1.0.4. Filtered graphs report only `winnow`. Unused allowances
are BSD-2-Clause, CC0-1.0 and Zlib; the macOS and Windows graphs also report
Apache-2.0 WITH LLVM-exception. These warnings remain visible. No exception or
policy relaxation was added. The [combined output](dependency-policy.ndjson)
changes only local path prefixes in the copied information log.

## Helper correction and limits

The first reporting helper failed after all policy, metadata and notice commands
had passed. It did not normalize Cargo metadata's `source:null` for local/path
packages before calling `startswith`. Its exit-1 result and original helper hash
are preserved. The corrected registry-only helper reused the completed scans;
it did not rerun or weaken them. All fresh registry checks then passed.

Target graphs are policy filters, not native compilation or runtime acceptance.
These observations do not approve the vendored code or establish vulnerability
absence. Whole-cache/expiry/timer/query/variable-byte limits, deterministic slow
DNS, native input/clipboard, positive Windows migration, original independent
security/provenance acceptance and trusted distribution remain open.
Provider-stopped expiry and original provenance review were not rerouted.
GitHub private reporting remains deferred at the user's request.
