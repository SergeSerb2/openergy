# Independent M1 Python extractor review — 2026-10-24

**Source: CLEAR for the exact pinned data-only recipe. P1: none. P2: none.**
This verdict requires the documented parent-controlled directory with no
concurrent mutation. Extraction was **NOT RUN**. Interpreter, loader, helper,
and native product qualification remain **UNVERIFIED**.

Review worktree: `/Users/serge/.t3/worktrees/openergy/t3-c8ee917b`, starting branch
`t3/c8ee917b`. Before commit the branch was named `t3/m1-python-extractor-review`;
the worktree path and HEAD were unchanged. The starting checkout was clean at
`d131b6f35c9ec449812d2bf0e6c4552c749f3365`. Candidate
`83334d4235c91f4dddb55dd43eed2d1eabdc82a1` and the imported parent have
identical Git blobs for all three candidate files. No candidate file changed.

I read the complete candidate [README](../m1-python-prerequisite-data-2026-10-24-t3-27c54a9f/README.md),
[receipt](../m1-python-prerequisite-data-2026-10-24-t3-27c54a9f/receipt.json), and
[381-line recipe](../m1-python-prerequisite-data-2026-10-24-t3-27c54a9f/safe-extraction-recipe.py.txt).
I also read the relevant real `tools/release/release.py` entry path and the
retained M1 prerequisite context. Parent-supplied M1, strict SSH, physical
ED25519 verification, Xcode 27.1, Swift 6.4, SDK 27.0, macOS 27.0.1 arm64,
and system Python 3.9.6 context was not independently queried in this review.

## Independently inspected data

The [review-authored reader](read-retained-data.py.txt) used system
`/usr/bin/python3 -I -S -B`. It opened retained files read-only. It did not
import or execute the recipe, the supplied scanner, any archive content, or
any project helper. It parsed Python syntax and literal dictionaries as data.
It read the archive in memory and wrote no archive member to disk.

The single data-reader invocation exited 0 with empty
[stderr](data-reader.stderr.txt). Its complete structured observations are in
[data-observations.json](data-observations.json). This is independent data
inspection, not an extraction test or runtime result. The invocation was:

```sh
/usr/bin/python3 -I -S -B \
  docs/verification/m1-python-extractor-review-2026-10-24-t3-c8ee917b/read-retained-data.py.txt \
  > docs/verification/m1-python-extractor-review-2026-10-24-t3-c8ee917b/data-observations.json \
  2> docs/verification/m1-python-extractor-review-2026-10-24-t3-c8ee917b/data-reader.stderr.txt
```

All twelve private files listed by the receipt matched their exact byte
counts and SHA256 values. The private receipt matched the committed receipt.
All thirteen private files were regular mode0600, owner UID 501, with one
hardlink each. The private leaf directory was mode0700, owner UID 501.
File metadata showed no symlink ancestor in the exact retained path. These
are local file observations; they do not establish future target file state.

I parsed the retained primary `release-20251014.json` and `SHA256SUMS`, rather
than relying only on receipt assertions. They identify Astral release
`254523032`, tag `20251014`, published `2025-10-14T22:17:22Z`, immutable true,
draft false, and prerelease false. The exact archive asset is `304305223`;
the checksum asset is `304306156`. Both local asset sizes and digests match
the API data. There is exactly one checksum entry for the exact archive name.
No new network request or acquisition was made.

Exact SHA256 bindings:

```text
archive  16526691B  931db8f735e18700d4eab9ee39dbbd0b4c114d7d039dd2707b2d932ded039698
metadata  1486052B  e4b40842a5021bd46e84197bca665db337f13daec157ad9a0979081fd3a321b8
checksum   116506B  4f5250ed42c07b384ba079fc1c6150186a9846c370111f8dac50b1e3795225ff
receipt     20538B  ea5aaf52ae45e21dc9b8bcb9b18fcfcf1a2045f459701c3213cbc0065fbdb270
recipe      18787B  bab38f8ed1bf4ab1d15843fc8ad3872c553e6665faf5eb97e432261f51f9b027
manifest           59f67ef4f7da255b45d55e3d43ac5a0f8699d4986d59292b3ea59d57f2d2fdd7
```

An independent physical-header walk checked every tar checksum and payload
boundary. A separate standard-library `tarfile` reader agreed on each header
offset, payload offset, and size. I hashed every regular payload and checked
all padding and the final zero tail. All 1,844 complete member records matched
the retained scan and its manifest hash: 1,836 regular files, 8 symlinks,
46,504,040 payload bytes, and 47,918,592 decompressed bytes. There were no
explicit directories, hardlinks, GNU/PAX metadata, devices, or sparse entries.
The exact archive implies 163 directories. The longest path is 109 UTF-8
bytes; the longest component is 38 bytes. The largest file is 17,893,344 bytes.

All paths remain under `python/`, with no traversal, file/link parent,
duplicate name, or NFD/casefold alias. All eight links use a sibling basename
and target an existing internal regular file directly. The interpreter is a
50,248-byte regular file with SHA256
`d81701904d7a58f6abb3ab339c1d38d48cd24a39b37645cf7338c07a1e8e70e6`.

Independent byte parsing confirmed all eight detected Mach-O files are thin
arm64 and declare macOS minimum 11.0.0 and SDK 15.5.0. Load commands match
the receipt. The interpreter loads the bundled library through
`@executable_path/../lib/libpython3.13.dylib`. The library's `/install/...`
LC_ID_DYLIB is an install name, not a separate observed absolute dependency.
The retained upstream prefixes and remaining system-library dependencies
still require actual loader qualification. `patchlevel.h` declares 3.13.9
final, serial 0; the literal sysconfig dictionary declares standard GIL and
`aarch64-apple-darwin`. These statements describe bytes, not executed code.

## Source assessment

- **CLI and Python 3.9:** The explicit `--extract-data-only` entry requires
  exactly one absolute private-directory argument (lines 377–381). Syntax
  independently passed a Python 3.9 grammar parse. The code uses standard
  library interfaces available in that bootstrap generation. It does not
  need `tomllib`, `hashlib.file_digest`, site packages, or target Python to
  perform extraction. The documented `-I -S -B` invocation isolates imports
  and suppresses bytecode output. Actual macOS 3.9 execution was not done.
- **Validation before output:** `scan()` requires the exact size and SHA256
  on the opened regular, single-link, current-user-owned mode0600 archive
  (lines 120–127). It reads the entire compressed stream and validates every
  member and the complete graph before the first output mkdir (line 318).
  `require()` uses explicit exceptions, so optimization does not disable it.
- **Paths and links:** Root/component checks and alias checks include implied
  ancestors (lines 40–74, 183–194, 226–251). Complete graph checks reject a
  regular file or link as an extraction parent. For these exact bytes, all
  links are direct internal regular-file links; generic chain/hardlink
  branches are not needed. Directory traversal uses held descriptors and
  `O_DIRECTORY | O_NOFOLLOW` (lines 266–293). The fresh destination refuses
  reuse. No file is written through an archive link; symlinks come last.
- **Modes and metadata:** The recipe sets umask077; mkdir uses0700; regular
  creation uses 0600, `O_CREAT | O_EXCL | O_NOFOLLOW`, and `fchmod(0600)`
  (lines 300, 318–351). It never restores source execute/special bits,
  ownership, timestamps, ACLs, or xattrs. Symlink mode is not claimed to be
  0600; directory access contains those links. Failure leaves incomplete
  private output and cannot be reused as a fresh destination.
- **TOCTOU:** The archive is opened once through the held base descriptor,
  checked on that descriptor, and kept open. A nonblocking shared flock is
  advisory. Scan checks device/inode/size/mtime/ctime stability, then rewinds.
  Copy uses validated uncompressed offsets and checks each payload hash.
  The final archive hash starts at byte zero (lines 28–37, 252–256, 310–369).
  Directory descriptors and exclusive creation avoid path-following writes.
  The source explicitly requires no concurrent directory mutation (line 298).
  It does not prevent a same-owner writer or privileged process from changing
  files or replacing a directory between mkdir and open. This is a stated
  precondition, not a claimed defense against those actors. Future use must
  keep that precondition and recheck transferred bytes; this review does not
  transfer present local file observations to the target.
- **Bounds:** The exact archive is pinned below the 256 MiB archive cap.
  Scan caps raw expansion and regular payload separately at 1 GiB, logical and
  physical headers at 12,000, paths at 4,096 UTF-8 bytes, GNU metadata at 4,097
  bytes, and resolution depth at 40 (lines 18–20, 137–180, 199–221, 235–236).
  Writes have a separate 1 GiB counter and chunked reads; the exact archive has
  monotonically increasing regular-file payload offsets. There is no wall
  time or filesystem quota guarantee. These source limits plus the exact
  measured data support this case, not arbitrary replacement archives.

The seven scanner function ASTs in the recipe match those in the retained
research scanner. Neither scanner was executed here. This equality is a
source binding only; the independent reader supplies the data cross-check.
There is no concrete P1/P2 issue within the requested exact-data scope.

## Future interpreter activation and qualification

There is no identified source blocker to the parent first completing this
exact data-only extraction, then enabling only owner execution on the exact
regular `extracted-data/python/bin/python3.13` file, changing 0600 to 0700.
Before that change, bind the successful extraction and that file's identity,
hash, ownership, and mode on the target. Do not use a recursive chmod or
activate the archive's other executable files. The `python` and `python3`
symlinks already resolve to that same interpreter; they are not independent
binaries. This paragraph assesses the future action; this review performed
no extraction, chmod, executable activation, or target invocation.

The narrow real release-tool qualification can directly call the exact
absolute interpreter with `-I -S -B`, the exact source tree's absolute
`tools/release/release.py`, and **`version` with no `--tag`**. At this reviewed
source, lines 57–96 and 561–584 take the configuration-read path and print the
workspace version, currently `0.1.0`. This path does not call `run()`, `git()`,
`native()`, a compiler, a product binary, or a network command. Top-level
PATH/CARGO_BUILD_JOBS assignments affect that Python process only. Required
source configuration files must be present and bound with the script. Script
SHA256 is `96817425c76fa697fc10d7a073698358fb142e3f92995efd4e09931d49a1f995`.

A future exit 0 and expected version output would qualify loading this
interpreter, the standard-library imports used on that path (including
`tomllib`), and the real configuration-read path on that target. It would not
exercise `hashlib.file_digest`; its definition is present in the retained
stdlib as data, and release.py uses it on other paths. Its actual behavior
and other needed helper APIs remain unqualified by a `version` result.
Build, package, product launch, native input, update, and release acceptance
do not follow from that result. No new human-approval condition is introduced
by this source review.

## Residual limits and retained status

No independent signature or attestation, source-to-binary equivalence, code
security audit, or current security qualification was established. The
checksum and API asset digest share the publisher trust domain. Historical
HTTPS acquisition is retained evidence; this offline review does not replay
or independently authenticate that transport. Mach-O metadata supports the
target architecture candidate, but does not establish successful dyld load,
OS trust acceptance, or runtime correctness on macOS 27.0.1.

There was no extraction, supplied helper execution/import, test, mock,
runtime qualification, remote connection, new network fetch, process/port/OS
query, or product/native/update action. Only this new review folder was
added. No global Python/Homebrew/PATH/config installation occurred.
Original FAILED/UNVERIFIED runs remain unchanged, including the original
ordinary 23/25 result and old Windows Off-stop status. The expired Sparkle
operation remains closed. This review changes no historical verdict.

[review-bindings.json](review-bindings.json) binds the report, reader,
observations, empty stderr, candidate Git blobs, and source input hashes.
Source **CLEAR** remains separate from extraction **NOT RUN** and native
qualification **UNVERIFIED**.
