# Hardening pause handoff — 2026-10-22

The user requested: finish current work, then pause the goal until told to resume.
The current bounded discovery integration and already-running reviews/Windows
diagnosis are finished. This is a checkpoint, not completion or release approval.

## Integrated and checked

- Branch: `t3/public-release-readiness`.
- Bounded discovery candidates `884ff876` and `da668c3` are integrated as
  `efca42e` and `dddf9fe`. Exact runtime source:
  `dddf9fef012f9d56ea6435be36bd5c74a08087d7`.
- Release examples passed 14/14 startup, 19/19 retained-discovery and 4/4 actual
  owned-shutdown cases. Build, strict workspace/vendor Clippy, formatting,
  archive/source verification and notices passed. All 129 source inputs stayed
  unchanged. [Integration evidence](discovery-bounded-integration.md).
- The final fresh independent source/evidence review found no concrete P1/P2
  issue in the discovery candidate and closed its cancelled-startup finding.
  The review did not run binaries; the parent integration run did.
- Vendor dependency policy passed the combined graph and five platform graphs
  with zero advisory errors or warnings. Duplicate and unused-license warnings
  remain recorded. These checks are not native platform validation.
  [Policy evidence](vendor-dependency-policy.md).
- Windows supplement `fb13484` is integrated as `fb4bfcc`. It links the frozen
  library from `07adb0a`, not the new discovery implementation. Ten real driver
  refusal cases, IPv6 frame exchanges and later lifecycle contracts pass.
  The four unchanged 500 ms OS controls time out. Separate six-second controls
  return `ConnectionRefused`, errno 10061, after 2.02–2.05 s. The delay's cause
  remains unresolved. Original full/refusal-only exit-1 results stay unchanged.
  [Windows evidence](windows-dns-supplement.md).

No owned discovery or Windows fixture executable or assigned listener remained
after the recorded postflight checks. Both active child tasks finished and
started no follow-up. Installed/live sharing, native input/clipboard and
production state were preserved.

## Separate IPC candidate: one remaining P2

Candidate `231b7f6908498d5d9b3380ed7fa8f568ac7b39c3` remains separate and clean
in worktree `t3-407c48fc`. It is **not integrated**. Its real-process evidence
reproduces and fixes the tested buffer-retention, native-config expansion and
progressing-subscriber shutdown cases. Fresh independent review found one
remaining source-supported P2. [Full review](ipc-aggregate-review.json).

Synergy 3 import calls `derive_synergy3_links` and `link.into_owned()` before
`cfg.validate()` checks the derived-link budget. A schema-17 file with a roughly
1 MiB generated source name and 100 adjacent target rectangles can therefore
materialize roughly 100 MiB of copied source names before rejection. The input
can remain below 16 MiB. The daemon owner runs this import even with
`apply:false`. This trigger has source evidence; the reviewer did not reproduce
it at runtime.

The next fix should preserve borrowed final Synergy 3 links through its
identical-key overwrite rule, check their byte charge before owning names, and
propagate the error to the importer. Verify through real `config.import_synergy`
with `apply:false`, a bounded RSS guard, rollback/recovery and a valid control.
Then obtain a new independent review of that revision before integration.

The review found no further concrete P1/P2 issue in native-config budget order,
capped response framing, receive-buffer shrinking, shared event frames, Unix
stop-aware writes or Windows overlapped cancellation ownership. Windows native
compile/runtime acceptance of this IPC candidate remains absent. Saved evidence
was audited against immutable source hashes; it was not independently rerun.

## Resume sequence and remaining gates

1. Fix and reproduce the remaining IPC Synergy 3 import-budget finding, then run
   a fresh independent review. Keep the current failed/baseline evidence intact.
2. Integrate an accepted IPC revision. Run the real combined daemon flows and
   the existing virtual E2E suite with the corrected port range. The older full
   88/89 run remains failed; its P07 setup issue is separately diagnosed and
   fixed. No full combined daemon E2E ran on the new discovery source before pause.
3. Perform fresh scoped native Windows validation for the combined accepted
   source. The Windows supplement proves only its frozen `07adb0a` scope.
4. Keep whole mDNS cache/expiry, timer/query work and variable-record-byte bounds
   open. Delivery state is capped at 64 services; no automatic refill guarantee
   exists. Active OS DNS calls remain uncancellable and deterministic slow-DNS
   timing is unverified.
5. Keep native input/clipboard, positive interactive Windows migration, original
   independent security/release-provenance acceptance and trusted signed/notarized
   distribution as release gates. Do not risk the live sharing connection.

The provider-stopped expiry/NSEC/orphan-record experiment and original blocked
release-provenance security review remain unverified. Do not reroute or rephrase
them. GitHub private-reporting setup remains deferred at the user's request.

## Repository and retained evidence

All current source/evidence changes are local. No push, merge, publication,
repository visibility change or protection change occurred. The remote PR #2
branch was directly checked at `ba7881904b1e49c42067ba340c441056cdeb33f8` before
this checkpoint. Its green checks do not validate these later local commits.

Local evidence pointers, retained outside Git:

- `/tmp/oe-bounded-discovery-integration-state.json`: exact inputs, frozen
  release examples, commands, logs and owned-shutdown reports.
- `/tmp/oe-vendor-policy-state.json`: immutable candidate snapshot, verified
  cargo-deny tool and raw seven-command/six-graph output.
- `/tmp/oe-discovery-active-agents.json`: task IDs, child threads, model options
  and current handoff state. Both final tasks are terminal.

No TypeScript package changed. No new unit tests, mocks or computer-use checks
were added. The supplementary Windows helper retains its recorded EOF blank
line so its committed bytes match the compiled source hash. Original vendor
whitespace and failed raw evidence are preserved with their documented scope.
