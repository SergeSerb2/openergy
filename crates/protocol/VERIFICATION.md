# WP1 wire codec verification

For the later framing and legacy-repeat fixes, see
[the release audit](RELEASE-AUDIT.md). The results below are the original WP1
snapshot, before those fixes.

Verified on 2026-10-21, macOS arm64, Rust 1.99.1, Docker 29.8.2.
Only `crates/protocol/` source files changed. No dependencies were added.
No unit tests, mocks, native input APIs, host pasteboard APIs, TCC requests,
production ports, or LAN connections were used.

## Implementation and caller contract

- All current message signatures in `docs/re/03-core-protocol.md` §4 are
  implemented. DKRP always has its trailing language string; DKDL is selected
  by `KeyDown.lang = Some`. Wheel order is horizontal, then vertical.
- Each connection gets one `Decoder`. Its first frame must be an 11-byte
  Hello or a HelloBack of at most 1024 payload bytes. It accepts either
  product name without regard to case and preserves its exact spelling.
  A second hello produces `DecodeError::BadHello`.
- Packet, string and vector caps are 4 MiB, 1 MiB and 1,048,576 elements.
  The caller's packet cap can tighten the hard cap. Frame/hello size checks
  run as soon as the length prefix arrives, before payload allocation.
  Names, language strings and SECN enforce the architecture's UTF-8, NUL
  and byte-length rules. Odd LSYN data reaches session for its ignore/warn
  policy; the encoder emits only even-length language lists.
- `feed` reports fatal framing errors. `next_message` returns complete
  messages or typed field errors. Callers close on either error.
  `Unknown.payload_len` counts bytes after the code. An unknown frame is
  consumed whole, and the following message remains available.
- Additive method: `pub fn Decoder::finish(&mut self) -> Result<(), DecodeError>`.
  Call it on TCP EOF to detect a partial prefix or payload. Existing public
  signatures and message/error variants remain intact.
- `encode` appends a complete frame to the caller's reused Vec. Validation
  errors leave it unchanged. Input encoding uses no temporary heap buffers.
  Unknown messages cannot be re-encoded because the frozen type retains
  their size, rather than their bytes.
- `protocol::keys` exports all 172 named KeyIDs from Appendix A, plus
  `keys::modifiers`, `keys::buttons` and wheel units. `protocol::options`
  retains the descriptive names and adds all 25 four-character ID aliases.
- The frozen clipboard marshal/unmarshal helpers implement wire containers:
  sorted text/HTML/DIB formats, UTF-8 validation for text/HTML, checked
  lengths and unknown-format skipping. Architecture §2.6 assigns production
  chunk scheduling, capped reassembly, timeout and ownership policy to
  `session::clipboard`. Protocol exposes `Chunk` and `CLIPBOARD_CHUNK_LEN`;
  the probe's capped transfer observer is verification code only.
- The library and Rust probe forbid unsafe code. No implementation body
  returns the retained Step 0 `NotImplemented` error variants.

## Release gates

Exact commands, from the worktree root:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt -p openergy-protocol
cargo build --release
cargo clippy --release -p openergy-protocol -- -D warnings
cargo clippy --release -p openergy-protocol --examples -- -D warnings
git diff --check
```

Trimmed real output:

```text
Finished `release` profile [optimized] target(s) in 4.69s
Checking openergy-protocol v0.1.0
Finished `release` profile [optimized] target(s) in 0.25s
Checking openergy-protocol v0.1.0
Finished `release` profile [optimized] target(s) in 0.17s
```

All commands exited 0. The whole workspace release build had zero warnings.

## Golden and hostile-input probe

```sh
cargo run --release --example wire_probe -- golden
```

`fixtures/session-1.8.hex` transcribes every row of 03 §13. Those bytes are a
constructed specification example. The probe compares an independently
written message list to each fixture and feeds the two TCP directions to
separate decoders, as required by the first-frame contract.

Trimmed real output:

```text
isolated prefix probe: 20000 oversized lengths + 10000 empty 4 MiB claims PASS
getrusage prefix peak_rss_bytes=2326528
golden transcript: 42 frames, 525 bytes, exact encode/decode PASS
incremental: 5000 deterministic split schedules in both TCP directions (bytewise and batched drain) PASS
additional variants, signed boundaries, mandatory repeat language, 1 MiB chunk, reused Vec PASS
hostile limits, UTF-8/NUL, hello, DSOP, DCLP size/marks/totals, EOF, Unknown then CALV PASS
clipboard container: sorted formats, unknown skip, truncation; 8 MiB / 259 DCLP packets with interleaved input PASS
hostile arbitrary framed payloads: 20000 completed without panic
oversized prefix repetitions: 10000 rejected before payload allocation; golden PASS
getrusage full peak_rss_bytes=40730624
```

Python's standard `resource.getrusage(RUSAGE_CHILDREN)` measures the actual
Rust worker processes without FFI or a new Rust dependency. Prefix-only RSS
is measured first in a separate process. The full stress peak includes
large frame fixtures, the 8 MiB binary container and its assembled copy.
This run does not measure daemon idle RSS.

Directed failures include u32::MAX and 4 MiB+1 frame lengths, a 1025-byte
hello claim, truncated hello/EOF, invalid UTF-8/NUL names, odd/oversized DSOP
counts, oversized strings, missing DKRP language, trailing bytes, zero mouse
button, invalid DCLP mark/decimal/overflow/end/length and assembled-total
mismatch. Exact 1024-byte HelloBack and 4 MiB unknown frames pass. Unknown
followed by CALV stays synchronized. Failed encoding leaves prior output
bytes intact.

## Real Deskflow and Barrier sockets, in both roles

Build the probe, then run the repeatable driver:

```sh
cargo build --release -p openergy-protocol --example wire_probe
export LAB_NAME=oe-lab-wp1 LAB_PORT_RANGE=24910-24919
python3 crates/protocol/examples/interop_probe.py
```

The driver sets the same lab identity and range and always calls `lab.sh down`
in its cleanup. It only terminates probe processes that it started.
It invokes the following real modes:

```sh
tools/interop-lab/lab.sh server deskflow --port 24910 --screens lab,openergy-mac --position right --tls off
target/release/examples/wire_probe client --server 127.0.0.1:24910 --name openergy-mac --duration-secs 18
target/release/examples/wire_probe server --port 24960 --hold-ms 8000
tools/interop-lab/lab.sh client deskflow --server host.docker.internal:24960 --name lab --tls off

tools/interop-lab/lab.sh server barrier --port 24911 --screens lab,openergy-mac --position right --tls off
target/release/examples/wire_probe client --server 127.0.0.1:24911 --name openergy-mac --duration-secs 18
target/release/examples/wire_probe server --port 24961 --hold-ms 8000 --product Barrier --minor 6
tools/interop-lab/lab.sh client barrier --server host.docker.internal:24961 --name lab --tls off
```

The host servers bind **127.0.0.1 only**. OrbStack's documented host alias
successfully reaches those listeners. Deskflow uses Synergy 1.8; Barrier
uses Barrier 1.6. The probe omits LSYN/DKDL when sending to the 1.6 client.
The current 1.8 DKRP codec remains strict; no legacy repeat extension was
added. Native Windows Synergy, TLS, native macOS input and DIB rendering are
outside this codec verification.

For each reference server, the driver sets the container clipboard, crosses
its right edge with relative motion, moves, clicks, types three keys, scrolls,
and returns left. The probe announces virtual clipboard ownership on enter.
A second local clipboard change and edge crossing makes the primary regain
ownership and emit CCLP, followed by DCLP. Every received frame must decode;
coordinates, buttons and wheel values must also pass plausible-value checks.
The client asserts setup, keepalive, every requested input kind, leave,
CCLP/DCLP and a complete nonempty UTF-8 text transfer.

For each reference client, the probe performs Hello/QINF/[LSYN]/DINF/CIAK/
CROP/DSOP/CALV, then sends enter, absolute movement, a left click, one letter,
a wheel notch, CCLP and Start/Data/End text chunks. During the hold, the
real lab pointer must equal `(321,234)` and `lab.sh clipboard get` must equal
the expected 29-byte string. It then sends COUT. `lab.sh events dump` must
show exactly two raw button down/up pairs and one raw key down/up pair, with
the exact click/key/wheel order and details. The pinned Xvfb US key map is
used for the letter check. No key values or clipboard contents are printed.

Final real result (trimmed):

```text
wire_probe client <- deskflow server (24910)
client PASS counts={"CALV": 5, "CCLP": 1, "CIAK": 1, "CINN": 2, "COUT": 2, "CROP": 1, "DCLP": 9, "DKDL": 3, "DKUP": 3, "DMDN": 1, "DMMV": 1, "DMUP": 1, "DMWM": 1, "DSOP": 1, "Hello": 1, "LSYN": 1, "QINF": 1}, nonempty_text_transfers=2
wire_probe server -> deskflow client (24960)
X11 pointer=321,234; clipboard exact comparison PASS bytes=29
server PASS hello-back, DINF=1920x1080, CALV echo; input + clipboard sent, COUT sent
XInput exact raw event counts PASS {'RawButtonPress': 2, 'RawButtonRelease': 2, 'RawKeyPress': 1, 'RawKeyRelease': 1}; raw motion=4
XInput exact click/key/wheel detail and order PASS (key contents not logged)
wire_probe client <- barrier server (24911)
client PASS counts={"CALV": 8, "CCLP": 1, "CIAK": 1, "CINN": 2, "COUT": 2, "CROP": 1, "DCLP": 9, "DKDN": 3, "DKUP": 3, "DMDN": 1, "DMMV": 1, "DMUP": 1, "DMWM": 1, "DSOP": 1, "Hello": 1, "QINF": 1}, nonempty_text_transfers=2
wire_probe server -> barrier client (24961)
X11 pointer=321,234; clipboard exact comparison PASS bytes=29
server PASS hello-back, DINF=1920x1080, CALV echo; input + clipboard sent, COUT sent
XInput exact raw event counts PASS {'RawButtonPress': 2, 'RawButtonRelease': 2, 'RawKeyPress': 1, 'RawKeyRelease': 1}; raw motion=4
XInput exact click/key/wheel detail and order PASS (key contents not logged)
Deskflow 1.27 / Barrier 2.4, both roles PASS
container=/oe-lab-wp1 running=false image=openergy-interop:latest
```

No WP1 implementation gap or cross-crate contract change is pending.
Production clipboard transfer management remains assigned to WP6 by §2.6.
