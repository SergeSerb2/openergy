# Real slow-DNS fixture: source and prerequisites only

Research date: 2026-10-23, America/Vancouver. Source reviewed:
`72e0233a1fe55b2c50fb231de2e0354877a0de22`.

**A documented application DNS configuration API exists. A qualified fixture
does not yet exist.** Windows offers the strongest candidate without replacing
the product resolver. Apple also documents an application setting that reaches
`getaddrinfo`, but its encrypted fallback has extra prerequisites. Therefore,
“ordinary per-process configuration is impossible” is not a supported finding.
Slow active lookup and resolver queue head-of-line (HOL) evidence remain open.

This task read source, saved reports and public primary documentation. It ran
no build, product, fixture, DNS query, network probe or runtime inventory. Public
documentation retrieval was the only network use. No unit tests, mocks, injected
results, source implementation, resolver interposition, delegation or publication
were used. Only this document changed. The supplied inventory says no local
VM/Lume/QEMU/container environment exists; that inventory was not repeated and
no environment was provisioned. This note grants no runtime permission.

## Current source contract

Pointers below are relative to the reviewed commit, not a later moving branch.

- [reactor.rs](../../crates/net/src/reactor.rs), lines 763–806 and 1217–1266:
  each hostname job has a monotonic private `id`, host, port, absolute deadline
  and shared cancellation flag. Numeric IPs bypass DNS. Both pending/connections
  and `dns_outstanding` limit admission to 16. Cancelling pending work does not
  free its outstanding DNS slot.
- The same file, lines 1277–1310: one lazy `openergy-dns` worker, channel capacity
  16, sequential `recv`, cancellation/deadline check, then the real
  `(host, port).to_socket_addrs()`. Each consumed job sends one completion and
  wakes the reactor. There is no interrupt for an active OS lookup. The comment
  is an explicit limitation, not evidence of a new regression.
- Lines 1165–1182: `Shutdown` and `DisconnectAll` cancel/remove pending requests
  and emit one `ConnectFailed { host, port, error: "shutdown" }` each. Shutdown
  also removes the listener. Both preserve a reusable driver/handle. Lines
  1881–1887: Drop closes the mailbox and flags pending jobs; it neither joins
  the DNS worker nor produces a caller event after destruction.
- Lines 1613–1667: maintenance decrements outstanding accounting before rejecting
  a stale result ID. A live result is checked against its original deadline;
  expiry removes pending state, flags cancellation and emits `connect_timeout`.
  Thus a late DNS result must not open a socket or emit a second public failure.
- Lines 1624–1632, 365–388, 1687–1689 and 1718–1754: returned address order and
  the original deadline carry through initial TCP, fallback and unfinished TLS.
  Pending deadlines cap poll wait. Lines 1761–1770 and 1877: maintenance runs on
  both sides of the socket poll. This does not make an unpolled caller punctual.
- [lib.rs](../../crates/net/src/lib.rs), lines 95–170: public Connect and
  ConnectFailed identify host/port, with no resolver ID. Existing reactor traces
  at 1410–1411 and 1820–1822 identify TCP attempts/readiness, not DNS entry/exit.
  No DNS job observer currently supplies internal start/end timestamps.

Rust's official pages identify the reviewed online std as **1.99.1**. Its
[tuple implementation](https://doc.rust-lang.org/src/std/net/socket_addr.rs.html#200-213)
uses the system hostname lookup; the
[socket implementation](https://doc.rust-lang.org/src/std/sys/net/connection/socket/mod.rs.html#349-359)
calls `getaddrinfo`. Changing the DNS server through an ordinary OS setting
can preserve this path. Calling a different resolver and passing its addresses
to NetDriver cannot exercise it.

## Evidence that stays open

- [R5](release-integration-2026-10-23/windows-dns-contract-review-r5.md), lines
  7–31: short OS fixture timing and caller deadlines are separate. A six-second
  observation cannot relax a five-second contract. Fallback needs actual attempt
  identity/order and one original deadline.
- [R11](release-integration-2026-10-23/dns-verifier-r11-integration.md), lines
  12–30: source/saved-evidence review, no independent runtime acceptance.
- [R12](release-integration-2026-10-23/windows-dns-final-8281eb7.md), lines 3–16:
  normal **94.304 s / exit 2**, conflict **93.533 s / exit 2**; normal completed
  eight 16-job batches, conflict skipped eight unsafe batches. Both qualified
  refusal matrices passed 10/10. All six deadline receipt brackets crossed the
  caller bounds. Strict deadlines and full acceptance remain unverified.
- [R13](release-integration-2026-10-23/windows-dns-final-review-r13.md), lines
  10–18: audit only, no added executions; slow DNS remains open. The later
  [historical refusal run](release-integration-2026-10-23/windows-dns-historical-refusal-615c16e1.md),
  lines 1–3, passed 10/10 and captured no old fault. It supplies no negative witness.
- The [current verifier](../../crates/net/examples/dns_queue_cancel_c4806a3d.rs),
  lines 1386–1411, uses fast localhost and one-nanosecond queued expiry; lines
  1659 and 1693 explicitly report slow lookup unavailable/not measured. Its
  [original evidence](../../crates/net/examples/dns_queue_cancel_c4806a3d-evidence/README.md),
  lines 64–95, retains both canonical exit-1 refusal failures, the excluded shared
  target comparison and absent slow-DNS proof. No result here revises those records.

## Normal public OS capabilities

**Windows: concrete candidate, not runtime-qualified.**
[DnsSetApplicationSettings](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnssetapplicationsettings)
accepts custom application DNS servers on Windows build 22000 or later.
[DNS_APPLICATION_SETTINGS](https://learn.microsoft.com/en-us/windows/win32/api/windns/ns-windns-dns_application_settings)
defines `DNS_APP_SETTINGS_VERSION1` and `DNS_APP_SETTINGS_EXCLUSIVE_SERVERS`
(`0x1`), which excludes system-configured DNS servers. Passing NULL settings
instead selects fallback and is unsuitable for confinement.

A future owned verifier could set this once **inside its own process, before
NetDriver construction or any lookup**, then call the unchanged NetDriver API.
[DNS_CUSTOM_SERVER](https://learn.microsoft.com/en-us/windows/win32/api/windnsdef/ns-windnsdef-dns_custom_server)
supports `DNS_CUSTOM_SERVER_TYPE_UDP`, a NULL template and `SOCKADDR_INET` storage.
Use one literal `127.0.0.1` server, flags zero, with the standard DNS port 53.
The docs do not establish arbitrary-port handling; do not assume an ephemeral
port works. Bind and retain an exclusive owned socket first. If it is unavailable
or needs a forbidden host change, stop. Do not take an existing listener.
[DnsGetApplicationSettings](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsgetapplicationsettings)
can read back the application setting; its allocated server array must be freed.

These are configuration APIs, not a replacement DNS implementation in the
product. **Inference needing qualification:** the DNS namespace provider used by
Rust's exact ANSI
[getaddrinfo](https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfo)
will honor this exclusive setting in the owned executable. The documentation
does not name Rust, guarantee wire queries on a cache hit, or exclude every other
name mechanism. The API success code/readback alone proves none of those things.
Fully qualified fresh ASCII names, an actual server question and a release-driven
successful NetDriver connection are mandatory witnesses. No single-label name,
`.local`, localhost, cache flush, NRPT or system DNS changes qualify.
Before any future query, establish that the chosen FQDN cannot invoke an external
or multicast naming path under unchanged host policy. Exclusive DNS-server
selection alone is not that proof. If documentation cannot establish confinement,
use the isolated-OS prerequisite; do not discover leakage by a live trial.

**macOS: public route exists; confinement and TLS trust are unresolved.**
Apple's [WWDC20 session, 10:17–12:07](https://developer.apple.com/videos/play/wwdc2020/10047/)
documents configuring `NWParameters.PrivacyContext.default` with
`requireEncryptedNameResolution(true, fallbackResolver: ...)`. It explicitly
applies to POSIX `getaddrinfo`, without replacing that function. Apple also says
system DNS configuration takes precedence over the supplied fallback.
[Resolver server addresses](https://developer.apple.com/documentation/network/nw_resolver_config_add_server_address(_:_:))
avoid a DNS bootstrap lookup. The
[TLS resolver endpoint](https://developer.apple.com/documentation/network/nw_resolver_config_create_tls(_:))
uses its hostname for certificate validation.

This does not document a plain-UDP per-process nameserver switch. An owned
loopback DoH/DoT server would need valid authentication under unchanged OS trust,
literal bootstrap addresses, proof that no system resolver overrides it, and
proof that trust/revocation handling causes no forbidden external traffic. No
credential was sought or read. Do not substitute HTTP, disable verification,
install a CA/profile/extension, or change TLS stores to fill these gaps. The
documented capability alone is insufficient for a confined Mac run today.

`DnsQueryEx` custom-server lists, `DNSServiceGetAddrInfo`, direct DNS tools, or
Network-framework connections used *instead of* the std lookup do not verify this
product path. Environment timeout/search options do not establish server routing.
No DYLD hooks, hosts files, global resolver changes, public blackholes or timing
load are candidate fixtures.

## Finite future fixture contract — no execution authorized here

Preferred prerequisite: qualify the Windows API in a uniquely owned executable
with unchanged production library bytes. Otherwise supply a separately prepared,
disposable OS environment with its own DNS configuration and loopback stack,
no external network path and no live sharing. The existing inventory supplies
neither that environment nor a qualified server. Do not provision one in this task.

The future DNS server must serve only a per-run finite manifest, for example
`j00.<case>.<nonce>.openergy.test.`. Use a fresh absolute name per job and never
reuse a name for an admission retry. Return only literal owned loopback addresses,
TTL zero, no delegation or recursion. Hold **all** address-family questions and
retransmissions for the selected head name until release. Answer each subsequent
manifest name promptly. Unexpected questions, cache/special-name bypass, any
off-loopback destination, early OS failure, missing witness or budget overflow
stop the case as unverified. A real authoritative server withholding its own
reply preserves the OS resolver; a sleep or injected completion in NetDriver does not.

Record a manifest mapping case/job label → unique name → unique owned TCP port,
plus driver/process identity and command-entry/exit brackets. The server records
question name/type, transaction ID, peer endpoint, receipt, hold, release and reply.
DNS packets carry no NetDriver job ID or destination TCP port. This mapping gives
external job identity, not an observed private `ResolveJob.id`. Lack of a wire
question alone cannot prove that `to_socket_addrs` was never called. If exact
resolver-call start/end counts are required, a documented passive OS observer
that identifies the owned process/thread and call is a further prerequisite;
no such observer was qualified here. Do not add a production control hook.

Use four cases, once each, with no rerun-to-green:

For each case submit only the head until its real question arrives, within
500 ms of the Connect command. Then submit followers. A missing or later question
does not qualify the case. The held interval starts at that question.

1. **Live HOL control:** head and one follower use 10 s deadlines. Require the
   head's actual question, hold for 3.5 s, and require no follower question or
   connection before release. After release, require both named jobs to resolve,
   connect to their owned ports and exchange exact bidirectional frames. Early
   failure or absence of release-driven success is no active-hold witness.
2. **DisconnectAll**, then **Shutdown**, as separate cases: admit one held head
   and 15 unique queued jobs with 10 s deadlines. The seventeenth request must
   fail with `16-connection limit`. After the real head question, issue the reset.
   Require exactly 16 identity-matched `shutdown` failures. Poll while the server
   still holds. A single fresh hostname request just before release must fail
   with `DNS request limit`, after completion maintenance has had chances to run.
   Require a numeric-loopback frame roundtrip during the hold. This checks that
   pending cancellation preserves active accounting while the reactor stays usable.
3. **Expiry:** admit the held head and 15 queued jobs with 1 s caller deadlines.
   Keep the reply held for the same 3.5 s. Require 16 exact typed timeout failures
   before release, no queued-name questions, and retained DNS admission accounting
   as above. An earlier `dns_failed` or legitimate refusal does not pass expiry.

After each reset/expiry release, drain through public polls for at most 2 s, then
require one fresh hostname recovery on the **same driver, original handle and
waker**, with exact frames. Close it normally and require completed closure. To
check full slot recovery, admit
16 new named jobs without intervening maintenance and require the seventeenth to
hit the pending limit; cancel this refill normally and drain it. Continue observing
for 5 s after final release/drain: no obsolete connection, extra failure, wrong
endpoint or lost fresh recovery. Hold sockets for
every obsolete TCP port throughout so a late attempt cannot reach an unowned peer.
Then close/drop the fixture normally and verify no owned process or endpoint
remains, within the cleanup budget. Preserve any failure even if cleanup succeeds.
Drop/worker-exit behavior remains source-only in this four-case plan.

Deadline oracle: bracket the actual command deadline in
`[command_enter + timeout, command_exit + timeout]`. Preserve every poll entry,
exit and returned event. Observe an empty poll before the earliest bound, then
start the first post-expiry poll after the latest bound while DNS remains held;
it must return the timeout in that iteration. No later poll may repair failure.
Overlapping receipt/deadline intervals remain unverified for strict punctual
delivery, as in R12. This deliberate post-expiry check proves maintenance behavior,
not an event timestamp at or before the exact deadline or TCP/TLS fallback timing.

Hard budgets for the whole future four-case campaign: at most **160 hostname
submissions**, **16 outstanding per driver**, **4 numeric controls**, **20 frame
roundtrips** with payloads at most 64 bytes, **256 DNS requests and 256 replies**
(each at most 512 bytes; at most 256 KiB combined DNS payload), **32 held questions**,
**36 simultaneous TCP listeners**, one DNS listener, **4 fixture processes**,
**4,096 recorded events / 4 MiB evidence**, and **120 s wall time including cleanup**.
Each head hold is at most 4 s; startup qualification is at most 5 s; each recovery
and final cleanup phase is at most 5 s. Poll waits are at most 10 ms with at most
12,000 calls total. Limits are stop conditions, not expected performance claims.
Uncontrollable OS traffic or retry behavior that cannot fit them is a missing
capability, not permission to raise limits or change host policy.
Missing preconditions are UNVERIFIED. A false behavior assertion with qualified
preconditions is FAIL. Preserve the first output, timing brackets and exit; neither
a later poll nor an audit supplies an independent rerun or changes that verdict.

Future preparation must use Rust 1.99.1, locked/offline inputs where possible,
jobs 2, macOS deployment target 15 if applicable, and measured effective nice 15
or Windows BelowNormal without nested nice increments. Configure private paths,
TLS policy and discovery off before startup; use the virtual platform if a daemon
is involved. No default IPC, TCP 24800, UDP 5353, live Mac/Windows sharing, installed
app, production config/TLS, TCC/login/service/account/firewall/DNS/toolchain default
may change. A bounded cancellation witness is not OS lookup cancellation or a
whole-process resource bound. Original release-provenance/security, crafted mDNS
expiry/NSEC/orphan, and IPC/Synergy3 budget tasks remain stopped and untouched.
