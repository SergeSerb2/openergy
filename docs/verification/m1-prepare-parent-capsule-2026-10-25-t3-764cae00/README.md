# Inert fixed M1 prepare parent capsule

**IMPLEMENTED_UNREVIEWED. execution=false. Independent review is next.**
No human gate, execution permission, publication or runtime result is created.

Exact parent: `05b056cdc25a137f820efb59ae5004ce98781770`.
Only this new folder is owned. All prior tracked blobs and modes remain exact.

The complete inert [capsule](prepare-parent.py.txt) is 105,367 bytes / 1864 lines.
SHA-256: `3c7c6e639ee64ca2e8ec49b27c993921ee4e7abff90928fd2f498135f2b20135`.
The exact [patch](from-reviewed-source-payload.patch) uses the cleared full SOURCE
payload at f78f2be/import920eac7 as its baseline, SHA-256
`8d7844a20ce347b7be52c07a516b8d9136bdb5c067e450bea24eb3453441afa6`.
The baseline supplies bounded retained-handle collection and exit/EOF/KILL rules.

The large diff is intentional: metadata SOURCE invocation is replaced by one
SYSTEM entry helper and one unchanged private prepare invocation. The capsule
adds fixed artifact/source/raw admission and a strict INITIAL-to-R9 writer
transition. It removes metadata parent-record generation and the old unmetered
post-child publisher. It does not rerun SOURCE or50 native metadata commands.
The full [function inventory](FUNCTIONS.md) contains 43 functions, including
nested functions, plus the unchanged Stop class. Relative to the baseline it has
5 changed, 18 added and 21 unchanged
whole function/class ranges. All unchanged hashes and old/new ranges are retained
in [function-inventory.json](function-inventory.json).

The immutable prepare SHA-256 remains
`16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`.
The common writer SHA-256 remains
`20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c`.
The helper SHA-256 remains
`bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428`.
No edit to those files, the controller, product source or prior data occurred.

Read [PARENT-CONTRACT.md](PARENT-CONTRACT.md) for the full fixed entry, required
actual parent inspections, exact vectors/environment, original time envelopes,
once guard, safe common import/conversion, failure publication, new prepared
bytes and external SSH/collector/EOF/D4/retention predicates. It names the exact
remote/local causal-enclosure proof gap. The capsule has no local observer or
cross-host clock bridge. No latency assumption or new allowance fills that gap.

[config.template.json](config.template.json), [config.schema.json](config.schema.json)
and [finite-plan.json](finite-plan.json) stay false/null and non-admitting.
[template-proof.json](template-proof.json) records the current literal/prefix/
suffix/hash/AST-data/restoration proof. It is not a future rendered payload proof.
[pin-bindings.json](pin-bindings.json) retains frozen17, initial20, completed7,
212 raw identities, fixed8 and exact public source/review bindings. The pending
8 files total136360B and receipt SHA remains
`67daf1812cd935ae420eb10a14755ed65aeb096cdd213ee0c2ea4886731abd53`.
Neither pending DATA nor publisher is cleared or published by this folder.
The requested publisher import fragment de85e did not resolve; original95404dc
was inspected from public Git and no import identity was inferred.

Actual validation used public Git/files, raw modes/hashes, strict JSON data,
Python3.9 AST data, whole-function comparisons and exact in-memory patch
reconstruction. [static-checks.json](static-checks.json) retains197 checks on the
first source snapshot. [final-source-static-checks.json](final-source-static-checks.json)
retains15 focused checks after guarded-import and receipt-metadata edits,
plus unchanged whole-function proof for the other prior checks. Both snapshots
and their exact hashes remain distinct. [input-bindings.json](input-bindings.json)
binds38 public Git inputs. [SHA256SUMS](SHA256SUMS) covers all other folder files.

No supplied/proposed capsule, supplied checker, preparer, common module or AST
was executed, imported or compiled into executable code. No private/state file,
target, SSH, network, native/process/port/GUI action, test/unit/mock, build,
signing, credential, child, PR, push or monitoring ran. No TypeScript changed.
The explicit SOURCE-only commission excludes E2E execution. Commit hooks and
signing are disabled for the one folder-only commit.

No actual clock equivalence/rate, remote/local enclosure, latency, pipe delivery,
exit/EOF/D4, NI15 continuity, signal/reap, hard preemption, fault/power durability,
copy-tree/signature/entitlement/native qualification, ordinary runtime, UI,
performance or release acceptance follows. This capsule starts no product flow
and grants no lifetime past flow_start+180 or watchdog cancellation.

Preserve strict SDK FAILED_UNVERIFIED helper1/SSH1; old M1 app25/25 app0/
controller1 FAILED_NOT_ACCEPTED; earlier23/25 FAILED; post65431 EADDRINUSE cause
UNKNOWN; whole-flow NI15/runtime UNVERIFIED; all old failed files/guards; missing
original second post-bind, bundle-after, outcome/archive; stripping warning;
unknown complete outer build duration/exit; Windows SSH255/noRoute
FAILED_UNVERIFIED/unknown remote state; and the CLOSED updater operation.
