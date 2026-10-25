# Independent file-qualification SOURCE/DATA review

2026-10-25, America/Vancouver. **Bounded SOURCE CLEAR: no concrete P1/P2
found in the tracked source/data reviewed here. execution=false. Runtime
UNVERIFIED.** This verdict covers the new file-only helper and recipe bindings.
It does not establish fresh artifact, prefix, SDK, copy or runtime qualification.
It adds no human approval gate.

The checkout started clean at parent
`22e411d1f1a4514ce6598227806ffa1d41a2347e`. Candidate preparation is
`3a7335ceaa761af2c6530351658db0ba1787b40a`, imported under
`docs/verification/m1-next-reviewed-ordinary-flow-2026-10-25-t3-a8a4ece5/`.
All 11 candidate blobs equal that commit at the reviewed parent. Their Git
modes are 100644. All ten SHA256SUMS entries match. All candidate JSON parses.

I read the full candidate README, helper, qualification and metadata recipes,
next-operation and bindings. I also read the templates, static-check record,
reader-error record, full operator plan and receipt contract. The receipt schema,
source manifest, materialization records, tracked artifact assessment and saved
attempt/build reviews supply the relevant data contracts. [bindings.json](bindings.json)
records exact input hashes, module bindings and the 50 expanded command vectors.

**File algorithms and modes.** Locations below refer to candidate files.

- `file-qualification.recipe.py.txt:21-27,47-51,73-118` uses a 120-second
  real-time alarm and requires an absolute canonical evidence root with mode0700.
  The private prefix and launch file must also be canonical. The prefix walk
  rejects other file types and link targets outside the prefix. It requires
  exactly 2,006 entries and 46,504,040 regular bytes, the pinned prefix digest,
  and the pinned executable digest and mode0700.
- Lines 83-104 preserve the declared prefix algorithm. The root has no digest
  row. A link row contains relative path, `l` and exact readlink text. A directory
  row contains relative path, `d` and numeric `stat.S_IMODE`. A regular row adds
  numeric mode, length and SHA-256. The helper sorts complete rows and hashes
  compact `json.dumps(..., separators=(',', ':'), ensure_ascii=True)` plus LF
  in UTF-8. This agrees with the tracked build algorithm at
  `m1-sampler-source-repair-r3-2026-10-24/controller.py:80-108`. No symlink mode
  or root row was added to the historical digest. Root mode is a separate fact.
- Lines 107-115 authenticate the pinned source-checks, limits and full build
  source bindings. All ten unique safe module names resolve to their exact
  candidate Git blobs. I checked their hashes, lengths and line counts as data.
  I did not parse or execute their Python. The controller hash is unchanged.
- Lines 119-147 compare the complete source leaf-path set and all 702 regular
  file lengths, raw hashes and modes. Independently, those 702 manifest records
  equal every tracked non-docs path at product commit
  `ed7d96bf53da1a539fa2acde3b7133613221fd4b`, totaling 4,773,386 bytes. There is
  no archive, newline or executable-mode normalization.
- Lines 54-70,137-143 preserve bundle list semantics: sorted relative paths,
  octal permission strings, directory markers, exact link text, and regular
  lengths/hashes. Link resolution must stay inside the bundle. The accepted
  list has exactly 167 records and equals the pinned full inventory. The bundle
  root is excluded from that list and checked separately for mode0700.
- Lines 144-147 bind the ten SDK headers by canonical path, length and hash.
  Every declared header triple equals the tracked assessment's
  `hash-bindings.json#/embedded_headers`. This checks the recipe's authenticity
  against that assessment. It does not inspect the current SDK or re-prove ABI
  compatibility.
- Lines 148-158 require the prepared copy at the canonical new private root
  directly under `/private/tmp`, with the declared name prefix and exact
  `Openergy.app` location. Private-root and bundle owner IDs must equal the
  evidence-root owner ID. Both roots require mode0700. Source path and all
  copied inventory records must agree with prepared data and the build basis.

`source` and `entry-template` read the disabled artifact template. `entry` and
`copy` read the completed artifact receipt. All four authenticate the prefix
and ten modules. Only `source` and `copy` perform the full source/bundle/header
checks. Only `copy` inspects prepared-copy data. These branches agree with the
declared uses. The observation schema and zero helper exit do not assert any
artifact gate.

**Startup, metadata and record contracts.**

`qualification.recipe.json:21-29,45-68` and `next-operation.json:156-250` place
trusted system Python, with `-I -S -B`, before private-Python entry. The version
command uses `entry-template` first. Later writers, prepare and controller use
`entry` authentication. Private Python does not authenticate itself. The bound
prepare/controller vectors retain `-B` and sibling imports. The explicit
startup environment and update-disable fields remain present.

`metadata.recipe.json:127-216` expands to 49 per-object commands and one deep
verify. The seven binary paths and corrected codesign evidence pointers agree
with the tracked assessment. In particular, Autoupdate uses supplement records
4..7, and the daemon uses records 0..3. The other objects retain their original
label mappings. Architecture/load/detail commands select arm64 as declared.
Strict verification includes all architectures. No product command is added.

The comparison at `metadata.recipe.json:64-70` preserves actual exit0 and both
raw streams. It substitutes only the bound bundle prefix. Only labels ending
`-verify` compare sorted lists of complete UTF-8 `splitlines()` values per
stream. Duplicate and empty lines remain significant. Other output remains
exact after path substitution. Warnings, XML and distinct empty/no-payload
results are not rewritten. This matches the comparison described in the
tracked saved-attempt review. A new copy compares with fresh source data under
the same saved basis. Old first-nine and completion records remain historical.

The recipes preserve one metadata pass per bundle, 120 seconds per phase,
10 seconds per command within the phase, 1 MiB combined command output,
8 MiB evidence files and 32 MiB aggregate evidence. Reaping and both copy file
checks share the original copy-phase deadline. A helper invocation's alarm does
not extend that deadline. Source initial files remain exclusive0600 inputs
charged by prepare. Later publication uses the existing reserved writer.
Actual child exits remain distinct from transport or writer status. Failure
retains first and partial records without repetition or a completion pass.

**Exact hashes and independent proof limits.**

Candidate SHA-256 values:

```text
file-qualification.recipe.py.txt  bf2ddd03f00bedbf799b4eee518a2b7b25d1eb9a6df06a2f55c419252f6fa428
qualification.recipe.json       07ebb740dc83a12deffee0094d6116e04d27bb3949ad7a3f6cd3d7fd7fe64537
metadata.recipe.json            f97b12054aceb745cadcd3159d25f4693b3570a44d216ffdd96d9c1ddf05701e
next-operation.json             f4b6eea1ad6dab12db27227a0eb8945c22c7c3aebb3f0fc854122aac84fb74b1
bindings.json                   63f896f99d36d66af75458088c8c94cbd37f34552cd02980290cde914ad4de96
```

All 26 declared repository input lengths and hashes match tracked blobs. The
source binding remains
`20e011251151cec81f69c4b80b0072d9fd0a98c1eaaf0e6c74d83ff752546617`.
The ten-module source-checks digest remains
`2bde5cf1f9396df5c7d8518caaadcdd69eafc5ff2f39117158efaf87e8d080f5`.
The expanded 50 source argv/label records have SHA-256
`95bd71b433b7ecd9cf1dc46ca33a5887909873472ac78b9626cb0fac3a02eb42`
under the exact serialization recorded in this review's bindings. That digest
is a new review-data digest, not the saved EXPECTED-data digest.

The private raw inventory, prefix measurement, old launchers, metadata outputs
and readiness records were not read. Their recorded paths, lengths and hashes
were treated as declarations. Thus I did not independently recompute the saved
EXPECTED canonical JSON digest
`b46d03fec326f76748fb0e181a0bb607ad4cef58198efaa62e762c22336e30e0`,
the bundle inventory digest
`e8255fcb45e50be4ea11710e921f64d8a60368fde2f217a7044e07b0d490e78f`,
or the live prefix digest
`c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67`.
The tracked reports support their lineage. They supply no new measurement here.
No fresh source/copy metadata equality or actual exit result is asserted.

The helper is deliberately an observation tool. Parent duties in the existing
contract still include current-user ownership, fresh root and filesystem
selection, exact0600 materialization, immutable data bindings, assessment of
the reported prefix-root mode, and the complete receipt/raw-data gates. The
helper does not enforce all of those duties. Its header list comes from the
exact materialized recipe. Its evidence references require parent assessment.
Whether the ten saved SDK paths pass its strict canonical-path assertion on
the target machine remains unverified. An alias would fail closed. No target
path mismatch is established by this tracked-only review.

The alarm and accepted row/byte counts do not prove a hard OS-call or memory
bound. Recursive enumeration and path-based checks do not exclude concurrent
replacement, restoration between checks or same-user races. No such broader
claim is made. The full existing receipt contract, including signatures,
entitlements, ABI, plist, no-transfer and successful-build evidence, remains
necessary. Helper success alone cannot enable the artifact receipt.

Controller `2b5fc1f505b1f0426b8f1e8b00537a0b6e805fc7` remains SHA-256
`08f1bbea8aeb5852b4aaddd0f3f693c7717c69e5e898462baa90b49cba41e3a4`.
Its imported clear report equals the original report at
`b8a422145a5ab5dc197304f6420b85654b5f14ea`. The prior missing `self.spec`
finding remains closed by that one-line repair. I did not re-review unchanged
controller logic. No additional independent or human gate is requested.
`paths.EVIDENCE=null` is an expected future operation binding. Parent may choose
that routine path under the existing contract.

Old M1 remains **FAILED_NOT_ACCEPTED**: app25/25, app exit0, controller exit1,
first strict post65431 EADDRINUSE, cause **UNKNOWN**. **Whole-flow NI15 remains
UNVERIFIED**. The original second post-bind, bundle-after, outcome and archival
remain missing. Old failed records, guards and metadata remain unchanged.
The build's stripping warning and unknown complete outer duration/exit remain.

Only this review folder was written. Independent readers used Git blobs,
text, JSON and hashes. No supplied helper/source/AST was executed or imported.
No AST parse, test, mock, runtime flow, native parser, live process/port query,
remote query, private-state read/write, credential, build, signing, integration
or child-agent action occurred. No TypeScript changed. Hooks and commit signing
are disabled for the review commit. Native, release, performance, continuous
NI15 and host-restoration acceptance remain outside this verdict.
