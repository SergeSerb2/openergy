SOURCE-ONLY correction. No corrected or saved task program was run. Native Windows
virtual Off stop remains FAILED / UNVERIFIED from the original attempt. This is
an implementation handoff for a fresh independent source/evidence review.

Only `windows-stop-controller-repair-r2-dbcc0160/` is owned. The checkout started
clean at `e68432f9d12ad74fecd679f022beb1494e7028ff`. Production files match the
artifact source across the saved export paths. The original owner, evidence,
private artifact, Windows roots, and guards were not changed.

The six files in [source/](source/) are inert UTF-8 text, with `.txt` suffixes.
They contain the complete proposed controller and its supporting source.
[repair.patch](repair.patch) is an exact unified diff from the saved six-file
baseline. `common.ps1.txt` is byte-identical and has no patch hunk. No product
source or build helper is part of the patch. [inputs.json](inputs.json) binds the
exact original files, their Git blobs, sizes, and hashes. [SHA256SUMS](SHA256SUMS)
binds this handoff. [static-validation.json](static-validation.json) records only
data parsing, source comparison, and patch materialization.

The complete original README, `runtime.ps1`, `common.ps1`, `observer.cs`, and
`controller.py` were read as data at evidence commit
`6010722042604f77842db3ce491bee42f52a1265`. The qualification and collection source,
actual first runtime error, runtime/build/cleanup results, manifests, and selected
preservation receipts were also read. Their owner is
`/Users/serge/.t3/worktrees/openergy/t3-c8a3489c`, folder
`docs/verification/release-integration-2026-10-23/daemon-stop-native-windows-c8a3489c/`.
The artifact source is `e1ef1243988d072cb9afc33fd2f5a56ca70e2f5a`, including
production stop fix `2485c142` and source/evidence review `63915871`.

The original evidence remains as recorded:

- One strict native release Clippy passed in 20.854 seconds. One release build
  passed in 83.494 seconds. They used installed Rust/Cargo 1.99.1, locked/offline
  resolution, jobs 2, MSVC 14.44.35207, SDK 10.0.26100.0, and the native MSVC target.
  The sampled process chains were BelowNormal/session 0. No release linker was
  sampled. Its priority remains UNVERIFIED.
- The AMD64 PE32+ executable is 4,139,520 bytes, SHA-256
  `4984ef1222bbcd551dbebdda9db2fc623ff42a3560c5261806f09ad8f3dbda2f`.
  The build-stage `artifact.executed=false` is not a claim that the runtime never
  executed it. Version exited 0; absent-daemon status exited 1. The foreground
  private virtual role-Off daemon then started.
- The one flow failed at original `runtime.ps1:115`, while `File.ReadAllText`
  opened the active `runtime/config/openergy.log`. The saved error says the file
  was in use by another process. The saved daemon log contains the expected
  private listening marker. This is a reader defect, not a demonstrated product
  stop failure.
- No healthy public status/config, invalid-method request, or stop request ran.
  There was no stop acknowledgement, wire-ID acceptance, normal exit code, normal
  stop measurement, or running-Off TCP/UDP inventory. The 10-second normal-stop
  deadline never began. Before/after exclusive binds are not a running inventory.
- Owned recovery followed the readiness error before the 15-second ceiling. The
  forced recovery exit code was not recorded. Its generic
  `only_after_declared_deadline=true` field is not normal-stop deadline evidence.
  The saved final audit found the four recorded identities gone, private pipes
  absent, and the fresh Windows root removed. Saved selected old fixtures,
  registry/toolchain/default/tool states, live-sharing identity, and local prior
  evidence matched. These are historical receipts, not fresh host queries.
- There was no flow retry. The private artifact and final 46,836,736-byte bundle
  remain available locally. The bundle SHA-256 is
  `3247d9f23cd5fb078c7c048de7f5ae48e4d71614ab4cd5dcc5a39658d7ea6908`.

The defect response follows the actual writer. At the artifact commit,
`crates/daemon/src/logger.rs:30` opens with create/write/append and has no Windows
share-mode override. Rust 1.99.1 documents the default as read/write/delete
sharing. This writer's existing write access also has to be allowed by the new
reader's share flags. The Windows API documents that incompatible sharing can
reject a new open. See [Rust OpenOptionsExt](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode)
and [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).

The new reader opens one stream with `FileMode.Open`, `FileAccess.Read`, and
`FileShare.ReadWrite | FileShare.Delete`. The .NET constructors expose access and
sharing separately; granting write/delete sharing gives this reader no write or
delete access. See [FileShare](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileshare?view=net-9.0)
and [FileStream constructors](https://learn.microsoft.com/en-us/dotnet/api/system.io.filestream.-ctor?view=net-9.0).

The reader scans forward once from offset zero. It reads at most 4,096 bytes per
poll, caps both observed file length and cumulative read offset at 2,097,152
bytes, checks for shrinkage and unexpected offsets, and keeps only the marker
overlap between chunks. The cap comes from the product logger's 2 MiB rotation
threshold; it is a harness refusal bound, not a new product limit. It does not
seek, reread, follow rotated files, or issue a product request. A missing file
alone can remain pending. Directory, access, sharing, and other I/O faults remain
explicit. The pre-existing private config directory must already exist.

Readiness has one absolute QPC deadline, 15 seconds from immediately before daemon
resume. Every poll and asynchronous read uses the remaining part of that same
deadline. The controller rejects completion after the deadline. The marker is
only a launch signal: the unchanged subsequent real public `status` and
`config.get` checks still establish healthy virtual Off acceptance. The logger
also writes to stderr and can disable its file sink after an open failure. This
repair does not switch readiness to stderr or accept an absent log as healthy.

The adjacent source corrections are limited to the same native flow:

- Native child handles enter the owned list while the child is still suspended,
  before fallible managed process lookup or job-membership observation. The
  existing BelowNormal creation flag, explicit stdout/stderr/NUL handle list,
  direct parent, creation identity, exact executable, session 0, and resume order
  remain required. An unresumed owned child has an explicit native-handle cleanup
  path. No job, process group, desktop, or window-station change is added.
- CLI creation, identity work, resume, exit observation, and observer completion
  share one absolute 15-second deadline. The old extra 2-second observer wait is
  removed. The observer still forwards real frames unchanged. Acceptance requires
  the exact method, non-null matching serialized wire ID, and real CLI/daemon
  PIDs. No substitute response or additional product call is added.
- The exit observation must precede the absolute 10-second deadline from the
  observer's pre-forward tick. A late first observation cannot prove an earlier
  exit. Failure recovery first settles the observer, reads its final forwarding
  tick, and preserves the full allowance before any daemon kill. If the observer
  cannot settle, the source refuses a premature kill and records unresolved
  ownership. Forced recovery records its reason, retained-handle signal, and
  actual exit code. All observed owned final exit codes are retained separately.
  Forced exit cannot count as normal acceptance.
- Live CLI output hashes are left null on timeout; hashing an active output sink
  must not replace the first flow failure with another sharing error. Completed
  CLI output checks stay unchanged. Native TCP/UDP table errors remain explicit;
  only rows for the exact owned daemon PID enter evidence.
- The new Python controller reuses the verified retained binary and immutable
  build receipt. It has no compiler stage and no old-guard reset. Qualification
  still verifies the immutable source export, transfer bytes, and selected shared
  snapshots. It omits compiler version commands and the registry copy. The
  existing postflight stage additionally checks saved PID/creation generations
  and the two private pipes after the runtime controller exits. Collection must
  verify the executable and original receipt before removing only the new root.
- The outer controller reserves its existing 30-second recovery allowance inside
  its one 2,400-second budget and prevents recovery recursion. Its runtime wait is
  `150 + 30` seconds; native runtime acceptance still has its original 150-second
  ceiling. The 40-second stop-entry reserve is the sum of existing CLI (15),
  normal-stop (10), and native recovery (15) allowances. These sums are controller
  scheduling reserves, not new stop contracts.

The retained inventory was checked only at the exact paths named by the original
private-location receipt. It includes the executable, final bundle, source tar,
source manifest, original state metadata, and build receipt. No transport
selection, endpoint, credential, key, home-directory inventory, or live host state
was read. [retained-inventory.json](retained-inventory.json) gives relative paths
and byte/hash results without the private root prefix. Reuse is the selected next
path. Missing or changed bytes are a prerequisite failure; there is no build
fallback.

The fresh review must assess the complete corrected source and
[NEXT-RUN.md](NEXT-RUN.md), including the binary-reuse adapter and recovery
ordering. Broader matters remain outside this correction: external SSH-job and
handle-table accounting; security/provenance and product pipe ACL acceptance;
the unsampled release linker; hardware, migration, UI, DNS/discovery, and release
acceptance. QPC checks and asynchronous reads do not prove that arbitrary Windows
metadata, CIM, process creation, or filesystem cleanup calls can be preempted.
The inherited private stdout/stderr disk sinks have no byte cap; this correction
does not establish a total disk-use bound. The outer guard supplies bounded
transport containment; real completion and
cleanup must be established from the future receipts. No OS-level hard real-time
claim is made.

Only JSON decoding, Python AST parsing without import, immutable file hashing,
exact source comparison, and in-memory patch materialization ran for this repair.
No PowerShell parser/import, C# import/compiler, task helper, program, preflight,
build, test, mock, simulation, socket/pipe probe, process/OS query, remote command,
signing, or non-documentation network action ran. PowerShell/C# syntax and the
corrected runtime are UNVERIFIED. There is no `SOURCE READY`, one-second stop,
cross-platform performance, or release acceptance claim.

Live Mac/Windows sharing, installed apps/config/TLS/default IPC, TCC, login items,
services, tasks, accounts, firewall, global DNS, toolchain defaults, selected old
fixtures, and shared state were not changed. There was no TCP 24800/UDP 5353,
native input/clipboard, desktop/window-station action, new job/process group,
Docker, credential/private-key access, delegation, push, or publication. Crafted
mDNS, the original provenance/security review, IPC/Synergy 3 import budget, and
the isolated Swift build gate remain stopped. Running Sparkle/relaunch approval
is separate and was not touched.
