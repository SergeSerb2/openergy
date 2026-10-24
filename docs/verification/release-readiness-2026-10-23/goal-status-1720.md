# Release completion map — 2026-10-23

**Release acceptance is incomplete.** Retain the root's **roughly 70% overall**
planning estimate. It is not measured. A proposed **60–80% planning range** makes
that uncertainty visible; it is subjective, not a confidence interval or release
probability. No numerical weights are justified yet. The broad goal, “stable,
secure, fast and lightweight as possible,” has no finite denominator. Check counts,
audit counts and this checklist must not become a completion percentage.

## Basis and evidence levels

This map uses local Git and saved reports at exact parent
`a121376fe6c1715fa99d72559336ca3b3a73b439`. Both the root checkout
`/Users/serge/.t3/worktrees/openergy/t3-52422617` and this isolated child started
clean at that commit. The latest child-status brief was supplied at **17:24:48
America/Vancouver**. The requested filename is retained; it is not a 17:20
runtime observation.

- **Implementation:** source exists at the stated commit. Integration and source
  identity need their own evidence.
- **Source/audit:** a reviewer inspected source or saved records. This adds no
  independent runtime execution.
- **Saved real flow:** a recorded real caller/process flow passed or failed on
  its exact source, artifact, platform and workload.
- **Native/trust/publication:** desktop behavior, installation/relaunch, OS trust
  and public release each need their own proof. A virtual flow or signed feed
  cannot close them.

This child performed read-only inspection and wrote only this document. It ran
no build, runtime, network operation, test, desktop action or new scan. Existing
reports remain unchanged. The newer idle-TLS commit below was read as an immutable
Git object; it is **not an ancestor of the frozen parent**.

## Proposed finite release checklist

Use these gates for one named release candidate, rather than for an unlimited
optimization goal. The root/maintainer must first name supported platforms,
required native flows, finite workloads and acceptable resource/latency limits.
No weighted total is valid before that scope is agreed. Each unchecked gate
remains open even when some evidence within it passes.

| Gate | Implementation and source/audit evidence | Saved real-flow evidence | Completion condition and owner |
| --- | --- | --- | --- |
| [ ] 1. Freeze the candidate and acceptance scope | Current parent is `a121376`; later child work is separate. | Reports bind older exact sources and artifacts, not one universal final artifact. | Root names the final source/artifacts, platform matrix, workloads and thresholds; bridges unchanged inputs and validates material changes. |
| [x] 2. Recorded combined virtual CLI/IPC/protocol flow | Source `f7b10e7`; report binds runtime source to reviewed `e6a9125`. | **89/89**, zero cleanup failures; **30 Docker scenarios excluded**. | Closed only for the recorded private virtual artifact and suite [E1]. It does not close final-candidate or native acceptance. |
| [x] 3. Recorded Mac discovery startup/recovery/owned shutdown | Candidate `e6a9125`; R9 audits source and saved records. | **61/61 twice**; working-observer old-bug baseline **45/61**; retained **19/19**; owned shutdown **4/4**. | Closed for those recorded cases [E2]. Whole-cache bounds and unexecuted rejection cases remain separate. |
| [ ] 4. Windows discovery startup and applicable cleanup | R10 finds no concrete P1/P2 in its source/evidence scope. | Startup **105/106** and **113/115**, both **exit 1**; required failed-send witnesses are missing. Retained **19/19** passes. | Windows owner needs the missing witnesses within each required run. Earlier normal shutdown/Drop OS proof is on `e4f48ba`, not automatic same-source proof on `e6a9125` [E3]. |
| [ ] 5. Windows DNS complete normal/conflict coverage | R11 source and R13 saved-evidence reviews close their stated verification findings. | R12 normal and conflict commands both **exit 2**. Qualified refusal **10/10 each** and same-caller recovery pass. Normal queue/lifecycle passes; conflict skips eight unsafe batches. | DNS owner must close the missing conflict coverage and strict timing gates. Skips and build success grant no full acceptance [E4]. |
| [ ] 6. Strict TCP/fallback/TLS deadlines and slow DNS | Existing reviewed verifier source does not supply missing runtime witnesses. | R12 deadline brackets cross the caller bounds. Historical `615c16e1` refusal run passes **10/10**, but captures no old fault. | Strict timing, fallback through the original deadline, deterministic slow-DNS evidence and the required historical negative witness remain unverified [E4, E5]. Do not repeat runs merely to obtain green. |
| [ ] 7. New idle-then-poll TLS case | Opt-in example/source/evidence `9f6625c`; no production library change. **Independent review pending; not integrated at this parent.** | First eligible post-expiry poll and same-driver/handle pinned-TLS `PASS`/`BACK` recovery pass. **Overall FAIL, exit 1** at immediate cleanup rebind, `EADDRINUSE` (48). | Root first audits the saved failure, source and cleanup evidence. Later absence observations do not change the original verdict or prove its cause [E6]. |
| [x] 8. Recorded packaged app model/IPC flow | Packaged source `d647a01`; current-flow report at `d83807d`; scoped background source review found no concrete P1/P2. | **25/25**, real app model/IPC and bundled daemon with a private virtual peer, rendering disabled. | Closed for this recorded model flow, including stalled writes and supervised crash recovery [E7]. Rendered UI, caller cancellation and native input/clipboard remain separate. |
| [ ] 9. Finite performance and resource acceptance | Existing source observations and bounded measurements establish no sustained queue or whole-process cap. | Event measurement **12/12 paced checks**; short role-off idle observations exist. | Root defines budgets; performance owner needs sustained workload, recovery and exact-artifact evidence. Unfinished refresh tasks, callback queue depth, loaded-peer use and whole-cache/timer/query/variable-byte bounds remain open [E8]. Stopped work stays excluded below. |
| [ ] 10. Current source/dependency assurance | Dependency policy covers `e4f48ba`; source-history snapshot covers only `7ad8265` and ancestors. | Six policy graphs pass; recorded duplicate/unused-license warnings remain. History raw exits are **1 / 0**; **95 source-hash findings**, none unresolved, at that old snapshot. | These are scoped checks, not current-head security clearance. Original security/provenance review remains stopped. Root must not promote the old snapshot to a current scan [E9]. |
| [x] 11. Recorded local packaging and update feed | Sparkle **2.10.0**, universal bundle/framework/helpers and archive/feed signing are included; scoped reviews close their findings. | Real feed selection/failure paths and local signature/archive checks pass. Saved live Cloudflare check sees a **signed empty feed**. | Closed for local packaging and the recorded empty-feed check [E10]. **No application archive is published**; Developer ID and notarization remain open. |
| [ ] 12. Updater installation, daemon handoff and relaunch | Corrected private stopped-target preparation has a clean scope review. Original CLI build-effects failure/unaccepted artifact is preserved. | Brief says the **17:16-approved stopped-target installer run is already owned by a separate agent**. **No result is supplied.** | Existing installer owner returns exact first-run evidence. Stopped-target installation cannot prove running-target relaunch, manual update UI or daemon handoff [E11]. Do not duplicate that run. |
| [ ] 13. Native desktop and positive Windows migration | Native backends and prior dated reports exist; virtual/session-0 checks do not establish interactive acceptance. | Current native input/clipboard, supported screen/layout/recovery flows and positive Windows migration remain open. | Native acceptance owner needs a separate test account/machine with recovery input. Preserve live sharing, installed apps and permissions; no desktop automation is authorized here [E12]. |
| [ ] 14. Trusted artifact and publication | Local signatures and feed EdDSA signatures are separate from OS distribution trust. | No trusted Developer ID/notarization/manual UI/relaunch or published application archive is established. | Release owner supplies exact candidate trust, installation and publication evidence under separate authority. This task authorizes no signing-key access, app install, push, publication or Cloudflare change [E10–E12]. |

The checked boxes accept only the named saved scopes. They are neither four
completed release gates on a final candidate nor a numerator for a percentage.

## Dependencies, stopped scope and duplicate work

- **Keep all three provider-stopped tasks off limits:** original independent
  release-provenance/security review; crafted mDNS expiry/NSEC/orphan experiment;
  IPC/Synergy 3 budget work. They remain open and unaccepted. Do not resume,
  rename, reroute or use another task as their substitute. Old handoff advice
  to resume the IPC fix is superseded by this stop. Partial IPC work is not
  accepted implementation.
- **One installer owner:** the brief updates the older preparation report's
  “approval pending” status only. It supplies no execution or cleanup result.
  The original build's registration/assessment effects and unknown persistent
  state remain recorded. A repaired private copy does not clear that history.
- **One DNS evidence chain:** R11 source review, R12 native execution, R13 audit,
  historical refusal execution and R15 audit are distinct. Audits do not add
  runs. The historical positive result does not demonstrate a negative control.
  The idle-TLS case adds a different finite caller workflow, not continuous
  deadline or Windows proof.
- **Keep race witnesses local to their run:** neither two Mac startup passes nor
  pooled Windows witnesses close an incomplete Windows gate. The old **11/14**
  startup failure remains failed. Earlier-source cleanup proof cannot transfer
  without an exact applicability bridge.
- **Keep workload evidence distinct:** app **25/25** and event **12/12** are
  different runs; the event overlay did not repeat the full 25-step flow. The
  combined and earlier virtual suites overlap. Do not add their counts or
  describe short idle samples as a soak, memory cap or measured speedup.
- **Read report dates and pins:** the combined report's pending final Mac audit
  is superseded by R9. The R12 report's “historical control none ran” is
  superseded by the later historical run, whose negative gate is still open.
  The source-history file's “current-parent” title means **`7ad8265`**, not
  `a121376`. No old failure or raw exit is relabelled by this map.

## Next useful work, ordered by release risk

1. **Root: audit existing idle-TLS source and records, without runtime.** Check
   exact bindings, event/deadline predicates and the failed cleanup condition.
   Retain `EADDRINUSE`, first outputs and exit 1. Report whether the cause can be
   established from saved evidence; do not infer it from TIME_WAIT or later
   absence and do not rerun the recorded case.
2. **Existing installer owner, then root: finish and audit the already-approved
   operation.** Await its exact source/artifact IDs, first verdict, negative and
   positive installation results, helper/cache observations and remaining host
   effects. Do not start a parallel installer. Keep relaunch/UI/trust gates open.
3. **Windows/DNS owners: prepare a read-only gap-to-evidence plan.** Name each
   missing startup witness, conflict skip and timing interval, plus its exact
   acceptance rule and safe ownership prerequisite. Keep the passing historical
   result and unqualified macOS listener-shutdown method unchanged. Any later
   authorized finite real flow must preserve first failures and complete cleanup;
   no blind retries or new command is authorized by this map.
4. **Root: freeze a finite release contract and evidence ledger.** Select the
   supported candidate/platforms, workload limits and native/trust owners. Bind
   applicable saved artifacts to that candidate; expose missing coverage rather
   than rerunning accepted unrelated flows or reopening stopped tasks.
5. **Performance owner: design one bounded sustained event-delivery follow-up.**
   Use the two measured app observations to define an explicit workload, queue/
   task or observable backlog measure, resource ceiling, recovery and cleanup
   criteria. Missing observability is a gap, not a bound. Keep stopped IPC,
   Synergy 3 and crafted expiry work excluded. Execution needs its own scope.
6. **Native/release owners: plan the remaining separate-environment checks and
   distribution decisions.** Native input/clipboard, positive Windows migration,
   manual UI, running-target relaunch, Developer ID/notarization and publication
   cannot be supplied by this doc-only child or the stopped-target installer.

For any later permitted runtime, retain the current constraints: uniquely owned
literal loopback fixtures; discovery disabled and virtual platform where relevant;
private config before startup; no TCP 24800 or UDP 5353; Rust 1.99.1, nice 15,
two jobs, macOS deployment 15, locked/offline where possible. Do not add unit
tests, mocks, global dependencies, Docker, desktop automation or host/default
changes. Do not access production secrets or alter live sharing.

## Exact evidence index

These are saved reports inspected for this map, not new independent executions.
Commit IDs name source or evidence as specified; integration does not add a run.

- **E1:** [Combined virtual flow](../release-integration-2026-10-23/combined-virtual.md),
  source `f7b10e7550aa11200385b06b69f6dcb2328dd9b5`.
- **E2:** [Final Mac report](../release-integration-2026-10-23/discovery-startup-r7-repair.md)
  and [R9 audit](../release-integration-2026-10-23/discovery-final-evidence-review-r9.md),
  evidence `8e891f1575483f003756b6b84dc157eb049a9986`, source
  `e6a9125be2fc59fbe92fef6ea351b222298bfc52`.
- **E3:** [Windows startup](../release-integration-2026-10-23/windows-observed-startup.md)
  and [R10 audit](../release-integration-2026-10-23/windows-observed-startup-review-r10.md),
  evidence `1d46512fb816d43e9fa4691c5bd7ae6a15d377cb`;
  [earlier-source owned cleanup](../release-integration-2026-10-23/windows-owned-cleanup.md),
  library `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`.
- **E4:** [R11 source integration](../release-integration-2026-10-23/dns-verifier-r11-integration.md),
  reviewed source `9a4ac2eed77a8ed8bddd61e060be14a506df570a`;
  [R12 Windows execution](../release-integration-2026-10-23/windows-dns-final-8281eb7.md),
  runtime parent `8281eb79f54725ddc8fe8bae83dbd0f44fa5d885`, evidence
  `a8f7f9af8e1ff1a1d2cf7cca15812826504a51d9`, integration
  `b93fc8c688d11bb057b6b6f0041966d8cd0168fc`;
  [R13 audit](../release-integration-2026-10-23/windows-dns-final-review-r13.md).
- **E5:** [Historical run](../release-integration-2026-10-23/windows-dns-historical-refusal-615c16e1.md)
  and [R15 audit](../release-integration-2026-10-23/windows-dns-historical-review-r15.md),
  source `615c16e1df71bc0b350aeee4f1f34bceba5a59e0`, evidence
  `1d9b6a6ff0f234bd663647b325adf3d47f1743ca`;
  [macOS fixture method ruled out](../release-integration-2026-10-23/macos-owned-refusal-qualification.md).
- **E6:** Child source/evidence `9f6625c951835295419ed1c53565581a7fa9681a`;
  immutable Git paths
  `docs/verification/release-integration-2026-10-23/idle-tls-deadline-2701060/README.md`
  and `derived.json` were read at that commit. They are absent from this parent
  tree. Runtime parent `270106002490ef3e135d4c6500df1c8ff452b45b`.
  No independent review result is supplied.
- **E7:** [Packaged app flow](../macos-app-current-flow-2026-10-23/README.md),
  evidence `d83807d542a86a99ad83978cc460028d972373e1`, packaged source
  `d647a01a7b83f79187d1a15c067a763647f01a31`;
  [background source review](../macos-app-current-flow-2026-10-23/background-review.md).
- **E8:** [Event measurement](../macos-event-delivery-2026-10-23/README.md)
  and [R2 audit](../macos-event-delivery-2026-10-23/review-r2.md), evidence
  `b72f729f6e0adbfe8a4b8d62dbc0c0d0ca43838a`, integration
  `f520268917375ed31c104a142ba676dd0e5ba0a2`;
  [combined idle sample](../release-integration-2026-10-23/combined-idle-resources.md),
  source `f7b10e7550aa11200385b06b69f6dcb2328dd9b5`.
- **E9:** [Dependency policy](../release-integration-2026-10-23/dependency-policy.md),
  checkpoint `e4f48ba47ea69aa76a08de5cf0cf61c68d903c03`;
  [history snapshot](../release-integration-2026-10-23/source-history-current-snapshot.md),
  scanned parent `7ad8265f9237aad8d37672095c884f051b5e4c2f`.
- **E10:** [Sparkle integration and saved hosting checks](../sparkle-integration-2026-10-23/README.md),
  source `d647a01a7b83f79187d1a15c067a763647f01a31`;
  [packaging scope](../sparkle-packaging-2026-10-23/README.md).
- **E11:** [Original build-effects audit](../sparkle-cli-build-preparation-2026-10-23/review-r2.md);
  [stopped-target preparation](../sparkle-stopped-fixtures-2026-10-23/README.md)
  and [corrected scope audit](../sparkle-stopped-fixtures-2026-10-23/review-r2.md),
  reviewed `5f91d89277a396d3976ca129be9b82d647fb09c2`, review integration
  `9f62abed902e1e24ff6787fe695422e4bf6dfbba`.
  **17:16 approval/running status comes only from the supplied latest-child
  brief. No installer result/evidence ID was supplied or inferred.**
- **E12:** [Existing native/release gates](../../plan/readiness.md) and
  [current integration index](../release-integration-2026-10-23/README.md) at
  frozen parent `a121376fe6c1715fa99d72559336ca3b3a73b439`. Their dated status
  lines retain the qualifications and later updates stated above.
