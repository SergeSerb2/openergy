# Fresh M1 ordinary flow: static handoff

2026-10-24, America/Vancouver. Owner `t3-5463ce44`; initial clean HEAD
`1f5b72e491bfbe2872dbe9f233e599ff786d6f64`.

**SOURCE/DATA ONLY. SCOPED BUILD RETAINED. RUNTIME UNVERIFIED.** Parent alone
owns remote access, artifact qualification, preparation and runtime. The ordinary
headless flow is already authorized. No new human approval gate is added.

The one completed build uses candidate
`7f20c9cdd97c1dc313f007b879babc726acbb914`, R4 review
`fb7b0d97ba4842021d8315956800c9aa8ec20dfe`, and raw source
`ed7d96bf53da1a539fa2acde3b7133613221fd4b` (702 non-docs files).
The parent's M1 baseline is macOS27.0.1/26A434 arm64,
Xcode27.1/27A9275, Swift6.4/SDK27.0 and qualified private Python3.13.9.
One ordinary bundle.sh produced arm64 app/daemon, build1, ad-hoc signatures,
updates disabled. No app launch occurred per the parent.

[retained-build-bindings.json](retained-build-bindings.json) pins the actual
retention/export receipts, both archives and all 44 retained evidence files.
Their hashes/lengths were checked. The app archive matches all 167 inventory
records as data: 95 regular files, 9 symlinks, 10,050,709 regular bytes;
root mode0700. No artifact was extracted or executed. Its 4,692,177-byte archive
SHA-256 is `7eb92de1ae6af988f8925546624a9f69fe20309aa8ea15f13fcf0b9ddf1cea21`.
App SHA-256: `094d02b3831ee6565ae1628407756ad48053a0268f2a489c65bbde28b768d775`.
Daemon/peer SHA-256: `17065c334290b34c7a1b69483f409655e072eb0505ff2a0cc0bf6ed5f77a7bb2`.
All 2008 saved build rows report NI15; they prove no future-flow priority.

The full independent saved-build audit
`3d176f04184ba90e9922e0c309458696592ddc5e` (parent import `6bcdadc`) found no
concrete P1/P2 in its evidence scope. Its exact report and bindings are pinned.
101.050310709 seconds is worker-operation time. The parent reports a
103.8202-second SSH invocation envelope including launch/collection.
**Exact full-controller duration is not retained and is not claimed.** Preserve
the rust-objcopy SIGABRT/missing libLLVM warning and Swift deprecation warnings.
The export did not preserve timestamps, ACLs or xattrs. Saved Mach-O headers
are not a cryptographic signature, entitlement or Gatekeeper verification.

Reuse R9 `e34401c8842c2a93b5c355f4c515a79ee05cfa72`, folder
`docs/verification/ordinary-controller-uncertainty-handoff-r9-2026-10-24-t3-79efd32e/`.
Its full independent review is `8a97e3bf5b862d3f3ac1ab740a826e3e0bab0353`,
`docs/verification/ordinary-controller-R9-review-2026-10-24-t3-3f0af888.md`.
All eight modules and the full review were read.

[admit-fresh-m1.patch](admit-fresh-m1.patch) changes only historical admission:
`common.py` replaces two fixed paths with parent-bound source/Python values;
all its functions are unchanged. `prepare.py` replaces old 29/397/124 input
requirements and missing old bundle/hash dependencies with this exact build's
receipts, full 702-file inventory and new full bundle tree. It pins the new app,
daemon and five metadata hashes. Preparation keeps its original guards, byte
ledger, 120-second deadline, copy/tree/hash/ordinary-plist checks and modes.
Only input selection, root label, carried path/private-Python/update environment
values and explicit fresh-baseline metadata change in its main function.
`original_commit` remains historical lineage; `source_commit` names the new code.

The other six modules are byte-exact R9, including controller, ownership,
watchdog and process reader. No full modules are copied here. There is no new
harness, watcher, process policy or product predicate. [source-checks.json](source-checks.json)
binds the eight reconstructed texts; [bindings.json](bindings.json) pins exact
Git dependencies. [operator-plan.md](operator-plan.md), [receipt-contract.md](receipt-contract.md)
and the inert schema/templates define the finite handoff. Known artifact hashes
and recorded M1 paths are filled; qualification fields remain false/null and
execution remains false. Parent must recheck machine paths and complete real
evidence before launch. Receipt booleans/schema conformance alone are not proof.

Open prerequisites: independent review of this admission delta; actual parent
signature/entitlement/dependency/process-ABI and source/private-copy qualification;
remaining concrete receipt fields; then one real ordinary run with complete
OS exits, whole-flow NI15 and independent cleanup. The old signed bundle,
compiled app, icon and raw workspace state are not new prerequisites. Do not
search agent temporary data, reconstruct old inputs, rebuild or re-sign.

All 87 originals remain unchanged and **FAILED / NOT ACCEPTED, 23/25**.
[static-checks.json](static-checks.json) retains the reader's first error and the
local Windows cmd CRLF difference: 701 checkout files equal raw bytes; all 702
immutable raw Git bindings match. M1 admission still requires exact raw bytes.
The manifest binds all package files except itself; the final commit binds it.

Only this folder was written. Verification used own file/Git/hash/JSON/AST,
patch and archive-member data readers, without extraction. No supplied task
source/helper/checker/AST was imported, evaluated or executed. No remote/network,
preparation, test/mock/simulation, probe/OS/process/port query, build/signing,
product, credentials, GUI/native input, clipboard, TCC or delegation occurred.
No TypeScript changed. No push/publication. Hooks/signing are disabled for commit.
The expired Sparkle operation and other stopped scopes stay closed. This package
grants no runtime, native, update, trusted-release or host-restoration acceptance.
