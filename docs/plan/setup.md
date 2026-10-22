# Fresh setup and recovery

Openergy is alpha software. Read [readiness](readiness.md) and
[SECURITY.md](../../SECURITY.md) first. Keep a local keyboard and mouse available
until your exact topology works. Do not disable a working service to try an
unverified setup. No Synergy installation or import is needed for fresh setup.

## Roles and names

The **host/server** owns the physical keyboard and mouse. The **joined client**
receives input. Give each computer a unique screen name using ASCII letters,
digits, `_`, `-`, or `.`. A client's name must match a server layout entry.
Adjacent rectangles must touch; a disconnected rectangle has no crossing edge.
Layout dimensions are not a promise about physical monitor scaling.

For macOS app setup, follow [README.md](../../README.md). The rest of this guide
covers the headless CLI. Its `openergy` executable is different from the
`Openergy` macOS app. The bundled CLI is at
`dist/Openergy.app/Contents/Helpers/openergy` after a local app build.

## Build the CLI

Use the pinned Rust toolchain when present, plus Git and the native linker/SDK:

- macOS: Apple command line tools. The SwiftUI app also needs Swift 6 and a
  macOS 15+ SDK/runtime.
- Linux: a C compiler and linker (for example, the distribution's build-essential
  packages). Native desktop sharing needs a working X11 session with XI2, XTEST,
  XFixes, XKB, and RandR support. Wayland capture/control is unsupported.
- Windows: Rust's MSVC toolchain, Visual Studio C/C++ build tools, and Windows
  SDK. Run desktop sharing in the logged-in user's interactive session; SSH
  session-0 checks do not verify desktop input.

From a checkout:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_BUILD_JOBS=2 nice -n 15 cargo build --release --locked --bin openergy
```

On Windows, build from a developer PowerShell with the MSVC tools available:

```powershell
$env:CARGO_BUILD_JOBS = '2'
cargo build --release --locked --bin openergy
```

The output is `target/release/openergy` (`target\release\openergy.exe` on Windows).
This is a local build, not an installer or proof of a trusted distribution.

## Choose state paths and configuration

Use a directory owned by the account that will run the daemon. Create it before
starting. On Unix, use `umask 077` and mode `0700` for a new state directory.
On Windows, use a private user-profile directory and check its inherited ACLs.
Do not share private keys between computers.

Copy [examples/server.toml](../../examples/server.toml) to the host and
[examples/client.toml](../../examples/client.toml) to the client. They use the
names `studio-host` and `studio-client`, with the client to the right of the
host. Review and edit the copies before starting:

- Set `name` and the matching server `[[screen]]` names for your computers.
- Replace the example client's `192.0.2.10` address with your actual reachable
  host address. That documentation address is not a usable server.
- Set `server.listen`, `server.port`, and `client.remote` consistently. The
  explicit `server.port` overrides a port written in `server.listen`.
- Use an unused custom port, such as `25400`, for initial evaluation. Set it in
  both configs. A host listening on `0.0.0.0` accepts network connections on all
  IPv4 interfaces; restrict it and the firewall to your intended network.
- Keep `[tls] enabled = true` and `require_client_trust = true`. Review clipboard
  sharing before enabling a connection.

The examples use the traditional port 24800. Native daemon startup enables
that port when `OPENERGY_ALLOW_PORT_24800` is unset. Set it to `0` to keep the
guard active. Virtual backends require explicit opt-in and tests must never
opt into 24800. Prefer a custom port while another service is installed.

| System | Default config | Default control endpoint |
| --- | --- | --- |
| macOS | `~/Library/Application Support/Openergy/config.toml` | `openergy.sock` in that directory |
| Linux | `$XDG_CONFIG_HOME/openergy/config.toml`, or `~/.config/openergy/config.toml` when unset | `$XDG_RUNTIME_DIR/openergy.sock`, or `~/.local/share/openergy/openergy.sock` when unset |
| Windows | `%APPDATA%\Openergy\config.toml` | Private per-user named pipe selected from the socket path |

Pass `--config` to the daemon and `--dir` to identity/trust commands when using
an explicit location. `--config` does not change the control endpoint.

With explicit paths, the daemon puts `tls/` and `openergy.log` beside its config.
`--socket` must be supplied separately and reused by every control command.
On Windows it selects a pipe; it is not a Unix socket file and its pathname
does not itself grant access.

## Verify and approve identities

Use each computer's own state directory. Prepare a new host evaluation
directory from the repository root, then edit the copied configuration before
starting:

```sh
oe_bin="$PWD/target/release/openergy"
oe_state="$HOME/.local/share/openergy-evaluation"
umask 077
mkdir -p "$oe_state"
cp -n examples/server.toml "$oe_state/config.toml"
# On the client, copy examples/client.toml instead. Edit the copy before start.
"$oe_bin" fingerprint --dir "$oe_state" --synergy
```

`fingerprint` creates a local certificate/key if neither exists. It prints
`v2:sha256:<64 hex characters>`. A fingerprint is public identity information;
`tls/key.pem` is secret. Compare the host and client fingerprints through a
channel you already trust, such as reading the other screen yourself.

For approval before connection, run this on each computer with the **other**
computer's verified fingerprint:

```sh
oe_peer_fp='v2:sha256:REPLACE_WITH_THE_VERIFIED_64_HEX_DIGITS'
"$oe_bin" trust approve "$oe_peer_fp" --dir "$oe_state"
"$oe_bin" trust list --dir "$oe_state"
```

Offline approval stores the pin for **both roles**. For approval limited to a
pending connection's role, start the daemon, inspect `status.pending_trust`,
verify the fingerprint, then call live IPC:

```sh
"$oe_bin" ctl trust.approve --socket "$oe_state/ctl.sock" \
  --json "{\"fingerprint\":\"$oe_peer_fp\"}"
```

The app offers the same live approval. Unknown peers are held briefly; if the
request expires, let the client retry. Reject an unexpected request rather
than approving it to remove a warning. Discovery is optional; it does not
replace fingerprint verification.

## Start, inspect, and stop

On an intended Unix desktop, start the foreground daemon with the config copy:

```sh
# Choose macos on macOS, or x11 inside a Linux X11 session.
oe_backend=macos
"$oe_bin" daemon --platform "$oe_backend" \
  --config "$oe_state/config.toml" --socket "$oe_state/ctl.sock"
```

Use `--platform virtual` for a lab, as described in
[CONTRIBUTING.md](../../CONTRIBUTING.md). Native commands above are operational
setup instructions, not permission to test on an active input account.
In another terminal with the same `oe_bin` and `oe_state` variables:

```sh
"$oe_bin" status --socket "$oe_state/ctl.sock"
"$oe_bin" ctl status.get --socket "$oe_state/ctl.sock" --raw
"$oe_bin" disconnect --socket "$oe_state/ctl.sock"
"$oe_bin" connect --socket "$oe_state/ctl.sock"
"$oe_bin" ctl daemon.stop --socket "$oe_state/ctl.sock"
```

`connect` uses the configured remote. `connect HOST:25400` selects a new remote.
You can also use `ctl connection.connect --json '{"host":"HOST","port":25400}'`
with the same socket. Both forms persist the new remote and client role.

On Windows, use explicit paths in PowerShell (after copying/editing the config
and exchanging fingerprints):

```powershell
$oeExe = Join-Path $PWD 'target\release\openergy.exe'
$oeState = Join-Path $env:LOCALAPPDATA 'OpenergyEvaluation'
$oeConfig = Join-Path $oeState 'config.toml'
$oeSocket = Join-Path $oeState 'ctl.sock'
& $oeExe fingerprint --dir $oeState --synergy
& $oeExe daemon --platform windows --config $oeConfig --socket $oeSocket
```

In a second PowerShell with the same variables:

```powershell
& $oeExe status --socket $oeSocket
& $oeExe ctl daemon.stop --socket $oeSocket
```

These commands do not register a service or logon task. Native Windows desktop
acceptance is pending. UAC/secure-desktop and elevated applications can reject
ordinary desktop input; do not elevate the daemon as a general workaround.
See the [Windows backend](../../crates/platform-windows/README.md) and
[Windows daemon verification](../verification/windows-daemon-2026-10-21/README.md).

For the separate per-user installer and logon supervisor, see the
[Windows lifecycle guide](../../tools/windows/README.md). Read its current
validation limits before enabling startup. The foreground path above remains
useful for initial evaluation.

## Confirm behavior and recover

Confirm both peers report the intended names, connected state, and TLS enabled
and trusted. Check the listening address and selected port. On a separate
evaluation desktop, try harmless typing, screen crossings in both directions,
a small text clipboard transfer, then disconnect and confirm local control.
Also check your actual keyboard layout, monitor scaling, and needed clipboard
formats before relying on the setup.

If a peer does not connect, inspect `last_error`, `pending_trust`, matching
screen names, firewall rules, and each computer's `openergy.log`. An empty
Nearby list does not prevent a manual connection. An empty trust list needs
verified approval, not disabled TLS. A second daemon should be refused for an
already owned control endpoint. Do not delete that endpoint to bypass the lock.

Stop the daemon with `ctl daemon.stop` or Ctrl+C in its own terminal. For the
macOS app, choose Off to stop sharing or quit the app. Use local input to
recover if the network path fails. Preserve config and TLS state for diagnosis;
deleting an identity changes its fingerprint and requires reapproval.

For a lost or compromised peer, remove its pin through the app or
`trust revoke VERIFIED_FP --dir STATE_DIR`, disconnect, and check trust again.
For confirmed compromise, also rotate the affected identity through an agreed
recovery procedure and remove the old fingerprint on all peers.

## Optional Synergy migration

Keep the working service, its configuration, and a local recovery path. Import
to a **new output file**, inspect warnings, names, layout, addresses, TLS, and
clipboard settings, then use a separate custom port for evaluation:

```sh
"$oe_bin" import-synergy --from /path/to/synergy-settings \
  --out "$oe_state/imported.toml"
"$oe_bin" export-synergy-conf --config "$oe_state/imported.toml" \
  --out "$oe_state/exported-synergy.conf"
```

The importer reads Synergy files and writes the requested output. It does not
prove that every imported option is implemented. Review
[V1-LIMITS.md](../../crates/config/V1-LIMITS.md). The offline import does not apply
the result to a running daemon. The app's import and `config.import_synergy`
IPC change live configuration; use those only for an intended migration.

Openergy's `fingerprint --synergy` output can be exchanged with a compatible
peer's `trusted-servers` or `trusted-clients` store. Use that peer's own current
instructions for paths and pin requirements; the existing
[Synergy core report](../verification/windows-synergy-1.21.1/README.md) is scoped
to 1.21.1-beta. Do not infer support for all Synergy-family versions.

The existing [Windows switch script](../../tools/windows/switch-from-synergy.ps1)
is an optional migration tool with fixed installation/service assumptions. It
can stop Synergy, register a task, and change service/startup settings; it is
not a fresh-install prerequisite. Review it and retain recovery state before
using it. Fresh evaluation should use the foreground commands above.

For maintainer builds and distribution limits, see [releasing](../releasing.md).
