# Source-history secret scan refresh — 2026-10-22

The real Gitleaks command scanned all **216 reachable commits** through
`54aa50b9eafc51f6eef2589c100c509eaac7e415`. It exited **1** and reported
**64 `generic-api-key` findings**. This is not a zero-finding or passing raw scan.

Every reported value is a recorded SHA-256 of tracked Rust source. For each
finding, the review read its exact line from the historical verification JSON,
required a Rust path and a 64-character lowercase hexadecimal digest, then
hashed that Rust Git blob at the same commit. All 64 digests matched. No
credential was identified among these findings. No finding remains unclassified.

The [machine record](source-history-refresh.json) includes all 64 source paths,
evidence commits, line numbers and verified digests. Their repeated appearances
come from 11 verification records: five source files occur 11 times each, and
the earlier macOS input spike's key map occurs nine times. The scanner matched
the word `key` in the source-path fields beside high-entropy digests.

## Tool and policy

Gitleaks **8.30.1** ran from a private temporary directory. Its official
[release archive](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1)
matched the upstream asset digest and checksum file. Archive SHA-256:
`b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
The executed binary's hash is in the machine record. No global installation
or repository dependency changed.

The scan used the tool's built-in default rules and built-in allowlists.
It ignored inline `gitleaks:allow` comments and used a dedicated empty ignore
file. No project Gitleaks configuration was found. No configuration environment
override was present. No custom allowlist, baseline, suppression or history
rewrite was added to obtain the result. The original redacted report remains
local; its SHA-256 is recorded for integrity.

Reproduce from the repository root with that tool version. `PRIVATE_DIR` must
refer to a private directory outside the repository:

```sh
: > "$PRIVATE_DIR/empty.gitleaksignore"
gitleaks git . --log-opts=54aa50b9eafc51f6eef2589c100c509eaac7e415 \
  --redact=100 --ignore-gitleaks-allow \
  --gitleaks-ignore-path "$PRIVATE_DIR/empty.gitleaksignore" \
  --max-archive-depth 2 --max-decode-depth 5 --timeout 120 \
  --no-banner --no-color --report-format json \
  --report-path "$PRIVATE_DIR/findings.json"
```

Keep exit status 1 visible. Verify each reported digest against its historical
source blob before classifying it. Do not treat a matching digest pattern alone
as evidence that a finding is harmless.

## Scope and remaining gates

This read-only scan covers the named reachable history. It does not cover
uncommitted files, unreachable objects, external systems, installed state or
subsequent candidate commits. It does not prove the project free of secrets or
vulnerabilities. The worktree was clean after the scan.

Native input/clipboard acceptance, positive Windows migration, the original
independent security-review acceptance and trusted signed/notarized distribution
remain open. GitHub private reporting remains deferred by the maintainer.
