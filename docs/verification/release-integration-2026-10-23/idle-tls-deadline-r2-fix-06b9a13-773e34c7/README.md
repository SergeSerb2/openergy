# R2 Windows listener compile fix

Source fix: `06b9a139ec855a6f697ded393fa4c2c894110613`.
Isolated starting parent: `a121376fe6c1715fa99d72559336ca3b3a73b439`.
Candidate `9f6625c951835295419ed1c53565581a7fa9681a` was cherry-picked
without changes as `f238375dd6a2f7b660817511da00a41d47d6ccb9`.

The fix adds exactly three lines to
`crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs`.
Unix keeps the existing `bound_socket::listen(Owner)` call. Windows selects
the existing `bound_socket::listen_owned(Owner)` wrapper. This follows the
main example's existing platform selection. The main example, its default,
full and refusal routines, other examples, and production libraries have
no fix-related changes. No test or mock was added.

The fixed module Git blob is `91c84ffada825055e0163a1fd53e7d94f8c5d1ee`.
Its SHA-256 is
`9cd3475f9c13c8e28ea54e0f344b55ae795267b22ea1676c45e5659a64f1f1d4`.
[source.json](source.json) binds the source commit, tree and input hashes.
[source-fix.patch](source-fix.patch) is the exact three-line patch; its
SHA-256 is `aa473856d217aeaaf77445b563ebd342aaf46d85fc0e5e305494bf90087a32c1`.

The read-only R2 review was read at
`/Users/serge/.t3/worktrees/openergy/t3-58c8956e/docs/verification/release-integration-2026-10-23/idle-tls-deadline-r2-9f6625c-58c8956e/review.md`.
Its SHA-256 at that read was
`7d9e93668da49f370b380f83cbc5ff622ca9557395db1bdac9d3136be200b4df`.
It was not edited or copied. No further review was delegated.

## Fresh macOS format and compile results

All commands ran once on the fixed source. Format check, scoped release
Clippy with warnings denied, and scoped release build each returned 0.
Rust was `1.99.1 (58cf8f9c3 2026-10-13)` on `aarch64-apple-darwin`.
The controller had measured effective nice 15. Its subprocess calls used
no further nice increment. Actual cargo/rustc child priority was not
sampled and remains **UNMEASURED**. No priority-only retry was made. The
commands used jobs 2, offline/locked Cargo, and
`MACOSX_DEPLOYMENT_TARGET=15.0`. Static `vtool -show-build` inspection
reported `minos 15.0`. No example executable was started.

The build used installed Rust binaries directly, with a private
`CARGO_HOME`, private target directory, and a verified copy of the cached
registry. It copied no credentials or global Cargo config. The original
registry byte manifest and selected Cargo/Rustup default-file hashes were
equal before and after. See [result.json](result.json), the exact command
and result records in [raw/clippy.json](raw/clippy.json),
[raw/release.json](raw/release.json), and
[raw/format-check.json](raw/format-check.json). All raw stdout/stderr files
are retained, including empty streams.

The Mac binary SHA-256 is
`c49c6cfc13fd2f3e54e6f41632b39123eda5095011f114992388f41c73c0bb29`.
This is a newly compiled, unexecuted binary. It is distinct from the
candidate's saved runtime binary.

## Native Windows compile UNVERIFIED

Read-only SSH preflight found the installed native Rust 1.99.1 toolchain,
MSVC 14.44.35207 and Windows SDK libraries. The tools were absent from the
initial shell PATH but present at their installed paths. The existing
native Rust/MSVC discovery was intended to use those files. No toolchain
or dependency was installed, and no persistent PATH or default was changed.

[windows-source-manifest.json](windows-source-manifest.json) binds the
exact Git archive and every exported file. The export contains source
from the fixed commit, with no overlays, shims or line-ending conversion.
The first source-transfer setup did not reach the payload write. A
read-only check found the owned root empty, with no payload, build guard
or compile record. At 166.477834 seconds, the setup process had consumed
0.359375 CPU seconds. Its actual priority was **BelowNormal**, session 0.
The exact unique encoded setup command identified one owned process,
PID 13184. It was stopped before any compiler or example started.

The first SSH setup exit 1 and its raw stdout/stderr are retained in
[raw/windows-setup-and-build.json](raw/windows-setup-and-build.json).
The local controller also exited 1 at its final assertion. The evidence
fetch returned 0 and confirmed all seven expected build files absent.
No transfer or compile retry followed. The failure stage was before
`payload.tar` creation; its underlying transport/input cause was not
established. This is neither a Windows compiler pass nor a compiler error.
The intended archive manifest does not establish a remote build binding.

An exact-command process check then found no remaining setup process and
confirmed the owned root was still empty. Only that empty owned directory
was removed. See [raw/windows-owned-setup-stop.stdout](raw/windows-owned-setup-stop.stdout)
and [raw/windows-owned-cleanup.stdout](raw/windows-owned-cleanup.stdout).
No remote cache copy or source extraction occurred. There were zero native
compiler invocations and zero example executions. The installed tools and
their preflight hashes are evidence of available inputs only.

## Preserved original failure and acceptance limits

All 37 files under the candidate's
`idle-tls-deadline-2701060/` evidence directory remain byte-for-byte equal
to candidate `9f6625c`. See
[original-evidence-preservation.json](original-evidence-preservation.json).
The original saved behavior checks remain inherited evidence: pending TLS
before expiry, first eligible timeout poll, and same-caller/handle pinned
TLS PASS/BACK recovery. They were not independently rerun here.

The original overall **FAIL / exit 1** from the immediate stalled-port
`EADDRINUSE` cleanup bind is unchanged. Its cause remains unresolved. The
later postflight does not replace the failed immediate cleanup gate.
There were zero runtime attempts for the fixed source. No old case was
retried, no installer fixture was accessed, and no production port,
discovery, native sharing backend, desktop action, service, account,
firewall, DNS, production config, signing key or token was used or changed.
All wider runtime, security, provenance and release gates remain unchanged.

The build-only controllers and exact scripts are retained for audit.
Their local attempt guards prohibit repeating these recorded attempts.
Some preflight JSON fields named `started_local` or `start_local` were
sampled after the read-only command returned. Their elapsed durations are
measured; those timestamp labels do not prove the command start time.
The setup transport record and process creation/stop records have their
own independent timestamps.

The first packaging check stopped on raw compiler-output whitespace, and
Git warned that it would convert CRLF in the Windows transcripts. The
first packaging failure is retained. The directory-local `.gitattributes`
now preserves raw files with `-text -diff`; no log bytes were edited.
The final staged blobs are checked against every manifest entry.
[artifacts.json](artifacts.json) lists all evidence files except itself.
