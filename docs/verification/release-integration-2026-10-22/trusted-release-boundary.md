# Trusted release workflow boundary

The PR #2 P2 review at source `2b902d9` correctly identified a trust-boundary
failure: a dispatch-selected ref supplied the first validation script on a
persistent release runner, then supplied the publishing script in the job with
`contents: write`. A selected workflow definition could also choose the manual
self-hosted path in `verify.yml`.

Review: https://github.com/SergeSerb2/openergy/pull/2#discussion_r4346064408

## Change

Both workflows now use standard disposable public hosted VMs only. Every job
skips while private, including manual runs. There is no self-hosted override,
no absent-runner queue, and no private hosted fallback. Local verification,
packaging, and explicit local draft creation remain available.

The release pipeline has three separate jobs:

- The read-only validator checks out the immutable default-branch workflow
  SHA. Trusted code checks a literal version tag, exact local/remote tag object
  and peeled source SHA, reviewed ancestry at the frozen workflow commit,
  default-branch continuity, metadata consistency, legal files, and the notice
  inventory's exact lockfile hash. Candidate Cargo/Swift/plist/JSON files are
  Git blob data, never executable validation code. The validated tag, tag-object
  SHA, source SHA, version and trusted SHA become frozen job outputs.
- Read-only candidate jobs check out that source SHA on fresh native workers.
  The workflow asserts real OS/CPU architecture before running candidate code
  and asserts archive target/source before upload. Existing clean-checkout,
  exact-tag, version, notice freshness, locked build and archive smoke gates stay.
- The publisher gets a fresh worker and checks out only the original trusted
  SHA. It imports only trusted code with Python isolated mode and bytecode
  writing disabled. It does not check out the candidate, run Cargo/Swift,
  extract an archive, or execute an artifact. It validates artifact data with
  the existing all-four-target, signing, notice, checksum and unique bounded
  embedded BUILD.json gates, including case aliases. It also refuses symlink
  and unexpected download entries. It rereads the frozen remote tag/source
  after archive validation, immediately before explicit draft creation.

Candidate jobs cannot replace the validation job's outputs or publisher code.
Repository write permission is limited to the isolated publisher job. Its
command receives GH_TOKEN only in the final trusted publishing step. All
checkout actions omit retained credentials; action versions are pinned. Tag
inputs and resolved values enter commands as quoted environment arguments.
No release is published automatically or by this verification.

Remote reads and release creation are not an atomic tag lock. Repository review
controls define reviewed default-branch history, and release tags must remain
immutable. This implementation changes no repository rule, visibility, tag,
runner registration, credential, or installed state.

## Real command verification

Python 3.15 ran the full CLIs at low priority against disposable real Git
repositories and a real local-file bare origin. No Git or GitHub command was
mocked. The candidate source and archive payload each had an execution sentinel.
Neither sentinel ran. The publisher's successful controls used `--check-only`;
all rejection paths stopped before any GitHub mutation command.

The machine-readable record is [trusted-release-boundary.json](trusted-release-boundary.json).
It records **82 release CLI invocations**:

- The existing 48 archive-provenance CLI checks passed: one unchanged historical
  baseline reproduction plus 47 checks of the refactored current CLI. The
  existing four ZIP/tar case-alias controls/rejections also passed. Their real
  Mac archive payload remained data, with verification-only provenance fields.
- Twenty-five trusted-source/publisher checks passed. These cover an annotated
  tag on a non-HEAD reviewed ancestor; branches, qualified refs, expressions,
  shell-looking input and output-injection newlines; an unreviewed source;
  version mismatch; invalid default branch; wrong or dirty trusted checkout;
  empty versus all-four-target data; embedded tamper, ZIP/tar case aliases,
  extra files, symlinks and checksum tamper; moved remote/local tags; a changed
  annotated tag with the same peeled source; and rewritten default history.
- The tag-move race used real Git trace events to observe the first source
  validation, then changed the local-file origin tag during real gzip/tar
  parsing. The final remote check refused it. The archive was never extracted.
- Five additional source-data checks passed: inconsistent Rust metadata, a
  stale notice/lockfile hash, symlink metadata and a workspace path escape were
  refused. Swift source containing an execution sentinel was read as data and
  never evaluated.

The four-target publisher controls are real ZIP/gzip-tar parser inputs with
verification metadata, legal text, and a non-executed sentinel payload. They
are not native build proofs or artifacts approved for a release.

Driver commands and raw results remain local:

```sh
nice -n 15 /opt/homebrew/bin/python3 /tmp/oe-release-trust-cli.py
nice -n 15 /opt/homebrew/bin/python3 /tmp/oe-release-source-data-cli.py
nice -n 15 /opt/homebrew/bin/python3 /tmp/oe-release-alias-regression.py
# Existing archive-provenance driver, copied into a fresh disposable lab:
lab=$(cat /tmp/oe-trust-regression-path)
nice -n 15 /opt/homebrew/bin/python3 "$lab/check-cli.py"
```

The JSON command arrays use `$TRUST_LAB`, `$SOURCE_LAB` and `$REGRESSION_LAB`
placeholders. Their actual disposable paths are recorded only in local pointer
files under `/tmp`. The verification created only local fixture commits/tags
and used local `update-ref` operations; it did not push a repository ref.

Two real architecture-step invocations accepted this Darwin/ARM64 host and
refused an Intel expectation. Both workflow files passed `actionlint` without
suppressions. `release.py check` and `git diff --check` passed. No Rust, Swift,
TypeScript, dependency manifest, or lockfile changed, so these checks did not
rebuild native binaries or repeat unrelated platform suites.

A hosted workflow run was unavailable under the private-repository cost policy
and was not dispatched. No successful remote draft mutation, publication,
keychain/certificate operation, app launch, native input/clipboard, service,
login item, TCC setting, production IPC, or port 24800 was used. No unit tests,
mocks, isolated function tests, dependencies, or paid provider were added.
