# Linux x86-64 artifact and Python 3.11 packaging

Date: 2026-10-22 (America/Vancouver). The [machine record](linux-amd64-python311.json)
preserves exact inputs, original failures, source review, results, and hashes.
This used AMD64 containers on an ARM64 Mac. It is artifact and real-command
evidence under emulation, not physical x86-64 desktop or performance acceptance.

## Actual failure and fix

The locked Rust 1.99.1 x86-64 workspace release build, strict all-targets
workspace Clippy, and formatting passed at `84bd43b`. Packaging first stopped
because the offline cache lacked a Windows crate needed for all-target notice
checks. A preparation-only container fetched locked sources. Product execution
then resumed with networking disabled.

The next package attempt exposed a product bug on Python 3.11.2:
`TarFile.extractall()` rejected the `filter` argument. The documented minimum
was Python 3.11, so the Linux package command did not work throughout that
range. Extraction filters were backported to later 3.11 patch versions; they
are absent from [CPython 3.11.2](https://github.com/python/cpython/blob/v3.11.2/Lib/tarfile.py).

Commit `b50a778` keeps the minimum and replaces extraction of the freshly
generated local tar payload. It uses exact staged filenames, checks regular
entry type and file size, derives destinations from staging, creates files
exclusively, compares streamed SHA-256 digests, and normalizes permissions.
Unexpected, duplicate, missing, linked, special, or mismatched file data cannot
complete packaging. This is not an arbitrary downloaded-archive extractor.
The local writer does not emit sparse entries. An empty sparse map is a minor
source-review nuance recorded in JSON; size/content checks still constrain it.

Independent round 14 source review found no concrete introduced P1/P2 in this
scoped fix. It confirmed that `hashlib.file_digest` exists in Python 3.11. The
review did not run candidate code or independently repeat runtime results. It
does not replace the unavailable extra release-provenance review.

## Build and runtime scope

The official Rust 1.99.1 Debian Bookworm image was resolved by digest; its
identity is in JSON. Compiler and product commands used reduced-priority
launchers and two Cargo jobs, with two CPUs and 4 GiB assigned. Source and runtime state were
private. Product containers had `network=none` and used only their own loopback.
No host ports, native display, installed process, live input connection,
clipboard, service, login task, or port 24800 was used. No unit tests or mocks
were added.

The package source was clean `b50a7786d1064e07a2e02a7f3de1dc17e205501d`.
Only release tooling changed after the Rust build. All 112 Rust/Cargo input
hashes still matched after runtime checks; Cargo.lock stayed unchanged.

| Real interface check | Result |
| --- | --- |
| Package with Python 3.11.2 | Staged and freshly extracted CLI/virtual IPC smoke passed; notice freshness passed |
| Existing artifact destination | Exit 1 before replacement; archive hash unchanged |
| Direct daemon/CLI failure and recovery checks | 15/15 passed |
| Focused external scenarios | 30/30 passed; zero exclusions and cleanup failures |
| Live-worker parent | 30 status calls succeeded over 4.045 s with zombie leader/two threads; after worker exit, daemon exited zero and removed socket |

The 30 scenarios cover S20 parent-off/connected shutdown, S6 TLS trust approval,
S19 config patch, D2 restart recovery, all 23 Q clipboard/input regressions,
and A1/A2 modifier, wheel, direction, version, heartbeat and sequence behavior.
The parent flow observed a real pidfd. This x86-64 run did not repeat the
ARM64 syscall-denial experiment. It did not run S15 or third-party Docker peers.

## Checked local artifact

| Item | Value |
| --- | --- |
| File | `openergy-0.1.0-x86_64-unknown-linux-gnu-unsigned.tar.gz` |
| Archive size | 2,260,648 bytes |
| Archive SHA-256 | `e021d50d0ea68da826ac51ebd7d6331dd51f79776816be06c6747812fa5560d6` |
| Daemon size | 4,889,464 bytes |
| Daemon SHA-256 | `62fd1a9568e50589db274193f9360b50ce45099bc0eea3d40488af6f98097af7` |
| Payload | Nine regular files, 5,036,287 bytes; embedded BUILD.json equals sidecar; dirty=false |
| Executable format | ELF64, little-endian, AMD x86-64 |

The executable imports libc, libgcc_s and the x86-64 dynamic loader. Its highest
recorded GLIBC version symbol is 2.34. That inventory does not establish support
for every distribution. The archive remains local, unsigned and unpublished.
The six owned containers, two local fixture images, and newly pulled base image
tag were removed after preserving the artifact and raw evidence privately.

Physical desktop acceptance, positive Windows migration recovery, confidential
reporting, final PR security review, and trusted distribution remain open. The
[readiness gates](../../plan/readiness.md) still apply.
