# One idle-then-poll TLS deadline workflow

Parent: `270106002490ef3e135d4c6500df1c8ff452b45b`.
One macOS ARM64 release run on 2026-10-23 at 17:14 PDT. **Overall FAIL,
exit 1**, at the final immediate cleanup bind. No runtime retry was made.
No NetDriver deadline bug was reproduced. No production library changed.

The new explicit `--idle-tls-deadline-only` mode uses the existing real
example. It does not run its default/full/refusal routines. Removing only
the added module declaration and option dispatch reproduces the parent's
example bytes exactly, including every existing predicate and maximum.

The owned raw TCP peer accepted the caller and received 218 real TLS bytes.
rustls `Acceptor` parsed a genuine ClientHello. It sent no TLS response and
kept the socket open. Caller connection `ConnId(2)` was still present with
no TLS version, fingerprint or Connected event. This proof finished
397.995750 ms before the earliest possible internal deadline.

The caller deliberately stopped polling. This idle is a caller workflow,
not product grace or an extension. The execute entry/return bracket was
35,826,208–36,050,958 ns from the recorded monotonic epoch. With the unchanged
500 ms budget, the deadline bracket was 535,826,208–536,050,958 ns. All 81
preparation polls returned before the conservative 285,826,208 ns cutoff.
No poll crossed expiry to establish this precondition.

The first poll after idle entered at 537,919,375 ns, 1.868417 ms after the
latest possible deadline. That same poll returned at 537,991,667 ns with
exactly one matching `connect_timeout:` / `Timeout` failure. The pending
connection was gone. **This first-eligible-poll check passed.**

The caller then made 78 finite late-event polls over at least 100 ms.
There were no late or duplicate events. The stalled client reached EOF.
The same caller, retained handle and waker then opened pinned TLS 1.3 to
the owned healthy NetDriver peer. Caller `ConnId(3)` sent exactly `PASS`;
peer `ConnId(2)` sent exactly `BACK`. Another finite 100 ms observation
followed the received frame. No caller recreation, reset, reconfiguration,
transport change or timeout change occurred before recovery completed.
**Same-caller/handle recovery passed.** Each driver then emitted exactly
one matching Shutdown disconnect. All eight public events, 335 API call
brackets, five traces, IDs, commands and received peer bytes are retained.

The process used exactly two outbound TCP connections and finished in
0.983154375 s under the external 15 s watchdog; the harness cap is 12 s.
It ran at verified nice 15, with Rust 1.99.1, jobs 2, cached offline/locked
dependencies and macOS deployment target 15.0. Scoped release Clippy with
warnings denied, release build and rustfmt check passed. No unit tests ran.

The final immediate exclusive bind to `127.0.0.1:25119` failed with Darwin
`EADDRINUSE` (48). The bind to healthy port 25118 passed. The original FAIL
is retained without changing its predicate or substituting later evidence.
Both drivers, handles, waker, accepted client and listeners were dropped;
the process exited and its fresh private config and identities were removed.
A separate read-only postflight found no matching lsof socket rows and no
LISTEN/ESTABLISHED rows for either fixture port. netstat showed two client
TIME_WAIT rows. lsof also reported an SMB stat warning; its private mount
path is redacted in the committed copy. The cause of the immediate rebind
failure is unresolved. This is not evidence of a NetDriver deadline defect,
and the entire case is not a pass.

Continuous strict TCP/TLS timing, original-deadline fallback, Windows
acceptance, historical negative controls, DNS/cache expiry, and all original
security/provenance and release gates remain unchanged and UNVERIFIED.
The historical task was not retried. No discovery, production ports, native
backend, service, global config, installer or provider-stopped work ran.

[Raw run](raw/runtime.stdout), [execution and cleanup record](raw/runtime.json),
[derived findings](derived.json), [source bindings](source.json),
[copy ledger](copy-ledger.json), and [single-run controller](run-once.py)
retain the evidence. The controller's exclusive guard prevents rerunning
this recorded case. The source and binary hashes refer to the exact code
that ran; no source change followed the run.
