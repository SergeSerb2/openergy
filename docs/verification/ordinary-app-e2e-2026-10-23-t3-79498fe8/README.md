# Ordinary packaged app E2E: failed controller, no retry

One ordinary packaged `--e2e` invocation ran from source root
`059f052f965bb942d104448b064a9d0e68b1fda1` on 2026-10-23,
America/Vancouver. **This run is not accepted.** The unchanged app routine
wrote all 25 checks: **23 passed and 2 failed**. The required real-peer
connection and rejoin checks failed after the controller stopped the peer
early. [outcome.json](raw/outcome.json) records this failed result.

The first failure was in this task's new evidence controller. Its process
sampler incorrectly required the supervised daemon's process group to equal
the app PID. The actual daemon had its own process group:
app PID/group `89176`; initial child PID/group `89180`, parent `89176`.
The controller asserted before saving that child's full identity. Its normal
cleanup sent one public `daemon.stop` to the private peer. The same sampler
assertion then interrupted cleanup. Both original tracebacks and the frozen
controller remain unchanged. This controller error invalidates the intended
acceptance run; the peer failures do not establish a product regression.

The app remained alive after the controller exited. It completed the default
routine and wrote [the complete report](raw/recovery-report.json) and
[`E2E FAILED` log](raw/final-app.log). The report includes passing settings,
layout, logs, rollback, ordered intent, stalled-write responsiveness, crash
recovery, ordinary legacy warning and empty-import error checks. These are
individual reported observations. They do not replace the failed integrated
flow. The app's OS exit status was not collected after the controller died;
the source's `exit(1)` branch is not reported as an independently collected exit.
No flow retry, second app launch, crafted import input or optional test mode ran.

The read-only recovery inventory still saw app `89176` and daemon `89180`
with their exact private argv, parent, separate groups, UID, nice 15 and
start times. This preceded the default routine's later stopped-write case.
By the recovery cleanup invocation, all products had already exited, so it
sent **no signals**. The first absence observation was at most **49.538119 s**
after the app's exec receipt. Both private sockets were absent. A later final
inventory explicitly checked all known PIDs and groups and found none.
The recovered daemon's exact PID/start/group and priority were not observed
after the controller failed. No complete process-identity or priority claim
is made for that part of the run.

Both literal high loopback ports, **65371/65372**, passed their first exclusive
bind before launch without reuse or fallback. Reservations remained held
until the respective peer/app launch. Both first postflight exclusive binds
passed. Each retained the port-filtered TCP state before the bind. The real
peer reported its exact owned PID and virtual platform before the app launched.
No later peer retry or replacement occurred.

Artifact reuse passed preflight:

- All **22 target Swift files**, `Package.swift`, `Package.resolved` and the
  normal source Info.plist matched the reviewed `7287e9` build record. The
  complete target file set was checked. The app input hash was
  `95b3891e49e479b7caec9a1e18d0e861d7aae57f1bd178b7455b4bc7920aab0c`.
- All **397 declared runtime inputs**, including the final uninstrumented
  `main.rs` and reviewed `2485c14` runtime, matched this root by path, size
  and SHA-256. The daemon input hash was
  `23e3f9dbee1496b247a5b316811f22c9330b103f979907d1d4fadf2e57399deb`.
- All **124 pinned Sparkle artifact records** matched the retained build
  artifact inventory. Workspace resolution matched the root lockfile:
  Sparkle 2.10.0, revision `eef1a539a373c1f1a320624b1130fc5de7b2e100`,
  checksum `17e28312b8e18ab7cdbbe09a6fb28cc55a5479ec6c371dbc07cdecd2a14fd959`.
- Both reused executables are **arm64 only**. No universal runtime claim applies.

The recorded old `bundle.sh` differs from this root. It is not a target Swift
compile input and did not run. This task used the root's real
`tools/release/sparkle.py` to configure ordinary metadata, embed the complete
framework/helpers/license and sign inside-out with `--sign -`, options 0 and
no timestamp. It passed the tool's architecture, relocation, minimum-OS,
symlink, nested-signature, deep-signature and entitlement checks. The icon was
copied as an ancillary resource from the retained ordinary bundle; its source
and exact hash are recorded. No compiler or old build-state clone was needed.

The fresh bundle was
`/private/tmp/oe-ordinary-79498-l7aurhai/Openergy.app`. It used normal
`dev.openergy.app` metadata and `CFBundleExecutable=Openergy`, with no
`OpenergyPrivateUpdateFixture` key and no `OpenergyPrivateLauncher` file.
Signing changed the copied executable hashes. The exact **executed** app hash
was `53ba6f431811eb1178c39b8b7ad7cdb425b0088e7c893b8dcab8e601ad338c23`.
The exact **executed daemon and peer** hash was
`1c558733a56ada70661f8a8493eae7deaf3813ce1bb572d0592fe86900b6b971`.
The complete signed bundle inventory was unchanged after the attempt.

The exact product argv and private environment allowlists are in the same-PID
exec receipts. Absolute `setpriority(..., 15)` preceded exec; no relative nice
wrapper ran. The sampler retained 20 process rows over 11 rounds, all at nice
15, before its failure. These samples do not cover the complete flow.
The reused daemon's recorded build used Rust 1.99.1, jobs 2, deployment 15.0,
locked offline resolution and no CLI trace overlay. Its original strip warning
(`rust-objcopy` SIGABRT / missing `libLLVM.dylib`) is copied byte-for-byte,
as are the inherited Swift warning stream and build result receipts. No build
or toolchain repair ran. Those build records are audited inherited evidence.

[source-review.md](source-review.md) records the startup and isolation gates
read before execution. Private HOME, CFFIXED_USER_HOME, temporary, config, TLS
and socket paths existed in the allowlisted environment before process launch.
The app's config directory was absent; the unchanged harness created its own
safe Off config before constructing the model. Rendering was disabled with
`OPENERGY_E2E_NO_RENDER=1`; native backend selection and discovery were disabled,
and port 24800 was refused. All configured network endpoints were the two
literal private loopback fixtures.

`raw/predeclared.json` retained the one-flow limit, 120-second watchdog,
60-second owned-cleanup allowance, existing 16 MiB IPC frame cap, unchanged
1,000,000-byte stalled-write input, 300 ms request timeout, existing routine
waits, sample cap and log/evidence byte caps. The watchdog was cancelled by
the controller's failed `finally` path before the app completed. Therefore
this record does not certify continuous watchdog coverage or successful
bounded controller cleanup. Recovery retained the resulting evidence and
observed absence without another product flow.

No source/code changes, unit tests, mocks, GUI rendering, desktop interaction,
installed app, production config/TLS/default IPC, live sharing, native input
or clipboard, TCC, login item, service, task, account, firewall, global DNS,
Docker, credential/Keychain access, signing key, push or publication action
was used. No fixture was configured for TCP 24800, UDP 5353 or non-loopback
networking. The three provider-stopped tasks were not resumed or reframed.
No delegation ran. Running private-update-fixture mode, updater CLI/install,
running-target replacement, relaunch, native behavior and trusted release
remain outside this result.

The first archive-validator invocation also failed: it counted the package
manifest as a target Swift file. There are 22 target files plus one Swift
package manifest, making 23 Swift inputs. `prepared.json`'s `swift_files=23`
is this total, not the target count. The first validator script and traceback
are retained separately. Only that archive-count assertion and explanatory
prose were corrected. No product flow, setup or packaging step was repeated.

Commands run once, in order:

```sh
PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8/prepare.py
PYTHONDONTWRITEBYTECODE=1 /opt/homebrew/bin/python3 docs/verification/ordinary-app-e2e-2026-10-23-t3-79498fe8/controller.py
```

Preparation exited 0. The controller exited 1. Its streams, first runtime
failure and first cleanup failure remain retained. Subsequent actions were
read-only recovery inventories, copying the now-complete report/logs, checking
owned absence and first postflight binds, and validating this evidence archive.
The frozen controller is intentionally not repaired or run again.
