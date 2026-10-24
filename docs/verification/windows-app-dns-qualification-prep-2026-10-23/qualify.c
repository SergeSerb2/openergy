/* SOURCE + NATIVE BUILD ONLY. Never executed for this preparation.
 * Settings-only candidate: no name argument, lookup, send, receive, or resolver.
 * Fixed proof gates below intentionally refuse before any DNS API or bind.
 * Changing a gate requires new source/safety review and separate authorization.
 */
#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0A00
#include <winsock2.h>
#include <ws2tcpip.h> /* Defines _WS2TCPIP_H_: exposes ServerAddr in SDK union. */
#include <windows.h>
#include <windns.h>
#include <iphlpapi.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#if !defined(_M_X64)
#error This source qualifies only the installed native MSVC x64 ABI.
#endif

typedef DWORD (__cdecl *SetSettings)(DWORD, const DNS_CUSTOM_SERVER *, const DNS_APPLICATION_SETTINGS *);
typedef DWORD (__cdecl *GetSettings)(DWORD *, DNS_CUSTOM_SERVER **, DNS_APPLICATION_SETTINGS *);
typedef VOID (__cdecl *FreeServers)(DWORD *, DNS_CUSTOM_SERVER **);
typedef BOOL (WINAPI *CheckVersion)(LPOSVERSIONINFOEXW, DWORD, DWORDLONG);

_Static_assert(sizeof(void *) == 8, "x64 pointers");
_Static_assert(DNS_APP_SETTINGS_VERSION1 == 1, "settings version");
_Static_assert(DNS_APP_SETTINGS_EXCLUSIVE_SERVERS == 1, "exclusive flag");
_Static_assert(DNS_CUSTOM_SERVER_TYPE_UDP == 1, "UDP server type");
_Static_assert(sizeof(DNS_APPLICATION_SETTINGS) == 16, "settings size");
_Static_assert(__alignof(DNS_APPLICATION_SETTINGS) == 8, "settings alignment");
_Static_assert(offsetof(DNS_APPLICATION_SETTINGS, Version) == 0, "version offset");
_Static_assert(offsetof(DNS_APPLICATION_SETTINGS, Flags) == 8, "flags offset");
_Static_assert(sizeof(((DNS_APPLICATION_SETTINGS *)0)->Flags) == 8, "flags width");
_Static_assert(sizeof(DNS_CUSTOM_SERVER) == 56, "server size");
_Static_assert(__alignof(DNS_CUSTOM_SERVER) == 8, "server alignment");
_Static_assert(offsetof(DNS_CUSTOM_SERVER, dwServerType) == 0, "type offset");
_Static_assert(offsetof(DNS_CUSTOM_SERVER, ullFlags) == 8, "server flags offset");
_Static_assert(offsetof(DNS_CUSTOM_SERVER, pwszTemplate) == 16, "template offset");
_Static_assert(offsetof(DNS_CUSTOM_SERVER, ServerAddr) == 24, "address offset");
_Static_assert(offsetof(DNS_CUSTOM_SERVER, MaxSa) == 24, "address storage offset");
_Static_assert(DNS_ADDR_MAX_SOCKADDR_LENGTH == 32, "address storage size");
_Static_assert(sizeof(SOCKADDR_INET) == 28, "SDK address union size");
_Static_assert(_Generic(&DnsSetApplicationSettings, SetSettings: 1, default: 0), "set ABI");
_Static_assert(_Generic(&DnsGetApplicationSettings, GetSettings: 1, default: 0), "get ABI");
_Static_assert(_Generic(&DnsFreeCustomServers, FreeServers: 1, default: 0), "free ABI");
_Static_assert(_Generic(&VerifyVersionInfoW, CheckVersion: 1, default: 0), "version API ABI");

/* NOT runtime options. The reviewed docs do not resolve these proof obligations.
 * No macro, environment value, file, or CLI flag can grant these proofs.
 * This is not a runnable qualification campaign or a permission receipt.
 */
static const struct {
    BOOL process_scope_and_exit_cleanup_proven;
    BOOL input_ownership_and_default_reset_proven;
    BOOL unchanged_host_no_external_or_multicast_path_proven;
    BOOL separately_authorized_runtime_revision;
} proof = { FALSE, FALSE, FALSE, FALSE };

typedef struct {
    HMODULE module;
    SetSettings set;
    GetSettings get;
    FreeServers release;
} Api;

static void receipt(const char *event, DWORD code)
{
    printf("{\"event\":\"%s\",\"code\":%lu,\"pid\":%lu,\"tick_ms\":%llu}\n",
           event, (unsigned long)code, (unsigned long)GetCurrentProcessId(),
           (unsigned long long)GetTickCount64());
    fflush(stdout);
}

/* Union conversion avoids incompatible-function-pointer casts. SDK _Generic
 * assertions above check every DNS function pointer type without linking it.
 */
static int qualify_api(Api *api)
{
    union { FARPROC raw; CheckVersion version; SetSettings set; GetSettings get; FreeServers release; } fn;
    HMODULE kernel = GetModuleHandleW(L"kernel32.dll");
    OSVERSIONINFOEXW version = {0};
    DWORDLONG conditions = 0;
    if (!kernel) { receipt("unverified_kernel32", GetLastError()); return 0; }
    fn.raw = GetProcAddress(kernel, "VerifyVersionInfoW");
    if (!fn.raw) { receipt("unverified_version_export", GetLastError()); return 0; }
    version.dwOSVersionInfoSize = sizeof(version);
    version.dwMajorVersion = 10;
    version.dwMinorVersion = 0;
    version.dwBuildNumber = 22000;
    conditions = VerSetConditionMask(conditions, VER_MAJORVERSION, VER_GREATER_EQUAL);
    conditions = VerSetConditionMask(conditions, VER_MINORVERSION, VER_GREATER_EQUAL);
    conditions = VerSetConditionMask(conditions, VER_BUILDNUMBER, VER_GREATER_EQUAL);
    conditions = VerSetConditionMask(conditions, VER_SERVICEPACKMAJOR, VER_GREATER_EQUAL);
    conditions = VerSetConditionMask(conditions, VER_SERVICEPACKMINOR, VER_GREATER_EQUAL);
    /* The embedded Windows-10 compatibility manifest is a build input. Without
     * it the version check may refuse; there is no alternate version path. */
    if (!fn.version(&version, VER_MAJORVERSION | VER_MINORVERSION | VER_BUILDNUMBER |
                    VER_SERVICEPACKMAJOR | VER_SERVICEPACKMINOR, conditions)) {
        receipt("unverified_build_22000_required", GetLastError()); return 0;
    }
    receipt("os_minimum_build_22000_verified", 0);
    api->module = LoadLibraryExW(L"dnsapi.dll", NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
    if (!api->module) { receipt("unverified_dnsapi_system32", GetLastError()); return 0; }
    fn.raw = GetProcAddress(api->module, "DnsSetApplicationSettings"); api->set = fn.set;
    fn.raw = GetProcAddress(api->module, "DnsGetApplicationSettings"); api->get = fn.get;
    fn.raw = GetProcAddress(api->module, "DnsFreeCustomServers"); api->release = fn.release;
    if (!api->set || !api->get || !api->release) {
        receipt("unverified_required_export_missing", ERROR_PROC_NOT_FOUND); return 0;
    }
    receipt("all_three_exports_present_not_called", 0);
    return 1;
}

/* No addresses, adapter names, suffixes or host policy enter the receipts.
 * This is only an adapter-DNS preflight, NOT proof of all host-policy routing.
 * GAA_FLAG_INCLUDE_ALL_COMPARTMENTS is reserved/unsupported; do not use it.
 */
static int global_dns_preflight(void)
{
    ULONG size = 16384, status = ERROR_BUFFER_OVERFLOW;
    IP_ADAPTER_ADDRESSES *buffer = NULL;
    unsigned int adapters = 0, servers = 0;
    int ok = 0;
    for (unsigned int attempt = 0; attempt < 2; ++attempt) {
        if (size > 1024 * 1024) { receipt("unverified_adapter_buffer_limit", size); goto done; }
        buffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size);
        if (!buffer) { receipt("unverified_adapter_alloc", ERROR_NOT_ENOUGH_MEMORY); goto done; }
        status = GetAdaptersAddresses(AF_UNSPEC, GAA_FLAG_INCLUDE_ALL_INTERFACES |
                   GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST |
                   GAA_FLAG_SKIP_FRIENDLY_NAME, NULL, buffer, &size);
        if (status != ERROR_BUFFER_OVERFLOW) break;
        HeapFree(GetProcessHeap(), 0, buffer); buffer = NULL;
    }
    if (status != NO_ERROR || !buffer) { receipt("unverified_adapter_enumeration", status); goto done; }
    for (IP_ADAPTER_ADDRESSES *a = buffer; a; a = a->Next) {
        if (++adapters > 1024) { receipt("unverified_adapter_count_limit", adapters); goto done; }
        for (IP_ADAPTER_DNS_SERVER_ADDRESS *d = a->FirstDnsServerAddress; d; d = d->Next) {
            const SOCKADDR *s = d->Address.lpSockaddr;
            if (++servers > 4096 || !s || d->Address.iSockaddrLength < (int)sizeof(SOCKADDR)) {
                receipt("unverified_dns_address_or_count", servers); goto done;
            }
            if (s->sa_family == AF_INET && d->Address.iSockaddrLength >= (int)sizeof(SOCKADDR_IN)) {
                ULONG addr = ntohl(((const SOCKADDR_IN *)s)->sin_addr.s_addr);
                if ((addr >> 24) == 127 || addr == 0) {
                    receipt("unverified_global_dns_loopback_or_unspecified", 0); goto done;
                }
            } else if (s->sa_family == AF_INET6 && d->Address.iSockaddrLength >= (int)sizeof(SOCKADDR_IN6)) {
                const IN6_ADDR *addr = &((const SOCKADDR_IN6 *)s)->sin6_addr;
                const UCHAR *b = addr->u.Byte;
                if (IN6_IS_ADDR_LOOPBACK(addr) || IN6_IS_ADDR_UNSPECIFIED(addr) ||
                    (IN6_IS_ADDR_V4MAPPED(addr) && (b[12] == 127 ||
                     (b[12] == 0 && b[13] == 0 && b[14] == 0 && b[15] == 0)))) {
                    receipt("unverified_global_dns_loopback_or_unspecified", 0); goto done;
                }
            } else { receipt("unverified_dns_address_family", s->sa_family); goto done; }
        }
    }
    if (!adapters || !servers) { receipt("unverified_no_adapter_dns_evidence", 0); goto done; }
    receipt("adapter_dns_preflight_only_no_confinement_claim", servers);
    ok = 1;
done:
    if (buffer) HeapFree(GetProcessHeap(), 0, buffer);
    return ok;
}

static SOCKET own_udp53(SOCKADDR_IN *address)
{
    SOCKET s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    BOOL exclusive = TRUE, got = FALSE;
    int length = sizeof(got), address_length = sizeof(*address);
    SOCKADDR_IN actual = {0};
    if (s == INVALID_SOCKET) { receipt("unverified_socket", (DWORD)WSAGetLastError()); return s; }
    if (!SetHandleInformation((HANDLE)s, HANDLE_FLAG_INHERIT, 0) ||
        setsockopt(s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, (const char *)&exclusive, sizeof(exclusive)) ||
        getsockopt(s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, (char *)&got, &length) || !got ||
        bind(s, (const SOCKADDR *)address, sizeof(*address)) ||
        getsockname(s, (SOCKADDR *)&actual, &address_length) ||
        address_length != (int)sizeof(actual) || actual.sin_family != AF_INET ||
        actual.sin_port != address->sin_port || actual.sin_addr.s_addr != address->sin_addr.s_addr) {
        receipt("unverified_exclusive_loopback_udp53", (DWORD)WSAGetLastError());
        closesocket(s); return INVALID_SOCKET;
    }
    receipt("owned_exclusive_127_0_0_1_udp53_retained_no_io", 0);
    return s;
}

static int free_readback(Api *api, DWORD *count, DNS_CUSTOM_SERVER **servers)
{
    api->release(count, servers);
    if (*count != 0 || *servers != NULL) { receipt("fail_free_did_not_clear_outputs", 0); return 0; }
    receipt("readback_array_freed_and_cleared", 0);
    return 1;
}

static int read_default(Api *api, DNS_APPLICATION_SETTINGS *settings, const char *event)
{
    DWORD count = 0, status;
    DNS_CUSTOM_SERVER *servers = NULL;
    *settings = (DNS_APPLICATION_SETTINGS){ DNS_APP_SETTINGS_VERSION1, 0 };
    status = api->get(&count, &servers, settings);
    receipt(event, status);
    int ok = status == ERROR_SUCCESS && count == 0 && servers == NULL &&
             settings->Version == DNS_APP_SETTINGS_VERSION1 && settings->Flags == 0;
    /* Free any returned array even on an unexpected error result. */
    if (servers && !free_readback(api, &count, &servers)) ok = 0;
    if (!ok) receipt("unverified_expected_default_process_settings", 0);
    return ok;
}

/* Source only. One set, one readback, one restore, one default readback.
 * This sequence must NEVER be used as evidence that process exit clears DNS
 * state. Pending proof includes API argument lifetime and zero-server reset.
 * No retry on set/readback/restore failure. Keep the owned socket until restore.
 */
static int configure_readback_restore(Api *api, const SOCKADDR_IN *address)
{
    DNS_APPLICATION_SETTINGS before, after;
    DNS_APPLICATION_SETTINGS desired = { DNS_APP_SETTINGS_VERSION1, DNS_APP_SETTINGS_EXCLUSIVE_SERVERS };
    DNS_CUSTOM_SERVER server = {0}, *returned = NULL;
    DNS_APPLICATION_SETTINGS readback = { DNS_APP_SETTINGS_VERSION1, 0 };
    DWORD count = 0, status;
    int ok, restored;
    if (!read_default(api, &before, "baseline_get_status")) return 2;
    server.dwServerType = DNS_CUSTOM_SERVER_TYPE_UDP;
    server.ullFlags = 0;
    server.pwszTemplate = NULL;
    server.ServerAddr.Ipv4 = *address;
    /* The first mutation is below. All immutable proof and runtime gates,
     * adapter preflights, export checks and exclusive bind precede this call. */
    status = api->set(1, &server, &desired);
    receipt("set_exclusive_status", status);
    ok = status == ERROR_SUCCESS;
    if (ok) {
        status = api->get(&count, &returned, &readback);
        receipt("configured_get_status", status);
        ok = status == ERROR_SUCCESS && count == 1 && returned &&
             readback.Version == desired.Version && readback.Flags == desired.Flags;
        if (ok) {
            const DNS_CUSTOM_SERVER *r = returned;
            ok = r->dwServerType == server.dwServerType && r->ullFlags == 0 && r->pwszTemplate == NULL &&
                 memcmp(r->MaxSa, server.MaxSa, DNS_ADDR_MAX_SOCKADDR_LENGTH) == 0;
        }
        receipt(ok ? "exact_exclusive_readback" : "fail_exclusive_readback", 0);
        if (returned && !free_readback(api, &count, &returned)) ok = 0;
    }
    /* Even a failed mutation attempt can require cleanup. Never assume atomic
     * failure. Do not pass NULL settings (that selects fallback). */
    status = api->set(0, NULL, &before);
    receipt("restore_zero_custom_servers_status", status);
    restored = read_default(api, &after, "restored_get_status");
    restored = restored && status == ERROR_SUCCESS;
    receipt(restored ? "default_restored_and_read_back" : "fail_restore_exit_cleanup_UNVERIFIED", 0);
    receipt("process_exit_cleanup_NOT_proven_by_this_command", 0);
    return ok && restored ? 0 : 3;
}

int main(int argc, char **argv)
{
    Api api = {0};
    WSADATA winsock;
    SOCKET owned = INVALID_SOCKET;
    SOCKADDR_IN address = {0};
    int result = 2;
    if (argc != 2 || strcmp(argv[1], "--settings-only") != 0) {
        fputs("SOURCE PREPARATION ONLY: sole candidate mode --settings-only; no lookup mode.\n", stderr);
        return 2;
    }
    receipt("source_preparation_no_runtime_authorization", 0);
    if (!proof.process_scope_and_exit_cleanup_proven || !proof.input_ownership_and_default_reset_proven ||
        !proof.unchanged_host_no_external_or_multicast_path_proven || !proof.separately_authorized_runtime_revision) {
        receipt("UNVERIFIED_STOP_before_DNS_APIs_or_socket_bind_missing_fixed_proofs", 0);
        return 2;
    }
    if (!SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS) ||
        GetPriorityClass(GetCurrentProcess()) != BELOW_NORMAL_PRIORITY_CLASS) {
        receipt("unverified_priority", GetLastError()); return 2;
    }
    if (!qualify_api(&api)) goto module_cleanup;
    if (WSAStartup(MAKEWORD(2, 2), &winsock) != 0) { receipt("unverified_winsock", 0); goto module_cleanup; }
    if (!global_dns_preflight()) goto socket_cleanup;
    address.sin_family = AF_INET;
    address.sin_addr.s_addr = htonl(0x7f000001UL);
    address.sin_port = htons(53);
    owned = own_udp53(&address);
    if (owned == INVALID_SOCKET) goto socket_cleanup;
    /* Recheck after binding to narrow a policy/configuration race. This does
     * not eliminate the race or qualify unrelated policy paths. */
    if (!global_dns_preflight()) goto socket_cleanup;
    result = configure_readback_restore(&api, &address);
socket_cleanup:
    if (owned != INVALID_SOCKET) {
        DWORD status = closesocket(owned) == 0 ? 0 : (DWORD)WSAGetLastError();
        receipt("owned_socket_close_status", status);
        if (status) result = 3;
    }
    {
        DWORD status = WSACleanup() == 0 ? 0 : (DWORD)WSAGetLastError();
        receipt("winsock_cleanup_status", status);
        if (status) result = 3;
    }
module_cleanup:
    if (api.module) {
        DWORD status = FreeLibrary(api.module) ? 0 : GetLastError();
        receipt("dnsapi_unload_status", status);
        if (status) result = 3;
    }
    receipt("settings_only_exit_no_lookup_attempt", (DWORD)result);
    return result;
}
