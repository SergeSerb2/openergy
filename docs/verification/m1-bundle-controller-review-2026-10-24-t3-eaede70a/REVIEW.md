# Fresh M1 bundle controller: independent source review

Date: 2026-10-24, America/Vancouver. Reviewer worktree: `t3-eaede70a`.
Candidate: `65b95a7c6643beba780e6284968b246496bbd4c9`.
Reviewed package: `docs/verification/m1-fresh-build-plan-2026-10-24-t3-c52ad306/`.

**SOURCE HOLD: one P2 finding; no P1 finding.** This is a source verdict only.
Execution remains false. No candidate code, supplied helper, compiler, signing,
product command, remote command, test, mock or simulation ran during this review.
The candidate was not changed. Only this report and its bindings were added.

## P2-1: normal directory removal can abort the filesystem sampler

Locations: [controller.py](../m1-fresh-build-plan-2026-10-24-t3-c52ad306/controller.py),
lines 58–59, 448–458 and 572–577; [bundle.sh](../../../apps/macos/scripts/bundle.sh),
lines 32–35 and 104.

`usage()` runs `os.walk(..., onerror=unreadable)`. That callback raises every
directory-scan error. The adjacent `FileNotFoundError` handler protects only
the subsequent `lstat()` call. It does not protect `os.walk` when a previously
listed child directory disappears before its recursive `scandir()`.
The hash-bound CPython 3.13.9 archive confirms this path in `os.py`, lines
369–372: a failed `scandir(top)` calls the supplied `onerror` callback.

The actual bundle interface creates `.openergy-bundle.*`, moves its staged
`Openergy.app` tree to the final destination, and removes its staging tree in
the EXIT trap. Thus a sampler can list a directory in that tree before the
move or removal, then try to scan its former path after the move or removal.
This is normal behavior of the unchanged command. The resulting ENOENT escapes
`usage()`, becomes the controller failure reason, and requests group recovery.
It can fail the one-shot operation even when no size or entry cap was exceeded
and the bundle command itself completed successfully.

Treat a vanished descendant directory consistently with the vanished-file case.
Continue to refuse other scan errors and loss or replacement of the owned root.
Keep the same caps, shared deadline and recovery policy. This requires a source
repair and review before this candidate can receive SOURCE CLEAR. No runtime
incidence rate or reproduction is claimed; the finding follows from the actual
walk callback and the actual bundle directory operations.

## Exact review and data checks

I read all 637 controller lines, all 12 wrapper lines, all 322 README lines,
the budgets, template, source bindings, workspace closure and static-check
record. Every package file equals its candidate Git blob. All eight entries
in its `SHA256SUMS` match. The three requested identities are:

- Controller: `9b13db9b9f382ffcf17a3be1d311374208191680c89fa493bfbdad1be631eb98`.
- Template: `3d2e63163ae559096aeeda656bc01ca7e8e7c54454822c3e3a9a8e1d4b59be83`.
- Bindings: `20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617`.

A reviewer-authored standard-library data reader parsed the controller AST
using Python 3.11 grammar. It did not import or execute the controller. It
independently compared every one of the 702 source records with raw Git blobs,
lengths, SHA-256 values and executable modes at
`ed7d96bf53da1a539fa2acde3b7133613221fd4b`. The selection is exactly all tracked
non-docs files: 4,773,386 bytes. The current product sources have no diff from
that source commit. Independent TOML reads confirm 11 member manifests,
13 local manifests and 24 local path edges, including `tools/e2e` and the
`vendor/mdns-sd` patch. They match the retained workspace closure.

I also read the actual retained archives without extraction. All archive
hashes and all payload records match their bindings: source R3 has 702 regular
files; Rust has 225; Cargo has 210. There are no directory, link, sparse or
special entries in these three exact archives. Rust has 20 mode0755 and
205 mode0644 records. The source has 16 mode0755 and 686 mode0644 records.
The controller maps the Cargo records to mode0644 within its private root.
The R3 archive is 5,365,760 bytes with SHA-256
`e28b00daeaf5de62ab12f1e0165a5ba1b815d22ca9139a74b54f599e20ada277`;
its receipt matches
`7ec86206f0ad7a0931451e46c0e056e6dea07223cd60e4e05d4037f1b4f5c0bb`.

## Preparation findings and evidence scope

I read the [native tool qualification](../m1-native-tool-qualification-2026-10-24/README.md)
and its JSON, and the [prerequisite staging evidence](../m1-prerequisite-staging-2026-10-24/README.md)
and its JSON and first failure. Their public manifests match their file bytes.
All ten private records named by the native-tool qualification also match
their retained lengths and SHA-256 values. Those were read as data only.

The initial 435-file source omission is addressed in the new R3 input set.
Its original version-helper failure remains FAILED. The R2 Git-archive CRLF
mismatch also remains FAILED. R3 preserves the raw-blob equality predicate;
it does not reinterpret either failure as a pass.

The retained successful R3 qualification records the real pinned CPython
3.13.9 interpreter calling `release.py version`: exit0, `0.1.0`, empty stderr,
0.157396 seconds. The retained Rust/Cargo records show both real version
commands at 1.99.1, exit0, empty stderr, and bootstrap NI15. The retained
Cargo metadata command uses `--locked --offline --no-deps`; its result has
11 workspace members, 11 workspace packages and an unchanged lockfile.
These resolve the stated preparation blockers within their exact scope.
They do not establish a compiler build, Swift resolution or bundle result.
The first failed quoting transfer remains intact; the successful separate
transfer does not overwrite its result.

The M1 physical fingerprint, direct SSH route, logged-in desktop and full
Xcode baseline are parent-verified facts supplied for this task. I did not
contact the M1 or inspect a live process, port or OS state. The operation still
needs the parent's concrete paths, prefix digest and absolute deadline.
That binding is not a new human approval requirement for the already
authorized ordinary build.

## Other requested source controls

No additional actionable P1/P2 was found in these controls:

- The disabled template fails before root creation or priority changes.
  Concrete binding checks include recipe hashes, source commit, canonical
  absolute paths, an absent fresh root, owned mode0700 parent, and 32 GiB free.
  The Python prefix digest covers sorted paths, modes, lengths, file hashes
  and internal link text, with explicit entry and byte caps. Interpreter
  authentication before startup remains the parent's responsibility.
- Exact archive hashes precede extraction. Extraction uses exclusive regular
  file writes, checks member paths, lengths, modes and hashes, rejects links
  and extra or missing members, and starts in the new root. No old Swift
  scratch, build database, target output or Git metadata is reused.
- The actual `bundle.sh`, `priority.py`, `release.py` configuration/version
  path and `sparkle.py` embed/sign/verify paths were inspected. Resolve is
  called once, with a 240-second cap; ordinary bundle is called once, with
  a 900-second cap. No fallback, retry or second app build is added. The
  existing `--show-bin-path` call queries the path. The icon script is a
  normal bundle step. `release.py package` is not invoked.
- The Swift wrapper uses `exec`, preserves arguments and stdout, and adds
  private cache/config/security paths plus disabled Keychain/netrc lookup
  only for `build` and `package`. It leaves the normal engine, sandbox,
  explicit pinned resolution and build's disabled automatic resolution intact.
  The icon-script invocation passes through without package options.
- The copied real Rust tools precede system tools; no rustup shim is selected.
  Cargo is offline and locked. Jobs remain 2, deployment remains 15.0,
  `UNIVERSAL=0` selects the verified arm64 host, and the Rust build target is
  `aarch64-apple-darwin`. Sparkle source revision and binary checksum remain
  pinned. The normal helper embeds and verifies that artifact.
- The signing environment selects `-`. The helper maps this to ad-hoc signing
  with `--timestamp=none`, signs nested code and the app, then verifies
  architecture, linkage and signatures. It does not select a named key.
  The bundled daemon's isolated `version` branch only prints the version;
  no app launch, daemon start, updater or native input flow is added.
- The directly forked keeper calls `setsid()` before READY. Before G, the
  parent binds the kernel PID/UID/start identity, PGID and observed NI15.
  Group inventory has a 2,048-PID buffer and refuses a full or uncertain
  inventory. Samples require observed NI15 and use the same kernel identity
  in final checks. No process name or guessed/reused PID is signalled.
- The keeper owns the unique control pipe. After G, its separate lifeline
  handles T, EOF, uncertainty and the shared stop deadline. It sends TERM to
  its own anchored group, waits five full monotonic seconds, then sends KILL.
  It remains alive through the grace period. Caught signal handlers do not
  leak ignored TERM into executed tools. The parent only requests recovery
  through the pipe. Uncertain parent identity still closes that pipe.
- Per-command timeouts lead to the common failure path without immediately
  killing the direct child. Total work stops at the shared deadline minus
  15 seconds, with five seconds for TERM grace and ten for reporting. No
  stage renews the deadline. The normal Sparkle helper's internal timeout
  behavior remains a separately documented limit.
- Combined logs retain at most 16 MiB; sample data retain at most 8 MiB.
  The root has sampled 24 GiB / 200,000-entry caps. Worker regular files have
  inherited 2 GiB RLIMIT_FSIZE, and core dumps are disabled. Metadata has
  finite timeouts and a 16 KiB acceptance limit after command completion.
  Postflight checks cover bound source, copied tools, seed data, original
  archives/receipts and Python prefix, then inventory the produced bundle.
  No root deletion or host-restoration success is claimed.

The local macOS 27.0 SDK headers agree with `PROC_PGRP_ONLY=2`,
`PROC_PIDTBSDINFO=3`, the `proc_listpids`/`proc_pidinfo` C argument and return
types, and the declared `proc_bsdinfo` field order and widths. With MAXCOMLEN16,
the layout is 136 bytes; start fields follow nice at offsets120 and128.
The local header also defines SZOMB5. This was a header/data check, not a
libproc call or native ABI runtime qualification.

## Residual limits and handoff

Before G, setup failure cannot start a payload. After G, missing or failed
worker evidence requests recovery; a missing terminal receipt is failure.
Errors opening initial logs occur before G. Later failed writes can prevent
a complete result receipt; the pipe/deadline recovery is separate from those
parent evidence writes. It does not establish complete evidence on a full or
stalled filesystem.

Filesystem accounting and process priority are sampled. Unseen, short-lived or
session-escaping descendants and Xcode/XPC services remain UNVERIFIED. Normal
OS-managed temp/cache writes are permitted outside the owned-root accounting.
There is no outer filesystem-denial profile, universal quota, universal
descendant observation, hard real-time OS-call bound, or host-restoration
claim. Parent collection is not implemented by this controller; a later
collection must retain its actual completeness, byte and deadline limits.

No future bundle-completion receipt would establish native acceptance or
trusted distribution. The original ordinary result remains FAILED / NOT
ACCEPTED, 23/25; native Windows stop remains FAILED / UNVERIFIED. The expired
Sparkle operation remains closed. The old isolated failure, its predicates
and its unproved cause are unchanged. Runtime for this candidate is NOT RUN /
UNVERIFIED. This review grants no native, updater, release or cleanup clearance.

The only source blocker from this review is P2-1. Repair the directory-churn
handling, preserve all other predicates, and obtain review of the exact new
candidate before concrete parent binding and execution.
