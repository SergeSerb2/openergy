# Metadata reader: terminal work deadline repair

Source only; execution=false. The reader is the exact `7875987` source plus
three lines in `capture`: check the pump deadline after its final exit poll,
record the observed capture-completion time, and check the work deadline after
the capture loop. The unchanged query is carried byte-for-byte alongside it.
The [patch](reader-deadline.patch) and [bindings](bindings.json) retain exact
inputs. Runtime of this revision remains UNVERIFIED.

This addresses reader R1 from independent review
[`f9aaf96`](../windows-metadata-reader-review-2026-10-25-9d61c036/REVIEW.md).
A late final poll retains the real child exit but records capture TimeoutError;
the larger settlement/finalization budget cannot make late work COMPLETE.
All byte caps, stream handling, exit collection, command shapes, remote-state
limits and explicit cooperative OS-call deadline limits remain unchanged.

The same review's driver D1 is addressed by the separately committed SIGPIPE
check correction. Driver D2 is addressed in the current
[full local CLI driver](../metadata-reader-local-cli-2026-10-25/verify-cli.py):
absolute case and whole-flow deadlines are checked after actual reader exit,
after evidence reads, before accepting a case, before marking the flow passed,
and before returning CLI success. Each case's external allowance consists of
the reader's declared total plus one explicit second for interpreter startup.
The whole flow remains bounded at 28 seconds, with at most two extra seconds
to reap an owned reader after outer timeout. An external observation of a late
exit must reject success even if an earlier file says passed.

The two earlier local flows remain immutable. They exercise full real local
commands and do not reproduce the terminal scheduling race. Their results
belong to the earlier exact reader/driver hashes. AST and hash checks passed
for this repair; fresh independent review and full CLI validation are pending.
No unit test, mock, remote connection, Windows query, native parser, build,
product, signing, old guard reset or historical attempt occurred here. No
TypeScript changed. Installed launcher limits and native stop stay UNVERIFIED.
