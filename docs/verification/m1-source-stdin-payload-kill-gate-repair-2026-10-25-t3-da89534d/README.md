# Inert SOURCE stdin payload: post-poll KILL gate repair

2026-10-25, America/Vancouver. SOURCE/DATA ONLY. execution=false.
IMPLEMENTED_UNREVIEWED. A new independent review is required before use.
No SOURCE CLEAR, execution grant, caller acceptance or human gate is supplied.
Only this new folder is owned. Its exact parent is
`0dad1c00d0a27f8266ed8e138f780a9d94656cdc`.

[source-payload.py.txt](source-payload.py.txt) is the full inert replacement.
[payload-from-fa72.patch](payload-from-fa72.patch) is the exact three-hunk
unified patch from candidate `fa72a0d865f12ae94b91f84400a31f51026c5b16`,
import `dde69c794ae6b547393c81a3de411b6c9af1d179`. The baseline source is
37,122 bytes, SHA-256
`dc208d02222c92418046139ecf622c797881af46b3edc1b8a891742b22f9fd9f`.
The patch was reconstructed in memory only. No old file was edited.

The input independent review is
`4b5d756769cdc540abd7f07c92fead31b9530101`, import
`cac1b01`,
[`REVIEW.md`](../m1-source-stdin-payload-review-2026-10-25-t3-de175193/REVIEW.md).
It is HOLD for PAYLOAD-P2-1. This implementation responds only to that finding;
it does not close the finding on the reviewer's behalf.

## Narrow source change

`observe_exit`, lines 428-435, now returns the immediate shared-clock sample
for every successful actual poll/wait return, including `None`. It still
stores the first real exit and real reap before sampling. A clock failure
cannot turn a real returned code back into null. An existing real exit stamp
is never replaced. `None` cannot supply an exit or reap.

`collect`, lines 438-549, retains the eligibility poll's returned value and
sample in `kill_eligibility_exit` and `kill_eligibility_observed_ns`. These two
status fields start null. After a returned `None`, KILL is permitted only if
that sample is strictly less than both the unchanged `reap_end` and `end`.
Equality is late. A late sample sets `timed_out` and
`unread_or_dropped_unknown`, then raises `TimeoutError` before changing
`kill_attempted` or calling KILL. The existing handler records the settle
error and preserves the first collection failure. An actual real code and
its observation remain retained even if the surrounding deadline fails.

The actual `proc.wait(timeout=remaining(reap_end))` is byte-exact. There is
still only one retained direct SYSTEM caller handle. There is no new poll,
signal target, retry, drain, clock allowance or timer change. All other 26
functions and every byte outside these two top-level functions are exact.
[function-inventory.json](function-inventory.json) records all 28 functions,
their old/new hashes and exact ranges. [repair-contract.json](repair-contract.json)
records the ordering and retained boundaries.

The predicate is a source ordering repair. Scheduling can still delay code
after a sample. Poll, clock, signal, wait and other OS calls can stall. This
package makes no hard preemption, guaranteed signal time, timely reap,
containment or runtime claim. A clock failure leaves the existing failure and
null/unknown rules in force. Null reap still blocks all later file writes.

## Retained contracts and pending dependencies

[inherited-controls.md](inherited-controls.md) is a byte-exact copy of the
baseline's full README. Its source controls remain applicable. Its old R5/R6
progress notes and old package identity describe that historical package;
this README supplies the current package identity and dependency status.
The schema and null CONFIG are byte-exact copies. No source CONFIG token was
changed. All 22 caller/review/hash/prerequisite slots stay null.

Caller R7 is context only:
`13fccbe1d547198d5025748d20af5fb5c3a8c287`, import `0dad1c0`, source SHA-256
`c9a3b1bcfae0da2eced035c2c9e4048a92dc18340c940148a486cf12348b0987`.
Its compact success receipt interface is separately unreviewed. The payload
still captures stdout as opaque bytes. It neither parses that receipt nor
changes capture based on its shape. The parent must bind the final caller
and independent review, then validate its full retained result and all 53
SOURCE command records through the reviewed interface. An identity comparison
of CLI, parent schema, pins or vectors is not caller acceptance.

Transport repair `1c46c16b225924d6664d33c06382a3cd05581272`, import
`afb68e5`, also awaits independent review. No transport source or route was
changed. The local limits stay stdout128KiB, stderr64KiB, combined192KiB and
outer135s. Caller capture stays1MiB combined during reads; console stays at
most16KiB. The original shared120s phase includes parent/guard publication,
spawn, capture, kill/reap, accounting, file publication and console. Work
ends at phase-end minus2s; reap ends at minus1.5s. There is no added time.

The only future root remains the existing M1 root ending `4b20357c1503`.
There is no new root or qualification retry. Real parent inspection of all
seven originals and the exact remote five-file verification precede NEW
`inspected_ns`, then NEW phase start. Old clocks stay in their original
domains. Parent v2, 17 pins, ten modules, 50 vectors, exact SYSTEM `-I -S -B`
CLI/cwd/ten-key environment and caller-owned SYSTEM entry-template remain.

All SOURCE evidence remains INITIAL: exclusive nofollow0600, one raw write,
short-write rejection, file/root fsync, no ledger. The8MiB file/32MiB total
limits retain16MiB product,4MiB cleanup and64KiB accounting. Root-only counting
stays bounded. There are no wrapper writes while the caller lives or is
unreaped. After actual reap, every later write re-counts caller-created files.
Raw/status prefixes, actual exit/null, EOF/drop uncertainty, first failure,
separate publication errors and partial durable references remain unchanged.
The console remains one nonblocking raw write followed by `os._exit`, with no
shutdown retry. No private interpreter entry, PID lookup, group/product/
watchdog signal, COPY, prepare, controller, artifact constructor, materializer
or SSH implementation was added.

## Static evidence and limits

The author-owned [static-reader.py](static-reader.py) reads source as AST data,
strict JSON, local Git blobs, hashes and patch text. It does not import,
compile to executable code, or invoke supplied source or supplied checkers.
[static-checks.json](static-checks.json) records its actual static results.
[source-bindings.json](source-bindings.json) records exact public input
identities and null final bindings. [SHA256SUMS](SHA256SUMS) covers all other
files. [finite-plan.json](finite-plan.json) remains false with zero execution
counts and null review/clearance fields.

Static checks cover exact baseline/import/current identity; inherited 44
public declarations and17 pins; ten-module names;50 ordered vector identity;
R7 parent/CLI/env data equality; AST syntax and function preservation; strict
JSON and closed null CONFIG; post-poll branch ordering; fixed deadlines/wait;
in-memory forward/reverse patch reconstruction; and file hashes/modes.
These checks are not an independent source review or behavior proof.

The full staged `git diff --check` returns2 for two blank context lines in
`payload-from-fa72.patch` (lines9 and10). Their single space is the required
unified-patch context prefix. Those bytes remain exact. The separate check
of all files except the literal patch returns0. Both results are retained in
`static-checks.json`; the full whitespace check is not reported as passing.

No supplied source/checker execution or import, tests/unit tests/mocks,
private-state access, remote connection, native command, process/port query,
child agent, build, signing or activation occurred. No TypeScript changed.
Only the new folder is committed, with hooks and signing disabled.

Old M1 app25/25 app0/controller1 FAILED_NOT_ACCEPTED, strict SDK
FAILED_UNVERIFIED helper1/SSH1, first strict post65431 EADDRINUSE cause UNKNOWN,
whole-flow NI15 UNVERIFIED, earlier23/25 FAILED, all historical failures and
original files/guards remain. The original second post-bind, bundle-after,
outcome/archive gaps, stripping warning and unknown outer build duration/exit
remain. Runtime, native/UI, artifact, ABI, signature, performance, release and
integration acceptance remain UNVERIFIED. A new independent review is required
before use; no clearance declaration is manufactured here.
