# Windows application-DNS source and saved-build safety review

Reviewed 2026-10-23, America/Vancouver. Target:
`2a7756f00502d6ca582c373012838eff2aa648ab`, directory
`docs/verification/windows-app-dns-qualification-prep-2026-10-23/`.
All source line references below refer to that commit. The target is outside
this child's base checkout; it was read through Git objects without checkout.

**No concrete P1/P2 finding within this source and saved-build scope. All four
proof gates remain CLOSED. This review grants no helper execution or DNS mutation.**
Only this review document changed. Review work used local source/evidence reads,
byte/hash comparisons at observed effective nice 15, and public Microsoft pages.
No target script/helper, build, remote command, DNS API, DNS bind, query, product,
test or mock ran. No key/configuration read, desktop driving or further delegation
occurred. Audited native receipts add no independent native execution.

- **Entry and ABI:** `qualify.c:6–59,259–279` uses the installed headers, x64-only
  layout/signature assertions and four literal false `static const` gates.
  The sole candidate argument is `--settings-only`; other arguments return 2.
  The valid argument also returns 2 before priority changes, DLL loading,
  adapter inventory, Winsock initialization, bind or settings calls. There is no
  lookup mode or environment/file/CLI gate override. The controller exposes only
  SDK inspection and build actions; its native process launches are compiler,
  linker and static inspectors. It never launches `qualify.exe`.
- **Dormant API/state path:** `qualify.c:79–114,189–257` checks the manifest-aware
  build-22000 minimum, resolves all three exports from System32, and refuses a
  missing export. Saved SDK declarations support `ULONG64 Flags`, the stated
  layout and the DNS `__cdecl` pointer declarations; `VerifyVersionInfoW` uses
  `WINAPI`. This is native x64 evidence only. Returned arrays use
  `DnsFreeCustomServers`, with cleared count/pointer checks. The sequence requires
  strict baseline defaults, one EXCLUSIVE UDP server, exact readback, one explicit
  zero-server restore attempt even after a failed set, then default readback.
  Restore failure returns 3. NULL settings fallback is never selected. These
  are source paths, not evidence that setting, freeing or reset worked.
- **Reservation/preflight concept:** `qualify.c:120–189,277–311` caps each adapter
  inventory at two calls/allocations, 1 MiB per allocation, 1024 adapters and 4096
  DNS entries. It rejects unknown, empty, loopback and unspecified DNS evidence,
  including mapped IPv4. It sets/reads EXCLUSIVEADDRUSE before the sole literal
  `127.0.0.1:53/UDP` bind, verifies the endpoint and repeats inventory. There is
  no wildcard bind, reuse, DNS send/receive loop or query implementation.
  Socket reservation and adapter inventory cannot prove absence of unrelated
  consumers, other policy paths or races. README lines 107–166 retain those blocks.
- **Exact saved bindings:** all 7 `source-manifest.json` entries and all 26
  `evidence-manifest.json` entries match committed sizes and SHA-256, including
  Windows output line endings. Both transport script/stdout/stderr hashes match.
  Reconstructed transfer-input bytes match the recorded hash; all 7 embedded
  returned evidence files match their extracted files and transfer summary.
  The six pinned prerequisites match SDK inspection. All 12 recorded build-input
  hashes match before/after; the three transferred build sources match Git bytes.
  This checks the defined input subset, not every transitive build input.
- **Artifact and first build:** saved source SHA-256 is
  `9d99aba050cec623cd0cf15be4d23b1ca093f4e0d7da21408cee67b8ed8c9845`.
  The recorded 506880-byte executable SHA-256 is
  `7f27ee5675ee3b3fe4cbd6579c5efff86a55ca55a9afe60bbc4dec91627b2459`;
  the 71859-byte object SHA-256 is
  `c742bc5c2d7e6a132df1274a9a49c8b5608148a8a12c7b068df993e221e22cc4`.
  Artifact bytes are not in Git and were not independently retrieved/rehashed.
  Saved MSVC 14.44.35207 / SDK 10.0.26100.0 receipts show first C17 `/W4 /WX`
  compile/link success and passed SDK assertions. Build elapsed 951 ms; transport
  elapsed 2.576254042 s. Each transport records one attempt, exit 0, no timeout
  and local nice 15. The records show no retry or build failure. Zero-run/API/bind
  fields are declarations, not independent runtime instrumentation.
- **Priority/ownership/cleanup:** native parent PID 4816 records BelowNormal and
  session 0; direct compile/link/import/export PIDs 47404/8248/5724/35100 each
  record BelowNormal, exit 0, no timeout and an exited handle. Jobs 1 stays within
  limit 2. No kill is recorded. Saved ACL receipts and fixed transfer paths bind
  writes to the fresh `oe-appdns-prep-20261023-1ac50255-a447cb28` Temp root;
  its unexecuted artifacts remain there. Direct handle exits and transport return
  do not prove every unseen descendant's lifetime. The 8/40/55-second tool/build/
  transport values describe the build plan; this successful record does not
  qualify timeout cleanup or universal wall-time bounds. Static Winsock imports
  remain ordinal-only; the report correctly avoids a complete PE call-graph claim.

The Microsoft [setter](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnssetapplicationsettings),
[getter](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsgetapplicationsettings),
[free function](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsfreecustomservers)
and [settings structure](https://learn.microsoft.com/en-us/windows/win32/api/windns/ns-windns-dns_application_settings)
support the parameter and allocation handling above. These pages do not establish
strict process identity, child/concurrent-instance scope, retained-input lifetime,
failure atomicity, zero-server reset semantics or post-exit/crash cleanup.
EXCLUSIVE success/readback also does not prove Rust 1.99.1 `ToSocketAddrs`
confinement; Microsoft's [getaddrinfo contract](https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfo)
allows namespace-provider aggregation. Lifecycle/reset and unchanged-host
fallback/cache/LLMNR/NetBIOS confinement remain UNVERIFIED.

README lines 124–166 give a future receipt plan: source proof and separate
authorization first; zero queries for initial settings qualification; only then
at most two fresh absolute names, 16 questions/16 responses, 512 bytes each,
16 KiB combined, one process/DNS socket/TCP endpoint and 10 seconds including
cleanup. Required witnesses include process/thread identity, actual owned-socket
questions/peers, exact unchanged-std/NetDriver frame success, default/exclusive/
restored readbacks and an independent source-backed exit/crash lifecycle receipt.
These are proposed limits, not an implemented or qualified server/watchdog.
Neither a cleanup attempt nor readback alone proves restored defaults. Missing
confinement proof keeps the query path closed pending a separately prepared
isolated OS; this review prepares no such environment.

Research `097e998:61–83` retains normal 94.304 s/exit 2, conflict 93.533 s/exit 2,
all six crossing deadline brackets, and both original exit-1 refusal failures.
This compile/audit changes none of them. Actual application-DNS setting, OS lookup,
slow lookup/HOL and strict deadline acceptance remain open. The three specified
provider-stopped tasks remain untouched. No live sharing, installed app,
production config/TLS/default IPC or host/toolchain default changed.
