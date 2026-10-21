# WP2 verification

Date: 2026-10-21. Branch: `t3/synergy-config-import-export`.
Original implementation commit: `dc2015ff8f0d9b1b8b7903a1aa6c4f38dbd9aa12`.
This evidence was refreshed for the native-layout architecture decision.
The follow-up starts from `ebb2067068f40f9794aba94320af39d6a8b6bf32`.
All committed edits are under `crates/config/`. No new dependencies.

## Release gates

```text
$ export PATH="$HOME/.cargo/bin:$PATH"
$ cargo fmt -p openergy-config -- --check
(no output; exit 0)
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 15.94s
$ cargo clippy --release -p openergy-config -- -D warnings
    Finished `release` profile [optimized] target(s) in 0.11s
```

Whole-workspace release build: zero warnings. Config Clippy: zero warnings.
No unit tests, Rust test modules, mocks or reference-source code were added.

## Real CLI and daemon run

`verify.py` only executes real binaries and compares their outputs. It runs a
real virtual daemon, uses its real Unix socket, stops that child through IPC,
and then reads its persisted TOML through a fresh CLI process.

```text
$ python3 crates/config/verify.py
PASS: CLI import/export round trips; JSON and source precedence; strict failures; native edge geometry, Synergy editor fidelity, L layout and ranged overrides; invalid patches preserve file and live state.
invalid patch sha256 before/after: 6f9b817eda101d6284f2494a7f9cc41294f4f6fd42d2251ee41200609790eb63 / 6f9b817eda101d6284f2494a7f9cc41294f4f6fd42d2251ee41200609790eb63
Real CLI invocations: 83
Evidence: /tmp/oe-wp2/verification.log
```

Both fixture exports are byte-identical after reimport and reexport. This
compares the representable Synergy core view, not local Openergy fields that
Synergy's language cannot carry. The script also checks editor-only link
ranges, local.json server/client role selection without INI, JSON corner
insertion order, aliases, protocol=barrier, clipboard KiB conversion, mouse
hotkeys, fractional range warnings, negative switch-timer handling, default
import-directory resolution, and output temp-file cleanup on write failure.

## Fixture import output and resulting TOML

These fixtures contain only synthetic names and documentation addresses.
The command output below is complete; stdout is shown before stderr.

```text
$ target/release/openergy import-synergy --from crates/config/fixtures/synergy3 --out /tmp/oe-wp2/config.toml
Imported 2 screens, 2 directed links; role client; 1 trust candidates.
Wrote /tmp/oe-wp2/config.toml
warning: Trust candidates returned in ImportReport.trusted_servers; approve them through the trust store before connecting
warning: core INI disables peer fingerprint checking; Openergy retains require_client_trust=true
warning: data.computers[].fix.{sync_language,invert_scroll,prevent_sleep} and scroll_sensitivity: local values map to global input fields when identity is available; remote per-screen values have no global mapping
warning: data.computers[].hostname: no Openergy mapping
warning: data.computers[].misc.screenCount: no Openergy mapping
warning: data.developers.dragAndDrop: no Openergy mapping
warning: data.hotkeys[].actions[]: action has no Openergy mapping
warning: data.performance.sync: no Synergy core effect; clipboard send and receive remain enabled
warning: data.security.key_length: no Openergy mapping
warning: db.json rects retain 622x440 editor layout units; physical display sizes are unavailable; imported links are explicit service ranges
warning: synergy.conf supplies authoritative screens and links; matching db layout metadata was retained
warning: tls/trusted-servers line 2: unsupported or malformed pin record ignored
```

`/tmp/oe-wp2/config.toml`:

```toml
version = 1
role = "client"
name = "mac-book"

[options]
barrier = false
win32_keep_foreground = true
disable_lock_to_screen = false
default_lock_to_screen = false
switch_needs_shift = false
switch_needs_control = false
switch_needs_alt = false

[server]
listen = "0.0.0.0:25020"
port = 25020
heartbeat_ms = 3000

[client]
remote = "192.0.2.10"
autoconnect = true
heartbeat_ms = 3000

[tls]
enabled = true
require_client_trust = true

[clipboard]
enabled = true
send = true
receive = true
max_bytes = 4096000

[input]
relative_mouse = false
invert_scroll_y = true
scroll_sensitivity = 1.25
sync_language = false
prevent_sleep = false

[switch]
delay_ms = 100
double_tap_ms = 250
dead_corners = []
corner_size_px = 0

[hotkeys]
lock_to_screen = "ctrl+alt+l"

[macos]
click_numbering = "native"

[[screen]]
name = "studio-pc"
os = "windows"
aliases = []

[screen.rect]
x = 201.0
y = 0.0
w = 200.0
h = 110.0

[screen.links]
left = ""
right = ""
top = ""
bottom = ""

[[screen.links.ranges]]
edge = "left"
from_range = [
    23,
    91,
]
to = "mac-book"
to_range = [
    0,
    100,
]

[screen.modifiers]
shift = "shift"
ctrl = "ctrl"
alt = "alt"
meta = "meta"
super_ = "super"

[screen.options]
half_duplex_caps_lock = false
half_duplex_num_lock = false
half_duplex_scroll_lock = false
dead_corners = ["top_left"]
corner_size_px = 8
xtest_is_xinerama_unaware = false

[[screen]]
name = "mac-book"
os = "macos"
aliases = []

[screen.rect]
x = 0.0
y = 25.0
w = 200.0
h = 75.0

[screen.links]
left = ""
right = ""
top = ""
bottom = ""

[[screen.links.ranges]]
edge = "right"
from_range = [
    0,
    100,
]
to = "studio-pc"
to_range = [
    23,
    91,
]

[screen.modifiers]
shift = "shift"
ctrl = "super"
alt = "alt"
meta = "meta"
super_ = "ctrl"

[screen.options]
half_duplex_caps_lock = true
half_duplex_num_lock = false
half_duplex_scroll_lock = false
dead_corners = []
xtest_is_xinerama_unaware = false
```

```text
$ target/release/openergy import-synergy --from crates/config/fixtures/synergy1 --out /tmp/oe-wp2/legacy.toml
Imported 2 screens, 2 directed links; role server; 0 trust candidates.
Wrote /tmp/oe-wp2/legacy.toml
warning: synergy.conf has no local identity: selected its first screen as server; confirm role and name
warning: synergy.conf line 47: press action switchInDirection has no Openergy mapping
warning: synergy.conf line 47: release action keyUp has no Openergy mapping
warning: synergy.conf line 48: press action lockCursorToScreen has no Openergy mapping
warning: synergy.conf line 49: unsupported option "screenSaverSync"
```

`/tmp/oe-wp2/legacy.toml`:

```toml
version = 1
role = "server"
name = "studio-pc"

[options]
barrier = true
win32_keep_foreground = true
disable_lock_to_screen = false
default_lock_to_screen = false
switch_needs_shift = false
switch_needs_control = false
switch_needs_alt = false

[server]
listen = "127.0.0.1:25020"
port = 25020
heartbeat_ms = 3000

[client]
remote = ""
autoconnect = true
heartbeat_ms = 3000

[tls]
enabled = true
require_client_trust = true

[clipboard]
enabled = true
send = true
receive = true
max_bytes = 4096000

[input]
relative_mouse = true
invert_scroll_y = false
scroll_sensitivity = 1.0
sync_language = true
prevent_sleep = false

[switch]
delay_ms = 150
double_tap_ms = 250
dead_corners = ["top_left"]
corner_size_px = 8

[hotkeys]
lock_to_screen = "ctrl+alt+l"

[macos]
click_numbering = "native"

[[screen]]
name = "studio-pc"
aliases = []

[screen.links]
left = ""
right = ""
top = ""
bottom = ""

[[screen.links.ranges]]
edge = "left"
from_range = [
    20,
    80,
]
to = "macbook"
to_range = [
    0,
    100,
]

[screen.modifiers]
shift = "shift"
ctrl = "super"
alt = "alt"
altgr = "alt"
meta = "meta"
super_ = "ctrl"

[screen.options]
half_duplex_caps_lock = false
half_duplex_num_lock = true
half_duplex_scroll_lock = false
dead_corners = [
    "top_left",
    "top_right",
    "bottom_left",
]
corner_size_px = 12
xtest_is_xinerama_unaware = false
preserve_focus = true

[[screen]]
name = "macbook"
aliases = ["laptop"]

[screen.links]
left = ""
right = ""
top = ""
bottom = ""

[[screen.links.ranges]]
edge = "right"
from_range = [
    0,
    100,
]
to = "studio-pc"
to_range = [
    20,
    80,
]
```

## Export and round trip

```text
$ target/release/openergy export-synergy-conf --config /tmp/oe-wp2/config.toml
section: screens
    studio-pc:
        shift = shift
        ctrl = ctrl
        alt = alt
        meta = meta
        super = super
        halfDuplexCapsLock = false
        halfDuplexNumLock = false
        halfDuplexScrollLock = false
        xtestIsXineramaUnaware = false
        switchCorners = none +top-left
        switchCornerSize = 8
    mac-book:
        shift = shift
        ctrl = super
        alt = alt
        meta = meta
        super = ctrl
        halfDuplexCapsLock = true
        halfDuplexNumLock = false
        halfDuplexScrollLock = false
        xtestIsXineramaUnaware = false
        switchCorners = none
end

section: links
    studio-pc:
        left(23,91) = mac-book(0,100)
    mac-book:
        right(0,100) = studio-pc(23,91)
end

section: options
    protocol = synergy
    address = 0.0.0.0:25020
    heartbeat = 3000
    relativeMouseMoves = false
    clipboardSharing = true
    win32KeepForeground = true
    disableLockToScreen = false
    defaultLockToScreenState = false
    switchNeedsShift = false
    switchNeedsControl = false
    switchNeedsAlt = false
    clipboardSharingSize = 4000
    switchDelay = 100
    switchDoubleTap = 250
    switchCorners = none
    switchCornerSize = 0
    keystroke(Control+Alt+l) = lockCursorToScreen(toggle)
end
warning: synergy.conf exports screen/link/core options only; role, remote host, TLS, local input settings and physical rects are not represented
```

```text
$ target/release/openergy import-synergy --from /tmp/oe-wp2/config.conf --out /tmp/oe-wp2/config-roundtrip.toml
Imported 2 screens, 2 directed links; role server; 0 trust candidates.
Wrote /tmp/oe-wp2/config-roundtrip.toml
warning: synergy.conf has no local identity: selected its first screen as server; confirm role and name
$ cmp /tmp/oe-wp2/config.conf /tmp/oe-wp2/config-roundtrip.conf
(no output; exit 0)
$ cmp /tmp/oe-wp2/legacy.conf /tmp/oe-wp2/legacy-roundtrip.conf
(no output; exit 0)
```

The export matches the documented Synergy grammar. A Synergy/Deskflow process
was not launched to parse it: this package has no authorized native core
server run or Docker assignment. Real-peer interoperability remains a later
interop-lab gate.

## Real daemon layout and patch evidence

The daemon launch was:

```sh
target/release/openergy daemon --platform virtual --socket /tmp/oe-wp2/oe-wp2.sock --config /tmp/oe-wp2/layout.toml --foreground
target/release/openergy ctl config.get --socket /tmp/oe-wp2/oe-wp2.sock --raw
```

The private socket directory is deliberate: Step 0 applies mode 0700 to the
socket parent. A direct `/tmp/oe-wp2.sock` path would change `/tmp` permissions.
`/tmp` remained `drwxrwxrwt`. Only the virtual backend was used; no input
injection, cursor changes, clipboard writes or TCC requests occurred. The
Step 0 daemon opens no TCP listener or connector. No LAN host or port 24800
was contacted. No Synergy core or Docker container was started.

The unmodified zero-spacing partial fixture returns exactly these two
`derived_links`. The native path uses original screen points; it applies no
Synergy expansion or probes. There is no backward PC.right → Mac link.
See LAYOUT.md for the hand-computed 0–100 and 21–89 endpoints.

```json
[
  {
    "edge": "right",
    "from": "macbook",
    "from_range": [
      0,
      100
    ],
    "to": "studio-pc",
    "to_range": [
      21,
      89
    ]
  },
  {
    "edge": "left",
    "from": "studio-pc",
    "from_range": [
      21,
      89
    ],
    "to": "macbook",
    "to_range": [
      0,
      100
    ]
  }
]
```

An explicit `macbook.links.right = ""` removes only the first link. An explicit
`right = "studio-pc"` overrides that edge's partial destination range with
`[0,100]`. Both cases pass through config.patch and config.get.
Moving PC x to 1513 stays within the inclusive one-point tolerance:

```sh
/Users/serge/.t3/worktrees/openergy/t3-f6288390/target/release/openergy ctl config.patch --socket /tmp/oe-wp2/oe-wp2.sock --raw --json '{"patch":{"screen":[{"aliases":[],"name":"macbook","rect":{"h":982.0,"w":1512.0,"x":0.0,"y":300.0}},{"aliases":[],"name":"studio-pc","rect":{"h":1440.0,"w":2560.0,"x":1513,"y":0.0}}]}}'
```

```json
[
  {
    "edge": "right",
    "from": "macbook",
    "from_range": [
      0,
      100
    ],
    "to": "studio-pc",
    "to_range": [
      21,
      89
    ]
  },
  {
    "edge": "left",
    "from": "studio-pc",
    "from_range": [
      21,
      89
    ],
    "to": "macbook",
    "to_range": [
      0,
      100
    ]
  }
]
```

Moving PC x to 1513.01 produces no links. Further real daemon checks confirm:

- Exactly one point of perpendicular overlap produces no link.
- Two points of overlap on 100-point spans produce ranges `[98,100]` and
  `[0,2]` with the correct reciprocal directions.
- Two points on 10,000-point spans round to empty ranges; config.patch warns
  and config.get returns no links. The export CLI also prints the warning.
- Explicitly disconnecting both affected edges removes that rounding warning.
- Adjacent 0.25-point-wide screens produce only the facing right/left pair.
- Rects sharing only a corner produce no links.

Actual rounded-empty warning:

```text
Layout overlap rounds to an empty integer percentage range; link omitted
```

The db.json-only path still uses the separate service converter. Its normal
editor fixture retains the PC source range `[22,91]`, rather than the
`[23,91]` produced by unexpanded native geometry. A zero-spacing editor import
retains the historical extra direction as an explicit imported link:

```json
[
  {
    "edge": "left",
    "from": "studio-pc",
    "from_range": [
      22,
      91
    ],
    "to": "mac-book",
    "to_range": [
      0,
      100
    ]
  },
  {
    "edge": "right",
    "from": "studio-pc",
    "from_range": [
      22,
      91
    ],
    "to": "mac-book",
    "to_range": [
      0,
      100
    ]
  },
  {
    "edge": "right",
    "from": "mac-book",
    "from_range": [
      0,
      100
    ],
    "to": "studio-pc",
    "to_range": [
      22,
      91
    ]
  }
]
```

This confirms the two paths use different rules through the real interfaces.
The native three-screen L fixture now uses exact touching edges at x/y=1000.

The three-screen L returns:

```json
[
  {
    "edge": "right",
    "from": "studio-pc",
    "from_range": [
      0,
      100
    ],
    "to": "macbook",
    "to_range": [
      0,
      100
    ]
  },
  {
    "edge": "bottom",
    "from": "studio-pc",
    "from_range": [
      0,
      50
    ],
    "to": "linux-box",
    "to_range": [
      0,
      100
    ]
  },
  {
    "edge": "left",
    "from": "macbook",
    "from_range": [
      0,
      100
    ],
    "to": "studio-pc",
    "to_range": [
      0,
      100
    ]
  },
  {
    "edge": "top",
    "from": "linux-box",
    "from_range": [
      0,
      100
    ],
    "to": "studio-pc",
    "to_range": [
      0,
      50
    ]
  }
]
```

The script checks adjacent `[0,50)` and `[50,100)` explicit source ranges as
valid. A second range starting at 49 is rejected. It also checks RFC 7386
nested deletion/defaults, including object-on-null semantics, and the warning
that `server.port` wins over the listen-string port.

## Invalid patches and unchanged file evidence

The first invalid patch moves the PC to x=1400, overlapping the Mac. The
second adds `server.typo`. Other failures cover ports 0/70000, unknown link
targets, zero scroll sensitivity and a nonobject root patch. Each request
returns `invalid_config`; the daemon continues serving requests. The script
compares both the disk hash and full live `config.get` before and after every
failure.

```text
Overlap: invalid config: screen rects overlap: "macbook" and "studio-pc"
Unknown key: parse error: unknown field `typo`, expected one of `listen`, `port`, `heartbeat_ms`
sha256 before: 6f9b817eda101d6284f2494a7f9cc41294f4f6fd42d2251ee41200609790eb63
sha256 after:  6f9b817eda101d6284f2494a7f9cc41294f4f6fd42d2251ee41200609790eb63
```

Strict TOML also rejects `unknown_setting` under `[input]` at source line 2.
Malformed conf inputs fail through the real CLI with line diagnostics:
missing end, unknown section, invalid Boolean/range, unknown target,
overlapping ranges and duplicate aliases. Invalid imports leave an existing
output file unchanged.

```text
$ target/release/openergy ctl daemon.stop --socket /tmp/oe-wp2/oe-wp2.sock --raw
{"stopping":true}
```

The child exited and the socket was removed. Persisted config was then
exported and reimported successfully in two fresh CLI processes.

CLI import outputs and CLI export outputs are 0600:

```text
-rw------- /tmp/oe-wp2/config.toml
-rw------- /tmp/oe-wp2/legacy.toml
```

Step 0's separate daemon writer creates its replacement file as 0644 under
the default umask. WP6 should use the already-added helper:

```rust
// Existing public helper in openergy_config::cli:
pub fn write_atomic(path: &std::path::Path, text: &str)
    -> Result<(), openergy_config::ConfigError>
```

## Read-only live-data import from the initial WP2 run, redacted

The real preferences import used `--from ~/Library/Preferences/Synergy`
and `--json --out /tmp/oe-wp2/private-real-config.toml`. Stdout/stderr were
captured directly into mode 0600 files inside a 0700 directory.

Passed: 2 screens; role `client`; 2 directed links (left, right); 1 trust candidate; 31 warnings.

Computer names, IDs, addresses, fingerprints and all licensing data are
omitted here. No real-data fixture was committed. The importer did not read
certificate/private-key files, change live settings, modify trust, or make
network connections.

## Integration limits and contract follow-up

All frozen function signatures remain unchanged. Additive fields in the owned
crate carry ranged links, screen/global core options and trust candidates;
their exact types and compatibility notes are in LAYOUT.md.

The orchestrator's architecture decision is implemented. Native layouts use
facing original edges within one point, with more than one point of
perpendicular overlap. The expanded/probe service converter is separate and
is called only by the Synergy 3 db.json importer. No function signature change
was required. Explicit overrides, integer percentages and warning behavior
remain available. No layout architecture question remains open for WP2.

The daemon config.get/config.patch wiring works; derived links were verified.
WP6 owns the remaining daemon persistence-mode and socket-parent-permission
fixes. No changes outside this work package were made.
