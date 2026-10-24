# Current connected app idle baseline: UNVERIFIED

The single runtime attempt failed in the controller before app readiness. It did
not produce a connected app idle baseline. No retry ran. No resource budget,
memory bound, speedup, or production change follows from this result.

The first process identity check saw the newly started app PID while it was
still the `nice` launcher:

```text
AssertionError: ('app', '/usr/bin/nice', '/private/tmp/oe-idle-9df59d98-gw9bek1k/Openergy.app/Contents/MacOS/Openergy')
```

This is a controller startup race, not an observed Openergy failure. The
controller required the final executable path on its first sample. It did not
allow the launcher to finish `exec` before that check. The partial sample was
not appended. [raw/os-samples.json](raw/os-samples.json) is therefore empty.
The app wrote no stdout, stderr, model markers, config directory, or report.
App readiness, trusted peer connection, 30-second idle, and the final public
disconnect/rejoin/readback are all **UNVERIFIED**. There are no CPU, RSS,
thread, FD, process-state, or effective-priority series to report.

## What ran

Source parent: `8c9f0abbdf4016e22738b5b5a51e7c35185e3596` in the isolated
child checkout `/Users/serge/.t3/worktrees/openergy/t3-9df59d98`. The retained
private root is `/private/tmp/oe-idle-9df59d98-gw9bek1k`.
The final read-only parent-root check found clean HEAD
`af5ab9ae5205c79f30fb97db180679c361f09c7e`. This task made no write to that
checkout. Its private build used the supplied parent above, not the later root
HEAD. [raw/root-readonly-postflight.json](raw/root-readonly-postflight.json)
records this distinction.

The private whole-app release build passed on its first attempt in **25.934
seconds**. It copied the pinned Sparkle dependencies to a new owned root,
disabled automatic resolution, and used two build jobs with deployment target
15.0. The build used Swift 6.4 (`swiftlang-6.4.0.34.1`), Xcode 27.1
(`27A9275`), SDK 27.0, on macOS 27.2 (`26B5112a`), Darwin arm64. The current
Rust toolchain check returned `rustc 1.99.1 (58cf8f9c3 2026-10-13)`. No Rust
rebuild ran. No TypeScript source or validation was involved.

All **456** recorded source inputs for the cached daemon match this parent
byte for byte. The daemon was copied from the prior bundle, not rebuilt. Its
SHA-256 is `350db4541b95f67b8a295419d71933eb4832d07de37881c781664d212a29d572`.
The private app executable after its private ad-hoc bundle signature is
`92a8d614b3fb6cfd063dfa6660a9afa5f63e42cf2ab1583edaf88bd6f656ddb0`.
The compiled executable before that signature is recorded separately.

[harness-overlay.patch](harness-overlay.patch) is the complete executed
overlay. It adds an opt-in finite idle branch to a copy of `AppEndToEnd`; the
whole original harness remains present. All other copied app inputs match
production, including `AppModel`, `DaemonClient`, `IPCSocket`, and
`DaemonSupervisor`. Neither production sources nor the parent checkout was
used as a writable target. The baseline and final hash checks found no source
or historical cache changes. No artifact binary or TLS private key is in this
directory.

The owned peer daemon reached `listening`, with virtual platform, TLS enabled,
PID **75667**, name `idle-peer-9df59d98`, and literal endpoint
`127.0.0.1:62643`. Its real public `status.get` readback supplied its public
certificate fingerprint. The app launch PID was **75683**. Port **62644** was
reserved for its private host config. Both ports came from exclusive ephemeral
loopback reservations. The peer config existed before its startup; the original
app harness was to create a fresh Off config before model startup. That app
initialization was never observed. No app daemon PID was established.

The peer's private config accepted client certificates, as in the prior
event-delivery fixture. The planned app branch required exact server fingerprint
approval and `tls.trusted == true` before idle. Neither approval nor this trust
check ran. This record adds no TLS security acceptance.

Only **one public IPC request** is observed: the peer readiness `status.get`.
There are zero traced app requests. The app never created the config directory
required before model initialization. No steady phase or recovery request ran.
The attempt used one owned daemon process; the second daemon was never observed.

## Priority and compilation limits

The executed command wrapped the controller in `nice -n 15`, and the controller
also wrapped the compiler, app, and peer launches in `nice -n 15`. This is a
deviation: the relative increments accumulate and can saturate at nice 20.
**Actual compiler, app, and peer effective priority is UNMEASURED.** The first
sample failed before it could save the `ps` priority record. The successful
build must not be described as verified effective nice 15. This result was not
rerun to change priority. The executed controller is retained without repair.
Any later authorized run must observe parent priority, use one suitable wrapper
or an absolute priority, and sample the actual target processes.

An unrelated cargo/rustc pair was seen during initial inspection. The saved
compiler inventories immediately before and after this build are empty.
Continuous compiler overlap during the build was not measured. Runtime stopped
before a complete OS sample; it supplies no steady CPU-idle claim.

## First failure and cleanup

The controller sent SIGTERM only to its two owned process groups. The peer
logged normal goodbye and stopped with exit 0; the app launch exited -15.
No SIGKILL was needed. Both groups and both tracked PIDs were gone. Neither
private IPC socket remained. `lsof` found no entries for either assigned port,
and fresh exclusive loopback binds succeeded for both. Cleanup completed at
**0.864 seconds** from controller runtime entry; process cleanup reached
**0.735 seconds**. The failure happened within that startup interval.

[raw/runtime-first-failure.txt](raw/runtime-first-failure.txt) retains the first
traceback. [raw/cleanup.json](raw/cleanup.json) contains the exact cleanup record.
[results.json](results.json) binds the raw files and their SHA-256 values. Each
raw file was copied without transformation from the new private root.

The commands actually executed were:

```sh
nice -n 15 python3 docs/verification/macos-idle-resources-2026-10-23/measure.py
nice -n 15 python3 docs/verification/macos-idle-resources-2026-10-23/measure.py --measure /private/tmp/oe-idle-9df59d98-gw9bek1k
```

No unit test, mock, rendered/native UI, desktop automation, installed app,
production config/TLS/IPC, service, account, TCC, login, firewall, DNS, toolchain
default, TCP 24800, UDP 5353, Docker, global dependency, publication, or
Cloudflare change was used. The provider-stopped work was not resumed.
