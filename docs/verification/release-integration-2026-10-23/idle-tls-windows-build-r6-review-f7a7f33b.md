# R6 native Windows build: independent saved-evidence review

**No actionable P1/P2 findings.** The saved evidence supports one successful
native strict release Clippy invocation and one successful scoped release
build of the fixed source. This review did not reproduce compilation or
runtime behavior. Strict deadlines and fixed-source runtime remain
**UNVERIFIED**. The original immediate runtime cleanup remains **FAIL**;
its cause remains unresolved.

The exact evidence commit is `99bb7d68ed8977c328ed227dc13703a318a77a06`.
Its parent and this review's starting commit are both
`1fd5144628c0494f1ebf561581e0d84fd7231523`. The evidence commit adds only
72 files under `idle-tls-windows-build-r6/`. It changes no source or prior
evidence. Paths below refer to that directory at the exact evidence commit.

This review read local immutable Git objects, saved receipts, and private
originals reached through the evidence owner's ignored location metadata.
It used local byte comparisons, archive reads, JSON parsing, and hashes.
It did not execute a repository helper, build, compiler, test, probe, SSH,
API, network command, product, or desktop action. It did not delegate work.
The only written file is this review, in the T3-owned worktree
`t3-f7a7f33b`, branch `t3/audit-windows-build-evidence`.

The source is `06b9a139ec855a6f697ded393fa4c2c894110613`, tree
`4aef0f7764638d0d9d02608b5469852935741af4`. Its entire fix adds three
lines to `crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs`:
the Unix call selects `bound_socket::listen`, and the Windows call selects
`bound_socket::listen_owned`, at lines 132–135. The latter wrapper exists
at main-example lines 518–520. The main example includes the module
unconditionally at lines 14–15. The module Git blob is
`91c84ffada825055e0163a1fd53e7d94f8c5d1ee`, with SHA-256
`9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4`.
The previous review `8704f63ffef83c245b362bf38add9c989c15a757` remains a
separate source and Mac saved-evidence audit.

All 558 source-manifest paths, Git blob IDs, byte counts, SHA-256 values,
CRLF counts, and LF counts match the fixed Git source and the private
`source.tar` members. The archive has no extra file, duplicate file, link,
or unsafe path. Its size is 4,065,280 bytes and its SHA-256 is
`6d11dc7edeb4ac4fd18058d2a56015c0e808f3aa7e61a2a483f17a1e993246a4`.
The source tree ID also matches. The private and committed manifests are
byte-identical. The Windows `source-before.json` and `source-after.json`
match each other and all 558 manifest entries, including read-only state,
archive hash, manifest hash, and module hash. There is no overlay or line
ending conversion in this binding.

All 71 entries in `artifacts.json` match the committed bytes and hashes.
The manifest covers all 72 files except itself. All 51 copy-ledger entries
match their private original bytes and hashes. Each of the 48 committed
copies matches its ledger and the stated UTF-8 BOM removal and known
prefix replacements, with line endings preserved. The three declared
omissions are the two full shared snapshots and duplicate source manifest.
Their private originals remain available. No executable or archive is
committed.

The executed controller SHA-256 is
`c0b1887b426ac89d53b475c061f46adc8fe85ee7d2f0e0d57a1f732b4c2352e8`.
The private frozen controller is byte-identical to `controller.py`.
All seven raw command receipts bind this identity. Their stdout, stderr,
and script hashes match private originals. Reconstructing the original
argument arrays, including UTF-16LE encoded scripts, matches every command
signature. The saved setup, qualification, build, postflight, and cleanup
scripts match the controller templates and the same fresh root. The
eight saved checkpoint phases agree with those receipts and outputs.

The controller uses explicit repository-root Git operations for source
pathspecs (`controller.py:41–47,50–85`). The saved sequence has one setup,
one `scp -O` upload, one qualification, one build stage, one postflight,
one `scp -O` evidence download, and one cleanup. Every command returns 0
without timeout. SSH control stdin is immediate EOF. The seven uploaded
payload files match their transfer manifest and Windows confirmation.
The four PowerShell scripts match `workflow-bindings.json` and the private
uploaded copies. The transfer manifest itself is the eighth uploaded file.
The saved result records nice 15 and deployment target 15.0 for the local
controller. No Windows workflow correction, resume, compiler retry, or
watchdog stop appears in the saved sequence.

The compiler receipts agree with `build-start.json`, `build-result.json`,
and the exact scoped arguments in `build.ps1:24–25`. Both commands select
`--locked --offline --release --target x86_64-pc-windows-msvc --jobs 2
-p openergy-net --lib --example dns_queue_cancel_c4806a3d`; Clippy adds
`-- -D warnings`. Clippy returns 0 in 18,688 ms. The release build returns
0 in 47,248 ms. Their private output and process-sample hashes match the
receipts. Both stdout files are empty. The compiler stderr files end with
successful optimized release completion and contain no compiler failure.
The SSH stderr progress records are PowerShell module-preparation CLIXML.

`qualification.json` records native Rust/Cargo 1.99.1. The child script
selects the pinned toolchain, private Cargo home, target and temp paths,
jobs 2, and installed MSVC 14.44.35207 / SDK 10.0.26100.0 paths
(`build.ps1:5–23`). It sets process environment variables; it has no command
to change toolchain defaults or persistent environment settings.
Qualification compares the private registry copy's paths and file bytes
with the 6,300-file original registry, and makes registry source files
read-only. It copies no global Cargo configuration or credentials.

The 104 process observations contain 3 Cargo, 1 cargo-clippy, 84 rustc,
7 clippy-driver, 3 cl, 1 link, and 5 Cargo build-script identities. All
observed processes are BelowNormal in session 0. Every sample's parent
chain reaches its recorded Cargo root with consistent creation times;
there are no missing sampled parents or duplicate process identities.
Clippy root PID 36004 reaches cargo-clippy 9236, Cargo 32416, and the
example's clippy-driver 43860. Release root PID 36460 directly parents
the example's rustc PID 8524. Their command lines identify the requested
example, native target, pinned tools, and fresh target directory.

There are 56 Clippy samples and 48 release samples, with 30 and 6 saved
observation races. The only sampled `link.exe` belongs to compilation of a
build script during Clippy. The final example linker is not sampled.
Some short-lived executable paths are null. These limits do not establish
priority for every child process. The README expressly limits its priority
claim to observed identities. The watchdog branches were not exercised.
Cargo's required build scripts ran as compiler work; the workflow contains
no invocation of the built example, app, or daemon, including `--help`.

The private evidence bundle is 44,900,352 bytes, with SHA-256
`f85dfb49cc0425bdfa7788cd7b109135bcb762d1b930ec092185254aebee2717`.
Its hash matches postflight stdout and the result. All 26 archive members
equal the downloaded private files. All 24 remote evidence-manifest
entries match their bytes and hashes; the remaining members are that
manifest and the executable. The executable is 2,521,600 bytes, with
SHA-256
`862d20212522ccd4bba242d1651af9a8ca960e381a818205f07cf874aa53958d`.
It matches `build-result.json`. Static header reads identify an AMD64
PE32+ image. No executable was run by this review. The controller checks
bundle, manifest, and executable bytes before it requests cleanup
(`controller.py:270–300`).

The private shared-before and shared-after snapshots are byte-identical:
20,834,593 bytes, SHA-256
`12c9c06aa4f8fbded3e66cdc16fc197b66faa8a81eaeea6909b600de9b58b165`.
They cover nine prior fixture trees, the original registry and pinned
toolchain, eight selected default-file states, sixteen tool binaries, and
one live-sharing process identity. Tree counts and the redacted summary's
entry hashes match the originals. Postflight agrees with these snapshots.
It records a protected root, 9,971 ACL items, no invalid ACL/reparse item,
no bad sampled priority/session, and no remaining owned build process.
This establishes preservation of the saved snapshot fields, without
claiming an exhaustive audit of all host settings.

The cleanup script binds the exact fresh root name and parent directory,
refuses remaining owned processes, and removes only that root. Its raw
receipt reports exit 0. Its private stdout equals `result.json`'s cleanup
object: no owned process before removal and `owned_root_removed: true`.
This is saved build-fixture cleanup evidence. It supplies no runtime
cleanup or strict-deadline acceptance.

The R5 owner's complete 22-file local tree, including ignored private
location metadata, matches `r5-preserved-before.json`. All 21 committed
R5 files also equal their immutable `4c98c28be71fe43a014ec59606b422ca89268e6f`
objects. Its private first-failure original matches the R5 copy ledger.
Its guard SHA-256 remains
`38ddd5e5778429ce47a26e1be9eb67979961c5b73fcebc7915910ea024012579`.
The initial and amended controllers, failed result, traceback, and first
command transcript remain present. The failed initial controller uses the
evidence directory as Git CWD; its archive command fails with exit 128
before the first SSH action. R6 neither resets that guard nor reruns R5.

All 70 prior evidence bindings from
`52cbf8c3c41b15789500ce7be620d203006630a9` match their immutable Git
objects and remain unchanged in the R6 commit tree. All 37 original
idle-TLS runtime evidence files also remain byte-identical to
`9f6625c951835295419ed1c53565581a7fa9681a`. In particular, the original
runtime stdout SHA-256 remains
`759fa0cec0df9fb7689cc38571cd4e04a836589bfb45286b7676baeb2dc05200`.
Its stalled-port cleanup still reports `AddrInUse` 48 and overall FAIL.
The earlier stalled SSH-stdin setup supplies no compiler success.

The later local packaging failure is preserved with its exit-1 result,
first error, original checking-code hash, and correction description.
Both corrected SCP receipt entries retain initial-derived-copy hashes;
those hashes match the original export transformation. The final copies
match the additional forward-slash profile masking. The corrected
`package-evidence.py` matches `packaging-verification.json`'s script hash.
Its source performs local reads, checks, and derived-copy writes only.
This correction supplies no repeated remote or compiler invocation.

The supported result is native Windows compilation for this fixed source
and exact library/example scope. No fixed-source runtime, continuous strict
deadline, original cleanup repair, wider security/provenance, publication,
or broader release acceptance follows from it. The three provider-stopped
tasks remain out of scope and were not resumed or reframed.
