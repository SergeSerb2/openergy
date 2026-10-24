# R4 independent source and saved-evidence review

**No actionable P1/P2 findings.** The fixed source closes R2's Windows
listener name/signature defect. Native Windows compilation remains
**UNVERIFIED**. The original runtime remains **FAIL / exit 1**, with the
immediate cleanup error's cause unresolved.

## Reviewed inputs and boundary

- Fixed source: `06b9a139ec855a6f697ded393fa4c2c894110613`, tree
  `4aef0f7764638d0d9d02608b5469852935741af4`.
- New saved evidence: `52cbf8c3c41b15789500ce7be620d203006630a9`, whose
  direct parent is the fixed source commit.
- Original candidate: `9f6625c951835295419ed1c53565581a7fa9681a`, parent
  `270106002490ef3e135d4c6500df1c8ff452b45b`.
- Unchanged cherry-pick: `f238375dd6a2f7b660817511da00a41d47d6ccb9`, parent
  `a121376fe6c1715fa99d72559336ca3b3a73b439`. Its complete patch equals the
  original candidate's complete patch byte for byte. It directly precedes
  the fixed source commit.
- Review workspace parent: `5a9783738c5b649844805e1614da7ea37cfbf2bb`.
  The full prior record, `idle-tls-deadline-review-r2.md`, was read here;
  its Git blob is `1c7439baa4b722b0892a023b32175c74b9b5cc93`.

This review read immutable local Git objects and calculated hashes. It did
not integrate the source commits. No build, fixture, product runtime,
probe, network or remote access, desktop action, installer access, test,
mock, or further delegation occurred. Only this review document was added.
Live sharing, installed apps, production settings, services, credentials,
and toolchain defaults were not accessed or changed by this review.

Evidence paths below are relative to
`docs/verification/release-integration-2026-10-23/idle-tls-deadline-r2-fix-06b9a13-773e34c7/`
at the evidence commit.

## R2 defect closure and scope

The entire fix is three added lines in
`crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs:132-135`.
On Unix, the original one-argument `bound_socket::listen(Owner)` expression
remains at line 133. On Windows, line 135 selects
`bound_socket::listen_owned(Owner) -> Result<TcpListener>`, exposed by the
main example at lines 518-520. It no longer resolves to the private
two-argument Winsock foreign function at line 436. Both branches produce
the same local `TcpListener` type for the common following code. This
matches the existing platform selection at main-example lines 598-601
and 1024-1027. No new platform-condition regression was found in the fix.
This is source-level closure of the reported defect, not a Windows build
result.

Removing precisely those three added lines restores the candidate module's
exact bytes. The main example is unchanged from the candidate. Removing
only its original opt-in additions restores the original parent's exact
bytes and SHA-256
`530c880eb74f6c95d24a0612c4fa8773fa21d256765fc3f9f6260801fa303a30`.
Thus the existing default/full/refusal routine bodies and acceptance
predicates are preserved. The new mode returns before their execution.
Its configurations select private TLS directories and `mdns: None` before
driver construction; its addresses are literal loopback.

There is no production net-library diff against the isolated starting
parent. The fix changes no main example, dependency, lockfile, other
example, or production library. The evidence commit adds exactly 70 files
under its evidence prefix and changes no source.

## Manifest and raw-record audit

- All 69 entries in the new `artifacts.json` match committed byte counts
  and SHA-256 values. The directory has 70 files; the manifest excludes
  only itself, as declared. Empty streams and raw Windows CRLF bytes are
  included. The local `.gitattributes` is exactly `raw/** -text -diff`.
- All seven `source.json` input hashes and Git blob IDs match the fixed
  source. Its tree and patch hash match. The saved `source-fix.patch`
  equals the actual commit diff byte for byte.
- All 14 raw command JSON records were read. Every declared stdout,
  stderr, and script hash matches its saved file. The exact setup command
  encoded in both ownership scripts decodes to the saved setup script.
- All 558 exported Windows source files match the fixed commit's bytes,
  sizes, and hashes. Reconstructing the local Git archive produces the
  declared 4,065,280 bytes and SHA-256
  `6d11dc7edeb4ac4fd18058d2a56015c0e808f3aa7e61a2a483f17a1e993246a4`.
  Reconstructing the three-file payload produces 4,188,160 bytes and
  SHA-256 `bcc0e2f1f4e60001ec5b51979cb8338ca201a77d9f9285b3db8d15275a7472d3`.
  Its base64 transport bytes match the recorded stdin hash. These local
  bindings do not establish delivery or a remote build binding.
- The saved Mac before/after shared-input records are byte-identical.
  Their registry summary equals the private-copy summary. They cover the
  listed cache/default files; this review did not inspect the live cache
  or independently establish preservation of every host setting.
- All 37 original evidence files match the candidate byte for byte in
  the cherry-pick, fixed-source, and new-evidence commits. Their
  preservation ledger matches. All 36 original artifact-manifest entries
  also match. No first-failure transcript was replaced.

## Saved compile, Windows stop, and cleanup evidence

The saved Mac format, scoped release Clippy with `-D warnings`, and scoped
release build records each report exit 0, without timeout. They identify
Rust 1.99.1, jobs 2, locked/offline Cargo, a private Cargo home and target
directory, and deployment target 15.0. Saved static `vtool` output reports
`minos 15.0`. The release stderr retains a `rust-objcopy` stripping warning
caused by a missing `libLLVM.dylib`; Cargo still exits 0. No toolchain fix
or retry is claimed. The measured nice 15 value is the controller's value.
Actual cargo/rustc child priority was not sampled and remains
**UNMEASURED**. These are audited saved compile records, not new checks.
The controller does not execute the binary; `result.json` declares zero
fixed-source runtime attempts.

Windows preflight records show the installed native Rust version and
existing MSVC/SDK inputs. Version queries do not constitute compilation.
The progress record has no guard, build-start/result record, or private
cache. The stop record shows one exact-command setup match, PID 13184,
an empty owned root, no payload, and no build guard. It records 166.477834
seconds elapsed, 0.359375 CPU seconds, BelowNormal priority, and session 0.
The stop script limits its action to that unique setup process under
those conditions. The build script is downstream of payload write and
extraction, which this saved sequence did not reach.

The first setup transport record retains exit 1 and empty stdout/stderr.
The separate controller completion transcript retains its final assertion
failure; it is identified as copied tool output. The evidence fetch exits
0 and reports all seven expected build files absent. The later cleanup
record and script show no remaining exact setup-command match, zero items
in the owned root, and removal of only that empty directory, without a
recursive delete. These records support the narrow owned-process/root
cleanup claims. They do not identify the underlying transport/input cause
or prove compiler success. No native compilation or example execution was
reached, and no compile/transfer retry is recorded. Some local preflight
timestamp labels were sampled after completion, as the README discloses;
they cannot prove command start times.

The first packaging failure is also retained and labeled as a transcription.
The directory-local attributes preserve the original raw bytes, including
the compiler whitespace and Windows CRLF. The final committed manifest
checks pass; this packaging correction is not a build/runtime rerun.

## Acceptance limits retained

The original saved transcript still reports pending TLS qualification,
first eligible timeout poll, and same-caller/handle/waker pinned TLS
PASS/BACK recovery. R2's detailed behavior audit is inherited; R4 did not
reproduce it. The original raw tail still reports stalled-port
`EADDRINUSE` 48, healthy-port bind PASS, cleanup FAIL, and overall FAIL.
Its runtime JSON retains one attempt, no retries, exit 1, and no watchdog
firing. A later postflight cannot replace the failed immediate cleanup
gate or establish its cause.

The saved handoff declares zero fixed-source runtime attempts. R4 ran
no runtime. Continuous strict TCP,
fallback/TLS timing, native Windows acceptance, historical negative gates,
and the wider security/release/provenance gates remain **UNVERIFIED**.
This review does not resume the provider-stopped work or establish release
acceptance.

## Exact source and evidence identities

- Fixed module Git blob: `91c84ffada825055e0163a1fd53e7d94f8c5d1ee`;
  SHA-256 `9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4`.
- Main example Git blob: `367621a2dcf96cbc86b28552b7e517ce0d2752be`.
- New `artifacts.json` Git blob: `980baf24a8d7ffa4c1f59cd24e9d7f9517a38ae3`;
  SHA-256 `07b2c5c8a7baf0612be92a8b15f4b33a64540742a831481ecece025718d54802`.
- New `source.json` Git blob: `d6e1d4b313a2835f775f3eef2a1457ae7d86567b`.
- `source-fix.patch` Git blob: `bda9b83e64c459bb869193ff59932b94f3ba71c5`;
  SHA-256 `aa473856d217aeaaf77445b563ebd342aaf46d85fc0e5e305494bf90087a32c1`.
- Windows setup transport JSON Git blob:
  `0ca1276cfca7b27c5c8d70f5db4875a05e3feecb`.
- Windows owned-stop stdout Git blob:
  `ec01a74a31dbbc9f0d23f9b065e8381ab5a28f27`.
- Windows owned-cleanup stdout Git blob:
  `094b18c000182357e0fef3de986a01cef427ff06`.
- Original `idle-tls-deadline-2701060/raw/runtime.stdout` Git blob:
  `2021043e2730007fc0daafd0195f57407e06ba25`; SHA-256
  `759fa0cec0df9fb7689cc38571cd4e04a836589bfb45286b7676baeb2dc05200`.

The new Mac binary hash is a saved claim
(`c49c6cfc13fd2f3e54e6f41632b39123eda5095011f114992388f41c73c0bb29`).
The executable is absent from the committed bundle, so R4 did not check
its bytes. It is distinct from the original runtime binary and supplies
no fixed-source runtime proof.
