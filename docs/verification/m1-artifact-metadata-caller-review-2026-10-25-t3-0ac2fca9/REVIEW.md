# Independent finite metadata caller SOURCE/DATA review

2026-10-25, America/Vancouver. **HOLD: two concrete P2 findings. No P1 found.
execution=false. Runtime UNVERIFIED.** No source repair or machine operation
is part of this review. This report adds no human approval gate.

Candidate is `621e0e3a9037e15b391e05b964dfcdb25da01515`, imported as parent
`d1a0f109606603695c2ffaec3afa32143e4a6205`. The checkout started clean at that
parent. All ten candidate files equal the candidate Git blobs byte for byte.
Only this new review folder is owned or changed.

Locations below refer to
`../m1-artifact-metadata-caller-2026-10-25-t3-43cef465/metadata-caller.py.txt`
unless another file is named. I read all 529 caller lines, the full README,
finite plan, pins, vectors, bindings and coordination data. I also read the
referenced preparation README, helper and recipes, next-operation, templates,
full operator plan and receipt contract. I inspected common's writer contract
to assess this caller's use of it. I did not re-review unchanged controller
logic, the independently cleared file helper, or the canonical SDK data.

## P2 R1: Give cross-process times an explicit, compatible clock domain

Locations: caller **442-444**, **219-222**, **471-477**, and **47-72**;
candidate README **87-94,120-126,168-173**. The defect enters at the phase-start
interface and also affects the prior-helper ordering checks.

The parent supplies a `time.monotonic()` start. Line443 compares that number
directly with this process's clock. Line444 then uses `start+120` as a local
deadline. Line222 compares prior helper times with the caller's current time;
lines472 and476 compare the old observation with the phase start. The schema
has no clock identity or conversion binding.

The supplied target premise is SYSTEM Python3.9 on macOS and private
Python3.13. CPython3.9.6's macOS implementation initializes a static `t0` in
each process and subtracts it from Mach time. Thus separate3.9 processes have
different origins. This is directly visible in the official
[CPython3.9.6 source, lines772-821](https://github.com/python/cpython/blob/v3.9.6/Python/pytime.c#L772-L821).
The official [Python time documentation](https://docs.python.org/3.13/library/time.html#time.monotonic)
records the macOS cross-process change in3.10. The
[CPython3.13.9 source, lines1074-1094](https://github.com/python/cpython/blob/v3.13.9/Python/pytime.c#L1074-L1094)
converts the Mach tick value without the3.9 process-local subtraction.

Consequently, same host does not make the imported numbers comparable. A
longer-lived3.9 parent can supply a start greater than a newly started SOURCE
caller's local reading, so line443 rejects a valid phase. A prior qualification
can similarly have a finish greater than the new caller's reading even when
it completed first. A small3.9-origin start supplied to COPY3.13 yields an
already-expired local deadline on an ordinarily running host. Other offsets
can falsely admit ordering or give more than the parent's remaining budget.
These are deductions from the source, not newly executed target outcomes.

Minimal repair: define and bind an explicit shared clock for NEW cross-process
observations, then safely convert its remaining phase time to each consumer's
local clock where needed. In particular, common's unchanged `io_until` expects
its own `time.monotonic()` domain; passing an arbitrary shared-clock absolute
value straight through is insufficient. Alternatively, let the parent establish
causal ordering and retained within-process elapsed limits, while carrying a
conservative, bound remaining phase budget to the caller. SOURCE and COPY
must still include entry authentication, collection, comparison, publication,
kill and reap in their original120s phases. Keep the10s command limit.

Do not retag old `~0.x` times with a new clock identity, invent an offset, reset
the120s on caller entry, remove ordering predicates, or repeat qualification.
The parent-reported full source-file qualification keeps its own process-local
120s result. This finding concerns consumption of that result and the NEW
metadata phase; it does not revoke that earlier reported pass.

## P2 R2: Preserve a collected command failure before raw publication

Locations: caller **375-381**, **399-407**, **504-509**, **514,519-522**.
The candidate README's first-error retention claim is at **187-190**.

`collect` returns the actual status and timeout/EOF/overflow/kill facts at
line376. The next three operations write stdout, stderr and the OS record.
Only after all three succeed can `first_failed_comparison` be assigned at
lines403-404. The failed `perform` call also never returns an item for the
result's native or auxiliary record list.

For a concrete source path, let collection return a nonzero actual exit or a
timeout, then let a raw write or its fsync fail. For example, an exhausted
regular reservation raises at the first `writer.put`. That exception skips
the sticky failure assignment. `main` records only the publication exception
as `first_error`; its preservation branch sees no failed comparison. The
fallback direct-child list retains only PID, actual exit and a handle flag.
It loses the command label and the already-observed timeout/EOF/overflow/drop
and kill/reap detail. A later successful result write, if possible, still has
the writer error as the first cause. This remains a failed run, but it loses
the required earlier command-failure evidence. It affects both writer modes.

Minimal repair: retain the active label and bounded collector status in the
result or writer immediately after collection. If that status already fails,
latch it as the first observed command failure before attempting raw writes.
Keep later publication errors in a separate field and preserve partial/missing
raw references accurately. Keep payload comparison and its authoritative
comparison record AFTER durable raw/status publication. Do not manufacture a
comparator result, repeat a command, introduce a second writer, borrow reserves,
or open a late publication path. The existing protection for a comparison
already assigned at line404 can remain.

This finding follows static exception flow. No writer fault, mock, unit test,
command failure or runtime experiment was executed.

## Other source and data checks

- All nine candidate SHA256SUMS entries match. All candidate JSON parses. All
  17 input pins and 13 declared repository-input length/hash bindings match
  tracked Git bytes. All ten materialized module destinations, lengths and
  hashes agree with the manifest, source-checks, pins and exact controller
  candidate `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7`. Their full bindings are in
  [bindings.json](bindings.json). Controller remains unchanged and outside
  this logic review.
- The50 unique vectors exactly equal independent expansion of the public
  metadata recipe: seven objects in order, seven verbs each, then deep verify.
  Autoupdate and daemon use their direct code objects. arm64 details, strict
  all-architecture verification and all flags match. There is no arbitrary
  argv or shell input, product invocation, retry or completion command.
- Lines184-199 hash all17 inputs, the vector file and the exact declared basis
  before use. They require50 basis labels/argv, integer exit0 and string
  outputs. This review verifies those checks and tracked bindings. It DOES
  NOT independently verify the private basis bytes or saved outputs.
- Lines171-181,235-245,435-479 bind the caller environment, cwd, root owner,
  mode and filesystem; SOURCE flags; COPY's exact private interpreter and
  `-B` flags; and the hash checks before common import. The parent still owns
  the exact SYSTEM launcher and COPY's full prefix authentication BEFORE
  private entry. Inspection after entry cannot establish that earlier gate.
- Lines480-502 place the exclusive once record before caller-owned commands.
  SOURCE runs SYSTEM `entry-template`, exact private `-I -S -B --version`,
  fixed Python lipo and the50. COPY takes its only bundle path from prepared
  data, validates canonical root/owner0700/filesystem/source binding, reads
  fresh source records, then runs SYSTEM copy file checks before and after
  the50. All share one phase deadline, subject to R1.
- Lines259-367 cap combined capture DURING nonblocking reads at1MiB, using
  remaining capacity plus one overflow byte. Both EOF flags, actual retained
  Popen status, drops/uncertainty, kill delivery and reap are distinct. There
  is one Popen call site, with `shell=False`, closed fds, DEVNULL stdin and
  the fixed environment/cwd. Success requires actual0, both EOF, reap and no
  error, overflow, drop uncertainty, timeout, deadline loss or kill attempt.
  No `communicate` or later drain/completion pass exists.
- The local collector holds0.5s for direct-child kill/reap inside10s and ends
  at most phase-end minus1s. Pipe close times are included in the final loss
  check. Nested alarms and the unchanged parent outer collector contract
  fail on stalls; they do not prove a hard real-time OS bound. No inferred
  wait status, process-group kill, universal containment or extra command
  allowance is present.
- Lines370-409 save raw streams and actual status before payload comparison.
  Only `-verify` labels sort complete UTF-8 splitlines per stream; duplicate
  and empty lines remain. Other text remains exact after only the declared
  bundle substitution. COPY compares both the saved basis and fresh source.
  Already-constructed failed comparisons survive later comparator writes;
  R2 identifies the earlier unprotected raw-publication boundary.
- Lines122-168 sum initial source files before any source write, require
  owned0600 regular files and owned0700 directories, and reserve product16MiB,
  cleanup4MiB and accounting64KiB within32MiB. Each source write is exclusive
  and fsynced. Every COPY write, including guard, parent envelope, streams,
  status, comparison and result, calls exact common.write_bytes with
  `critical=False`; no ledger initialization/reconstruction is called.
  Common's lock, pre-write double charge, exclusive final link and8MiB/file
  bound apply. No cleanup/product reserve is borrowed.
- Lines503-525 keep publication and result authority bounded. There is no
  artifact receipt writer, prepare, controller or product launch. Observation
  completion and helper facts do not grant artifact acceptance. The full
  receipt contract still requires raw/native exits, signature/entitlement,
  ABI, plist, build and transfer/no-transfer evidence. No caller fallback can
  fill missing facts. Parent outer failure observations remain its duty.

## Exact bindings and proof limits

```text
caller                 26bcd57988734f54b2baa14a2b54886ab9ee09c2db1e342a137a49d23c42e18b
input-pins.json        31a17c129e0af6c8c0437458c01fa8c28cffb89f29ee327648a31fed96608352
command-vectors.json   e3ff5756ab3b35a5c815e50ac1fad6520117836e080849260de87592bd327fa0
declared private basis 1670da99b768e01de3fc4ecd4d644fba7031ea09e55d97c1395290bfb681c05f
file helper            bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428
common                 20a9c16030b2de53b99870f25bada68137bfc6fbc5b67cc29f8e878312d4129c
controller             08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4
canonical SDK recipe   d90586546d6b3127dca26147b9d090544d29d7a8b229479966dae68ef25d3b11
```

Caller is30,793 bytes /529 lines. Source-checks remains
`2bde5cf1f9396df5c7d8518caaadcdd69eafc5ff2f39117158efaf87e8d080f5`;
build-source bindings remains
`20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617`.
The canonical SDK recipe differs only in the ten header paths; all other
data equals the original recipe. Prior helper CLEAR `20187fe1b286851f1416437c40cc27108269b542`,
SDK data CLEAR `c490566f5306d79f2537e9bbce3c75db79a279d3`, and controller CLEAR
`b8a422145a5ab5dc197304f6420b85654b5f14ea` keep their original scope.

Parent reports one NEW full source-file qualification: helper0/SSH0,
source702/bundle167/prefix2006,46,504,040 prefix regular bytes, ten modules,
ten headers and prefix-root0700. Its declared facts SHA-256 is
`ebd466312ed346991a9cdd72c64535dbc0696af82b4061b9d79d91fcc937d192`.
I did not read its private root or five retained raw fact files. This is
parent-provided evidence with a separate audit pending, not this review's
runtime claim. Its own-process file deadline result is preserved.

No private-state read or write occurred. The private basis, old outputs,
prefix executable/tree and target interpreter version are declared bindings
or supplied target premises. No supplied source or AST was executed/imported;
no AST parse was needed. Only independent file/Git/JSON/hash readers and
official Python documentation/source reads were used. No tests, mocks, native
parser/command, live process/port query, remote target operation, private Python,
prepare, copy, controller, product, credential, build, signing, integration or
child-agent action ran. No TypeScript changed. The commit disables hooks and
signing. SOURCE50/private-version/prepare/copy/controller/product entries in
this review remain zero. Runtime remains UNVERIFIED.

The old strict SDK alias attempt remains FAILED_UNVERIFIED, helper1/SSH1.
The older M1 app25/25, app0/controller1 attempt remains FAILED_NOT_ACCEPTED;
first post65431 EADDRINUSE cause UNKNOWN; whole-flow NI15 UNVERIFIED. The
original second post-bind, bundle-after, outcome and archival gaps, earlier
failures, stripping warning and unknown full outer build duration/exit remain.
No failed root or guard was reopened. No native/UI/release/performance or
continuous-priority acceptance follows from this report.
