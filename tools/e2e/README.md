# Openergy external E2E runner

`openergy-e2e` runs real binaries and real sockets. It links only `ipc` from
this workspace. It does not link `session`, `protocol`, or a platform backend.
There are no unit tests, mocks, test doubles, or desktop input calls.

## Run

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
cargo clippy --release -p openergy-e2e -- -D warnings
target/release/openergy-e2e list
target/release/openergy-e2e run
# Quote shell globs. Match the scenario name or TOML filename.
target/release/openergy-e2e run --scenario 'S13-*' --keep
target/release/openergy-e2e run --scenario 'L*' --report /tmp/lab-results.json
```

Default destination ports are **24990–24999** for local peers and
**24930–24939** for Docker publications. Change them with `--port-range A-B`
and `--lab-port-range A-B`. Ranges must not overlap. Both have at most 101
ports. The lab range starts at 24900 or above. Port 24800 is always refused,
including a range that contains it. The runner reserves free ports before
startup. It releases a reservation immediately before launching its owner.
Another program can still claim a port during that unavoidable handoff;
startup then fails with retained artifacts.

`OPENERGY_BIN=/absolute/path/openergy` or `--openergy PATH` selects the daemon.
The default is the binary beside the runner. `--scenario-dir DIR` selects an
external catalogue. `--json PATH` is an alias for `--report PATH`. List/help
exit 0; a completed run with any failed scenario exits 1; CLI/schema errors
exit 2. An empty selection is an error and never reports a green run.

Scenarios run in filename order, one at a time. Each gets a private 0700
`/tmp/oe7-PID-TIMESTAMP/sN` directory. Configs, sockets, TLS files, logs,
fixtures, and command output stay there. Each daemon receives explicit
`--config`, `--socket`, `--platform virtual`, and isolated `OPENERGY_CONFIG_DIR`,
`OPENERGY_TLS_DIR`, `OPENERGY_SOCKET`, XDG paths, and
`OPENERGY_DISABLE_DISCOVERY=1`. The runner never changes `HOME`. The daemon
must honor the explicit config parent for TLS/identity paths; the environment
names are additional isolation hints, not new frozen daemon CLI flags.

A node uses only numeric **127.0.0.1:assigned-port** endpoints. No LAN address
or native platform is allowed. Configs are checked again immediately before
spawn, including files produced by CLI imports. Patches cannot delete network
settings and restore production defaults. `--allow-desktop`, native
`synergy-core`, native permission requests, and live config import through
arbitrary IPC are refused. Use the confined CLI import path instead.

## Docker isolation

The existing `openergy-interop:latest` image must be built first using
`tools/interop-lab/README.md`. The runner never changes that lab directory.
For every lab node it copies `lab.sh`, `container.sh`, and `session.sh` into
its private node directory. This is necessary because the lab script mounts
`.state` relative to itself even when `LAB_NAME` changes. It exports a unique
`LAB_NAME=oe-lab-wp6-PID-sN-NODE` and an assigned one-port `LAB_PORT_RANGE` on
**every** command. Each lab node has its own container, display, and trust
files. Docker clients use the lab-approved `host.docker.internal:assigned-port`
alias; host listeners and publications remain bound to 127.0.0.1.

Cleanup calls the copied `lab.sh down`, then removes only that owned container.
It never kills another user's process or touches another container. Process
nodes have separate process groups. Cleanup resumes stopped children, sends
TERM, waits, then sweeps the whole owned group with KILL. A separate cleanup
guardian watches an EOF pipe. It also tears down registered groups and labs
if the runner receives SIGINT, SIGTERM, or SIGKILL. Normal cleanup failures
make a scenario fail. The guardian is best effort if Docker itself is down.

## Scenario format

A scenario is TOML data. Each file starts with a requirement comment and has
`name`, `requirement`, optional `blocked_by = ["WP6"]`, and `timeout_ms` (default
5000). `blocked_by` only annotates a product failure. It does **not** skip a
scenario or turn failure into success. `node` and `step` are arrays of tables.
Every step is schema-checked before any child starts, even after earlier
steps that currently depend on unfinished work packages.

```toml
# Requirement: A real virtual pair connects and forwards input.
name = "example-connect"
requirement = "Plain handshake and ordered virtual injection"
blocked_by = ["WP6"]
timeout_ms = 5000

[[node]]
id = "server"
kind = "openergy"
role = "server"
platform = "virtual"
config = { server = { heartbeat_ms = 500 } }

[[node]]
id = "client"
kind = "openergy"
role = "client"
remote = "server"

[[step]]
do = "wait_connected"
node = "client"
peer = "server"

[[step]]
do = "inject"
node = "server"
events = [{ type = "mouse_move_abs", x = 1919, y = 540 }, { type = "mouse_move_rel", dx = 20, dy = 0 }]

[[step]]
do = "inject"
node = "server"
events = [{ type = "key_down", key = 97, modifiers = 1, button = 1 }, { type = "key_up", key = 97, modifiers = 1, button = 1 }]

[[step]]
do = "assert_recorded"
node = "client"
expect = [{ event = { type = "key_down", key = 97, modifiers = 1 } }, { event = { type = "key_up", key = 97 } }]
```

### Nodes

`id` is a unique ASCII letter/digit/underscore/hyphen name, at most 48 chars.
Common fields are `kind`, `role`, `name`, `remote` (another node ID or an
assigned numeric loopback address), `port` (optional fixed port within its
range), `autostart` (default true), and `tls` (default false).

- **openergy**: `role = "off" | "server" | "client"`, `platform = "virtual"`
  only, `config` is a JSON-shaped merge override of the full generated config.
  Use the frozen serde name **`screen`**, not `screens`. `require_client_trust`
  defaults false for disposable peers; explicitly enable it for mutual trust.
  `trust = { trusted-servers = ["v2:sha256:…"], trusted-clients = […] }` seeds
  private files in `NODE/tls`. `parent = true` creates a real gated parent and
  a separate exit observer for S20. Layouts otherwise place unique screen
  names side by side in node declaration order at 1920×1080.
- **lab**: `peer = "deskflow" | "barrier"`, role server/client, `screens` is
  the lab's screen-list or classic-config string, `position` is its layout
  direction, `protocol` is the lab's dialect selector. `trust.fingerprint`
  seeds the peer's pin. One container per node permits real lab↔lab scenarios.
- **raw**: role server creates a loopback TCP listener; role client connects
  to `remote`. The runner owns the stream. There is no implicit handshake or
  heartbeat responder: all traffic is explicit step data.

### Values and expectations

`save = "result"` saves a step result. `${result.path.to.field}` reads a saved
JSON value. An entire variable string preserves its JSON type; interpolation
within a longer string renders it as text. Arrays use numeric path segments.
Built-ins are `${dir}`, `${fixtures}`, and `${NODE.port}`, `.dir`, `.socket`,
`.config`, `.name`. Undefined variables fail; they never become empty strings.

Object expectations match a subset of fields, so added ledger/status fields
are harmless. Array expectations preserve order and length. Field predicates
use dotted paths or JSON pointers and `$exists`, `$ne`, `$gte`, `$lte`,
`$contains` (array subset or string substring), and `$len`. TOML has no null;
use `$exists` on a nested leaf to distinguish absent/null-parent data.

```toml
[[step]]
do = "wait_status"
node = "server"
fields = { "peers" = { "$contains" = { name = "client", connected = true } }, "stats.events_out" = { "$gte" = 2 } }
timeout_ms = 5000
save = "status"
```

### Steps

All waits have a bounded `timeout_ms`, defaulting to the scenario value,
capped at 180 seconds. Socket reads and writes use absolute deadlines, so
partial frames cannot keep resetting the timeout. Failures show field/operation
names and byte counts; the runner does not print key or clipboard contents.

| Step | Fields and behavior |
| --- | --- |
| `wait_status` / `assert_status` | `node`, `fields` predicates; polls `status.get`. |
| `wait_connected` | `node`, optional `peer`; checks a connected status/peer. |
| `inject` | `node`, `events` in frozen `InputEvent` JSON. Optional numbering controls `seed`, `hid_seed`, `now_ms`, `click_interval_ms`, `click_distance` pass through in `virtual.inject` params. |
| `ledger_mark` | `node`; saves last sequence and count. |
| `assert_recorded` | `node`, `expect` ordered subsequence; optional `since` exclusive sequence, `exact = true` for no extra records. Empty expect requires an empty selected ledger. |
| `assert_recorded_tail` | Same ordered subsequence, but its last match must be the final ledger record. `expect_tail` on `assert_recorded` is an alias. |
| `assert_numbering` | `node`, `seed`, `min_downs`, optional exact `states` array. Checks sequence order, increasing down numbers above seed, per-button matching drag/up numbers and click states. All buttons must be up unless `allow_held = true`. |
| `clipboard_set` | `node`, `text` or confined `fixture`, optional `kind` (`text_utf8`, `html_utf8`, `dib`), `encoding = "bytes" | "base64"`. Or explicit `formats` array. Sends `virtual.clipboard {set:{formats:[…]}}`. |
| `clipboard_get` | `node`; sends `{get:true}` and can save the result. |
| `assert_clipboard` | `node`, `kind`, any of `text`, `sha256`, `bytes`, or `empty = true`; polls real clipboard reads. Accepts `formats` at the result root or under `data`. Hash checks decode bytes, not just reported metadata. |
| `fixture_dib` | confined `path`; writes deterministic BI_RGB DIB, returns `path`, byte count, SHA-256. |
| `set_screens` | `node`, `rects`; sends `virtual.set_screens {rects}`. |
| `set_permissions` | `node`, `permissions`; sends that object to `virtual.set_permissions`. This never calls native TCC APIs. |
| `config_patch` | `node`, `patch`; merge patch over IPC. Optional `expect`, `fields`, or `expect_error` code. |
| `ctl` | `node`, `method`, optional `params`, `expect`, `fields`, `expect_error`. Any virtual-safe IPC method; native permissions and live imports are rejected. |
| `subscribe` / `wait_event` | First subscribe on `node`; then wait for `event` and optional `expect` data on that persistent socket. |
| `kill` / `pause` / `resume` | Owned `node`; signal TERM/KILL/STOP/CONT/INT/HUP. Pause/resume select STOP/CONT. |
| `stop` / `start` / `restart` | Node lifecycle; `stop` checks clean daemon exit unless `force = true`. Restart uses the existing on-disk config. |
| `parent_exit` / `wait_exit` | Release S20's parent gate; require actual daemon exit `code` (default 0), recorded by its observer. Ordinary children also support `wait_exit`. |
| `raw_send` | `node`, `hex` or builder fields; optional bounded `repeat`, `shutdown_write`, `allow_closed` for flood probes. |
| `raw_expect` | `node`, hex/builder bytes; exact bytes unless `framed = true` (one bounded frame payload prefix). `scan = true` skips unrelated complete frames until the deadline. Use `payload_only = true` on builders with framed expectations. |
| `raw_expect_closed` | Require EOF/reset; drains bounded reads until deadline. |
| `lab_drive` | `node`, xdotool `args` array; runs inside Xvfb. |
| `lab_pointer` | `node`, optional `fields` over parsed `{x,y,screen,window}` from xdotool's shell output. |
| `lab_events_dump` | `node`, optional POSIX `regex` or `nonempty`; content is not returned in the report. |
| `lab_clipboard` | `node`, `action = "get" | "set"`, `text` for set, optional `expect_text` for get. Container selection only. |
| `lab_ctl` | `node`, `args`; allows fingerprint/events/status/logs commands. Fingerprint result has `synergy` and `colon` strings. |
| `set_lab_trust` | Stopped lab `node`, `fingerprint`; seed its next launch. |
| `set_lab_screens` | Stopped lab `node`, confined classic-config `path`; seed its next launch. |
| `copy_identity` | Stopped target `node`, `source` node; copies only disposable PEM and trusted-server/client files. |
| `cli` | `node`, `args`, optional exit `code`, stdout `regex`, combined `output_regex`, `stdout_path`. Runs only offline Openergy CLI subcommands with explicit confined paths/config. `--apply` is forbidden. |
| `write_file` | confined `path`, `text` or text `fixture`; expands variables in fixture templates. |
| `assert_file` | confined `path`, optional `exists`, octal string `mode`, POSIX extended `regex`, `sha256`, `toml_expect`; returns size/hash. Unix sockets are checked without reading them. |
| `assert_json` | `actual`, optional subset `expect`, `fields`; useful with saved results. |
| `sleep` | `ms`, at most 60 seconds. Prefer status/event waits; heartbeat/multiclick windows are explicit exceptions. |
| `measure` | `node`, `kind = "resources"`: sample RSS and cumulative process CPU deltas over `window_ms` (default 60000), limits `rss_bytes`, `cpu_percent`. `kind = "latency"`: `receiver`, `count`, `p99_ms`; correlate each injected move with its next recorded sequence. |

Latency measures the elapsed external inject→IPC-ledger round trip. This is
an upper bound on capture→recorded latency, including observer IPC overhead.
It reports p50/p95/p99/max. It is not an in-process sub-millisecond benchmark.
Resource CPU is a window delta, not `ps %cpu`'s lifetime average. `ps` time
precision limits very short windows; S15 uses the required 60-second window.

Raw builders: `hello`, `hello_back` (`product`, `major`, `minor`, `name`),
`screen_info` (`width`, `height`), `code`/`unknown` (`code`, `payload_hex`),
`options` (`values`, a flat DSOP integer vector), `oversized`, and `garbage`.
They are original encodings from `docs/re/03-core-protocol.md`. The runner does
not call the protocol codec. Default hello is Synergy 1.8. Barrier probes
explicitly select Barrier 1.6, matching the real 2.4 lab peer.

## Catalogue and evidence

S1–S21 implement the r2 catalogue. S2 has a raw wire-order companion; S13
splits hostile cases so one failure cannot hide another; S17 splits the
replacement matrix; S20 has an off-role parent-watch companion independent
of networking. S0 and R1/R2 exercise current real-daemon control/lifecycle
paths. I1–I5 expand across Deskflow/Barrier, TLS/plain, both directions,
clipboard, edges, oversize clamps, and exported configs. L1–L4 are lab-only
checks and can pass while Openergy networking is unfinished.

`verification.md` records the current complete run, exact commands, and a
scenario-by-scenario result table. `verification.json` is the machine report.
A failed product expectation stays FAIL, with the scenario's WP annotation.
A runner failure is labelled **runner bug**. Docker/tool availability is a
separate **environment** failure; none of these categories is a skipped pass.

Success directories are deleted unless `--keep` is set. Failures always keep
their directory. The human report prints the path and redacted stderr tails.
Artifacts include all process stdout/stderr files, real lab state/logs,
last status, and a redacted ledger dump. Runner-created command captures
store byte counts/hashes after use. Ledger dumps omit key values/clipboard
bytes; fixture files contain synthetic data only. Third-party lab logs are scrubbed after the lab stops. X11 event captures
become size/hash records; input/clipboard debug lines are redacted. No key or
clipboard content is printed in reports.

## Integration requests and limits

These requests do not change another crate's frozen signatures:

1. **Large virtual clipboard IPC encoding:** frozen `ipc::read_frame` caps
   frames at 16 MiB. An 8 MiB `Vec<u8>` serialized as JSON integers can exceed
   that cap. The runner implements an additive JSON alternative:

   ```json
   {"method":"virtual.clipboard","params":{"set":{"formats":[{"kind":"dib","bytes_base64":"…"}]}}}
   ```

   Proposed WP3/WP6 behavior: accept exactly one of `bytes: number[]` and
   `bytes_base64: string` per format; return `bytes_base64` for large get
   results. No Rust public signature change is required. Base64 for the
   fixture is about 11.2 MiB and fits the existing IPC frame limit. S9/S10/I3
   select this representation and remain blocked until the daemon accepts it.
2. **Virtual timing/numbering controls:** the generic IPC request already
   supports `virtual.inject {events, seed, now_ms, click_interval_ms?,
   click_distance?}`. WP3/WP6 must apply the supplied seed and monotonic time
   at the receiving virtual sink for S18. The scenario sends empty-event control
   updates to the **client**, then injects actual input at the server; no seed
   or clock is added to the wire protocol. These are additional JSON params,
   not native event timestamps or platform-crate linkage.
3. **Barrier client dialect:** Barrier 2.4 refuses a Synergy hello and speaks
   Barrier 1.6. The frozen daemon config has no outgoing dialect selector.
   I4 Barrier cases are present and explicitly annotated. Proposed additive
   config fields, if needed by WP6: `ServerCfg { protocol_product: String,
   protocol_minor: u16, .. }` with defaults `"Synergy"`, `8`; configure
   `"Barrier"`, `6` for those cases. The runner does not invent config keys
   or change that crate. Deskflow's `protocol` selector affects only lab nodes.
4. **Identity CLI isolation:** WP4 must honor `--config PATH` (or the explicit
   `OPENERGY_CONFIG_DIR`/`OPENERGY_TLS_DIR`) for fingerprint/trust CLI commands.
   The runner never uses the user's default identity. Its trust files sit
   beside each scenario config. If WP4 chooses a different explicit selector,
   adapt the runner's CLI args before enabling those scenarios.

The DIB fixture is 8,388,648 bytes: a 40-byte BITMAPINFOHEADER followed by
exactly 8 MiB of 2048×1024, 32-bit bottom-up BI_RGB pixels. Blue is `i % 251`,
green `(i / 251) % 251`, red `(i * 17) % 251`, alpha 255 for pixel index i.
There is no BMP file header. `fixture_dib` reports its reproducible SHA-256;
R1 checks that hash against the actual saved file.

The root Cargo.lock belongs to the orchestrator. Building this crate updates
its package dependency edge list (add `toml`/`sha2`, remove `openergy-config`);
WP7 commits only `tools/e2e`. The orchestrator must regenerate the workspace
lockfile when integrating this commit. No new dependency package/version is
introduced beyond the existing workspace lock resolution.

To verify forced interruption of the runner itself through real processes:

```sh
python3 tools/e2e/verify_teardown.py
# Optional ranges stay separate from other active scenario runs.
python3 tools/e2e/verify_teardown.py --port-range 24998-24999 --lab-port-range 24938-24939
```
