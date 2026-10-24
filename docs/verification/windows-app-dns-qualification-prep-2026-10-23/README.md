# Windows application-DNS command: source and native build only

Prepared 2026-10-23, America/Vancouver. This directory belongs only to this
preparation. Product/library source, production state, other worktrees and other
Windows build roots/caches are outside its ownership.

**Runtime qualification is UNVERIFIED. No executable run is authorized here.**
The standalone `qualify.c` has one candidate argument, `--settings-only`. It has
no name argument, lookup mode, server loop, packet I/O, replacement resolver,
interposition, or product dependency. Four fixed false source gates stop before
any DNS settings API, adapter-DNS inventory or socket bind. There is no CLI,
environment or manifest override for a proof gate. This preserves an explicit
limitation rather than filling missing evidence with an acknowledgement flag.

The research input is commit `097e998`, file
`docs/verification/slow-dns-fixture-prerequisites-2026-10-23-a3f15192.md`.
It is absent from this child's base checkout, so it was read with `git show`.
That note authorizes no runtime. Its original failures, deadline brackets,
missing slow lookup/HOL proof and stopped tasks remain unchanged. Saved evidence
was read only to locate the existing SSH/native toolchain. None of its old
workflows was run. Auditing it adds no independent execution.

## Primary documentation and ABI evidence

The public Microsoft pages below were read on 2026-10-23. The native read-only
preflight preserved numbered installed-header extracts in
`evidence/inspect-sdk.stdout`, plus input hashes. No DNS API was called.

- [DnsSetApplicationSettings](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnssetapplicationsettings)
  documents application custom servers and minimum Windows build 22000. A NULL
  settings pointer selects fallback. This source never uses that form.
- [DNS_APPLICATION_SETTINGS](https://learn.microsoft.com/en-us/windows/win32/api/windns/ns-windns-dns_application_settings)
  specifies version 1 and `DNS_APP_SETTINGS_EXCLUSIVE_SERVERS = 0x1` to exclude
  system-configured DNS servers. Its syntax uses `ULONG64 Flags`, although the
  member prose labels it ULONG. Installed SDK `WinDNS.h:851–879` confirms the
  64-bit field and all three function declarations, without `WINAPI`.
- [DNS_CUSTOM_SERVER](https://learn.microsoft.com/en-us/windows/win32/api/windnsdef/ns-windnsdef-dns_custom_server)
  documents UDP type 1, NULL UDP template and `SOCKADDR_INET` address storage.
  Include `ws2tcpip.h` before `windns.h` to expose `ServerAddr` in this installed
  SDK. `windnsdef.h:1624–1673` supplies the union and 32-byte backing storage.
- [DnsGetApplicationSettings](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsgetapplicationsettings)
  returns server count/array and settings. No configured servers means zero and
  NULL. Its remarks misidentify the array parameter as `pSettings`; the signature
  and installed SDK identify `ppDefaultServers` as the server-array output.
- [DnsFreeCustomServers](https://learn.microsoft.com/en-us/windows/win32/api/windns/nf-windns-dnsfreecustomservers)
  takes addresses of that count and array, returns VOID, and clears both outputs.
  The source checks those postconditions. It never uses a generic heap free for
  this allocation.

Compile-time assertions check the real installed declarations, not copied DNS
structs: native MSVC x64 pointers 8 bytes; settings size/alignment 16/8, Version
offset 0, Flags offset 8 and width 8; server size/alignment 56/8, type/flags/template/
address offsets 0/8/16/24; address storage 32, `SOCKADDR_INET` 28. `_Generic`
checks exact DNS function pointer signatures against the SDK, including the
absence of `WINAPI` on these declarations. These assertions are ABI constraints
in the command source, not unit tests or mocks. Only x64 is supported.

The future source resolves all three DNS exports from System32 `dnsapi.dll`.
Missing functions stop the command; no alternate DNS path exists. Static export
inspection, if built, is separate from dynamically loading or calling the DLL.

Version qualification uses the documented
[VerifyVersionInfoW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-verifyversioninfow)
and a build-22000 minimum. It checks the major/minor, build and both service pack
members. This API is deprecated but remains documented for desktop apps. Its
manifest dependence is explicit. The supplied compatibility manifest has the
[Windows 10/11 supportedOS GUID](https://learn.microsoft.com/en-us/windows/win32/sysinfo/targeting-your-application-at-windows-8-1).
The native linker embeds it. Missing/false version evidence fails closed; there
is no ntdll version-API fallback or guessed running OS build.

## Scope and lifetime: missing source proof stops mutation

Microsoft calls these settings per-application. The reviewed pages do not define
whether application identity is strictly the calling process, whether children
inherit it, whether concurrent instances share it, how long the setter retains
its input, whether state persists after exit/crash, or an explicit reset-to-default
operation/atomic-failure contract. The zero-server setter form is syntactically
documented. Treating it as a semantic reset or proof of exit cleanup needs more
evidence. Bounded searches of Microsoft Learn for this setter with `process`,
`lifetime`, and `reset` returned no results; this is a research limit, not proof
that no other documentation exists.

No process-only or exit-cleanup guarantee is claimed. Fixed gates require source
proof of process scope/exit cleanup, input ownership/default reset, unchanged-host
confinement, and separate runtime authorization. All remain false. Until those
are resolved, even a settings-only mutation must STOP. A new source revision and
safety review are required before opening them. This task does not grant that
review or authorization.

The dormant sequence is concrete for that future review: qualify version and all
exports; initialize Winsock; reject loopback/unspecified adapter DNS; bind and
retain only literal `127.0.0.1:53/UDP` with exclusive use; repeat the preflight;
read strict default application settings (zero servers, NULL array, version 1,
zero flags); set one UDP server with exactly EXCLUSIVE; read exactly one matching
server, template/flags/address bytes and settings; free any returned array; restore
zero custom servers with the captured explicit settings; read strict defaults;
close only the owned socket, clean Winsock and unload the DLL. Preserve every
status, process identity and tick receipt. A failed set also triggers one cleanup
attempt, since failure atomicity is unknown. There is no mutation retry. Neither
restore success nor process exit is inferred from API success alone. Restore
failure is exit 3 and an explicit UNVERIFIED cleanup receipt, never PASS.

This dormant reset sequence remains blocked by the missing contract above.
Stack inputs stay alive through all calls and readback/restore. That is not proof
that the API does not retain them beyond the function or process lifetime.

[GetAdaptersAddresses](https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getadaptersaddresses)
enumeration includes all NDIS interfaces in the current context and examines
both families, including mapped loopback. It uses at most two allocations/calls,
1 MiB, 1024 adapters and 4096 server entries. Missing, empty, unknown-family or
loopback/unspecified entries stop. It emits counts/status only, no host addresses
or names. The all-compartments flag is documented as unsupported. Thus this is
an adapter-DNS preflight, not a complete policy inventory or proof against a
configuration race. Rechecking narrows a race but cannot remove it.

[SO_EXCLUSIVEADDRUSE](https://learn.microsoft.com/en-us/windows/win32/winsock/so-exclusiveaddruse)
must precede bind. The source sets and reads back that option, then checks the
literal local endpoint. It never sets REUSEADDR or binds a wildcard/other port.
An unavailable bind stops; no existing listener is taken or stopped. This first
command keeps a socket reservation only. It does not receive or answer packets.
Unknown host-wide DNS consumers or live policy remain a hard blocker even if
the literal bind would succeed.

## Finite contract before any future real DNS request

There is no query implementation in this command. A future query revision needs
both source proof and separate confined runtime qualification. The Microsoft
[ANSI getaddrinfo documentation](https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfo)
says the DNS namespace can aggregate providers and use a hosts file or other
name mechanisms. The EXCLUSIVE description does not establish that Rust 1.99.1
`std::net`/`getaddrinfo` honors it, or rule out LLMNR, NetBIOS, multicast, NRPT,
suffix search, caches, hosts/special-name bypass, encrypted DNS or service-mediated
activity. A successful settings readback is not query-confinement evidence.

Before a real request, the owner must prove all of the following without a trial
that could leak a name:

1. No existing adapter, policy, service or unrelated process can direct traffic
   to the claimed loopback DNS endpoint. Read-only adapter enumeration alone is
   insufficient. Policy inventory must be complete and unchanged for the case.
2. The one owned process exclusively retains literal UDP 53. An owned finite
   authoritative server and an exact fresh absolute ASCII FQDN/type manifest are
   ready before any lookup. First settings qualification has **zero DNS requests**.
   A separately authorized path qualification may use at most two fresh names
   `q00.<nonce>.openergy.test.` and `q01.<nonce>.openergy.test.` (A/AAAA only),
   16 questions and 16 responses, 512 bytes each, 16 KiB combined, one process,
   one DNS socket, one literal TCP endpoint and 10 seconds including cleanup.
   No recursion, delegation, CNAME, search variants, single-label or `.local` name,
   public blackhole, cache flush or OS policy change is allowed. Unexpected traffic
   or budgets stop UNVERIFIED. These are proposed future limits, not permission.
3. Source evidence establishes the exact Rust 1.99.1 ANSI `getaddrinfo` path and
   application scope for every worker thread. Runtime then needs process/thread,
   actual question/name/type/transaction/peer and owned-socket witnesses. A release-
   driven connection to the owned literal endpoint and exact frame exchange must
   succeed through unchanged std/NetDriver, with no alternate resolver. Cache hits
   or no question do not qualify it. Do not assume absent questions mean no call.
4. Read and record default process settings before mutation, exact exclusive
   settings after mutation, then exact restored defaults. Free every allocated
   readback. An independent source-backed lifecycle/parent receipt must establish
   no state escapes process exit, including failure/crash and owned-process cleanup.
5. Source proof must exclude all external/multicast fallback for those exact names
   and types under unchanged host policy **before** runtime. Passive observations
   can then confirm confinement but cannot retrospectively authorize a leaking
   trial. If that proof is unavailable, the query path stays UNVERIFIED and requires
   a separately prepared isolated OS with no external network path or live sharing.
   Do not provision it, change the shared host, or substitute a direct resolver.

Only after path qualification could the research note's four slow-DNS cases be
considered, with their original finite budgets and deadline oracle. This first
command proves no slow lookup, queue HOL behavior, deadline, cancellation,
resource bound or product acceptance.

## Build-only workflow and restrictions

`controller.py inspect-sdk` is one read-only remote SDK/toolchain inspection.
`controller.py build-only` is one source-transfer/build workflow with create-new
guards. It transfers only fixed source/manifest files through existing private
SSH, pins their hashes and the inspected prerequisite hashes, and creates a new
current-user-only Temp directory. It never exports the repository or copies a
cache. The native compiler, linker and static PE inspector use installed inputs.
Their process-local INCLUDE/LIB/PATH/TEMP do not change defaults. Compilation is
MSVC x64 C17, `/W4 /WX /Od /Z7 /MT /guard:cf`; jobs 1 (limit 2), no package manager,
network fetch, tool installation, Rust build, global dependency, or test runner.
Rust 1.99.1 remains the future product requirement; it is not used by this C CLI.
There is no macOS build, so the deployment-15 requirement is not applicable here.

The controller observes effective nice 15 without nested increments. Native
preflight/build parents and direct tool processes must report BelowNormal. Each
native tool has an 8-second limit; the build plan has a 40-second budget and the
SSH invocation a 55-second limit. Timeout/failure is preserved with no retry.
Only returned Process handles/tree are eligible for termination on failure;
there is no process-name kill or unrelated-process inspection. Direct tool exit
receipts are not proof of every possible unseen descendant's lifetime.

The generated executable must never run here, including `--help`. The build
performs static imports and installed DLL export inspection only. Its hashes and
first output belong in the evidence directory; binaries, objects, archives and
credentials do not belong in this repository. See `report.md` for the actual
result. The helper's future `--settings-only` argument above describes source; it
is not a request or instruction to execute it.

No DNS settings/query API, DNS bind, OS configuration change, live Mac/Windows
sharing, installed app, production config/TLS, default IPC, TCP 24800, UDP 5353,
TCC/login/service/account/firewall/DNS/toolchain default, Docker, desktop driving,
unit test, mock, delegation, push, publication or Cloudflare change is allowed.
Original release-provenance/security, crafted mDNS expiry/NSEC/orphan and
IPC/Synergy3 budget work remain stopped and are not resumed by this preparation.
