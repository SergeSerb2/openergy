# Source and primary API rationale

The original `os_evidence` started `lsregister -dump`, captured the whole shared
database, then filtered its text. Filtering after capture cannot limit the
native command's work or make its whole database output arrive within five
seconds. The saved timeout occurred before product startup. Raising that budget,
accepting a timeout, or repeating the scan would not be this narrow repair.

## Public ABI and ownership

Apple documents
[LSCopyApplicationURLsForBundleIdentifier](https://developer.apple.com/documentation/coreservices/1449290-lscopyapplicationurlsforbundleid?language=objc)
as taking one non-null `CFStringRef` and an optional `CFErrorRef *`, and returning
a nullable `CFArrayRef` of matching application URLs. The caller must release
the array and any returned error. The API is deprecated, with a documented
NSWorkspace replacement. It remains declared in the inspected SDK. This repair
uses that narrow C interface as requested; it has no Objective-C dispatch,
desktop automation, compilation dependency or fallback. Availability and ABI
were reviewed as source, not tested by loading the framework.

[api/sdk-evidence.json](api/sdk-evidence.json) freezes file sizes, SHA-256 hashes,
line ranges and relevant text from seven local Apple SDK headers and two text
linker stubs, all read as
data under `/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/`:

- `LSInfo.h:135-162` declares the actual two-argument function and its null/error
  semantics. `LSConstants.h:28-40` fixes `kLSApplicationNotFoundErr` to `-10814`.
- `CFBase.h` declares CF references as pointers, macOS LP64 `CFIndex` as signed
  long, `CFTypeID` as unsigned long, Boolean as unsigned char, and `CFRange` as
  two `CFIndex` fields. The helper therefore uses `c_void_p`, `c_long`,
  `c_ulong`, `c_ubyte`, and that exact two-field structure. It never leaves a
  pointer-returning native function with ctypes' default C-int result type.
- `CFString.h` fixes UTF-8 to `0x08000100` and encoding to `UInt32`. Its
  `CFStringGetBytes` uses `CFRange` by value, byte/Boolean flags, a caller-sized
  byte buffer and `CFIndex *usedBufLen`. Complete character conversion is
  required; a partial conversion fails. No terminating-NUL scan or truncation
  is used. Embedded NUL is rejected.
- `CFArray.h` declares count and index as `CFIndex` and array values as borrowed
  pointers. The array stays alive throughout URL reads. Each URL's runtime CF
  type is checked before URL-specific functions are used.
- `CFURL.h` declares `CFURLGetString`, `CFURLGetBaseURL`, and
  `CFURLCopyFileSystemPath`. POSIX path style is `CFIndex` value zero.
  The path copy decodes percent escapes but does not resolve against a base.
  A URL with a base is redacted. `CFURLIsFileReferenceURL` is a Boolean predicate;
  file-reference URLs are hashed/classified before path conversion, without
  resolving their resource. No file resolution or existence probe is used.
- `CFError.h` declares `CFErrorGetDomain` as a borrowed reference,
  `CFErrorGetCode` as `CFIndex`, and `kCFErrorDomainOSStatus` as an exported CF
  string constant. The helper reads that pointer using `c_void_p.in_dll` and
  compares domains with `CFEqual`, rather than classifying an arbitrary domain
  by code alone.

The two SDK `.tbd` files list macOS targets, the native symbol exports and the
exact versioned framework install names used by `CDLL` in the helper. They are
read as text only. No linker, framework load or live symbol lookup ran here.

Each identifier created by `CFStringCreateWithCString`, returned array, returned
error and copied filesystem-path string has one matching `CFRelease` in a
`finally` path. Array elements, URL strings, base URLs, error domains and the
exported domain constant are borrowed. They are used while their owner lives
and are not separately released. These choices follow Apple's
[Core Foundation ownership policy](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFMemoryMgmt/Concepts/Ownership.html).
A controller deadline can kill this direct observer child before its finally
blocks finish. This is bounded child failure, not a guarantee of orderly native
destruction or shared service cleanup.

## Status, bounded work and publication

Apple specifies NULL with `kLSApplicationNotFoundErr` when no matching app is
found. The helper records `absent` only for NULL plus a typed CFError in the
OSStatus domain with code `-10814`. A returned empty array is separately
reported `empty`; a nonempty array is `found`. Those are successful metadata
observations, not statements about running processes or all host registrations.
A NULL without that exact error, another domain/code, simultaneous array/error,
type error, partial string conversion, count/byte excess or native failure makes
the child fail. The controller's unchanged `ro()` accepts exit zero only.
An incomplete queried-ID list also fails. Timeout never means absence or PASS.

The helper accepts exactly the frozen app ID and its `.driver` ID, in order.
It permits exactly two arguments, each at most 128 characters and together at
most 256 UTF-8 bytes. It accepts no caller-supplied root or query pattern.
The controller uses its same Python3.15 executable with `-I -S -B`, avoiding
site initialization and bytecode writes. Both source texts check that version.
No package install or Swift/Xcode build is needed.

Before any native query, a fixed output-envelope calculation reserves at most
`2 * 16 * (6 * 4096 + 512) + 8192 = 811008` JSON bytes, below the 1,048,576-byte
publication cap. Six covers worst-case ASCII JSON escaping of each bounded
UTF-8 path byte. The fixed overhead covers identifiers, status, hashes,
classification, counts and framing. These sizes do not expand with host state.
Before any URL index is read, `CFArrayGetCount` must be between zero and 16.
The helper never truncates an excessive array and calls it complete.

Before conversion, every CF string must contain at most 4,096 UTF-16 units.
`CFStringGetBytes` then writes into a fixed 4,096-byte buffer with lossByte zero
and external-representation false. The full requested range must convert.
Only the returned bounded byte count is read. No API-derived size controls an
allocation or a result iteration. Python serializes bounded records with ASCII
escaping and checks final bytes before stdout publication. No native error
description, userInfo, raw exception text or raw outside-root URL is published
by the Python serializer.

The public API has **no caller-set count limit, allocation bound or timeout**.
It may allocate its native result before the helper can inspect the count.
This source does not prove a strict native peak-memory bound or five-second
completion. It bounds subsequent processing and serialized output, and uses
the existing absolute five-second direct-child deadline for both ID queries
together, including interpreter/framework startup. Cancellation, kill, bounded
reap and observer/reader accounting remain the existing controller mechanisms.
Before loading either framework, the helper duplicates its own stdout pipe for
JSON publication and directs only this child's native stdout/stderr descriptors
to `/dev/null`. This prevents native diagnostic text from bypassing structured
outside-path redaction. The saved pipe receives only the bounded JSON record.
Descriptor-isolation failure stops before native calls. Normal OS logging and
security/registration settings remain unchanged; the existing scoped OS log
capture remains. No diagnostic or descriptor behavior was exercised here.
The unchanged `ro()` still records streams, return status and failure evidence.
No new watchdog, thread or ownership protocol is introduced.

## Scoped paths and precise claim

For each bounded CFURL string, the helper records a SHA-256 digest of its exact
UTF-8 bytes. It reports a path only for an absolute local `file` URL whose POSIX
path has a canonical lexical spelling under a frozen owned root. All other URLs
receive only a fixed classification and that digest. It does not disclose an
outside path, URL, host name or error description in structured JSON. Hashes
provide stable bounded comparison, not secrecy against guessing a known path.

The nine permitted reporting roots are the exact fixture root plus the eight
existing cache roots. A component-boundary comparison prevents sibling-prefix
matches. No symlink, file, process or bundle is resolved, opened or adopted.
`within_frozen_root_lexical` deliberately describes text scope. It grants no
ownership, launch, signal, deletion or unregister authority. All original
independent process and filesystem ownership gates remain separate.

The four original capture phases still record job/listener observations plus
the scoped LS metadata, and the original exact root/ID OS diagnostic capture
remains. This changes the LS metadata observation contract explicitly: no
whole-database record details, timestamps or complete nested-helper registration
inventory are claimed. Two-ID URL metadata cannot establish actual exec,
NSWorkspace relaunch delivery, Gatekeeper/XProtect outcomes or full registration
and security restoration. The other missing independent runtime evidence remains
unverified. Normal registration, notifications and security checks stay enabled.
