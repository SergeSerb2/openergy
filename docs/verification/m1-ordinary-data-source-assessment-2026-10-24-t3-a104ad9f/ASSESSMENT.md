# Saved M1 ordinary artifact: DATA/SOURCE assessment

Static qualification is supported for the intended arm64 execution input. No
concrete static incompatibility or missing arm64 code-object record remains in
the supplied observations after the supplement. This includes the
`m1_process_abi` gate as the receipt contract defines its header/interface
qualification. Runtime is **UNVERIFIED / NOT ACCEPTED**. This report is not an
artifact receipt, run receipt, execution approval, or new permission gate.

Scope: read the saved `receipt.json`, all eight `retained/*.json` files,
`supplement-reference.json`, unchanged R9 `macproc.py`, the parent plan and
receipt contract, bound Sparkle pin/plist source, completed-build BSD source
and saved process JSONL, and the supplied admission-review commit. Read files
and parse JSON/text only. No task module was
executed, imported, compiled, or parsed as Python code. No OS metadata tool,
process probe, remote call, native parser, product, signing, build, test, or
network action was run by this assessment. No private input was changed.
`hash-bindings.json` binds the exact local inputs and embedded header text.

The six qualification outputs, readback, and supplement match every recorded
remote byte count and SHA-256. All ten embedded SDK header texts also match
their recorded byte counts and SHA-256. The original records remain intact.

## R9 process ABI

The local R9 file is exactly 8,075 bytes, SHA-256
`d4f6fe358bd9af786fc4c2cba61f1a47f3f4015c918aae85cf8e6aa1c6d3e6c7`,
matching the parent's immutable binding. There is no M1 reader patch.

The actual saved SDK is `MacOSX27.0.sdk`. Its `sys/proc_info.h:59-82`
agrees with R9 `macproc.py:8-15` in field order, signedness, and width.
The type chain is now present: `sys/types.h` includes the uid/gid type headers;
these alias `__darwin_uid_t` / `__darwin_gid_t`; `sys/_types.h` defines both as
`__uint32_t`. `arm/_types.h` defines that as unsigned int and `__int32_t` as
int, with the 64-bit integer types based on long long. `__darwin_pid_t` is
signed 32-bit; the PID/group fields *in proc_bsdinfo* are unsigned 32-bit.
R9 correctly uses `c_int` for API PID arguments and enumeration, and
`c_uint32` for the structure's PID/group fields. `nice` is signed 32-bit.
`param.h:95` sets `MAXCOMLEN=16`, so the two arrays are 16 and 32 bytes.

Under the target Darwin arm64 C ABI, natural alignment gives these offsets:
the twelve initial 32-bit fields occupy 0 through 47; `comm` starts at 48;
`name` at 64; `nfiles`, `pgid`, `jobc`, `tdev`, `tpgid` at 96, 100, 104, 108,
112; `nice` at 116; `start_sec` at 120; `start_usec` at 128. Total size is 136,
alignment 8, with no extra padding before the two 64-bit fields. Neither the
retained declaration nor R9 introduces packed layout. This is a source-derived
layout, not a compiled `sizeof` result or an executed ctypes measurement.

There is also real, already-saved kernel-read context for this layout.
The exact completed-build controller (SHA-256
`cd5dc620e6adbec09798e600a99f55c0368b8eeff10d1117e1f7793ca1baf4ee`)
defines the same field order/types/arrays at lines 354-360, naming the final
fields `seconds` / `microseconds`. Lines 365-376 require ctypes size 136 and
use proc_pidinfo flavor 3 with a full-size-result check. Its retained
`processes.jsonl` matches the build binding hash
`d93547dd19882ae876f14d5506a8f4bdf01a088ee564ef564befcc672f390987`.
Reading that saved data gives 2,008 rows, 90 PID/seconds/microseconds identities,
and nice values exactly `[15]`. This strengthens the BSD-layout qualification
with prior actual-machine use. It proves neither future R9 argv decoding nor
future ordinary-flow priority, identities, or success. No new kernel read ran.

`proc_info.h:723-724` gives flavor 3 and `sizeof(struct proc_bsdinfo)`.
`libproc.h:93,96,102` exactly matches R9's argument and return declarations:
`int proc_listallpids(void *, int)`,
`int proc_pidinfo(int, int, uint64_t, void *, int)`, and
`int proc_pidpath(int, void *, uint32_t)`.
`sysctl.h:188,264-266,808-809` gives CTL_KERN=1, KERN_PROCARGS2=49 and
`int sysctl(int *, u_int, void *, size_t *, void *, size_t)`.
R9 uses these selectors and the matching pointer/unsigned-int/size_t types.
Darwin arm64 has 32-bit int/unsigned int and 64-bit pointers/size_t; the saved
private interpreter's arm64 architecture and Python 3.13.9 version support
that target selection. Headers support the interface, not live symbol binding.

R9's argv decoder assumes a four-byte signed little-endian argc, then an
executable-path NUL string, alignment NULs, then NUL-separated argv. It permits
1..128 arguments, reads at most 1,048,576 bytes, and requires strict UTF-8.
It ignores the environment after the selected argument count. M1's little-endian
ABI is consistent with the integer decode. The saved headers declare selector
49 under `__APPLE_API_UNSTABLE`; they do not specify the complete returned
byte serialization. Thus format compatibility is an interface inference, not
an observed payload or an independent kernel-format specification in this set.
The fixed capacity is not a measured ARG_MAX, and 128 is a reader limit, not
an OS limit. The split/count check does not independently establish that the
last argument has a terminating NUL, and leading empty argv entries are outside
this decoder's intended nonempty executable-argv form.

These limits do not show a static mismatch for the prescribed private actor
commands. They must remain explicit. Real rows must still establish successful
reads, byte counts, PID/start/parent/group/uid identity, microsecond starts,
path/argv coherence, and NI15 through the actual flow. No mock or simulated
probe supplies those facts. This assessment does not turn header agreement
into runtime acceptance or broaden the R9 parser's supported inputs.

## Artifact architecture, signatures, entitlements, and loads

All 52 original metadata commands have saved exit 0. The supplement adds eight
codesign command records, all exit 0, plus an arm64 private-Python architecture
record with exit 0. The latter retains label/output/status but no argv; it is
metadata evidence, not a runtime process-reader test.

All seven observed Mach-O binaries have an arm64 slice. App and daemon are thin
arm64. Autoupdate, Sparkle, Updater, Downloader, and Installer retain full
`x86_64 arm64` architecture lists. Do not thin or rewrite these upstream files.
App SHA-256 is
`094d02b3831ee6565ae1628407756ad48053a0268f2a489c65bbde28b768d775`;
daemon/peer SHA-256 is
`17065c334290b34c7a1b69483f409655e072eb0505ff2a0cc0bf6ed5f77a7bb2`.
All seven hashes agree with their records in the complete saved bundle tree.

The original `object-0-*` codesign records inspect Sparkle.framework, not
Autoupdate. Original `object-5-*` codesign records inspect Openergy.app, not
the daemon. Their lipo and otool records do target the individual binaries.
Use supplement records 0..3 for the daemon's verify/display/entitlements/
requirements and 4..7 for Autoupdate's. Do not assign the original containing
object's identifier, CDHash, or empty entitlement output to those executables.

The corrected seven objects have these arm64 identifiers and CDHashes:

- App: `dev.openergy.app`, `07f9138c67bb59ebdc8c04536fc0c9cb108b0fb0`.
- Daemon: `dev.openergy.daemon`, `7a22fc8bdf7d7e91593fdec6b1791a3f7f549f7d`.
- Sparkle: `org.sparkle-project.Sparkle`, `3f3e05da4485ba252a1177c7c26e8e6d369d01ba`.
- Autoupdate: `Autoupdate-55554944f723d84042cd352fbb0485760ff7597a`,
  `62f4c0cdc10f828fddb18badc84fe943810c30ef`.
- Updater: `org.sparkle-project.Sparkle.Updater`, `36c117bb659fa53238129deebac0cd3a6e033157`.
- Downloader: `org.sparkle-project.DownloaderService`, `ae3b5c6eafba74a880a6380ca264c8f283e548d7`.
- Installer: `org.sparkle-project.InstallerLauncher`, `9f978435a99227740bf22c9eea775cf5d2e9d653`.

All corrected displays say `Signature=adhoc`, flags `0x2(adhoc)`,
`TeamIdentifier=not set`, and internal requirements count 0, size 12.
Requirements commands exit 0 with empty stdout; retain that result rather than
invent an explicit named requirement. App, daemon, and framework entitlement
commands exit 0 with no entitlement payload. Updater and both XPC services
emit empty dictionaries. Autoupdate emits exactly
`com.apple.application-identifier = org.sparkle-project.Sparkle.Autoupdate`.
The raw XML and the deprecated `:-` option warning are retained. These distinct
empty/no-payload results must not be collapsed into invented entitlement data.

Strict per-code-object verification and strict deep outer-bundle verification
all exit 0, with valid-on-disk / designated-requirement success text. Verify
commands include `--all-architectures`. Detailed signature, requirement,
entitlement, and load-command output is arm64 only; no corresponding detailed
x86_64 qualification is claimed. The receipt targets arm64 execution.

The saved `otool -L` and `-l` outputs agree. Sparkle's
`@rpath/Sparkle.framework/Versions/B/Sparkle` entry is **LC_ID_DYLIB**, its
install name, not an additional dependency. The app contains the one actual
non-system dylib load, with that same name. Its recorded LC_RPATH order is:

1. `/usr/lib/swift`
2. `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-6.2/macosx`
3. `@executable_path/../Frameworks`

The third expands from `Contents/MacOS/Openergy` to `Contents/Frameworks`,
yielding the exact inventoried `Sparkle.framework/Versions/B/Sparkle`, whose
arm64 slice is qualified above. The other two search roots remain recorded;
their candidate existence/absence and actual dyld selection were not measured
by these observations. This proves a valid embedded resolution candidate, not
a runtime loader trace. No actual dylib load names a developer-toolchain file.
All other 86 dylib loads across the seven objects use `/usr/lib/` or
`/System/Library/` paths, including the app's weak Swift loads. Executable
LC_LOAD_DYLINKER entries name `/usr/lib/dyld`. No other non-system load or
unresolved bundle-relative dependency is present in the saved arm64 commands.
This does not verify the availability or contents of every OS library.

## Other artifact gates and remaining boundary

The saved bundle inventory has 167 entries: 95 regular files totaling
10,050,709 bytes, nine internal symlinks, and 63 directories. Root mode is 0700.
It records `matches_build=true` and `source_files_verified=702`. App, daemon,
Info.plist, licenses, resources, and embedded code remain in that full tree.
The parent already completed the 702-file raw Git/source qualification and
complete Python prefix measurement; this assessment does not rerun either.
The prefix record has 2,006 entries / 46,504,040 regular bytes, executable mode
0700, exact bound executable hash and prefix digest
`c46c9234c07785e90b66774ca161f20c7ef72c0a87323f52583fa4edb1186e67`,
canonical/authenticated-before-entry true, and Python 3.13.9 version exit 0.
The saved version argv uses `-I -S -B --version`; that is not authorization to
change the R9 controller's launch argv or its required private startup rules.

Ordinary metadata has BundleID `dev.openergy.app`, executable `Openergy`, APPL,
version `0.1.0`, build `1`, minimum system `15.0`, source-transform match true,
and all-plists-no-private-fixture true. The complete inventory has no private
launcher. Sparkle is the retained pinned 2.10.0 build input, revision
`eef1a539a373c1f1a320624b1130fc5de7b2e100`, archive SHA-256
`17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
The full bound build source inventory, Package.resolved, and release-tool
constants agree on these pins. Actual generated values are
`SUEnableAutomaticChecks=false`, `SUAutomaticallyUpdate=false`,
`SUAllowsAutomaticUpdates=true`, with no `SUFeedURL` or `SUPublicEDKey`.
Preserve all three values; the last is not evidence of updater execution.
The existing successful-current-build bindings remain the source for build
success, immutable Sparkle provenance, and the original full build inventory.
The retained `rust-objcopy` SIGABRT / missing `@rpath/libLLVM.dylib` warning and
missing complete outer build OS-exit record stay explicit; neither is repaired
or replaced by the new metadata observations.

The transfer record explicitly selects the original build bundle on the same
machine, no transfer for execution, archive not extracted, and tree agreement.
There is no destination tree to compare at this source qualification stage.
Timestamps/ACLs/xattrs are NOT CLAIMED. The required post-preparation private-copy
tree/signature/entitlement comparison remains a later parent operator step
under the existing contract. This report does not assert that copy exists.

Thus the saved inputs support the requested static artifact gates, using the
parent's already completed source/build/prefix facts with the corrected direct
code-object observations. The remaining limits above are limits on what this
evidence proves, not an added human approval cycle. Parent must still produce
the exact closed artifact receipt with complete raw references, and later
retain the private-copy qualification and actual single-flow runtime evidence.
No Gatekeeper, certificate trust, notarization, release, native UI, or runtime
acceptance result follows from these ad-hoc verification records.

The parent supplied source-admission R2 clearance commit
`08ac9ae4eff90099c83f5314eba0482f8b3f9cbe`; its saved review states no concrete
P1/P2 in SOURCE/DATA scope and keeps runtime unverified. This assessment does
not repeat that controller/admission review. The parent's report of eleven
exact staged data files, without supplied-module or product execution, changes
no runtime conclusion here.
