# Embedded archive provenance follow-up

The real `draft` CLI at source `4a0143e376cbaf5cf276fa545d7854ef0b95418f`
accepted an archive whose root `BUILD.json` differed from its sidecar. The
sidecar matched the clean checkout, local tag, version, target, notices and
signing policy, and the archive checksum was correct. The command stopped only
at `draft needs all four native platform archives; no partial release`. Thus
the checksum bound the archive bytes to the checksum file, but no check bound
the archived provenance to the validated sidecar.

`draft` now reads the embedded manifest without extraction. Both previously
accepted extensions remain supported: `.zip` and `.tar.gz`. The archive must
match its extension and contain exactly one `BUILD.json` at its root. The entry
must be a regular file of at most 1 MiB. The read is also capped at 1 MiB plus
one byte. Its complete parsed JSON must equal the sidecar. Duplicate, nested and
alternate paths, links, directories and other special entries fail. ZIP files
without Unix type bits remain supported; explicit special-file types fail.
All existing source, tag, notices, signing and checksum gates remain in place.

## Real-command evidence

Verification used Python 3.15.0 at `/opt/homebrew/bin/python3`, with `nice -n 15`
and no rebuild. Disposable local clones had no Git remotes. Each clone had a
clean checkout and a local-only `v0.1.0` tag pointing to its HEAD. The fixed
verification snapshot was `f93422970019042939e7fe888f81cdf0ae2b444e`;
its `tools/release/release.py` SHA-256 was
`82fa05ef25561fc1387a027fc9c5a6757306b63e44ba6246937d98c906b9e214`.
That snapshot commit exists only in the disposable clone.

The existing Mac candidate came from
`/tmp/openergy-release-candidate-final/openergy-0.1.0-aarch64-apple-darwin-adhoc.zip`,
SHA-256 `7fbb0d5d2d23506623c01f64b1ec9aca6e3bf9c172401ed4d20f01af739a9751`.
Its original provenance names source `7a1e077` and has no release tag, so it
cannot pass a tagged-source gate unchanged. Verification copies replaced only
the embedded/sidecar provenance with the local clone commit and tag. All 16
other archived files remained byte-identical to the candidate. The tar case
contained those same files in a real gzip-compressed tar archive. These copies
are CLI validation fixtures, not newly built or release-approved artifacts.

The mismatch cases changed only the embedded `verification` field and wrote
the correct checksum for the resulting archive. Full command paths were:

```sh
lab=/tmp/openergy-provenance-mmmagcyi
nice -n 15 /opt/homebrew/bin/python3 "$lab/baseline/tools/release/release.py" \
  draft --tag v0.1.0 --artifacts "$lab/baseline-mismatch"
# Exit 1: release: draft needs all four native platform archives; no partial release

nice -n 15 /opt/homebrew/bin/python3 "$lab/fixed/tools/release/release.py" \
  draft --tag v0.1.0 --artifacts "$lab/cases/zip-mismatched-embedded"
# Exit 1: release: archive BUILD.json does not match its provenance sidecar

nice -n 15 /opt/homebrew/bin/python3 "$lab/fixed/tools/release/release.py" \
  draft --tag v0.1.0 --artifacts "$lab/cases/tar-mismatched-embedded"
# Exit 1: release: archive BUILD.json does not match its provenance sidecar

nice -n 15 /opt/homebrew/bin/python3 "$lab/fixed/tools/release/release.py" \
  draft --tag v0.1.0 --artifacts "$lab/cases/zip-existing-mac-payload"
nice -n 15 /opt/homebrew/bin/python3 "$lab/fixed/tools/release/release.py" \
  draft --tag v0.1.0 --artifacts "$lab/cases/tar-existing-mac-payload"
# Both exit 1 at the missing-four-target guard after validating the archive.
```

All **48 full `draft` CLI invocations** returned the expected result: one
baseline reproduction and 47 fixed-command checks. The fixed checks covered:

- Valid Mac payloads in ZIP and tar.gz, semantic JSON equality with different
  key order/spacing, and a DOS ZIP regular entry without Unix type bits.
- Exactly 1 MiB accepted and 1 MiB plus one byte refused in both formats.
- Mismatched, missing, nested-only, duplicate-root, root-plus-nested and
  `./BUILD.json` or backslash-alias manifests in both formats.
- ZIP symlink and Unix/DOS directory entries; tar symlink, hardlink, directory
  and FIFO entries. No archive member was extracted or materialized.
- Invalid JSON, truncated ZIP/gzip data, swapped ZIP/tar contents and plain tar
  mislabeled as `.tar.gz`.
- Changed checksum, two archive candidates for one sidecar, an empty artifact
  directory, and each existing sidecar version/commit/tag/dirty/notices/signing/
  notarization/target refusal.

The real `check` command also passed in the working checkout and with
`--tag v0.1.0` in the clean fixed clone. `git diff --check` passed.

Every nonempty input held one sidecar and one target only. Even valid inputs
stopped at the existing missing-four-target guard. The command path never
reached `gh release create`. No GitHub release-state command, push, remote tag,
visibility change, native app launch, installed state, desktop, keychain,
production IPC or network port was used. No unit tests, mocks, isolated function
tests or dependencies were added. Fixture creation and full-command driving
used only Python's standard library. Raw fixture commands and outputs remain
local under the disposable directory above.

This verifies archive provenance checks only. It does not rebuild the candidate,
authenticate a publisher, prove that its binaries came from the declared source,
or exercise the four-target upload path or native desktop acceptance.

## Independent review: case aliases

The next independent review reproduced a second ambiguity: a valid root
`BUILD.json` plus a later `build.json` passed the exact-case filter. Real ZIP
and tar extraction on the case-insensitive temporary filesystem replaced the
validated manifest with the unchecked case alias. Two controls and two malformed
archives reached the old missing-four-target guard. The review record remains
local at `/tmp/oe-r5-review-rti16i6x/evidence.json`.

The filter now counts manifest basenames without case sensitivity and still
requires exactly one precisely named root `BUILD.json`. Four more real CLI
checks passed: both aliases were refused, and both controls reached only the
missing-four-target guard. The fixed verification snapshot was
`ae8c2ebae07fc2ecbf006fd00dbad22fd3c84ef9` in a clean clone without remotes;
script SHA-256 was
`d4ba934d74c408c98c5b754619144ca91c2db6dc21f24f63e415aca17ea2828d`.
The exact results remain at `/tmp/oe-r5-alias-fix-y8km1lnz/report.json`.
Each case again held one target only. No command reached GitHub release creation.
