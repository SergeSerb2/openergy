# Private update bootstrap: independent R4 source and saved-build review

Reviewed on 2026-10-23 (America/Vancouver). **No concrete P1 or P2 finding was
found in this bounded review. Both prior P2 findings are repaired in source.**
This result does not approve integration, a launch, an update, or a relaunch.
There is no independent runtime reproduction in this review.

## Exact scope and ownership

- Candidate: `7287e9aa4caf6570fc4116a89908d548a258f302`.
- Actual candidate parent: `8826c9570d6ffe1c6d8f744e82a6aedd28bba4d7`.
  That parent is the held `a0342aef6b4e4712e7bf73c992b788f26d0e98da`
  change applied on `1fd5144628c0494f1ebf561581e0d84fd7231523`.
  The held and applied macOS source/original-evidence paths compare equal.
- Review checkout: `/Users/serge/.t3/worktrees/openergy/t3-3eaee08d`, branch
  `t3/review-private-update-isolation`, initially clean at
  `f97939aebb130cd1c13b511e880326154692aeea`.
  Candidate files were read as exact Git blobs. No candidate checkout,
  cherry-pick, or integration occurred here.
- Read the complete original
  `docs/verification/private-update-bootstrap-2026-10-23/README.md`, the complete
  R2 review at commit `62969e571f96ac7c3ba0d7a82bf4069544d24e46`, and the new
  `docs/verification/private-update-bootstrap-repair-2026-10-23-t3-54de37ad/README.md`.
  The R2 review SHA-256 recomputes to
  `ec29c95890a4d61f219428177fdb2c38505e2ff1f48b96527b12d29f38f204c5`.
- Read all 300 launcher lines, all 39 template lines, all 146 packaging lines,
  the complete 463-line Swift helper, the inherited dispatch, and relevant real
  AppModel, DaemonSupervisor, updater, and production packaging source.
  Source line references below refer to the candidate.

This review owns only this new document. All prior source, raw evidence,
manifests, failures, and private artifact bytes remain unchanged. Read-only analysis
used Git blobs, JSON/hash comparisons, and direct reads of retained artifact and
cache bytes. No saved controller, packaging implementation, app, launcher,
daemon, product CLI/helper, installer, build, compiler, unit test, or mock ran.
No network, process/port/OS assessment, desktop/computer use, keychain, secret,
signing key, fixture creation, installed-app action, live sharing, production
config/TLS/default IPC, TCC, login/service/task/account/firewall/global DNS,
toolchain-default change, Docker startup, push, or publication occurred. No TCP
24800 or UDP 5353 activity occurred. Rust and TypeScript were not invoked.
No further delegation occurred. The three provider-stopped investigations
remain out of scope.

## Response to every prior finding

### R2 R1: failed whole-plist load — repaired in source

`PrivateUpdateFixture.swift:23` now requires a non-nil
`Bundle.main.infoDictionary`; failure calls `refuse` and exits 78. It no longer
converts that failure to an empty dictionary. Lines 25-28 still permit an
ordinary, successfully loaded metadata-absent plist, while refusing a parsed
fixture identifier with missing metadata. Present invalid metadata cannot fall
through to normal startup.

The supported private bundle entry also has an independent refusal path.
`launcher.c:225-247` reads the actual derived `Contents/Info.plist` and requires
the complete exact template plus validated fields. Missing/unreadable bytes,
binary or malformed plists, absent private metadata, alternate formatting,
truncation, and extra bytes fail this entry instead of selecting production
dispatch. `read_file` at lines 65-86 checks a canonical path, no-follow open,
current UID, regular type, one link, allowed mode, size, complete read, and no NUL.
The launcher has no production fallback. This correction is executable source,
not acceptance of the original README's malformed-plist exclusion. The repair
README expressly supersedes that exclusion; the historical README is preserved.

No malformed-plist launch or LaunchServices behavior was reproduced here.

### R2 R2: framework-bearing first image — repaired for the supported entry

`Info.plist.in:5-6` selects `OpenergyPrivateLauncher` as `CFBundleExecutable`.
The fixture's first image is separate from `Contents/MacOS/Openergy`.
The complete C source has no Bundle, CF, Foundation, AppKit, Sparkle, or
subprocess call. Its saved and retained Mach-O bytes have one dylib load:
`/usr/lib/libSystem.B.dylib`.

`launcher.c:183-273` validates entry arguments, identity, paths, metadata, safe
config, marker file safety, and the empty lock file. Lines 278-294 form a fresh
15-entry environment: HOME, CFFIXED_USER_HOME, TMPDIR, both XDG paths, the eight
explicit OPENERGY path/control values, PATH, and LANG. No inherited environment
array is passed. Lines 295-298 enter Root and call `execve` on the fixed protected
Swift image with the exact internal arguments. That environment therefore exists
at the protected image's kernel/dyld load boundary. There is no first Swift image
that must repair HOME and re-exec. Both initial and later no-argument bundle
entries select this same source path when both fixture versions carry it.

The launcher sets and verifies absolute nice 15 and umask 077 at lines 187-191.
The Swift helper requires the internal argument pair at line 33 and verifies
the already-private environment, cwd, umask, and nice at lines 292-311. It no
longer calls setenv/chdir/setpriority to repair them after framework load.
A direct no-argument protected-image entry with valid private metadata refuses;
it is not the supported bundle entry. Internal arguments alone do not activate
private startup in an ordinary metadata-absent production bundle.

This addresses the source ordering objection without narrowing the original
protected-image requirement. It does not prove zero host effects from dyld,
libSystem, frameworks, LaunchServices, logging, or external Sparkle helpers.
It supplies no hostile same-UID mutation/injection guarantee or OS sandbox.
Bundle.main resolution after exec and the actual no-argument relaunch remain
runtime acceptance items.

## Complete source checks and limits

- **Exact parser and fields.** `launcher.c:117-144` compares every literal template
  span and final suffix. Each of the six fields must occur exactly once and be
  nonempty and shorter than 512 bytes. The fixed template carries no alternate
  executable, root, schema, or feed selection. ID and Root must agree with the
  actual derived target location. UUIDs must be lowercase and distinct. Build
  and port must be canonical positive decimals within their bounds. Root uses
  the restricted ASCII path alphabet. The public-key check requires canonical
  base64 syntax for 32 bytes; it is not key trust or signature validation.
  The field alphabets prevent XML delimiters/entities from being substituted.
- **Paths and owners.** The launcher derives Root from its resolved fixed entry
  suffix, requires the non-root real/effective UID and matching GIDs, and accepts
  only the original two canonical temporary parents. Root/target/private
  directories require UID ownership and mode 0700. Executables are canonical,
  owned, single-link regular files with owner execute and no special or
  group/world write bits. The bundle walk rejects unsafe types, hard links,
  unsafe modes, broken links, and resolved targets outside the bundle. Private
  trees permit only 0700 directories and 0600 single-link regular files without
  symlinks. Both traversals preserve the 8192-descendant and 2047-byte path
  bounds. This is path validation, not resistance to concurrent same-UID edits.
- **Config and refusals.** Socket absence and the less-than-104-byte socket path
  bound are required. Config must equal the fixed off-role/literal-loopback,
  empty-remote, no-autoconnect, no-TLS, no-clipboard bytes, including the final LF.
  An existing socket is refused without unlink or kill. Marker file safety,
  readability and size are checked in C. Its JSON identity, allowed two builds,
  finite timestamps and at-most-21600-second entry lifetime are checked again in
  Swift at lines 124-135, under the private environment and before the model.
  Invalid marker meaning can thus load the private Swift/framework image before
  refusal; it does not reach normal/model startup. No automatic shutdown at the
  marker deadline is implemented or claimed.
- **Packaging parser and writes.** `package-entry.py:41-71` checks bounded thin
  64-bit little-endian executable Mach-O headers, supported CPU, PIE, bounded load
  commands, the sole libSystem dylib load, macOS/deployment 15.0, and forbidden
  alternate dylib/rpath/dyld-environment commands. It does not authenticate an
  arbitrary Mach-O by its load shape. Exact source/artifact identity remains a
  separate future preparation prerequisite. Lines 79-127 require an already
  private unsigned bundle at the fixed `/private/tmp` target, owned canonical
  paths, exact four-key metadata, ID/root/build/nonce/port/public-key agreement,
  and the original protected executable name. An ordinary production plist
  cannot pass those gates. All input checks precede exclusive output at lines
  129-141. Launcher copy and pending plist use exclusive files; the final rename
  replaces the existing private plist. Partial preparation can leave owned files
  and is explicitly not launch acceptance. The helper creates no root, marker,
  config, key, feed, archive or signature. It is not called by normal bundle.sh.
- **Real model/lifecycle.** The launch lock remains exclusive/nonblocking and
  CLOEXEC. Swift validates both the declared launcher and the actual protected
  image. Explicit NSApplication/model use follows validation and environment
  verification. `AppModel(demo: nil)` retains its existing `--e2e` integration
  guard; the real supervisor receives the private daemon/config/socket and
  virtual-platform arguments. Readiness, 15-second deadline, PID/status/config
  checks, receipt writes and quit/shutdown code are unchanged by this repair.
  There is no AppUpdater singleton, normal SwiftUI scene, AppEndToEnd dispatcher,
  or update/install operation on this private path.
- **Receipts and child exit.** Receipt creation remains exclusive, no-follow,
  directory-relative, 0600, bounded to 4096 bytes, and file-fsynced. Partial
  receipts must fail consumer validation. Authentication and directory crash
  durability are not claimed. DaemonSupervisor.stop still has a SIGKILL path
  without a subsequent join. A receipt, nil PID, shutdown return, app exit, or
  CLI result does not prove child exit. Independent old-child PID/start/path
  observation remains required and is not replaced by this review.
- **Production activation boundary.** Recomputed comparisons confirm all nine
  protected production/package files are byte-identical to `1fd5144`: AppUpdater,
  AppEndToEnd, UpdaterEndToEnd, AppModel, DaemonSupervisor, Package.swift,
  Package.resolved, production Info.plist, and bundle.sh. OpenergyApp.swift remains
  byte-identical to the held first-dispatch change. The normal plist still uses
  `dev.openergy.app` and `Openergy` and has no private metadata. Production feed,
  public-key, mode, updater and lifecycle gates remain intact in source. The
  fixture template has a public-key placeholder and no feed URL. No concrete
  key was created/read. This is not a restart of the stopped release-security
  review or a production runtime claim.

## Recomputed saved-evidence bindings

All values below were recomputed from bytes in this review, except where a saved
observation is explicitly identified. Hash equality is evidence preservation,
not independent execution of the behavior described by a record.

- Repair manifest: exactly **56** raw paths, no missing or extra raw paths,
  every size and SHA-256 matches. Manifest SHA-256:
  `6a5015dd75d57704ce80735a96c7de1d876bfc39fbe0306b1176dc39c2aff323`.
  Ordered compact/sorted-key record inventory ID:
  `929587353dfe4f5ebb9b79590225597ea5eeda9bcebead5be423e75d49ac577a`.
- All **29** build-source records match the candidate, including all **22** Swift
  app source files. Canonical source inventory ID:
  `69453dd326f6b0309dabb0bf9e8d9cab882afca92912e0e4f48543fc8765595e`.
  All four final owned-source Git blob IDs, sizes, and SHA-256 values match.
  Recreating the generated header as data gives exactly the committed header:
  `26269a77826b4b158d7da9eccb29c25ad91019faafdeaa303e310abc9762bdad`.
  The retained launcher bytes contain the exact template plus NUL.
- All **22** original evidence files, including README, manifest, and **20** raw
  records, equal the held change byte-for-byte. The original manifest's sizes,
  hashes and complete raw path set match. Its SHA-256 remains
  `bcae8d493b8eaf88b0745ef39c4283e462425c6859e80f962f2f3faaca780c78`;
  its inventory ID remains
  `b7f7976f721697e1a9f3820d2e1377c46d66311c077d1154faf7f6b261f9bf08`.
- Both saved compiler results record exit **0**, no timeout. Direct compiler
  stdout/stderr hashes match their result records and the manifest. The commands
  select the outer `(deny network*)` profile, private allowlisted build
  environment/cache/scratch paths, native arm64 deployment **15.0**, locked
  dependencies, jobs **2**, and the whole Openergy product. Automatic resolution,
  updates, prefetch, netrc and keychain are disabled. These are audited command
  records, not a new network observation. Sparkle remains pinned at 2.10.0,
  revision `eef1a539a373c1f1a320624b1130fc5de7b2e100`.
- Saved launcher elapsed time: **0.414777 seconds**. Saved app elapsed time:
  **25.784859 seconds**; stdout reports product completion in **24.42 seconds**.
  App stdout names all 22 inventoried Swift app files and DEBUG/arm64/macOS15
  compilation. Recount: **901** sampled observations across **29** sampled PIDs
  (5 launcher observations/3 PIDs; 896 app observations/26 PIDs), all nice **15**,
  including **355** swift-frontend and **7** clang observations. The controller
  source uses absolute setpriority and records its own nice 15; it does not use
  nested relative wrappers. Sampling can miss short children. No current process
  priority or process existence was inspected.
- Clang has no source warning/error diagnostic. App streams retain the repeated
  helper deprecations at lines 109 and 147, SwiftPM deprecation/verbose manifest
  output, and no compiler error diagnostic. The manifest tooling's macOS14 target
  is distinct from the app's macOS15 target. The saved app stdout includes normal
  ad-hoc `codesign --sign -`; this is not signing-key use or release trust proof
  by this review.
- All six final artifact-audit commands bind to the correct preserved r3
  stdout/stderr streams. Direct reads of the retained private artifacts independently
  match their recorded sizes and hashes: launcher **35,416 bytes**,
  `e1f024c4c919021d3976f92d59491f904f86107421c705e36cbf583509cc4c5c`;
  protected app **6,191,952 bytes**,
  `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`.
  Parsing their bytes as data confirms arm64 executable/PIE and macOS15 minimum.
  The launcher has 17 load commands, LC_MAIN, the sole libSystem dylib load, and
  no rpath or dyld-environment command. Neither artifact was invoked.

## Cache audit and retained failures

The saved original-before, initial-copy, and original-after inventories are
byte-identical and each has **777** byte/symlink records. They also match the
held inventory. The private copy's post-build inventory differs only in
`checkouts/Sparkle/.git/index`: size stays **69,503**, SHA-256 changes from
`e3900ec0ca6eace576754e3d560e23420f258e39d8a16d400647a37086b47ed6` to
`b2390058613c0e95c278d64873abdfbce063946edb11ae075d2d822e61333b2b`.
The three saved reconciliation JSON files are byte-identical.

This review also read the retained original selected cache and retained private
copy as data. Both complete path sets and all 777 entries match their respective
saved inventories; none is missing or mismatched. Directly parsing each retained
version-2 Git index, without running Git against those checkouts, gives **602**
records and the same staged path/mode/object-ID digest:
`363ab5c9a643195363e3792570f004a6573ce3f5c163d77ebb2542a8c568ac74`.
Both index checksums pass. This confirms the stated byte/symlink and staged-object
limits. It does not establish inode, timestamp, ACL, or other metadata invariance.

The first controller still failed at build-once.py line 166 after both compile
results were saved. `raw/controller-completion.txt` retains exit **1** and its
AssertionError; it identifies its tool-completion text as a transcription and
the compiler streams as direct originals. The strict equality assertion was not
weakened or made green. Its unsaved in-memory prebuild workspace-state hash
cannot establish a before/after state-file claim. No compiler retry is inferred
from the later file/load-command audits.

The first postbuild script used filenames that differ only by `-l`/`-L` case.
The surviving first `otooll` records are byte-identical to the final dylib
listings. The overwritten first load-command bytes remain unavailable. This
review does not call later load-command bytes the recovered first output.
The first traceback remains. The second script uses exclusive output and
retains its FileExistsError on the old launcher file-command output; the records
written before that refusal remain. The third uses distinct r3 labels, and its
six final command bindings match. All three scripts and both failed-audit
completion records remain in the 56-record manifest.

The repair also retains both source-discovery failures and the Python confstr
availability check. The original evidence retains the interrupted-session
`Unknown process id 38332` and glob-discovery failure; they were not repeated as
runtime observations. The parent's separate 114-entry old-scratch removal and
restoration advisory remains explicitly parent-reported. This review did not
audit that unrelated attempt or claim metadata restoration.

This review's initial reads of the two candidate READMEs in the review checkout
returned `No such file or directory`; reading their exact candidate Git blobs
resolved the source location without changing checkout state. Its first
read-only hash script verified the repair manifest, then stopped with
`KeyError: 'records'` because the original manifest uses `files`/`file` instead
of `records`/`path`. The corrected reader completed the remaining comparisons.
No evidence file was written, overwritten, or repaired by either audit command.

## Acceptance boundary

The saved material supports one whole arm64 DEBUG app compile/link and one
launcher compile/link, with the disclosed failed overall controller and audit
attempts. It supports no assembled fixture, current daemon build, universal or
trusted signed package, positive/negative launch, isolation/refusal behavior,
private IPC readiness, child exit, official CLI update/install, or no-argument
relaunch acceptance. No build or runtime was repeated here.

Both prior objections are closed as source findings for this exact candidate's
supported bundle-carried entry. The future separately approved run still must
observe initial no-argument entry, environment before the protected image,
Bundle.main resolution, real model/owned private virtual child and readiness,
malformed-input refusals, independent old-child exit, replacement identity,
no-argument bundle relaunch, and final cleanup/host effects. External Sparkle
helpers and LaunchServices are outside the launcher's process environment
guarantee. Neither libSystem-only linkage nor these build records establishes
zero shared-host effects. Candidate integration and runtime approval remain
ungranted by this review.
