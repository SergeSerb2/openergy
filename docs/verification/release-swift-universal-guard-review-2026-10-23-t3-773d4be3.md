# Held Swift universal guard: independent source review

Date: 2026-10-23, America/Vancouver. Owned isolated worktree:
`/Users/serge/.t3/worktrees/openergy/t3-773d4be3`.
Current branch: `t3/review-swift-universal-guard`.
Review base: `32dc870e039ed35b3f79ba5e45181bb4ba7e8410`.

**No actionable new P1/P2 finding in this bounded review. The original
universal-mode P2 is closed at source level by the early rejection guard.**
The two saved real invalid CLI outcomes support that narrow conclusion.
Ordinary defaults have source comparison only. The held opt-in remains
unintegrated, the isolated release build remains failed, and packaging and
runtime acceptance remain open. This document gives no source-ready,
integration, or release approval.

## Exact inputs and review boundary

- Guard source: `46d2c3b604ca9414cc75fba598f5c0a9e5a7e8dc`, whose parent is
  `60ca525948e076ce034d8e304dff1ddffdb6f13d`.
- Saved evidence/owner HEAD: `6badf9582ad7faad8404e1d169b1ab4575fa4947`, at
  `/Users/serge/.t3/worktrees/openergy/t3-0966fd74`.
- Evidence prefix: `docs/verification/release-swift-app-qualification-2026-10-23-t3-0966fd74/`.
  References to `raw/` below mean files under this prefix at that commit.
  The report is its `README.md`.
- Original held opt-in: `fc33bdc0cbb261d0a096ec459b5ba0dce25ae9ec`;
  original evidence: `567eb51d48d88731ecaf0272b1be98e35f1e3a93`.
- Earlier independent review: `899bdb1af594c3ee1949dc9e83b5272c425f7d63`,
  `docs/verification/release-swift-isolation-review-2026-10-23-t3-73b2eeaf.md`.
  I used its original P2 and scope as context, not as this round's conclusion.
- Saved causal research: `5979cff6f9cb2206e401fa3d34b6c0587a5e8eff`,
  `docs/verification/release-swiftpm-atomic-write-research-2026-10-23-t3-57d3a189.md`.
  Its candidate path and cause limits remain separate from guard acceptance.

This is a new review of the held objects. I read the complete guard-source
commit diff, the complete 143-line bundle script, the original opt-in diff,
the repair relative to that opt-in, and the full new evidence README. I read
the two saved CLI recipes, starts, results, complete streams, snapshots,
qualification record and relevant controller code as data. I also read the
narrow build failure records and saved research context. I did not repeat the
prior full evidence audits or recertify the original opt-in's broader scope.

Only this new review document is owned. Neither held source commit is an
ancestor of this review base. Neither was checked out or integrated here.

## Source closure and ordinary argument policy

At the guard source, `apps/macos/scripts/bundle.sh:8-13` checks the two quoted
values with `${...:-0}` immediately after `set -euo pipefail`. When both are
exactly `1`, it emits the single-architecture diagnostic to stderr and exits
1. This branch contains only shell parameter expansion, `[[`, `echo` and
`exit`. It precedes the first command substitution and external command:
`dirname` at line 15 and `uname` at line 21. It also precedes opt-in path
checks, signing-policy/config helpers, destination creation at line 56,
Cargo work, and both Swift build calls. Thus the rejected pair cannot reach
the two-architecture selection at line 69 or Swift argument construction.

The full commit adds the opt-in to its parent. Comparing its bundle/docs
content with original opt-in `fc33bdc0` leaves only the seven-line guard and
the single-architecture documentation change. Outside verification documents,
only `apps/macos/scripts/bundle.sh` and `docs/releasing.md` differ between those
two source snapshots. The saved `raw/held-to-source.patch.gz` is byte-identical
to that independently generated two-file diff.

For an unset, empty, or `0` opt-in, the guard falls through. The default
executable remains `swift`; isolation flags remain empty and the icon command
remains the one-word `swift` array. The same release/jobs/resolution/scratch
arguments reach both Swift build and bin-path calls. `UNIVERSAL=1` still
selects arm64 and x86_64 when the opt-in is off. `UNIVERSAL` remains restricted
to 0/1 at line 52; the opt-in retains its 0/1 validation at line 27. No guard
operand executes or interpolates a command. I found no new argument-splitting
or ordinary-default regression in this comparison. These paths were not run.

For enabled single-architecture mode, executable/path validation and the
native/disable-sandbox/credential flags are unchanged from the original held
opt-in. Canonical tools, private fresh state, and an enforced outer policy
remain operator duties. The checks do not establish those properties.
`docs/releasing.md:137-164` states that boundary and now excludes isolated
universal builds. This closes the original backend-override trigger without
qualifying the single-architecture build or the broader opt-in.

## Two saved real CLI outcomes and input bindings

Let `P` mean the exact saved root
`/Users/serge/.t3/worktrees/openergy/t3-0966fd74/.build/release-swift-20261023-t3-0966fd74-once`.
Each saved recipe invokes the real `P/repo/apps/macos/scripts/bundle.sh`
through `/bin/bash -x`, the real priority launcher and `sandbox-exec` using
`P/package.sb`. The recipe and start argv agree. Both recipes set
`OPENERGY_SWIFTPM_OUTER_SANDBOX=1` and `UNIVERSAL=1`. Their only environment
difference is `DIST`: `P/invalid/existing-dist` versus `P/invalid/absent-dist`.
Both use `P/invalid/absent-swift` and `P/invalid/absent-cargo` for scratch and
Cargo target state. The absent scratch intentionally shows that rejection
precedes the enabled-mode directory requirement.

The existing case returned 1 in 0.045348125 seconds. The absent case returned
1 in 0.048302791 seconds. Neither result records a timeout or controller error.
Both complete stderr traces show only `set`, the two successful `1` comparisons,
the exact guard diagnostic and `exit 1`. Both stdout streams decode to zero
bytes. Their stderr hashes both equal
`c907ecdf73c09f0887a9d3bbb399dd638e15c64dc24ef53985ca7cbbca191baf`.
The trace stops before any bundle external work; the enclosing sandbox and
priority launcher necessarily ran before the bundle entered its guard.

I compared the decoded `invalid-before.json` and `invalid-after.json` directly:
they are byte-identical. They retain the existing 43-byte sentinel, SHA-256
`528a0150a1326d1db3762f248068f60184d98b96f7e5608c6a80ef2dd83e4e68`,
and `sentinel-link` target `Openergy.app/sentinel`, plus selected metadata and
the source inventory. The absent destination, scratch and Cargo target have
no entries. The controller's saved checks and `invalid-qualification.json`
also record their continued absence. These are saved observations, not new
filesystem probes. Read atime is excluded; ACL/xattr completeness is not proved.

The script's Git-blob SHA-256 is
`3efe901e78a18d87c419a7efdac99614f1576c5535dfeac63aa86f67ba767bd2`;
the releasing document's is
`da91282cc2d7728c1f86c30fafdaf0e82855027eaa8fac6fd94a174a2449119d`.
Both independently match `source-binding.json` and the before/after CLI source
records. The source binding names exact commit `46d2c3b604ca9414cc75fba598f5c0a9e5a7e8dc`.

The start-record environment digest has a narrower meaning than the full
per-case environment. `raw/controller.py:71` hashes the common `environment.json`,
not the `env` passed to `Popen`. Both CLI starts therefore contain the base
digest `89c9c5329d480f3df5eff973b891dfbf5e0b2cb6023fe1a822f7430f9ed8c5e4`.
The per-case overrides are instead supported by the full recipes, controller
lines 102-106 passing that same environment object to the real command, and
the two complete guard traces. I do not treat the start digest as a hash of
the overridden environment. This limit does not change the observed `1`/`1`
rejection or its source closure.

I checked exported and decoded SHA-256 values against the ledger for 24
selected records: the source binding, repair patch, CLI recipes/starts/results/
streams/snapshots/qualification, and narrow build-context records. All matched.
I also checked the controller hash against the README binding. I did not
recheck all 82 exports, any of the earlier 97 exports, retained private originals,
or live old-cache inventories. The saved final preservation audit reports
unchanged earlier source/evidence/cache/guard state; this round does not add
a fresh preservation or original-metadata-restoration claim.

## Failed build context and remaining proof

The saved build recipe and start contain the same app-only release command:
one arm64 architecture, native backend, jobs 2, private state paths, selected
SDK, disabled inner sandbox, and `--product Openergy --verbose`. It bypasses
bundling and is separate from the two invalid bundle CLI calls.

`raw/app-build-result.json.gz` records exit 1 after 18.165488707993063 seconds,
without timeout or controller error. `raw/build-stop.json.gz` records one build
and an explicit stop. Complete stderr ends at line 11 with the generic I/O
code 1 diagnostic naming `Openergy.build/output-file-map.json`; its hash is
`340324068d5d0ed9eb0dd6df0afbee08017d5f50f4bafe8dbbf8fab816b36f27`.
Stdout is empty. Earlier lines show project and Sparkle manifest compiler/linker
commands, plus the native-backend deprecation warning. They contain no Openergy
application frontend command or completed app link. Saved output inspection
records the map, `master.swiftdeps`, and release executable as absent.

The saved priority audit observes NI15 for one command exec chain. The transient
canonical `swift-frontend` path belongs to the dispatcher in that chain.
Separate manifest compiler/clang/linker NI15 was not independently captured.
There is no application-frontend success. Cleanup covers known sampled identities
and groups; it does not prove that every possible descendant was observed.

The missing map and generic diagnostic do not identify the failing filesystem
operation, temporary path, NSError chain, or cause. The saved research identifies
only a candidate atomic replacement path. It proves neither that path's use by
the failed process nor a supported private configuration repair. The saved empty
log query also excludes no cause. The guard addresses a different input pair
and does not explain this single-architecture failure.

The isolated Swift packaging gate stays stopped pending an approved disposable
macOS environment with process-scoped filesystem/Foundation instrumentation.
No new build attempt or executable receipt is authorized by this review. Whole
app completion, bundle assembly, signing, ZIP/mode/link checks, staged/extracted
IPC, UI, launcher and runtime/package acceptance remain unproved. Running-target
Sparkle approval and all three provider-stopped tasks remain pending or stopped.

## Actions taken in this round

I used only read-only Git/file/decompression/JSON/hash comparisons and authored
this document. No supplied program was run or imported. No project CLI, manifest,
build, package, signing, runtime, process/port/network query, test, mock, simulation,
delegation, or computer use occurred. I made no installed-app, live-sharing,
production config/TLS/default IPC, TCC/login/service/task/account/firewall/global
DNS/toolchain change, credential/key access, TCP24800/UDP5353 use, Docker, push,
or publication action. The review commit contains this document alone; its exact
hash and the post-commit clean-state check are reported in the final handoff.
