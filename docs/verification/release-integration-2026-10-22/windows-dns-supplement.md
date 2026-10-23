# Supplementary native Windows refusal diagnosis

The supplementary NetDriver refusal matrix and later lifecycle/IPv6 contracts PASS on the frozen native library built from `07adb0a3cdc4cff1aa77ce31dbe078a48db56bd5`. The strict 500 ms OS controls still FAIL. The original full and refusal-only exit-1 results in [windows-dns-cancellation.md](windows-dns-cancellation.md) remain unchanged. This supplement does not grant overall Windows acceptance.

A new protected current-user-only diagnostic directory held the helper, executable, logs, runtime, TEMP and TMP. The old frozen fixture was read only. All 658 source files, both original/frozen executables, all seven recorded libraries, original archive/manifest/scripts and all 30 original evidence files passed the postflight hash audit. No long package build or original probe was repeated. The parent's separate `tools/e2e` changes are outside this pinned native scope.

The [helper source](windows-dns-supplement/source/windows_dns_supplement_v3.rs) links directly to the exact previously recorded `libopenergy_net.rlib` (`1ba0d30e9f9649ce598b6d1035772d3712690e4a58e4011e41faa7ce3f241695`) and `libopenergy_session.rlib` (`afedd1f9fe61cd52ab14373cf276a6b1e4884e1000ea3ddcfb4733aefdaa003e`). Seven shared helper functions are byte-identical to the original probe. The original later contracts body has only three documented changes: separate its refusal-matrix call, explicitly restore the same three-second connection timeout, and return unit instead of the removed refusal result. No contract assertion was removed. Exact function hashes and build arguments are in the [JSON evidence](windows-dns-supplement.json).

The standalone helper compiled with existing native MSVC Rust 1.99.1, `-D warnings`, release optimization, panic abort and thin LTO. Compilation took 10,653 ms and exited 0. One rustc invocation ran at a time, with `CARGO_BUILD_JOBS=2`, `BelowNormal` priority and session 0. No toolchain/default change occurred. The original exact-source strict package release Clippy result remains the earlier exit 0; it was not rerun.

```text
helper source  17231 bytes   009193b58d548fd5b2453599db81e7167ff35aee9f623aef1995c2fe70c92675
helper exe     2376704 bytes 6f8cfe24486017782bbfbb2838fea0da71a2c9dcf2bc5596133f77f4dfc1dcb3
helper PDB     2256896 bytes 576c580209b57369257a0e7063aba096abbb1b0d70e335d9c6ea374173511661
```

OS mode ran once and exited 1 in 10,418 ms. It measured numeric IPv4 and IPv6 on private ports 25117 and 25119 with separate 500 ms and 6,000 ms `std::net::TcpStream::connect_timeout` attempts. Before each attempt, it set and read back `SO_EXCLUSIVEADDRUSE=1`, bound the exact loopback address, confirmed a duplicate std bind failed with `AddrInUse`/errno 10048, then closed its owned listener. All eight exclusive controls passed.

All four 500 ms attempts returned `TimedOut`, with no raw OS errno, after 502,310–515,060 microseconds. All four separate longer attempts returned the required `ConnectionRefused`, errno 10061, after 2,019,656–2,051,868 microseconds. No silent/timeout outcome was counted as refused. The strict OS mode therefore stays failed because of its short-window observations, even though all longer controls passed. These observations support a short-observation mismatch. They do not establish the cause of the approximately two-second refusal delay. No firewall, DNS, hosts, stack, interface or network setting was changed.

NetDriver mode ran independently of the failing OS prerequisite and exited 0 in 60,325 ms. It kept the five-second connection deadline and observed every case for a full six seconds, including events after the first result. For each port it exercised numeric IPv4, numeric IPv6 and three localhost connections. All ten cases emitted exactly one matching `ConnectFailed` whose message began `connect_refused:` and classified as `ConnectErrorKind::Refused`. There were no false `Connected` events or duplicate/late extra events. Numeric first events arrived in 2,012,906–2,049,742 microseconds; localhost first events arrived in 4,054,102–4,092,032 microseconds. Each final observation lasted at least six seconds. No product classification error was exposed in these cases.

Contracts mode ran once and exited 0 in 2,206 ms, with empty stderr. It passed same-driver, original-handle, same-waker, DisconnectAll listener preservation, Shutdown listener closure, valid reconfigure, invalid-reconfigure recovery, localhost, numeric loopback and driver-drop assertions. Four real frame round trips used the original exact PASS/BACK frame checks. The two explicit IPv6 results were:

```text
HEALTHY host=::1 peer=[::1]:25116 frame_roundtrip=PASS
HEALTHY host=localhost peer=[::1]:25116 frame_roundtrip=PASS
```

The actual Rust resolver order remained IPv6 `::1`, then IPv4 `127.0.0.1`, at both refusal ports. IPv6 was available. The earlier post-reconfigure localhost frame path used the IPv4-only healthy listener; the later localhost frame path selected the IPv6 listener. No per-socket attempt trace was added. All traffic used only assigned loopback ports; discovery, TLS and native backends were disabled.

All first helper failures were preserved. The first fixture setup used an unavailable .NET Core ACL overload and exited 1 before directory creation. The Windows PowerShell replacement created one protected fixture but its transport timed out at 55 seconds; its remote exit is unknown. Recovery reused that fixture and completed the input audit in PowerShell 7. The first standalone compile exited 1 with E0463 until the existing host dependency search path was included. The second exited 1 with E0308/E0277 for the helper's Windows RawSocket type; the metadata-only lookup run retained the same error. The third exited 1 because strict warnings rejected an unused copied queue helper. Only the diagnostic source/build arguments changed. No warning was suppressed and no product source was edited. Raw logs, source versions and exact exit records remain in the JSON/private evidence. The final Git whitespace check exited 2 for one blank line at the helper EOF. That formatting warning is retained so the committed helper stays byte-identical to the compiled source and its recorded hash.

Postflight passed at 21:52:13 America/Vancouver. The helper source/executable and all frozen inputs kept their hashes. The protected new directory and all 47 audited items had only current-user access. TEMP was empty. No executable from either fixture remained, and no TCP listener or UDP endpoint remained on 25100–25119. No runtime timeout or forced cleanup occurred. Owned evidence files and binaries are retained for review; no unrelated process was stopped. The private downloaded raw bundle, including the helper executable/PDB, has SHA-256 `e2e9be541e578978b019625aa19208c324b966d0b665bccba83c57cf0ee44d0a`.

The archived build/run/audit scripts under [windows-dns-supplement](windows-dns-supplement) record the exact diagnostic workflow. They ran only inside the new protected private fixture, with the old frozen path supplied as `-FrozenPath`. They are evidence, not changes to the product probe.

The original short-window acceptance failure remains open. Active OS DNS timing/cancellation, deterministic slow-DNS, provider-stopped crafted expiry, native input/clipboard, positive Windows migration, privilege/provenance and trusted distribution gates remain unverified. No installed daemon/backend, live sharing, production config/TLS/credentials/default IPC, 24800/5353, desktop/windowstation/TCC/login, services/tasks/elevation/accounts, global limits, network settings, toolchain defaults, Docker, publication, push/merge, GitHub posts or other-thread messages changed. No unit tests or mocks ran.

This completes only the assigned bounded diagnosis. No new investigation or follow-up was started. Work stops after this scoped evidence/source commit until the user resumes it.
