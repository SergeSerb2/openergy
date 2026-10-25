# Disabled future CLI requirements

These are source handoff requirements, not commands executed in this task.
Use the three exact source hashes in SHA256SUMS after parent review. Make new
immutable private copies with the reader and query adjacent under their present
names. The full driver stays `verify-cli.py.txt`; Python can run an explicit text
path. Do not import either file, modify old receipts, or reuse an old run directory.

The local driver interface is:

```text
<qualified-cpython> -B <private-copy>/verify-cli.py.txt
  --reader <private-copy>/metadata_reader.py.txt
  --reader-sha256 <frozen-reader-sha256>
  --output <new-absolute-private-output-directory>
```

The output directory must not exist. Its parent must be owned by the caller,
mode 0700. The driver requires its own nice value 15. It runs each fixed real
local case once: uname success, false nonzero, partial-output sleep timeout,
and yes output cap. It stops on the first failure. There are no mocks, receipt
simulations, retries or remote commands. The four commands and their predicates
remain the `93cd2e6` cases, including actual SIGPIPE or consistent SIGKILL for yes.

Use the same executable for the parent observer, driver and reader on one Mac.
Qualification is macOS CPython >=3.10; validate the requested local 3.15 separately
from M1 3.13.9. The driver rejects other platforms/interpreters and mismatched
clock descriptors. Each reader is a direct Popen child, with no interpreter
wrapper or remote launcher. Compare only time.monotonic values from that local
process family. A receipt copied from another host is not comparison evidence.

Reader budget allocations are work 2 seconds (0.25 for partial-timeout), cleanup
0.5, finalize 0.5 and total work+1. A separate 1-second interpreter-startup
allowance starts before reader spawn. The invocation deadline is the earlier
of that outer allowance and the 28-second whole-flow deadline. The driver keeps
up to 2 additional seconds only to collect its owned reader after outer timeout;
that branch fails and makes no descendant or remote cleanup claim.

The driver must collect the actual reader exit and record a parent monotonic
timestamp immediately after wait. It must validate the initial receipt's clock
basis, finite ordered start/work/absolute bounds, exact budget arithmetic,
source/binding/caps, and startup interval. Accept only an exit observation before
both the reader absolute and driver invocation bounds. Preserve all D3 error,
EOF, byte/hash and kill predicates. Keep D2 checks after wait, evidence reads and
validation, before flow success and after the result write at CLI return.
Earlier receipt elapsed/Boolean fields cannot replace exit observation.

The separately assigned parent must retain actual driver exit 0 and observe it
within the 28-second flow bound, using that invocation's recorded local start and
the same shared monotonic clock. It must also provide the already-authorized
external execution envelope. A stale pass file, missing exit, late observation,
reader absolute overrun, interrupted write or unknown child exit fails acceptance.
Hard OS-call preemption is not implemented or proved by either program. Do not
add a watchdog or broaden process authority under this proposal.

The reader's separate interface remains:

```text
<qualified-cpython> -B <private-copy>/metadata_reader.py.txt
  --binding <absolute-private-command-json>
  --binding-sha256 <sha256-of-exact-command-file-bytes>
  --run-dir <new-absolute-private-run-directory>
  --work-seconds <finite-positive-work>
  --cleanup-seconds <finite-positive-cleanup>
  --finalize-seconds <finite-positive-finalize>
  --total-seconds <finite-positive-total-at-most-120>
  --stdout-cap <1-through-1048576>
  --stderr-cap <1-through-1048576>
  --aggregate-cap <1-through-1048576>
```

Work+cleanup+finalize must not exceed total. Streams are independent and share
the aggregate cap. The receipt is a fixed 65,536 bytes. Input is capped at
131,072 bytes; the adjacent query is hash-bound. Local bindings retain schema
`openergy.metadata-reader-command.v1`, purpose `local-reader-verification`,
and exact argv, with no extra keys.

Remote execution is disabled in this package. Any later separately assigned
metadata operation must use purpose `windows-installed-metadata`, exact argv,
query_sha256, the prebound private selector_binding_sha256 and command_layers.
The client prefix must equal bindings.json exactly. Keep the six native tokens
and the literal_native/resolved_executable layer constructors unchanged. Bind
the actual complete layers; an explicit null effective limit remains UNVERIFIED.
The 32,767 UTF-16-unit ceiling is policy, not an installed effective limit.
Use only the six fixed installed files. Do not infer config/default/route/native
qualification from prefix equality, local success or complete metadata output.
Raw command, selector, metadata and output must remain in new private storage.
