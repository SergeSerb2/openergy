# macOS platform release audit — 2026-10-21

Scope: `crates/platform-macos` only, based on parent commit `06b6128`.
No native probe, capture, event posting, cursor mutation, pasteboard access,
permission query/request, observer registration, or power assertion ran here.
The installed app and its state were not touched. No new tests or dependencies
were added. Existing virtual scenarios ran through real isolated daemons.

## Problems fixed

- Capture cleanup previously used one `hidden` flag for both hiding and cursor
  disassociation. A successful show after failed reassociation forgot the failed
  operation. A failed show during enter rollback also lost its hide ownership.
  These operations now retain separate state, use the original display ID,
  retry failed cleanup, and return leave errors to callers. Enter/stop races
  recheck stop state under the cursor lock.
- The lifecycle monitor left its control source registered when it exited.
  Control-source ownership now invalidates the source on every exit, including
  partial setup failure, for both monitor and capture threads.
- A full notice queue could discard sleep/lock/capture failure and only log.
  It now retains overflow state and retries the existing notification-failure
  notice during later sends and permission polling. The daemon already handles
  this failure as a safe leave and input release in either role.
- Layout notifications could pass captured keys to local applications before
  the outer loop rebuilt the map, including an up needed by the remote peer.
  Capture now keeps using the previous complete table until replacement. A
  failed sink layout refresh retains its dirty flag so later keys can retry.
- Key-map errors included logical/physical key values that daemon error logs
  could expose. Errors now keep the operation and native status only.
- Direct native-poster visibility requests could show without an owned hide
  or repeat a hide. Visibility operations are now idempotent at that boundary.
- Display-list growth detection could not work: the native API reports the
  count actually written, capped by the supplied capacity. One spare slot now
  detects growth and retries, including growth after an empty first query.
- Clipboard handles could classify a generation sampled during another
  handle's replacement as a local change. Metadata classification and payload
  reads now share the replacement lock.
- Oversized native NSData reads returned a generic failure, so the daemon
  could not emit its clipboard-blocked result. They now return
  `ClipboardTooLarge` with byte count and limit.
- Clipboard requests cloned/decoded early formats before discovering invalid,
  duplicate, or oversized later formats. A borrowed validation pass now rejects
  the complete request first and reserves text space before PNG-to-DIB expansion.
  The lower-level writer also checks the whole input budget before copying.
- DIB conversion created a BMP copy and invoked the native BMP decoder before
  rereading every pixel through its own validated parser. It now reads the
  bounded DIB directly. Text line-ending normalization reuses its owned UTF-8
  buffer, and pasteboard preparation avoids extra intermediate byte-vector
  copies. Empty format sets use an explicit clear.
- Applied the CI owner's fixed-array chunk iteration change for the planned
  Rust 1.99.1 minimum. The root manifest remains owned by CI.

## Verification

Host build target: `aarch64-apple-darwin`. Toolchain: Rust/Cargo 1.99.1.
All Cargo commands used `nice -n 15` and `CARGO_BUILD_JOBS=2`.

```sh
cargo fmt -p openergy-platform-macos -- --check
cargo build --release --locked -p openergy-platform-macos --lib --examples
cargo build --release --locked -p openergy-daemon -p openergy-e2e
CLIPPY_CONF_DIR="$AUDIT_DIR/clippy-config" cargo clippy --release --locked \
  -p openergy-platform-macos --all-targets -- -D warnings
git diff --check
```

The temporary `clippy.toml` contains `msrv = "1.99.1"`, matching the CI owner's
pending root minimum-version update. It does not change repository config.
All checks exited 0. Clippy reported no code warnings; it printed the expected
configuration warning that its temporary MSRV differs from the unmerged root
manifest. Integration must include CI's Rust minimum-version update.
Build and Clippy output is retained under
[`verification/release-audit`](verification/release-audit).

The selected catalogue contains unchanged copies of these existing scenarios:
`A1-modifier-and-receiver-wheel`, `A5-unavailable-clipboard-readback`,
`S3-input-order`, `S4-disconnect-release`, `S5-disconnect-release`,
`S8-text-clipboard`, `S9-large-dib`, `S10-clipboard-cap`, `S12-display-change`,
`S16-permissions`, and `S18-click-numbering`.

```sh
target/release/openergy-e2e run --scenario-dir "$AUDIT_DIR/scenarios" \
  --openergy "$WORKTREE/target/release/openergy" \
  --port-range 25200-25219 --lab-port-range 25220-25239 \
  --keep --report "$AUDIT_DIR/results.json"
```

Result: **11/11 passed**, including the unavailable clipboard and over-cap
failure cases. Every node uses the virtual platform. The runner passes explicit
private config, socket and TLS paths, disables discovery, and owns cleanup.
No Docker peer was needed. The retained
[JSON result](verification/release-audit/virtual-e2e.json) replaces worktree
and temporary artifact paths with placeholders.

Virtual E2E ran after the main input/clipboard fixes. The later durable notice
overflow change and CI chunk-iteration change are covered by the final native
build and Clippy pass; virtual scenarios do not execute those macOS paths.

## Native limits and integration follow-up

This is build/source evidence for the native changes, not native runtime proof.
The virtual sink shares the click-numbering model; it does not run the macOS
injector. No new claim is made about activation, dead-key/IME behavior, modifier
aliases, clipboard codec compatibility, capture, cleanup recovery, TCC grant
requirements, negative-origin/Retina/hot-plug displays, sleep/lock notifications,
power assertions, latency, or memory use. Native acceptance still needs the
separate disposable test Mac described in [TEST-MAC.md](TEST-MAC.md). Native
pasteboard/codec scratch allocation is not a hard RSS bound, and NSPasteboard
replacement still has no atomic rollback guarantee.

Cross-scope source findings for the daemon owner:

- `adapter::platform` chooses macOS click numbering only at construction.
  `Runtime::apply_config` accepts changed `macos` settings without rebuilding
  the backend, so a live Native/Legacy change does not reach the injector.
- `Runtime::notice(Wake | Unlock)` stores fresh permissions but does not call
  `sink.permissions_changed` before it starts a new connection. The normal
  health refresh updates the cached post grant later; early post-wake input
  can use a stale cached grant.
- Capture/translation failure disables the capture worker. An unchanged
  granted permission snapshot does not schedule another enable in
  `refresh_health`; transient native capture failures need an explicit retry
  policy. This audit preserves typed notices and does not alter daemon policy.

## Clipboard clear follow-up — 2026-10-21

The session owner requested an explicit distinction between a real clipboard
clear and an unavailable snapshot. This follow-up requires platform commit
`ffdd07a` (`PlatformError::ClipboardUnavailable`). The macOS reader now returns
that variant for own-write suppression, a missing pasteboard, missing data for
an advertised type, a snapshot that changes during its read, and a nonempty
pasteboard containing only unsupported types. A stable empty type list still
produces successful empty `ClipboardData`. Invalid data and size-limit failures
retain their existing errors.

The isolated branch's platform crate does not yet contain this shared variant.
Validation used a private source copy of commit `670cd0f`, overlaid with this
follow-up's macOS files and the exact `crates/platform` source from `ffdd07a`.
The real workspace's platform files, root manifest, and lockfile were unchanged.

```sh
nice -n 15 env CARGO_BUILD_JOBS=2 cargo build --release --locked \
  -p openergy-platform-macos --lib --examples
nice -n 15 env CARGO_BUILD_JOBS=2 CLIPPY_CONF_DIR="$AUDIT_DIR/clippy-config" \
  cargo clippy --release --locked -p openergy-platform-macos --all-targets \
  -- -D warnings
cargo fmt -p openergy-platform-macos -- --check
git diff --check
```

Build and Clippy exited 0 against that real shared dependency. Clippy's temporary
MSRV is 1.99.1, as above. Redacted output is retained in
`verification/release-audit/clipboard-clear-build.txt` and
`verification/release-audit/clipboard-clear-clippy.txt`.

No native clipboard or permission API ran. No unit tests, mocks, or new test
commands were added. This follow-up has build/source evidence only; the session
and daemon owners verify actual clear forwarding through the real virtual
daemon interfaces after integration. The earlier 11/11 result is unchanged and
does not claim native coverage or verification of the new session behavior.
