# COPY-only direct-signal metadata caller

2026-10-25, America/Vancouver. **IMPLEMENTED_UNREVIEWED. SOURCE/DATA ONLY.
execution=false. Independent fresh source review REQUIRED.** No source CLEAR,
runtime acceptance, execution authority or new operator approval is issued.
Only this new folder is owned, at parent
`4ea32a7538aa1b996e60fe7dd185497cbba75356`.

The complete inert [caller](metadata-copy-caller-direct-signal.py.txt) uses
the future fixed leaf `metadata-copy-caller-direct-signal.py.txt`. It must stay
beside the old `metadata-caller.py.txt` in the SAME existing ROOT when a later,
separately reviewed composition publishes it. **It was not materialized.**
Do not add it to ROOT before prepare: the existing prepare admission requires
the exact original 260 names, then 264 with its four durable outputs. Do not
overwrite the old caller, reset ROOT, retry prepare or repeat SOURCE.

[input-bindings.json](input-bindings.json) links each frozen public input by
full commit, repository path, Git blob/mode, length and SHA-256. Key inputs:

- R9 candidate `9b50dd7e3a672eafe319be33141cb875d54a471e`,
  [source](../m1-metadata-r9-repair-2026-10-25-t3-9b2c2354/metadata-caller.py.txt),
  46,908 bytes, SHA-256
  `01623cd4759f96cd0066a9456a2680bff476bc37b5b1acb61ec40388582a5e9f`.
- Full R10 `1c0e4bcdc2dddcb64a3277db34d5ccd5e872fab2`,
  [report](../m1-metadata-r10-review-2026-10-25-t3-a1a624d3/REVIEW.md), SHA-256
  `ad868577561145dcba9e74db7138e5118544e721ca3629f5c0c4f1808203c496`.
  Its historical R9 verdict does not clear this new caller.
- Full COPY review `f33a1fd761770cb2a2961f9cf55b59dc4aeb40d5`, import
  `931d2c3364e177abd6615d5bd26811d9d854f8a8`,
  [report](../m1-copy-parent-full-review-2026-10-25-t3-9edf92e5/REVIEW.md), SHA-256
  `33550d9e4fca61a4425eadfe095b8a8234a825a1978b1a263852e3dac365d53f`.
  It identifies the extra internal Popen.kill poll, possible silent signal
  skip, and the unsigned failure-exit schema defect.
- Current COPY repair `c82cc12b8a2d709d8ce1e384545475159b24d2a9`, import
  `6ef1663caee7ec8e421daaaf9effca6796ecedf0`,
  [source](../m1-copy-parent-signal-schema-repair-2026-10-25-t3-f3ffc1d7/copy-parent.py.txt),
  120,718 bytes, SHA-256
  `94efc9b4de615485827e471c3e3f31250b28ea64f35228955b91154e4ba2a22a`.
  It has not run. Its own collector repair does not compose this alias.

The [exact zero-context patch](from-frozen-r9.patch), full changed-function
[before](changed-functions.before.txt) and [after](changed-functions.after.txt),
[function inventory](function-inventory.json), [global diff](global-diff.patch)
and [global inventory](global-inventory.json) account for all changed bytes.
Only `collect` and `main` change. All other function/class bodies, including
the nested alarm handler, stay byte-exact. Reversing the seven exact edit
segments restores every R9 source byte. The module header and one new constant
are the only global changes; imports and the `os._exit` entry stay exact.

`collect` retains the final actual poll, immediate shared CLOCK6 sample,
actual-code-before-clock reap handling, both strict original ends and bounded
wait. It uses `os.kill(proc.pid, signal.SIGKILL)` once, only for an overflow or
timeout, with a retained unreaped child and a timely None eligibility result.
`kill_syscall_returned` becomes true only after that call returns. It proves
no delivery, exit or reap. ESRCH and all other errors follow the original
failure path; actual exit still comes only from the retained returncode.
No extra poll, OS/PID query, group target, drain, process or time is added.
See [SIGNAL-AND-SCHEMA.md](SIGNAL-AND-SCHEMA.md) for exact limits.

`main` now accepts only the same five-argv interface with mode `copy` and the
exact new leaf. It rejects SOURCE before root/clock work. The old SOURCE
branches remain unchanged dead code after that gate. NEW COPY full results
use `openergy.metadata-observations.v3`; saved SOURCE still requires old v2.
`fresh_source` and generic `complete` are byte-exact. The latter never reads
`kill_delivered`, so the successful old SOURCE data remains usable.

The public [saved SOURCE validation](../m1-source-metadata-observed-2026-10-25-t3-52422617/validation.json)
records 53 actual zero exits, no KILL, full result SHA-256
`7482a637159e0389c5cd4c505778919bb49354437bc9feb1cf008d9db395712d`,
and 212 raw/status/comparator references. Its complete raw manifest and six
boundary identities remain in the unchanged frozen parent. This task checked
those public identities only. It did not reopen the private evidence.
[compatibility-observations.json](compatibility-observations.json) binds the
saved public data and the current parent's incompatible consumer functions.

All 52 COPY observations stay: copy-files-before, ordered native50,
copy-files-after, with 208 durable raw/status/comparator refs and both actual
native comparisons against saved basis AND saved fresh SOURCE. Writer, regular
R9 reservations, 17 pins, ten modules, vectors, completed-file guards, full
prefix authentication, COPY startup10, helper command10, COPY120, shared
CLOCK6, original S/A and stream1MiB caps stay exact. Parent stdin still has
exactly seven keys and authentication19; source_file_qualification stays null.
No private writer or common.command call is added. Full result and compact
success512 construction stay exact except for the named v3/key changes.
The longer status key uses existing capacity; no new byte allowance is granted.

[closed-cli.json](closed-cli.json) and
[child-status-v3.schema.json](child-status-v3.schema.json) are closed DATA
contracts. Signed actual/eligibility exits and nulls are retained. Clock, PID
and byte domains remain nonnegative; success still requires actual0. These
are schema-domain observations, with no synthetic receipt or mock test.

[PARENT-DELTA.md](PARENT-DELTA.md) specifies the required later composition.
**Materialization, exact parent integration and actual run remain BLOCKED**
until separate exact parent/materializer composition review and the actual
one-shot run. The false [plan](finite-plan.json) keeps future review,
materialization, composition and result slots null. No materializer or parent
implementation is supplied here. The current COPY parent cannot launch this
caller as-is.

[baseline.json](baseline.json) retains parent/tree/count and canonical opening
inventory digest. [preservation.json](preservation.json) checks every old blob,
index entry and actual working bytes/modes, including the existing CRLF file.
It does not copy the large historical inventories. The author-owned static
reader records [observations](static-observations.json), and
[SHA256SUMS](SHA256SUMS) binds every other packet file. Hooks and signing are
disabled for the folder-only commit; final commit/scope/clean status is reported
outside the packet to avoid self-reference.

Only public Git/text/strict JSON/hash/AST-data readers ran. No supplied source
or checker was executed, evaluated, imported or compiled to executable code.
No private/state, remote/network/native/runtime, test/mock, build/signing,
target child, delegated agent, PR or push action ran. Explicit SOURCE/DATA
scope excludes CLI/E2E execution; no TypeScript changed.

Preserve SDK FAILED_UNVERIFIED helper1/SSH1; M1 app25/25 app0/controller1
FAILED_NOT_ACCEPTED; earlier23/25 FAILED; first post65431 EADDRINUSE cause
UNKNOWN; NI15/runtime UNVERIFIED; missing original second post-bind,
bundle-after/outcome/archive; stripping warning and unknown complete outer
build duration/exit; Windows SSH255/noRoute FAILED_UNVERIFIED and unknown
remote state; all failed evidence and guards; updater CLOSED. Frozen
common.command, port_diagnostic and old SOURCE signal branches remain separate
audits. No signal/reap success, hard deadline, private startup, clock-rate,
crash durability, D4, native COPY, signature/ABI, UI, release or runtime
acceptance follows from this packet.
