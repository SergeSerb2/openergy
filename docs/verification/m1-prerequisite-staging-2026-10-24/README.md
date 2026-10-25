# M1 prerequisite data staged — 2026-10-24

Seven prerequisite files, **266,257,298 bytes**, were transferred to one fresh
private directory on the user-verified M1 route. The receiver and the independent
post-transfer hash reader ran at actual nice 15. The destination is mode0700;
all seven regular files are current-user-owned mode0600 with one link. Their
byte counts and SHA-256 values match the prepared local inputs. The local
sources were checked again after the transfer.

This was a data copy, not an extraction or installation. No downloaded Python,
Rust compiler, source helper, product, app, updater, manifest resolution, build,
signing or package command was executed on the M1. The existing SSH configuration,
system Python, shell profile, toolchain defaults and installed apps were not
changed by this staging operation. Native qualification remains UNVERIFIED.

The files are the three [source/Rust/Cargo archives](../m1-build-unblock-2026-10-24/build-inputs.json),
the [pinned Python archive](../m1-python-prerequisite-data-2026-10-24-t3-27c54a9f/README.md),
their two receipts, and the inert Python extraction recipe. The parent independently
checked the retained official release metadata and SHA256SUMS against the actual
Python archive: 1,844 members, 1,836 regular files, eight symlinks and 46,504,040
payload bytes. It did not extract or execute archive contents.

## Preserved first transfer failure

The first data transfer failed with exit1 before the rsync receiver started.
Nested shell quoting damaged its inline `python -c` receiver command:
`SyntaxError: invalid syntax` and `zsh:1: unknown sort specifier`. Its private
directory, first receipt, exact executed stager, stdout and stderr remain intact.
No signal was needed. This was not a product/build attempt.

The corrected transfer used another fresh directory. A small non-executable
receiver source file replaced the nested inline shell code. System Python
reads that file, sets and verifies absolute nice15, then execs rsync. The
correction did not change file-byte/hash/mode checks, transfer limits, identity
checking or any previous product acceptance predicate. It completed in
**3.732603 seconds**. That is an operational transfer duration, not a product
performance result.

[staging.json](staging.json) binds both receipts and the exact executed stager
hashes. Full endpoint/path-bearing evidence remains in the dedicated private
verification-state tree. The failed first root was not reused, merged into or
deleted. No existing build database, output tree or workspace-state file was
copied.

The original ordinary run remains FAILED / NOT ACCEPTED (23/25), native Windows
Off stop remains FAILED / UNVERIFIED, and the expired Sparkle operation remains
closed. M1 setup does not repair or retry any of those results. Fresh Python
extraction source review and the separate M1/Windows operation plans are still
in progress at this checkpoint.
