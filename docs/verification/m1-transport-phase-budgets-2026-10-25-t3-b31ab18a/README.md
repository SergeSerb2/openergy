# Inert M1 transport closed phase budgets

2026-10-25, America/Vancouver. **IMPLEMENTED_UNREVIEWED. execution=false.
source_clear=false. runtime_acceptance=false.** Independent source review is
next. This implementation creates no review verdict, execution authority or
human approval gate.

Exact clean parent: `04444ec2dce5dd96334d807f9c2caff86aefae64`. Only this new
folder is owned. Every older tracked blob and mode must remain unchanged.
No supplied or proposed source was executed, imported, compiled or parsed as
AST. No runtime, test, unit, mock, failure injection, private/state access,
network/SSH, process/port query, native/GUI, build, signing or credential action
ran. No child agents, PR, push or monitoring were used.

[transport-collector.py.txt](transport-collector.py.txt) is the complete inert
collector: 28,060 bytes, 626 lines, SHA-256
`695dc01461b31e04eb7b8c4a12848350f26a45731f05b0ddca971c05749e4df6`.
[collector-from-04444ec.patch](collector-from-04444ec.patch) is the exact
baseline-to-new unified patch: 2,000 bytes, SHA-256
`a20c3afc4caa940d49b7e29ccf54f27cca0bfeba18ecb078fddff5abf487e82c`. Its baseline is the 27,834-byte R4 collector at the exact parent,
SHA-256 `d312814e74c6f42e2a86d56648450dc145531df6292ea8c79ccf1768f34b14e4`.
The baseline's SOURCE CLEAR report is `a7e22725cbbf07b5e6ac7e7aa7e7fb0d3dff93b4`,
imported at `81252214f7c391c465510193acf48115ec3d95db`. That verdict applies
only to the baseline.

Exactly three functions change:

- `arguments` admits exactly `metadata-source`, `metadata-copy`, `prepare` and
  `controller` through the new fixed `PHASE_WORK_SECONDS` table.
- `Collector.__init__` selects work seconds from that table and derives work,
  settle and absolute ends from the original parent start.
- `Collector.deadlines` records selected work/total seconds in the existing
  deadline record shape. Existing guard and receipt calls use that function.

Both metadata phases and prepare use exactly 130 work + 2 settle + 3 final =
135 seconds. Controller uses exactly 335 + 2 + 3 = 340 seconds. Work ends at
S+130/S+335, settle by S+132/S+337, and final publication by S+135/S+340.
The nine mandatory flags remain exact. No arbitrary duration or new CLI flag
is added. All other 22 complete function ranges remain byte-identical,
including `Collector.settle`, `Collector.run`, capture and every I/O helper.
All other source bytes match after excluding those three functions and the
single added phase table.

The controller340 envelope is already prescribed in the exact public
[next operation](../m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/next-operation.json)
and full operator/receipt contracts. It includes 120 seconds of file admission,
120 seconds of flow, at most60 seconds of cleanup, and evidence. Product life
still ends at `flow_start+180`. Internal clocks, prepare/controller/common/
watchdog/caller and payload logic stay unchanged. The detached watchdog must
never be canceled. This folder performs or simulates none of those operations.

The transport still treats its pinned payload as opaque bytes. It keeps the
exact reviewed 29-token SYSTEM stdin SSH route, input/stream/record caps,
no-follow ownership/one-link/stable-read checks, exclusive durable guard and
root-parent fsync, retained payload snapshot FD, actual status rules and the
post-poll eligibility deadline gate. Only the exact retained LOCAL SSH handle
can receive KILL on cap/deadline within the selected original envelope. Remote
exit/state stay null/UNVERIFIED unless separately evidenced. No remote/product/
group signals, watcher, restart, retry, drain or guard reset are added.

D4 remains external: the parent retains the actual collector OS exit and
immediately samples the same running local Mac clock strictly before the
original selected absolute end, BEFORE any receipt read. Late or unknown D4
rejects even a COMPLETE receipt. The new source manufactures no actual exit,
D4 or native proof. This local raw adapter is not the R9 reserved writer;
full raw/caller records, encoding, receipts and publication accounting still
use their exact existing limits and reservations.

The parent already records one actual DATA and one SOURCE pass: all53 children
actual0, all50 saved basis matches, all212 raw/status/comparison files retained,
collector0/SSH0/both EOF/D4. This task checked those public records as data; it
did not reopen private evidence. That result does not accept ordinary runtime.
No prepare, copy or controller evidence is supplied by this extension. All
historical FAIL/UNKNOWN/UNVERIFIED labels, warnings and the closed updater
operation remain, as listed in [finite-plan.json](finite-plan.json).

[FUNCTION-CHANGES.md](FUNCTION-CHANGES.md) contains all six complete before/after
function blocks. [function-bindings.json](function-bindings.json) inventories
all25 functions and every unchanged function hash.
[phase-budget.contract.json](phase-budget.contract.json) specifies closed CLI,
phase arithmetic, full record fields, environment, caps and publication limits.
[parent-observer.contract.json](parent-observer.contract.json) supplies the
updated inert D4 interface. [caller-summary-limit.json](caller-summary-limit.json)
is byte-identical historical size/integration data.
[bindings.json](bindings.json) pins 18 public inputs to exact Git blobs/modes,
the original/imported review, and the new core artifacts.
[AUTHOR-OBSERVATIONS.md](AUTHOR-OBSERVATIONS.md) records source reasoning and
proof limits. [static-checks.json](static-checks.json) records actual author
checks. [static-reader-errors.json](static-reader-errors.json) retains the first
author reader failure: an incorrect hard-coded37-field count, corrected to an
exact comparison of all33 unchanged receipt fields. Collector and contract did
not change for this correction. [SHA256SUMS](SHA256SUMS) covers every other file, with no self-hash.

Only data checks were authorized: exact Git identities/modes, unique-key finite
JSON, lexical full function ranges/hashes, unchanged bytes, fixed budget data,
complete function blocks and unified-patch byte reconstruction. No Python
syntax/AST or runtime qualification is claimed. Timing, actual while-read cap
behavior, OS stalls, signal delivery/reap, actual D4, filesystem fault/power-loss
durability, future private route/payload integration, reserved publication and
ordinary/native/artifact/release acceptance remain UNVERIFIED. Review remains
next; no human gate is created. Commit hooks and signing are disabled.
