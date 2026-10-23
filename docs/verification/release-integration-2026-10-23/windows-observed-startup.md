# Native Windows observed startup verification — 2026-10-23

**Windows revised startup acceptance is FAIL / unverified on exact source `e6a9125be2fc59fbe92fef6ea351b222298bfc52`.** Both fixed runs exhausted at least one required eight-attempt `Disconnected` witness budget. Every positive-run failed check is a missing-coverage check; no product defect is inferred. Retained-800 passed 19/19. Both explicit observer-disable controls produced their required intentional failures and normal cleanup observations.

[Structured evidence](windows-observed-startup.json) contains every check, Accepted/Disconnected outcome, ordered trace, per-run coverage table, public cleanup barrier, command, source/tool/library/artifact hash and invariance result. [Exact workflow](windows-observed-startup/workflow), [source manifest](windows-observed-startup/source-manifest.json), [artifact manifest](windows-observed-startup/artifact-manifest.json), and [full copied outputs](windows-observed-startup/outputs) are distinct new evidence. No candidate source, assertion, threshold, capacity, sleep or workload was changed. Each of the five commands ran once.

| Run | Exit | Candidate checks passed/total | Duration | Strict verdict |
| --- | ---: | --- | ---: | --- |
| startup-1 | 1 | 105/106 | 38,267 ms | FAIL: coverage |
| startup-2 | 1 | 113/115 | 37,836 ms | FAIL: coverage |
| retained-800 | 0 | 19/19 | 41,266 ms | PASS |
| missing-startup-observer | 1 | 0 checks; early coverage error | 4,062 ms | PASS as negative control |
| missing-cleanup-observer | 1 | 95/124 | 110,992 ms | PASS as negative control |

The source includes `2253049d72f4ee5aa079045432c5014bb0c4be78`. Parent R8 review at `787a4afb8aa80b47192bd28c0ac60e662bef810d` was read from immutable Git. It provides source review, not runtime approval. The reported Mac 61/61 passes and working-observer 45/61 buggy baseline were not independently reproduced on Windows and were not used to fill Windows coverage. No Windows buggy-baseline run was performed.

## Actual startup coverage

Each run requires a real failed initial send for all four replacement combinations, within eight finite attempts per combination. In the table, values are actual **Accepted / Disconnected** replacement outcomes. An accepted immediate drop remains an accepted lifecycle and is never counted as a failed-start witness.

| Replacement over old view | Startup 1 | Startup 2 |
| --- | ---: | ---: |
| limited-over-limited | 8 / 0 | 1 / 1 |
| standard-over-limited | 5 / 1 | 8 / 0 |
| limited-over-standard | 1 / 1 | 8 / 0 |
| standard-over-standard | 0 / 1 | 2 / 1 |

Startup 1 failed only `limited-over-limited-disconnected-coverage`. Startup 2 failed `standard-over-limited-disconnected-coverage` and `limited-over-standard-disconnected-coverage`. Every missing case used all eight attempts and observed eight Accepted outcomes with zero Disconnected outcomes. **Coverage is required per run; witnesses are not pooled across runs.** No further attempts or retry-to-green runs were made.

The 32 distinct starts were also classified by actual send outcome. Run 1 recorded three Disconnected and 29 Accepted, with gauge 30 at its barrier. Run 2 recorded six Disconnected and 26 Accepted, with gauge 27. Both retained the independent anchor, met the exact eventual state requirement, and passed fresh update checks. Neither these temporary accepted states nor missing replacement coverage is labelled an orphan-state product defect.

For every actually witnessed old-standard failure, the original probe drained after the failed-send public barrier, then observed a fresh later repeated search. The full traces contain two such pairs in startup 1 and one in startup 2. Unwitnessed combinations remain failed coverage. The report preserves all initial Accepted outcomes; those records precede replacement commit and are not commit acknowledgements. Later public command barriers, resolution, identity and exact-state checks retain their separate meaning.

Both positive runs also passed the mandatory accepted, resolved, committed and active standard browse Drop. Each recorded **exactly one** actual normal `Disconnected` removal for `_oe-drop-std._tcp.local.`, followed by its public barrier with `count=1 valid=1 passed=true`. That required case did not depend on which immediate-drop attempts happened to be Accepted.

## Negative controls

The exact candidate supports both requested environment interfaces. The same frozen executable ran them once each, using `ProcessStartInfo.Environment` overrides only. Both keys were removed from each new child environment first; the selected control was set to `0`; overrides were removed after exit. Parent values stayed unchanged. No machine/user environment value changed.

With `OPENERGY_STARTUP_OBSERVE=0`, the probe exited 1 before normal checks because its initial anchor send had zero observer matches:

```text
discovery_recovery: unverified initial send: SEND type=_oe-anchor._tcp.local. kind=limited outcome= matches=0
UNVERIFIED_CLEANUP publisher_exits_zero=true worker_shutdown_ack=true
```

The error is from stderr; the cleanup line is from stdout. This is an intentional missing-observation failure, not a 0/0 acceptance pass. Its normal publisher/worker cleanup branch ran.

With `OPENERGY_STARTUP_CLEANUP_OBSERVE=0`, startup-send observations stayed enabled and the mandatory held-standard setup passed. The probe then reported:

```text
AUTOMATIC_CLEANUP type=_oe-drop-std._tcp.local. kind=standard witnessed_in_time=false matched=0 valid=0 states=1 passed=false
ACCEPTED_STANDARD_DROP_COVERAGE type=_oe-drop-std._tcp.local. committed_active=true normal_removal=false
```

The relevant automatic-removal and coverage checks failed, so this is a valid negative control. Its 29 failed candidate checks also include three natural missing replacement witnesses; those are kept separately in the JSON, not attributed to disabling cleanup logging. The full invocation remains a failure, as intended. It used the existing explicit-stop fallback after the failed automatic-removal check and finished with publisher exit 0 and worker shutdown acknowledgement. No normal case or control timed out or required forced cleanup.

## Exact source, isolated build and frozen artifacts

A new current-user-only protected Windows TEMP fixture held fresh source, target, binary, runtime and TEMP/TMP directories. The archive SHA-256 is `5c5657785c8d8b11ea37cc81b89bfe27f4a1a0e099a497047511700251f72b56`; source-manifest SHA-256 is `e1e41e648872e05f1f0a82d86f2bef7f9872db48a174b89f4244881a4dc8e3cb`. All 863 regular files matched the exact Git tree. Of these, 862 matched blobs byte for byte. The one required CRLF conversion is `tools/windows/openergy-run.cmd`: 502-byte clean blob, 511-byte archive file, exact Git clean-filter blob `9cd6f37ce26dde4edcad6f808492171f8daf3462`. No check was weakened. Source files stayed read-only and matched after build and runtime.

The existing offline registry was copied into a new private Cargo home: 6,300 files, 289,293,697 bytes. All copied bytes were checked, and all 4,094 copied registry source files were made read-only and remained unchanged. Cargo used only that new home and writable private target. The original registry/cache and Cargo metadata files were not selected for writes.

Existing pinned native MSVC Rust 1.99.1 ran two sequential steps, with the build wrapper in session 0, `BelowNormal`, and `CARGO_BUILD_JOBS=2`:

```text
cargo clippy --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-net --lib --example discovery_recovery -- -D warnings
cargo build --locked --offline --release --target x86_64-pc-windows-msvc --jobs 2 -p openergy-net --lib --example discovery_recovery
```

Clippy exited 0 in 17,724 ms; release build exited 0 in 44,832 ms. No unit tests or unrelated probe runtime ran. Cargo.lock, toolchain, compiler, dependency/library and installed native-tool hashes are recorded. All 1,018 files in target/bin were frozen read-only and retained their hashes. The same executable supplied all five cases:

```text
bin/discovery_recovery.exe  1,036,288 bytes
SHA-256 3de61efe6aa61f88b89723a015961cfdd8f15bf88078789905d9321fb77e21eb
```

| Frozen library copy | Bytes | SHA-256 |
| --- | ---: | --- |
| `libmdns_sd-d2bc7df45f7006c8.rlib` | 3,941,368 | `ee0326005dd79485c64c7b061a92c31739220be80978d4ec29884041420aa138` |
| `libopenergy_net.rlib` | 2,033,956 | `53062888a5fbc4859aeb62671093fd666e5b1bdd8545cd14fd2f1921d01c57e2` |

Commands, in fixed order, where `<BIN>` is the new private frozen binary directory and `<RUNTIME>` is the new private runtime directory:

```text
<BIN>/discovery_recovery.exe startup <RUNTIME>/startup-1 25090 25094
<BIN>/discovery_recovery.exe startup <RUNTIME>/startup-2 25092 25096
<BIN>/discovery_recovery.exe probe <RUNTIME>/retained-800 25090 25094
OPENERGY_STARTUP_OBSERVE=0 <BIN>/discovery_recovery.exe startup <RUNTIME>/missing-startup-observer 25092 25096
OPENERGY_STARTUP_CLEANUP_OBSERVE=0 <BIN>/discovery_recovery.exe startup <RUNTIME>/missing-cleanup-observer 25090 25094
```

The last two lines describe child-local overrides, not shell or machine environment changes. Startup/control outer deadlines were fixed at 360 seconds and retained at 120 seconds; internal probe waits and assertions were unchanged. Preflight and after-run endpoint inventories passed before each next case. Only assigned loopback 25080–25099 and existing OS-assigned command-wake sockets were used.

## Cleanup, invariance and limits

All startup/control runs used normal publisher finish/exit and worker-shutdown acknowledgement. Retained separately reports publisher exit 0, sentinel-worker acknowledgement and Discovery shutdown submission. Successful nonempty OS process/TCP/UDP inventories then found no owned executable or assigned endpoint after each process exited and at final audit. **These are distinct proof levels.** No actual owned-worker disappearance while a helper remains alive was measured on e6a9125. Immutable dfa405c actual-OS evidence applies to old e4f48ba only and was not rerun or transferred.

The full before/after byte/hash/read-only/ACL audit passed for 2,693 files across the three prior fixtures, all 6,300 original registry files and all 166 pinned compiler-tree files. Root/directory ACLs and original Cargo/Rustup global metadata presence/hashes also matched. The dfa405c helper, binaries and evidence were unchanged. Live sharing process identity and tool defaults were preserved. The new protected fixture had no reparse point. Final private TEMP was empty, with no owned executable or assigned listener/endpoint remaining.

A local workflow-inspection Python expression initially had `SyntaxError: unmatched ')'`. It failed before native build/runtime, was corrected, and is preserved with its stderr hash. The two actual positive startup failures and fixed-sequence wrapper exit 1 are also preserved. All transport exits were observed; none was converted from unknown to success.

Private originals and the exact Windows locator remain under `/tmp/oe-win-observed-startup-20261023-PbXCUE`; `private-evidence-sha256.json` indexes them. The downloaded raw bundle SHA-256 is `16e326f129853e19749a5d23763a3471cc74e7f20ab0796f33fdd29735d7336a`. Public reports redact personal roots/host/user/SID and process IDs, omit unrelated machine inventories, and record original/copied hashes. The copied artifact manifest uses LF for repository text policy; its original Windows hash is retained. Source-manifest and workflow source bytes remain exact.

Windows startup acceptance remains unverified. Original e4f48ba 11/14, diagnostic runs, retained 19/19, control 23/23 and dfa405c evidence remain immutable historical results. Original DNS 500 ms refusal failures and other gates remain open: slow DNS, whole-cache/expiry/timer/query/record-byte bounds, native input/clipboard/backend, positive Windows migration, security/provenance/privilege and trusted distribution. No stopped IPC/Synergy3 work, original expiry/NSEC/orphan experiment or blocked provenance review was continued or rerouted. No installed/live state, desktop/windowstation/TCC/login/service/task/elevation/account/credential, network/firewall/global limit, Docker/tool installation/default, signing/publication/GitHub/remote-ref work was performed. No mocks, computer use, agents or other-thread messages. Private reporting remains deferred.
