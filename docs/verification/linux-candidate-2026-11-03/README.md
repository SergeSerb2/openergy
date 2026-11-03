# Linux and interop verification — 2026-11-03

This record covers the published **0.1.0 alpha**, source
`e5d86e2389badf0c45fc2fcef8c4f478b3a2a284`, for SER-212, SER-213 and
SER-201. It does not establish stable-release or native desktop acceptance.
The three issues were read in full before verification. Repository rules came
from `CONTRIBUTING.md`, `tools/e2e/README.md`, `tools/interop-lab/README.md`
and `docs/releasing.md`.

## Environment and isolation

The Linux worker runs Ubuntu 24.04.5 LTS, aarch64, kernel
`7.2.12-orbstack-00394-gbecdd6130068`. Rust 1.99.1, rustfmt and Clippy were
installed in a new private temporary directory. Cargo state and target output
also stayed there. Builds used two compiler workers and nice 15. Checks of
314 build/runner input files found no changes from the source commit.
Other workers edited documentation in the shared checkout during this run.

Docker was absent from the Linux PATH and local sockets. Later access to the
host confirmed Docker 29.8.2 arm64 on macOS 27.2, build 26B5112a. A fresh
`git archive` source copy and separate host build output supplied the virtual
daemon and runner for Docker interop. This used no app, native macOS backend,
permissions, host clipboard, desktop input or installed sharing service.

Every daemon used private config, TLS and socket paths. Local endpoints were
loopback; plaintext cases stayed in the lab. Port 24800 was disabled. The
initial main run selected local `25600-25700` and lab `25800-25900`, each 101
ports. Supplementary checks used `25720-25799`; completed smoke/teardown
checks used other portions of the original assigned ranges. These commands
finished before the corrected coordination ranges arrived. Subsequent runs
used local `25600-25699` and lab `25800-25899`, with disjoint subranges for
concurrent host lab workers. Exact historical ranges remain in the reports.

Raw logs, generated identities and diagnostic directories remain outside the
repository. Retained JSON omits stderr tails, private artifact paths and state
directory locations. No unit tests or mocks were added or run.

## Build and audit

[Build/audit results](build-audit.json) record commands, timings and binary
hashes. These commands passed on Linux with the pinned toolchain:

```sh
python3 tools/release/release.py check
cargo fmt --all -- --check
cargo fetch --locked
python3 tools/release/notices/generate-notices.py --check
cargo clippy --release --locked --workspace --all-targets -- -D warnings
cargo build --release --locked --workspace
cargo-deny --color never check
```

Cargo-deny 0.20.2 passed advisories, bans, licenses and sources against advisory
database commit `965bf9ad083b58e6340d65829d85217b18be9290`. Warnings concerned
duplicate `windows-sys`/`winnow` versions and three unused license allowances.
Tool downloads matched the publishers' GitHub asset digests.

The existing `release.py verify --suite smoke` command **refused** this host:
`unsupported native release host: aarch64-unknown-linux-gnu`. Its packaging
policy supports x86-64 GNU Linux. That policy was not changed or bypassed.
The individual build checks above and the existing `release.py:smoke` helper
passed. The helper exercised version, absent-daemon error, real virtual IPC,
an invalid-method error and clean shutdown.

[Secret-scan review](secret-scan.json) retains the exact locations and review
classification. Gitleaks 8.30.1 scanned the source snapshot and all 104 commits
reachable from the source SHA, with redaction and `--ignore-gitleaks-allow`.
**Both scanner commands exited 1**, with 93 findings each. Every finding was
checked at its reported revision/line: all were JSON SHA-256 source/content
hashes, not credentials. No credential was found in these results. This does
not prove that every possible secret format is absent.

## Real-process results

The first strict Linux run completed **91/93**, with 30 Docker cases excluded:
[initial report](linux-e2e-initial.json). Both failures remain recorded:

| Scenario | Initial result | Focused retry |
| --- | --- | --- |
| A7 trust writer crash | Could not observe approval lock ownership. | [1/1 passed](linux-a7-retry.json). |
| S15 resources and latency | p99 8.091 ms exceeded 5 ms; p50 0.421 ms, max 42.714 ms. | [1/1 passed](linux-s15-retry.json), p99 0.280 ms, same 5 ms limit. |

A7's writer can finish before the runner observes its brief lock interval.
The passing retry supports timing sensitivity; it does not erase the first
failure or establish its cause. S15's initial idle RSS was about 4.5 MB per
daemon with 0.0% measured CPU. The latency metric includes external injection
and IPC observation. It is not physical input latency. Stress probes and host
build/lab work overlapped portions of the initial run.

The initial Linux invocation was:

```sh
"$LINUX_PRIVATE/target/release/openergy-e2e" run --no-docker \
  --port-range 25600-25700 --lab-port-range 25800-25900 \
  --report "$LINUX_PRIVATE/linux-e2e.json"
```

Focused retries used `--scenario 'A7-*'` and `--scenario 'S15-*'` in separate
invocations. The complete repeat used local `25600-25699` and lab
`25800-25899`. Every run kept the default slowdown of **1**.

Two disjoint Docker batches completed **30/30**, with no exclusions:
[lab reports](lab-e2e.json). They used unmodified catalogue TOMLs copied into
two private scenario directories. Deskflow 1.27.0 and Barrier 2.4.0 passed both
roles, TLS and lab plaintext, wrong-pin refusal, input/clipboard paths,
exports and repeat-key cases. They do not verify Synergy core 1.21.1.
The host runner invocations, from the private source directory, were:

```sh
"$HOST_PRIVATE/target/release/openergy-e2e" run \
  --scenario-dir "$HOST_PRIVATE/lab-a" \
  --port-range 25600-25624 --lab-port-range 25800-25824 \
  --report "$HOST_PRIVATE/host-lab-a.json"
"$HOST_PRIVATE/target/release/openergy-e2e" run \
  --scenario-dir "$HOST_PRIVATE/lab-b" \
  --port-range 25625-25649 --lab-port-range 25825-25849 \
  --report "$HOST_PRIVATE/host-lab-b.json"
```

The complete strict Linux repeat finished **93/93**, with 30 Docker cases
excluded: [repeat report](linux-e2e-repeat.json). A7 and S15 passed without
code, scenario, threshold or slowdown changes. The initial 91/93 result
remains part of this record; a passing repeat does not establish the cause of
either initial failure.

[Catalogue coverage](catalogue-coverage.json) checks that these 93 Linux cases
and the 30 Docker cases account for all **123** source scenarios, without
overlap or omissions. These are separate runs with different daemon hosts,
not a single native-platform 123-case run. A prior whole-catalogue host run
and a prior Linux repeat were interrupted before final JSON output; their
partial console passes are not complete-run evidence.

[Supplementary reports](supplemental.json) passed with the real Linux binary:

| Existing script | Passed cases |
| --- | ---: |
| `verify_daemon_audit.py` | 15 |
| `verify_ipc_deadlines.py` | 19 |
| `verify_identity_files.py` | 20 |
| `verify_identity_contention.py` | 4 |
| `verify_trust_contention.py` | 6 |
| `verify_trust_scaling.py` | 9 |
| `verify_trust_refresh.py` | 4 |
| `verify_connection_churn.py` | 4 |

Each parameterized suite selected the compiled private binary and a private
report path. Port-bearing supplementary suites used `--port-range
25720-25799`. IPC deadline checks used a private Unix socket and the off role.
The 100 ms response budgets, peer-cap behavior and trust/identity lock recovery
remain scoped to these synthetic virtual peers and this machine.

[X11/lab details](x11-and-lab.json) identify the images, peer versions and
binary hashes. The existing Xvfb `verify.sh all` run passed input capture and
injection, clipboard formats and unavailable/clear behavior, grab/keymap
recovery, repeated backend teardown, and independent image-pixel checks,
including a 10,498,508-byte PNG/INCR transfer. It used private source and cache
mounts in a new container. This is isolated Xvfb evidence, not physical X11
desktop acceptance.

The existing forced-interruption verifier also passed. It observed an owned
real daemon and Docker lab, killed only its runner, then confirmed that the
guardian removed the daemon and lab container.

## Downloaded alpha artifacts

[Artifact results](artifacts.json) include full hashes, manifests, source
comparison and extracted-binary smoke details. All release assets were
downloaded read-only from [v0.1.0](https://github.com/SergeSerb2/openergy/releases/tag/v0.1.0),
published 2026-11-02. The annotated tag resolves to the exact source SHA.

| Archive | SHA-256 |
| --- | --- |
| macOS universal ZIP | `acd5c81113670edd34658d14ca489ce083758acc4df1ae5676b4308bcda6e9b0` |
| Windows unsigned ZIP | `abd5d4f80f1141e5893881d82bdfa0dd5f93f49a8d3a43a863801f8bb9cd53b9` |
| Linux unsigned tar.gz | `e866b3a5475fe9d41464fc69e5a2d805b24454122476f6ece888a34ae08321e9` |

Each archive matches both its checksum sidecar and GitHub asset digest. All
three carry project licenses and third-party notices identical to the exact
source. Linux and Windows each have one root `BUILD.json`, equal to their
JSON sidecar, declaring that SHA, Rust 1.99.1, clean source and **unsigned**
binaries. These are provenance claims, not independent authentication of
compiled bytes.

The macOS ZIP has **no `BUILD.json` and no JSON provenance sidecar**. Its
Info.plist declares version 0.1.0, build 108. Matching legal files and the
release target provide indirect source evidence only. This remains a
provenance gap for SER-213. macOS signatures, notarization, stapling and
Gatekeeper must be assessed on macOS by the separate platform worker.

The GitHub source archive contains all 831 tracked files. Of those, 830 match
Git blob bytes exactly. `tools/windows/openergy-run.cmd` differs only by the
CRLF conversion specified in `.gitattributes`; normalized bytes are equal.

The extracted Linux executable's hash is
`aa5bb4d8a121c4339edfa4c0c1af8c741048103d792c038301967c75b6645beb`.
Direct execution in the arm64 root filesystem failed with missing
`/lib64/ld-linux-x86-64.so.2`, exit 255. A pinned Debian x86-64 Docker image
then ran it successfully under emulation, with a read-only binary mount,
private tmpfs state and `--network none`. Two real virtual daemons connected
over container loopback with mutually approved TLS pins; version, absent
daemon, invalid IPC method and both clean exits passed. This is an emulated
package smoke check, not native x86-64 desktop acceptance.

## Remaining gates

Do not mark SER-212, SER-213 or SER-201 complete from this record alone.
Native macOS/Windows/physical X11 input acceptance, native x86-64 Linux
acceptance, final stable-candidate artifacts, macOS package source provenance,
and the separate platform signature/install checks remain outside this
record. Synergy core 1.21.1 was not run here; the Linux Docker lab supplies
Deskflow and Barrier only. No Linear state, release, tag, commit or PR was
changed by this worker.
