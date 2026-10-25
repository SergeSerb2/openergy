# M1 Python prerequisite: verified data only

Prepared on 2026-10-24 (America/Vancouver) at root
`ed7d96bf53da1a539fa2acde3b7133613221fd4b`, branch `t3/27c54a9f`.
The checkout was clean before this task. This package supplies pinned archive
data and an inert future extraction recipe. Native qualification is **UNVERIFIED**.

The parent supplied this target context: M1 arm64, macOS 27.0.1, Xcode 27.1,
Swift 6.4, SDK 27.0, `/usr/bin/python3` 3.9.6, and no Homebrew or named
Python 3.11+. This task did not verify that context on the target. The parent
alone owns the remote connection and will supply exact Rust 1.99.1 separately.

## Pinned official input

Selected standard GIL CPython **3.13.9 final**, Astral release **20251014**,
`install_only`, `aarch64-apple-darwin`:

`cpython-3.13.9+20251014-aarch64-apple-darwin-install_only.tar.gz`

- [Official release](https://github.com/astral-sh/python-build-standalone/releases/tag/20251014): release ID `254523032`, published `2025-10-14T22:17:22Z`, `immutable=true`, `draft=false`, `prerelease=false`.
- [Exact release API metadata](https://api.github.com/repos/astral-sh/python-build-standalone/releases/tags/20251014): retained as `release-20251014.json`.
- [Archive](https://github.com/astral-sh/python-build-standalone/releases/download/20251014/cpython-3.13.9%2B20251014-aarch64-apple-darwin-install_only.tar.gz): asset ID `304305223`.
- [Checksum asset](https://github.com/astral-sh/python-build-standalone/releases/download/20251014/SHA256SUMS): asset ID `304306156`, from the same release asset list.

Exactly one archive was downloaded. Its actual size is **16,526,691 bytes**.
Its SHA256 matches both the API asset digest and the single exact-filename
entry in `SHA256SUMS`. The checksum asset also matches its own API size and
digest. Downloads used public HTTPS without credentials. The archive and checksum
redirected to GitHub's `release-assets.githubusercontent.com`. The acquisition
code restricted redirects to official GitHub delivery hosts. No `latest`,
floating branch, mirror, or second archive was used. The API's recorded
`target_commitish=main` was not followed.

SHA256 bindings:

```text
archive:  931db8f735e18700d4eab9ee39dbbd0b4c114d7d039dd2707b2d932ded039698
metadata: e4b40842a5021bd46e84197bca665db337f13daec157ad9a0979081fd3a321b8
checksum: 4f5250ed42c07b384ba079fc1c6150186a9846c370111f8dac50b1e3795225ff
receipt:  ea5aaf52ae45e21dc9b8bcb9b18fcfcf1a2045f459701c3213cbc0065fbdb270
recipe:   bab38f8ed1bf4ab1d15843fc8ad3872c553e6665faf5eb97e432261f51f9b027
```

Metadata is 1,486,052 bytes. `SHA256SUMS` is 116,506 bytes. The 20,538-byte
[receipt](receipt.json) is also retained privately with identical bytes. It binds
all private evidence files, selected member hashes, complete observed Mach-O
load metadata, source URLs, release/asset IDs, limits, and inspection results.
The receipt does not contain its own hash; the binding above supplies it.

## Private storage and priority

Private base: `/Users/serge/.local/state/openergy-verification`.
New directory, mode0700:

```text
m1-python-prerequisite-2026-10-24-t3-27c54a9f
```

Exact archive path relative to that base, mode0600:

```text
m1-python-prerequisite-2026-10-24-t3-27c54a9f/cpython-3.13.9+20251014-aarch64-apple-darwin-install_only.tar.gz
```

All retained private files are mode0600. No archive member was extracted.
The archive stays outside Git. Before acquisition, the research agent set its
own Codex process PID `27777` to actual **NI15** and verified it with `ps`.
The host Python data-acquisition process also verified actual NI15.
`priority-before-acquisition.json` retains the process observation. All later
shell and Python checks inherited NI15 and checked it again.

## Complete static inspection

The task-authored scanner used the host Python standard library to read bytes.
It read every tar header and payload, hashed every regular file, and validated
the complete path/link graph. It created no archive-member files. Earlier scan
observations are retained. A final scan followed review changes to reject other
Mach-O formats and to make checksum reads start at offset zero. The results did
not change. No tests, mocks, supplied project helpers, or target code ran.

- Limits: download **268,435,456 bytes (256 MiB)**; regular payload and complete
  decompressed tar stream each **1,073,741,824 bytes (1 GiB)**; **12,000 members**
  and **12,000 physical headers**. Metadata download was capped at 4 MiB and
  checksum download at 1 MiB. Names are capped at 4,096 UTF-8 bytes and link
  resolution at 40 hops.
- Actual: **1,844 members and physical headers**; **1,836 regular files**,
  **8 symlinks**, no hardlinks, no explicit directory entries;
  **46,504,040 payload bytes**, **47,918,592 decompressed tar bytes**.
- No absolute/traversal paths, duplicate names, casefold/canonical Unicode
  aliases, unsafe path characters, devices, FIFOs, sparse files, PAX metadata,
  non-file payloads, special permission bits, or unsupported member types were
  accepted. Alias checks include all implied ancestor directories.
- No member has a file or symlink as an extraction parent. All eight symlinks
  use relative sibling names and resolve directly to internal regular files.
  There are no dangling links, cycles, or paths that escape the `python/` root.
- Executable path containment was checked as part of that graph. The future
  recipe strips all source execute/special bits and writes no file through a
  link. These checks do not establish that executable contents are trustworthy.

`patchlevel.h` declares `PY_VERSION="3.13.9"`, final release level and serial 0.
The literal `build_time_vars` dictionary declares `Py_GIL_DISABLED=0`, build and
host triples `aarch64-apple-darwin`, and deployment target `11.0`. The dictionary
was parsed with host `ast.literal_eval`; it was never imported.

All eight detected Mach-O files are thin arm64. Their `LC_BUILD_VERSION` data
declares macOS platform 1, minimum **11.0.0**, and build SDK **15.5.0**. The host
`file` tool read the validated interpreter bytes from stdin and reported
`Mach-O 64-bit executable arm64`. The main executable loads the bundled Python
library via `@executable_path/../lib/libpython3.13.dylib`. The library's install
name and sysconfig prefixes retain upstream `/install` values; this task made
no relocation edits. File metadata makes this an architecture/OS candidate for
the stated M1 target. It does not prove successful loading on macOS 27.0.1.

## Inert parent recipe

[safe-extraction-recipe.py.txt](safe-extraction-recipe.py.txt) is non-executable
text, Git mode100644. Its syntax passed a Python 3.9 grammar AST parse. Its shared
scanner source matches the task-authored static scanner. The extraction code
was reviewed as text and **was never executed**. No end-to-end extraction or
runtime result is claimed.

For a future parent-controlled operation, copy the private data and reviewed
recipe to the target. Use a new parent-owned mode0700 directory with no symlink
ancestors; preserve archive mode0600. Check the transferred byte counts and the
hashes above against this commit. The existing system Python 3.9.6 can interpret
the recipe; no package, bootstrap download, cache, or configuration change is
needed. This example is inert documentation and was not run:

```sh
/usr/bin/python3 -I -S -B /absolute/path/safe-extraction-recipe.py.txt \
  --extract-data-only /absolute/path/to/private-data-directory
```

The recipe sets and verifies its own NI15. It opens the archive without following
symlinks, requires a regular current-user-owned mode0600 file, checks exact size
and SHA256, and validates every member before creating any output. It refuses an
existing `extracted-data` destination. It creates directories at mode0700 and
regular files at mode0600 with directory descriptors, `O_NOFOLLOW`, and exclusive
creation. It copies only validated payload offsets, checks each file hash, creates
internal symlinks last, and rechecks the archive hash. It never restores source
owners, execute bits, timestamps, ACLs, or xattrs. It installs no packages and
runs no archive content. On an error it stops; partial output remains inert and
must not be treated as a completed install or reused as a fresh destination.

The future interpreter path would be
`extracted-data/python/bin/python3.13`. It intentionally remains mode0600.
Enabling owner execution and native qualification are separate parent actions.
The parent must assess code trust and the actual OS loader/security behavior,
then qualify the exact interpreter and required helper APIs such as
`hashlib.file_digest` under the current task scope. No shell-profile, PATH,
system Python, or global package change is part of this recipe.

## Evidence boundary

Official HTTPS metadata, asset IDs, the release's immutable flag, exact sizes,
and checksum agreement prove the observed publisher provenance and byte binding.
The checksum shares the publisher's trust domain. This work did not verify an
independent signature or attestation, audit source-to-binary equivalence, review
the code for security, or make a code-trust decision. It does not claim the
newest patch or a current security qualification.

There was no M1 connection, extraction, target Python execution/import, package
installation, key/credential use, global cache/config mutation, or product,
update, build, or native run. Historical **FAILED/UNVERIFIED** evidence remains
unchanged. The expired Sparkle approval remains closed and was not reused or
reopened. These prerequisite data do not change any prior gate or verdict.
