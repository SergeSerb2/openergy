# Private M1 build tools qualified — 2026-10-24

The exact pinned Python interpreter loaded on the verified M1 and ran the real
`tools/release/release.py version` command successfully against a complete source
snapshot. Output was `0.1.0`, exit0, with empty stderr. Separately, the private
Rust and Cargo binaries both returned their pinned **1.99.1** versions, exit0,
with empty stderr. These are actual tool commands, not source-only predictions.

No app, daemon, updater, native input, manifest resolution, compiler build,
signing or packaging ran. Native app and release acceptance remain incomplete.
All setup used new private directories and per-process environment values.
No system Python, shell profile, default Rust toolchain or installed app was
changed by the agent.

## Exact qualification

The [independently reviewed Python extractor](../m1-python-extractor-review-2026-10-24-t3-c8ee917b/README.md)
ran once through the M1 system Python3.9 CLI. It exited0 with empty stderr in
0.924728 seconds. It wrote inert mode0600 files and checked their hashes.
The parent then verified the interpreter's ownership, regular-file identity,
single link and SHA-256 before enabling owner-only execution on that one file.
The interpreter hash remains
`d81701904d7a58f6abb3ab339c1d38d48cd24a39b37645cf7338c07a1e8e70e6`.

The successful release version command used `-I -S -B`. It took 0.157396 seconds.
It qualifies interpreter loading, the imports and real configuration-read path,
including `tomllib`. It does **not** exercise `hashlib.file_digest`, a compiler,
the GUI, update installation or native input. The pinned archive provenance is
recorded separately; no independent signature or source-to-binary equivalence
is claimed.

The Rust setup verified every one of the 225 regular toolchain files and all
210 Cargo seed files after copying. The toolchain contains 639,160,693 regular
bytes. Only files whose bound original source mode had owner-execute received
owner-execute in the private copy; 20 paths were enabled. Other files remain
mode0600, and directories are0700. The original local toolchain was not modified.
The real version commands used the private tool prefix and private HOME, TMP,
CARGO_HOME and RUSTUP_HOME. Bootstrap processes verified absolute nice15. No
claim about sampled compiler priorities is made because no build ran.

## Preserved source-preparation failures

The first real release-tool qualification failed before compilation: the
435-file source selection omitted `tools/e2e/Cargo.toml`. The exact first
receipt, private stderr and source tree remain unchanged. The root workspace
also requires the local `vendor/mdns-sd` patch. The initial selected archive is
not a complete build input set.

A subsequent local Git-archive export contained all 702 non-docs files but
stopped its raw-blob equality check. `.gitattributes` converted
`tools/windows/openergy-run.cmd` to CRLF. That archive and first mismatch are
retained; no build or M1 command used it. No comparison predicate was weakened.

The final new snapshot reads immutable raw Git blobs at
`ed7d96bf53da1a539fa2acde3b7133613221fd4b`. All **702 files**, **4,773,386 regular
bytes**, all **11 workspace members**, and `vendor/mdns-sd` match their Git blobs
and SHA-256 values. It includes every tracked root except `docs`, and no `.git`
metadata. The successful version command used this separate fresh source tree.
It does not establish Git-based release-package provenance.

Final source archive: **5,365,760 bytes**, SHA-256
`e28b00daeaf5de62ab12f1e0165a5ba1b815d22ca9139a74b54f599e20ada277`.
Its complete receipt SHA-256 is
`7ec86206f0ad7a0931451e46c0e056e6dea07223cd60e4e05d4037f1b4f5c0bb`.

[qualification.json](qualification.json) binds the new source, tool-command
results and private evidence labels. Full path-bearing records and executed
bootstrap source remain private. First failed results are preserved separately.
No historical FAILED / UNVERIFIED result, old guard or expired Sparkle approval
is changed. The M1 build plan and new Windows baseline still require their
independent source reviews and concrete run bindings.
