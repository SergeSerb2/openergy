# M1 sampler R4: independent source review

Date: 2026-10-24, America/Vancouver. Reviewer worktree: `t3-ecd3814b`.
Exact candidate: `7f20c9cdd97c1dc313f007b879babc726acbb914` (`7f20c9c`).
Package: `docs/verification/m1-sampler-source-repair-r3-2026-10-24/`.
Prior candidate: `65b95a7c6643beba780e6284968b246496bbd4c9`.
Prior independent review: `931f7fd79357eeb8f6b7a245bbf094408f578f75`.

**SOURCE CLEAR. No concrete P1 or P2 findings. Prior P2-1 is closed by this
source repair. Runtime remains NOT RUN / UNVERIFIED.** This verdict applies to
the exact source package above. It does not claim a successful M1 operation.

## Exact inputs and minimal change

I read the full [prior review](../m1-bundle-controller-review-2026-10-24-t3-eaede70a/REVIEW.md),
including every control and proof limit, its bindings, and the full inherited
[R1 plan](../m1-fresh-build-plan-2026-10-24-t3-c52ad306/README.md).
I read all 656 current controller lines, all 12 Swift entry lines, the README,
budgets, template, patch and static record. I parsed the complete source-binding,
input-binding and workspace-closure JSON as data. Candidate source and AST were
never imported or executed.

Independent reviewer data checks establish:

- All 11 package files match their exact candidate Git blobs and modes. The
  candidate commit adds only these 11 files. All 10 `SHA256SUMS` entries match.
- All 11 input bindings match their stated commits, lengths and SHA-256 values:
  nine prior-package files and both prior-review files.
- `repair.patch` is byte-for-byte the independently generated controller and
  template unified diff. Only `usage` and `main` differ in the controller AST.
  All other top-level nodes, functions and classes are identical, including
  strict `unreadable`, `prefix_hash`, `operation`, `keeper`, `Processes` and `BSD`.
- In `main`, the only AST changes are the two creation-token assignments and
  the added token argument at both `usage` calls. Removing only those changes
  from the parsed AST makes it identical to the prior `main` AST.
- Swift entry, source bindings, workspace closure and budgets are byte-identical
  to R1. Template `execution` remains false. Only its operation ID and controller
  digest change. All three recipe hashes match the exact current files.
- All 702 bound non-docs source paths, raw Git blob lengths, SHA-256 values and
  executable modes match `ed7d96bf53da1a539fa2acde3b7133613221fd4b`:
  4,773,386 regular bytes. Product source has no Git diff from that commit.
  Independent TOML reads confirm 11 members, 13 local manifests and 24 path
  edges, including `tools/e2e` and `vendor/mdns-sd`.

Exact SHA-256 values:

- Controller: `cd5dc620e6adbec09798e600a99f55c0368b8eeff10d1117e1f7793ca1baf4ee`.
- Template: `0d3ded36f73b320e31a4214631dbf941de0e540debcf9baeb01de6ef18eb7b4e`.
- Source bindings: `20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617`.

The candidate whitespace check exits 2 for `repair.patch` lines 5, 6, 37 and 38.
These are the single-space context markers for blank lines in the exact unified
patch. This result is retained. It is not a controller defect or a P1/P2 finding.
The reviewer data reader also first failed an assertion that assumed Python
3.15 `ast.dump` prints `ctx=Load()` on Name nodes. The corrected reader compares
Name identifiers and Load node types directly. All comparison predicates remain.
Neither reader ran candidate code. Details are in [review-bindings.json](review-bindings.json).

## Repair assessment

At controller lines 456-462, the usage-specific callback returns only for
`FileNotFoundError` with `errno.ENOENT` and a string filename whose Path is
absolute, has no `..` component, differs from the root, and is a component-wise
descendant of that root. This covers the listed child directory that disappears
before recursive scanning. The actual unchanged `bundle.sh` creates staging at
lines 32-36, moves the app at line 104 and removes staging in its EXIT trap.
The prior failure path therefore has a direct source repair.

The path check is lexical. It uses the canonical absolute root established by
`absolute()` and paths supplied by the real `os.walk`; it does not resolve a
vanished child. It does not certify arbitrary caller-supplied paths or symlink
targets. Unknown, non-string, relative, root, outside-root and `..` paths do not
enter the exception. Permission errors, ENOTDIR, EIO, another errno, another
exception class and all other walk errors still fail. No broad OSError catch or
blanket ENOENT suppression was added.

At lines 505-507, the parent captures the newly created root's device and inode
before evidence writes and fork. It keeps this token; neither sampling nor the
worker refreshes it. Both the periodic call at line 592 and the final call at
line 586 use that same token. The final check precedes release of the keeper.

Each call checks `lstat` before and after traversal, at lines 449-454, 464 and
475. Both checks require directory type, current real UID, exact mode0700 and
the original device/inode pair. A missing root fails; a root scan error cannot
qualify as a descendant error. If root loss makes a child scan return ENOENT,
the post-walk root check still fails. A replacement that remains present at a
sample boundary fails even if it has the same owner and mode. Replacement
between samples is caught by comparison with the creation token on the next
sample. This closes the replacement hole from refreshing the expected token
per sample.

`followlinks=False` remains explicit. The existing `lstat` FileNotFoundError
handler and entry/byte counting are unchanged. The strict `unreadable` callback
at lines 58-59 still protects the immutable Python-prefix walk at line 88.
Normal mutable-child churn no longer relaxes immutable-input hash checks.

## Preserved controls and evidence boundaries

The full prior-review controls remain applicable. Source inspection and the
exact unchanged AST/input comparisons preserve these conditions:

- Disabled execution is refused before root creation or priority changes.
  Recipe/source hashes, concrete canonical paths, absent fresh root, owned
  mode0700 parent and 32 GiB free-space requirement remain. Parent interpreter
  authentication must precede startup; the full prefix digest is checked again.
- Archive hashes precede exact exclusive extraction. Link, special, extra,
  missing and mismatched records are refused. No old Git, target, Swift scratch,
  build database or compiled state is reused. Bound inputs are checked again
  after the operation.
- One resolve call retains its 240-second cap; one ordinary bundle call retains
  its 900-second cap. No retry or fallback appears. Normal SwiftPM engine,
  sandbox, pinned Sparkle revision/checksum, disabled automatic build resolution,
  private dependency paths and Keychain/netrc restrictions remain.
- Direct Rust/Cargo 1.99.1 tools, offline locked Cargo, jobs2, deployment15.0,
  arm64, and ad-hoc signing with no timestamp remain. The isolated bundled
  daemon version command remains the only product command inherent in bundle.
  No package, app launch, daemon start, updater or native-input flow is added.
- Before G, the keeper's kernel PID/UID/start identity, PGID and observed NI15
  remain bound. Group inventory still refuses uncertainty or a full 2,048-PID
  buffer. Final checks retain the same identities. The parent uses the unique
  control pipe, never a process-name or guessed/reused PID/PGID signal.
- The independent keeper lifeline still handles T, EOF, uncertainty and the
  common stop deadline. It retains its live group anchor through TERM, five
  full monotonic seconds, then KILL. Caught handlers do not leak ignored TERM
  into executed tools. Per-command timeout still requests that recovery path.
- The 1,800-second absolute deadline is converted once and never renewed.
  Work stops 15 seconds early, with five seconds for TERM grace and ten for
  reporting. Root accounting retains sampled 24 GiB / 200,000-entry caps and
  the nominal two-second interval. Combined logs remain 16 MiB, samples 8 MiB,
  metadata acceptance 16 KiB, inherited regular-file RLIMIT_FSIZE 2 GiB, and
  core dumps disabled. Missing worker or terminal evidence remains failure.
- Postflight input checks, bundle inventory, no root deletion and no
  host-restoration claim remain. The normal Sparkle helper's internal timeout
  and captured-output limits remain separately scoped.

This review rechecks prior-bound public interface/configuration bytes. Prior
archive-payload, SDK-header and private qualification checks remain evidence
from the full prior review; I did not repeat them here. The parent's verified
M1 SSH identity, Xcode27.1 / Swift6.4 / SDK27.0 / macOS27.0.1 arm64 baseline,
Python3.13.9 version-helper success, Rust/Cargo1.99.1 version success and offline
11-member metadata result remain supplied preparation evidence. They establish
no compiler build or bundle result for this candidate.

## Residual limits and handoff

Accounting and process observation remain sampled. Root checks do not hold an
open directory descriptor or prove uninterrupted identity between checks.
Replacement followed by restoration between checks, inode reuse, arbitrary
same-user adversarial mutation and symlink races are not excluded. No universal
filesystem quota, outer filesystem-denial profile, universal descendant
observation or hard real-time filesystem/OS-call bound is claimed. Unseen,
short-lived or session-escaping descendants and Xcode/XPC services remain
UNVERIFIED. Normal OS-managed build temp/cache writes outside the owned root
are allowed on this new baseline.

Full or stalled filesystems and failed evidence writes can prevent complete
receipts. Pipe/deadline recovery does not prove evidence completeness. Parent
collection remains separate work with its own actual byte, completeness and
deadline limits. No root cleanup or host restoration is authorized by this
source verdict.

Parent may bind the exact R3 recipe to new concrete paths, the verified complete
Python-prefix digest and a new absolute deadline, then enable the already
authorized ordinary build. No new human approval requirement is introduced.
The supplied prefix digest is
`c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67`
(2,006 entries; 46,504,040 regular bytes). I did not hash the live M1 prefix.
The staged R1 recipe remains execution=false and unexecuted; its old build
root remains uncreated as reported by the parent. No spent root is reset.

The old ordinary result stays FAILED / NOT ACCEPTED, 23/25. Windows native stop
stays FAILED / UNVERIFIED. The expired Sparkle operation stays closed. The old
Swift atomic-write cause stays unproved, with its original predicates intact.
SOURCE CLEAR grants no runtime, native, updater, trusted distribution, release
or full-product acceptance.

No candidate import/execution, AST execution, supplied-helper execution, tests,
mocks, simulations, fixture/probe creation, OS/process/port query, remote/network
command, compiler, resolution, build, signing, product, native or update action
occurred. No delegation occurred. Only this report and its bindings were added.
