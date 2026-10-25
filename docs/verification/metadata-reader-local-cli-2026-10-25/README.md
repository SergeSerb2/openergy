# Metadata reader: full local command flow

The driver invokes the complete reader CLI once for each real local command:
`uname`, `false`, a shell that emits a prefix and execs `sleep`, and `yes`.
It checks actual reader and child exits, exact saved output hashes, the byte
cap, partial output after timeout, and collection after overflow. It stops on
the first unexpected result. It uses no imported reader functions or mocks.

Status: **PREPARED; NOT RUN**. Python AST parsing passed. The reader source is
still being prepared in its separately owned worktree. Its final exact hash
must be supplied before this driver can run. Keep its source unchanged during
verification. Independent review and actual CLI results remain pending.

The caller supplies an absent output path below an owned private mode-0700
directory. Raw command bindings, output and receipts stay there. The driver
runs at nice15 and has a 28-second work envelope plus at most two seconds for
reaping its retained reader handle after an outer timeout. An outer timeout
leaves the reader's child state UNVERIFIED and stops further cases. The tool
does not promise to interrupt an unresponsive kernel or filesystem call.

No remote connection, native Windows query, product launch, old fixture, old
guard reset, repeated case, TypeScript change or release action is part of this
flow. Local results cannot accept the Windows stop path, establish installed
launcher limits, or change any historical FAILED / UNVERIFIED result.

```text
python3 -B verify-cli.py --reader /absolute/path/metadata_reader.py.txt \
  --reader-sha256 EXACT_REVIEWED_SHA256 \
  --output /absolute/private/owned-parent/new-unique-flow
```
