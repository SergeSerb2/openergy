# Source-history merge-diff supplement — 2026-10-23

This supplement closes the explicit merge-parent-diff coverage gap for exact
source `ca618418dc8c4d93b8fd04eeda3fefa01003b8e7`. The merge-only Gitleaks **8.30.1**
invocation returned **exit 0 with 0 findings**. It reported **11 commits scanned**.
Its raw exit was saved before any log or report parsing.

The primary result remains **exit 1 with 84 `generic-api-key` findings**,
**220 scanned commits**, and **231 reachable commits including 11 merges**.
All 84 primary values remain classified as source-file SHA-256 audit records;
no primary finding is unresolved. The parent repeated 96 source-blob comparisons;
the primary evidence contains those 96 checks. This supplement did not repeat
them. The retained primary raw report was rehashed and still matches
`f32505a06c28f8ea409ec4ee064dc3ad86e961baaf67471cc3833903834f36c7`.
The three primary documents and parent-owned README remain byte-for-byte unchanged.

The [machine record](source-history-merge-supplement.json) contains every full
merge ID and parent ID, tree IDs, per-parent patch hashes, exact commands,
sanitized command trace, raw artifact hashes, counts and limits.

## Exact source, tool and policy

The session used GPT-6.1-sol, xhigh reasoning and STANDARD service mode, verified
through the current T3 runtime catalog. No child agent was launched.
The isolated checkout started clean at `542ab5df5549cb68991356d0dba69f3c7b470098`.
That checkout HEAD is not the scan selector. Only merge commits reachable from
exact `ca618418dc8c4d93b8fd04eeda3fefa01003b8e7` were scanned. No `--all`, other
refs, uncommitted files, later reports or ordinary-commit scanner re-scan was used.

The retained official-release archive and executable were rehashed before use:

- Archive SHA-256: `b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
- Executable SHA-256: `ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`.
- Version command: `8.30.1`, exit 0. Git: `git version 2.54.0 (Apple Git-157)`.

Local `Gitleaks git --help` confirmed the single string `--log-opts` option.
Local `git help log` confirmed `--merges` and full separate parent diffs with
`--diff-merges=separate`. External diff and text conversion were disabled to
prevent helper execution; no such drivers were configured. No historical code
was run. Nothing was downloaded, installed or updated.

The scan used the same built-in default rules and allowlists, a dedicated empty
ignore file, ignored inline allow comments, and 100 percent redaction. It retained
archive depth 2, decode depth 5 and timeout 120 seconds. No root `.gitleaks.toml`
or configuration override was present. No rule filter, baseline, custom config,
new suppression or weaker detector policy was used.

From the repository root, substitute only the verified executable and private
mode-0700 directory paths. Private output files use mode 0600:

```sh
umask 077
: > "$PRIVATE_DIR/empty.gitleaksignore"
env -u GITLEAKS_CONFIG -u GITLEAKS_CONFIG_TOML \
  GIT_TRACE="$PRIVATE_DIR/git-command.trace" \
  GIT_TRACE2_EVENT="$PRIVATE_DIR/git-trace2.jsonl" \
  "$GITLEAKS" git . \
  --log-opts="--merges --diff-merges=separate --no-ext-diff --no-textconv ca618418dc8c4d93b8fd04eeda3fefa01003b8e7" \
  --redact=100 --ignore-gitleaks-allow \
  --gitleaks-ignore-path "$PRIVATE_DIR/empty.gitleaksignore" \
  --max-archive-depth 2 --max-decode-depth 5 --timeout 120 \
  --no-banner --no-color --report-format json \
  --report-path "$PRIVATE_DIR/findings.json" \
  > "$PRIVATE_DIR/history.stdout" 2> "$PRIVATE_DIR/history.stderr"
scan_status=$?
printf '%s\n' "$scan_status" > "$PRIVATE_DIR/history.exit"
```

The actual invocation used the absolute checkout path. The private
`history-result.json` records its argument vector and saved status. The Git
trace recorded this underlying history command:

```sh
git log -p -U0 --merges --diff-merges=separate --no-ext-diff --no-textconv ca618418dc8c4d93b8fd04eeda3fefa01003b8e7
```

The trace also records one local `ls-remote --quiet --get-url` lookup and three
`cat-file blob` reads for archive files at reachable merge `06b61287afd7f31aa3002d252389f2c2b17f73f5`.
The URL lookup does not contact a remote. All five traced Git processes exited 0.

## Parent coverage and counts

`git rev-list --merges --parents` at the exact source lists **11 unique merges**
and **22 parent pairs**. Replaying the exact traced Git command emitted
**21 separate parent patches**, with **21 log headers covering all 11 merge IDs**.
Ten merges emitted both parent patches. Merge
`06b61287afd7f31aa3002d252389f2c2b17f73f5` emitted only its first-parent patch.
Its second parent, `f6401e98f0203cb0e13c954e482ba6b6444e70f6`, has the identical tree;
its direct diff is empty. No nonempty parent diff is missing.

Each emitted patch matched a direct
`git diff -p -U0 --no-ext-diff --no-textconv <parent> <merge>` byte-for-byte.
Twenty entries have one extra log-entry separator LF; the last has none.
The machine record gives the exact comparison and hashes for all 22 parent pairs.
These direct comparisons check patch coverage and do not scan ordinary commits.

The scanner counter of 11 equals the number of distinct merge IDs, while the
Git output has 21 repeated parent log entries. These are different counts.
The scanner reported approximately 11,694,687 bytes; the full replayed Git stream
contains 11,620,607 bytes. Those byte metrics are distinct and are not snapshot counts.
The scan is patch-based evidence, not proof that every historical snapshot
was checked.

## Findings, raw evidence and first failure

The raw incremental report is an empty JSON array. It has **0 reported locations**,
**0 classifications**, **0 duplicate finding rows across parents**, and
**0 unresolved locations**. There were no new values to classify, and no primary
identity was reused. The separate merge result does not turn the primary raw
exit 1 into a passing or zero-finding result.

Incremental raw report SHA-256:
`37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`.
Raw Git patch-stream SHA-256:
`96020396e849b3bdf3a7e230dd248e92f21d26b0b213046d03eb8fd83a7e9b20`.
The machine record includes all retained raw output, trace, status, help and
per-parent diff hashes. Raw patches and candidate-bearing artifacts remain private.

The first post-scan trace-parser assertion failed because it expected an argv
prefix of `git log`. Trace2 used an absolute Git executable and `-C <repo> log`.
The parser was corrected against the recorded argv. No scanner rerun was needed.
Both `history.exit` and `history-result.json` already held the authoritative
incremental exit 0 before that failure. The expected empty Git configuration
lookup returned 1; it is not a scan failure.

## Limits and unchanged gates

This supplement closes only the stated merge-diff gap at the exact source and
under the stated detector policy. It makes no claim that the repository is free
of secrets or vulnerabilities. Uncommitted files, later commits and reports,
other refs, unreachable objects, external systems and this new report commit
are outside scan scope.

The provider-stopped IPC task in `t3-7ed97fb2`, older expiry/NSEC/orphan-record
testing, and the original blocked release-provenance security review were not
accessed, continued, reviewed, rerouted, rephrased or replaced. Native
input/clipboard, positive Windows migration, provenance/security acceptance and
trusted distribution remain open. Private GitHub reporting remains deferred.

No product changes, builds, runtime traffic, unit tests, mocks, computer use,
other-thread messages or additional agents were used. No installed/live sharing,
state/TLS/default IPC, desktop/TCC/login/service/task/account/credential,
firewall/network/global-resource, 24800/5353, signing, publication, GitHub-post,
push or merge changes were made. No credentials were rotated or modified.
Only the two new `source-history-merge-*` documents are committed.
