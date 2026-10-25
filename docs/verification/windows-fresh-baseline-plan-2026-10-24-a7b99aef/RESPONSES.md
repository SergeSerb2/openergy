# Implementation responses — source only

The source base is reviewed R9 `762240ac041a835707b7ae3867b4e15b1e06b8c0`,
read with the R10 review, R7 design/plan/responses, R8 review, R6 findings,
release-readiness handoff, and bounded fixture-recovery reports. Input bindings
record exact immutable bytes. This is a new candidate; R10 does not clear it.

**Replace the missing historical selection completely.** The controller no
longer consumes `baseline-selection.json`, the nine old labels, or old
TEMP-relative paths. Common reads `fresh_roots`; qualification and postflight
drop the old failed-root absence gate. The old e1ef124/module/archive defaults
are replaced by exact current-source bindings. Local published-evidence hashes
remain a separate unchanged-history check. They do not query historical hosts.

**Make the new scope explicit.** New schema fields permanently say historical
preservation UNVERIFIED and original runtime FAILED_UNVERIFIED, with zero original
stop requests and no changed history. Fresh roots have only new `fresh-<id>`
labels, nonempty sentinels, and their own before/after state. Existing registry,
toolchain, selected default/tool state and sharing identities are recorded as
current preflight state. No old path absence/equality is asserted.

**Preserve containment and cleanup.** The R6 root/drive/absolute/parent checks
remain in admission, now tightened to one safe operator-bound child component.
Windows setup normalizes and validates actual TEMP before creation, checks every
ancestor for reparse points, creates one fresh private direct child, and binds
nonce/SID/ACL/numeric creation file time. The new safety module walks one directory
level at a time and refuses a reparse child before descent. Cleanup validates
that identity and removes only individual entries in this exact new root.
Process references refuse deletion and never authorize killing unknown processes.
Setup failure attempts bounded owned cleanup. Known-root later failures attempt
collection and one bounded retention audit if collection/removal fails. Unknown
ownership, incomplete collection, timeout or storage faults remain unresolved;
the source does not claim guaranteed restoration on every external failure.

**Keep R9 compiler ownership.** Build changes only its receipt's source-commit
expression. Direct Process.GetProcessById and narrow missing-PID catch remain.
Registration, first retained creation handle, strict descendant verification,
600-second work, shared 30-second settlement, 630-second receipt ceiling,
660-second outer admission, first-error retention and independent closure remain
exact. No source change adds kill authority, compiler retry, or Clippy command.

**Keep the actual R2 flow.** The observer is exact R9/R2 bytes. Runtime changes
only its source-commit admission expression. Its remaining tail is byte-exact
R9, including all suspended-child identity, reader sharing/bounds, real
version/negative/healthy/config/invalid/stop commands, serialized IDs, ACK,
independent retained-handle exit 0, full ten-second normal-stop budget and
15-second owned recovery. Forced exit remains failure. Common's new ownership
check runs before product creation. No unit test, mock, fake PID or substitute
response was added.

**Bind current source freshly.** Future local binding requires exact clean HEAD,
full tree, exact export member set and Git blobs, archive/manifest bytes/hashes,
line endings and safe regular archive members before transport. Qualification,
build and runtime carry that same state binding. Historical e1ef124 is refused
as the current default. This package records ed7d96bf as a static reference only;
after integration, the parent must bind its then-current exact clean commit and
archive before a real future run. The chosen route is one pinned fresh release
build with jobs 2, locked/offline and no Clippy repeat.

**Keep finite work and evidence.** The original single 2,400-second clock and
30-second final transport reserve stay. All original stage ceilings and one-shot
counts stay. New setup, tree/source bounds and failed-collection audit are
specified in plan.json and NEXT-RUN.md. Successful removal still follows private
archive verification. The new collector also verifies flat unique members,
exact member set, current state bytes, decoded before/after equality and runtime
pass. Failure preserves first error, raw receipts and root retention status.

**Keep this task inert.** Complete texts, exact patch, schemas, current Git
snapshot, input/source/package bindings and static reports are materialized only
in the new docs folder. No source/helper/AST was executed or imported; no
PowerShell/C# syntax check, OS/transport probe, build, runtime, signing, unit test,
mock, private-selector read, credential access, delegation or publication ran.
Native syntax and all actual acceptance remain UNVERIFIED. Independent whole
candidate review is the next step, not an already-completed result.
