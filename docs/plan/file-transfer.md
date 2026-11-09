# File transfer and cross-screen drag-and-drop: design

Status: design only. This document changes no behavior.
Audience: maintainer first, then the implementer of each phase.
Scope: Synergy-family wire 1.6–1.8, Rust workspace, macOS app plus daemon.

Terms in this document:

- **Host** is the server role. It owns the physical keyboard and mouse.
- **Client** is a joined screen. It receives input from the host.
- **Peer** is either role on one connection.
- **Explicit send** means: the user picks a file, the peer receives that file.
- **Drag-and-drop** means: the user drags a file over the screen edge and
  drops it on the other computer.
- **Classic** means the Synergy/Barrier/Deskflow `DFTR`/`DDRG` wire behavior.
- **Openergy-only** means new wire messages only Openergy understands.

## 0. Summary and recommendations

Openergy already parses the classic file messages. It discards them with one
notice per peer. No working file transfer exists today.

The classic protocol is weak. `DFTR` carries only a total size and raw bytes.
It carries no file name, no transfer ID, no offset, no acknowledgement, and
no cancel. `DDRG` carries names and sizes, but the reference code writes only
the first file. Current Deskflow has removed drag-and-drop. Current Synergy,
in the build this repo reverse-engineered, has only no-op stubs. One narrow
classic target may still exist: Barrier 2.4 and old Synergy builds.

So this design splits the work:

1. **Explicit send is Openergy-only.** New small control messages add an
   offer, an answer, a transfer ID, a SHA-256, and a cancel. Bulk data then
   moves in small chunks with the present network priorities. This feature
   needs no new OS drag APIs. It ships first.
2. **Drag-and-drop reuses the same Openergy data path.** OS drag hooks come
   later, one platform at a time. The drop lands in the configured receive
   directory. Dropping "into an app window" is not a version 1 goal.
3. **Classic `DFTR`/`DDRG` is an evidence-gated extra.** A lab pass must prove
   a real peer accepts it before any compat send is enabled. Until then,
   Openergy keeps its present discard behavior for classic peers.

Recommended defaults, subject to §10:

- File transfer enabled, receive by prompt, drag disabled.
- Max file size 1 GiB. Hard cap 16 GiB.
- Receive directory: `Received/` inside the Openergy state directory.
- Name conflict: never overwrite. Add a ` (1)` suffix.
- File features require a verified TLS peer. Plaintext peers cannot use them.
- One regular file per transfer. No directories. No symlinks in version 1.

Non-goals for this design series: folder transfer, clipboard file paste,
drop-into-application, Wayland drag, and any claim of Synergy file compat
without a lab pass.

## 1. What the code does today

This section is fact, not proposal. Line numbers drift; symbols are the anchor.

### 1.1 Protocol codec (`crates/protocol`)

- `Message::FileTransfer { mark: u8, payload: Vec<u8> }` and
  `Message::DragInfo { count: i16, payload: String }` already exist in
  `src/lib.rs`. The codec maps them to `DFTR%1i%s` and `DDRG%2i%s` in
  `src/codec.rs` (`decode_frame`, `encode`). The crate has no dependencies.
- Payloads use `blob()`/`text()`: a `u32` length plus bytes, capped by
  `MAX_STRING_LEN` = 1 MiB. Frames are capped by `MAX_FRAME_LEN` = 4 MiB.
  The decoder checks lengths before it copies payload bytes.
- The codec does **not** validate `DFTR` marks, and it accepts a negative
  `DDRG` count. Session code must validate both. The codec also requires
  the `DDRG` payload to be valid UTF-8. A classic peer can send platform
  paths in a local encoding. On Windows paths that can make decode fail.
  The error today is a connection close (`BadFrame`), not a skipped drag.
  Section 3 returns to this.
- `DCLP` clipboard chunks show the family pattern: mark `1` start with an
  ASCII decimal total, mark `2` data, mark `3` end with empty payload.
  `Chunk` in `src/lib.rs` models this. The codec strictly checks the
  decimal total for `DCLP`.
- There is no `DFIL` message in this protocol family. No file in this repo
  and no Deskflow protocol header defines it. The file messages are
  `DFTR` and `DDRG` only.
- `docs/re/03-core-protocol.md` §4 records the same two codes for the
  inspected Synergy Mac build. It marks both "unsupported here": the server
  proxy methods at `ClientProxy1_5` level are no-ops, and no payload parser
  exists in that build. It warns: do not infer `DFTR` marks or content
  format from `DCLP`. Section 2 supplies that missing evidence from
  Deskflow source instead.

### 1.2 Session (`crates/session`)

- Both roles discard the messages today. In `src/server.rs` and
  `src/client.rs`, `Message::FileTransfer` and `Message::DragInfo` call
  `Peer::decline()` in `src/common.rs`. It logs once per peer and emits one
  `transfer_unsupported` error notice. Framing stays in sync. A following
  mouse-up still processes (`docs/plan/architecture-r2.md` §4.4).
- Clipboard transfer state lives in `Transfers` (`src/clipboard.rs`):
  one `Incoming` buffer, a queue of at most 3 `Outgoing` buffers, each an
  `Arc<Vec<u8>>` held whole in memory. Emission is paced open-loop: at most
  one chunk per millisecond (`next_chunk_ms`). Data chunks use
  `CHUNK_BYTES = 32 * 1024 - 18`, so one full frame is at most 32 KiB.
  Incoming transfers time out after 10 s. The total cap is
  `clipboard.max_bytes`, hard-capped at 64 MiB (`HARD_CAP`). A new grab
  cancels queued and partial state for that clipboard ID.
- Direction policy is explicit. Session code checks `clipboard.enabled`,
  `clipboard.send`, and `clipboard.receive` on each path, plus the peer's
  `CLPS`/`CLSZ` options from `DSOP`. It also gates clipboard sends on peer
  minor version: the client requires `version.minor >= 6`
  (`src/client.rs`), and the server skips peers below 1.6
  (`src/server.rs`, `payload` path). File policy must mirror this shape.
- Session machines are deterministic. Time, I/O, and platform effects enter
  as events and leave as `Action` values (`src/lib.rs`). File disk I/O must
  therefore live outside session, like clipboard reads live in the daemon
  clipboard worker.
- Edge switching stops while a mouse button is held.
  `motion()` in `src/server.rs` computes `can_switch` from
  `local.buttons.is_empty()` plus lock and modifier gates. A file drag
  holds a button by definition. Drag across the edge needs a new, narrow
  exception here (§5.2).
- Hello gives no file capability and no Openergy identity. The server sends
  `Synergy 1.8`, or `Barrier 1.6` when `options.barrier` is set
  (`src/server.rs`). The client echoes the server's product string in
  hello-back (`src/client.rs`). A third-party peer and an Openergy peer look
  the same at hello. Section 3.3 designs detection around that fact.

### 1.3 Network (`crates/net`)

- Priority has three classes (`src/lib.rs`): `Input`, `Control`, `Bulk`.
  Per connection, `Queues` in `src/reactor.rs` holds three FIFO queues plus
  one active packet. `choose()` prefers control barriers and slow-peer
  control, then input, then control, then bulk. `write()` sends at most one
  whole bulk frame per poll round (`bulk_done`) and 64 KiB in total per
  round. Bulk can therefore not starve input by design.
- Bulk admission is bounded. A frame marked `Bulk` must be at most
  `BULK_SLICE` = 32 KiB on the wire, or `Mailbox::prepare` rejects it as
  `InvalidFrame`. A per-connection `BULK_WINDOW` of 256 KiB caps reserved
  bulk bytes in the mailbox; past it, sends fail with `QueueFull`.
  The socket send buffer is also capped at 256 KiB.
- Control gets a 1 MiB mailbox allowance above the 8 MiB budget. Small
  control frames are the right home for offer/answer/cancel traffic.
- `cancel_clipboard()` clears the **whole** bulk queue of a connection.
  That is correct while clipboard is the only bulk user. It becomes wrong
  the day files share the queue. Section 6.2 requires a per-stream purge.

### 1.4 Daemon runtime and the deferred Bulk backlog

- `Action::SendClipboardChunk` is the only action routed to `Bulk`
  (`send_clipboard` in `crates/daemon/src/runtime.rs`). Every other
  `Action::Send` becomes `Input` for input-shaped messages and `Control`
  for the rest. A new file chunk action must follow the clipboard route,
  or file data would wrongly travel as control.
- New deferred backlog: `deferred_bulk: BTreeMap<ConnId, BulkBacklog>`.
  When the driver queue is full, chunks wait in the daemon, gated on
  `ConnInfo.queued_bytes` against 256 KiB. Order is strict: behind a
  backlog, nothing overtakes. The backlog aborts a transfer after 2 s with
  no progress, or above 16 MiB of deferred bytes. Abort publishes
  `clipboard.blocked`, cancels the session transfer, and clears net bulk.
  Disconnect and restart clear the whole map.
- Two consequences for files. First, queued entries are owned `Message`
  values in memory, so the backlog must stay shallow: pull chunks from
  disk on demand (§6.1). Second, the 2 s stall abort is tuned for
  clipboard, where a fresh copy is cheap. Files need a longer rule (§6.2).
- Config changes already reset transfer state. A clipboard config change
  replaces each peer's `transfers` in session and drops backlog entries in
  the daemon. File config changes must do the same (§7.3).

### 1.5 Config, IPC, platform, and E2E surfaces

- `ClipboardCfg` in `crates/config/src/lib.rs` is `{ enabled, send, receive,
  max_bytes }`, all default-on, `max_bytes` default 64 MiB. `Config` uses
  `deny_unknown_fields`. A downgraded daemon rejects an unknown future
  section; §11 records the rollback note. Synergy import maps
  `clipboardSharing`/`clipboardSharingSize` and warns that
  `developers.dragAndDrop` has no Openergy mapping
  (`crates/config/VERIFICATION.md`; `docs/re/02-service.md` shows the field
  is stored but unused by the Synergy service too).
- IPC is frozen types in `crates/ipc/src/lib.rs`: method names, event
  names, and a `Status` object with a `clipboard` block
  (`last_transfer`, `last_blocked`). The daemon dispatches in
  `dispatch()` in `crates/daemon/src/runtime.rs`. Unknown methods return
  a typed `unknown_method` error. Events go through `EventBus`, whose
  per-client queue holds 256 events and drops from the front. Progress
  events must be sparse or they will evict useful events.
- The macOS app is a thin IPC client (`docs/plan/ui-contract.md`):
  `apps/macos/Sources/Openergy/Daemon/DaemonClient.swift`,
  `Model/Status.swift`, `Model/AppModel.swift`, `Views/SettingsPanes.swift`.
  It renders only from IPC status and events. A daemon-first phase needs
  no app change.
- The platform crate has no file or drag surface. `InputEvent` in
  `crates/platform/src/lib.rs` has keys, buttons, moves, and wheel only.
  Traits are `InputSource`, `InputSink`, `ClipboardBackend`,
  `ScreenSystem`. Drag support needs new trait surface (§5.3).
  Explicit send needs none: paths enter through IPC and the daemon's own
  file worker.
- The E2E runner (`tools/e2e/README.md`) runs real daemon binaries and
  real sockets, virtual platform only. Useful steps already exist:
  `ctl` (any IPC method), `config_patch`, `write_file`, `assert_file`
  (exists, mode, regex, `sha256`), `subscribe`/`wait_event`,
  `raw_send`/`raw_expect` with a `code` builder, `kill`/`pause`/`resume`,
  and `assert_clipboard` with hash checks. The runner links only `ipc`.
  The interop lab (`tools/interop-lab/README.md`) pins Deskflow 1.27.0 at
  wire 1.8 and Barrier 2.4.0 at wire 1.6, both on Xvfb. Its `session.sh`
  passes no drag-drop flag to either peer today.

## 2. Classic wire behavior, from Deskflow and Barrier source

Deskflow is the open continuation of the Synergy core. Its pre-removal code
defines the classic behavior. Deskflow 1.22.0 then removed "the broken drag
and drop support". Deskflow 1.27.0, the lab peer, keeps the two message
definitions as deprecated documentation. Read the two with care: the 1.27
header comments disagree with the older code in two places. The code wins.
Both points below name the conflict.

### 2.1 `DFTR`: file bytes, no name

Format: `DFTR%1i%s`. After the code: one mark byte, then a length-prefixed
payload. Since protocol 1.5 (`ClientProxy1_5`, `FileChunk`). Marks mirror
`DCLP`:

| Mark | Payload | Meaning |
|---|---|---|
| `1` | total size, ASCII decimal | Start. The receiver clears its buffer. |
| `2` | raw bytes | Data. The receiver appends. |
| `3` | empty | End. The receiver checks size. Any other size is an error. |

Details that bind an implementation:

- The start payload is decimal text. `StreamChunker::sendFile` builds it
  with `sizeTypeToString`, a `stringstream` decimal conversion. The 1.27
  header comment says "file size (8 bytes)". That comment is wrong. This
  matches `docs/re/03-core-protocol.md` §5 for `DCLP`: the start chunk's
  `31 34` is ASCII `14`, not a binary integer.
- One buffer quirk needs a lab capture before byte-exact claims.
  `FileChunk::start` and `FileChunk::data` write one trailing NUL byte
  inside their chunk buffer. The likely wire payload excludes it, because
  the receiver appends payload bytes and then checks the exact total.
  An Openergy classic receiver must parse a start total of leading digits
  and may accept one trailing NUL. Its `DCLP` parser is stricter. Do not
  reuse that strictness blindly for `DFTR`.
- Classic senders use 512 KiB data chunks (`g_chunkSize`). Classic
  receivers append any chunk size, so a smaller sender chunk stays
  compatible in structure. Openergy must use small chunks (§6.2). The lab
  must prove a real receiver takes them.
- Assembly is uncapped and global. `FileChunk::assemble` appends to one
  `std::string` with static counters and no size limit, no transfer ID,
  and no offset. A second start silently clears the first transfer.
  Openergy must never copy this. It also must not classic-send large files
  to such a receiver without consent-scale care.
- There is no acknowledgement and no cancel message. The 1.27 header says
  the receiver aborts "by closing connection". Sender-side,
  `StreamChunker::interruptFile` only sets a flag; the loop breaks and
  still sends the end mark. The receiver then sees a size mismatch and
  reports an error. Between chunks, the sender posts keep-alive events so
  a long send does not look dead. Openergy gets the same effect from its
  control-priority `CALV` and its paced emission.
- Keep-alive during a long classic *receive* has no chunk-level reply.
  The connection stays alive because data itself arrives. A silent,
  half-sent classic transfer can only die by heartbeat or by a receiver
  timeout. Openergy's classic receiver needs its own deadline (§6.2).

### 2.2 `DDRG`: drag names and sizes

Format: `DDRG%2i%s`. After the code: a 2-byte file count, then one
length-prefixed string. `DragInformation::setupDragInfo` builds the string
as repeated `path,size,` groups: full source path, comma, decimal size,
comma. `parseDragInfo` reads each group back. It keeps only the base name
after the last `/` or `\`. It then stores the declared size.

Conflicts and limits:

- The 1.27 header says paths are NUL-separated. The code uses commas.
  The code wins. A file name with a comma in it breaks classic parsing in
  a way neither side can detect. Classic interop can never support comma
  names; Openergy-only offer names can.
- The count can name many files, but the feature is single-file in fact.
  `DropHelper::writeToDir` writes only `fileList.at(0)` and then clears
  the list. Synergy's own user FAQ states one file only, no folders,
  Windows Vista or newer and macOS 10.7 or newer, and that the file lands
  on the peer's desktop after a few seconds. Treat classic multi-file as
  undefined behavior, not a feature.
- Classic drag is gated per process, not per wire option. The core takes
  an `--enable-drag-drop` flag. Deskflow's `ArgParser` honors it only on
  Windows Vista+ and macOS builds; on X11 it logs "ignoring
  --enable-drag-drop, not supported on linux." Barrier shares this core
  lineage, so expect the same on the lab's Barrier build, to be proven on
  the real binary in Phase 0 (§8.3).
  Once both proxies exist at 1.5 or above, the wire messages are parsed
  regardless of any `DSOP` option. There is no standard option ID for
  drag, file size, or file enable. No classic peer negotiates files.
- Order on the wire is: `DDRG` first, when the drag crosses; then one
  `DFTR` transfer for the file. The receiver holds the pending name list
  (the "fake drag" list) and, on transfer end, writes the first file into
  the drop directory with `DropHelper`. If `DFTR` arrives with no pending
  `DDRG`, the old server only logs the start.
- `DDRG` travels in both directions in the protocol headers
  (Primary ↔ Secondary). Which computer starts a drag decides the data
  direction. The server proxy parses client-originated `DFTR`/`DDRG`; the
  client proxy parses server-originated ones. The repo's Synergy RE build
  stubs both (§1.1).

### 2.3 Clipboard file paste is not a family feature

Some users copy a file in one file manager and expect paste on the peer.
That needs clipboard file-URL formats (for example, `text/uri-list` or
macOS file promises) plus a byte channel. This protocol's clipboard has
three formats only: text, HTML, and DIB bitmap
(`crates/protocol/src/clipboard.rs`; `docs/re/03-core-protocol.md` §5
shows PNG, TIFF, and
file-URL interchange were not established in that core). Clipboard paste of
files is out of scope. Explicit send covers the same user goal with clear
consent. Note this in the user docs when the feature ships.

## 3. Interop verdict: what is compatible, what is Openergy-only

### 3.1 Peer matrix

"Wire" means the peer can parse the frames without error. "Works" means a
file lands on disk at the other side. Only "works" rows may feed user docs.

| Peer | Wire `DFTR`/`DDRG` | Working file transfer | Design stance |
|---|---|---|---|
| Openergy (this design) | Yes, plus new Openergy-only control codes | Explicit send; drag in later phases | Primary target |
| Deskflow 1.27.0 (lab, 1.8) | Codes documented as deprecated; feature removed in 1.22.0 | **No**. Send and receive paths are gone | Never classic-send by default. Lab proves our frames do no harm (§8.3) |
| Synergy, RE'd Mac build (1.8) | Tags recognized in proxy 1.5; methods are no-ops; no payload parser (§1.1) | **No** | Same as Deskflow. Expect discard, verify no crash |
| Barrier 2.4.0 (lab, 1.6) | Yes: pre-removal fork keeps 1.5 proxies and `--enable-drag-drop` | Unproven, likely no on this lab: the shared core ignores the flag on Linux/X11 (§2.2), and the lab peers run on Xvfb. Wire parsing may still occur; a stored file is what counts | Only classic candidate. Phase 0 decides on the real binary (§8.3) |
| Old Synergy 1.6–1.8 era, Windows/macOS | Yes | Yes, single file to desktop, per the Synergy user FAQ | Not in the lab. Classic send stays behind an opt-in toggle (§5.4) |
| Input Leap | Not pinned in this repo (the lab README notes it is absent from the lab's package index) | Unknown in this repo | No claim. Needs its own lab peer before any claim |
| Peers below wire 1.5 | No 1.5 proxy. Unknown-frame handling in old proxies can be a protocol error | No | Never send `DFTR`/`DDRG` or Openergy codes. Version gate in session |

Openergy hello accepts wire minor up to 8 and stores the peer version.
Classic send rules key off that stored version, like the clipboard 1.6
gate does today. Openergy-only send rules key off capability (§3.3), not
version, because version alone cannot identify an Openergy peer (§1.2).

### 3.2 Byte-level compat list for the classic path

If classic work proceeds after Phase 0, these points keep it honest:

- Send `DDRG` with count `1` and payload `name,size,` only. Use the base
  name, not a source path. The classic string exposes full paths. Old
  senders leaked the sender's directory layout to the peer and to logs.
  A base name parses the same on a classic receiver and leaks less. Lab
  must confirm a real receiver accepts a path with no slash:
  `parseDragInfo` searches for the last slash before the comma. When no
  slash exists, its index math is fragile. If the lab shows failure, send
  `./name,size,` instead. Decide from capture, not from reading alone.
- Send `DFTR` start as pure decimal digits, data as raw slices, end as an
  empty payload. Slice data to Openergy's bulk frame budget (§6.2).
- Accept classic data chunks up to the codec's 1 MiB string cap. Classic
  512 KiB chunks fit. A peer chunk above 1 MiB fails decode and closes
  the connection. That is a pre-existing codec bound. State it as a limit;
  do not raise `MAX_STRING_LEN` for this feature.
- Validate `DDRG` count is `1` for a working drop. Count `0`, negative, or
  above `1` in version 1: log once per peer with byte counts only, drop
  the pending drag, keep the connection. Never write more than one file
  from a classic drag, matching `DropHelper`.
- Treat a classic `DFTR` start with no pending `DDRG` as a drop, not an
  implicit explicit send. There is no file name to write to, and inventing
  one hides peer bugs. Count it in a `file.blocked` style event with a
  reason code, so "no silent drops" holds (§3.3).
- Keep the `DDRG` UTF-8 decode bound in the codec. Accept the cost: a
  classic Windows peer can send a non-UTF-8 path and lose the connection
  today. Mitigation is a codec change to carry drag payload as bytes,
  listed as an open choice in §10 (D9). Do not loosen `DragInfo` for
  other payloads silently in phase 1.

### 3.3 Capability detection and "no silent drops"

No standard handshake exists, so detection is layered:

1. **Openergy capability, server to client:** the server adds one private
   `DSOP` pair, option ID bytes `OFIL` (`0x4F46494C`), value = bit mask
   (1 = explicit v1, 2 = Openergy drag, 4 = classic-send willing).
   Unknown `DSOP` IDs are ignored as complete pairs by family peers
   (`docs/re/03-core-protocol.md` §6; Openergy's `Options::apply` keeps
   them in `raw` and ignores their meaning). Add a second pair `OFSZ`
   (`0x4F46535A`) with the server's receive cap in KiB, mirroring `CLSZ`.
   A live lab check (§8.3) must confirm Deskflow and Barrier tolerate
   these pairs before the server sends them by default. Until then, send
   them only when file transfer is enabled, which is also the kill switch.
2. **Openergy capability, client to server:** a client that sees `OFIL`
   answers after `Phase::Ready` with one new control frame, `OFCP`,
   carrying the same mask and cap. A client connected to a third-party
   server never sees `OFIL` and never sends `OFCP` or any Openergy file
   code. An Openergy server records client caps only from `OFCP`.
   This bootstrap order means no new-code frame ever reaches a peer that
   did not first opt in through `DSOP`.
3. **Classic capability:** there is none to detect. It is a config claim
   by the user (§5.4), gated by stored peer minor `>= 5` and by product:
   default off for `Synergy` 1.8 lab-class peers (Deskflow, RE'd Synergy),
   available as an explicit opt-in for `Barrier` 1.6 or older Synergy
   after Phase 0 evidence. The UI and `ctl` must show "unverified peer"
   wording for the classic path permanently.

"No silent drops" rules, both directions:

- Send side refuses early with a typed error: peer has no file capability,
  feature disabled by direction, file too large, TLS gate, or peer busy.
  It never starts a chunk stream at a peer that cannot finish it.
- Receive side answers every well-formed Openergy offer once: accept or
  reject with a reason code (§4.1). Malformed offers are protocol errors
  at the codec/session bound and are logged with sizes only.
- Classic receive always ends in one visible outcome per transfer:
  completed file, `file.blocked`, `file.failed`, or the existing
  once-per-peer unsupported notice while the feature is off.
- Status exposes per-peer file capability (`none`, `openergy`, `classic-
  unverified`) so the app can grey out Send with a reason before the user
  picks a file (§7.4).

## 4. Feature A: explicit file send

User story: on either computer, the user picks one file and a connected
peer. The peer confirms (or policy allows). The file appears in the peer's
configured receive directory. Both sides can cancel. Input and clipboard
keep working during the transfer.

### 4.1 New Openergy-only messages

Four new codes. All require `Phase::Ready`. Control codes travel at
`Priority::Control`. Data travels at `Priority::Bulk`. The codec gains
`u64` read/write helpers (8-byte big-endian); all other fields reuse
present codec patterns (`u8`, `u32`, bounded `%s`). Every frame code is
exactly four bytes: `decode_frame` reads four code bytes and
`Writer::code` takes `&[u8; 4]` (`crates/protocol/src/codec.rs`), so a
five-byte tag would dispatch as a different, unknown code. No existing
code uses the `OF` prefix.

| Code | `Message` variant | Fields after code | Priority |
|---|---|---|---|
| `OFCP` | `FileCapability { mask: u32, max_kib: u32 }` | `%4i%4i` | Control |
| `OFOF` | `FileOffer { transfer: u32, kind: u8, name: String, total_len: u64, sha256: [u8; 32] }` | `%4i%1i` + name `%s` + `%8i` + 32 raw hash bytes | Control |
| `OFAN` | `FileAnswer { transfer: u32, accept: u8, reason: u8, resume_offset: u64 }` | `%4i%1i%1i%8i` | Control |
| `OFTR` | `FileData { transfer: u32, chunk: FileDataChunk }` | `%4i%1i%s`, marks 1 start / 2 data / 3 end / 4 cancel | Bulk for marks 1–3; Control for mark 4 |

Field rules:

- `transfer`: sender-chosen, nonzero, unique per connection and direction.
  The daemon picks a random start value at connection setup and then
  increments, skipping zero. Random starts make resume matching (§4.5)
  safe against a restarted peer reusing small IDs. **Namespace rule:**
  in every `OF**` frame — offer, answer, data, and cancel alike —
  `transfer` is in the offer *originator's* namespace, no matter which
  peer sends the frame. Both endpoints therefore resolve a frame as
  `(connection, originator direction, transfer)`: the originator finds
  its outbound record, the receiver its inbound record. Without this
  rule, simultaneous opposite-direction sends that happen to pick the
  same ID make a cancel ambiguous — frame direction alone cannot say
  whether the sender of the cancel means "stop sending to me" or
  "I stop sending to you".
- Identity outside the wire codec: a wire `transfer` ID alone identifies
  nothing. Two clients can offer the same ID at once, as can opposite
  directions on one connection, and inbound IDs are peer-chosen. Every
  consumer past decode therefore uses a qualified key,
  `(ConnId, originator direction, transfer)`, where originator
  direction is offer-sender → receiver, independent of who sent the
  frame being decoded. On top of that, the daemon assigns
  each transfer a daemon-unique handle, `xfer` (monotonic `u64` from a
  random start, never reused within a daemon run), when the offer or
  send is created. Session maps `xfer` ↔ qualified key. Worker commands
  and events, staging names (§6.1, §7.2), IPC methods, events, and
  status (§7.4) carry `xfer` only — never a bare wire ID and never a
  `ConnId`. Resume matching (§4.5) stays scoped to one connection and
  direction, where the wire ID plus hash is unambiguous.
- `kind`: `0` explicit send, `1` drag drop (§5). One data path serves both
  features; policy and consent differ by kind.
- `name`: base name only. Codec bound 255 bytes, UTF-8, no NUL (reuse the
  `SECN` text pattern). Session rejects path separators and the rest of
  the §7.2 list. The sender sends the name it wants the file to have;
  it never sends a source directory.
- `total_len` and `resume_offset` are `u64`. Binary here, not decimal:
  these messages are Openergy-only, so no classic parser quirk applies.
  The classic path (§5.4) converts to decimal at its edge.
- `sha256`: hash of the whole file, computed by the sender's file worker
  before the offer (§6.1). Cost is one read pass. It buys end-to-end
  verification, resume identity, and duplicate detection.
- `OFTR` mark payloads: start carries an 8-byte `total_len` again (cheap
  consistency check against the offer), data carries raw bytes, end
  carries no payload, cancel carries a 1-byte reason. A receiver that
  sees data for an unknown or refused transfer discards the frame and
  counts it once per transfer; framing needs no recovery.
- `OFAN.accept` values: `0` reject or final failure (see `reason`), `1`
  accept and start data, `2` stored (final result, §4.2). Any other value
  is a session protocol error for that transfer.
- `reason` codes in `OFAN` and cancel share one small enum in session:
  `user_rejected`, `too_large`, `disabled`, `bad_name`, `busy`,
  `no_space`, `hash_mismatch`, `cancelled`, `stalled`, `io_error`,
  `unsupported`. IPC error codes in §7.4 mirror these strings.

Rejected alternative: extend `DFTR` with new marks (4 = name, 5 = cancel)
and skip new codes. Classic `FileChunk::assemble` returns an error for
unknown marks, and a name-first `DFTR` stream at a classic peer would be
indistinguishable from a corrupt drag. New codes keep the failure clean:
peers that never opted in never see them (§3.3).

### 4.2 Wire flow

```text
Sender                                             Receiver
  │  OFOF  transfer=7 kind=0 name total sha256      │   Control
  │ ───────────────────────────────────────────────▶ │  validate + consent (§7)
  │  OFAN  transfer=7 accept offset=0                │   Control
  │ ◀─────────────────────────────────────────────── │
  │  OFTR  transfer=7 mark=1 total                  │   Bulk
  │  OFTR  transfer=7 mark=2 bytes ...              │   Bulk, paced (§6.2)
  │  OFTR  transfer=7 mark=3                        │   Bulk
  │ ───────────────────────────────────────────────▶ │  verify size + SHA-256
  │  OFAN  transfer=7 accept=done (reuse OFAN)      │   Control, final result
  │ ◀─────────────────────────────────────────────── │
```

- The final result reuses `OFAN`: `accept=2` means stored, `accept=0`
  with a reason means failed after data (hash, write, rename). The sender
  shows success only after this frame. TCP delivery alone never means
  "stored".
- One active outbound and one active inbound transfer per peer and
  direction. Further sends queue in the daemon (§4.6). This mirrors the
  serialized `DCLP` rule and keeps `Transfers`-class state small.
- The offer expires. If no answer arrives in 60 s, the sender cancels
  locally and reports `file.failed` with reason `timeout`. The receiver
  applies the same 60 s bound to its prompt path (§7.1).
- Zero-byte files are legal: offer, accept, start, end, done. The hash is
  the SHA-256 of the empty input. No special case in staging.
- Enter/leave (cursor position) does not gate explicit send. The target
  is a named peer, like clipboard's per-peer fan-out, not the active
  screen. Server fan-out to many clients at once is not a version 1
  feature: one `file.send` call names one peer.

### 4.3 Session and daemon state shape

Session stays deterministic. New event and action types:

- Events in: `FileSendRequested { xfer, kind, name, total_len, sha256,
  peer }` (daemon → session after the worker hashed the file),
  `FileChunkReady { xfer, bytes }` (pull model, §6.1),
  `FileStageResult { xfer, ok, reason }` (worker write outcome),
  `FileAnswerRequested` is not a session event: consent lives in the
  daemon (§7.1), and its decision enters as
  `FileOfferDecided { xfer, accept, reason, resume_offset }`.
  Daemon↔session and session↔worker traffic carries the `xfer` handle
  (§4.1); session alone translates it to `(conn, direction, transfer)`
  for wire frames.
- Actions out: `SendFileChunk { conn, transfer, chunk }` (the `Bulk`
  route; `conn` plus outbound direction qualifies the wire ID),
  `StageFileChunk { xfer, bytes }` and
  `FinishFile { xfer }` / `DropFile { xfer, keep_part }` toward
  the file worker, plus `Notify` for IPC events. Wire control messages
  reuse `Action::Send`, which the daemon already maps to `Control`;
  the `OFTR` cancel mark (4) also goes through `Action::Send` for that
  reason, while marks 1–3 use `SendFileChunk`.
- Per-peer session state: a small `FileTransfers` struct beside clipboard
  `Transfers` in `Peer`: one outbound record (offer metadata, next offset
  to request, answered flag, pacing deadline), one inbound record (offer
  metadata, staged byte count, deadline), and the capability mask and cap
  from §3.3. It holds metadata, never file bytes, except the one chunk in
  flight inside an action.

### 4.4 Size limits

Four independent bounds, tightest wins:

1. Sender config `file.max_bytes` and receiver config `file.max_bytes`
   (the receiver's value reaches the sender as `OFSZ`/`OFCP` KiB).
2. Code hard cap `FILE_HARD_CAP`, proposed 16 GiB, enforced in session like
   clipboard `HARD_CAP`. Config validation rejects values above it, like
   `MAX_CLIPBOARD_BYTES` today.
3. Receiver free-space check at accept time: free bytes on the receive
   volume must exceed `total_len` plus 64 MiB headroom. Fail = reject with
   `no_space` before any data moves. A second check on write error aborts
   mid-stream.
4. Per-frame codec caps, unchanged.

Oversize offers are rejected at the `OFAN` step. Oversize local picks fail
in the IPC call itself. Neither case sends a single data chunk.

### 4.5 Resume and cancel

- **Cancel, both sides, any time:** send `OFTR` mark 4 at `Control`
  priority so it passes queued bulk. Either side may send it, and the
  §4.1 namespace rule keeps it unambiguous: the ID is the originator's,
  so the originator cancels its outbound record and the receiver
  cancels its inbound record for that ID — an opposite-direction
  transfer with the same numeric ID is a different record and is never
  touched. The sender stops pulling chunks, purges deferred and net
  bulk for that transfer (§6.2), and reports `file.failed/cancelled`.
  The receiver stops staging. User-initiated cancel deletes the
  partial file. Network or stall abort keeps it (§6.2).
- **Resume is a later phase (§8.1 phase 3), but the wire carries it from
  day one.** The accept answer includes `resume_offset`. In milestone 1
  the receiver always answers `0` and deletes partials on disconnect.
  The resume phase keeps the `.parts/<xfer>` partial plus a sidecar
  record `{peer, sha256, total_len}` for 24 h (startup cleanup keeps
  sidecar-backed partials inside that window); a new offer from the
  same peer with the same hash and size adopts the partial under its
  new `xfer` and resumes at the staged length. Hash of
  the staged prefix is checked by re-hashing the partial on resume
  accept, then streaming continues into the final whole-file hash.
  Any mismatch restarts at offset `0`. Classic peers get no resume: no
  offset exists in `DFTR`; a classic retry starts over.
- **Disconnect:** session drops the peer record wholesale today
  (`peers.remove` on the server, `peer = None` on the client). The daemon
  also clears deferred bulk on disconnect. The file worker gets a drop
  notice per active transfer and applies the keep/delete rule above.

### 4.6 Multi-file and directory policy (explicit send)

- The IPC call takes one path. "Send these 5 files" is 5 calls. The
  daemon queues them per peer and runs one at a time. Progress events
  carry the `xfer` handle, so a future GUI can group them itself. No
  batch wire concept in version 1.
- Directories are rejected with `is_directory`. No archive creation, no
  recursive walk. A future phase may add "send folder as `.tar`" as an
  app-side action; the daemon stays out of archiving.
- Symlinks are rejected with `is_symlink` in version 1. Following them
  silently can read outside the folder the user picked from. This is a
  §10 decision (D8) because some users expect send-what-it-points-to.
- Special files (FIFO, device, socket) are rejected. The worker opens
  with a regular-file check on the opened handle, not only on the path,
  so a swap between check and open cannot smuggle one in.
- A file that grows or shrinks during the hash pass fails with
  `changed_during_read`: size and mtime are captured before hashing and
  rechecked before the first chunk. Do not try to send a live tail.

## 5. Feature B: drag-and-drop across the screen edge

User story: the user starts dragging one file on computer A, pushes the
pointer through the shared edge to computer B, and releases. The file
lands in B's configured receive directory. This is §4's data path plus OS
drag hooks and one session exception. It ships after explicit send.

### 5.1 Drag wire flow (Openergy peer)

```text
Source screen                         Target screen
 drag held, edge reached               (cursor still on source)
  │  DDRG-classic is NOT used here.      │
  │  OFOF kind=1 name total sha256  ───▶ │  Control. Target arms a pending
  │                                      │  drop; shows drag feedback (§5.3)
 pointer crosses, user releases (DMUP)  │
  │  OFAN path: target answers accept ── │  on release over target screen
  │  OFTR marks 1/2/3 (§4.2)        ───▶ │  Bulk. Same verify + done frame.
 pointer released off-target / Esc      │
  │  OFTR mark=4 cancel (drag_aborted) ▶ │  Target disarms. No bytes staged.
```

- `kind=1` separates policy: drag uses `drag_send`/`drag_receive` toggles
  and its own consent rule (§7.1). The offer is sent at edge contact, not
  at drag start, so local-only drags cost no network traffic and no hash
  pass is wasted. Edge case: the hash pass on a large file delays the
  offer. The drag stays armed on the source meanwhile; if the user
  releases early, the daemon sends cancel and nothing crosses. Accept
  this delay in version 1; do not skip the hash for drag.
- If the drag comes back to the source screen without a drop, the source
  sends cancel. The target must tolerate cancel for an unknown transfer
  (double-cancel safe).
- Drop position (x, y) is recorded for feedback only. The file still
  lands in the receive directory (§0). No target-app drop in version 1.

### 5.2 The held-button edge exception

Today `can_switch` requires no held buttons (§1.2). Add one narrow rule in
`motion()`: when the daemon reports an armed file drag for this crossing
(new server event `DragArmed { files: 1 }` from the capture side), edge
switching may proceed with the button held, and the held button must not
be released toward either screen during the crossing. The peer-visible
sequence stays ordinary: leave/enter as today, then the remote mouse-up
on physical release. Invariants:

- The exception applies only while a drag is armed and only toward the
  peer named in the armed offer. It never applies to plain window drags.
- If the connection drops mid-drag, present release paths still run
  (`release_remote`, disconnect cleanup). A stuck remote button is a
  release-blocking bug; scenario F14 (§8.2) covers kill-mid-drag.
- Client-originated drag (user drags on a joined screen toward the host)
  needs the mirror path: the client cannot switch screens itself. The
  host still owns switching. In version 1 of drag, support host→client
  only, and state the limit. Client→host can ride the same `kind=1`
  offer once client-side drag detection exists (§5.3); the wire does not
  change.

### 5.3 Platform surface needed (new trait, additive)

New optional trait in `crates/platform`, default "unsupported", so every
backend compiles unchanged:

```text
trait DragFiles: Send {
    fn begin_tracking(&mut self, sink: DragSink);   // source side
    fn drag_feedback(&mut self, armed: bool);       // target side cursor/overlay
}
enum DragEvent { Started { paths: Vec<PathBuf> }, Cancelled, Released }
```

Per-OS reality, in build order:

- **macOS first.** The daemon already runs inside the app's TCC grant. But
  drag initiation belongs to a dragging session in a GUI process; the
  headless daemon cannot join Finder's drag. Practical version 1: the
  macOS **app** hosts a thin edge overlay/drag monitor
  (`NSDraggingSource`/`NSPasteboard` drag types) and reports drag events
  to the daemon over a new IPC method (`drag.armed`, §7.4). The daemon
  code path stays identical for later platforms. This is the same split
  as permissions dialogs today.
- **Windows second.** `IDataObject`/`CF_HDROP` with an `IDropSource`
  tracker in the capture process. UAC/UIPI limits from `SECURITY.md`
  apply unchanged.
- **X11 last.** XDND detection against the root window is fragile across
  window managers; Wayland is unsupported product-wide. X11 drag is a
  separate spike, not a phase promise.
- **Virtual backend:** a scripted `DragEvent` injection, mirroring
  `virtual.inject` and `virtual.clipboard`, so E2E covers drag without a
  desktop (§8.2). No real-desktop drag test exists in CI. Native drag
  joins the manual acceptance matrix (§8.4).

### 5.4 Classic drag send (opt-in, evidence-gated)

When the peer has no Openergy capability but the user enabled classic drag
for verified peer classes (§3.1), the send path swaps only its wire edge:

1. Send `DDRG` count `1`, payload from §3.2, at `Control` priority.
2. On drop, stream `DFTR` marks 1/2/3 at `Bulk` priority with §6.2
   slicing and pacing. The file worker and pull model are unchanged.
3. No accept, progress, completion, resume, or receiver cancel exists.
   The sender reports "sent (unverified)" after the end frame is queued.
   Local cancel before the end frame stops the stream; the receiver's
   exact-size check then fails on its side, which is the classic outcome.
4. Cap classic sends by `file.classic_max_bytes` (proposed default
   32 MiB): classic receivers assemble uncapped in memory (§2.1). The
   user's opt-in does not waive the cap.

Classic drag *receive* mirrors this: `DDRG` arms a pending record (name
from §7.2 rules, declared size checked against caps), `DFTR` bytes stage
to the same partial-file writer as §4, end triggers size check and rename.
It ships only with its own receive toggle, default off (§7.3).

### 5.5 Multi-file and directory policy (drag)

Version 1 drag accepts exactly one regular file. A drag with zero files
(text, image, URL drags) is ignored by the file system and stays a normal
pointer drag. A drag with 2+ files or any folder is refused as a whole at
arm time with a `file.blocked` reason `multi_file_drag`: no partial
transfer, no first-file-only surprise (that classic behavior in §2.2 is a
bug to avoid, even while mirroring its wire). The pointer drag itself
continues as input, so the user can drop back locally.

## 6. Shared data path: workers, backpressure, and memory bounds

### 6.1 Pull model with a daemon file worker

Clipboard holds whole payloads in memory (`Arc<Vec<u8>>`, ≤ 64 MiB).
Files must stream. Add a file worker beside the clipboard worker in
`crates/daemon/src/workers.rs`, with the same shape: a bounded command
channel in, a bounded event channel out, one thread, a `Waker` to wake the
I/O loop. It is the only component that touches file bytes on disk.

Send side (pull, depth 1 plus the net queue):

1. IPC `file.send` hands the worker `{ path, peer }`. The worker checks
   the file (§4.6), hashes it in 1 MiB reads, then raises
   `FileSendRequested` to session (§4.3).
2. After `OFAN` accept, session emits `SendFileChunk` only for chunk data
   it already holds. It holds at most one chunk ahead: each
   `FileChunkReady` refill is requested when the previous chunk leaves
   session. The worker `pread`s at the session-given offset, so session
   remains the single owner of progress. Session never blocks on disk;
   the worker never decides order.
3. Memory per active send: one ≤ 32 KiB chunk in session or backlog, plus
   the driver's 256 KiB bulk window. File size does not change daemon
   memory. Scenario F2 asserts this with a `measure` step (§8.2).

Receive side mirrors it: session hands each data chunk to the worker as
`StageFileChunk`. Worker commands carry the daemon-unique `xfer` handle
(§4.1), never a wire ID. The worker appends to
`<receive_dir>/.parts/<xfer>` in arrival order (single active inbound
per peer keeps this trivial) and streams a running SHA-256. `FinishFile`
makes the worker compare size and hash, `fsync` the file, then
atomically rename it from `.parts/` into place (§7.2). A
failed finish deletes or keeps the partial per §4.5 and reports the
reason. The worker also owns the receive-directory lock: two inbound
transfers from different peers on a server stage into distinct partial
names, but final renames serialize so conflict suffixes cannot race.

Classic receive reuses this worker. No in-memory assembly anywhere.

### 6.2 Backpressure contract

Fixed budgets (arithmetic in Appendix A):

- Data chunk payload: at most `FILE_CHUNK_BYTES = 32 * 1024 - 17` bytes, so
  one `OFTR` frame on the wire is at most `BULK_SLICE`. Classic `DFTR`
  sends use `32 * 1024 - 13`. Inbound frames stay under the existing
  codec caps; inbound chunk size is the sender's choice.
- Pacing: session emits at most one bulk chunk per peer per millisecond,
  shared across clipboard and file. Add the file tick beside
  `Transfers::tick` in `Peer::tick`, with **clipboard first**: if the
  clipboard tick emitted a chunk this millisecond, the file tick waits.
  A 64 MiB clipboard can therefore delay, but never corrupt, a file
  transfer; a 1 GiB file adds at most 1 ms of chunk latency to clipboard.
  Input is untouched: the driver already prefers input and control over
  bulk and writes one bulk frame per poll round (§1.3).
- Deadline: `Peer::next_deadline` must also wake for a pending file tick,
  like it does for pending clipboard chunks. A pending *hash* or *stage*
  step sets no session deadline; the worker event wakes the loop.
- Inbound expiry: an accepted inbound record (Openergy or classic) that
  sees no data chunk for `max(30 s, 3 × heartbeat)` aborts in session.
  The worker then applies the §4.5 keep/delete rule for the partial.
  This replaces the clipboard receiver's 10 s rule, which is too short
  for a contended bulk queue.

Daemon backlog changes (the §1.4 mechanism, generalized):

- Tag deferred entries by stream: replace the tuple's `ClipboardId` with
  `enum BulkTag { Clipboard(ClipboardId), File(u32) }`. Abort, drain, and
  cap accounting stay per connection. The pull model keeps file depth
  near zero, so the 16 MiB deferred cap remains a clipboard-scale guard,
  not a file buffer.
- Split the stall rule: clipboard keeps 2 s. File streams abort after
  `max(30 s, 3 × heartbeat)` with no driver-accepted bulk chunk. Note the
  heartbeat death rule (3 × heartbeat of no peer traffic) usually fires
  first on a truly stuck socket; the file rule covers a live-but-blocked
  peer (input flows, reads stopped). Abort keeps the partial file.
- Replace `cancel_clipboard`'s whole-queue clear with a per-stream purge.
  Packets already encode full frames; purge by 4-byte code at frame
  offset 4 (`DCLP` vs `OFTR`/`DFTR`), no `Packet` layout change. The
  clipboard abort path must call the `DCLP`-only purge from now on.
  Scenario F8 fails without this change, by design.
- Export `BULK_WINDOW` (today a private constant the daemon mirrors as
  `BULK_GATE_BYTES` with a comment). One source of truth once two bulk
  streams share the gate.

What must never happen: file cancel must not wait behind bulk (control
priority, §4.5); file data must never ride `Action::Send` into control
priority (§1.4); a file offer must never be answered `accept` while the
worker has no staging slot (answer `busy` instead).

## 7. Security model, config, IPC, and logging

### 7.1 Trust and consent

Baseline, unchanged: a joined peer can already inject input and read the
clipboard. `SECURITY.md` is explicit that TLS does not protect against a
compromised peer. Files still raise the stakes in two ways: they persist
after disconnect, and a stored file can be executed later. Policy:

- **TLS gate.** File send and receive require `tls.enabled` and a
  certificate-verified connection. The daemon already tracks
  `certificate_verified` per connection (`ConnInfo`) and passes the
  fingerprint to the server machine via `ConnectionMetadata`. The client
  machine needs the same plumbing before it can send or accept; list it
  in the milestone (§9). Plaintext or unverified: offers are rejected
  with `tls_required`, `file.send` fails with the same code, and the app
  explains how to enable TLS. There is no per-transfer override (§10, D5).
- **Receive consent for explicit sends:** `file.receive_mode` is `prompt`
  (default), `auto`, or `reject`-equivalent via `receive = false`. Prompt
  raises `file.offered` over IPC; the app answers with `file.answer`;
  60 s without an answer rejects with `timeout`. With no GUI attached
  (terminal daemon), prompt still works through `ctl file.answer`, and
  the 60 s default keeps an unattended daemon from accepting silently.
  The daemon, not session, owns the pending-offer wait, exactly like
  held untrusted peers wait for `trust.approve`.
- **Drag consent is weaker by nature.** The host drives the client's
  pointer, so a "drop" on the client was performed by the host user. On
  the receiving client, that is sender intent, not receiver intent.
  Therefore drag receive default is **off**, separate from explicit
  receive. When on, `drag_receive` acts as standing receiver consent for
  drops into the receive directory only.
- Consent remembers nothing across transfers in version 1. No "always
  allow this peer" shortcut in the daemon; `receive_mode = auto` is the
  deliberate way to skip prompts, per local config only.

### 7.2 Confinement, names, and overwrite policy

Receive directory handling, all in the file worker:

- Resolve `file.receive_dir` once per config apply: expand `~`, require
  an absolute result, create it with mode `0700` if missing, then
  canonicalize. All staging and final paths derive from this canonical
  root. If the root is later replaced by a symlink, the next config apply
  or daemon start re-resolves it; within one run, open the root directory
  once and use relative opens where the platform allows it. Whether a
  missing custom path is created or fails config apply is §10, D3.
- Name rules (offer and classic): reject empty; reject `/`, `\`, NUL, and
  control characters; reject `.` and `..`; reject names over 255 bytes;
  reject the reserved staging namespace (`.parts` itself and any name
  starting `.part-`, so a received file can never look like daemon
  staging state); trim trailing dots and spaces (Windows); reject
  Windows drive and ADS forms (`:` anywhere) and reserved device stems
  (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`,
  case-insensitive, with or without extension). Classic `DDRG` paths
  are first reduced to a base name like the reference code does, then
  the same rules apply. No Unicode normalization in version 1: bytes
  are UTF-8 by codec rule.
- Staging lives apart from received files: partials sit in a private
  `<receive_dir>/.parts/` directory (mode `0700`, created with the
  root), named `<xfer>` — daemon-chosen from the daemon-unique handle
  (§4.1), never peer-chosen and never derived from the offered name.
  Final name comes from the validated offer name. Rename from `.parts/`
  to the root is atomic on the same volume and uses no-replace
  semantics: on conflict, append ` (1)`, ` (2)`, … before the
  extension, trying at most 100 suffixes, then fail with
  `name_conflict`. **Overwrite is never a mode.** A peer must not be
  able to replace a file the user already has.
- Stored files get mode `0644` masked by umask, with no execute bits,
  regardless of sender-side permissions. No xattrs, ACLs, timestamps, or
  ownership cross the wire in version 1. macOS quarantine attributes are
  a later hardening item (§10, D10); the daemon never auto-opens a file.
- A kill or crash mid-stage leaves partials in `.parts/`. At startup
  the worker sweeps only that directory: it deletes partials older
  than 24 h (or all of them in milestone 1, where resume is off).
  Completed files in the root are never cleanup candidates, whatever
  their names. Partial cleanup is logged by count and bytes only.

### 7.3 Config

New `[file]` table, mirroring `ClipboardCfg`'s shape and defaults care:

```toml
[file]
enabled = true            # master switch, both features
send = true               # explicit send out
receive = true            # explicit receive in
receive_mode = "prompt"   # prompt | auto
receive_dir = ""          # empty = <state dir>/Received
max_bytes = 1073741824    # 1 GiB; validated <= 16 GiB hard cap
on_conflict = "suffix"    # suffix | reject (never overwrite)
drag_send = false         # phase 4; parsed from milestone 1
drag_receive = false      # phase 4; parsed from milestone 1
classic_send = false      # §5.4 opt-in; classic receive follows drag_receive
classic_max_bytes = 33554432  # 32 MiB cap for classic sends (§5.4)
```

Rules:

- Validation in `Config::validate` beside the clipboard cap: size bounds,
  enum strings through serde, `receive_dir` character hygiene like
  addresses (no control chars). Warnings: plaintext TLS already warns;
  add "file transfer requires TLS; it will stay off" when `file.enabled`
  meets `tls.enabled = false`.
- Live change semantics mirror clipboard: any `[file]` change drops
  session file state, clears deferred file entries, cancels worker
  staging (deleting partials), and rejects pending offers. Implement it
  in the same `apply_config` branch that handles `cfg.clipboard` today.
- Synergy import: map stored `developers.dragAndDrop = true` to a warning
  only ("drag-drop needs explicit file.drag_send/drag_receive"), never to
  enabled drags. Export writes nothing for files: no classic config key
  carries this state, and inventing one would corrupt re-import.
- The server's `DSOP` gains the §3.3 pairs only when `file.enabled`. The
  client's `OFCP` mask clears bits its config disables. Mask bits are
  read from the receiver's side ("I can receive explicit sends" is the
  explicit bit); name them that way at the option constants.

### 7.4 IPC and GUI surface

Daemon-first. New entries in `crates/ipc` (append-only, like past virtual
methods). Every `xfer` below is the daemon-unique handle from §4.1 — the
only transfer identity IPC ever sees; wire IDs and connection IDs stay
inside the daemon:

| Method | Params | Result / errors |
|---|---|---|
| `file.send` | `{ peer, path }` | `{ xfer }`; errors: `peer_unknown`, `peer_no_file_support`, `file_disabled`, `tls_required`, `not_a_file`, `is_directory`, `is_symlink`, `too_large`, `io_error` |
| `file.answer` | `{ xfer, accept }` | `{ ok: true }`; error `unknown_xfer` |
| `file.cancel` | `{ xfer }` | `{ ok: true }`; error `unknown_xfer` |
| `drag.armed` (app → daemon, phase 4) | `{ paths }`, then `{ released }` or `{ cancelled }` | `{ xfer }` |

- `file.send` path policy: the caller is local (owner-only socket), so
  any path the daemon user can read is allowed, like
  `config.import_synergy {path}` today. The worker re-validates per
  §4.6. The GUI passes paths from its own file picker; `ctl file.send`
  serves scripts and E2E.
- Peer addressing: by screen name, as in status `peers[].name`. The
  server resolves it to the one `Ready` connection, the client to its
  server. No connection IDs across IPC (they are runtime-only values).

Events (through `events.subscribe`, same pipe as `clipboard.blocked`):

| Event | Data | Rate rule |
|---|---|---|
| `file.offered` | `{ xfer, peer, name, bytes, kind, expires_unix_ms }` | once per offer |
| `file.progress` | `{ xfer, peer, direction, bytes_done, bytes_total }` | at most 4 per second per transfer, coalesced in the daemon (§1.5 queue bound) |
| `file.completed` | `{ xfer, peer, direction, bytes, kind, verified, unix_ms }` | once |
| `file.blocked` | `{ peer, bytes, limit, reason, unix_ms }` | once per refused offer or pick |
| `file.failed` | `{ xfer, peer, direction, reason, unix_ms }` | once |

Status gains `file: { enabled, receive_dir, active: [{ xfer, peer,
direction, kind, bytes_done, bytes_total, state }], last_completed,
last_blocked }` and `peers[].file = { capability, max_bytes }`. Two
deliberate omissions: status carries **no file names** and no hashes.
Status objects are retained in E2E artifacts and bug reports (§7.5). The
app gets the name from its own send call or from the transient
`file.offered` event and keeps it in memory only.

macOS app work (phase 2, thin): Send File action per connected peer
(`NSOpenPanel` → `file.send`), an offer sheet bound to `file.offered`
(shows name, size, peer; Accept/Reject → `file.answer`), compact progress
in the menu-bar panel from `status.file.active`, and a Files section in
Settings (`receive_dir` picker, toggles, max size). Future Linux/Windows
GUIs need the same four methods only — no new daemon contract.

### 7.5 Audit logging and retained evidence

Follow the clipboard rule already in force: logs and IPC notices carry
byte counts, limits, peer screen names, and reason codes — never payload
content. For files, **the file name is payload-class data**:

- Daemon log lines use transfer ID, peer, direction, byte counts, state,
  and reason. Never a name, never a local path beyond the configured
  receive directory at startup, never a hash. `SECURITY.md` already warns
  that logs can contain paths; this feature must not add per-file paths
  beyond that standing warning.
- E2E reports and retained artifacts follow `tools/e2e/README.md`: names
  and byte counts in step summaries, contents only as hashes. Scenario
  code asserts received files by `assert_file` SHA-256, and fixture file
  names in scenarios are synthetic. `verify_*.py` companions must print
  sizes and digests, not directory listings.
- Retained `verification.*` outputs and lab logs must scrub file names
  the way third-party lab logs are scrubbed today. Add the scrub pattern
  when the lab gains file scenarios (§8.3); treat a leaked name in
  retained evidence as a scenario failure.
- The one place a name appears is the live `file.offered` IPC event and
  the receiver's own filesystem. That matches trust prompts (fingerprints
  also travel live IPC but are summarized, not dumped, in artifacts).

## 8. Phased plan and end-to-end verification

Verification rule for every phase: end-to-end only, through real binaries
and real sockets. **No unit tests, no mocks.** Existing unit tests stay,
but no phase adds one. Each phase extends `tools/e2e/scenarios/` and runs
the full selected catalogue on the candidate commit. A phase is done when
its scenarios pass and all prior scenarios still pass.

Runner additions needed once (milestone 1):

- Step `fixture_file`: writes a deterministic binary fixture in the
  scenario dir from `{ path, bytes, pattern }` and returns its SHA-256.
  It extends the `fixture_dib` idea; the pattern must vary by offset so
  swapped or repeated chunks change the hash.
- Steps `file_send` / `file_answer` / `file_cancel`: thin sugar over
  `ctl` with the §7.4 methods, so scenarios read as user actions.
  `file_send` takes a confined fixture path like `write_file` does.
- Nothing else new: `assert_file` already checks `sha256` and `exists`,
  `wait_event` already matches event payloads, `config_patch` already
  drives live policy changes, and the `code` raw builder can emit `OFOF`,
  `OFTR`, `DDRG`, and `DFTR` payloads as hex.
- Config for all file scenarios: `receive_dir` inside `${dir}`, never a
  host path. The runner's existing confinement checks must grow to cover
  `file.receive_dir`, like import paths today.

### 8.1 Phase list

| Phase | Contents | Gate scenarios |
|---|---|---|
| **0. Classic evidence** (lab only, no product change) | `DSOP` unknown-pair tolerance; `DFTR`/`DDRG` behavior of lab peers; Barrier `--enable-drag-drop` trial | §8.3 captures; write-up amends this file |
| **1. Explicit send core** (= milestone §9) | Protocol, session, daemon worker, config, IPC, backlog purge; Openergy↔Openergy, virtual pair | F1–F10 |
| **2. macOS app + `ctl`** | Send action, offer sheet, progress, Files settings | F1–F10 still pass; manual app checklist §8.4 |
| **3. Resume + queue polish** | §4.5 resume, per-peer send queue, duplicate-name UX | F11, F12 |
| **4. Drag, Openergy peers** | Platform trait + virtual injection, §5.2 exception, macOS app drag reporting; then Windows | F13–F15; manual drag checklist §8.4 |
| **5. Classic receive/send** (only with Phase 0 proof) | §5.4 wire edges behind toggles | F16 + §8.3 lab scenarios |

Phase order is load-bearing: 1 before 4 (data path before OS drag),
0 before 5 and before default-on `DSOP` file pairs, 2 anywhere after 1.

### 8.2 Virtual-pair scenarios (F-series)

All nodes use virtual daemons and assigned loopback ports, like the
present catalogue. Unless a scenario says otherwise, TLS is on with
seeded trust, `receive_mode = "auto"` (each test then covers consent in
F5), and both directions are exercised where the roles allow it.

| ID | Scenario | Pass evidence |
|---|---|---|
| F1 | Small round-trip: 1 KiB and 0-byte files, server→client and client→server | `assert_file` SHA-256 equals `fixture_file` digest; `file.completed` with `verified: true`; sender success only after done frame |
| F2 | Large round-trip: 8 MiB file; input injected mid-transfer; `measure resources` on both daemons | Hash equal; injected keys recorded in order during transfer; RSS growth bounded (no whole-file buffering) |
| F3 | Oversize: receiver `max_bytes` 1 MiB, send 2 MiB; then send 1 MiB | First fails `file.blocked/too_large`, no file, nothing left in `.parts/`; second succeeds; connection and clipboard still work |
| F4 | Cancel: sender cancels mid-transfer (8 MiB, cancel after first progress); receiver-cancel variant | No final file; `file.failed/cancelled` both sides; next send succeeds (no poisoned bulk queue) |
| F5 | Consent: `receive_mode = prompt`; answer reject, then answer accept on a second send | Reject leaves no file and sender sees `user_rejected`; accept stores the file. Offer-timeout arm: see the note below this table |
| F6 | Direction toggles + live patch: `send=false`, `receive=false`, `enabled=false`; patch mid-transfer (Q10 pattern) | Each send fails with `file_disabled`/`disabled`; mid-transfer patch aborts and a fresh send under restored config succeeds |
| F7 | Hostile names via raw peer: `../x`, `a/b`, `a\b`, `C:x`, `CON.txt`, trailing-dot name, 300-byte name, comma name on Openergy path | Every case rejected or safely stored under a valid name inside the receive dir; `assert_file` proves nothing exists outside it; comma name succeeds on the Openergy path |
| F8 | Stall/backpressure: raw stopped reader (A6 pattern) plus a concurrent clipboard send and key stream to a second, healthy client | File abort is visible (`file.failed`), IPC stays responsive, healthy client input and clipboard are unaffected; clipboard abort does not kill a file transfer in the mirrored run (per-stream purge, §6.2) |
| F9 | Kill mid-transfer: `kill` the receiver, restart, resend | Sender fails cleanly; restart starts with no staged state (milestone 1); resend hash-matches |
| F10 | TLS gate: plaintext pair; TLS pair with untrusted (held) peer | `file.send` returns `tls_required` in both cases; no offer bytes on the wire (`raw_expect` absence pattern); clipboard behavior unchanged |
| F11 | Resume (phase 3): kill at ~50% (progress-gated), restart, resend same file | Accept carries `resume_offset > 0`; transferred bytes shrink accordingly (progress totals); final hash equal; changed-content retry restarts at 0 |
| F12 | Conflict: pre-place same-name file, send twice | Original bytes intact (`sha256` unchanged); received copies land as ` (1)`, ` (2)`; `on_conflict = reject` variant fails with `name_conflict` |
| F13 | Virtual drag (phase 4): scripted drag start, edge cross, release | File lands with matching hash; recorded ledger shows no stuck button after release |
| F14 | Kill mid-drag (phase 4): connection cut while drag armed | Remote buttons released on the survivor's next input; no partial file; no stuck drag state after reconnect |
| F15 | Drag refusals (phase 4): two-file drag; folder drag; `drag_receive=false` | `file.blocked/multi_file_drag` or `disabled`; pointer input still flows; nothing staged |
| F16 | Classic wire, raw peer (phase 5): hand-built `DDRG` + `DFTR` with 512 KiB chunks and with 32 KiB chunks | File stored, hash equal; declared-size lie ends in `file.failed`; `DFTR` without `DDRG` ends in `file.blocked` |

Timeouts deserve care: scenario deadlines stay the runner's existing
kind (bounded `timeout_ms`, `--slowdown` for CI). Do not add product test
hooks that shorten the 60 s offer bound; F5's timeout arm either pays the
minute once in a slow-lane scenario or the bound becomes config
(`file.offer_timeout_ms`, default 60000) — prefer the config key, since
headless users may want shorter prompts anyway. Decide in milestone 1.

### 8.3 Interop-lab work (Phase 0 and Phase 5)

Phase 0 runs against the pinned lab peers (Deskflow 1.27.0, Barrier
2.4.0) with Openergy as the raw/virtual side. It answers four questions.
Record answers and packet captures (sizes and digests only, §7.5) in this
file and in `tools/interop-lab/verification.json` style output.

1. Do both peers keep input, clipboard, and connection healthy when
   `DSOP` carries the two extra private pairs? Run the existing I-series
   hello/clipboard checks with file pairs injected by a raw server.
2. What does each peer do on well-formed classic `DDRG` + `DFTR`? Expect
   Deskflow: ignore or drop (feature removed). Expect Barrier: the flag
   is likely ignored on X11 per §2.2, so storage is unlikely; prove what
   the real binary does. Check process
   liveness, connection state, files under the container home and
   desktop paths, and log lines (scrubbed).
3. Does a Barrier client accept 32 KiB-sliced `DFTR` chunks, a basename-
   only `DDRG` payload, and a start total with no trailing NUL? (§3.2.)
4. What does each peer do with an unknown code (`OFCP`-shaped frame sent
   by mistake)? This bounds the blast radius of a capability bug even
   though the design never sends one uninvited.

Phase 5 adds lab scenarios **only** for rows Phase 0 proves working, in
the I-series naming style (`I6-barrier-classic-file`, plain and TLS),
with `session.sh` gaining an explicit drag-drop flag option. Any peer row
Phase 0 cannot prove stays "unsupported" in user docs, with the negative
result kept as a scenario that asserts *no file and a healthy session* —
a passing negative is evidence, not a skipped pass.

### 8.4 Manual acceptance (phases 2 and 4)

Virtual runs cannot prove GUI or OS drag behavior. Add rows to
`docs/verification/native-acceptance.md`: macOS app send/accept/reject/
cancel with real peers on a separate test account; drag on macOS
(Finder → peer) and later Windows; receive-directory permission failures;
disk-full behavior on a small test volume. Follow the standing safety
rule: never drive the maintainer's live desktop for these checks.

## 9. First milestone (exact first PR)

**Milestone 1: "Explicit file send between two Openergy daemons."**
One PR. Openergy↔Openergy only. Virtual-pair verification. It ships the
whole vertical path below, and nothing from phases 2–5.

In scope, by file:

- `crates/protocol`: `Message` variants `FileCapability`, `FileOffer`,
  `FileAnswer`, `FileData` with a nested `FileDataChunk` enum; `u64` codec
  helpers; mark and name-length validation at the codec bound; option
  constants `OFIL`/`OFSZ`; `FILE_CHUNK_BYTES`-adjacent constants belong to
  session (see next). Extend `examples/wire_probe.rs` round-trip lists
  for the four codes (example binary, not a unit test).
- `crates/session`: new `src/file.rs` with `FileTransfers`, reason enum,
  name validation, `FILE_HARD_CAP`; `Peer` fields; `Peer::tick` and
  `next_deadline` wiring; server and client dispatch (offer/answer/data/
  cancel, capability from `DSOP`/`OFCP`); `ServerEvent`/`ClientEvent`/
  `Action` additions per §4.3; config-change reset beside the clipboard
  reset; `decline()` stays for classic codes with the feature off.
- `crates/net`: export `BULK_WINDOW`; code-based bulk purge; no queue or
  priority changes.
- `crates/config`: `[file]` table per §7.3, validation, warnings;
  import warning for `developers.dragAndDrop`; `MAX_FILE_BYTES` constant
  pattern beside `MAX_CLIPBOARD_BYTES`.
- `crates/ipc`: method names `file.send`, `file.answer`, `file.cancel`;
  `Status.file` and `PeerInfo.file`; event name constants.
- `crates/daemon`: file worker (§6.1); `dispatch` handlers; consent hold
  and 60 s expiry; `BulkTag` generalization, file stall rule, `DCLP`-only
  abort purge; `apply_config` file branch; client-side
  `certificate_verified` plumbing into both machines (share the server's
  `ConnectionMetadata` pattern); progress coalescing; partial cleanup at
  startup (delete-all in this milestone).
- `tools/e2e`: `fixture_file`, `file_send`, `file_answer`, `file_cancel`
  steps; scenarios F1–F10. If the Phase 0 `DSOP` tolerance answer is
  still open when this PR lands, the server sends `OFIL`/`OFSZ` only
  when `file.enabled` (per §3.3), and release notes must call out that
  old third-party clients then see two extra unknown option pairs.

Out of scope, explicitly: macOS app screens (status/events must still be
complete for a later GUI), `ctl` sugar beyond the generic method call,
resume offsets above zero, drag of any kind, classic send or receive,
multi-peer fan-out, and `receive_dir` migration if the user moves it
(partials in the old root are orphans until startup cleanup).

Acceptance for the PR: F1–F10 pass; the full non-Docker catalogue passes
(`--no-docker` run); `cargo build --release` and the repo clippy gate
pass; README and `docs/plan/readiness.md` move "no file transfer" to the
new tested limits in the same PR.

## 10. Decisions needed from the maintainer

Defaults below are the recommendations used above. Each is reversible in
config or in a later phase unless it says otherwise.

| ID | Decision | Recommendation (used in this design) | If you choose otherwise |
|---|---|---|---|
| D1 | Ship state | `file.enabled = true`, `receive_mode = prompt`, drag off | Ship `enabled = false`: milestone 1 still works, adoption is slower, and first-run risk is lower |
| D2 | Max size | Default 1 GiB, hard cap 16 GiB | A lower default (for example 256 MiB) only moves F3's numbers; the hard cap must stay ≤ `u64` and above any default |
| D3 | Receive directory | `<state dir>/Received`, created `0700`; missing custom dirs are created too | `~/Downloads` is friendlier but mixes peer files with browser downloads and inherits that folder's ACLs. Fail-on-missing is stricter and can break first run after a volume unmounts |
| D4 | Name conflicts | Suffix ` (1)`, never overwrite | `reject` is available in config. True overwrite is not offered in any case |
| D5 | Plaintext peers | File features hard-require verified TLS | Allowing plaintext matches clipboard today, but stored-file risk is higher (§7.1). The gate is one check per direction either way |
| D6 | Auto-accept | Prompt every explicit receive; `auto` only by config | Per-peer remembered trust would need a trust-store sibling and a revoke story; deferred, not impossible |
| D7 | Classic investment | Fund Phase 0 captures; build Phase 5 only on proof | Skipping classic entirely is coherent: delete §5.4 and keep `DDRG`/`DFTR` discard forever |
| D8 | Symlinks in explicit send | Reject in version 1 | Follow-the-link sends target bytes under the link's name; needs a same-folder escape warning in the app |
| D9 | Non-UTF-8 classic names | Keep codec UTF-8 bound; accept connection loss for that peer class | Carrying `DragInfo` payload as bytes softens it to a dropped drag, at the cost of a codec type change |
| D10 | macOS quarantine xattr | Not in milestone 1; consider in phase 2 | Setting `com.apple.quarantine` aligns with download semantics but triggers Gatekeeper prompts for harmless files |

Two smaller choices ride along: offer timeout as config
(`file.offer_timeout_ms`, §8.2) yes/no, and the classic-send cap
(32 MiB proposed). Neither blocks milestone 1.

## 11. Risks, compatibility impact, and rollback

- **Highest risk: bulk fairness regressions.** Two bulk users on one FIFO
  can reintroduce the stalls the deferred backlog was built to fix. The
  §6.2 contract (shared pacing, clipboard first, per-stream purge) is
  the mitigation; F2 and F8 are the proof. Do not merge milestone 1 with
  F8 red.
- **Disk and worker faults are new crash surface.** The file worker is
  the only new thread and the only new disk writer. It must fail closed:
  any staging error aborts one transfer and reports a reason; it never
  panics the I/O loop and never writes outside the canonical root (§7.2).
- **Capability bootstrap depends on tolerated unknown `DSOP` pairs.** If
  Phase 0 finds a peer that disconnects on them, the fallback is to gate
  `OFIL` emission on an explicit config flag (default off for users with
  third-party clients) until a safer channel exists. The rest of the
  design is unchanged.
- **Config downgrade:** old daemons reject the new `[file]` table
  (`deny_unknown_fields`). Rollback = remove the table (or keep an old
  config copy) before starting the older binary. Wire rollback is clean:
  with `file.enabled = false`, behavior returns to today's parse-and-
  discard for `DFTR`/`DDRG`, no `DSOP` pairs, no new codes.
- **No frozen-contract breaks.** Protocol, session, and IPC changes are
  additive variants, methods, and status fields. The e2e runner accepts
  added status fields by design (subset matching). Clipboard, input, and
  trust paths change only at the two named spots: per-stream bulk purge
  and `apply_config`'s new branch.
- **Docs debt on ship:** `docs/plan/readiness.md` ("No file transfer"),
  `README.md`, `SECURITY.md` data-paths paragraph, and
  `crates/config/V1-LIMITS.md` must move in the shipping PR, or the
  limits text will lie in the safe direction and confuse support.

## Appendix A. Frame-budget arithmetic

`BULK_SLICE` = 32 768 counts whole encoded bytes, including the 4-byte
length prefix (see `Mailbox::prepare`, which measures the encoded frame).

- `OFTR` data frame: 4 length + 4 code + 4 transfer + 1 mark + 4 payload
  length = 17 bytes of overhead. Max payload = 32 751 bytes.
- `DFTR` data frame: 4 + 4 + 1 + 4 = 13. Max payload = 32 755 bytes.
- `DCLP` for comparison: 4 + 4 + 1 id + 4 sequence + 1 mark + 4 length
  = 18, giving the existing `CHUNK_BYTES = 32 768 - 18`. The file
  constants use the same derivation: define them from a named overhead,
  never as bare numbers.
- At one chunk per millisecond per peer, the ceiling is about 31 MiB/s
  gross per direction, shared with clipboard. A 1 GiB file takes at
  least ~33 s at full pace on an idle LAN peer. This is a pacing choice,
  not a wire limit; raising it later means revisiting input-latency
  evidence (S15-style `measure` runs), not just the constant.

## Sources

Inspected during this design pass; claims above stay within what these
say. Repo files are cited inline by path.

- [Deskflow protocol types, v1.27.0 (DFTR/DDRG deprecated definitions)](https://raw.githubusercontent.com/deskflow/deskflow/v1.27.0/src/lib/deskflow/ProtocolTypes.h)
- [Deskflow FileChunk.cpp, v1.21.0 (marks, decimal start, assembly)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/deskflow/FileChunk.cpp)
- [Deskflow StreamChunker.cpp, v1.21.0 (512 KiB chunks, interrupt, keep-alive)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/deskflow/StreamChunker.cpp)
- [Deskflow DragInformation.cpp, v1.21.0 (comma-joined drag string)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/deskflow/DragInformation.cpp)
- [Deskflow DropHelper.cpp, v1.21.0 (first-file-only write)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/deskflow/DropHelper.cpp)
- [Deskflow ClientProxy1_5.cpp, v1.21.0 (1.5-level parsing)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/server/ClientProxy1_5.cpp)
- [Deskflow base String.cpp, v1.21.0 (decimal size conversion)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/base/String.cpp)
- [Deskflow ArgParser.cpp, v1.21.0 (`--enable-drag-drop`, ignored on Linux/X11)](https://raw.githubusercontent.com/deskflow/deskflow/v1.21.0/src/lib/deskflow/ArgParser.cpp)
- [Deskflow release v1.22.0 ("Remove the broken drag and drop support")](https://github.com/deskflow/deskflow/releases/tag/v1.22.0)
- [Synergy user FAQ (single-file drag-and-drop, Windows/macOS, desktop drop)](https://github-wiki-see.page/m/extrealm/synergy/wiki/User-FAQ)


