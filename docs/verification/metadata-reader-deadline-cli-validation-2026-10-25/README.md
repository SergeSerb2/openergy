# Real local CLI validation after the deadline repairs

The deadline-fixed reader `97cc8f9` ran through its complete local CLI. The
final driver `93cd2e6` returned0 and accepted success, ordinary child failure,
partial-output timeout and output-cap failure. Actual reader exits were0,1,1,1;
collected child exits were0,1,-9,-13. The nonzero case had no reader error,
complete EOF capture, no overflow/discarded bytes and no local kill.

An earlier run of the same reader with the `97cc8f9` driver also completed its
four cases. A separate real child closed both output streams and kept running;
the reader recorded capture TimeoutError, returned1, killed its retained child
and collected -9. Both EOFs were observed and zero bytes were captured. This
exercises closed-output liveness. It does not deterministically reproduce the
review's terminal scheduling race.

All invocations used new private directories and immutable source copies.
[results.json](results.json) binds the exact source hashes, actual exits,
errors and stream accounting. [The private index](private-evidence-index.json)
binds all retained files. All 50 files from the two earlier local flows were
rehashed and remain byte-exact; the first failed driver result remains failed.
Changes to the driver and reader justify these distinct revision checks. No
case was repeated within an invocation or after an unexpected result.

The reader source cleared independent review `ef9a635` with explicit limits.
The final nonzero driver correction is under fresh review. These local command
flows use real processes and raw pipes, without unit tests or mocked Windows.
There was no Windows connection, metadata query, product, M1 app flow, build,
signing or TypeScript change. Native PowerShell, installed launcher versions,
effective limits and native stop remain UNVERIFIED. OS-stall preemption,
nonempty stderr, competing caps, disk faults and native parser behavior have
no new proof here. A stored receipt alone is not acceptance without actual
CLI exit and the independently observed outer time.
