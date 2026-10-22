# Windows daemon verification — 2026-10-21

Windows 11 x64 MSVC daemon and CLI control work through private byte pipes.
Real TLS peers passed input, clipboard, and disconnect-release checks in both
network roles. All native desktop acceptance remains pending.

No unit tests were added. `source.rs` and `sink.rs` are unchanged. Synergy
processes, services, and files were not changed. No test used TCP 24800.
All host test TCP endpoints used 127.0.0.1 ports 24820–24829 and SSH forwarding.
Only the virtual backend ran on the Mac. All Windows processes used SSH session 0.

## Implementation commits

- `8de7dcb` — secure Windows pipes, common daemon loop, config paths and lifecycle.
- `3c327b9` — bounded client shutdown, `ctl status` alias, Windows workspace warnings.
- `bf1aa43` — clear inherited Ctrl+C-ignore before registering the console handler.
- `77306b2` — recycle pending pipe instances when a client closes during accept.

The final documentation commit adds this evidence and replay drivers.

## Build commands and results

Mac Rust: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release -p openergy-e2e
```

All passed, exit 0, zero warnings. Final logs ended with `Finished release
profile [optimized]` and `Finished dev profile [unoptimized + debuginfo]`.

The installed Windows stable was 1.98.0. The workspace E2E package requires
1.99. The already-installed nightly is `rustc 1.99.0-nightly (969b803cb
2026-08-09)`; no toolchain was installed or updated. With that toolchain:

```sh
ssh windows 'New-Item -ItemType Directory -Path "$env:USERPROFILE\openergy-build" -Force | Out-Null'
git archive HEAD | ssh windows 'tar -x -C "$env:USERPROFILE\openergy-build"'
ssh windows 'Set-Location "$env:USERPROFILE\openergy-build"; $env:PATH="$env:USERPROFILE\.cargo\bin;"+$env:PATH; $env:RUSTUP_TOOLCHAIN="nightly"; cargo build --release --locked; cargo clippy --workspace -- -D warnings'
```

Both passed, exit 0, zero warnings. Final Windows release build: 37.19 s;
workspace clippy: 2.42 s. An optional Mac-to-MSVC cross check could not build
`ring`: the Windows C SDK was absent (`fatal error: 'assert.h' file not
found`). The native MSVC build and clippy above establish Windows compilation.

## Full Mac catalogue

The ordinary catalogue requires lab ports >=24900. A disposable source copy
changed only its three port guards and selected its copied container script
instead of the image's older guard. Product and checked-in harness behavior
were unchanged. The installed `openergy-interop:latest` image was reused.

Actual run:

```sh
CARGO_TARGET_DIR="$PWD/target" cargo build --release -p openergy-e2e \
  --manifest-path /tmp/oe-win-port-suite/Cargo.toml
OPENERGY_BIN="$PWD/target/release/openergy" target/release/openergy-e2e run \
  --port-range 24820-24824 --lab-port-range 24825-24829 \
  --report /tmp/openergy-windows-port-suite.json
```

**73/73 passed; 0 failed.** `S13-flood` passed on the first run (no rerun).
S15: server/client RSS 9,355,264 / 9,338,880 bytes; 60 s CPU 0.0333% /
0.0167%; 1,000 external inject-to-ledger samples, p99 **1.044 ms**, max
1.277 ms (limit 5 ms).

To reproduce the confined harness, use `prepare-suite.py "$PWD"
/tmp/oe-win-port-suite`, build with the manifest above, and set
`OPENERGY_BIN=/tmp/oe-win-port-suite/openergy`. The script refuses an existing
destination. It keeps all child executables and state in the disposable lab.

## Windows real control path

The first SSH session launched `openergy.exe daemon --platform virtual
--config <lab>\control\config.toml` and waited for its recorded process.
A second SSH session used the same isolated `%APPDATA%` and ran:

```text
openergy.exe status                         PASS: state=idle, role=off
openergy.exe ctl status.get --raw            PASS: exit 0
openergy.exe disconnect                     PASS: exit 0
openergy.exe connect                        PASS: exit 0
openergy.exe daemon --platform virtual ...   exit 1: another daemon owns <pipe>
openergy.exe ctl daemon.stop --raw          PASS: daemon exits 0
```

`ctl status` also passes. The committed drivers replay the expanded checks:

```sh
scp docs/verification/windows-daemon-2026-10-21/verify-control.ps1 windows:openergy-build/verify-control.ps1
scp docs/verification/windows-daemon-2026-10-21/pipe-errors.ps1 windows:openergy-build/pipe-errors.ps1
ssh windows '& "$env:USERPROFILE\openergy-build\verify-control.ps1"'
ssh windows '& "$env:USERPROFILE\openergy-build\pipe-errors.ps1"'
```

Key output:

```text
PASS config patch saves and applies
PASS second daemon rejected
PASS kernel DACL: protected, one allow ACE, current user SID only
PASS event bus subscription delivery
PASS logs subscription delivery
PASS fragmented frame survives read timeouts
PASS unknown method returns typed error
PASS 65th client rejected
PASS client slot recovers
PASS ipc-stop exit=0
PASS absent daemon error
PASS watched daemon stays alive with parent
PASS parent-watched exit=0
PASS daemon shares private session-0 console
PASS Ctrl+C sent to private session-0 console
PASS ctrl-c exit=0
PASS CLI read timeout about 10 seconds
WINDOWS_CONTROL_PASS
PASS 100 clients connect and close before a request
PASS oversized frame closes only that client
ERROR_CHECK_EXIT=0
PASS busy pipe fails after about 1 second
```

The DACL check called `GetSecurityInfo` on an open kernel pipe handle, converted
the descriptor, and checked DACL protection, exactly one allow ACE, and SID
equality with `WindowsIdentity.GetCurrent().User`. No SID was printed. A login
as another user was not attempted. `PIPE_REJECT_REMOTE_CLIENTS` is set in each
`CreateNamedPipeW` call; remote SMB access was not exercised.

Ctrl+C was generated only in a new private session-0 console. Its process list
included the owned daemon. The initial failure showed the inherited ignore flag;
clearing it fixed the same real console scenario. Console-close dispatch,
logoff, and OS shutdown were not separately exercised.

The rapid-close check first exposed OS error 232 (`The pipe is being closed.`).
The listener now creates another instance before closing that pending instance.
All expanded control checks pass with this fix. Shutdown also passes with a
client holding an incomplete frame. No synthetic input or desktop clipboard
API was used in these control scenarios.

Offline Windows trust CLI also passed: `trust list` returned two role records;
`trust revoke` persisted their removal; `trust approve` persisted both again;
`trust import-synergy` imported from a synthetic lab fixture. The real Synergy
configuration was never opened or changed.

## TLS network lab

Both host processes used `--platform virtual`, separate configs and identities.
The Windows `server`/`client` aliases and Mac aliases were used directly.
Windows launching SSH sessions stayed open until daemon exit; short CLI requests
ran in separate sessions. The trust CLI approved each peer's lab fingerprint
without printing it. Status confirmed TLS enabled and `trusted=true` at both ends.

```sh
mkdir -p /tmp/oe-win-port-net
cp target/release/openergy /tmp/oe-win-port-net/openergy
ssh -o ExitOnForwardFailure=yes -o ServerAliveInterval=10 \
  -L 127.0.0.1:24820:127.0.0.1:24820 \
  -R 127.0.0.1:24822:127.0.0.1:24822 windows \
  '"TUNNEL_READY"; while(!(Test-Path "$env:USERPROFILE\openergy-build\lab\tunnel.stop")){Start-Sleep -Milliseconds 200}'
# In another shell while the tunnel is open:
python3 docs/verification/windows-daemon-2026-10-21/network.py
# Close the owned tunnel naturally:
ssh windows 'New-Item -ItemType File -Force "$env:USERPROFILE\openergy-build\lab\tunnel.stop" | Out-Null'
```

Windows server 127.0.0.1:24820 → Mac client via `-L`: **PASS**.
Mac server 127.0.0.1:24822 → Windows client via `-R`: **PASS**.
Negotiated TLS was 1.3 with AES-256-GCM-SHA384.

Each direction checks key down/up, mouse down/up, absolute and relative move
input, and wheel delivery in the receiving virtual ledger. The edge-crossing
move is first in the injected batch. Clipboard data uses
`virtual.clipboard {set:{formats:[{kind:"text_utf8",bytes:[...]}]}}` and
`{get:true}`; returned base64 is decoded before comparing exact bytes. Clipboard
transfer passes in both directions for each pair. Each client receives a held
key and mouse button, then disconnects; both pressed ledgers become empty.
Final marker: **`NETWORK_PASS`**. All four owned daemon processes stop cleanly.

## Native session-0 start

```text
openergy.exe server --platform windows --config <lab>\native\config.toml --socket <lab>\native\ctl.sock
NATIVE pid=26160 session=0
NATIVE platform=windows state=listening
NATIVE_EXIT=0
INFO openergy::runtime: listening on 127.0.0.1:24824
INFO openergy::runtime: shutdown: leave and goodbye
INFO openergy::runtime: daemon stopped
```

No initialization or hook error was logged. This proves startup and shutdown,
not real input/clipboard behavior. No interactive session was used.

## Input-source risks (review only)

- Suppression depends on the hook thread's `captured` flag and the asynchronous
  remote-state callback. Commands and callback decisions can briefly disagree
  during transitions. Queue pressure can pass an event to Windows while the
  pointer is still clipped; real transition behavior remains unverified.
- `grab(false)` returns immediately if restoring `ClipCursor` fails, before
  cursor visibility and position are restored. The worker ignores that command
  error. Disconnect cleanup can therefore leave capture state or cursor state
  behind if a Win32 restoration call fails.
- Windows can silently remove low-level hooks after `LowLevelHooksTimeout`.
  There is no health check or hook reinstall path. Raw input and clipping can
  outlive the suppression hook. Hook callbacks also call translation and cursor
  APIs and can log errors synchronously, so timeout behavior needs real testing.
- `Source::command` waits for a hook-thread reply without a deadline, and
  `MessageThread::drop` joins without a deadline. A stalled message thread can
  block disconnect or process shutdown. None of these desktop failure paths
  was exercised or changed in this port.

Microsoft documents silent hook removal after a timeout in
[LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc).
The other risks above come from the current source and worker command paths.

## Retained artifact and cleanup

Windows binary: `%USERPROFILE%\openergy-build\openergy.exe`, identical to
`target\release\openergy.exe`, **4,041,728 bytes**.

SHA-256: `250a497ba225a5d136314d08aab49cd36b0ed29637c18118cea3588e300a1ed4`.

The lab daemons and tunnel are stopped. Disposable Windows lab config/state,
identities, trust files, and Mac /tmp labs are removed. The two Windows binary
copies are retained for the orchestrator. Docker catalogue containers are gone.
No changes were pushed and no PR was opened.
