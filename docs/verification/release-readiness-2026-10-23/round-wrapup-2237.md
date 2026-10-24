# Round close — 2026-10-23

The user requested closure at 22:37:22 PDT. All three current delegated tasks
are complete, with no pending child runs. No further repair, review or runtime
was started after that request. The overall hardening goal remains incomplete.
The earlier **about 70%** figure is a subjective planning estimate, not a
measured acceptance score. No fixed completion date is supported.

## Work supported by evidence

- The daemon stop fix exits the main loop after sending its response. It keeps
  the shutdown flush and joins. One real Mac role-Off sample per revision
  observed 1,254.038 ms before and 252.767 ms after. Both exact acknowledgements,
  unsignalled exit 0 and cleanup passed. These are single samples, not a general
  speedup or exit-bound claim. [Source and flow](../daemon-stop-latency-2026-10-23-t3-b205d42e/README.md).
- The real app caller-cancellation flow passed 19/19 checks. The same client
  recovered after cancelled callers' original deadlines. This does not supply
  current combined-app acceptance. [Evidence](../macos-caller-cancellation-2026-10-23/README.md).
- Signing-intent, early private-bootstrap isolation and absolute release-command
  priority fixes are locally integrated with source reviews. Builds and scoped
  ad-hoc signing checks passed. Named signing, secure timestamps, notarization,
  actual running-update handoff and full host restoration remain separate.
  [Signing](../macos-signing-purpose-2026-10-23/README.md),
  [bootstrap review](../private-update-bootstrap-r4-review-2026-10-23-t3-3eaee08d.md),
  [priority](../release-priority-2026-10-23/README.md).
- Native Windows strict release Clippy and the release build passed for exact
  source `e1ef124`, with Rust 1.99.1, locked/offline and jobs 2. The stop runtime
  did not reach a stop request. [Native evidence](../release-integration-2026-10-23/daemon-stop-native-windows-c8a3489c/README.md).
- Real ClientHello/deadline/recovery and connected-idle behaviors have saved
  evidence. Both strict cleanup-bind failures remain FAIL, with no proven
  specific cause. They do not establish a retained product descriptor.
  [Causal review](../cleanup-rebind-causal-review-2026-10-23-t3-0fcc7d19/README.md).

## Latest source handoffs and exact remaining blockers

| Workstream | Saved result | Remaining work |
| --- | --- | --- |
| Ordinary app controller | R7 `d56c65e` repairs local unresolved tracking and receipt-I/O fallback; R8 `172ee85` confirms preservation checks. | **One P2:** fresh recovery Inventory objects lose previously detected uncertainty and can permit early peer teardown. Carry that bounded state across the handoff, then review again. Exact signed app/daemon inputs are also missing. No new ordinary flow ran. |
| Windows stop controller | R2 `7d92633` has a fresh R4 `6b0e4f1` with no actionable source P1/P2. | PowerShell/C# syntax and real native acceptance remain unverified. Old executable, raw receipt, state/evidence and exact transport are absent. |
| Windows data recovery | `6b7a029` recovered exact historical source tar and manifest, including all 558 Git/size/hash/line-ending bindings. | The new build adapter/proposal is **source only and unreviewed**. Exact transport selector and nine fixture path mappings remain missing. No remote command, compiler or product ran. |
| Sparkle scoped LS query | `a7492bc` replaces the global dump with a fixed-ID API helper; static checks and 59 freeze entries pass. | **Independent review is pending.** No helper/controller ran. Private artifacts, a viable concrete operation and fresh explicit consent are still required before another running-target operation. |
| Nonempty entitlements | `9fd2607` records successful real whole-app signing/verification and both-architecture fixture-dictionary preservation. | Raw records, output and bound prior inputs disappeared before export. Surviving transcribed observations do not establish saved-evidence acceptance. No signing retry ran. |

The ordinary source and review are retained at
[R7 handoff](../ordinary-controller-repair-r7-2026-10-23-t3-ad9d06e1/README.md)
and [R8 review](../ordinary-controller-R8-review-2026-10-23-t3-f51c60d1.md).
Other handoffs:
[Windows recovery](../release-integration-2026-10-23/windows-prerequisite-recovery-r5-86e0bfd1/README.md),
[scoped LS source](../sparkle-scoped-ls-source-repair-2026-10-23-t3-2d3b20d4/README.md),
[entitlement evidence limits](../macos-nonempty-entitlements-2026-10-23-t3-aefa82e2/README.md).

The recovered source archive is 4,065,280 bytes, SHA-256
`775e04301b1f3816ccf39fd088e1100da4344be5b171df7330b0049a7f589adf`.
The recovered manifest is 188,309 bytes, SHA-256
`a208cbb73d29b4939fcd4a43eae67769d1763823fc6a77979bb7abe96ef243dc`.
Neither is an executable or a new native result. The bounded transport lookup
stopped at 80 items. It found a record of a historical match, not its selector
value. No guess, connection or credential reconstruction followed.

## Closed Sparkle attempt

The user approved the exact one-shot operation at 21:01:00 PDT. Its unchanged
reviewed controller exited 1 at live preflight when `lsregister -dump` exceeded
the five-second command limit. No target, daemon, updater, installer, relaunch,
HTTP bind/request, IPC request or final quit began. This invocation is closed.
No retry, marker renewal, guard reset or approval reuse occurred.
[Approval, failure and selected postflight](../sparkle-running-preflight-failure-2026-10-23/README.md).

## Other gates that stay open

- A successful isolated Swift release/package path. The output-file-map I/O
  cause remains unproved. The build gate stays stopped pending suitable
  approved isolated instrumentation. Old byte/link recovery does not establish
  original metadata restoration.
- Complete current app/daemon/peer flows, strict cleanup, resource acceptance,
  native Windows stop and the real update/relaunch lifecycle.
- Native UI, input, clipboard and hardware acceptance; strict DNS/discovery
  witnesses and the safe slow-DNS fixture prerequisites.
- Named Developer ID signing, secure timestamp and notarization acceptance.
- Current-candidate security/provenance acceptance. The crafted-mDNS, original
  provenance/security and IPC/Synergy3 import-budget tasks remain stopped.

## Suggested order for a later round

1. Fix the one ordinary-controller uncertainty handoff and obtain a fresh review.
2. Review the scoped LS source and Windows fresh-build adapter.
3. Restore or qualify exact executable, signed-bundle, transport and baseline
   prerequisites. Keep newly derived inputs distinct from lost historical bytes.
4. Run the bounded real flows after their gates are satisfied. Preserve first
   failures; do not retry closed operations or replace required proof with unit tests.
5. Close isolated packaging, native/manual, signing and current release gates.

All handoffs are local. Nothing was pushed or newly published in this round.
No installed app or live input-sharing setup was changed by these source-only
handoffs. The older declared cache/shared-state limits remain; this is not a
full host-restoration claim. [Durable continuation state](continuation-2210.json)
and the [full evidence index](../release-integration-2026-10-23/README.md) record
the exact boundaries. The round is closed, rather than the goal marked complete.
