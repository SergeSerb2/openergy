# R2 source and saved-evidence review

Reviewed immutable candidate `9f6625c951835295419ed1c53565581a7fa9681a`, parent `270106002490ef3e135d4c6500df1c8ff452b45b`, through Git objects. Review workspace HEAD was `a121376fe6c1715fa99d72559336ca3b3a73b439`. No fixture, build, test, probe, desktop action, or installer access occurred. Only this review file was added.

## Finding

**P2 — Select the Windows listener wrapper in the new module.** At `crates/net/examples/dns_queue_cancel_c4806a3d/idle_tls_deadline.rs:132`, the new code calls `bound_socket::listen` without a platform guard. The Unix module exposes that one-argument wrapper at the main example's line 419. The Windows module instead exposes `listen_owned` at line 518; its name `listen` denotes the private two-argument Winsock foreign function at line 436. The unconditional module declaration at lines 14–15 makes this a Windows compilation error even when the new option is unused. It prevents building the existing default/full/refusal example on Windows. Use the existing `#[cfg(unix)]` / `#[cfg(windows)]` selection pattern at lines 598–601 or 1024–1027. This conclusion follows from source; no Windows compilation was run. No P1 finding.

## Saved evidence audit

Evidence prefix: `docs/verification/release-integration-2026-10-23/idle-tls-deadline-2701060/` in the candidate commit.

- All 36 entries in `artifacts.json` match the committed bytes. The directory has 37 files, including the manifest itself. All 31 copy-ledger byte counts and hashes match. The only declared redaction is the lsof warning's private mount path; its original hash differs as declared. The private original was not inspected.
- All eight source-input SHA-256 values match. All nine listed production net source hashes match both candidate and parent; `crates/net/src` has no diff. Removing only the new example additions restores the parent's exact bytes and SHA-256 `530c880eb74f6c95d24a0612c4fa8773fa21d256765fc3f9f6260801fa303a30`. This confirms existing routine bodies and predicates are preserved; the Windows compilation finding still applies.
- The raw transcript contains 335 consecutive API call records and 335 matching operation records. All results are `Ok(())`; brackets are ordered and do not overlap. There are 81 preparation polls, 78 late-event polls, and 85 recovery polls on each driver. The eight event records exactly reproduce the final per-driver transcripts, with indices 0–3 on each side. Five traces show exactly two outbound attempts and one healthy inbound acceptance, with matching connection/address identities and no fallback trace.
- The pending command bracket is 35,826,208–36,050,958 ns. The unchanged 500 ms budget gives deadline bounds 535,826,208–536,050,958 ns. Every preparation poll returns before the 285,826,208 ns cutoff. The raw peer receives one 218-byte TLS record containing a ClientHello; record and handshake lengths are exact, and its SHA-256 matches the derived record. Source uses rustls `Acceptor` to parse these received bytes and does not write a response. The pending proof at 137,830,458 ns precedes the earliest deadline by 397,995,750 ns.
- No API call occurs during the recorded idle interval. Call 83 is the first call after idle; it enters 1,868,417 ns after the latest deadline and emits the sole matching timeout. The same pending connection is removed. Recovery retains the caller, handle, waker, transport mode, and timeout. Caller connection 3 and peer connection 2 complete pinned TLS 1.3 and exchange exactly one PASS/BACK frame each. The event lists contain no false Connected event for failed caller connection 2. The late-event and post-BACK observation intervals exceed 100 ms. Shutdown adds exactly one matching disconnect on each side.
- Saved Clippy, release, and format records report success with Rust 1.99.1, offline/locked Cargo, jobs 2, nice 15, and deployment target 15.0. These are inherited records, not fresh validation. The runtime record retains one attempt, no retry, exit 1, no watchdog firing, and process elapsed time 983,154,375 ns. Its private runtime inventory and removal record contain no key contents.

## Cleanup and limits

The immediate stalled-port bind returns `EADDRINUSE` 48 at raw transcript line 697; the healthy-port bind passes at line 698. Source drops both drivers, the retained handle/waker, accepted client, and listener before these checks. Overall **FAIL / exit 1 remains unchanged**. This is a failed cleanup evidence gate; these records do not identify its cause or establish a product deadline defect.

The postflight occurs about 27.5 seconds after process exit. Its filtered netstat output contains two client-end TIME_WAIT rows matching the recorded ephemeral ports, with no LISTEN/ESTABLISHED rows. lsof returns no socket rows but also warns that output may be incomplete. There is no contemporaneous socket-state capture at the failed bind. Thus the saved records do not establish that the immediate error was expected TIME_WAIT behavior, and the later postflight cannot replace the failed check.

Source/binary bindings are internally consistent. The execution and source manifests record binary SHA-256 `35374d73d9a9c8ad68ce7a43b9387241f2d5ee92eab4f01ce7ad78e163aec855`; the executable is not retained in this committed bundle, so its bytes were not independently checked. This review audits saved evidence only. Continuous strict TCP/TLS/fallback timing, the matching historical negative, Windows runtime acceptance, and all wider UNVERIFIED gates remain unchanged.

## Exact evidence identities

- New module Git blob: `7d214e27c49abcf0643eee3fa7c6977022469f0f`; SHA-256 `2c3be2b299867d2779160d62035cb3133e53216a8a001f3ec12275296935bd02`.
- Main example Git blob: `367621a2dcf96cbc86b28552b7e517ce0d2752be`.
- `raw/runtime.stdout` Git blob: `2021043e2730007fc0daafd0195f57407e06ba25`; SHA-256 `759fa0cec0df9fb7689cc38571cd4e04a836589bfb45286b7676baeb2dc05200`.
- `artifacts.json` Git blob: `f51a21f71630fa9994acd2bf3b83c4183739d792`.
- Received ClientHello SHA-256: `1f5f78d69d334d0bc65f2a89ceabcb9e0d3b02bf7588c4a3cc974ef811cefce5`.
