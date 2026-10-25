# Future parent CLI

This is an inert interface description. It does not authorize execution.
Run the exact source text directly with Python, or copy it byte-for-byte to a
future private source directory. Keep `metadata-query.ps1.txt` beside the reader.
No source substitution is supported. No package installation is needed.

```text
python3 /absolute/path/metadata_reader.py.txt \
  --binding /absolute/private/path/command.json \
  --binding-sha256 <64-lowercase-hex-of-exact-command.json-bytes> \
  --run-dir /absolute/private/mode-0700-parent/new-unique-run \
  --work-seconds <positive-seconds> \
  --cleanup-seconds <positive-seconds> \
  --finalize-seconds <positive-seconds> \
  --total-seconds <positive-seconds> \
  --stdout-cap <positive-bytes> \
  --stderr-cap <positive-bytes> \
  --aggregate-cap <positive-bytes>
```

All options are mandatory. No time or capture cap has a CLI default. All times
must be finite. Work + cleanup + finalization must be <= total, and total must
be <= 120 seconds. All three byte caps must be <= 1,048,576. A smaller combined
cap takes precedence. The absolute clock starts on entry to capture, before any
file preparation. The work phase cannot use the cleanup/finalization allocation.
Cleanup ends at the earlier of its one shared allowance or total minus the
finalization reserve. An earlier work failure does not reset total time.

Dependencies: Python >=3.10 with the standard library, a POSIX host with
nonblocking pipe/selector, O_NOFOLLOW and O_DIRECTORY support, and a writable
private filesystem that supports file and directory fsync. The supplied private
parent must be owned by the current user and mode 0700. The run directory must
not exist. The parent must keep all outputs private and outside the checkout.
There is no target-discovery, environment, SSH configuration or credential API.
OpenSSH and Windows PowerShell are dependencies only for a separately assigned
Windows metadata operation; their installed versions remain UNVERIFIED.

The new directory contains only `stdout.bin`, `stderr.bin`, and `receipt.json`.
Raw files are mode 0600. The receipt is fixed at 65,536 bytes; trailing whitespace
is JSON padding. It binds preserved byte counts/hashes, observed-but-not-saved
counts, EOF/overflow, the first error class/phase, local spawn/kill/exit evidence,
caller budgets/caps, and reader/query/input hashes. It records no argv or exception text. Raw
streams can still contain private data and must not be published automatically.
The CLI prints no raw data. Exit 0 requires complete local capture, local exit 0,
no first error, both EOFs, final work before the absolute deadline, and (in
metadata mode) all eight valid records and empty stderr. All other outcomes use
exit 1. `--help` uses argparse's normal help exit. Local verification nonzero is
not propagated as the reader's exit: its exact value stays in the receipt.

Two closed binding shapes are accepted. The [contract](input-contract.json)
describes them; it is not an executable binding or authority receipt.

- `purpose=local-reader-verification`: exact keys `schema`, `purpose`, `argv`.
  The future parent supplies an explicit absolute executable and its exact argv.
  This exercises the same full capture/settlement/receipt CLI. It skips Windows
  response parsing. It cannot supply Windows qualification or metadata proof.
- `purpose=windows-installed-metadata`: also requires `query_sha256`,
  `selector_binding_sha256`, and `command_layers`. The reader does not open a
  selector. The future parent must obtain and bind that data under its own scope.
  Argv must be `/usr/bin/ssh`, the exact source SSH options, one pre-bound selector,
  and the six-element native argv: `powershell.exe -NoLogo -NoProfile
  -NonInteractive -EncodedCommand <Base64-of-exact-query-UTF16LE>`.
  There is no shell=True, local shell interpolation, compressed payload or stdin
  script. `query_sha256` binds the adjacent query's exact UTF-8 file bytes.

Argv is bounded at 64 strings and 65,536 UTF-8 bytes including argument NULs.
It contains no embedded NUL. The complete Windows native line and the complete
1..4 caller-bound layers each must fit their policy and any supplied effective
limit. Layers use R6 `literal_native` or `resolved_executable` only. Their field
is `effective_limit_including_nul`: explicit integer 1..32767 or null for unknown.
Null supplies no installed limit evidence. No installed version or independent
review assertion is fabricated by the reader.

A future parent can choose explicit metadata budgets of 30 / 3 / 2 / 35 seconds
and caps 524288 / 65536 / 589824 bytes (stdout / stderr / combined). These are a
finite proposed allocation, not materialized inputs or evidence that Windows
will answer in time. The query itself refuses work after 20 seconds between OS
calls. It caps each file at 268435456 bytes, combined hashed bytes at 536870912,
runtimeconfig at 65536, each output record at 262144 UTF-8 bytes and total stdout
at 524288. An over-cap runtimeconfig fails; its content is never truncated to fit.
Version strings are capped at 4096 characters each. JSON admission first uses a
conservative six-byte-per-UTF16-unit escaping bound, then checks actual bytes.
This can refuse a record below the exact byte cap; it cannot omit its fields.

Only an independent external parent can bound a stuck OS call. It must record
the actual reader exit and external elapsed time. The receipt alone does not
prove completion. A local child kill, a metadata end record, SSH exit 0 or reader
exit 0 does not prove the remote process's current state or effective limits.
