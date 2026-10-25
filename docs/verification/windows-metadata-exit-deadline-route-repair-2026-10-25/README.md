# Reader exit deadline and saved client prefix repair

Source proposal only. Execution is disabled; this revision is NOT RUN / UNVERIFIED.
Base: `ab036fef8eb80f619fe4d146d7d3748331ff97ca`. Only this new folder is owned.
The full goal is incomplete. Parent review and real full CLI validation follow
the source freeze. This package is not an approval or a terminal result.

The inert [reader](metadata_reader.py.txt) changes the reader at `97cc8f9` in
two ways. Its initial fixed-size INCOMPLETE receipt includes the local monotonic
clock basis, reader start, work deadline and absolute deadline before spawn.
Its closed SSH prefix now matches the saved metadata-query client prefix.
The [full driver](verify-cli.py.txt) extends `93cd2e6`. The 87-line
[query](metadata-query.ps1.txt) is byte-exact, SHA-256
`3404c46e256d41c17c1c7efaea1364216075f7988e1e5d38f588b43c92358443`.
The two small patch texts contain only these source deltas.

D4 is addressed in source by requiring an independent parent observation of
reader exit strictly before the reader's bound absolute deadline, as well as
the separate invocation deadline. This applies to all four cases, including
expected reader exit 1. The driver validates finite, ordered bounds, their exact
declared budget arithmetic, the startup interval, source/command/cap bindings
and the clock basis. The parent timestamp follows the real wait and precedes
receipt reads. A late observation fails even if the reader actually exited
earlier. Neither saved elapsed time nor the earlier deadline Boolean can pass
this check. Missing or invalid evidence fails the case; no exit is synthesized.

The reader's receipt fsync can still return after its absolute deadline and leave
earlier bytes on disk. The new acceptance rule rejects D4 on that path. It does
not prove transaction durability, eliminate other storage/close faults, or
interrupt OS stalls. `hard_wall_clock_enforced=false` and
`external_envelope_required=true` remain. Actual driver exit and an independent
outer observation are still required, even if a stored result says passed.

Clock comparison is confined to a direct parent and reader child on the same
Mac, using the same qualified CPython interpreter. The driver requires macOS
CPython >=3.10 and a finite, nonadjustable monotonic clock. It launches the reader
with `sys.executable` and checks the resolved interpreter, Python version,
platform and clock descriptor in the private receipt. The
[official Python 3.13 documentation](https://docs.python.org/3.13/library/time.html#time.monotonic)
states that the clock is shared across processes, including macOS since 3.10.
The requested local CPython 3.15 and M1 CPython 3.13.9 are separate qualification
targets; neither was run here. Descriptor agreement supports this fixed launch
topology; it is not a general clock-origin or cross-host attestation. Never
compare local Mac, M1, remote Windows Stopwatch or wall-clock values to each other.

The exact allowed saved argv was read as JSON data. Its prefix, before the
private selector and six native tokens, matches the new constant. The change
adds `ConnectionAttempts=1` and `ControlPath=none`, and changes keepalive 10/2 to
5/3. All other public options retain their order and values. There is no arbitrary
transport/options mode, selector change, config/default assumption or discovery.
The native six-token construction, both closed layer constructors and six file
paths remain unchanged. The historical native payload contains a different query.
This new instrumented operation has a different payload and whole argv. The
prefix match does not claim whole-argv equality, effective config, route coverage
or remote acceptance. Only public options and hashes enter this package; no
alias, endpoint, selector, raw command or private metadata values are copied.

R1's final work checks, D1's actual SIGPIPE/SIGKILL settlement, D2's outer gates
and D3's strict nonzero predicate are preserved. Stream caps, raw prefixes,
sticky first errors, real child exits, local-only settlement, privacy and
remote-state limits remain. Remote exit/current state always stays UNVERIFIED;
remote cleanup remains false. There is no forged terminal receipt.

[Bindings](bindings.json) retain exact immutable inputs and the complete prior
reports by hash. Those reports retain their own limits. The 179 historical Git
blob identities match; their rows are not copied here. The earlier 50 private
local files were not reopened or written. Their public audit/index is bound,
with no fresh private-byte claim. The later `3c52f53` report binds 58 latest files
and records a final `93cd2e6` four-case pass plus closed-output timeout behavior
for reader `8f55fbb7...`. Those are earlier-revision observations, not D4 proof.
The first failed driver flow remains FAILED/INCOMPLETE.

[Static checks](static-checks.json), [CLI requirements](CLI.md), the disabled
[finite plan](plan.json) and SHA256SUMS complete the handoff. No supplied code
or AST was executed or imported. No tests, mocks, fixtures, helper/native parser,
runtime, remote action, process/port query, build, product, credentials, private
write or child agent ran. Only the three explicitly allowed private JSON files
were read. No TypeScript changed; compiler/runtime checks are outside this scope.
The commit disables hooks and signing. All earlier paths and results are intact.

Windows native/parser/runtime/stop, installed versions, effective limits and
historical preservation remain UNVERIFIED. Old metadata stays
TIMED_OUT_UNVERIFIED; original FAILED/UNVERIFIED outcomes and zero native stop
requests remain. The old query and Sparkle windows stay closed. There is no
retry, recovery, release, signing, deployment or performance acceptance.
