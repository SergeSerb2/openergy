# Parent receipt contract

Both templates are inert. The artifact template contains recorded new build
hashes and paths; its false/null qualification fields still block admission.
Nulls are unresolved data, never acceptable values.
`receipt.schema.json` describes two closed record shapes. Its file references
bind absolute paths, lengths and SHA-256 without embedding endpoints or secrets.
The schema is not an artifact checker. The parent must bind complete raw data
and assess the conditions below. Receipt booleans alone cannot prove a gate.

## Artifact receipt, before preparation

The current output now exists in the retained build data. Complete
`artifact-receipt.json` from that exact input and actual parent qualification. Keep
the successful build's original `bundle-files.json` and its original app path
in `build_bundle_path`. `source_bundle` is the qualified execution input, which
can have a different path after an exact transfer. Preserve every inventory
record and mode. App/daemon hashes are new measurements of these actual bytes.
The daemon executable is also the peer executable; do not substitute a peer.
`retained-build-bindings.json` pins the full original inventory hash, app and
daemon hashes, all 44 raw evidence records and both archives. It preserves the
stripping warning, scoped build result and export metadata limits. It does not
contain a complete outer build OS-exit record or current signature verification.

The evidence map contains these exact references:

- `build_result`, `build_worker`, `build_bundle_exit`, `build_operation` and
  `bundle_files`: unedited current build JSON records, bound to the reviewed
  recipe. The admission patch checks the success fields in the operation plan.
- `artifact_qualification`: parent's parsed ordinary bundle metadata and a
  per-code-object list of real read-only architecture/signature/entitlement/
  dependency observations with raw output references and actual exit codes.
- `python_qualification`: exact existing private Python prefix and executable
  qualification, version, mode, architecture and complete prefix digest. The
  known digest is a binding, not a fresh prefix measurement by this task.
- `abi_qualification`: M1 SDK/header/interface evidence for the unchanged R9
  process reader's `proc_bsdinfo`, libproc selectors and KERN_PROCARGS2 decoding.
  The R9 layout is 136 bytes with microsecond process starts. Header agreement
  and later real process identities are separate evidence; no simulated probe.
- `transfer_qualification`: explicit same-machine/no-transfer record, or exact
  already-retained archive and later extraction records with hashes, modes,
  symlinks and unchanged signature/entitlement checks. The existing export
  followed build closure; do not regenerate it. Timestamps, ACLs and xattrs
  were not preserved by that export. This declared limit stays separate from
  required exact executable signature/entitlement data.

Every referenced raw output also appears in `supporting_evidence`. The patch
checks the references' bytes and hashes. Keep each at most 8 MiB and the list
at most 128 records; cap tool output in memory before reserved publication.
Do not export credentials, environment dumps or private keys.

Required artifact gates mean:

- `successful_current_build`: all exact retained build success conditions in
  step 1 hold. The build status still says acceptance unverified.
- `source_702_raw_git_bytes_modes`: exact complete current raw Git inventory,
  source root, all 702 bytes/modes, legal resources, package closure and recipe
  agree. No historical 29/397/124 cardinalities apply. The pinned inventory
  retains its original archive/framework/Python fields, not a reduced rewrite.
- `runtime_artifacts_bytes_modes_symlinks`: complete source and any transferred
  tree agree with the actual build inventory. All executable hashes, mode bits,
  root mode, internal links, embedded code, resources and licenses are recorded.
- `arm64_app_daemon_and_embedded_slice`: app and bundled daemon are exactly the
  intended arm64 build. Embedded Sparkle Mach-O objects contain the qualified
  arm64 slice. Record actual full architecture lists and load-path resolution;
  do not demand that upstream framework fat binaries become thin or alter them.
- `signatures_entitlements_unchanged`: real read-only strict bundle verification
  and per-object verification succeed; retain identifiers, CDHashes, signature
  mode/requirements and full entitlements as data. Bind the actual app, daemon,
  framework/helpers/XPC contents. Parent compares source/destination/private-copy
  signatures and entitlements. No signing or named identity access is permitted.
- `ordinary_info_no_fixture_no_launcher`: valid plist, BundleID
  `dev.openergy.app`, executable `Openergy`, package type APPL, build string `1`,
  new version consistent with source, minimum system version `15.0`, no
  `OpenergyPrivateUpdateFixture` key and no private launcher. Match ordinary
  source template fields plus the exact current `configure-plist` transformation.
  The production executable may contain fixture-dispatch source; absence of
  the activating metadata and launcher is required. Do not edit compiled code.
- `sparkle_pinned_updates_disabled`: retain complete actual embedded tree and
  pinned Sparkle 2.10.0 revision/checksum from the immutable build bindings.
  Require `SUEnableAutomaticChecks=false` and no `SUFeedURL` or `SUPublicEDKey`
  in the generated plist. Record all other Sparkle values exactly, including
  unchanged source-template values. Do not insert an endpoint/key, run an
  updater, launch private fixture code or reopen the old Sparkle operation.
- `qualified_private_python`: exact prefix/executable metadata and hashes
  agree, startup is private, and the parent authenticated it before entry.
- `m1_process_abi`: the real machine/header/interface qualification supports
  R9's unchanged reader; actual runtime rows still must satisfy all predicates.
- `archive_transfer_integrity`: transfer/no-transfer receipt is complete and
  full tree/signatures/entitlements are preserved. File identity alone is not
  a trust, notarization or Gatekeeper result.

The parent separately records the copied bundle's qualification after the one
preparation and before controller launch. The artifact receipt stays immutable;
do not rewrite it after preparation to add a new copy path. Bind the new copy
receipt in the final run receipt. Its absence blocks product launch at the
operator boundary. R9's source/copy tree/hash checks still run in their original
places. The parent must apply the schema contract as well as the mechanical
admission checks; the patch intentionally does not implement a new generic
schema, archive or signature-validation framework.

## Final run receipt

Record `openergy.m1-ordinary-run.v1` after the single attempt. For failed or
unverified runs, preserve null/missing evidence explicitly in the declared
fields and state the first error in limitations. Never fill a missing fact
with an inferred PID, exit code, inherited priority or successful cleanup.
`ACCEPTED` requires every gate true and every mandatory evidence reference.

The mandatory records include prepared-copy qualification, session, app result,
app OS exit, collector terminal, main observations, watchdog ready/exit/absence,
recovery terminal, bundle after, outcome and the parent's actual controller OS
exit. Additional actor observations, identity/command/exec/registered/spawn/exit
receipts, journal/counters, signal records, original raw report, first errors,
pre/post ports and bounded-output receipts are retained as supporting evidence.
No file reference alone replaces inspection of its complete predicates.

Acceptance requires all of the following:

- All 25 unchanged original default/require-peer report rows pass, actual app
  OS exit is 0 and the report is complete. `AppEndToEnd.swift` is byte-exact to
  the historical ordinary input: SHA-256
  `cf3bc66eec68fbf0f508e487773f280759529f0a49782525c92522a10ed4f357`.
  No predicate, frame size, stalled-write/request bound or wait is relaxed.
- Complete actual OS exits close every direct command intent/spawn/handle/PID
  set. Include app, peer, peer-status and attempted peer-stop wrappers/products,
  plus separate watchdog and controller real OS statuses. No report-only exit
  inference. The outer controller cannot reap app-owned supervised children;
  retain R9's exact independent identity/ESRCH and unchanged supervisor routine
  proof for those children, without inventing wait statuses or universal proof.
- Actual whole-flow NI15 evidence covers controller, watchdog, wrapper/product
  transitions, peer, CLI waits, app, both daemon starts, real stopped interval,
  late/recovered operation and cleanup. Require NI15 before and after details.
  A single early sample or asserted inheritance cannot meet this gate.
- Exact initial/recovered daemon identities and parent chains, actual separate
  groups, observed real SIGSTOP interval and routine crash/recovery all appear.
- The peer stays until independent app and supervised-tree absence, with the
  inherited journal state and exact revision guard at permission and wrapper
  release. No unresolved token, pending clock, overflow, lost handoff, unclosed
  launch or stale revision can establish absence.
- Final known PID/start/group/socket absence, actual direct exits, watchdog
  exit/absence and original port observations are complete. Do not delete stale
  sockets or use a port result as a substitute for process absence.
- No abort, forced cleanup, fallback recovery, publication loss or failed budget
  occurs. Require successful normal recovery, empty recovery TERM/KILL delivery
  maps and no cleanup signal escalation. The routine's intentional stopped and
  killed initial daemon remains part of the required product predicate; it is
  distinct from forced cleanup. Recovery may retain `recovered=true` after a
  failed run; that never makes the run acceptable.
- Full original source and new signed bundle inventories remain unchanged.
  Source/destination/private-copy signatures and entitlements agree. All fixed
  time/count/byte limits, first records, rename-only archive and one-attempt
  rules hold. Parent outer output collection is complete and within the same
  ordinary evidence reservations.

`outcome.passed`, `report` text or schema conformance alone cannot accept the
flow. Missing any required fact is FAILED / NOT ACCEPTED or UNVERIFIED. Preserve
the first failure and stop. No retry, rerun, rebuild, signing repair, historical
payload search or new human approval cycle belongs to this handoff.
