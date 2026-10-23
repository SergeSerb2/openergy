# Source-history current-parent scan — 2026-10-23

Exact accepted parent: `7ad8265f9237aad8d37672095c884f051b5e4c2f`. Gitleaks **8.30.1** ran two full
patch-history scans against only this commit and its reachable ancestors:

- Non-merge scan: **exit 1**, **95 `generic-api-key` findings**, **243 commits scanned**.
- Separate merge-parent scan: **exit 0**, **0 findings**, **11 merges scanned**.

All 95 findings were verified as source-file SHA-256 records. **None remain
unresolved.** The 84 prior locations remain; the 11 new locations occur in
`virtual-baseline.json` at evidence commit `988e7837f212fcbde355e8e2f7452e82faf5a412`.
Each new record matches the specific Git source bytes at its declared source
`ca618418dc8c4d93b8fd04eeda3fefa01003b8e7`. All 95 candidates were bound to their exact evidence
blob, reported line and unique JSON leaf. The checks included **107 source-blob
comparisons** and the existing vendor archive member and recorded revision.
No credential was identified among the reported findings. The raw non-merge
exit remains **1**; classification does not make it a passing raw scan.

The [machine record](source-history-current-snapshot.json) contains exact
commands, coverage-set hashes, all 23 new commit IDs, baseline bindings,
sanitized evidence for the 11 new locations and private artifact hashes.

The selected history contains **254 reachable commits**, compared with 231 at
the prior primary. The exact set difference contains **23 non-merge commits**
and **no new merges**. This refresh repeats the full scans; it does not rely
on the prior result to omit patches. The current 95 include the original 84.

All **243 non-merge commits emitted patches**. The merge replay accounted for
**22 parent pairs**: **21 emitted nonempty patches** and **one identical-tree
empty diff**. Each emitted patch matched its direct parent-to-merge diff,
allowing only the recorded log separator LF. All parent pairs and patch hashes
match the unchanged prior merge supplement. No nonempty parent diff is missing.
This is coverage of patches at the selected parent, **not a scan of every
historical full snapshot** or proof that the repository is secret-free.

The retained official archive and executable were rehashed before use:

- Executable SHA-256: `ba52fb1bfabbcde42f032afad3d6e0b19dff8ed105229a16e7caa338bbc0e84f`.
- Archive SHA-256: `b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
- Version command: `8.30.1`, exit 0. No download, install or global change.

The detector policy is unchanged: built-in default rules and allowlists,
100 percent redaction, an empty private ignore file, ignored inline allow
comments, archive depth 2, decode depth 5 and timeout 120 seconds. No custom
config, rule filter, scanner baseline, new allowlist or suppression was used.
External diff and text conversion were disabled. The traced Git selectors were:

```sh
git log -p -U0 --no-merges --no-ext-diff --no-textconv 7ad8265f9237aad8d37672095c884f051b5e4c2f
git log -p -U0 --merges --diff-merges=separate --no-ext-diff --no-textconv 7ad8265f9237aad8d37672095c884f051b5e4c2f
```

Non-merge raw report SHA-256:
`837313f7b25405241fb6d61c25d68fbd03daebe4d0bc9a6038998251101aa517`.
Merge raw report SHA-256:
`37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`.
The scanner statuses were saved before parsing. Both runs completed in about
1.225 seconds total. Raw reports, patches, traces and helpers remain outside the
repository in a mode-0700 directory with mode-0600 files and a hashed inventory.
The machine record gives the private local locator. Prior reports, the README
and retained prior raw artifacts remain byte-for-byte unchanged.

Only these two new evidence files are added. JSON, exact finding identities,
source hashes, reachability, merge coverage and private permissions were checked.
No product services, product-port traffic, native input, settings, builds,
tests, mocks, dependencies, GitHub changes, push or publication were used.
Sibling refs/worktrees, uncommitted work, unreachable objects, later commits
and this evidence commit are excluded. No `--all` scan, child agents or
other-thread messages were used. Stopped expiry, IPC/Synergy3 and original
release-provenance/security-review work were not accessed or resumed.
