# Openergy: Synergy 3 core wire protocol and macOS platform report

Date: 2026-10-21. Research and documentation only. No implementation is included.

## 1. Scope, evidence, and confidence

**The inspected product is `synergy-core 1.21.1-beta`; its advertised wire version is `1.8`.** Do not send `1.21` as the wire version. The main interoperability target is the `Synergy` dialect at `1.8`, with TLS established before application framing. This report is a static specification of the supplied macOS build. It is not a captured session with the Windows server.

| Item | Result / evidence |
|---|---|
| Original | `/Applications/Synergy.app/Contents/MacOS/synergy-core`, arm64 Mach-O, 7,003,696 bytes |
| Original file SHA-256 | `616e13a00b9981bbff0fea25f73a162c410e86cc21eac78eee198f4513e9bd2b` — executable identity, not a certificate fingerprint |
| Preserved analysis input | `/tmp/openergy-re/core/synergy-core.pristine`, byte-for-byte copy |
| Static address convention | Unslid virtual addresses from `nm -n -C`; image base `0x100000000`. These are not live process addresses. |
| Tooling | REA 6.7.0: `doctor`, `--help`, `capabilities`, `inspect-macho`, and native `function` analysis. Ghidra 12.1.4 with Homebrew JDK 21 supplied the native provider. |
| Provider setup | Initial `doctor` found no configured Ghidra and Java 17 was too old. Setting `GHIDRA_INSTALL_DIR` to the installed Ghidra `libexec` directory and `JAVA_HOME` to the installed JDK 21 made native analysis succeed within the requested time budget. No provider fallback was required. |
| Supplemental analysis | Ghidra headless exported selected functions in bulk; `otool -tvV`, `nm`, binary data reads, ASCII/UTF-16 string extraction cross-checked constants and call arguments. Temporary analysis scripts and decompiler output remain under `/tmp/openergy-re/core/`. |
| Safe execution | A separate copy, `/tmp/openergy-re/core/synergy-core`, ran only `--help` and `--version`, each through `timeout 10`. Its rpath was extended to locate the installed Qt frameworks and it was ad-hoc signed. These changes were confined to that copy. All final static addresses/data were checked against the pristine copy. |
| Live system boundary | No Synergy process was attached, signalled, stopped, or restarted. No client/server mode was run. No connection to the Windows host or any host on port 24800 was attempted. No live settings, certificates, trust database, licence, or machine identity values were copied into this report. |

Evidence labels used below:

- **V**: verified in instructions, data, or an executed help/version command on the copy.
- **I**: inference from verified code paths; not established by a live experiment.
- **O**: unresolved. Do not turn it into a Rust constant or interoperability promise.

The decompiler often loses C++ varargs and virtual-call types. Field order was therefore checked against the call-site argument stores and the format-string data. No decompiled function body is reproduced here.

### Evidence index

| ID | Binary evidence / artifact | What it establishes |
|---|---|---|
| E01 | `PacketStreamFilter::readPacketSize` `0x1000987DC`, `write` `0x1000988A8`, `read` `0x100098668` | Packet framing and receive limit |
| E02 | `ProtocolUtil::vreadf` `0x10009D3D4`, vector writer `0x10009D7BC`, integer readers `0x10009DFA8` / `0x10009DFFC` / `0x10009E054`, `readVectorSize` `0x10009F524` | Scalar, string, and vector encoding |
| E03 | `_kMsg*` pointer table `0x10048FE58–0x10048FF80`; strings `0x10043E040–0x10043E1B2` | All message signatures in §4 |
| E04 | `Client::handleHello` `0x100014E38`; `ClientProxyUnknown` constructor `0x10004B110`, `handleData` `0x10004BEB4`, `initProxy` `0x10004BC58` | Hello dialects, versions, limits, errors |
| E05 | `ServerProxy::parseHandshakeMessage` `0x100019DC8`, `parseMessage` `0x10001AD9C`, `handleData` `0x10001A818`; `ClientProxy1_0::handleData` `0x100038440`, handshake parser `0x100038820`, normal parser `0x100038C08` | Dispatch, setup state, unknown messages |
| E06 | `sendInfo` `0x10001C700`, `recvInfo` `0x1000389AC`, `enter` `0x10001BFFC`, `leave` `0x10001C0D8` | Screen information, sequence, enter/leave |
| E07 | `ClipboardChunk` start/data/end/assemble/send `0x10008906C` / `0x100089128` / `0x1000891E4` / `0x10008925C` / `0x1000894C4`; `StreamChunker::sendClipboard` `0x1000ABB3C` | Clipboard chunks and chunk size |
| E08 | `IClipboard::unmarshall` `0x10008A718`, `marshall` `0x10008A928`; text/HTML/BMP format getters `0x10001EA94` / `0x10001FFB0` / `0x10001F2F8` | Clipboard inner format and IDs |
| E09 | `Config::readSectionOptions` `0x1000548E4`, `readSectionScreens` `0x100055774`; `ServerProxy::setOptions` `0x10001ABB4`; `Screen::setOptions` `0x1000A0D40`; `Client::setOptions` `0x1000141D8` | Option IDs, units, scope |
| E10 | `ClientProxy1_3` constructor `0x10003B934`, rate setter `0x10003BC64`, `keepAlive` `0x10003BDF8`; `ServerProxy::setKeepAliveRate` `0x10001A4E0` | Heartbeat defaults and multiplier |
| E11 | `_kKeyNameMap` `0x10048ED48`; `_kModifierNameMap` `0x10048F818`; `s_controlKeys` `0x100356D0C` | Key constants and static Mac translation table |
| E12 | `SecureSocket::initContext` `0x1000BAF50`, connect `0x1000BBAC4`, accept `0x1000BB3B8`, pin check `0x1000BB5F0`; certificate/fingerprint helpers `0x1000C291C` / `0x1000C2654` | TLS and certificates |
| E13 | `Fingerprint::toDbLine` `0x1000B26E4`, `fromDbLine` `0x1000B2A4C`, database read `0x1000B3178`, match `0x1000B3788` | Trust file syntax and matching |
| E14 | UTF-16 key strings `0x100438636–0x100438DF6`; `Settings::defaultValue` `0x10033D08C`, `serverConfigFile` `0x10033E214`; settings symbol objects `0x100508130` onward | INI catalog and defaults |
| E15 | `Config` section readers `0x100053CD4`, `0x100056350`, `0x100056BD0`; parser helpers `0x100057D6C`, `0x100057E38`, `0x100058480`, `0x100059B64` | Server configuration grammar |
| E16 | `Server::switchScreen` `0x10007141C`, neighbor mapping `0x100071FD4` / `0x100072380`, `isSwitchOkay` `0x100072B74`, `armSwitchTwoTap` `0x100073488`, disconnect recovery `0x100075A98` | Switching behavior |
| E17 | `OSXScreen::enable` `0x100028EE0`, button injection `0x1000286C0`, movement `0x100028368`, relative movement `0x100028AA8`, wheel `0x100028BCC` | macOS capture and injection |
| E18 | `CoreIpcServer` `0x1000B1D70`, `DaemonIpcServer` `0x1000B09BC`, common read/write/dispatch `0x1000ADA94` / `0x1000AE8D0` / `0x1000ADF40` | Local IPC |
| E19 | `/tmp/openergy-re/core/help.txt`, `version.txt` | Actual CLI output from the copy |

## 2. Transport, framing, and primitive values

TLS, when enabled, wraps the entire application stream. There is no STARTTLS message. When TLS is disabled, the same framed bytes go directly over TCP. TCP packet boundaries and TLS record boundaries have no application meaning. [V: E01, E12]

Each application packet is:

| Offset | Size | Encoding |
|---|---:|---|
| 0 | 4 | Unsigned big-endian payload byte count `N`; excludes these four bytes |
| 4 | `N` | Payload: usually a four-byte ASCII message code, then fields. The hello is the exception. |

The receive limit is **`0x00400000` = 4,194,304 payload bytes**, inclusive. `readPacketSize` compares the decoded unsigned size with this value and raises an input-error event when it is larger. The writer adds the length prefix but has no equivalent send-size guard in `PacketStreamFilter::write`. [V: E01]

| Format | Wire bytes | Interpretation / limits | Evidence |
|---|---|---|---|
| `%1i` | 1 | Low eight bits on write; byte on read. Signedness comes from the receiving field, not from the format. | E02 |
| `%2i` | 2, big-endian | Low 16 bits on write. Coordinates/deltas are read as signed two's-complement `i16`; key IDs/masks/buttons as `u16`. | E02, E06, mouse/key readers |
| `%4i` | 4, big-endian | Low 32 bits on write. Clipboard/enter sequence values are `u32` bit patterns. | E02, E06, E07 |
| `%s` | `u32be length`, then exactly that many bytes | No NUL terminator on wire; supports embedded zero bytes. Text encoding is defined by the message. Read limit **1,048,576 bytes**, inclusive. | `vreadf` compare at `0x10009D52C` |
| `%7s` | Exactly 7 bytes | Fixed-width string, with no inner length. Used for the hello protocol name. | E02, E03 |
| `%4I` | `u32be element_count`, then `element_count` × `u32be` | Count is a count of integers, not bytes or option pairs. Reader accepts at most **1,048,576 elements**; the packet cap is the tighter practical bound for a complete DSOP packet. | E02, `readVectorSize` |

Representable scalar ranges are u8 0–255, u16 0–65,535, i16 −32,768–32,767, and u32 0–4,294,967,295. Message-specific constraints below are tighter. There is no general UTF-16 string framing. Computer names, layout identifiers, secure-input app names, and clipboard text use UTF-8 where described. Bitmap and clipboard-container strings are binary.

### Invalid, short, and unknown packets

- The packet filter waits for the complete declared packet before making it ready. A short TCP read is not by itself a protocol error. [V: E01]
- After hello, handlers read a four-byte code. A completed packet that supplies only one to three code bytes produces an “incomplete message” disconnect. An empty packet does not provide a code. [V: E05]
- An **unknown code is logged and the currently available stream data is drained**, rather than always causing an immediate EBAD reply. The client and server proxy paths both do this. Do not assume unknown extensions can be skipped by parsing their fields. The drain can discard more already-buffered data than just the unknown code. [V: `ServerProxy::handleData` `0x10001A818`, `ClientProxy1_0::handleData` `0x100038440`]
- A malformed known message can throw a read/mismatch exception and disconnect. The initial server hello-back path sends EBAD for a bad-client/protocol exception and EICV for a version exception. Established parse exceptions do not imply a guaranteed EBAD response. [V: E04/E05, exception paths in disassembly]
- Use one logical message per application packet. No clean “ignore trailing bytes” contract was established. [I: E01/E05]

## 3. Hello, version selection, and setup

### Exact hello formats

Server hello payload:

`protocol_name[7] | major:u16be | minor:u16be`

For the Synergy dialect, the bytes are:

`53 79 6e 65 72 67 79 00 01 00 08`

With framing:

`00 00 00 0b 53 79 6e 65 72 67 79 00 01 00 08`

Client hello-back payload:

`protocol_name[7] | major:u16be | minor:u16be | name_byte_length:u32be | name_utf8[name_byte_length]`

The name is not NUL terminated and is not a machine UUID. It must match a configured screen or alias at the server. The full hello-back payload must fit **1,024 bytes**, so the structural maximum name length is **1,009 bytes** (`1024 - 7 - 2 - 2 - 4`). Name validation may impose further restrictions. [V: E03/E04]

| Constant / rule | Verified result | Evidence |
|---|---|---|
| Product version | `1.21.1-beta` | E19 and literal `0x1004394EC` |
| Advertised wire version | Major `1`, minor `8` | E19; hello/hello-back writer arguments in E04 |
| Synergy protocol name | Seven bytes `Synergy`, exact case on normal writes | UTF-16 literal `0x100438E58`, E04 |
| Barrier protocol name | Seven bytes `Barrier` | UTF-16 literal `0x100438E68`, E04 |
| Server dialect selection | Configuration accepts `protocol = synergy` or `protocol = barrier`; protocol selection is separate from TLS | E09/E15, `Server::protocolString` `0x100071020` |
| Client dialect acceptance | Compares the received seven-byte name with `synergy` and `barrier`, case-insensitively; echoes the received spelling | `Client::handleHello` `0x100014E38` |
| Client version reaction | Reads the server numbers but sends its own fixed `1.8`; no minimum-of-minors negotiation or rejection by server version was found in this function | E04 |
| Server version acceptance | **Exactly major 1 and minor 0 through 8 inclusive.** It chooses the corresponding `ClientProxy1_n`. Major 2, minor 9, and negative signed version fields fail. | `initProxy` `0x10004BC58`, compare/jump table |
| Server protocol-name validation | The hello-back parser consumes a seven-byte name into a temporary string but does not compare that field before selecting the proxy. Send the expected dialect anyway. | `handleData` `0x10004BEB4` |

**Interoperability choice:** send `Synergy` and `1.8` to the unmodified Synergy server. When acting as a server for this Mac client, advertise `Synergy 1.8`. Advertising an older server minor will not make this client downgrade its reply. [V/I: E04]

### Version-specific server proxies

These are the actual classes and overrides, not a claim that every historical peer still works.

| Minor | Build behavior / added override | Evidence |
|---:|---|---|
| 0 | DKDN/DKUP have key and mask only; DKRP has key, mask, count; DMWM has only vertical field. Clipboard data send/receive is stubbed in this build's base proxy. | `ClientProxy1_0` methods `0x10003956C`, `0x100039648`, `0x10003972C`, `0x100039A88`, `0x100039478`, `0x100039F54` |
| 1 | Adds the source key-button field. **Its repeat writer in this build also uses the current DKRP format with a trailing `%s` language string.** | `ClientProxy1_1` `0x10003A9F0`, `0x10003AAD8`, `0x10003ABE8`; E03 |
| 2 | Adds DMRM relative mouse movement. | `0x10003B2B4` |
| 3 | Uses x/y wheel deltas and CALV; defaults to 3 s heartbeat / 9 s timeout. | `0x10003BB30`, `0x10003BC0C`, E10 |
| 4 | Adds a stored Server reference; constructor adds no new wire message here. | `0x10003C480` |
| 5 | Recognizes DFTR/DDRG codes in the server proxy, but transfer methods are no-ops and no payload parser is implemented there. | `0x10003F904–0x10003F954` |
| 6 | Implements chunked DCLP send/receive. | `0x100042F4C`, `0x1000430E0` |
| 7 | Adds SECN secure-input notification. | `0x100046D80` |
| 8 | Sends LSYN during construction and emits DKDL for key-down, including language. | `0x10004A7BC`, `0x10004A804`, `0x10004AA64` |

The current Mac client has one current-message parser. It is not a family of historical client decoders. In particular, legacy DKRP packets without the current language suffix are not a safe target. [V: E03/E05]

### Setup state machine

| State | Server behavior | Client behavior / transition | Evidence |
|---|---|---|---|
| TCP / optional TLS | Completes SSL_accept first when enabled | Completes SSL_connect and server pin check first | E12; listener `0x100031A6C` / `0x100031D14` |
| Await hello | Sends framed hello | Parses hello; sends hello-back `1.8`; creates ServerProxy | E04 |
| Await DINF / options | Creates versioned proxy. Base constructor sends QINF. Version 1.8 constructor also sends LSYN. | Setup parser accepts QINF, CIAK, CROP, DSOP, CALV, CNOP, LSYN, CBYE and setup error messages. QINF produces DINF. | E04/E05; constructor `0x100037B60` |
| Information accepted | Valid DINF causes CIAK; proxy becomes available for server adoption | CIAK clears the “ignore mouse until info acknowledged” flag | E05/E06 |
| Adoption | Rejects unrecognized screen/alias with EUNK; duplicate connected screen with EBSY. Otherwise sends CROP then DSOP. | **DSOP completes client setup** and changes to the normal parser; the screen is enabled. | `Server::adoptClient` `0x100070494`, `sendOptions` `0x10006FAE0`, `Client::handshakeComplete` `0x100013D60` |
| Connected, inactive | Client exists but pointer is on another screen | Accepts maintenance/clipboard messages; input normally starts after CINN | E05/E06 |
| Entered | Sends CINN with entry position, sequence and modifier mask, then input events | Makes its screen active; COUT leaves and releases synthetic keys | E06; §9 |
| Disconnect | CBYE, error, heartbeat expiry, or socket close | Cleans up stream/screen; no CBYE acknowledgement is required | E05/E10 |

LSYN must arrive **before the first DSOP** for this client. The normal parser does not recognize LSYN. The observed server construction order is QINF, then LSYN, followed by adoption traffic after DINF. Separate TCP delivery may change when the client sees each queued packet. [V: E05, `ClientProxy1_8` constructor]

### Timeouts and error replies

| Value / code | Meaning / trigger | Evidence |
|---|---|---|
| 2 seconds | Client connection/initial-hello timer; set when connecting and removed after hello | `Client::connect` `0x100012F08`, `setupTimer` `0x1000136F4`, `handleHello` |
| 30 seconds | New server-side unknown-client handshake deadline | `ClientListener::handleClientAccepted` `0x100031D14`, argument to unknown proxy |
| 3 seconds | Default modern server heartbeat interval | E10 |
| 9 seconds | Default client keepalive alarm and server flatline interval, three times heartbeat | E10; `ServerProxy` constructor `0x100019B50` |
| `EICV major:u16 minor:u16` | Unsupported client version; this server replies with its own supported maximum `1,8` | `ClientProxyUnknown::handleData` exception handler, writer at `0x10004C1D0` |
| `EBSY` | Same canonical screen already connected, including alias collision | `Server::adoptClient` `0x100070494`, `addClient` `0x10006E494` |
| `EUNK` | Name is not an allowed screen/alias in server config | `Server::adoptClient` |
| `EBAD` | Initial hello-back parse/bad-client failure; also understood as a server-reported protocol failure | Writer `0x10004C22C`; E05 |

## 4. Complete message table for the current wire dialect

Direction: **S→C** means keyboard/mouse server to client; **C→S** means client to server. State **H** is setup after hello; **R** is connected, whether entered or inactive; **A** is the normal entered-screen use. These states describe intended use. The normal dispatcher does not reject every input packet solely because the screen is inactive.

All sizes below exclude the four-byte outer packet length. Field order is wire order. Each row's signature address is a direct constant verification in E03.

| Message | Direction; state | Fields after code | Payload bytes | Meaning / ranges | Signature address; handler |
|---|---|---|---:|---|---|
| QINF | S→C; H/R | None | 4 | Request screen information; client answers DINF | `0x10043E198`; `queryInfo` `0x10001AAC4` |
| CNOP | Both; H/R | None | 4 | No-op / processing acknowledgement; not the modern heartbeat | `0x10043E066`; E05 |
| CBYE | S→C; H/R | None | 4 | Close this connection; no reason string or acknowledgement | `0x10043E06B`; E05 |
| CINN | S→C; R→A | `x:i16, y:i16, sequence:u32, modifier_mask:u16` | 14 | Entry position in client's reported screen coordinates; sequence associates clipboard ownership with an entry; mask synchronizes modifiers/toggles | `0x10043E070`; `enter` `0x10001BFFC` |
| COUT | S→C; A→R | None | 4 | Leave; flush queued movement, release synthetic keys, send dirty clipboard data when enabled | `0x10043E081`; `leave` `0x10001C0D8`, `Client::leave` `0x100013E64` |
| CCLP | Both; R | `clipboard_id:u8, sequence:u32` | 9 | Clipboard ownership/grab notification, not the clipboard payload. IDs 0 and 1 only. | `0x10043E086`; `grabClipboard` `0x10001C15C`, server recv `0x100038DE8` |
| CSEC | S→C; R | `on:u8` | 5 | Server screensaver state, 0=off / nonzero=on; emit 0 or 1 | `0x10043E091`; `screensaver` `0x10001C1E0` |
| CROP | S→C; H/R | None | 4 | Reset half-duplex options, modifier mapping and keepalive alarm defaults; follow with DSOP when configuring | `0x10043E099`; `resetOptions` `0x10001B2E0` |
| CIAK | S→C; H/R | None | 4 | Acknowledge valid DINF; re-enable mouse handling after shape update | `0x10043E09E`; `infoAcknowledgment` `0x10001AB7C` |
| CALV | S→C then C→S; H/R | None | 4 | Server keepalive; client echoes CALV and resets its keepalive deadline | `0x10043E0A3`; E10 |
| DKDN | S→C; A | `key_id:u16, modifier_mask:u16, key_button:u16` | 10 | Key down without language. Accepted by this client; current 1.8 server normally sends DKDL. | `0x10043E0B8`; E05, `keyDown` `0x10001B6DC` |
| DKDL | S→C; A | `key_id:u16, modifier_mask:u16, key_button:u16, language:%s` | `14+L` | Language-aware key down; language typically a two-byte layout-language code such as `en`; empty is structurally valid | `0x10043E0A8`; writer `0x10004AA64`, E05 |
| DKRP | S→C; A | `key_id:u16, modifier_mask:u16, repeat_count:u16, key_button:u16, language:%s` | `16+L` | Repeat count is the third field; sender count truncates to 16 bits. Use positive counts. **Trailing string length is mandatory in this build.** | `0x10043E0D1`; reader `0x10001BD14`, writer `0x10003AAD8` |
| DKUP | S→C; A | `key_id:u16, modifier_mask:u16, key_button:u16` | 10 | Release the matching source key button. No language suffix. | `0x10043E0F2`; `keyUp` `0x10001B930` |
| DMDN | S→C; A | `button:u8` | 5 | Mouse button down; portable IDs 1–5 | `0x10043E10B`; `mouseDown` `0x10001BB9C` |
| DMUP | S→C; A | `button:u8` | 5 | Mouse button up, same ID | `0x10043E113`; `mouseUp` `0x10001BC58` |
| DMMV | S→C; A | `x:i16, y:i16` | 8 | Absolute global client coordinates. Last queued absolute move can replace earlier queued moves. | `0x10043E11B`; `mouseMove` `0x10001B408` |
| DMRM | S→C; A | `dx:i16, dy:i16` | 8 | Relative deltas; x right / y down. Queued relative moves are accumulated. | `0x10043E126`; `mouseRelativeMove` `0x10001B50C` |
| DMWM | S→C; A | `x_delta:i16, y_delta:i16` | 8 | Signed wheel units; 120 is one conventional wheel notch. See §7 for Mac scaling. | `0x10043E131`; `mouseWheel` `0x10001B614`, writer `0x10003BB30` |
| DCLP | Both; R | `clipboard_id:u8, sequence:u32, chunk_mark:u8, chunk:%s` | `14+L` | Chunked clipboard transfer; numeric marks 1/2/3. See §5. | `0x10043E144`; E07 |
| DINF | C→S; H/R | `x:i16, y:i16, width:i16, height:i16, obsolete_zone:i16, cursor_x:i16, cursor_y:i16` | 18 | Client sends obsolete_zone=0. Width/height must be 1–32767. Out-of-bounds cursor coordinates are replaced with screen center by server. | `0x10043E154`; E06 |
| DSOP | S→C; H/R | `values:%4I` | `8+4K` | Flat sequence of option-ID/value integer pairs; K is integer count, normally even | `0x10043E16E`; E09 |
| DFTR | C→S tag recognized; unsupported here | `mark:u8, payload:%s` | `9+L` | Signature exists, but there is no working transfer state machine in this build; do not infer marks or file content format from DCLP | `0x10043E176`; stubs `0x10003F908`, `0x10003F950`, E05 |
| DDRG | C→S tag recognized; unsupported here | `metadata_field:u16, payload:%s` | `10+L` | Signature exists; the 16-bit field is often described as a file count in ancestor protocols, but its meaning is not established by these no-op methods; no working decoder/encoder here | `0x10043E180`; stubs `0x10003F904`, `0x10003F954` |
| LSYN | S→C; **H only** | `languages:%s` | `8+L` | Concatenated two-byte language codes, e.g. `enfr`; no separator or inner count. Used before first DSOP. | `0x10043E191`; serializer/parser `0x1000AC988` / `0x1000AC4E0` |
| SECN | S→C; R | `application_name:%s` | `8+L` | UTF-8 name of app blocking server keyboard capture. This client's handler logs it; it is not an input-encryption toggle. | `0x10043E18A`; `0x100046D80`, `0x10001C414` |
| EICV | S→C; H | `major:u16, minor:u16` | 8 | Incompatible version; supported version is 1.8 | `0x10043E19D`; E04 |
| EBSY | S→C; H | None | 4 | Duplicate connected name | `0x10043E1A8`; adoption |
| EUNK | S→C; H | None | 4 | Unknown configured name | `0x10043E1AD`; adoption |
| EBAD | S→C; H/R | None | 4 | Protocol failure | `0x10043E1B2`; E04/E05 |

Legacy-only signatures retained in data are DKDN `%2i%2i` at `0x10043E0C6`, DKRP `%2i%2i%2i` at `0x10043E0E4`, DKUP `%2i%2i` at `0x10043E100`, and DMWM `%2i` at `0x10043E13C`. Do not use these with a 1.8 peer. [V: E03]

### Acknowledgements, movement, and liveness

The normal client parser sends **CNOP after a recognized normal message**, including a received CNOP. For CALV it first echoes CALV, then reaches the normal CNOP write. During setup, CALV is echoed without that common normal-parser CNOP path. The server accepts CNOP without echoing it, so this does not form an infinite echo loop. [V: E05, E10]

The modern server sends CALV on its own timer. Client CALV resets the server flatline timer. The server's data handler also resets its heartbeat timer after draining available valid input, so normal client traffic proves activity. The client's keepalive alarm is reset by CALV and option/reset processing; do not substitute arbitrary server input for CALV. A heartbeat option of 0 disables the associated timers; use a positive interval for ordinary sessions. [V: E05/E10 and `ClientProxy1_0::setOptions` `0x100039DC0`]

DMMV/DMRM are compressed in the client before other input actions. Absolute movement retains the last position; relative movement sums deltas. Leave and key/button handlers flush pending movement to maintain event order. CIAK matters after a screen-shape update: movement can be ignored until the information acknowledgement arrives. [V: E05/E06]

## 5. Clipboard ownership and chunk transfer

### Ownership, IDs, and sequence

| Constant / behavior | Meaning | Verification |
|---|---|---|
| Clipboard ID 0 | Normal clipboard | E07/E08; Mac clipboard getter ignores the supplied ID and uses its shared pasteboard |
| Clipboard ID 1 | Selection clipboard in the cross-platform model | `recvGrabClipboard` checks `<2` at `0x100038DE8`; two clipboard slots in Client/Server constructors |
| IDs ≥2 | Invalid for ownership notifications and completed chunks | `recvGrabClipboard`, `ClipboardChunk::assemble` |
| Enter sequence | Server increments its `u32` screen-switch serial, sends it in CINN, and the client reuses it in client-originated CCLP/DCLP | `Server::switchScreen` `0x10007141C`, `ServerProxy::enter`, `onGrabClipboard` `0x10001C5C8`, `onClipboardChanged` `0x10001C630` |
| Server-originated DCLP sequence | This build sends 0 | `ClientProxy1_6::setClipboard` `0x100042F4C`, StreamChunker argument |
| Clipboard grabbed locally | Announces ownership with CCLP; dirty data can be sent when leaving | `Client::handleClipboardGrabbed` `0x100014DC8`, `Client::leave` `0x100013E64` |
| Sequence filtering | Remote grabs require incoming sequence >= the last accepted sequence for that clipboard; the primary screen has a bypass. Data updates older than that stored clipboard sequence are ignored. | `Server::handleClipboardGrabbed` `0x1000742CC`, `onClipboardChanged` `0x100071B40` |

The sequence comparison is ordinary unsigned ordering, not an observed wrap-safe serial-number algorithm. Server-originated CCLP uses sequence 0, and the Mac client reads but does not use that received sequence in its grab handler. [V: `Server::handleClipboardGrabbed`, `onClipboardChanged`, `ClientProxy1_0::grabClipboard` `0x10003947C`, `ServerProxy::grabClipboard` `0x10001C15C`]

CCLP is not a “get clipboard data” RPC with a matching response ID. The server tracks who owns each clipboard and which clients need updated data. Enter/leave and clipboard changes cause later DCLP transfer. Do not send DCLP in response to every CCLP without tracking ownership.

### DCLP outer chunk

Every chunk has this payload layout:

| Offset | Size | Field |
|---|---:|---|
| 0 | 4 | ASCII `DCLP` |
| 4 | 1 | Clipboard ID |
| 5 | 4 | Big-endian sequence |
| 9 | 1 | Numeric mark |
| 10 | 4 | Big-endian chunk byte length |
| 14 | variable | Chunk bytes |

| Mark | Chunk contents | Receiver action | Evidence |
|---:|---|---|---|
| `0x01` | UTF-8/ASCII decimal representation of the total serialized clipboard byte count; no terminator | Parses base-10 size, clears accumulated data | E07; `ClipboardChunk::start`, `assemble`; StreamChunker calls `QString::number` |
| `0x02` | Binary slice of the serialized clipboard | Appends bytes | E07 |
| `0x03` | Empty `%s`, therefore four zero length bytes remain present | Checks clipboard ID and exact assembled byte count, then publishes clipboard data | E07 |

The sender uses **`0x80000` = 524,288 bytes per data chunk**. That is below the 1 MiB `%s` limit. Do not copy the in-memory chunk object's native-endian sequence storage to the wire: its `send` method passes that field through ProtocolUtil, which writes big-endian. [V: E07, `ClipboardChunk::send` `0x1000894C4`]

The inspected receiver uses static expected-size/assembly state, not a robust independently multiplexed transfer-ID protocol. Serialize transfers on a connection. There is no per-chunk acknowledgement message or chunk offset in DCLP. The sender queues start, data, and end in order. CNOP is normal message processing traffic, not a DCLP-specific ACK. [V: E07; `ServerProxy::setClipboard` `0x10001C260`, server receiver `0x1000430E0`]

The assembler does not establish a useful hard cap on the *total* reconstructed clipboard. The configured send cap and per-packet/per-string caps are separate. This distinction matters for an original Rust implementation's memory bounds. [V: E07/E09]

### Inner clipboard container

The reconstructed data is:

`format_count:u32be | repeated { format_id:u32be | data_length:u32be | data[data_length] }`

There is no alignment padding and no extra per-format string terminator. The writer considers three format slots, in ascending order 0, 1, 2. The reader validates the four-byte container header, each eight-byte format header, and each payload length. Valid format IDs are 0, 1 and 2. Ordinary unrecognized positive IDs at or above 3 are skipped by their declared data length. The recovered dispatch comparison is signed; do not treat high-bit-set IDs as valid formats in an original parser. Truncated data is logged, and the update stops. [V: E08]

| Format ID | Name | Wire representation | macOS conversion / evidence |
|---:|---|---|---|
| `0` | Text | UTF-8 bytes; not a Windows CF_UNICODETEXT UTF-16 buffer | `OSXClipboardAnyTextConverter::getFormat` `0x10001EA94`; UTF-16 converter explicitly calls UTF8ToUTF16 / UTF16ToUTF8 at `0x100020608` / `0x100020614` |
| `1` | HTML | UTF-8 HTML bytes, without a Windows CF_HTML offset-header wrapper in the Mac wire converter | HTML getter `0x10001FFB0`; `doToIClipboard` `0x1000201D4` validates UTF-8 or converts system encoding to UTF-8; `doFromIClipboard` copies bytes |
| `2` | Bitmap | Windows DIB-style data: bitmap information header, optional masks/palette, pixel bytes; **no 14-byte BMP file header** | BMP getter `0x10001F2F8`; `fromIClipboard` `0x10001F30C` adds `BM`/14-byte file header for macOS; `toIClipboard` `0x10001F4F4` removes it and handles pixel offset |

The DIB's own header fields are **little-endian**, unlike the outer clipboard headers. Do not globally byte-swap bitmap contents. The converter handles header-size-dependent palette/mask offsets, including BITMAPINFOHEADER-size 40 and bitfield cases. This analysis does not certify every compression/bit-depth variant. [V/O: `dibPixelOffset` `0x10001F278`, BMP conversion methods]

Mac text conversion changes CR line endings to LF when exporting and applies the reverse platform conversion when importing. UTF-8 is still the wire encoding. Embedded zero bytes are allowed by the container framing, but text converters that call C-string APIs can treat them as terminators. [V: `convertLinefeedToUnix` `0x10001F0B8`, `convertLinefeedToMacOS` `0x10001ECD0`, text conversion methods]

### Clipboard limits

`clipboardSharing` is a Boolean. `clipboardSharingSize` is an integer in **KiB (1024 bytes)**, despite log messages saying “KB”. The client's sender compares the **whole marshalled container** with `limit << 10` and refuses when size is **greater than or equal to** that byte limit. The server forwarding path instead refuses only when size is strictly greater than the limit; the equality boundary is asymmetric. [V: `Server::onClipboardChanged` `0x100071B40`] A zero limit disables sharing. Client and server constructors initialize the numeric limit to `0x7FFFFFFF`; this is not a safe total-allocation bound. [V: `Client::sendClipboard` `0x100013EC0`, `Client::setOptions` `0x1000141D8`, client data constant `0x100356010`, `Server` constructor `0x10006D704`]

## 6. DSOP option IDs

DSOP carries alternating 32-bit integers: `ID, value, ID, value, …`. The element count is twice the pair count. An ID shown as `HART` means the numeric value `0x48415254`, serialized as bytes `48 41 52 54`. Values use the 32-bit wire pattern; interpretation is option-specific.

Do not serialize a native four-character C/C++ literal or host-endian integer directly. Reject odd integer counts in an original parser; several inspected loops assume a following value. Unknown IDs can be ignored as complete pairs. [V/I: E02/E09]

| Configuration name | ID / hexadecimal | Value and scope | Evidence |
|---|---|---|---|
| `heartbeat` | `HART` / `0x48415254` | Milliseconds; positive interval, 0 disables timers; global and sent to clients | E09, writer call at `0x100054B84`; E10 |
| `switchCorners` | `SSCM` / `0x5353434D` | Dead-corner bit mask; global or per screen | `0x100054D74`, E09/E16 |
| `switchCornerSize` | `SSCS` / `0x53534353` | Dead-corner side length in pixels; global or per screen | `0x100054F60`, screen parser `0x100055774` |
| `switchDelay` | `SSWT` / `0x53535754` | Milliseconds; server clamps nonpositive delay to disabled | `0x100054D20`, `Server::processOptions` `0x10006F4E8` |
| `switchDoubleTap` | `SSTT` / `0x53535454` | Milliseconds for second edge hit; nonpositive disables | `0x100054C5C`, `processOptions` |
| `switchNeedsShift` | `SSNS` / `0x53534E53` | Boolean; server switching gate | `0x100054CCC`, `processOptions` |
| `switchNeedsControl` | `SSNC` / `0x53534E43` | Boolean; server switching gate | `0x100054F90`, `processOptions` |
| `switchNeedsAlt` | `SSNA` / `0x53534E41` | Boolean; server switching gate | `0x100054BD8`, `processOptions` |
| `relativeMouseMoves` | `MDLT` / `0x4D444C54` | Boolean; server chooses relative forwarding while locked to a remote screen | `0x100054FC8`, `processOptions`, §9 |
| `win32KeepForeground` | `_KFW` / `0x5F4B4657` | Boolean; Windows platform option; Mac platform setOptions is a no-op | `0x100054E00`; `OSXScreen::setOptions` `0x10002993C` |
| `defaultLockToScreenState` | `LTSS` / `0x4C545353` | Boolean; initial server cursor lock | `0x100054E4C`, `processOptions` |
| `disableLockToScreen` | `DLTS` / `0x444C5453` | Boolean; suppress lock-to-screen behavior | `0x100054E98`, `processOptions` |
| `clipboardSharing` | `CLPS` / `0x434C5053` | Boolean; global clipboard sharing | `0x100054EE4`, `Client::setOptions` |
| `clipboardSharingSize` | `CLSZ` / `0x434C535A` | KiB; 0 disables, positive integer sets send cap | `0x100054F30`, §5 |
| `halfDuplexCapsLock` | `HDCL` / `0x4844434C` | Boolean; per screen; handles toggle keys that do not produce ordinary paired events | `readSectionScreens`, `Screen::setOptions` `0x1000A0D40` |
| `halfDuplexNumLock` | `HDNL` / `0x48444E4C` | Boolean; per screen | Same |
| `halfDuplexScrollLock` | `HDSL` / `0x4844534C` | Boolean; per screen | Same |
| `shift` | `MMFS` / `0x4D4D4653` | Per-screen modifier remapping target enum | `0x100055774`, `ServerProxy::setOptions` |
| `ctrl` | `MMFC` / `0x4D4D4643` | Same | Same |
| `alt` | `MMFA` / `0x4D4D4641` | Same | Same |
| `altgr` | `MMFG` / `0x4D4D4647` | Same | Same |
| `meta` | `MMFM` / `0x4D4D464D` | Same | Same |
| `super` | `MMFR` / `0x4D4D4652` | Same | Same |
| `xtestIsXineramaUnaware` | `XTXU` / `0x58545855` | Boolean; X11-specific per-screen setting; no Mac implementation | `0x100055774` |
| `preserveFocus` | `SFOC` / `0x53464F43` | Boolean; per screen; accepted by parser, no effect in this Mac platform setOptions method | `0x100055774`, `0x10002993C` |

Modifier target enum: **0=None, 1=Shift, 2=Control, 3=Alt, 4=Meta, 5=Super, 6=AltGr**. Reset restores the identity map. The server proxy maps both modifier key IDs and mask bits through it. [V: `ConfigReadContext::parseModifierKey` `0x100059870`, `ServerProxy::translateKey` `0x10001C780`, `translateModifierMask` `0x10001C84C`, reset data `0x100356860`]

Corner mask: **top-left=1, top-right=2, bottom-left=4, bottom-right=8**. Masks combine with bitwise OR. `none` is zero. The text parser also supports grouped `top`, `bottom`, `left`, `right`, `all`, and signed `+` / `-` corner terms. [V: `parseCorner` `0x10005C9B4`, `parseCorners` `0x100057E38`, `Server::getCorner` `0x1000732FC`]

**No `screenSaverSync` configuration option or corresponding DSOP ID was found in this build's option parser.** CSEC itself is present, and `Server::onScreensaver` broadcasts the server's screensaver state. Do not invent an option ID from an older protocol description. `protocol` and `address` are server configuration controls, not DSOP pairs. [V: E09/E15, `Server::onScreensaver` `0x1000758C0`]

## 7. Keys, modifiers, buttons, and wheel units

### KeyID and source key button

The protocol uses a keysym-like logical key ID, distinct from the sender's physical key button. Printable BMP characters use their character value; named control keys use the private range below. **These are not raw X11 `0xFFxx` values.** For example, Return is `0xEF0D`, not `0xFF0D`. The wire key field is only 16 bits even though internal KeyID parameters are 32-bit. Supplementary Unicode handling cannot be inferred as a full 32-bit wire encoding. [V: E03/E11]

The source `key_button` is a stable identifier used to match down/repeat/up. It is not a macOS virtual keycode. A Mac sender encodes its native virtual keycode **plus 1**, reserving zero; reverse mapping subtracts 1. A receiver maps the logical KeyID, modifier state and layout to its own physical keys and remembers the source-button association. [V: `OSXKeyState::mapVirtualKeyToKeyButton` `0x1000228DC`, `mapKeyButtonToVirtualKey` `0x10002488C`, `KeyState` mapping methods]

### Modifier mask

| Mask | Meaning | Verification |
|---|---|---|
| `0x0001` | Shift | `_kModifierNameMap` entry `0x10048F858` |
| `0x0002` | Control | Entry `0x10048F838` |
| `0x0004` | Alt | Entry `0x10048F818` |
| `0x0008` | Meta | Entry `0x10048F848` |
| `0x0010` | Super; macOS Command maps here | Entry `0x10048F868`; `OSXKeyState::mapModifiersFromOSX` `0x100022178`, `pollActiveModifiers` `0x100022994` |
| `0x0020` | AltGr | Entry `0x10048F828` |
| `0x1000` | Caps Lock | `Screen::setOptions`, `KeyMap::initModifierKey`, Mac modifier polling |
| `0x2000` | Num Lock | Same |
| `0x4000` | Scroll Lock | Same |

These are logical protocol bits, not Quartz CGEventFlags. The Mac converter maps Shift/Control/Option/Command and lock states to/from the protocol. Do not use these numbers directly as CGEvent flags. The static control table maps both Meta and Super to Command virtual keycode `0x37`; default Mac capture uses Super for Command. [V: E11; Appendix B]

### Named keys

Appendix A gives the complete exported key-name table, with each entry address. Important families are:

| Family | Constants | Evidence |
|---|---|---|
| Editing | BackSpace `EF08`, Tab `EF09`, Linefeed `EF0A`, Clear `EF0B`, Return `EF0D`, Escape `EF1B`, Delete `EFFF` | E11, Appendix A |
| Navigation | Home `EF50`, Left `EF51`, Up `EF52`, Right `EF53`, Down `EF54`, PageUp `EF55`, PageDown `EF56`, End `EF57`, Begin `EF58` | Same |
| Function keys | F1–F35 = `EFBE` through `EFE0` in order | Same |
| Keypad digits | KP_0–KP_9 = `EFB0` through `EFB9` | Same |
| Modifiers | Shift_L/R `EFE1/2`; Control_L/R `EFE3/4`; CapsLock `EFE5`; Meta_L/R `EFE7/8`; Alt_L/R `EFE9/A`; Super_L/R `EFEB/C`; AltGr `EF7E` | Same |
| Media/browser | Audio keys `E0AD–E0B3`; browser keys `E0A6–E0AC`; Eject `E001`; Sleep `E05F` | Same |

### macOS translation algorithm

The binary has a **60-entry static special-key table** at `0x100356D0C`, eight bytes per entry: logical KeyID `u32le`, native virtual keycode `u32le`. Appendix B records all entries. Both right and left variants of some modifiers map to the same native virtual keycode in that table; do not “correct” the table by assuming every right modifier has a different entry. [V: E11]

Printable keys are not supplied by a fixed US ASCII table. `OSXKeyState::getGroups` (`0x100023184`) enumerates Text Input Services input sources. `getKeyMap` (`0x100022E44`, `0x1000233AC`) reads `kTISPropertyUnicodeKeyLayoutData`, uses the keyboard type, and builds entries for layouts, modifier combinations and dead keys through the Unicode keyboard-layout resource. Special keys are added for each layout group. Captured events supply Quartz keycode field 9; layout translation produces logical KeyIDs. Injection chooses suitable native keystrokes and modifiers, with special handling for modifier/toggle keys. [V: these functions; `OSXUchrKeyResource` symbols; `mapKeyFromEvent` `0x10002222C`]

LSYN's inner representation is **two-byte language units concatenated without separators**. For example, `enfr` represents two units. `getSerializedLocalLayouts` concatenates local layout strings; `setRemoteLayouts` steps through the received bytes two at a time. This is not JSON, a comma list, or a list of full macOS input-source IDs. For valid interoperable output, use an even byte length. DKDL/DKRP carry one current language string; the client uses it to select a compatible installed group when languageSync is enabled. [V: `0x1000AC988`, `0x1000AC4E0`, `ServerProxy::setActiveServerLanguage` `0x10001C8D8`]

### Buttons and scrolling

| Wire button | Meaning | Mac CGMouseButton | Evidence |
|---:|---|---:|---|
| 1 | Left / primary | 0 | Mapping array `0x100357240`; `mapDeskflowButtonToMac` `0x100028984` |
| 2 | Middle | 2 | Same |
| 3 | Right / secondary | 1 | Same |
| 4 | Additional button | 3 | Same |
| 5 | Additional button | 4 | Same |

The Mac button injection function ignores IDs outside 1–5. Button 0 is not a portable left button. Scroll is sent with DMWM, not mouse-button IDs. [V: bounds check `0x1000286E4–0x1000286EC`]

Modern DMWM is signed **horizontal delta first, vertical delta second**. `120` units is one conventional notch. The Mac injection converts vertical delta to line scrolling using approximately `int(delta * 3 / 120)`, then applies configured axis inversion and scale. The Quartz event uses line units (`1`) and two axes. Small sub-notch values can lose precision in this path. The old one-field writer uses notch conversion and is not the modern packet layout. [V: `OSXScreen::fakeMouseWheel` `0x100028BCC`, `ClientProxy1_0::mouseWheel` `0x100039A88`, modern writer `0x10003BB30`]

## 8. TLS, certificates, and trust files

### TLS ordering and constraints

| Property | Inspected behavior | Verification |
|---|---|---|
| Start | Immediately after TCP connection; before the framed Synergy hello | E12; Client/ClientListener socket construction |
| Methods | OpenSSL `TLS_client_method` / `TLS_server_method` | `initContext` `0x1000BAF50` |
| Context options | `0x16000080`: disables SSLv3, TLS 1.0 and TLS 1.1; includes ignore-unexpected-EOF flag | `_SSL_CTX_set_options` argument; local OpenSSL `ssl.h` flag definitions |
| Effective modern versions | TLS 1.2 and TLS 1.3 available; no application-level downgrade handshake | Above mask/methods; actual negotiated version was not captured |
| Cipher configuration | No application-specific cipher-list restriction in this context initializer; OpenSSL defaults apply | E12 and call-site inspection. A fixed negotiated cipher must not be assumed. |
| Certificate chain verification | Authenticated mode sets verify flags `3` = peer verification plus require peer certificate, but installs a callback that returns success rather than normal PKIX verification | `initContext`; `verifyIgnoreCertCallback` `0x1000BB36C` |
| Application authentication | SHA-256 certificate fingerprint equality against trusted peer records | `verifyCertFingerprint` `0x1000BB5F0`, E13 |
| Host name | Client calls SSL_set1_host with its configured computer name; this is not proof of working CA/hostname validation because the custom verification callback replaces the chain verification | `secureConnect` `0x1000BBAC4` |

### Client/server asymmetry and `checkPeerFingerprints=false`

There are numeric security levels 0, 1, 2 at the socket factory boundary: plaintext, encrypted, and encrypted with authenticated peer respectively. [V: `TCPSocketFactory::create` `0x1000C8E2C`, `createListen` `0x1000C8EBC`, `ServerApp::openClientListener` `0x1000A676C`]

- **Server:** TLS enabled plus checkPeerFingerprints=true selects level 2. It requests/requires the client certificate and checks its SHA-256 pin in `trusted-clients`. With the check false it selects level 1 and skips client pin validation. [V: `0x1000A676C`, `secureAccept` `0x1000BB3B8`]
- **Client:** Its constructor reads tlsEnabled; the connection creates level **2 whenever TLS is enabled**. `secureConnect` always loads the client certificate/key and, after SSL_connect, checks the server certificate in `trusted-servers`. This path does **not** consult checkPeerFingerprints. Thus false is not a reliable “disable server pinning” switch for this binary's client. [V: `Client::connect` `0x100012F08`, `secureConnect`]
- Pin mismatch or an unusable trust database causes failure/disconnection. No in-core “trust any certificate and continue” branch was found for that client flow. GUI approval can update the database separately, but the GUI's approval transaction was outside this binary-only scope. [V/O: E12/E13]

### Certificate generation

The helper `deskflow::generatePemSelfSignedCert` at `0x1000C291C` establishes:

| Constant | Value / behavior | Verification |
|---|---|---|
| Key algorithm | RSA | `EVP_PKEY_Q_keygen` with literal `RSA` |
| Default key size | 2048 bits; size is a helper parameter and settings default | `Settings::defaultValue` returns `0x800` for security/keySize |
| Serial | 1 | ASN1_INTEGER_set argument |
| Subject and issuer | Self-signed; only CN=`Synergy` is added by the helper | X509_NAME_add_entry_by_txt call; issuer copied from subject |
| Valid from | Generation time | X509_gmtime_adj on notBefore with 0 |
| Valid for | 31,536,000 seconds = 365 days | notAfter argument `0x01E13380` |
| Signature digest | SHA-256 | EVP_sha256 then X509_sign |
| Container | PEM private key and PEM certificate written to the same configured path; private key write uses no encryption cipher | PEM_write_PrivateKey / PEM_write_X509 calls |
| Extensions/version | No X509_set_version or extension creation in this helper; do not assume a generated v3 CA certificate | Helper call inventory |

The helper is present, but this analysis did not establish every UI/service trigger that invokes generation or renewal. It did not generate or read a real user's private key.

### Fingerprints and database format

`deskflow::sslCertFingerprint` (`0x1000C2654`) calls **X509_digest** on the certificate. That hashes the certificate's DER representation, not PEM text, a public-key-only blob, or the private key. It supports Qt hash enum **2=SHA-1** and **4=SHA-256**. Current peer verification explicitly requests enum 4. [V: E12/E13]

| Record form | Syntax / acceptance | Evidence |
|---|---|---|
| Current SHA-256 | `v2:sha256:<64 lowercase hex characters>` | `toDbLine` uses `v2:%1:%2`, lower-case hex; `fromDbLine` splits three fields |
| Current SHA-1 | `v2:sha1:<40 lowercase hex characters>` | Same; digest-size checks 20 vs 32 |
| Legacy SHA-1 | Twenty two-digit hex bytes separated by colons; exactly 59 characters and 19 colons | `fromDbLine` legacy branch |
| Lines | Text file, one record per line; empty lines skipped; invalid records not added | `FingerprintDatabase::readStream` `0x1000B3178` |
| Match | Algorithm and digest bytes must both match | `isTrusted` `0x1000B3788` |

A legacy SHA-1 record alone does not match a current SHA-256 verification request. Parsing support for SHA-1 is not evidence that current peer verification hashes with SHA-1. Trust paths are generated as `<tlsDir>/trusted-servers` and `<tlsDir>/trusted-clients` (`Settings::tlsTrustedServersDb` `0x10033E7AC`, `tlsTrustedClientsDb` `0x10033E9B0`). No actual pin is reproduced here.

## 9. Server switching, locks, and disconnect recovery

### Layout and edge crossing

The server owns the screen graph and the pointer's current logical screen. Clients do not exchange layout topology or decide which neighbor to enter. For the user's arrangement, the Windows screen's **left** link targets the Mac, and the Mac's **right** link targets Windows. The client still reports its own real geometry through DINF. [V: E15/E16]

At an edge, the server selects a connected neighbor for the source-edge interval, maps the pointer's position along that interval into the destination interval, places it just inside the target screen, sends COUT to the old client and CINN to the new one. CINN's x/y are destination coordinates, not deltas or server coordinates. A missing neighbor leaves the pointer clamped on the current screen. The mapping accounts for screen origins and differing dimensions. [V: `getNeighbor` `0x100071FD4`, `mapToNeighbor` `0x100072380`, `mapToFraction` `0x100071E4C`, `mapToPixel` `0x100071F0C`, `switchScreen`]

Link intervals use a normalized fraction. Selection is **start-inclusive, end-exclusive**. Mapping first normalizes the source interval, then scales into the destination interval. Configuration rejects overlapping source ranges. [V: `Config::Cell::getLink` `0x100052E10`, `CellEdge::transform` `0x100052EE4`, `inverseTransform` `0x100052ED4`, `readSectionLinks`]

| Gate | Observed rule | Evidence |
|---|---|---|
| Delay | First edge attempt starts a switch-wait timer; crossing waits until it expires. Changing target/direction or cancelling an edge attempt resets the pending switch. | `isSwitchOkay` `0x100072B74`, `startSwitchWait` `0x10007327C`, `handleSwitchWaitTimeout` `0x100074E28` |
| Double tap | First hit starts a time window; pointer must move back into the screen to arm the second hit, then hit the same edge within the window. It is an edge gesture, not a mouse-button double click. | `startSwitchTwoTap` `0x100073234`, `armSwitchTwoTap` `0x100073488`, `isSwitchOkay` |
| Retreat distance | Uses at least the jump zone and the server's stored movement threshold; constructor initializes that threshold to 3 pixels. | `armSwitchTwoTap`; `Server` constructor `0x10006D704` |
| Dead corners | Corners selected by SSCM block crossing within SSCS pixels. Per-screen corner settings are considered before global fallbacks. | `isSwitchOkay`, `getCorner` `0x1000732FC` |
| Required modifiers | Shift/Control/Alt gates require their configured masks before switching | `isSwitchOkay`; `processOptions` `0x10006F4E8` |
| Mouse button held | Primary-screen lock logic prevents ordinary edge switching while a mouse button is held; this is why unsupported cross-machine file dragging must not be inferred from edge switching alone. | `PrimaryClient::isLockedToScreen`, `Screen::isLockedToScreen` `0x1000A0EFC`, E16 |
| Explicit cursor lock | Scroll Lock/default lock state or a lockCursorToScreen action can lock the active screen. `disableLockToScreen` suppresses the lock check. | `Server::isLockedToScreen` `0x100071398`, constructor, `handleLockCursorToScreenEvent` `0x1000757E0` |
| Relative mode | DMRM forwarding is selected when relativeMouseMoves is enabled **and the server cursor-lock flag is on** while a remote screen is active. It is not an unconditional replacement for all DMMV traffic. | `onMouseMoveSecondary` `0x100073BC4` |

Server keyboard events keep logical key ID, modifier mask, repeat count, source button and language. A switch carries the current modifier/toggle mask in CINN. Ordinary character keys are not transferred as a raw Windows keycode to macOS. [V: `Server::onKeyDown` `0x1000746C4`, repeat `0x100074B48`, E06/E11]

### Leave and failure cleanup

For a secondary/client screen, `Screen::leave` calls the platform's **fakeAllKeysUp** interface before completing the leave. The secondary-interface vtable resolves this to `PlatformScreen::fakeAllKeysUp`, then `KeyState::fakeAllKeysUp` (`0x100099178`, `0x10009773C`). The latter walks tracked pressed buttons and synthesizes releases, rather than merely clearing a logical Boolean. Disconnect disable also calls fakeAllKeysUp. [V: `Screen::leave` `0x1000A04C4`, `disable` `0x1000A0294`, OSXScreen vtable at `0x10048A968` onward]

If an active remote client disconnects, the server's `removeActiveClient` / `forceLeaveClient` returns control to the primary screen, uses its cursor center, and clears pending switching state. It cannot deliver a final DKUP/DMUP over a dead TCP connection. Each client must therefore clean up its own injected state on leave/disconnect. [V: `0x100074FB8`, `0x100075A98`]

**Open limitation:** fakeAllKeysUp proves keyboard cleanup. It does not prove that all synthetic mouse-button states are released during a network loss in the middle of a drag. The Mac button state lives separately. The exact mid-drag button-release guarantee needs an isolated end-to-end check; this report does not assume it. No such check was run on the user's live input session.

CSEC is broadcast by the server, but the common `deskflow::Screen::screensaver(bool)` at **`0x1000A0C0C` is a no-op in this build**. Its caller chain is ServerProxy → Client → Screen. A lower-level OSXScreen screensaver method exists but is not called by that common wrapper. Thus decoding CSEC does not prove this Mac client starts/stops its screensaver. [V: instructions at `0x1000A0C0C`, `Client::screensaver` `0x1000141C8`, `ServerProxy::screensaver` `0x10001C1E0`]

## 10. Configuration and CLI

### Core settings file

The service/core uses Qt QSettings INI keys. In the user's setup the supplied file path is `~/Library/Preferences/Synergy/synergy-core.conf`. This report did not read the live values. A slash in a catalog key maps to an INI group, e.g. `security/tlsEnabled` becomes `[security]` plus `tlsEnabled=…`. Values use Qt conversion and enum handling, not the standalone server-file parser's Boolean grammar. [V: `QSettingsProxy::load` `0x100342DDC`, E14]

The following is the complete recovered static key catalog. “Empty/unset” means no specific default is established here; it does not prescribe a particular Qt on-disk representation. Where an enum is noted, its exact QVariant INI serialization must not be replaced with an assumed CLI string.

| INI group / key | Type / meaning; recovered default where verified | UTF-16 string address |
|---|---|---|
| client/dynamicConnectionInterval | Boolean despite its name; enables dynamic connection retry interval; false | `0x100438636` |
| client/invertYScroll | Boolean; reverse received vertical scroll; false | `0x100438678` |
| client/invertXScroll | Boolean; reverse horizontal scroll; false | `0x1004386A2` |
| client/yScrollScale | Numeric vertical multiplier; 1.0 | `0x1004386CC` |
| client/xScrollScale | Numeric horizontal multiplier; 1.0 | `0x1004386F4` |
| client/languageSync | Boolean; select installed layout from server language; true | `0x10043871C` |
| client/remoteHost | Server host/address setting; client resolves configured hosts | `0x100438744` |
| client/xdpRestoreToken | XDG desktop-portal session restoration data; not a Mac wire field | `0x100438768` |
| core/coreMode | Qt CoreMode enum: None, Client, Server; CLI positional mode selects runtime role | `0x100438796` |
| core/interface | Local network interface/address to bind | `0x1004387B2` |
| core/lastVersion | Stored core product version for settings upgrade | `0x1004387D0` |
| core/port | Numeric Synergy port; 24800 (`0x60E0`) | `0x1004387F2` |
| core/preventSleep | Boolean; inhibit sleep while core operates; false | `0x100438806` |
| core/processMode | Qt ProcessMode enum, Service or Desktop; default ordinal 1, Desktop | `0x10043882A` |
| core/computerName | Local screen name used in hello-back and core setup | `0x10043884C` |
| core/display | Platform display selection, mainly non-Mac use | `0x100438870` |
| core/useHooks | Boolean; platform input-hook setting; true | `0x10043888A` |
| core/language | Core/UI locale selection; separate from wire layout synchronization | `0x1004388A6` |
| core/enableEnterCommand | Boolean; allow configured local command on screen enter; false | `0x1004388C2` |
| core/enterCommand | Local screen-entry command string | `0x1004388F2` |
| core/enableExitCommand | Boolean; allow configured local command on screen exit; false | `0x100438916` |
| core/exitCommand | Local screen-exit command string | `0x100438944` |
| core/screenName | Legacy screen-name setting; settings upgrade/catalog retains it | `0x100438966` |
| daemon/configFile | Core configuration path used by daemon/service | `0x100438986` |
| daemon/elevate | Boolean; elevated service mode; defaults true outside portable mode | `0x1004389AA` |
| daemon/logFile | Daemon log path; generated default ends in `synergy-daemon.log` | `0x1004389C8` |
| daemon/logLevel | Daemon log verbosity | `0x1004389E6` |
| gui/autoHide | Boolean; hide GUI automatically; false | `0x100438A06` |
| gui/startCoreWithGui | Boolean; start core with GUI; false | `0x100438A20` |
| gui/enableUpdateCheck | GUI update-check preference | `0x100438A4A` |
| gui/updateCheckUrl | GUI update service URL setting | `0x100438A76` |
| gui/closeReminder | Boolean; close reminder; true | `0x100438A9C` |
| gui/closeToTray | Boolean; close window to tray; true | `0x100438AC0` |
| gui/logExpanded | Boolean; expanded log view; true | `0x100438AE0` |
| gui/symbolicTrayIcon | Boolean; symbolic tray icon; true | `0x100438B00` |
| gui/windowGeometry | Qt window geometry state | `0x100438B2A` |
| gui/shownFirstConnectedMessage | Boolean; first-connection notice state; false | `0x100438B50` |
| gui/shownServerFirstStartMessage | Boolean; initial-server notice state; false | `0x100438B8E` |
| gui/showVersionInTitle | Boolean; title preference; false | `0x100438BD0` |
| gui/ignoreMissingKeyboardLayouts | Boolean; GUI warning policy; false | `0x100438BFE` |
| log/file | Core log destination; generated default `~/synergy.log` | `0x100438C40` |
| log/level | Log level; FATAL, ERROR, WARNING, INFO, DEBUG, VERBOSE are present; default follows ordinary INFO/debug preference logic | `0x100438C52`; level strings `0x10043859E` onward |
| log/toFile | Boolean; enable file logging; false | `0x100438C66` |
| log/guiDebug | GUI/core debug logging preference | `0x100438C7C` |
| security/checkPeerFingerprints | Boolean; true; asymmetric client/server effect explained in §8 | `0x100438C96` |
| security/certificate | Combined PEM certificate/private-key path; default `<tlsDir>/synergy.pem` | `0x100438CD4` |
| security/keySize | RSA key size in bits; 2048 | `0x100438CFE` |
| security/tlsEnabled | Boolean; true; selects TLS before hello | `0x100438D20` |
| server/externalConfig | Boolean; use external standalone server config; false | `0x100438D48` |
| server/externalConfigFile | Standalone server config path; generated default ends in `synergy-server.conf` | `0x100438D74` |
| server/protocol | Qt NetworkProtocol setting; Synergy/Barrier selection. Do not assume its INI enum representation equals the standalone `protocol = synergy` spelling. | `0x100438DA8` |
| server/xdpRestoreToken | XDG portal restoration data; not a Mac wire field | `0x100438DC8` |

The descriptions of GUI and portal-only keys follow their Settings symbol names and are not a runtime audit of the GUI. Defaults come from `Settings::defaultValue` (`0x10033D08C`) and the static default-Boolean list initialization, visible at `0x10000CC54` and `0x10000D27C` in this image. GUI keys belong to the shared Settings catalog; their presence does not mean the core implements GUI behavior.

`Settings::serverConfigFile` selects server/externalConfigFile only when server/externalConfig is true; otherwise it uses the generated default server-file path. The user's supplied `synergy.conf` path is therefore a deployment/configuration choice, not a hardcoded universal basename. `[internalConfig]` / `internalConfig/protocol` also occur in settings upgrade/generated-state handling, outside the fixed user-key catalog. [V: `0x10033E214`, `upgradeSettings` `0x10033BA78`, UTF-16 strings `0x100439176`, `0x1004391A6`]

### Standalone `synergy.conf` grammar

This is a different language from the core INI file. The supplied user path is `~/Library/Preferences/Synergy/synergy.conf`; exact selection depends on the setting above. It has line-oriented `section: NAME` … `end` blocks. Recognized section names are **screens, aliases, links, options**. Data outside a section, duplicate screens/aliases, unknown sections/options, unknown link targets, and overlapping source-edge intervals produce a line-numbered configuration error. [V: E15]

Leading/trailing space and tabs are trimmed. `#` starts a comment through end of line. The line reader removes comments before later parsing, so quoting is not a general way to preserve `#` in a value. Integer values are decimal signed 32-bit values with no trailing junk. Boolean values are case-insensitive `true` or `false`; this parser does not accept arbitrary INI-style Boolean spellings. Screen names must be nonempty at Config validation; GUI computer-name cleanup is a separate policy. [V: `readLine` `0x10005429C`, `parseInt` `0x100057D6C`, `parseBoolean` `0x100058480`, `isValidScreenName` `0x100052A00`, `Settings::cleanComputerName` `0x10033D7BC`]

```text
section: screens
    WindowsPC:
    MacClient:
        halfDuplexCapsLock = false
end
section: aliases
    MacClient:
        MacAlias
end
section: links
    WindowsPC:
        left = MacClient
    MacClient:
        right = WindowsPC
end
section: options
    protocol = synergy
    heartbeat = 3000
    clipboardSharing = true
    clipboardSharingSize = 1024
    switchDelay = 0
    switchDoubleTap = 0
end
```

The names above are synthetic examples. This file was not installed or executed.

| Construct | Syntax / values | Verification |
|---|---|---|
| Screen | `name:` followed by its per-screen options | `readSectionScreens` `0x100055774`; all option names/IDs are in §6 |
| Alias | Canonical `name:` followed by one alias name per line | `readSectionAliases` `0x100056BD0`; duplicate aliases rejected |
| Link | `left = target`, `right = target`, `up = target`, `down = target`; links are directed, so add the reverse link separately | `readSectionLinks` `0x100056350` |
| Partial link | `left(0,50) = target(50,100)` | Same; interval helper `0x100059B64` |
| Interval | Two decimal numeric endpoints, `0 <= start < end <= 100`; omitted interval means the full edge. No percent sign. | `parseInterval`; values divided by 100 |
| Global options | Names and values in §6, plus `protocol` and `address` | E09/E15 |
| Protocol | `synergy` or `barrier` | `readSectionOptions`; invalid protocol exception string at `0x10043D4F5` |
| Address | Listen address with optional port; default port 24800 | Address parser call at `0x100054AD8`, NetworkAddress constructor |
| Corners | `none`, `all`, `top-left`, `top-right`, `bottom-left`, `bottom-right`, grouped top/bottom/left/right; combine with `+`/`-`, e.g. `all -bottom-right` | `parseCorners` / `parseCorner` |
| Modifier remap | `shift`, `ctrl`, `alt`, `altgr`, `meta`, `super` map to a target modifier name or `none` | `parseModifierKey` `0x100059870` |

Source link blocks require a canonical screen name; the parser explicitly rejects using an alias there. Alias lookup is used when resolving a connecting client's name and other canonical-name lookups. [V: `readSectionLinks`, `getCanonicalName` `0x100050EC8`]

### Hotkey conditions and actions

Rules belong in `section: options`. A rule uses `condition = action[, action …][; release-action …]`. The semicolon separates trigger-down actions from trigger-release actions. A plain keystroke/mousebutton action can install paired down/up behavior, while keyDown/keyUp and mouseDown/mouseUp select one edge explicitly. [V: `Config::readSectionOptions`, `parseCondition` `0x1000585B4`, `parseAction` `0x100058A60`]

| Condition / action | Accepted shape / meaning | Evidence |
|---|---|---|
| Condition `keystroke` | `keystroke(modifier+key)` | `parseCondition`; key-name table Appendix A |
| Condition `mousebutton` | `mousebutton(modifier+button)` | Same; decimal button ID |
| Condition `connect` | `connect()` or `connect(screen)` | Same; optional named screen |
| Action `keystroke`, `keyDown`, `keyUp` | `(modifier+key[,screens])` | `parseAction` |
| Action `mousebutton`, `mouseDown`, `mouseUp` | `(modifier+button)` | Same |
| `switchToScreen` | `(name)` | Same |
| `switchInDirection` | `(left)`, `(right)`, `(up)`, `(down)` | Same |
| `switchToNextScreen` | No arguments | Same |
| `lockCursorToScreen` | `()`, `(on)`, `(off)`, `(toggle)` | Same |
| `keyboardBroadcast` | `([on/off/toggle][,screens])` | Same |
| `restartServer` | `()` or `(restart)`; legacy parser/action retained | Same; diagnostic strings `0x10043BF48` onward |

Modifiers in keystroke syntax are joined with `+`, e.g. `Control+Shift+F12`. Destination screen lists use **colon separators**, e.g. `MacClient:WindowsPC`; `*` selects all. The comma separates the key argument from the screen-list argument. [V: `_kModifierNameMap`, `Config::parseScreens` `0x10005A180`, `parseKeystroke` `0x10005A458`]

Examples of syntax, not installed configuration:

```text
keystroke(ScrollLock) = lockCursorToScreen(toggle)
keystroke(Control+Alt+Left) = switchInDirection(left)
keystroke(Control+F12) = keystroke(Control+c,MacClient:WindowsPC)
```

### Actual CLI options

The successful `--help` command on the copy printed only these options:

| Option | Meaning |
|---|---|
| `-h`, `--help` | Display command-line help |
| `-v`, `--version` | Display version information |
| `--new-instance` | Skip the running-instance check |
| `-s`, `--settings <configFile>` | Override the core settings file |
| Positional `coremode` | `server` or `client` |

Usage is `synergy-core coremode [options]`. Old Synergy/Barrier CLI options such as `--name`, `--address`, `--enable-crypto`, or a bare trailing server address are **not listed by this executable's help**. The supplied live invocation uses the modern `client --settings …` form. `--new-instance` was not used during research. [V: E19]

## 11. macOS platform layer and the macOS 27 activation issue

### Capture and event handling

| Operation / constant | Inspected behavior | Evidence |
|---|---|---|
| Event tap location | `0` = kCGHIDEventTap | `OSXScreen::enable` `0x100028EE0`, call arguments at `0x100029030–0x100029044`; local CGEventTypes.h |
| Placement | `0` = kCGHeadInsertEventTap | Same |
| Options | `0` = kCGEventTapOptionDefault, not listen-only | Same |
| Event mask | All 64 bits set | Same |
| Run loop | CFMachPort source on a dedicated thread's CFRunLoop | `enable`, thread/run-loop code |
| Primary callback | Handles keyboard, mouse, scroll, hotkeys and disabled-tap notices; can suppress local events while control is remote | `handleCGInputEvent` `0x1000291EC` |
| Secondary callback | Returns the incoming event unchanged | `handleCGInputEventSecondary` `0x1000294A4`, single return path |
| Disabled tap | Timeout path attempts re-enable; loss of accessibility trust is treated as fatal | `handleCGInputEvent`, `checkAXPermissions` `0x10002ACF0` |
| Jump zone | 1 pixel from the Mac platform getter | `getJumpZoneSize` `0x100027938` |

Capture is global Quartz capture, not a Qt widget event loop. The program also uses Carbon hotkey facilities and checks Accessibility trust. A Rust implementation needs equivalent system permissions and an input-event loop, but it need not copy this C++ structure. [V: E17, Carbon/AX imports]

### Injection

| Input | How this binary synthesizes it | Evidence |
|---|---|---|
| Mouse down/up | Creates a Quartz mouse event with **NULL source**, selected left/right/other type, cached/fresh global pointer coordinates, mapped button; sets integer field **1** (kCGMouseEventClickState), sets modifiers, updates local button mask, then `CGEventPost(0,event)` | `fakeMouseButton` `0x1000286C0`; instruction range `0x1000288C4–0x100028924` |
| Mouse movement / drag | Creates moved/dragged events based on held button state, sets click state and delta fields **4/5**, sets flags, posts at HID tap | `postMouseEvent` `0x100028368` |
| Relative move | Reads current position with `CGEventCreate(NULL)` / CGEventGetLocation, adds deltas, then uses movement path | `fakeMouseRelativeMove` `0x100028AA8`; disassembly supplies the delta additions lost in high-level types |
| Wheel | Creates line-based two-axis scroll event with NULL source, sets modifiers, posts at HID tap | `0x100028BCC`, §7 |
| Keyboard preferred path | Opens `IOHIDSystem`; uses **IOHIDPostEvent** for native virtual keys. Types 10/11 are key-down/up; type 12 handles modifier flag changes. Event-data version argument is 2. | `postHIDVirtualKey` `0x10002421C`; call at `0x10002431C`; local IOKit event headers |
| Keyboard fallback | If low-level post fails, creates CGEventCreateKeyboardEvent with NULL source, sets logical/device modifier flags, posts at HID tap | `fakeKey` `0x1000245F8`, `postKeyboardKey` `0x100024520` |
| Warp | CGWarpMouseCursorPosition; if needed, dispatches the warp to the capture run loop and wakes it | `warpCursor` `0x100027774` |
| Cursor visibility | CGDisplayHideCursor / ShowCursor for selected display; both associate mouse and cursor with argument 1. Uses private CGS connection property `SetsCursorInBackground`. | `hideCursor` `0x100028DE0`, `showCursor` `0x100028CB0` |
| Enter/leave | Shows/hides cursor; primary enter resets local suppression interval to zero; secondary enter resets button mask and adjusts local-event suppression filters | `enter` `0x100029584`, `leave` `0x100029668` |

The mouse button path has **no explicit CGEventSourceCreate**, **no explicit window-under-pointer fields 91/92**, and **no app/window activation call**. It sets click count and modifiers, but does not establish an explicit target window. It already posts globally at the HID tap; it is not posting directly to one application's PID. [V: full `fakeMouseButton` instruction/call inventory; imported symbols]

The local Apple SDK headers confirm the numeric mapping: HID tap=0, combined-session source state=0, HID-system source state=1, mouse click-state field=1, window-under-pointer fields=91/92. Those source-state values are a different enum from tap location despite the overlapping numbers. The headers expressly allow NULL sources. Therefore **NULL is a verified design choice, not proof of an API violation**. [V: local Xcode SDK `CoreGraphics.framework/Headers/CGEvent.h`, `CGEventTypes.h`]

### Most likely cause and an original injection design

**Most likely area of failure [I]: macOS 27's activation/window-routing treatment of synthesized mouse events that carry no explicit source/window context.** The user's observation—controls receive the click but the inactive app remains inactive, while a title-bar click activates it—is consistent with input delivery succeeding and the window manager's activation decision differing from physical input. It does not point first to DMDN framing or button-number mismatch. This binary relies on WindowServer/AppKit to supply activation semantics, and contains no fallback for that decision.

This does **not** prove that setting one private-looking event field will fix it. The secondary tap is pass-through, so static evidence does not support blaming that callback for swallowing activation. Also, click count is maintained by the screen's click tracker; unusual double/triple-click state is a separate candidate to isolate.

The appropriate baseline for an original Openergy injector is:

1. Use a persistent, explicit CGEventSource created for **kCGEventSourceStateHIDSystemState (1)**. Keep source state separate from posting location.
2. Produce balanced down/up events at the intended **global** position. Set correct event type, mapped button, current modifiers and click count. Keep movement before mouse-down in stream order.
3. Post globally at **kCGHIDEventTap (0)** so normal window-server routing and activation can occur. A direct-to-PID post is not a demonstrated substitute for normal activation.
4. Tag/filter Openergy's own injected events in its capture path to prevent recapture when it acts as a server. This is a design recommendation, not a feature recovered from the Synergy mouse path.
5. On macOS 27, retain a compatibility policy for **activating the eligible app/window under the pointer before mouse-down** if global synthetic clicks still fail to activate it. The user's workaround supplies evidence for this fallback. Respect nonactivating panels, modal windows, and clicks intended for a background app.

Steps 1–3 are a controlled baseline, **not a proven fix for this macOS 27 regression**. Explicitly setting fields 91/92 to the correct hit-tested window is an experiment worth comparing, but this report cannot prescribe it as universally required or sufficient. App activation/AX window raising must also be separated from mere event injection; they can change first-click behavior and must follow an explicit product policy.

A safe later verification matrix is: NULL versus explicit HID source; click-state forced to 1 versus tracked count; window fields absent versus correctly populated; app-activation fallback off versus on. Check ordinary controls and title bars in several inactive apps, plus modifiers, double-click, drag, modal dialogs and nonactivating panels. Run that matrix in an isolated app/session with a local input recovery path. It was **not run on the user's desktop**.

### Secure input

`OSXScreen::getSecureInputApp` (`0x10002AFBC`) first calls **IsSecureEventInputEnabled**. When enabled, `getSecureInputEventPID` (`0x10002B0CC`) reads IORegistry session information using `IOConsoleUsers` and `kCGSSessionSecureInputPID`; `proc_name` supplies the app name. Unknown owner returns the literal `unknown`; no secure input returns an empty string. The server can send that UTF-8 app name in SECN to version-1.7+ clients. The receiving handler logs that the app blocks the keyboard. This is a notification mechanism, not a bypass of secure input. [V: these functions, `ClientProxy1_7::secureInputNotification`]

### macOS clipboard APIs and types

The core uses **Carbon Pasteboard APIs**, not direct NSPasteboard method calls: PasteboardCreate, Synchronize, Clear, PutItemFlavor, CopyItemFlavorData, and flavor enumeration. It opens the system clipboard pasteboard named `com.apple.pasteboard.clipboard`. These are the UTIs an NSPasteboard-based original implementation would need to bridge:

| macOS type | Protocol format | Evidence |
|---|---|---|
| `public.utf8-plain-text` | Text 0 | Converter/type literal `0x10043A236`; `0x1000204A8` |
| `public.utf16-plain-text` | Text 0; converted to/from UTF-8 on wire | Literal `0x10043A24D`; `0x1000205FC` |
| `public.plain-text` | Text 0; system-encoding conversion | Literal `0x10043A224`; `0x1000202B4` |
| `public.html` | HTML 1 | Literal `0x10043A218`; `0x10001FFB8` |
| `com.microsoft.bmp` | Bitmap 2; BMP file header converted to/from wire DIB | CFString returned by `0x10001F300`; §5 |

Converter registration order is HTML, BMP, UTF-8, UTF-16, plain text in `OSXClipboard` constructor `0x10001D760`. The getter handles the first pasteboard item and chooses a suitable flavor/converter. PNG/TIFF and multi-item/file-URL clipboard interchange were not established as wire formats in this core. [V/O: `OSXClipboard::get` `0x10001E1E0`, constructor]

## 12. Local core/daemon IPC

This protocol is **separate from Synergy TCP framing**. It uses Qt **QLocalServer / QLocalSocket**, which are Unix-domain local sockets on this Mac; no TCP listener or TLS layer is used by these classes. Logical endpoint names are **`synergy-core`** and **`synergy-daemon`**. The exact expanded filesystem socket path is selected by Qt and was not queried from the live process. [V: constructors `0x1000B1D70`, `0x1000B09BC`, common constructor `0x1000AD3E4`]

The server sets QLocalServer socket-options bit mask **7**. That numeric argument is verified; the exact filesystem permission effect was not independently checked against this bundled Qt build. Do not infer a strong authentication boundary from the hello's version check. [V/O: `IpcServer::listen` `0x1000AD6F8`]

Messages are UTF-8 text ending with **one LF byte (`0x0A`)**. No four-byte size prefix is added. The reader splits at LF and the command parser splits the resulting string on `=`. There is no recovered escaping scheme for literal LF or `=` in values. Send each complete command as one newline-terminated write: the inspected readAll path logs an incomplete fragment and does not preserve that partial buffer between calls. [V: `handleReadyRead` `0x1000ADA94`, `processMessage` `0x1000ADF40`, writer `0x1000AE8D0`]

| IPC message | Endpoint / direction | Meaning / reply | Evidence |
|---|---|---|---|
| `hello=1.21.1-beta` | Client→either server | Exact product-version match returns `hello=1.21.1-beta`; queued notifications are replayed | `processMessage` `0x1000ADF40` |
| `hello` without value | Same | `error=missing version`, flush and disconnect | Same; literal `0x10043EB83` |
| `hello=<other version>` | Same | `versionMismatch=1.21.1-beta`, then disconnect | Same; UTF-16 template `0x100438EBC` |
| `noop` | Client→either | `ok` | Same |
| `stop` | Client→core | `ok`, broadcasts `core` with no value, requests process stop | `CoreIpcServer::processCommand` `0x1000B1E84` |
| `start` / `stop` | Client→daemon | Requests starting/stopping core, replies `ok` | `DaemonIpcServer::processCommand` `0x1000B0AEC` |
| `logLevel=<level>` | Client→daemon | Requires a value; emits logLevelChanged; `ok` or `error` | `processLogLevel` `0x1000B0F7C` |
| `configFile=<path>` | Client→daemon | Requires nonempty path; emits configFileChanged; `ok` or `error` | `processConfigFile` `0x1000B11B4` |
| `logPath` | Client→daemon | Replies `logPath=<path>` | `processCommand` |
| `clearSettings` | Client→daemon | Requests clearing settings; replies `ok` | `processCommand` |
| `connectionState=<enum-name>` | Core→UI | Uses Qt enum **names**, e.g. Disconnected, Connecting, Connected; not a numeric wire integer | `ipcSendConnectionState` `0x1000B1B68`; literals `0x100438043–0x10043805B` |
| `connectedClients=<comma-separated names>` | Server core→UI | Current client-name list | `Server::sendConnectedClientsIpc` `0x100070C14`, literal `0x10043C553` |
| `connectionRefused=<…>` | Client core→UI | Connection refusal detail/type | `Client::refuseConnection` `0x100013B38`, literal `0x1004395B7` |
| `missingKeyboardLayouts=<…>` | Client core→UI | Layouts missing after setup | `ServerProxy::parseHandshakeMessage`, literal `0x1004398E0` |
| `unrecognisedClient=<name>` | Server core→UI | Name not in config | `Server::adoptClient` |
| `peerFingerprint=<redacted>` | TLS core→UI | Peer pin for approval/status; never reproduce a real value in logs/reports | `verifyCertFingerprint`, literal `0x10043F498` |
| `secureSocket=<…>` | TLS core→UI | Encryption/cipher status | SslLogger call sites, literal `0x10043F90F` |
| `retryIn=<…>` | Client core→UI | Reconnect-delay status | ClientApp retry path, literal `0x10043D0F3` |

The common dispatcher handles hello/noop, then delegates other command names. No completed-hello gate is checked in that dispatch function before other commands are delegated. A command not understood by the endpoint is logged. Broadcasting with an empty value sends just the command name; with a nonempty value it sends `name=value`. Notifications are queued when no IPC client is connected and replayed after a successful hello. The exact role of every UI-side consumer is outside the core binary. [V: `broadcastCommand` `0x1000AEC14`, `processMessage`]

## 13. Worked framed session in hex

This is a **constructed wire example, not a captured session**. Every row includes the four-byte big-endian packet size. The synthetic client name is `mac-demo`. Its screen is 1920×1080 at (0,0), left of the server, so entering from Windows uses the Mac's right edge at (1919,540). The example uses sequence 1 and no held modifiers.

If TLS is enabled, first complete an ordinary TLS 1.2/1.3 handshake and validate the configured pin(s). The bytes below are then the application plaintext **inside** TLS. TLS record hex is not fixed: random values, certificates, key exchange and record encryption change each session. No fabricated TLS ciphertext or real certificate data is supplied. With TLS disabled, these are the direct TCP bytes. [V: §§2–8]

The setup uses the binary's **CROP-before-DSOP** order. DKDN is included to demonstrate the accepted legacy-style current packet, then DKDL demonstrates what a 1.8 server normally emits. Real event timing can vary; this is a valid explanatory transcript, not a claim that an unmodified server emits this exact synthetic input sequence.

| # | Direction | Message | Full framed bytes, including length | Meaning |
|---:|---|---|---|---|
| 1 | S→C | hello | `00 00 00 0b 53 79 6e 65 72 67 79 00 01 00 08` | Synergy wire 1.8 |
| 2 | C→S | hello-back | `00 00 00 17 53 79 6e 65 72 67 79 00 01 00 08 00 00 00 08 6d 61 63 2d 64 65 6d 6f` | Synthetic 8-byte computer name mac-demo |
| 3 | S→C | QINF | `00 00 00 04 51 49 4e 46` | Request client geometry |
| 4 | S→C | LSYN | `00 00 00 0a 4c 53 59 4e 00 00 00 02 65 6e` | Setup language unit en |
| 5 | C→S | DINF | `00 00 00 12 44 49 4e 46 00 00 00 00 07 80 04 38 00 00 03 c0 02 1c` | Origin 0,0; 1920×1080; obsolete field 0; cursor 960,540 |
| 6 | S→C | CIAK | `00 00 00 04 43 49 41 4b` | Accept DINF |
| 7 | S→C | CROP | `00 00 00 04 43 52 4f 50` | Reset options before DSOP |
| 8 | S→C | DSOP | `00 00 00 20 44 53 4f 50 00 00 00 06 48 41 52 54 00 00 0b b8 43 4c 50 53 00 00 00 01 43 4c 53 5a 00 00 04 00` | Six integers: heartbeat 3000 ms, clipboard on, limit 1024 KiB |
| 9 | S→C | CINN | `00 00 00 0e 43 49 4e 4e 07 7f 02 1c 00 00 00 01 00 00` | Enter Mac from its right edge; sequence 1; mask 0 |
| 10 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 11 | S→C | DMMV | `00 00 00 08 44 4d 4d 56 07 08 02 1c` | Absolute move to 1800,540 |
| 12 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 13 | S→C | DKDN | `00 00 00 0a 44 4b 44 4e 00 61 00 00 00 1e` | Accepted no-language key-down a, source button 0x001E |
| 14 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 15 | S→C | DKUP | `00 00 00 0a 44 4b 55 50 00 61 00 00 00 1e` | Matching key-up |
| 16 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 17 | S→C | DKDL | `00 00 00 10 44 4b 44 4c 00 62 00 00 00 30 00 00 00 02 65 6e` | Current 1.8 key-down b, source button 0x0030, language en |
| 18 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 19 | S→C | DKRP | `00 00 00 12 44 4b 52 50 00 62 00 00 00 01 00 30 00 00 00 02 65 6e` | One repeat of b; count precedes source button |
| 20 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 21 | S→C | DKUP | `00 00 00 0a 44 4b 55 50 00 62 00 00 00 30` | Matching b key-up |
| 22 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 23 | S→C | DMDN | `00 00 00 05 44 4d 44 4e 01` | Left button down |
| 24 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 25 | S→C | DMUP | `00 00 00 05 44 4d 55 50 01` | Left button up |
| 26 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 27 | S→C | DMWM | `00 00 00 08 44 4d 57 4d 00 00 00 78` | One positive vertical wheel notch |
| 28 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 29 | S→C | CCLP | `00 00 00 09 43 43 4c 50 00 00 00 00 00` | Server owns clipboard 0; server sequence 0 |
| 30 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 31 | S→C | DCLP | `00 00 00 10 44 43 4c 50 00 00 00 00 00 01 00 00 00 02 31 34` | Start clipboard 0: ASCII total size 14 |
| 32 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 33 | S→C | DCLP | `00 00 00 1c 44 43 4c 50 00 00 00 00 00 02 00 00 00 0e 00 00 00 01 00 00 00 00 00 00 00 02 48 69` | Data: one format, text ID 0, two bytes Hi |
| 34 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 35 | S→C | DCLP | `00 00 00 0e 44 43 4c 50 00 00 00 00 00 03 00 00 00 00` | End; empty chunk string |
| 36 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 37 | S→C | COUT | `00 00 00 04 43 4f 55 54` | Leave Mac; release synthetic keys |
| 38 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 39 | S→C | CALV | `00 00 00 04 43 41 4c 56` | Server heartbeat |
| 40 | C→S | CALV | `00 00 00 04 43 41 4c 56` | Echo heartbeat |
| 41 | C→S | CNOP | `00 00 00 04 43 4e 4f 50` | Normal client processing acknowledgement |
| 42 | S→C | CBYE | `00 00 00 04 43 42 59 45` | Close; no client reply required |

The clipboard data row contains this exact 14-byte inner container:

`00 00 00 01 | 00 00 00 00 | 00 00 00 02 | 48 69`

That is format-count 1, text-format 0, data length 2, and UTF-8 `Hi`. The start chunk's `31 34` is ASCII `14`; it is not a 16-bit integer. The end chunk still contains a `%s` length field of zero. These distinctions are required for interoperability.

A corresponding client-originated clipboard announcement after CINN uses sequence 1, for example:

`00 00 00 09 43 43 4c 50 00 00 00 00 01`

The following client-originated DCLP chunks would use the same sequence 1 in each chunk header. If the client is still entered, dirty clipboard payload transmission can be deferred until its leave path; CCLP itself only announces ownership.

Other short examples:

| Message | Full framed bytes | Meaning / verification |
|---|---|---|
| DMRM | `00 00 00 08 44 4d 52 4d ff fb 00 02` | Relative move -5,+2; signed big-endian fields, E03/E05 |
| CSEC | `00 00 00 05 43 53 45 43 01` | Screensaver on notification; Mac common wrapper is a no-op, §9 |
| EICV | `00 00 00 08 45 49 43 56 00 01 00 08` | Supported version 1.8, E04 |
| EBSY | `00 00 00 04 45 42 53 59` | Duplicate name, E03/E04 |
| EUNK | `00 00 00 04 45 55 4e 4b` | Unknown name, E03/E04 |
| EBAD | `00 00 00 04 45 42 41 44` | Protocol failure, E03/E04 |

## 14. Differences from public Deskflow

No public repository was downloaded and no network protocol experiment was run. This section compares the inspected binary with the classic public Synergy/Barrier protocol model and Deskflow ancestry, **not with an independently fetched current Deskflow commit**. A claim that all rows differ from every Deskflow release would be unsupported.

| Area | What must come from this binary rather than a generic ancestor description | Verified anchor |
|---|---|---|
| Product versus protocol | Product 1.21.1-beta advertises wire 1.8 | E19/E04 |
| Dialect | Both Synergy and Barrier hello names remain; client echoes the server's spelling | E04 |
| Negotiation | Client always replies 1.8; server supports only major 1/minor 0–8 | E04 |
| Key repeat | Current DKRP has four 16-bit fields **and a language string**, even in the build's inherited 1.1 writer | E03, `0x10003AAD8` |
| Key down | Version 1.8 server emits DKDL with language; DKDN remains accepted | `0x10004AA64`, E05 |
| Language sync | LSYN is two-byte units and is handled only before first DSOP | `0x1000AC4E0`, E05 |
| Clipboard | Numeric 1/2/3 chunk marks; 512 KiB data chunks; UTF-8 text, HTML=1, bitmap=2 | E07/E08 |
| Old clipboard | Base proxy send/receive functions are stubs; a historical nonchunked clipboard description is not implemented by those methods here | `0x100039478`, `0x100039F54` |
| Drag/files | DFTR/DDRG strings and proxy recognition survive, but working payload/state code does not | `0x10003F904–0x10003F954`, E05 |
| Secure input | SECN carries a blocking application name from macOS secure-input detection | E17, §11 |
| Resource limits | 4 MiB packets; 1 MiB `%s`; 1 Mi-element vectors; 1024-byte hello-back guard | E01/E02/E04 |
| Unknown messages | Existing proxies log/drain rather than guaranteeing a fatal reply for every unknown code | E05 |
| Screensaver | CSEC exists, but this Mac common screensaver wrapper is disabled/no-op; no screenSaverSync option in parser | `0x1000A0C0C`, E09 |
| Trust files | Current SHA-256 DER-certificate pin matching; typed v2 records; SHA-1 parsing is legacy support | E12/E13 |
| Configuration | Qt INI plus a small modern CLI; many historical CLI switches do not appear | E14/E19 |
| IPC | Product-version-matched UTF-8 newline commands over Qt local sockets | E18 |

The public ancestor is GPL-licensed. The report records interface facts, constants and observed behavior in original prose; it does not incorporate source/decompiled function bodies. An implementation should be written independently from the documented interface facts and should keep any further reference-source review separate and identified.

## 15. Open questions and verification boundary

| Priority | Open question / practical consequence | What is established; what would close the gap |
|---|---|---|
| High | Windows Synergy 1.21.x parity | This is the Mac build's client and server code. No Windows binary or session capture was available. Verify its advertised version, DKRP suffix, DSOP settings and certificate requirements against a copy or an isolated endpoint. |
| High | macOS 27 inactive-window activation | Default-source/global HID injection and missing explicit window/activation context are verified; the causal OS behavior and a universal fix are not. Run the matrix in §11 without disrupting the user's live input. |
| High | Disconnect during a mouse drag | Keyboard release is verified. A complete forced release of all synthetic mouse buttons on abrupt disconnect was not established. Exercise down → disconnect → local recovery on an isolated client. |
| Medium | Exact failure response for every malformed established message | Unknown-code draining and several exception handlers were inspected. Do not assume each malformed packet produces EBAD. An isolated differential harness should record close/reply behavior. |
| Medium | Historical peers below 1.8 | Server proxy classes exist, but current repeat and clipboard behavior has incompatible/stub paths. The supported Openergy target should be 1.8 until older versions are separately verified. |
| Medium | Full drag/file transfer spec | This binary does not implement the data paths. Only the literal signatures are justified. External ancestor semantics would be a separately labelled source, not verified Synergy behavior. |
| Medium | Simultaneous clipboard streams and maximum total clipboard | Static assembly state and unbounded aggregate accumulation exist. No valid multiplexing contract or safe total bound was found. Keep original parsing bounded and transfers serialized. |
| Medium | DIB variants and Windows HTML conversion | Mac bridge and format IDs are verified; every DIB compression/bit-depth variant and the Windows native clipboard adapter were not analyzed. |
| Medium | Supplementary Unicode keys | Wire KeyID is 16 bits. No general supplementary-character wire escape was established. Do not silently invent UTF-16 surrogate packets. |
| Medium | Qt enum INI representation and every GUI-side default | Catalog and main defaults are recovered; arbitrary edited QVariant enum spellings were not exercised. Use isolated QSettings read-back before shipping config import/export. |
| Low | Exact Qt-expanded IPC path and access policy | Logical names, local transport, options mask 7, framing and commands are verified. No live socket was queried; matching Qt header/platform behavior should establish the exact path/permissions. |
| Low | Certificate auto-generation/renewal UI flow | Generator algorithm/defaults are verified. User-facing generation/renewal triggers belong to the service/tray integration and were not exercised. |
| Low | Full current Deskflow delta | No current upstream snapshot was fetched. §14 is an ancestor-model comparison, not a version-to-version source diff. |

Verification performed: successful copied-executable help/version calls; REA native analysis; a complete static headless analysis of the pristine copy; cross-checks with original arm64 instructions, symbol/data tables, format strings, and local Apple/OpenSSL headers; arithmetic audit of the constructed hex packets. No unit tests, implementation tests, live input injection, target-network traffic, or live GUI automation was used. Wire compatibility with the user's unmodified Windows server remains an end-to-end milestone for the implementation, not a result of this documentation task.

## Appendix A. Exported named KeyID constants

Every row is read directly from `_kKeyNameMap` in the pristine binary. Each entry is 16 bytes: a fixup-encoded name pointer followed by the numeric KeyID. The address is the entry address. These are interface names/constants, not copied implementation logic. Printable letter/digit keys use their character values; this table includes named punctuation aliases and special keys.

| Name | KeyID | Entry address |
|---|---|---|
| `AltGr` | `0xEF7E` | `0x10048ED48` |
| `Alt_L` | `0xEFE9` | `0x10048ED58` |
| `Alt_R` | `0xEFEA` | `0x10048ED68` |
| `AppMail` | `0xE0B4` | `0x10048ED78` |
| `AppMedia` | `0xE0B5` | `0x10048ED88` |
| `AppUser1` | `0xE0B6` | `0x10048ED98` |
| `AppUser2` | `0xE0B7` | `0x10048EDA8` |
| `AudioDown` | `0xE0AE` | `0x10048EDB8` |
| `AudioMute` | `0xE0AD` | `0x10048EDC8` |
| `AudioNext` | `0xE0B0` | `0x10048EDD8` |
| `AudioPlay` | `0xE0B3` | `0x10048EDE8` |
| `AudioPrev` | `0xE0B1` | `0x10048EDF8` |
| `AudioStop` | `0xE0B2` | `0x10048EE08` |
| `AudioUp` | `0xE0AF` | `0x10048EE18` |
| `BackSpace` | `0xEF08` | `0x10048EE28` |
| `Begin` | `0xEF58` | `0x10048EE38` |
| `Break` | `0xEF6B` | `0x10048EE48` |
| `Cancel` | `0xEF69` | `0x10048EE58` |
| `CapsLock` | `0xEFE5` | `0x10048EE68` |
| `Clear` | `0xEF0B` | `0x10048EE78` |
| `Control_L` | `0xEFE3` | `0x10048EE88` |
| `Control_R` | `0xEFE4` | `0x10048EE98` |
| `Delete` | `0xEFFF` | `0x10048EEA8` |
| `Down` | `0xEF54` | `0x10048EEB8` |
| `Eject` | `0xE001` | `0x10048EEC8` |
| `End` | `0xEF57` | `0x10048EED8` |
| `Escape` | `0xEF1B` | `0x10048EEE8` |
| `Execute` | `0xEF62` | `0x10048EEF8` |
| `F1` | `0xEFBE` | `0x10048EF08` |
| `F2` | `0xEFBF` | `0x10048EF18` |
| `F3` | `0xEFC0` | `0x10048EF28` |
| `F4` | `0xEFC1` | `0x10048EF38` |
| `F5` | `0xEFC2` | `0x10048EF48` |
| `F6` | `0xEFC3` | `0x10048EF58` |
| `F7` | `0xEFC4` | `0x10048EF68` |
| `F8` | `0xEFC5` | `0x10048EF78` |
| `F9` | `0xEFC6` | `0x10048EF88` |
| `F10` | `0xEFC7` | `0x10048EF98` |
| `F11` | `0xEFC8` | `0x10048EFA8` |
| `F12` | `0xEFC9` | `0x10048EFB8` |
| `F13` | `0xEFCA` | `0x10048EFC8` |
| `F14` | `0xEFCB` | `0x10048EFD8` |
| `F15` | `0xEFCC` | `0x10048EFE8` |
| `F16` | `0xEFCD` | `0x10048EFF8` |
| `F17` | `0xEFCE` | `0x10048F008` |
| `F18` | `0xEFCF` | `0x10048F018` |
| `F19` | `0xEFD0` | `0x10048F028` |
| `F20` | `0xEFD1` | `0x10048F038` |
| `F21` | `0xEFD2` | `0x10048F048` |
| `F22` | `0xEFD3` | `0x10048F058` |
| `F23` | `0xEFD4` | `0x10048F068` |
| `F24` | `0xEFD5` | `0x10048F078` |
| `F25` | `0xEFD6` | `0x10048F088` |
| `F26` | `0xEFD7` | `0x10048F098` |
| `F27` | `0xEFD8` | `0x10048F0A8` |
| `F28` | `0xEFD9` | `0x10048F0B8` |
| `F29` | `0xEFDA` | `0x10048F0C8` |
| `F30` | `0xEFDB` | `0x10048F0D8` |
| `F31` | `0xEFDC` | `0x10048F0E8` |
| `F32` | `0xEFDD` | `0x10048F0F8` |
| `F33` | `0xEFDE` | `0x10048F108` |
| `F34` | `0xEFDF` | `0x10048F118` |
| `F35` | `0xEFE0` | `0x10048F128` |
| `Find` | `0xEF68` | `0x10048F138` |
| `Help` | `0xEF6A` | `0x10048F148` |
| `Henkan` | `0xEF23` | `0x10048F158` |
| `Home` | `0xEF50` | `0x10048F168` |
| `Hyper_L` | `0xEFED` | `0x10048F178` |
| `Hyper_R` | `0xEFEE` | `0x10048F188` |
| `Insert` | `0xEF63` | `0x10048F198` |
| `KP_0` | `0xEFB0` | `0x10048F1A8` |
| `KP_1` | `0xEFB1` | `0x10048F1B8` |
| `KP_2` | `0xEFB2` | `0x10048F1C8` |
| `KP_3` | `0xEFB3` | `0x10048F1D8` |
| `KP_4` | `0xEFB4` | `0x10048F1E8` |
| `KP_5` | `0xEFB5` | `0x10048F1F8` |
| `KP_6` | `0xEFB6` | `0x10048F208` |
| `KP_7` | `0xEFB7` | `0x10048F218` |
| `KP_8` | `0xEFB8` | `0x10048F228` |
| `KP_9` | `0xEFB9` | `0x10048F238` |
| `KP_Add` | `0xEFAB` | `0x10048F248` |
| `KP_Begin` | `0xEF9D` | `0x10048F258` |
| `KP_Decimal` | `0xEFAE` | `0x10048F268` |
| `KP_Delete` | `0xEF9F` | `0x10048F278` |
| `KP_Divide` | `0xEFAF` | `0x10048F288` |
| `KP_Down` | `0xEF99` | `0x10048F298` |
| `KP_End` | `0xEF9C` | `0x10048F2A8` |
| `KP_Enter` | `0xEF8D` | `0x10048F2B8` |
| `KP_Equal` | `0xEFBD` | `0x10048F2C8` |
| `KP_F1` | `0xEF91` | `0x10048F2D8` |
| `KP_F2` | `0xEF92` | `0x10048F2E8` |
| `KP_F3` | `0xEF93` | `0x10048F2F8` |
| `KP_F4` | `0xEF94` | `0x10048F308` |
| `KP_Home` | `0xEF95` | `0x10048F318` |
| `KP_Insert` | `0xEF9E` | `0x10048F328` |
| `KP_Left` | `0xEF96` | `0x10048F338` |
| `KP_Multiply` | `0xEFAA` | `0x10048F348` |
| `KP_PageDown` | `0xEF9B` | `0x10048F358` |
| `KP_PageUp` | `0xEF9A` | `0x10048F368` |
| `KP_Right` | `0xEF98` | `0x10048F378` |
| `KP_Separator` | `0xEFAC` | `0x10048F388` |
| `KP_Space` | `0xEF80` | `0x10048F398` |
| `KP_Subtract` | `0xEFAD` | `0x10048F3A8` |
| `KP_Tab` | `0xEF89` | `0x10048F3B8` |
| `KP_Up` | `0xEF97` | `0x10048F3C8` |
| `Left` | `0xEF51` | `0x10048F3D8` |
| `LeftTab` | `0xEE20` | `0x10048F3E8` |
| `Linefeed` | `0xEF0A` | `0x10048F3F8` |
| `Menu` | `0xEF67` | `0x10048F408` |
| `Meta_L` | `0xEFE7` | `0x10048F418` |
| `Meta_R` | `0xEFE8` | `0x10048F428` |
| `NumLock` | `0xEF7F` | `0x10048F438` |
| `PageDown` | `0xEF56` | `0x10048F448` |
| `PageUp` | `0xEF55` | `0x10048F458` |
| `Pause` | `0xEF13` | `0x10048F468` |
| `Print` | `0xEF61` | `0x10048F478` |
| `Redo` | `0xEF66` | `0x10048F488` |
| `Return` | `0xEF0D` | `0x10048F498` |
| `Right` | `0xEF53` | `0x10048F4A8` |
| `ScrollLock` | `0xEF14` | `0x10048F4B8` |
| `Select` | `0xEF60` | `0x10048F4C8` |
| `ShiftLock` | `0xEFE6` | `0x10048F4D8` |
| `Shift_L` | `0xEFE1` | `0x10048F4E8` |
| `Shift_R` | `0xEFE2` | `0x10048F4F8` |
| `Sleep` | `0xE05F` | `0x10048F508` |
| `Super_L` | `0xEFEB` | `0x10048F518` |
| `Super_R` | `0xEFEC` | `0x10048F528` |
| `SysReq` | `0xEF15` | `0x10048F538` |
| `Tab` | `0xEF09` | `0x10048F548` |
| `Undo` | `0xEF65` | `0x10048F558` |
| `Up` | `0xEF52` | `0x10048F568` |
| `WWWBack` | `0xE0A6` | `0x10048F578` |
| `WWWFavorites` | `0xE0AB` | `0x10048F588` |
| `WWWForward` | `0xE0A7` | `0x10048F598` |
| `WWWHome` | `0xE0AC` | `0x10048F5A8` |
| `WWWRefresh` | `0xE0A8` | `0x10048F5B8` |
| `WWWSearch` | `0xE0AA` | `0x10048F5C8` |
| `WWWStop` | `0xE0A9` | `0x10048F5D8` |
| `Zenkaku` | `0xEF2A` | `0x10048F5E8` |
| `Space` | `0x0020` | `0x10048F5F8` |
| `Exclaim` | `0x0021` | `0x10048F608` |
| `DoubleQuote` | `0x0022` | `0x10048F618` |
| `Number` | `0x0023` | `0x10048F628` |
| `Dollar` | `0x0024` | `0x10048F638` |
| `Percent` | `0x0025` | `0x10048F648` |
| `Ampersand` | `0x0026` | `0x10048F658` |
| `Apostrophe` | `0x0027` | `0x10048F668` |
| `ParenthesisL` | `0x0028` | `0x10048F678` |
| `ParenthesisR` | `0x0029` | `0x10048F688` |
| `Asterisk` | `0x002A` | `0x10048F698` |
| `Plus` | `0x002B` | `0x10048F6A8` |
| `Comma` | `0x002C` | `0x10048F6B8` |
| `Minus` | `0x002D` | `0x10048F6C8` |
| `Period` | `0x002E` | `0x10048F6D8` |
| `Slash` | `0x002F` | `0x10048F6E8` |
| `Colon` | `0x003A` | `0x10048F6F8` |
| `Semicolon` | `0x003B` | `0x10048F708` |
| `Less` | `0x003C` | `0x10048F718` |
| `Equal` | `0x003D` | `0x10048F728` |
| `Greater` | `0x003E` | `0x10048F738` |
| `Question` | `0x003F` | `0x10048F748` |
| `At` | `0x0040` | `0x10048F758` |
| `BracketL` | `0x005B` | `0x10048F768` |
| `Backslash` | `0x005C` | `0x10048F778` |
| `BracketR` | `0x005D` | `0x10048F788` |
| `Circumflex` | `0x005E` | `0x10048F798` |
| `Underscore` | `0x005F` | `0x10048F7A8` |
| `Grave` | `0x0060` | `0x10048F7B8` |
| `BraceL` | `0x007B` | `0x10048F7C8` |
| `Bar` | `0x007C` | `0x10048F7D8` |
| `BraceR` | `0x007D` | `0x10048F7E8` |
| `Tilde` | `0x007E` | `0x10048F7F8` |

## Appendix B. Static macOS special-key translation table

The table has 60 entries (480 bytes). The input source key-button value for a Mac sender is the native virtual keycode plus 1. Unnamed entries are preserved as numeric constants; their semantic names were not recovered from the exported key-name table. Layout-based printable keys are built dynamically as described in §7.

| KeyID | Name | macOS virtual keycode | Evidence address |
|---|---|---|---|
| `0xEF51` | `Left` | `0x7B` | `0x100356D0C` |
| `0xEF53` | `Right` | `0x7C` | `0x100356D14` |
| `0xEF52` | `Up` | `0x7E` | `0x100356D1C` |
| `0xEF54` | `Down` | `0x7D` | `0x100356D24` |
| `0xEF50` | `Home` | `0x73` | `0x100356D2C` |
| `0xEF57` | `End` | `0x77` | `0x100356D34` |
| `0xEF55` | `PageUp` | `0x74` | `0x100356D3C` |
| `0xEF56` | `PageDown` | `0x79` | `0x100356D44` |
| `0xEF63` | `Insert` | `0x72` | `0x100356D4C` |
| `0xEFBE` | `F1` | `0x7A` | `0x100356D54` |
| `0xEFBF` | `F2` | `0x78` | `0x100356D5C` |
| `0xEFC0` | `F3` | `0x63` | `0x100356D64` |
| `0xEFC1` | `F4` | `0x76` | `0x100356D6C` |
| `0xEFC2` | `F5` | `0x60` | `0x100356D74` |
| `0xEFC3` | `F6` | `0x61` | `0x100356D7C` |
| `0xEFC4` | `F7` | `0x62` | `0x100356D84` |
| `0xEFC5` | `F8` | `0x64` | `0x100356D8C` |
| `0xEFC6` | `F9` | `0x65` | `0x100356D94` |
| `0xEFC7` | `F10` | `0x6D` | `0x100356D9C` |
| `0xEFC8` | `F11` | `0x67` | `0x100356DA4` |
| `0xEFC9` | `F12` | `0x6F` | `0x100356DAC` |
| `0xEFCA` | `F13` | `0x69` | `0x100356DB4` |
| `0xEFCB` | `F14` | `0x6B` | `0x100356DBC` |
| `0xEFCC` | `F15` | `0x71` | `0x100356DC4` |
| `0xEFCD` | `F16` | `0x6A` | `0x100356DCC` |
| `0xEFB0` | `KP_0` | `0x52` | `0x100356DD4` |
| `0xEFB1` | `KP_1` | `0x53` | `0x100356DDC` |
| `0xEFB2` | `KP_2` | `0x54` | `0x100356DE4` |
| `0xEFB3` | `KP_3` | `0x55` | `0x100356DEC` |
| `0xEFB4` | `KP_4` | `0x56` | `0x100356DF4` |
| `0xEFB5` | `KP_5` | `0x57` | `0x100356DFC` |
| `0xEFB6` | `KP_6` | `0x58` | `0x100356E04` |
| `0xEFB7` | `KP_7` | `0x59` | `0x100356E0C` |
| `0xEFB8` | `KP_8` | `0x5B` | `0x100356E14` |
| `0xEFB9` | `KP_9` | `0x5C` | `0x100356E1C` |
| `0xEFAE` | `KP_Decimal` | `0x41` | `0x100356E24` |
| `0xEFBD` | `KP_Equal` | `0x51` | `0x100356E2C` |
| `0xEFAA` | `KP_Multiply` | `0x43` | `0x100356E34` |
| `0xEFAB` | `KP_Add` | `0x45` | `0x100356E3C` |
| `0xEFAF` | `KP_Divide` | `0x4B` | `0x100356E44` |
| `0xEFAD` | `KP_Subtract` | `0x4E` | `0x100356E4C` |
| `0xEF8D` | `KP_Enter` | `0x4C` | `0x100356E54` |
| `0xEFE1` | `Shift_L` | `0x38` | `0x100356E5C` |
| `0xEFE2` | `Shift_R` | `0x38` | `0x100356E64` |
| `0xEFE3` | `Control_L` | `0x3B` | `0x100356E6C` |
| `0xEFE4` | `Control_R` | `0x3B` | `0x100356E74` |
| `0xEFE9` | `Alt_L` | `0x3A` | `0x100356E7C` |
| `0xEFEA` | `Alt_R` | `0x3A` | `0x100356E84` |
| `0xEFEB` | `Super_L` | `0x37` | `0x100356E8C` |
| `0xEFEC` | `Super_R` | `0x37` | `0x100356E94` |
| `0xEFE7` | `Meta_L` | `0x37` | `0x100356E9C` |
| `0xEFE8` | `Meta_R` | `0x37` | `0x100356EA4` |
| `0xEF7F` | `NumLock` | `0x47` | `0x100356EAC` |
| `0xEFE5` | `CapsLock` | `0x39` | `0x100356EB4` |
| `0xEF23` | `Henkan` | `0x68` | `0x100356EBC` |
| `0xEF2A` | `Zenkaku` | `0x66` | `0x100356EC4` |
| `0xE0C0` | `unnamed` | `0xA0` | `0x100356ECC` |
| `0xE0C1` | `unnamed` | `0x83` | `0x100356ED4` |
| `0xE0B9` | `unnamed` | `0x90` | `0x100356EDC` |
| `0xE0B8` | `unnamed` | `0x91` | `0x100356EE4` |
