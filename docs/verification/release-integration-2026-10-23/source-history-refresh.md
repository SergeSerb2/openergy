# Source-history secret scan refresh — 2026-10-23

Gitleaks **8.30.1** scanned the history selected by exact source commit
`ca618418dc8c4d93b8fd04eeda3fefa01003b8e7`. It exited **1** and reported
**84 `generic-api-key` findings**. The raw result is nonzero. It is not a
zero-finding or passing raw scan.

All 84 reported values were verified as source-file SHA-256 records. No
credential was identified among these findings. No finding remains unresolved
within the stated checks. The [machine record](source-history-refresh.json)
contains the policy, commands, raw artifact hashes and limits. The
[sanitized evidence](source-history-findings.json) records each location,
evidence blob, explicitly recorded source revision, source blob and classification.

## Scope and comparison

The checkout was clean at the exact source commit before and after the primary
scan. Git lists **231 reachable commits**, 15 more than the prior source
`54aa50b9eafc51f6eef2589c100c509eaac7e415`. Gitleaks reports **220 commits
scanned**. The graph contains 11 merge commits. A private command-trace replay
confirmed the underlying history command:

```sh
git log -p -U0 ca618418dc8c4d93b8fd04eeda3fefa01003b8e7
```

The command contains no `--all`. It does not request separate merge diffs.
Thus, this is patch-based detector evidence. It is not a scan of every commit
snapshot or proof that merge-only resolution content was checked. The prior
scan selected 216 reachable commits and reported 205 scanned commits, with the
same 11 merges. Its raw exit was 1 with 64 findings. Its retained private report
was rehashed and matched the earlier published digest.

All 64 prior finding locations remain. There are **20 new findings**:
seven in `discovery-bounded-integration.json`, six in
`dns-cancellation-and-readiness.json`, six in `discovery-owned-shutdown.json`,
and one in `vendor/mdns-sd/UPSTREAM.json`. There are no missing prior locations.

## Candidate checks

Each candidate came from its exact evidence Git blob and reported line. The
review reconstructed the redacted scanner match, bound it to that line, and
required a unique JSON leaf with a 64-character lowercase hexadecimal value.
It then hashed the Rust source blob at the revision explicitly named in that
evidence. All source commits read were reachable from the pinned source commit.
A match at the evidence commit alone did not control the classification.

For example, the six copied records in evidence commit `986063a` were checked
at their recorded source `f0f2ee5b7701add5a6fcdf040b563a072a905865`.
The seven records in evidence commit `e4f48ba` were checked at
`dddf9fef012f9d56ea6435be36bd5c74a08087d7`. All 83 verification-evidence
findings matched their recorded tracked-source hashes.

The remaining finding is `original_file_sha256["src/fuzz_api.rs"]` in vendor
metadata at `efca42e`. It matched the vendored Git blob and the same member of
the existing cached `mdns-sd-0.21.6.crate` archive. The archive SHA-256 is
`8562654710f72a9cbce4b0d5b7c366a2f94330ba1ce3740f15e932ea2d37b161`.
Its `.cargo_vcs_info.json` names the recorded upstream revision
`1a5cdb6ebb7299ee74dda75099461a2921b6e740`. The archive member SHA-256 is
`6d16f18f4e5b59b9194da86f0562b1ddb5cdc1151f43d80fe06bcde65e4b19a4`.
The archive was read without extraction, network access or code execution.
The seven selected source paths and vendor crate identity were also rechecked
at the requested parent. These checks support hash classification only.

## Tool, policy and reproduction

The retained official-release Gitleaks archive and actual executable were
rehashed locally before scanning. Archive SHA-256:
`b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
Executable SHA-256:
`ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`.
The version command returned `8.30.1`. No tool was installed or updated.

The scan retained the earlier built-in default detector policy, including
built-in allowlists. No project-root `.gitleaks.toml` or Gitleaks configuration
environment override was present. There was no rule filter, custom config,
baseline, new allowlist, suppression or history rewrite. The dedicated ignore
file was empty, and inline `gitleaks:allow` comments were ignored.

From the repository root, use the verified executable as `GITLEAKS` and a
mode-0700 directory outside the repository as `PRIVATE_DIR`. The executable,
checkout and private paths are the only path substitutions in this command.
Use private output files and preserve the returned status:

```sh
umask 077
: > "$PRIVATE_DIR/empty.gitleaksignore"
env -u GITLEAKS_CONFIG -u GITLEAKS_CONFIG_TOML \
  "$GITLEAKS" git . \
  --log-opts=ca618418dc8c4d93b8fd04eeda3fefa01003b8e7 \
  --redact=100 --ignore-gitleaks-allow \
  --gitleaks-ignore-path "$PRIVATE_DIR/empty.gitleaksignore" \
  --max-archive-depth 2 --max-decode-depth 5 --timeout 120 \
  --no-banner --no-color --report-format json \
  --report-path "$PRIVATE_DIR/findings.json" \
  > "$PRIVATE_DIR/history.stdout" 2> "$PRIVATE_DIR/history.stderr"
scan_status=$?
```

The primary raw JSON report SHA-256 is
`f32505a06c28f8ea409ec4ee064dc3ad86e961baaf67471cc3833903834f36c7`.
Raw reports, output, trace and local helper files remain private. The trace
replay had the same 84 finding locations. Its return code was not saved before
a local trace-parser assertion, so it supplies scope evidence only. The
primary scan's captured exit code 1 remains the authoritative raw status.

## Limits and unchanged gates

This evidence covers only the stated detector, policy and selected patch
history. It does not prove the repository free of secrets or vulnerabilities.
It excludes uncommitted files, unreachable objects, other refs, external
systems, installed state, later commits and the new report commit itself.
No code from history was run. No product source changed. No source/build,
TypeScript, unit, platform, latency or runtime acceptance checks were required
or run for these documentation additions. Both JSON files parsed, all 84 raw
location identities matched the sanitized evidence, its recorded SHA-256 matched,
and `git diff --cached --check` passed. The staged changes contain only the three
new source-history documents.

Stopped sibling IPC work, the older expiry/NSEC/orphan-record experiment and
the original release-provenance security review were not resumed or replaced.
Native input/clipboard, positive Windows migration, original security/provenance
acceptance and trusted distribution remain open. Private GitHub reporting
remains deferred. No publication, push, merge or credential change was made.
