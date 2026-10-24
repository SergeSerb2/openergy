# Windows input recovery — 2026-10-24

The exact previously used transport selector is recovered as private data. No
connection or native command ran. This is not a native build or stop result.

An independent data reader checked the published R6 setup script against its
historical SHA-256, then matched the exact historical command signature against
54 existing public Host/HostName/User metadata variants. Exactly one matched.
No private key, credential, known-host key or credential configuration was read.
The selector value remains in a mode-0600 JSON file under a new mode-0700 durable
verification root. No selector, endpoint or raw config is committed.

Bindings:

- Historical command signature: `771d4cc962030be4b74bf02659834163f8803710cda6b5ca48605b2d46184725`.
- Exact setup script: `6ff7f46984ccf26d11fe2eb344d21d7496cb6a5ddb21c9b5aefe87e1cc45e7bd`.
- Current private selection record: `5bcd9394e469759e0b66f6caf3563f46a3d4a6ef32808a675ec121fca35d15e9`.
- The surviving old private-location metadata still points to an absent root.
  Its existence does not recover the old binary, raw receipt, state or archive.

The first local serialization had a literal final escape instead of a newline.
It was corrected before any consumer or command used the record; JSON parsing
then passed. This changed only the newly owned private data file. The first
serialization hash was `73db99caf68877693c0adbcc02d5bf76af15f6519850a66b25e5226d6c5a99e8`.
No prior evidence or private originals were changed.

The nine exact historical fixture root bindings remain missing. Current host
state and retained payloads are not inferred from old hashes or transcribed
observations. Historical activity was read as data only. The root continuation
record retains the investigated thread/item boundaries; no task command was
replayed. The configured transport was not contacted.

The independent R6 source review found two P2 issues in the fresh-build adapter:
rooted fixture paths can escape TEMP; terminating build errors can bypass owned
compiler recovery. Both are assigned for source repair. This transport recovery
does not clear either finding or qualify a native stage. The old native Off stop
result remains **FAILED / UNVERIFIED**.

