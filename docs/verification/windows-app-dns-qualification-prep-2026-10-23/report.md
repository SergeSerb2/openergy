# First native build result

**SOURCE + NATIVE COMPILE PASS. Runtime and slow-DNS acceptance UNVERIFIED.**

Preparation base: `78879ccb603ae9d4019a0aa7384cd9e6a22192d6`.
Research read: `097e99839bf1dad74b985de9370ecdf9ea136333`, the named
slow-DNS prerequisites note. The preparation modifies only new files under this
directory. It does not alter the product resolver/library or any earlier report.

On 2026-10-23 at 18:11 America/Vancouver, one fresh private source-transfer/build
workflow completed. The first MSVC x64 C17 compilation passed `/W4 /WX` with no
warning or error output. The linker passed and embedded the compatibility
manifest. All SDK type/signature/layout assertions passed. No retry occurred.
Native build work took **951 ms**; its SSH transaction took **2.576254042 s**.
The Windows timestamps are preserved as returned, independently of the local
transport clock; no cross-host clock synchronization is claimed.

The existing inputs were MSVC `14.44.35207` and SDK `10.0.26100.0`. No Rust build
was needed for this standalone C command. There was no package/cache access,
dependency install, SDK/toolchain/default change or network package fetch.
The new directory was:

`C:\Users\serge\AppData\Local\Temp\oe-appdns-prep-20261023-1ac50255-a447cb28`

Its ACL was protected with one current-user-only rule. Only this fixture root
received source/build writes. No other Windows root/cache was selected. Artifact
files remain private there, unexecuted. No binary, object, archive or credential
was copied into this repository.

Source SHA-256 (`qualify.c`):
`9d99aba050cec623cd0cf15be4d23b1ca093f4e0d7da21408cee67b8ed8c9845`

Native artifact: `qualify.exe`, 506880 bytes, SHA-256:
`7f27ee5675ee3b3fe4cbd6579c5efff86a55ca55a9afe60bbc4dec91627b2459`

Object: 71859 bytes, SHA-256:
`c742bc5c2d7e6a132df1274a9a49c8b5608148a8a12c7b068df993e221e22cc4`

`source-manifest.json` binds all frozen command/controller/document sources and
the preflight's native prerequisite hashes. The build verified transferred
source and pinned prerequisites. Tracked source/tool/header/DLL/library hashes
matched before and after. This is a defined input subset, not a hash audit of
every transitive compiler/header/static-library input or the host filesystem.
`evidence-manifest.json` binds the returned reports and build output.
The evidence directory's Git attribute preserves original Windows line endings
so that committed output bytes retain their recorded hashes.

The controller observed effective local nice 15. Native PowerShell parent PID
4816 was in noninteractive session 0 at BelowNormal. The exact direct tool
receipts were compiler PID 47404, linker PID 8248, import inspector PID 5724 and
export inspector PID 35100. Each reported BelowNormal, exit 0, no timeout and
an exited Process handle. Jobs were **1, within the limit of 2**. No process was
killed. The native build PowerShell returned and the SSH process exited 0.
There was no process-name cleanup or unrelated process scan. These receipts
prove the observed direct handles exited, not the lifetime of any unseen short-
lived tool descendant. No helper/server process was started.

Static PE inspection of the installed System32 DNS DLL found all three exports:
`DnsFreeCustomServers`, `DnsGetApplicationSettings`, `DnsSetApplicationSettings`.
It did not load/call those functions. SDK declarations plus a DLL export do not
prove process scope, ownership, lifecycle, supported running OS version or
application behavior.

The generated PE's named imports have no name-lookup or DNS packet-I/O function.
Its Winsock imports are shown **by ordinal**, so the automated name-only import
check does not resolve or independently exclude every ordinal. Frozen source
review supplies the no-lookup/no-send/no-receive finding. No claim of full PE
call-graph verification or resolved-ordinal import proof is made. The build's
PASS means the native compile/link and recorded static checks passed.

There were **zero generated-executable runs**, including `--help`; **zero DNS
settings/query API calls, DNS socket binds, product runs, tests or mocks**. Only
public documentation retrieval, an existing private SSH transport, installed
file reads, native compile/link and static PE inspection occurred. No DNS
configuration/query commands, service/account/registry/firewall change, desktop
driving, Docker, delegation, push, publication or Cloudflare action occurred.

All four fixed proof gates are false. There is no runtime argument to override
them. This exact source stops before the DNS APIs, adapter preflight or bind if
ever invoked. That statement is a source finding, not an executed refusal test.
Process-only/application identity, child inheritance, post-exit persistence,
input ownership and zero-server reset remain undocumented in the reviewed pages.
Thus no one-process DNS mutation or process-exit cleanup claim is qualified.

The [preparation contract](README.md) lists the finite future settings/readback/
free/restore sequence and the exact source-versus-runtime prerequisites before
any real DNS request. The current helper cannot issue one. The reviewed sources
do not prove that EXCLUSIVE confines Rust 1.99.1 `getaddrinfo` or excludes
LLMNR/NetBIOS/cache/NRPT/suffix/other-provider paths under unchanged host policy.
The actual query path stays **UNVERIFIED and requires an isolated OS environment
if that missing no-external/no-multicast source proof cannot be supplied**.
No such environment was provisioned.

The root must obtain source/safety review before any future helper call.
This report and the research note provide no runtime permission. No old failure
was rerun or repaired by this compile. Slow DNS, HOL, strict deadline evidence
and all three stopped tasks keep their previous status.
