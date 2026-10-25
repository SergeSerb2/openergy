# Independent source/data review of fresh M1 ordinary admission

Date: 2026-10-24, America/Vancouver. Reviewer checkout:
`/Users/serge/.t3/worktrees/openergy/t3-af8da74b`, initially clean at
`6bcdadcd14d98cc4d6d36d2a782ef82dc89be443` on
`t3/review-m1-flow-admission`.

**Verdict: no concrete P1 or P2 in the bounded SOURCE/DATA ONLY scope.**
The two-module admission delta preserves the reviewed R9 execution and recovery
contracts. Runtime remains **UNVERIFIED**. This review is an inert record. It
creates no executable receipt, approval, qualification or runtime acceptance.
The ordinary flow is already authorized; this review adds no human approval gate.

## Exact inputs and independent integrity checks

Candidate: `02c9b174c8e13d4eaa6bf7692f20cbb44c93ae57`, folder
`docs/verification/m1-ordinary-fresh-flow-plan-2026-10-24-t3-5463ce44`, read from
the named `t3-5463ce44` worktree and compared with immutable Git objects.
R9: `e34401c8842c2a93b5c355f4c515a79ee05cfa72`, folder
`docs/verification/ordinary-controller-uncertainty-handoff-r9-2026-10-24-t3-79efd32e`.
Its complete independent review is bound at
`8a97e3bf5b862d3f3ac1ab740a826e3e0bab0353`,
`docs/verification/ordinary-controller-R9-review-2026-10-24-t3-3f0af888.md`.
Product source is `ed7d96bf53da1a539fa2acde3b7133613221fd4b`.

[review-bindings.json](review-bindings.json) records exact file lengths, SHA-256
values and Git identities. Its SHA-256 is
`0e2fde929315e3596e6d6a309206d8589aff69c1878209a2cfa8fcef09ae798e`.
The review commit binds both review files. My own standard-library data readers
confirmed these comparisons without running supplied source or checkers:

- All 13 candidate files match the candidate commit. All 12 manifest entries
  match. All nine JSON files parse. All 35 declared dependencies match their
  exact Git blob IDs, modes, lengths, SHA-256 values and this review checkout.
  All 19 R9 package files and its 18 manifest entries also match.
- All seven patch hunks reconstruct from exact R9 text in memory, with exact
  old/new positions, counts and removed bytes. No patch was applied on disk.
  All eight resulting module hashes and line counts match `source-checks.json`.
  The other six modules are byte-exact R9: controller, exec wrapper, process
  reader, ownership, probe and watchdog. No full source module was materialized.
- All 702 non-docs source records match the complete raw Git selection, bytes,
  lengths and executable modes at the product commit: 4,773,386 regular bytes.
  The exact source-bindings file retains its archive, Python, framework and
  workspace information. Its SHA-256 remains
  `20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617`.
  This is a raw-Git comparison, not an inspection of the live M1 source tree.
- All 87 original evidence files match their index, immutable original commit
  `498190feb00f51039f500ae618ac7de5fb113d86`, and this checkout. All 21 selected
  ordinary limits equal the original predeclared values.
- The retention/export receipts, both archives and all 44 evidence members
  match their recorded lengths and hashes. The evidence archive has no extra,
  missing, duplicate, link or special members and equals the retained raw files.
  The three saved build recipe files equal the exact reviewed build candidate.
- All 167 bundle inventory records match archive member types, modes, file
  lengths/hashes and link text. The archive has 168 members including its root:
  root mode0700, 95 regular files, 10,050,709 regular bytes and 9 symlinks.
  All nine link chains resolve within the archived app as data. No extraction
  occurred. The app archive SHA-256 is
  `7eb92de1ae6af988f8925546624a9f69fe20309aa8ea15f13fcf0b9ddf1cea21`.

## Focused common/prepare assessment

Line numbers in this section refer to the reconstructed source text, whose
hashes are bound in the JSON. R9 source references below use its unchanged files.

`common.py:13,17` changes only `REPO` and `PYTHON` to the two required parent-bound
environment values. An exact text comparison confirms every function, constant
and class remains unchanged. In particular, no deadline, byte reservation,
locked pre-write charge, failed-charge retention, first-record rule, bounded
reader, NI15 assignment or rename-only archive rule changes. Its new SHA-256 is
`8bcb99e34baef85ef021c558d6f2b8882376cb27f598ba08726710bf4e161a2d`.

`prepare.py:12-99` replaces obsolete historical input selection. It binds the
operation to the exact artifact receipt digest; requires the current build,
review, source and app/daemon hashes; requires all ten named artifact gates;
and binds exactly nine primary evidence references. All five fixed build-record
hashes match the real retained records. Their success fields and exact recipe
digests agree with the saved data. A recorded completed build stays acceptance
unverified. The new app hash is
`094d02b3831ee6565ae1628407756ad48053a0268f2a489c65bbde28b768d775`;
daemon and peer use the same executable, hash
`17065c334290b34c7a1b69483f409655e072eb0505ff2a0cc0bf6ed5f77a7bb2`.

Evidence refs require absolute paths, regular non-symlink files, exact lengths
and hashes, and the existing 8 MiB file ceiling. Source/bundle/Python launch
paths must be existing canonical absolute paths without dotdot. Environment
source/Python values must match the receipt. The interpreter hash agrees with
the saved qualified-Python binding. The prefix digest agrees with the executed
build operation. These saved bindings do not establish current prefix integrity
or authenticate an interpreter after it has already entered Python.

The pinned full source inventory is checked for exact file-set equality and
all 702 raw bytes/modes, without symlink or CRLF normalization. The source bundle
must equal the exact successful build's full inventory, including directories,
symlinks and modes. Its original build path remains separate from a transferred
execution input path. No old missing bundle, 29/397/124 set or old compiled app
hash is needed by the new admission path. Unused historical constants and
`original_commit` remain lineage; `source_commit` labels the fresh source.

`prepare.py:102-160` is exact R9 main text after four declared substitutions:
receipt/inventory selection, private-root prefix, environment/limits inputs,
and fresh-baseline metadata. This separate text comparison preserves NI15,
the 120-second file-I/O deadline, exclusive byte ledger and preparation guard,
private directories, same-filesystem requirement, symlink-preserving copy,
source/copy tree checks, ordinary plist checks, executable hashes and output
publication. Its new SHA-256 is
`16f2c5d9dce3f3bce8a871bd53eea06ebf1818080de5f9bdaae3997b86304e9a`.
The new environment carries the source/Python values to wrappers and watchdog,
disables user Python site startup and updates, and retains private HOME/temp/cache
paths. The unchanged controller supplies private config/TLS/socket paths,
virtual platform, discovery-off, no-render and explicit port24800 denial.

## Receipt, ordinary metadata and qualification boundaries

I read the complete schema, templates, operator plan and receipt contract.
All 39 internal schema refs resolve. Closed object properties equal their
required sets. The artifact schema requires ten true gates, all nine evidence
refs and supporting evidence. Its unresolved template and disabled operation
cannot meet admission. The ACCEPTED branch of the final-run schema requires
all twelve gates true and all thirteen mandatory file refs non-null. Failed or
unverified records can retain explicit nulls. No synthetic receipt was made and
no supplied or third-party schema checker was run.

The schema is deliberately a data contract. The patch does not implement real
signature, entitlement, prefix or ABI qualification, generic schema validation,
or all operator path/cap rules. This is explicit in the package, not an implied
machine check. The parent must inspect the bound raw evidence and apply the
full contract. Truth-valued fields and matching file hashes alone do not prove
these gates. This review does not convert that manual boundary into a new
human permission request.

The actual archived main plist equals the immutable source template after the
current `configure-plist` transformation: version0.1.0/build1, BundleID
`dev.openergy.app`, executable `Openergy`, type APPL, minimum15.0,
`SUEnableAutomaticChecks=false`, no feed URL and no update public key. Other
template fields remain exact, including `SUAllowsAutomaticUpdates=true` and
`SUAutomaticallyUpdate=false`; the contract does not require rewriting them.
All five archived Info.plists lack private-fixture metadata, and the full tree
has no private launcher. Fixture-related source remains compiled in the product;
its presence is not a fixture-mode admission failure. The ordinary entrypoint
uses `--e2e`, and the updater source excludes that mode from normal startup.

Parent gates correctly require actual arm64 app/daemon code, the embedded
Sparkle arm64 slices and full architecture lists/load paths. Upstream Sparkle
fat objects do not have to become thin. Sparkle2.10.0, its pinned revision and
binary archive checksum remain in the unchanged build bindings. Qualification
must retain strict bundle/per-object signature checks, actual CDHashes,
requirements, identifiers and full entitlements, then compare source,
destination and private-copy results. Ad-hoc signatures are not trusted-release
or Gatekeeper acceptance. No signing repair belongs to this handoff.

The full bundle inventory excludes the bundle root. The operator gate explicitly
requires root mode and copied-bundle qualification before product launch.
`prepared-copy-qualification.json` is an operator gate; the unchanged controller
does not consume it. The parent must record it through the existing writer and
stop on failure. The controller separately rechecks source receipt/bindings and
copied full-tree bytes before launch (`controller.py:386-450`). This division is
explicit and adds no new harness or recovery path.

The parent must authenticate the complete private Python3.13.9 prefix before
entry, use the real canonical executable, and control startup environment and
bytecode. It must qualify R9's real M1 libproc/BSD/KERN_PROCARGS2 interfaces.
The unchanged declared BSD layout is 136 bytes with microsecond starts; source
layout inspection does not establish actual SDK/kernel agreement. No ABI,
signature, entitlement, dependency tool or current machine path was queried here.

## Preserved R9 contracts

The full eight source texts and full independent R9 review were read. Exact
six-module preservation and the two narrow source changes leave these repairs
and limits in place:

- Actual interpreter executable/hash and wrapper argv0 remain distinct from
  launch-path binding. Executable/argv stage coherence and final BSD identity
  checks still precede detail publication. Reused/uncertain strangers do not
  gain argv access, ownership or signal authority.
- Durable launch intents precede spawn; retained handles precede post-spawn
  evidence. All-handle polling, abort state and output draining remain independent
  of evidence publication. Sealed intent/spawn/handle/PID/actual-OS-exit sets must
  close. Watchdog terminal-read errors cannot suppress actual-loss detection,
  the one fallback assignment or its recovery steps.
- Same-start wrapper/product transition after controller loss still needs its
  exact prior gate and explicit controller ESRCH. Recovery signals use fresh
  single-PID identity/parent checks. TERM's full five-second grace starts at
  successful delivery, and KILL requires a later advance inside the same deadline.
- Uncertainty lock, restore, durable begin/checkpoint, original pending clocks,
  negative registered-start index, sticky loss/overflow and exact revision guards
  remain in main, watchdog, fallback and cleanup-launch Inventory lifetimes.
  Reparenting or path-read failure cannot erase detected uncertainty. No state
  reset can grant peer permission. Peer stop still requires full independent
  app/supervised-tree absence and a fresh guarded wrapper release.
- One no-render default25 `--e2e --require-peer` flow, one launch per role,
  actual whole-flow NI15, 120-second flow and at most60-second cleanup remain.
  The product end stays start+180. Keep 6000 rounds/24000 rows, main4976/19904,
  cleanup1024/4096 and eight observations/two seconds per uncertainty.
- Keep 8 MiB/file, 32 MiB aggregate, eight autonomous2 MiB product slots,
  4 MiB cleanup including1 MiB terminal, and64 KiB accounting inside that total.
  New parent evidence uses the same reserved writer; initial inputs are charged.
  Journal capacity can fail before nominal count maxima fit; no cap is expanded.
  Archival remains a same-filesystem rename after verified recovery.
- Keep the original 16 MiB frames, 1,000,000-byte stalled write, 300 ms request,
  1 s responsiveness, 2 s stalled bound, 3/5/10/15 s waits and1.5 s supervisor
  stop. `AppEndToEnd.swift` remains exact historical ordinary bytes, SHA-256
  `cf3bc66eec68fbf0f508e487773f280759529f0a49782525c92522a10ed4f357`.

## Saved build scope, remaining proof and work limits

The saved independent build audit at
`3d176f04184ba90e9922e0c309458696592ddc5e` was read in full and bound, including
its timing correction. The saved metadata supports the stated M1/macOS27.0.1
build26A434, Xcode27.1/27A9275, Swift6.4/SDK27.0 and Rust/Cargo1.99.1 baseline.
The saved 2008 build rows have NI15. They prove no future-flow priority.
**101.050310709 seconds is worker-operation time, not controller duration.**
The reported103.8202-second SSH envelope is a parent report; exact full build
controller duration and a complete outer build OS-exit receipt are not retained
among the 44 files. No missing fact was filled or new build requested.

Preserve the rust-objcopy SIGABRT/missing `@rpath/libLLVM.dylib` warning and both
Swift deprecation warnings. No successful stripping claim follows. The export
did not preserve timestamps, ACLs or xattrs. Byte/mode/link identity does not
replace real signature, entitlement, ABI or runtime qualification.

Parent alone owns actual artifact/source/private-Python/ABI qualification,
preparation, private-copy qualification and the one real ordinary run. Actual
process identities, both daemon starts/groups, stopped/late/recovered NI15,
all direct OS exits, all 25 unchanged passing rows, peer retention and final
known PID/start/group/socket absence remain **UNVERIFIED**. Schema conformance,
report text or `outcome.passed` cannot replace the complete evidence. All 87
originals remain **FAILED / NOT ACCEPTED, 23/25**. The expired Sparkle operation
stays **CLOSED**.

Only this new review folder was written. No candidate, original evidence,
product source or integration changed. Readers used inert file/Git/hash/JSON,
patch text, archive members and plist data. Reader path/glob/shape errors are
recorded in the bindings; they caused no candidate/evidence write or supplied
execution. No supplied module/helper/checker/AST was imported, evaluated or
executed; no AST was needed. No remote/network, OS/process/port query,
preparation, product/helper launch, build/compiler, signing, test/mock/simulation,
desktop/native input, TCC, credentials, update operation, delegation, push or
publication occurred. No TypeScript changed. Hooks and commit signing are
disabled for the narrow review commit. This record grants no native, update,
trusted-release, universal-containment or host-restoration acceptance.
