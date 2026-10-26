# Openergy — UI ↔ daemon contract

Implementation requirements recorded on 2026-10-21. Example status objects and
process assumptions below describe the contract, not a completed first-run or
native desktop acceptance report. See [setup](setup.md),
[readiness](readiness.md), and [releasing](../releasing.md).

The macOS app is a thin client over the daemon IPC (architecture §2.10).
This file lists what the app needs. Round 2 freezes it into the `ipc` crate.

## Process model on macOS (decision)

- `Openergy.app/Contents/MacOS/Openergy` is the SwiftUI menu-bar app.
- `Openergy.app/Contents/Helpers/openergy` is the Rust daemon/CLI binary (not Contents/MacOS:
  on case-insensitive volumes `openergy` and `Openergy` are the same file name).
- On launch, the app connects to the IPC socket. If no daemon answers, the app
  spawns `openergy daemon --parent-pid <app pid>` as a child process.
  - The child inherits the app as its TCC "responsible process". The user grants
    Accessibility and Input Monitoring once, to "Openergy".
  - With `--parent-pid`, the daemon watches the parent (kqueue `EVFILT_PROC`
    `NOTE_EXIT`). When the parent exits, the daemon runs the shutdown order
    (§3.7) and exits.
  - If the daemon crashes, the app restarts it with backoff
    (0.5 s, 1 s, 2 s … 30 s cap).
- A daemon started from a terminal (`openergy daemon`) is also valid. The app
  connects to it and does not spawn a second one.
- Login: the app registers itself with `SMAppService.mainApp`. No LaunchAgent
  in v1.
- Socket: `~/Library/Application Support/Openergy/openergy.sock`. The directory
  has mode 0700, and the socket has mode 0600.
- Signing: `apps/macos/scripts/bundle.sh` signs the app and daemon with the
  selected identity. Set `OPENERGY_SIGN_IDENTITY=-` for an explicit local ad-hoc
  build. This is not Developer ID distribution or notarization; see
  [releasing](../releasing.md) for the maintained build procedure.

## Framing

`u32` big-endian length + UTF-8 JSON, as in §2.10. Requests:
`{"id":1,"method":"status.get","params":{}}`. Events after
`events.subscribe`: `{"event":"status.changed","data":{...}}`.

The first request and each unfinished request frame have a ten-second read
deadline. Additional partial bytes do not extend that deadline. Once a request
completes, the connection may stay idle for subscriptions. The next frame's read
budget starts when its bytes arrive, or after the preceding response if bytes
were already buffered. Clients must reconnect after a read-deadline closure.

The daemon sends `status.changed` with the FULL status object (not a diff),
coalesced to at most 10 per second. The app renders only from the latest status.

## `status.get` result (all fields always present; use null where unknown)

```json
{
  "daemon": { "version": "0.1.0", "pid": 1234, "uptime_s": 12, "platform": "macos" },
  "role": "off | server | client",
  "state": "idle | starting | listening | connecting | connected | reconnecting | error",
  "name": "macbook",
  "active_screen": "macbook",
  "cursor": { "screen": "macbook", "x": 100, "y": 200 },
  "local_screen": { "w": 3024, "h": 1964, "displays": [ { "x": 0, "y": 0, "w": 1512, "h": 982, "scale": 2.0 } ] },
  "server":  { "listen": "0.0.0.0:24800" },
  "client":  { "remote": "192.0.2.10:25400", "next_retry_ms": 2000, "attempt": 3 },
  "peers": [
    {
      "name": "studio-pc",
      "address": "192.0.2.10:25400",
      "role": "server | client",
      "connected": true,
      "since_unix_ms": 1792600000000,
      "rtt_ms": 0.9,
      "protocol": { "product": "Synergy", "major": 1, "minor": 8 },
      "screen": { "w": 2560, "h": 1440 },
      "tls": { "enabled": true, "version": "TLS1.3", "fingerprint": "AB:CD:…", "trusted": true },
      "secure_input_app": null
    }
  ],
  "tls": { "enabled": true, "fingerprint": "12:34:…" },
  "permissions": {
    "accessibility": "granted | denied | unknown | not_required",
    "input_monitoring": "granted | denied | unknown | not_required",
    "post_events": "granted | denied | unknown | not_required"
  },
  "secure_input": { "local_app": null },
  "clipboard": {
    "enabled": true,
    "last_transfer": { "direction": "in | out", "bytes": 12345, "formats": ["text"], "unix_ms": 0 },
    "last_blocked": { "bytes": 6500000, "limit": 4096000, "peer": "studio-pc", "unix_ms": 0 }
  },
  "pending_trust": [ { "fingerprint": "…", "address": "…", "name": null, "expires_unix_ms": 0 } ],
  "last_error": { "code": "connect_refused", "message": "Connection refused by 192.0.2.10:25400", "unix_ms": 0 },
  "stats": { "rss_bytes": 9000000, "events_in": 0, "events_out": 0 }
}
```

`role = off` means the daemon runs but does not share. The app's Off / Host /
Join switch maps to `off` / `server` / `client`.

## Config fields the UI edits (`config.get` / `config.patch`)

Add `config.patch {patch: <JSON merge patch over the config JSON>}`. The daemon
validates, writes the TOML atomically, applies it live, and returns
`{config, warnings[]}`. The app never writes TOML text itself. Keep
`config.set` for the CLI and import.

UI-required fields (TOML section names in brackets):

- `role`, `name`
- `[server] port` (and `listen` address)
- `[client] remote` (host or host:port), `autoconnect`
- `[tls] enabled`
- `[clipboard] enabled`, `max_bytes`
- `[input] relative_mouse`, `invert_scroll_y`, `scroll_sensitivity`,
  `sync_language`, `prevent_sleep`
- `[switch] delay_ms`, `double_tap_ms`, `dead_corners` (subset of
  `top_left|top_right|bottom_left|bottom_right`), `corner_size_px`
- `[hotkeys]` lock_to_screen (keystroke string like `ctrl+alt+l`), optional
- `[[screen]]` items for the server layout:
  - `name`
  - `rect = { x, y, w, h }`: position and size on the layout canvas, in layout
    units. The app keeps each screen's real aspect ratio.
  - optional per-screen `modifiers` remap and `os` hint
  - `links` are DERIVED from rect adjacency (touching edges, with overlap
    ranges giving the percentage ranges, as Synergy's converter does). An
    explicit `links` table, when present, overrides derivation for that
    screen. The daemon exposes the derived links in `config.get` as
    `derived_links` so the UI can draw them.
- `[macos] click_numbering = "native | legacy"`: default `native` (numbered
  synthetic clicks, see docs/focus-fix.md). `legacy` exists only to reproduce
  the old bug for comparison.

## Other methods the UI calls

`connection.connect {host?, port?}`, `connection.disconnect`,
`trust.approve/reject/revoke {fingerprint}`, `trust.list`, `discovery.list`,
`permissions.request {kind}`, `permissions.open_settings {kind}` (optional;
the app can open the deep links itself), `logs.tail {lines}`, `daemon.stop`,
`config.import_synergy {path}` (UI offers "Import from Synergy").

## Event names the UI handles

`status.changed` (full status), `peer.untrusted` (also in
`status.pending_trust`), `clipboard.blocked`, `log.line` (only while the
Activity view is open: `logs.subscribe` / `logs.unsubscribe`).
