# Openergy interoperability lab

Real third-party protocol peers run in Docker on an Xvfb display. Use this lab
to connect an Openergy **virtual** platform daemon to real Deskflow and Barrier
processes. The reference peers never run on the Mac desktop. All mouse,
keyboard, and clipboard commands below act on the container's `DISPLAY=:99`.

## Pinned software

| Component | Version / source | Wire protocol |
| --- | --- | --- |
| Base | Debian 13.1 slim, multi-arch image digest `sha256:a347fd7510ee31a84387619a492ad6c8eb0af2f2682b916ff3e643eb076f925a` | — |
| Deskflow | **1.27.0**, official Debian trixie arm64 package (`1.27.0.0` in dpkg) | `Synergy` **1.8** in this lab; optional `Barrier` **1.8** |
| Barrier | **2.4.0**, Debian bookworm package **2.4.0+dfsg-3**, unmodified `barriers` / `barrierc` binaries | `Barrier` **1.6** |

Deskflow 1.27.0 was the latest stable release on 2026-10-21. The continuous
release is a prerelease and is not used. Native arm64 runs under OrbStack;
no emulation or Flatpak is needed. The Dockerfile also selects the official
x86_64 package on amd64. Only arm64 was exercised here.

Official package SHA-256:

```text
arm64  4780856cca83dc8bcead0495584301d4f83f87f53341bdf447f0915b7b15c596
amd64  1f7de2f1fbc68de410339dbdb11ed0e5439bd49bfbfbf66f4467def1c6bb5f96
```

The build checks that checksum. Barrier comes from a signed Debian repository
with an exact package version. Only its headless binaries and copyright file
are copied into the final image, avoiding a second Qt GUI stack. Input Leap
and Barrier are absent from Debian trixie's package index; bookworm supplies
the secondary peer. Auxiliary X11/OpenSSL packages receive Debian security
updates at build time; those dependencies are not a full snapshot lock.

Sources: [Deskflow release](https://github.com/deskflow/deskflow/releases/tag/v1.27.0),
[Deskflow CLI](https://github.com/deskflow/deskflow/blob/v1.27.0/src/apps/deskflow-core/CoreArgs.h),
[Deskflow settings](https://github.com/deskflow/deskflow/blob/v1.27.0/src/lib/common/Settings.h),
[Deskflow protocol](https://github.com/deskflow/deskflow/blob/v1.27.0/src/lib/deskflow/ProtocolTypes.h),
[Debian Barrier package](https://packages.debian.org/bookworm/barrier),
[Barrier trust paths](https://github.com/debauchee/barrier/blob/v2.4.0/src/lib/common/DataDirectories_static.cpp).

## Start and stop

Run commands from this directory. Docker must be running. No host packages
are installed. Python and OpenSSL already on the host are used only by the
optional smoke runner for network probes.

```bash
./lab.sh build                 # openergy-interop:latest
./lab.sh up                    # openergy-interop; Xvfb :99, 1920x1080x24
./lab.sh status
./lab.sh down                  # stop; keep the image, container, and state
```

`up` publishes `127.0.0.1:24900-24910` by default. It never publishes the
production port. `LAB_PORT_RANGE` selects another range within 24900–65535
(at most 101 ports). Stop the container before changing the range or image.
`up` then recreates the container if needed. Repeated build/up/down/stop and
unchanged peer start commands are safe to repeat. An unchanged running peer
keeps its PID and connections. One peer owns this X display at a time;
starting a different peer or configuration stops the previous one.

```bash
./lab.sh down
LAB_PORT_RANGE=24900-24900 ./lab.sh up
```

Use this narrower range when a server or dummy listener on the Mac must bind
`0.0.0.0:24901`. A Docker publication on host 24901 conflicts with such a
listener. Alternatively choose a host server port outside the published range,
for example 24911. OrbStack resolves `host.docker.internal` to the Mac host;
the client smoke verifies this path with real traffic. `host.orb.internal`
is also accepted as a CLI argument, but only `host.docker.internal` was tested.

Logs, generated configs, trust files, and local certificate/private-key PEMs
live in ignored `.state/`. Only that folder is mounted from the host. There
is no host display, input device, Docker socket, or desktop clipboard mount.
The X server has no TCP listener. Peer server ports must be in the published
range. Client targets must use one of the two host aliases and a port at least
24900. The CLI rejects production port 24800 and arbitrary network targets.

## Server commands

```bash
./lab.sh up
./lab.sh server deskflow --port 24900 --screens 'lab,openergy-mac' --tls off
./lab.sh server barrier --port 24902 --screens 'lab,openergy-mac' --tls off

# Client screen above or below lab. right is the default; left is also supported.
./lab.sh server deskflow --port 24900 --screens openergy-mac --position top
./lab.sh server deskflow --port 24900 --client-name openergy-mac --position bottom

# Deskflow can also use its Barrier-name dialect, still protocol 1.8.
./lab.sh server deskflow --port 24900 --protocol barrier --tls off
```

`--screens` accepts `lab,CLIENT`, `lab CLIENT`, just `CLIENT`, or the literal
contents of a complete classic server config. Default: `lab,openergy-mac`.
The generated layout puts `lab` on the left and `CLIENT` on the right, with
links in both directions. `--position` is the client's position relative to
`lab`. Names use letters, digits, dots, underscores, and hyphens.

```bash
./lab.sh server deskflow --port 24900 --screens "$(cat my-layout.conf)" --tls off
```

Deskflow 1.27.0's CLI is:

```text
deskflow-core server --settings /lab/deskflow/settings.ini --new-instance
deskflow-core client --settings /lab/deskflow/settings.ini --new-instance
```

It does **not** accept the old `-c`, `--name`, or `--disable-crypto` CLI.
The lab writes `[core] computerName/display/interface/port`, `[client]
remoteHost`, `[server] externalConfig/externalConfigFile/protocol`,
`[security] tlsEnabled/checkPeerFingerprints/certificate`, and `[log]`
settings. Computer declarations are `[computer_lab] name=lab`, etc.
Deskflow 1.27 ignores `section: screens` in its external config. The lab
converts names from that section to INI groups and writes the remaining
links/options to `links.conf`. Per-screen legacy options are not converted;
use simple screen declarations in custom configs. Log level is `VERBOSE`,
which includes debug data, with file logging enabled.

Barrier uses `barriers --no-daemon --debug DEBUG2 --name lab --display :99
--profile-dir /lab/barrier --address 0.0.0.0:PORT --config ...` and
`barrierc` with the same common options plus its server address.
`--disable-crypto` selects plain TCP. The lab uses no GUI or daemon service.

## TLS and trust in both directions

```bash
./lab.sh server deskflow --port 24900 --tls on
./lab.sh fingerprint deskflow
openssl s_client -4 -tls1_2 -connect localhost:24900 </dev/null

./lab.sh server barrier --port 24900 --tls on
./lab.sh fingerprint barrier
openssl s_client -4 -tls1_2 -connect localhost:24900 </dev/null
```

Certificates are self-signed RSA-2048 identities generated with OpenSSL
inside the container. They persist across up/down. `fingerprint` emits exactly
two lines: `v2:sha256:<64 lowercase hex>` and uppercase colon-separated hex.
The digest covers the DER certificate. OpenSSL's self-signed verification
notice is expected; use the SHA-256 pin as the identity check.

| Peer | Combined private key + certificate | Trusted servers | Trusted clients |
| --- | --- | --- | --- |
| Deskflow | `.state/deskflow/tls/deskflow.pem` | `.state/deskflow/tls/trusted-servers` | `.state/deskflow/tls/trusted-clients` |
| Barrier | `.state/barrier/SSL/Barrier.pem` | `.state/barrier/SSL/Fingerprints/TrustedServers.txt` | `.state/barrier/SSL/Fingerprints/TrustedClients.txt` |

Trust files contain `v2:sha256:<hex>` lines. Both peers support SHA-256 pins.
Each new launch replaces the applicable trust file with the requested pin
or an empty file. A TLS client with no pin starts but refuses an unknown
server; there is no interactive trust dialog. A TLS server without
`--trust-fingerprint` encrypts the session without requiring a client pin.
To require and pin a client certificate:

```bash
./lab.sh server deskflow --port 24900 --tls on --trust-fingerprint "$OPENERGY_SHA256"
./lab.sh server barrier --port 24900 --tls on --trust-fingerprint "$OPENERGY_SHA256"
```

This enables Deskflow's `checkPeerFingerprints`; for Barrier the lab omits
`--disable-client-cert-checking`. Supply Openergy's client identity fingerprint,
and configure Openergy to present that certificate.

## Client commands

Start an Openergy **virtual** server on a host interface reachable through
OrbStack, using a dedicated config, socket, and port. Then:

```bash
./lab.sh down
LAB_PORT_RANGE=24900-24900 ./lab.sh up
./lab.sh client deskflow --server host.docker.internal:24901 --name lab --tls off
./lab.sh client barrier --server host.docker.internal:24901 --name lab --tls off

./lab.sh client deskflow --server host.docker.internal:24901 --name lab \
  --tls on --trust-fingerprint "$OPENERGY_SHA256"
./lab.sh client barrier --server host.docker.internal:24901 --name lab \
  --tls on --trust-fingerprint "$OPENERGY_SHA256"
```

The trust argument accepts plain hex, colon hex, or the complete
`v2:sha256:` line. If Openergy requires client trust, add the container peer's
`fingerprint` first line to Openergy's dedicated `tls/trusted-clients` file.
Use the same peer certificate for this client run. For Barrier, Openergy must
send a `Barrier` 1.6-compatible server hello. A Barrier 2.4 client rejects a
`Synergy` hello; it does not negotiate the name.

## Drive and observe the virtual desktop

```bash
./lab.sh events start
./lab.sh drive mousemove 500 500
./lab.sh pointer
./lab.sh drive click 1
./lab.sh drive type --clearmodifiers 'Text inside Xvfb'
./lab.sh drive mousemove 1919 540 sleep .2 mousemove_relative -- 20 0
                                    # enter a connected right client
./lab.sh clipboard set 'Clipboard text inside Xvfb'
./lab.sh clipboard get
./lab.sh events stop
./lab.sh events dump
./lab.sh logs deskflow
./lab.sh logs barrier
./lab.sh stop deskflow
./lab.sh stop barrier
./lab.sh down
```

`events start` runs line-buffered `xinput test-xi2 --root`. Repeating start
keeps the existing recorder. A new recorder replaces `.state/events.log`;
dump prints it. The log includes XTEST device IDs, Motion/RawMotion,
ButtonPress, and KeyPress events. The clipboard uses the X11 CLIPBOARD
selection, not PRIMARY; `set` keeps an xclip owner alive. Absolute xdotool
pointer warps alone did not trigger the servers' raw-motion capture. The
relative motion at the end of the edge example does; both real servers sent
`CINN` to a connected wire probe. Empty `get` before
any selection owner exists returns xclip's nonzero exit code.

`logs` works while stopped. Other peer/X commands require `up`.
`status` also works before the first start or after `down`.

## Point Openergy at the lab

Use the repository's planned CLI with a separate lab config, explicitly
selecting `--platform virtual` (never `auto` or `macos`). For example, once
the daemon implementation is available:

```bash
openergy daemon --platform virtual --config "$PWD/.state/openergy-client.toml" \
  --socket "$PWD/.state/openergy-client.sock" --foreground
```

Configure that file as role `client`, name `openergy-mac`, remote
`127.0.0.1:24900`, with TLS off for the first pass. Start the lab Deskflow
server with `--screens lab,openergy-mac --tls off`. Drive the container pointer
to its right edge, then type/click. Inspect Openergy's virtual input ledger
through its `virtual.recorded` IPC surface. Neither daemon instance may use
the user's normal config or IPC socket.

For TLS, set Openergy's `tls.enabled=true` and add the lab server's fingerprint
line to the dedicated config directory's `tls/trusted-servers`. The lab server
does not require a client pin by default. For mutual trust, also supply
Openergy's fingerprint to `server ... --trust-fingerprint`.

For the reverse direction, run Openergy with role `server`, name
`openergy-mac`, listen address `0.0.0.0`, port `24901`, virtual platform, and
a layout containing `lab`. Use the narrow publication range above. Start
the lab client; generate Openergy input through `virtual.inject` and observe
`events dump`, `pointer`, and `clipboard get` inside Docker. Select the wire
hello dialect/version required by the peer.

The Openergy CLI/IPC examples follow `docs/plan/architecture-r2.md`. They are
integration instructions, **not evidence of a completed Openergy daemon run**.
The daemon was still being implemented in parallel during lab verification;
no Openergy daemon-to-peer result is claimed here.

## Real smoke verification

```bash
./lab.sh build
python3 -I smoke.py
./lab.sh status                 # running=false
```

`smoke.py` exercises real processes through `lab.sh`, host TCP/TLS sockets,
and X11. It writes `verification.json` and always calls `down` after an active
run, including on failure. It does not import project internals, mock a peer
binary, post native host input, or add unit tests. Its small raw-wire listener
is an explicit protocol probe, not an Openergy substitute.

Captured server hellos (includes the four-byte big-endian frame length):

```text
Deskflow  Synergy 1.8  0000000b53796e6572677900010008
Barrier   Barrier 1.6  0000000b4261727269657200010006
```

`0000000b` = 11 payload bytes; next 7 bytes are the ASCII protocol name;
`0001` / `0008` or `0006` are the big-endian u16 major/minor numbers.
TLS carries the same framed hello after the handshake.

Captured client hello-back for name `lab`:

```text
Deskflow  0000001253796e6572677900010008000000036c6162
Barrier   000000124261727269657200010006000000036c6162
```

The 18-byte payload adds a u32 byte count (`00000003`) and `lab` (`6c6162`).
These are actual received bytes, not just version output. See `verification.json`
for the final result, TLS fingerprints, screen-info frames, and X event counts.

The final arm64 smoke run passed on 2026-10-21. Both clients completed plain
and TLS hello-back/info exchange through `host.docker.internal`; wire mouse
messages moved the real X pointer to `321,234` and injected clicks. Both servers
sent `CINN` after a scripted right-edge crossing. The recorder captured 49
motion, 15 button-press, and 36 key-press records (includes raw/cooked events,
so these counts are not physical action counts). UTF-8 clipboard set/get and
wrong-pin rejection passed. The final read-back was `running=false`.
`layout-verification.json` also records successful top/bottom layouts and
conversion of a literal classic config with a custom client name.

Certificate fingerprints from that run (these are disposable lab identities;
a fresh `.state/` generates different certificates):

```text
Deskflow
v2:sha256:9b5f52a65a9aebbbaa146c133e0abcb22b036b08ccd28a04fcc39cbc2525fdd1
9B:5F:52:A6:5A:9A:EB:BB:AA:14:6C:13:3E:0A:BC:B2:2B:03:6B:08:CC:D2:8A:04:FC:C3:9C:BC:25:25:FD:D1

Barrier
v2:sha256:77cf91e534bca5f603815ac8f21256b8894304856b8c17277f8a7e6aa117d3e0
77:CF:91:E5:34:BC:A5:F6:03:81:5A:C8:F2:12:56:B8:89:43:04:85:6B:8C:17:27:7F:8A:7E:6A:A1:17:D3:E0
```

## Limits

- The lab verifies reference peers and the host network path. The full
  Openergy-to-peer run remains for integration with the Rust virtual daemon.
- Barrier is a secondary 1.6 peer, not a Synergy 1.8 implementation. Input Leap
  is not installed. Wayland, native macOS input, image/file clipboard transfer,
  and multiple physical monitors are outside this lab.
- One shared Xvfb display means one active reference peer. A real app/window
  may be needed to consume typed text; the recorder still observes key events
  on the root display. A container-local WM is not installed.
- The smoke uses explicit IPv4 and TLS 1.2 for the localhost OpenSSL probe.
  This matches the loopback publication. Unconstrained TLS 1.3 probes were
  not reliable in this host's LibreSSL/reference-peer combination.
- Short hello-only TLS probes showed sporadic stalled TLS accepts in the
  reference peers. The exact cause is not established. The smoke delays the
  Python ClientHello by 100 ms, restarts peers between independent TLS probes,
  and permits up to three bounded OpenSSL attempts. `openssl_attempts` records
  how many were needed. This is not long-running or reconnect reliability proof.
- Barrier can leave a wrong-pin connection open after logging its rejection.
  The negative check requires that trust-error log and no protocol hello-back;
  it does not require immediate TCP close.
- Restarting the container resets X selection ownership, pointer state, and
  recorders. Configs, identity PEMs, and logs persist. Keep `.state/` private;
  it contains disposable lab private keys.
