Design for the new inert seven-source candidate. `execution=false`.

The patch base is exact R5, not the integrated branch's apparent ancestry. Every
R5 file matches `6b7a029ffbd8e41b411408bbcaf9b219e1f6592c` and its checksum. The
patch changes only `source/controller.py.txt` and `source/build.ps1.txt`. It has
no formatting-only source edits. Five complete source files are retained so a
reviewer can assess one whole candidate. Existing package files are never edited.

The fixture boundary has two gates. The first accepts only nonempty Windows
relative paths without root, drive, absolute form, or parent components. It runs
before transport. The second uses the actual setup TEMP string and Windows
lexical normalization. Equality is refused, and the common path must equal that
exact normalized parent. It also requires distinct normalized roots. Both gates
keep the original private hash and exact ordered nine labels. There is no host
discovery, filesystem resolution, fixture access, or data fabrication in mapping.
Existing native ACL/reparse and before/after checks remain required afterward.
Lexical containment is not a new filesystem or provenance acceptance claim.

Native build ownership has four phases:

1. Register one process object before creation. After creation returns success,
   retain the exact root handle before any fallible observation. Start output
   draining, then require actual executable/parent/creation/priority/session
   identity. The pinned toolchain, private environment, guard, and one release
   command remain unchanged.
2. Poll only the existing compiler-name set. Before and after each one-second
   CIM operation, check the same work deadline. Keep four finite ancestry passes.
   Admit a descendant only with a retained handle and matching generation,
   executable, command line, and current retained parent. Preserve observation
   races as races. Reject a reused PID as new ownership. Never infer unsampled
   compiler or release-linker priority.
3. Keep all output waits/writes, artifact checks/copy, and source checks inside
   the outer lifecycle. Catch failures once. Write the first error to a dedicated
   variable and retain later errors separately. A successful compiler exit does
   not release ownership before fallible artifact/receipt work completes.
4. In `finally`, signal live verified handles, starting with the root, before
   bounded waits. Use no new CIM/PID discovery for recovery. Observe signal and
   actual exit code. Settle streams only within the same reserve. A nested
   `finally` disposes each registered process and safe handle independently.
   Then attempt every receipt. A late or failed receipt fails the stage. Every
   receipt describes observed success, explicit false, null, or an error; none
   assumes that SSH or an ambient job terminated a compiler.

Root ownership follows successful creation, including when CIM identity fails.
That permits safe recovery of the exact created handle without relaxing build
acceptance. A descendant has a provisional entry while identity is acquired. It
becomes kill-eligible only after all ownership checks pass. A provisional entry
is still visited for handle disposal and truthful observation records.

The work deadline starts once at native build entry and expires at 600 seconds.
The recovery deadline is the earlier of failure/settlement entry + 30 seconds and
build entry + 630 seconds. Every blocking owned-process wait takes the smaller of
10 seconds and that deadline's remaining time. Each stream wait takes the smaller
of 5 seconds and the same remaining time. Handle closure is attempted even after
the wait budget is gone; no new wait window starts. The recovery finish tick is
checked against its deadline. Final receipts must also finish before build entry
+ 630 seconds. The existing outer build call remains 660 seconds, with admission
requiring that full allowance before the overall transport reserve. No overall
timer or one-shot guard can restart. Metadata and filesystem calls still have
the inherited arbitrary-call preemption limit; the source makes no hard real-time
claim.

The controller's `command`, `remote`, and `recover_transport_owner` methods remain
exact R2/R5 bytes. Its new build admission check is in `stage`. Its new `run`
exception handling preserves the first flow exception and attempts one postflight
path for any known new root. It records a later collection exception separately.
No failure admits runtime, a second release build, another stop, or root deletion
without the existing collection bindings. Postflight source itself is unchanged.
An attempt can fail to complete postflight under a transport or overall deadline;
that is unresolved cleanup, not permission to bypass a guard or report PASS.

The future receipt interface remains `openergy.fresh-release-build.v1` with one
`release` step. Additional ownership/error/deadline fields strengthen evidence.
Runtime still verifies source/archive hashes, step exit 0, no timeout, jobs 2,
AMD64 PE32+ bytes, and the actual new executable hash. It never accepts the old
binary hash as a new-build target. The local controller's build-stage exit must
also be zero before runtime can begin. If the final receipt write fails or runs
late, that outer stage result stays nonzero even if partial bytes were written.
Failed filesystem writes cannot guarantee a durable final record. Stderr and
independent collection remain evidence sources; missing evidence fails closed.

All readiness, wire, stop, native child recovery, source preservation, baseline,
and removal behavior outside these two source files remains unchanged. Common
and observer have no hunk. Common consumes only the controller's validated mapped
roots on this flow; adding a second reader or changing Snapshot would expand
this repair without addressing an additional finding.

No fixture was prepared and no target-specific path or remote-shell alias syntax
was accepted in this task. The parent reports the exact selector recovery; this
task does not inspect that private value. Nine missing historical fixture paths
still prevent execution. The existing private source archive and manifest require
future byte checks; this task only verifies their published bindings and all
558 immutable Git source members. The old raw/executable/state/evidence inputs
remain unrecovered. No source review can recreate those historical receipts.

Static validation is limited to file, Git blob, size, SHA-256, JSON, Python AST
data parsing, and exact in-memory unified-patch application. It executes no
supplied source, helper, checker, or AST. It parses or compiles no PowerShell/C#.
It creates no tests, mocks, simulations, fixtures, runtime state, guards, or
approvals. Total resource bounds, unsampled processes, kernel identity perfection,
native syntax/runtime, external SSH/job cleanup, and actual handle/OS completion
remain UNVERIFIED. All previously stopped gates and the closed expired Sparkle
one-shot remain stopped.
