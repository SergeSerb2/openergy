# Independent Windows R6 SOURCE/DATA review

Date: 2026-10-24, America/Vancouver. `execution=false`.

**SOURCE CLEAR for `4410818a9332292bb8bcbb24d7df5d21fa17ee9a`: no concrete
P1 or P2 found. The retained child and wrapper shapes are representable in v2.
The complete execution binding remains UNSATISFIED. Native syntax and runtime
remain UNVERIFIED.** One nonblocking documentation arithmetic error is recorded
below. This review creates no executable receipt, private preparation, admission
assertion, or new human approval gate.

The reviewed package is
`docs/verification/windows-admission-contract-repair-2026-10-24-3a26b095/`,
called P below. Its immutable Git tree is
`20d68fc7319c0560b5bef7a03c14dca04a3d146d`. All 23 files read in the supplied
parent folder match that commit. All 21 manifest members and 22 SHA256SUMS
members match, with exact membership. The reviewer started clean on
`t3/review-windows-r6-launch-binding` at
`b1188f1812ba51dd8d2eab78e4f942b266daa9c0`.

The full R6 handoff, all eight complete inert sources, contracts, finite plan,
result schema, evidence files, and exact R3 delta were read. The full R4 source
report and R5 report/data were read at their supplied external locations. Their
bytes match commits `b5e93a2fafbf6850798abc8d6018851b3c68d648` and
`a6552ba88a3c96063f558a5edc4cdb80a5234778`. All 27 declared immutable input
bindings match: 24 R3 files and three prior report files. R3 is
`23b41681e4019f086f1f760c9817553d79acff04`. Its response to the original
oversized-recovery finding was also read. The prior reports supply context;
their source clearance does not supply R6 native proof.

The exact 33,739-byte R3 patch regenerates from the six listed carried files.
Only `transport_argv` and `Controller.prepare_inputs` change in the controller.
All other function source segments are byte-exact R3. Module statements are
equal as AST data. Transport statements outside the layer loop are equal as
AST data. The preparation prefix and suffix outside the binding block are
byte-exact. All seven native files and the result schema are byte-exact R3.
The historical, baseline, budgets, data bounds, one-shot, pins, failure, removal,
and proof-limit plan sections retain their R3 values.

At `P/source/controller.py.txt:73-88`, the native argv still begins with the
fixed bare `powershell.exe` token and the same four flags and encoded argument.
The new loop only constructs admission strings. It does not put the bound
executable, prefixes, suffixes, or limits into the returned SSH argv.
`literal_native` retains the whole unchanged native line. `resolved_executable`
adds two executable quotes around the exact bound path, then takes the entire
serialized tail after `len(native[0])`. That token is 14 ASCII/UTF-16 units.
Its Windows serialization is unchanged and unquoted, so this cut retains the
leading separator, every flag, and the full encoded argument. There is no
search-and-replace, recursive transformation, evaluation, or callback.

At `:141-159`, v1, missing review assertions, a different selector binding,
empty or more than four layers, and unknown constructions refuse admission.
Each construction has an exact entry key set. Extra keys, mixed shapes, missing
fields, non-string text, or malformed JSON value types cannot reach a launch.
The limit check uses `type(...) is int`, so JSON booleans, floats, strings, zero,
negative values, and values above 32,767 refuse. A malformed layer can raise a
key/type exception instead of a custom diagnostic; the enclosing failure path
still prevents setup and transport. Python optimization is explicitly refused
before these assertions. The envelope remains an evidence-backed input: its
two true booleans cannot establish route coverage, versions, or effective limits.

The resolved path guard requires an ASCII drive letter, a local backslash root,
one colon, exact `ntpath.normpath` spelling, and a `powershell.exe` leaf under
case folding. Relative, drive-relative, UNC/device, parent/dot, mixed-separator,
redundant-separator, trailing-separator, stream-colon, wrong-leaf, embedded quote,
NUL, CR, and LF forms fail the stated checks. The guard preserves the supplied
case and Unicode spelling. Spaces and valid non-ASCII path characters remain
inside executable quotes and contribute their full serialized width. The fixed
leaf prevents a trailing backslash from consuming the closing quote. This is
an admission-path text constraint, not a complete Windows filename validator or
proof that a file exists. Exact retained route evidence must supply the path;
the new field neither discovers nor selects an executable for launch.

The literal prefix and suffix can contain quotes, whitespace, backslashes,
shell-looking text, and Unicode because they describe an already serialized
bound line. The source concatenates and measures them without interpretation.
Their actual spelling and meaning still require independent route evidence.
There is no shell escaping or expansion step in this admission loop. NUL in
either complete construction refuses at `:84`. Strict UTF-16LE encoding at
`:85` rejects unpaired surrogates and counts a supplementary character as two
code units. Every complete line includes one NUL and has its own independently
bound limit of at most 32,767. An equal-width line fits; any excess refuses.
These conclusions come from source inspection. No hostile fixture, renderer,
parser, mock, or simulation was run.

The saved route evidence was read only as inert data. The retained receipt
SHA-256 is
`5fd1f40dc8caf85226a9ce047862d5c72a34ef0d6ab68aa02f96775ffc2a2c00`.
Its command/stdout/stderr hashes match; parsed stdout equals metadata; the
saved Base64 argument decodes to the exact saved query text. The receipt
records exit 0, zero retries, and no product/controller execution. All six saved
parent links match. The saved files were hash-identical before and after these
reads. No selector file was opened, and its current hash was not independently
recomputed. The receipt hash binds the same retained observation used by R4/R5.

Independent comparisons of the existing strings establish both exact relations:

- The child is its quoted 57-unit saved executable plus the entire unchanged
  native tail. Its path passes the new text guard and matches saved command
  resolution. The complete child still does not contain the unchanged full
  native line, as R5 reported.
- The wrapper is a 45-unit prefix, the whole native line, and a one-unit closing
  quote. Its prefix uses the exact saved executable spelling and command option.
  The saved registry spelling differs in case, so it cannot replace that text.
- The three sshd ancestor commands carry no payload and are not additional
  per-payload wrapper templates.

Complete retained sizes, including one NUL, are 2,415 submitted, 2,460 child,
and 2,461 wrapper UTF-16 units. Thus R5's exact-representation objection is
resolved for this observation by the two closed constructions. No private
binding object or claimed effective limit was created. The short query does
not establish a future route, all implementation limits, or near-ceiling
native acceptance.

At `:203-213`, every `remote` call still encodes the main script once, constructs
the unchanged exact-owner recovery body around that complete encoded argument,
and separately encodes/admit its recovery. Both native lines and every bound
layer pass before `command` can launch the main command. This covers setup,
all four stages, cleanup, and the retained-root audit. Recovery at `:244-246`
uses that exact admitted argv directly. Admission failures propagate without
a launch, fallback, truncation, shorter selector, timer reset, or retry.
The other two external-command paths remain fixed-file scp upload/download.

The R3 encoder and decoder at `:41-71` are byte-exact. Source remains limited
to 1..65,536 strict UTF-8 bytes without NUL. Main and recovery each have the
10,400-character compressed Base64 cap. The complete 1,588-unit wrapper template
has a 12,025-unit substituted maximum and a 32,068-character encoded maximum.
The full recovery source maximum is 34,183 bytes and must independently fit its
compressed cap. Length/hash checks, 4,096-byte bounded expansion reads, strict
UTF-8 decoding, first-error retention, and independent resource disposal remain
before script creation/invocation. No codec behavior was exercised.

Conditional on the saved route construction remaining exact, the maxima for
BOTH commands are 32,135 submitted, 32,180 resolved child, and 32,181 wrapper,
including NUL. Their margins under the source's 32,767 ceiling are 632, 587,
and 586. The child prefix is `2 + 57 + (66 - 14) = 111`. These are complete-line
string bounds. Every actual line is still measured against its own effective
limit; the source does not use the wrapper/child inequality as a substitute.
The unchanged nested PowerShell 7 gate at `:248-256` measures its quoted path,
all flags, and the entire encoded invocation before the stage launch. The
inherited conservative product-line bound is 13,311 including NUL. Native
Cargo/tool/tar and compiler-generated lines retain their prior proof scope.

**P3-DOC-1, nonblocking:** `P/COMMAND-BOUNDS.md:121` prints
`(2 + 57 + (66 - 13)) + 32,068 + 1 = 32,180`. The left side is 32,181.
Replace `13` with `14` to match the fixed token, source slicing, observed child,
111-unit child prefix, and stated 32,180 bound. The code uses `len(native[0])`
correctly, so this typo does not weaken command admission or change the source
verdict. The first review arithmetic pass copied this document constant and
recorded 32,181 for the child. `static-data-checks.json` retains that result and
the correction from the 14-unit source token. No candidate source/input changed.

All ownership and failure paths retain their R3 bytes. The recovery selector
still uses the complete encoded suffix, a unique owner, sampled descendants,
creation/command/path rechecks, retained-handle identity, bounded waits, and
disposal. Zero/non-unique owners retain their prior outcomes. The controller
keeps the 2,400-second clock, 30-second transport reserve, exclusive guards,
immediate stdin EOF, strict SSH options, and precomputed one-shot recovery.
Unknown processes grant no kill or removal authority. Failed collection keeps
the first flow error and attempts one bounded non-destructive retained-root audit.

The native texts preserve the original intent: one pinned Rust/Cargo 1.99.1
locked/offline release build, jobs 2, private cache/target/TEMP, installed MSVC/SDK,
BelowNormal/session 0, and no Clippy/install/reuse fallback. The build still
registers ownership before creation, pins the root handle first, uses the direct
PID API with its narrow missing-PID catch, verifies sampled generations against
retained parents, and settles/disposes all registered handles under the shared
deadline. Forced recovery and late/failed receipts cannot produce acceptance.

The unchanged observer/runtime retain the one R2 virtual Off CLI flow, suspended
creation and explicit NUL/stdout/stderr handles, pinned shared log reader,
4,096-byte polls, 2 MiB cap, and the one 15-second readiness window. Readiness
does not supply health. Version, absent-daemon failure, actual healthy status,
config, invalid-method error, real wire IDs/PIDs, one stop forwarding attempt,
exact ACK, empty stop stderr, CLI exit 0, and independent retained-handle daemon
exit 0 remain required. Observer settlement precedes the final forwarding
snapshot. The full ten-second normal-stop allowance precedes eligible recovery.
ACL/SID/nonce/numeric directory generation, containment, reparse checks, verified
collection before removal, and per-entry nonrecursive cleanup remain unchanged.

Installed launcher/PowerShell implementation versions and effective limits remain
UNVERIFIED. The package preserves the parent's single VersionInfo/GetFileHash
query failure: its SSH parent timed out after 30 seconds before output, with no
retry or candidate/build/daemon/stop attempt. The package binds that first-failure
record to SHA-256
`1a340873cdaa1d4c92707364d780c02013ea9fdd514333b872931c4ad99ce434`.
This reviewer checked that the immutable package preserves this report and hash;
the separate private failure file was not reopened. No version or effective
limit is inferred. Exact future clean source/archive/manifest, selection, same
selector, installed implementation/configuration, complete layers, and actual
limits remain missing execution inputs. The v2 schema and true-boolean gates
keep an absent binding closed; external evidence is still needed before those
assertions can be truthfully materialized. This review adds no query or approval.

The supplied whole-list multiline-literal equality remains **false**: R3 has
nine and R6 has ten. Independent counted comparisons preserve all nine original
occurrences exactly. The sole addition is the four-character quote/NUL/CR/LF
path guard. The supplied failed predicate and traceback remain unchanged. That
reader-scope failure is retained separately from native-source equality.

The original Windows result stays **FAILED / UNVERIFIED**, with **zero
daemon.stop requests**. Historical preservation stays **UNVERIFIED**, **0/9**
qualified bindings after the closed 160-retrieval search. The expired Sparkle
window stays **CLOSED**. No historical record was changed or reconstructed.

Native syntax, installed decoder/launcher behavior, process/handle identity,
ACL/reparse/filesystem behavior, build settlement, collection, cleanup, and stop
remain UNVERIFIED. Arbitrary OS-call preemption, same-user path races, unsampled
generations, external SSH/job state, uncapped output, and total memory/disk retain
their stated limits. Static source or short-query sizes do not supply native,
package, signing, release, or deployment acceptance.

Only this review folder was written. Work used standard-library file/Git/JSON/
AST-as-data/string/hash readers. No supplied task/helper/checker/AST was executed
or imported. No candidate renderer, transport compressor/decoder, test, mock,
simulation, native parser, remote/network action, OS/process probe, private write,
build/product flow, source integration, signing, child agent, push, or publication
occurred. The review commit disables Git hooks and signing. Detailed immutable
bindings and retained false predicates are in `static-data-checks.json`; the
compact verdict is in `review.json`; `SHA256SUMS` covers all review files except
itself.
