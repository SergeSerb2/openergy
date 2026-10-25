# Windows metadata parent reader source proposal

`execution=false`. Parent: `b221d9807cbc4c3db5a30f2114399ce20942dcf1`.
Only this new folder is owned. Independent review is pending. Python syntax was
read as AST data. PowerShell syntax and all runtime behavior remain UNVERIFIED.
No source in this folder was imported or executed.

The old 30-second metadata failure stays TIMED_OUT_UNVERIFIED. Its reader lost
stdout/stderr when `subprocess.TimeoutExpired` escaped. The old query accumulated
all file objects before its sole JSON output. A slow file can therefore prevent
any file receipt. The retained command alone cannot identify which read stalled.
The private failure record supplies no exact local SSH exit status and no remote
exit evidence. These missing facts remain UNVERIFIED. No retry occurred here.

The proposed [parent reader](metadata_reader.py.txt) has an actual
[CLI](CLI.md). It launches one exact caller-bound argv with immediate stdin EOF.
It uses two nonblocking pipes and one selector. It writes raw bytes in chunks of
at most 4,096 bytes to exclusive private files. It checks each stream cap and the
combined cap before each read/write. At a full cap it reads at most one sentinel
byte to distinguish exact EOF from overflow. It records that byte as observed but
not retained. It does not silently call a truncated capture complete.

Timeout and other `BaseException` paths retain the first error class and phase.
They do not format exception text, which can contain connection data. Raw chunks
are written without user-space buffering. The failure path attempts fsync and a
fixed-size receipt before local settlement. A partial write records its exact
preserved byte count and hash. If a read chunk was not fully saved, both pipes
close so later bytes cannot turn the raw prefix into a file with gaps.

Settlement can kill only the retained local Popen child. It never starts another
command. Stream drain failure closes the pipes and then collects local exit in a
separate nonblocking poll loop. A repeated overflow cannot skip that loop. Exact
local return codes remain separate: SSH exit 255 stays 255; a POSIX signal stays
negative; an unobserved exit stays null. Reader CLI failure is exit 1. A local SSH
kill does **not** prove that the remote query stopped. Remote exit/current state
and remote cleanup remain UNVERIFIED in every receipt.

The [matching query](metadata-query.ps1.txt) emits a begin record, one complete
record per file, and a completion record. It retains the original six exact
installed paths, lengths, FileVersion/ProductVersion fields, full SHA-256, full
runtimeconfig text, own BelowNormal priority, own session, and metadata-only
scope. It uses one bounded streaming full-file hash per file. A size/time cap or
missing field fails the query; no shortened hash or omitted file can complete.
Version strings have a 4,096-character policy cap. Before JSON conversion, a
conservative escaping bound enforces each record and remaining output allowance.
That bound can refuse data whose exact serialized line would be smaller; refusal
retains failure, never a shortened metadata record.
Only the runtimeconfig is buffered, with a 65,536-byte ceiling and BOM-aware
strict UTF-8 default decoding. Unsupported text fails instead of guessing.
There is no process inventory, target discovery, product launch, transport config
or registry read, effective-limit probe, remote retry, recovery command, or compressed wrapper.

The reader checks the exact query hash and exact uncompressed native argv against
the future parent's private binding. It preserves the existing fixed SSH options.
It measures the complete submitted line and every supplied R6 closed command
layer as UTF-16 including NUL. Each has a 32,767-unit policy ceiling. A separately
bound smaller effective limit is enforced when supplied. An explicit null stays
UNVERIFIED; the reader never substitutes the API ceiling for an installed limit.
The caller must supply the actual complete route. Size data does not prove route
coverage or native acceptance. No private binding or selector was materialized.
The R6 controller's qualification gates and compressed recovery stay unchanged.

Safety ceilings in this proposal are refusal policies, not claims about installed
file sizes, implementation versions, configuration, command limits or resources.
See [the input contract](input-contract.json) and [finite plan](plan.json).
Even a future complete metadata response does not prove effective limits or
qualify Windows build/runtime/stop behavior. Original Windows FAILED/UNVERIFIED,
zero daemon.stop requests, and historical preservation UNVERIFIED stay unchanged.

Deadline checks bound requested selector waits, sleeps and repeated work. They
cannot interrupt a stuck fsync, open, read, write, process spawn, close, metadata
API, or other OS call. Hard wall-clock enforcement is **not proved**. A future
parent must supply an already-authorized external execution envelope and retain
its own elapsed time, reader exit and evidence. This package adds no watchdog or
authority to launch one. If that prerequisite is absent, execution stays closed.
Cleanup uses a shared remaining deadline, with a finalization reserve; it never
resets the absolute clock. No cleanup or success claim survives a missing/late
reader exit, unknown local exit, truncated stream or invalid metadata response.

Each raw file is bounded before capture; combined raw storage is at most the
chosen combined cap (maximum 1 MiB). The receipt occupies 65,536 bytes, allocated
before launch. Input binding is capped at 131,072 bytes. No unbounded communicate,
ReadToEnd on a live child, spool file, queue, or read-all child buffer is used.
Final metadata parsing is bounded by the captured stdout cap. These source caps
do not prove sufficient free disk/RAM, hard OS preemption or crash durability.
Disk faults can leave only an earlier receipt and a prefix. The caller must
retain a nonzero/missing reader exit as INCOMPLETE even if an earlier receipt
says COMPLETE. Receipt fields are never sufficient without the actual CLI exit
and the parent's external time evidence. Directory/receipt writes are not a
transaction across a host crash. Same-user path races and concurrent changes to
installed files are not eliminated. A full hash is not a provenance claim.

[Input bindings](input-bindings.json), [static checks](static-checks.json), the
manifest and SHA256SUMS bind this handoff. Existing public evidence and the two
permitted private data files were checked without modification. No auth, secret
or selector file was opened. No private file was written. No remote/network or
OS/process/port query, supplied source import/execution, native parser, helper,
test/mock/fixture, build, product, signing, child delegation, push or publication
ran. The commit disables hooks and signing.
