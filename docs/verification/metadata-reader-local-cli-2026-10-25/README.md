# Metadata reader: full local command flow

The driver invokes the complete reader CLI once for each real local command:
`uname`, `false`, a shell that emits a prefix and execs `sleep`, and `yes`.
It checks actual reader and child exits, exact saved output hashes, the byte
cap, partial output after timeout, and collection after overflow. It stops on
the first unexpected result. It uses no imported reader functions or mocks.

The first driver revision ran once against reader SHA-256
`e7fb7a8fde0f63246796d5209f85571d2de0418ab96536797101561bec39b380`.
Success, nonzero exit and partial-timeout cases passed. The output-cap case
stopped the driver with exit1: the reader collected actual child exit -13
(SIGPIPE), but the driver incorrectly required SIGKILL and a local kill call.
The original result remains **INCOMPLETE / FAILED** and is retained unchanged.

The corrected driver accepts actual collected SIGPIPE termination or a recorded
local kill followed by actual SIGKILL termination for that output-cap case. It
also requires the exact 128-byte expected prefix, the single overflow sentinel,
OverflowError and the saved stream hashes. No missing exit is accepted. This
does not change any product predicate or accept the first driver invocation.
The corrected revision `02ca76e` then ran one full flow in a separate fresh
private directory. All four cases passed their CLI checks; the driver exited0.
The reader exited0 for success and1 for each failure case. The actual collected
child exits were0,1,-9,-13 respectively. The same reader bytes were used in
both flows. No previous guard, case or saved result was rewritten.

[results.json](results.json) retains both invocations and their distinct
verdicts. [private-evidence-index.json](private-evidence-index.json) binds every
retained file by relative label, length, mode and hash. The corrected invocation
provides local reader CLI evidence. Independent review remains **PENDING**;
Windows metadata, native stop and installed limits remain **UNVERIFIED**.

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
