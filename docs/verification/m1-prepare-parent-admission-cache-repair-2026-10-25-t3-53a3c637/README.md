# Fixed M1 prepare-parent admission/cache repair

**IMPLEMENTED_UNREVIEWED_R4_REQUIRED. execution=false.** This folder implements
both full-R2 P2 repairs. Independent R4 review of these exact bytes is required.
It claims no independent SOURCE CLEAR, execution authority, runtime acceptance
or human approval gate.

Exact clean starting parent: `078ced42f232509118e038cd3abbc80225e80929`.
Frozen source candidate: `06634b58f1b4795684b9139de716b4fe8f6430e0`, imported at
`9379e2b50b566e3346ec934b322ef142db2ade39`. Full R2:
`f6e15d0803d52905636f9b916b50e0c0a23ef05a`, imported at the starting parent.
Its complete REVIEW.md SHA-256 is
`e50e67ef33aa690525f821fb3d77bf7601a1138740e9ee7c4910b1cfc1129981`.
R2's original HOLD_SOURCE,0P1/2P2 remains attached to its original bytes.

The full inert [source](prepare-parent.py.txt) is **106001 bytes/1872 lines**,
SHA-256 `fce82ca2775607fd86e0ef8dbb8c44fd3cd5a213815ae77cb9501327160ee57d`.
The exact [patch from06634b5](from-06634b5.patch) has four hunks, SHA-256
`3da43718972caf517b490b827e07d6a7cf12d5fec85c1d129bc62b99a3bf8755`.
Its raw baseline is105367 bytes, SHA-256
`3c7c6e639ee64ca2e8ec49b27c993921ee4e7abff90928fd2f498135f2b20135`.
All four hunks reconstructed the complete new source in memory with exact old
contexts. No old file was patched.

Changed functions are exactly `collect`, `import_common`, `main`:

- PREPARE-P2-1: collect retains launch_started_ns at1615, after alarm setup. Its
  prepare predicate at1617 requires that fresh sample+120s+0.5s strictly before
  unchanged work_end, before Popen at1619. The original helper predicate is
  unchanged and remains before its launch sample. The irreversible attempt
  assignment still precedes collect/Popen; any later failure consumes the attempt.
- PREPARE-P2-2: main rejects root __pycache__/common.pyc/common.pyo at1754 before
  inventory/growth/children. import_common rechecks at1435 after all17 pins and
  before SourceFileLoader entry. main rechecks at1792 before the irreversible
  private attempt. No cache deletion/root repair/retry occurs. Exclusive root
  ownership through exit remains mandatory. [CACHE-ANALYSIS.md](CACHE-ANALYSIS.md)
  states exact legacy loader selection and trusted standard-library cache scope.

[FUNCTIONS.md](FUNCTIONS.md), [function-inventory.json](function-inventory.json)
and full [before](changed-functions.before.txt)/[after](changed-functions.after.txt)
bodies account for all43 functions plus Stop. The other41 declaration ranges
retain exact bytes/hashes. [module-layout.json](module-layout.json) proves all40
entry/global/separator/main-guard segments unchanged. No function was added or
removed; no entry/global edit is omitted.

Remote128/work126/reap126.5 and local135=130+2+3 remain fixed. No S/A is renewed.
The exact vectors, ten startup keys, helper/source receipts,17 source pins,
ten modules,212 raw records/213334 bytes and all collector/writer rules remain
unchanged. prepare SHA-256 stays
`16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`;
common stays
`20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c`;
helper stays
`bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428`.
[pin-bindings.json](pin-bindings.json) retains all upstream identities.
[cli-contract.json](cli-contract.json) and the full repaired
[PARENT-CONTRACT.md](PARENT-CONTRACT.md) close the fixed caller interface.

[config.template.json](config.template.json) and [config.schema.json](config.schema.json)
are byte-exact copies of the frozen versions. All unresolved slots remain null;
execution remains false. The template cannot pass bind_config. The one literal
token remains bytes1288..3164, SHA-256
`278ba36ddd0b82a09c6f087ba152d27f15326af071407d7646490e21f74e8f5b`.
Canonical CONFIG stays
`f80fef90503c10fdbabf1a6814b32381fe75ac7b7a7c787ab18f0c184637d1f3`.
[template-proof.json](template-proof.json) regenerates the exact full source,
CONFIG/schema, token, prefix/suffix, normalized AST-data and restoration hashes.
It creates no populated render. actual_future_render remains null. An actual
future rendered payload needs its own complete proof and <=1MiB check.
[finite-plan.json](finite-plan.json) keeps every actual invocation count zero.

Existing artifact DATA is **ARTIFACT_DATA_BOUND** at
`4fbf8f0f32c08fcdfc1f02b3cb2a37b9b79a56ed`: eight136360-byte files and actual
publisher SSH0/collector0/both-EOF/D4 records. The full9374-byte receipt is
`cd3ae8c0e5e985f8352fc204cc247007d8133e262c5381ce710c087eb056818f`;
its guard is1398 bytes. The actual artifact receipt remains33785 bytes, SHA-256
`67daf1812cd935ae420eb10a14755ed65aeb096cdd213ee0c2ea4886731abd53`.
This repair changes none of those records and repeats no operation. Old pending/
unknown-import wording is historical; exact publisher import is
`edde85e8f740f6d33f156ce9795cf1232e3266f2`. Published DATA supplies no runtime
acceptance. V-READER-STARTUP remains UNVERIFIED.

[static-checks.json](static-checks.json) records **69 passing static checks** on
these exact final source bytes: raw four-hunk reconstruction; complete AST data,
whole-function and outside-function comparison; exact clock/vector/status/error/
KILL/writer retention; cache placement; strict actual JSON/schema traversal;
null non-admission; all source/module/raw/input/actual-public-DATA identities;
and full prior working-byte/mode preservation. No check calls supplied code.
[input-bindings.json](input-bindings.json) verifies95 public Git records,
including all38 original inputs and full candidate/import/review/DATA packets.
[bindings.json](bindings.json) retains precise source/review/artifact/library
identities. [manifest.json](manifest.json) binds every payload file; SHA256SUMS
binds that manifest and every other file except itself.

Only this new folder is owned. [baseline-tracked-inventory.json](baseline-tracked-inventory.json)
records all4298 old tracked Git blob/mode identities and exact working bytes/
filesystem modes. One existing `.cmd` has its required CRLF checkout conversion;
both raw Git and working identities are preserved separately. An initial own
read-only baseline assertion stopped on that conversion before any writes; the
corrected reader recorded both forms. This was no supplied-code/runtime attempt.
The folder-only commit disables hooks and signing. Final commit/tree/scope and
clean-checkout results are reported outside this self-contained packet to avoid
a self-referential commit hash.

No supplied capsule/helper/preparer/collector/checker/private Python was run,
imported, evaluated or compiled to executable code. No private-state read,
network/remote/native action, test/mock, build, child agent, signing, PR/push,
monitor or TypeScript change occurred. The task expressly excludes CLI/E2E work.
The last launch sample cannot prevent a later scheduling/OS stall. Root absence,
exclusive ownership, remote/local causal enclosure, real exits/EOF/D4, NI15,
copy/native/UI/performance/product/release acceptance remain unobserved here.
[LIMITATIONS.md](LIMITATIONS.md) preserves strict SDK FAILED_UNVERIFIED, old M1
FAILED_NOT_ACCEPTED, port cause UNKNOWN, NI15/runtime UNVERIFIED, Windows SSH255/
noRoute FAILED_UNVERIFIED/unknown remote state and updater CLOSED.
