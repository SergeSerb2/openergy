# Metadata exit deadline and saved route: independent source/data review

Result: **bounded SOURCE/DATA CLEAR; D4 addressed in source; no P1 or P2 found
within this scope**. This is no approval, executable receipt or runtime acceptance.

Review base: `bd0319df37abcbccfce302e435ff4f125c3b2e96`.
Candidate: `891a443fa29513bc9747e74f6981b8aaebd000e2`.
Scope: the 11 inert files in
`docs/verification/windows-metadata-exit-deadline-route-repair-2026-10-25`.
They match the candidate, imported base, reviewer checkout and parent worktree.
The candidate/import package diffs are byte-exact; unrelated tree differences
are outside this verdict. No source repair was applied.

**D4.** Reader lines 208–218 establish the local start, work and absolute bounds
and the interpreter/platform/monotonic descriptor. Lines 228–234 put them and the
declared budgets in the initial INCOMPLETE receipt. The full checkpoint writes
and fsyncs that receipt at lines 258–275. It runs at line 345, followed by directory
fsync, before the child spawn at line 367. This establishes the initial durable
binding on the normal storage path; it does not prove storage-fault durability.

Driver lines 26–38 require macOS CPython >=3.10 with a finite positive-resolution,
nonadjustable monotonic clock. The direct child uses `sys.executable` at lines
80–95. The resolved interpreter, implementation, version, platform and clock
descriptor must agree at lines 130–140. Numeric bounds and budgets must have
finite numeric types, exact expected allocations and arithmetic, strict deadline
order and a reader start inside the explicit one-second startup interval.

The driver collects the actual reader exit at line 98 and timestamps its
observation at lines 109–111, before receipt/raw-file reads. Lines 141–142 require
that timestamp to be strictly before **both** the bound reader absolute deadline
and the external invocation deadline. This common gate precedes every case,
including expected reader exit 1. Missing/invalid evidence fails. Timeout/reap
handling cannot reach case acceptance. Source, exact command bytes, purpose,
caps, actual reader/child exits and raw stream hashes are also checked.

On D4's path, the final receipt fsync returns after the bound absolute deadline.
The actual reader exit, then its parent's observation, must occur later.
The common gate rejects that observation even if the saved receipt still has
`first_error=null` and reader exit is the expected 1. Earlier receipt elapsed
values and the pre-write Boolean do not decide acceptance. A delayed parent can
conservatively reject an exit that happened on time.

This argument uses the fixed same-Mac direct-parent/reader topology and the
shared clock contract in the parent-verified Python 3.13 `time.monotonic`
documentation (macOS shared since 3.10). No network verification was done here.
Descriptor equality is not clock attestation. Do not compare local Mac, M1,
remote Windows Stopwatch or wall-clock origins.

**Route and retained behavior.** The three expressly allowed private JSON files
were read only as data and match their recorded lengths/hashes. The saved
21-token client prefix before the private selector and six native tokens equals
the candidate constant. Its only changes from the old constant are
`ConnectionAttempts=1`, `ControlPath=none`, and keepalive 10/2 to 5/3; all other
public option order/values match. No selector, alias, endpoint, raw argv or
metadata value is published. Receipt-to-command serialization linkage remains
NOT ESTABLISHED. This is a prefix binding, with no effective-config, default,
transport-variant or route-qualification inference.

The native six-token construction, both closed Windows layer constructors,
six fixed paths and metadata response parser are unchanged. The old saved
query payload differs; the new operation has a different whole argv.
The 87-line query remains byte-exact.

R1's post-pump and terminal capture work checks, sticky first errors, independent
stream/aggregate caps, bounded raw prefixes, exact observed child exits, separate
local settlement and privacy remain. D1/D3 case predicates and the real four-case
tuple are byte-exact to `93cd2e6`. D2's post-wait/read/validation, flow-success
and final CLI-return gates remain. Remote state stays UNVERIFIED and remote
cleanup stays false.

**Bindings and limits.** Both small patches reconstruct the exact full source
as bytes; neither was applied. Both Python texts parse as AST data. All four
review/package checksum lists match. Eighteen public inputs match their
immutable revisions, the review base and parent worktree. The full prior review
packages at `f9aaf96`, `ef9a635` and `6df8c389` are byte-exact and their
reports/limits are reused. The 179 historical paths have no Git delta; their
bodies were not reread. The earlier 50-file and latest 58-file private audits
are inherited from bound public reports/indices, with no new private-byte claim.

The initial local driver flow stays FAILED/INCOMPLETE. The bound `3c52f53`
public report covers the earlier `8f55fbb7...` reader and `8f4f95f0...` final
driver, including four accepted cases and the real closed-output timeout.
It is not new-candidate execution or D4 timing proof.

Reader SHA-256: `15d2a6275d70135857db059bdf1982f5071dd819c3c73ea3ea27e08cd9802bc4`.
Driver SHA-256: `59ba17b43e4414998cc171b897f173a8282c7277908a23636e1d086e8a4f9b59`.
Query SHA-256: `3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`.

**Remaining execution prerequisites.** The candidate remains NOT RUN / UNVERIFIED.
After source freeze, the parent must retain one full real four-case CLI flow on
one qualified Mac interpreter, with immutable source copies, new private output,
actual exits and the deadline/clock/raw-byte bindings. Actual driver exit 0 must
also be independently observed before its 28-second flow deadline under the
authorized external envelope. A saved pass file is insufficient. Requested local
CPython 3.15 and separate M1 CPython 3.13.9 remain unverified. No simulated clock
receipt or unit test is required by this review.

Hard OS preemption, transactional terminal storage, storage/close faults,
nonempty-stderr and competing-cap coverage remain unproved. Windows native
PowerShell/parser/runtime/stop, installed versions, effective limits, historical
preservation and remote exit/current state remain UNVERIFIED. Old metadata stays
TIMED_OUT_UNVERIFIED; original FAILED/UNVERIFIED outcomes and zero native stop
requests remain unchanged.

No supplied source/module/AST was executed or imported. No test, mock, fixture,
helper/native parser, runtime, remote/network action, OS/process/port query,
private write, build, signing, product/credential action or child agent ran.
Only static Git/file/AST/JSON/hash/patch-data checks were used. Review writes are
confined to this new folder. The report commit disables hooks and signing.
No TypeScript changed; compiler/runtime checks are outside this explicit scope.
