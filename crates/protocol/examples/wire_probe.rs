//! Real codec/TCP probe. No host input or clipboard APIs are used.
#![forbid(unsafe_code)]
use openergy_protocol::{
    encode, marshal_clipboard, options, unmarshal_clipboard, Chunk, ClipboardId, DecodeError, Decoder, Message,
    ProtocolVersion, CLIPBOARD_CHUNK_LEN, MAX_FRAME_LEN, MAX_STRING_LEN,
};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    process::{Command, ExitCode},
    time::{Duration, Instant},
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
const CLIPBOARD_CAP: usize = 64 * 1024 * 1024;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("wire_probe: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("golden") if args.iter().any(|s| s == "--prefix-worker") => prefixes(),
        Some("golden") if args.iter().any(|s| s == "--worker") => golden(),
        Some("golden") => {
            // Python exposes getrusage without adding FFI/unsafe or dependencies
            // to the codec. RUSAGE_CHILDREN measures the actual probe process.
            let status = Command::new("python3").arg("-c").arg(
                "import resource,subprocess,sys
for mode,label in [('--prefix-worker','prefix'),('--worker','full')]:
 r=subprocess.run([sys.argv[1],'golden',mode])
 u=resource.getrusage(resource.RUSAGE_CHILDREN)
 print('getrusage '+label+' peak_rss_bytes='+str(int(u.ru_maxrss*(1 if sys.platform=='darwin' else 1024))),flush=True)
 if r.returncode: sys.exit(r.returncode)"
            ).arg(std::env::current_exe()?).status()?;
            ensure(status.success(), "golden worker failed")
        }
        Some("client") => client(&args),
        Some("server") => server(&args),
        _ => Err("usage: wire_probe golden | client --server 127.0.0.1:PORT --name NAME [--duration-secs 18] | server --port PORT [--product Barrier] [--minor 6] [--hold-ms 8000]".into()),
    }
}

fn ensure(ok: bool, message: &str) -> Result {
    if ok {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}

fn arg<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w.first().is_some_and(|s| s == key))
        .and_then(|w| w.get(1))
        .map(String::as_str)
}

fn v18() -> ProtocolVersion {
    ProtocolVersion::V1_8
}
fn info() -> Message {
    Message::ScreenInfo {
        x: 0,
        y: 0,
        w: 1920,
        h: 1080,
        warp: 0,
        mx: 960,
        my: 540,
    }
}
fn opts() -> Message {
    Message::SetOptions {
        options: vec![
            (options::HEARTBEAT, 3000),
            (options::CLIPBOARD_SHARING, 1),
            (options::CLIPBOARD_SHARING_SIZE, 1024),
        ],
    }
}
fn clip(chunk: Chunk) -> Message {
    Message::ClipboardData {
        id: ClipboardId::CLIPBOARD,
        sequence: 0,
        chunk,
    }
}

fn transcript() -> Vec<Message> {
    use Message::*;
    vec![
        Hello {
            product: "Synergy".into(),
            version: v18(),
        },
        HelloBack {
            product: "Synergy".into(),
            version: v18(),
            name: "mac-demo".into(),
        },
        QueryInfo,
        LanguageSync { languages: "en".into() },
        info(),
        InfoAck,
        ResetOptions,
        opts(),
        Enter {
            x: 1919,
            y: 540,
            sequence: 1,
            modifiers: 0,
        },
        Noop,
        MouseMoveAbs { x: 1800, y: 540 },
        Noop,
        KeyDown {
            key: 0x61,
            modifiers: 0,
            button: 0x1e,
            lang: None,
        },
        Noop,
        KeyUp {
            key: 0x61,
            modifiers: 0,
            button: 0x1e,
        },
        Noop,
        KeyDown {
            key: 0x62,
            modifiers: 0,
            button: 0x30,
            lang: Some("en".into()),
        },
        Noop,
        KeyRepeat {
            key: 0x62,
            modifiers: 0,
            count: 1,
            button: 0x30,
            lang: "en".into(),
        },
        Noop,
        KeyUp {
            key: 0x62,
            modifiers: 0,
            button: 0x30,
        },
        Noop,
        MouseDown { button: 1 },
        Noop,
        MouseUp { button: 1 },
        Noop,
        MouseWheel { dx: 0, dy: 120 },
        Noop,
        ClipboardGrab {
            id: ClipboardId::CLIPBOARD,
            sequence: 0,
        },
        Noop,
        clip(Chunk::Start { total_len: 14 }),
        Noop,
        clip(Chunk::Data(vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, b'H', b'i'])),
        Noop,
        clip(Chunk::End),
        Noop,
        Leave,
        Noop,
        KeepAlive,
        KeepAlive,
        Noop,
        Bye,
    ]
}

fn hex(text: &str) -> Result<Vec<u8>> {
    text.split_whitespace()
        .map(|s| u8::from_str_radix(s, 16).map_err(Into::into))
        .collect()
}

fn frame(payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(payload);
    bytes
}

fn decode(bytes: &[u8]) -> std::result::Result<Message, DecodeError> {
    let mut decoder = Decoder::new(MAX_FRAME_LEN);
    let product = bytes.get(4..11);
    let is_hello = product.is_some_and(|p| p.eq_ignore_ascii_case(b"Synergy") || p.eq_ignore_ascii_case(b"Barrier"));
    if !is_hello {
        prime(&mut decoder)?;
    }
    decoder.feed(bytes)?;
    decoder.finish()?;
    decoder.next_message().ok_or(DecodeError::Truncated)?
}

fn prime(decoder: &mut Decoder) -> std::result::Result<(), DecodeError> {
    decoder.feed(&frame(b"Synergy\0\x01\0\x08"))?;
    decoder.next_message().ok_or(DecodeError::Truncated)??;
    Ok(())
}

fn prefixes() -> Result {
    for _ in 0..10_000 {
        for n in [u32::MAX, MAX_FRAME_LEN + 1] {
            let mut decoder = Decoder::new(MAX_FRAME_LEN);
            ensure(
                decoder.feed(&n.to_be_bytes()) == Err(DecodeError::OversizedFrame(n)),
                "oversized prefix",
            )?;
        }
        // Even a valid large claimed size must not reserve its payload yet.
        let mut decoder = Decoder::new(MAX_FRAME_LEN);
        prime(&mut decoder)?;
        decoder.feed(&MAX_FRAME_LEN.to_be_bytes())?;
    }
    println!("isolated prefix probe: 20000 oversized lengths + 10000 empty 4 MiB claims PASS");
    Ok(())
}

fn golden() -> Result {
    let messages = transcript();
    let fixtures: Vec<Vec<u8>> = include_str!("../fixtures/session-1.8.hex")
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
        .map(|s| hex(&s.split_whitespace().skip(2).collect::<Vec<_>>().join(" ")))
        .collect::<Result<_>>()?;
    ensure(fixtures.len() == messages.len(), "fixture row count")?;
    let mut stream = Vec::new();
    let mut reused = Vec::with_capacity(1024);
    for (message, fixture) in messages.iter().zip(&fixtures) {
        reused.clear();
        encode(message, &mut reused)?;
        ensure(&reused == fixture, "transcript encode differs from fixture")?;
        ensure(decode(fixture)? == *message, "transcript decode differs")?;
        stream.extend_from_slice(fixture);
    }
    println!(
        "golden transcript: {} frames, {} bytes, exact encode/decode PASS",
        messages.len(),
        stream.len()
    );
    // Each TCP direction gets its own decoder. A transcript includes both
    // Hello and HelloBack, which must never share one direction's decoder.
    let client_rows = [2, 5, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 41];
    let mut rng = Rng(0x8c31_a027_91e4_d605);
    for client_direction in [false, true] {
        let expected: Vec<Message> = messages
            .iter()
            .enumerate()
            .filter(|(i, _)| client_rows.contains(&(i + 1)) == client_direction)
            .map(|(_, m)| m.clone())
            .collect();
        let directional: Vec<u8> = fixtures
            .iter()
            .enumerate()
            .filter(|(i, _)| client_rows.contains(&(i + 1)) == client_direction)
            .flat_map(|(_, f)| f.iter().copied())
            .collect();
        for round in 0..2500 {
            let mut decoder = Decoder::new(MAX_FRAME_LEN);
            let mut rest = directional.as_slice();
            let mut received = Vec::new();
            while !rest.is_empty() {
                let n = if round == 0 {
                    1
                } else {
                    (rng.next() as usize % 193 + 1).min(rest.len())
                };
                decoder.feed(rest.get(..n).ok_or("split")?)?;
                rest = rest.get(n..).ok_or("split remainder")?;
                if round % 2 == 0 {
                    while let Some(message) = decoder.next_message() {
                        received.push(message?);
                    }
                }
            }
            decoder.finish()?;
            while let Some(message) = decoder.next_message() {
                received.push(message?);
            }
            ensure(received == expected, "random TCP split round-trip")?;
        }
    }
    println!(
        "incremental: 5000 deterministic split schedules in both TCP directions (bytewise and batched drain) PASS"
    );
    extra_messages()?;
    hostile()?;
    clipboard_probe()?;
    // Arbitrary frames, in addition to directed malformed known fields.
    for _ in 0..20_000 {
        let len = rng.next() as usize % 257;
        let payload: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        let _ = decode(&frame(&payload));
    }
    println!("hostile arbitrary framed payloads: 20000 completed without panic");
    // Out-of-range frame lengths are rejected repeatedly from just four bytes.
    for _ in 0..10_000 {
        let mut d = Decoder::new(MAX_FRAME_LEN);
        ensure(
            d.feed(&u32::MAX.to_be_bytes()) == Err(DecodeError::OversizedFrame(u32::MAX)),
            "oversized prefix",
        )?;
    }
    println!("oversized prefix repetitions: 10000 rejected before payload allocation; golden PASS");
    Ok(())
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn extra_messages() -> Result {
    use Message::*;
    let messages = vec![
        Hello {
            product: "bArRiEr".into(),
            version: ProtocolVersion { major: 1, minor: 6 },
        },
        HelloBack {
            product: "sYnErGy".into(),
            version: v18(),
            name: "écran".into(),
        },
        ScreenSaver { on: false },
        ScreenSaver { on: true },
        MouseMoveRel {
            dx: i16::MIN,
            dy: i16::MAX,
        },
        MouseWheel { dx: -120, dy: 240 },
        FileTransfer {
            mark: 254,
            payload: vec![0, 255, 128],
        },
        DragInfo {
            count: -1,
            payload: "raw\0metadata".into(),
        },
        SecureInputNotification {
            name: "application".into(),
        },
        ErrorIncompatible { major: 1, minor: 8 },
        ErrorBusy,
        ErrorUnknown,
        ErrorBad,
        SetOptions {
            options: vec![(0xffff_ffff, i32::MIN), (options::MOD_SUPER, -1)],
        },
        clip(Chunk::Start { total_len: u32::MAX }),
        clip(Chunk::Start { total_len: 0 }),
        clip(Chunk::Data(vec![0; MAX_STRING_LEN as usize])),
        KeyRepeat {
            key: u16::MAX,
            modifiers: u16::MAX,
            count: u16::MAX,
            button: u16::MAX,
            lang: String::new(),
        },
    ];
    let mut output = Vec::new();
    for message in &messages {
        output.clear();
        encode(message, &mut output)?;
        ensure(decode(&output)? == *message, "additional message round-trip")?;
    }
    // Append semantics and stable buffer capacity across input encoding.
    output.clear();
    for _ in 0..10_000 {
        encode(&MouseMoveAbs { x: -1, y: 42 }, &mut output)?;
    }
    let capacity = output.capacity();
    output.clear();
    for _ in 0..10_000 {
        encode(&MouseWheel { dx: -120, dy: 120 }, &mut output)?;
    }
    ensure(
        output.capacity() == capacity && output.len() == 120_000,
        "reused caller Vec",
    )?;
    println!("additional variants, signed boundaries, mandatory repeat language, 1 MiB chunk, reused Vec PASS");
    Ok(())
}

fn expect_error(payload: &[u8], expected: DecodeError) -> Result {
    ensure(decode(&frame(payload)) == Err(expected), "directed hostile error type")
}

fn hostile() -> Result {
    for n in [u32::MAX, MAX_FRAME_LEN + 1] {
        let mut d = Decoder::new(u32::MAX);
        ensure(
            d.feed(&n.to_be_bytes()) == Err(DecodeError::OversizedFrame(n)),
            "hard packet cap",
        )?;
        ensure(d.feed(&[]) == Err(DecodeError::OversizedFrame(n)), "poisoned decoder")?;
    }
    expect_error(b"Synergy\0\x01", DecodeError::Truncated)?;
    expect_error(b"Synergy\0\x01\0\x08\0\0\0\x01\xff", DecodeError::BadUtf8)?;
    expect_error(b"Synergy\0\x01\0\x08\0\0\0\x01\0", DecodeError::BadValue("NUL in text"))?;
    let mut large_hello = b"Synergy\0\x01\0\x08\0\0\x03\xf2".to_vec();
    large_hello.extend_from_slice(&vec![b'n'; 1010]);
    expect_error(&large_hello, DecodeError::BadHello)?;
    expect_error(b"DSOP\0\0\0\x01HART", DecodeError::BadValue("odd option count"))?;
    expect_error(b"DSOP\0\x10\0\x02", DecodeError::BadValue("vector length cap"))?;
    expect_error(b"DSOP\0\x10\0\0", DecodeError::Truncated)?;
    expect_error(b"DCLP\0\0\0\0\0\x02\0\0\0\x02a", DecodeError::Truncated)?;
    expect_error(
        b"DCLP\0\0\0\0\0\x03\0\0\0\x01a",
        DecodeError::BadValue("nonempty clipboard end"),
    )?;
    expect_error(
        b"DCLP\0\0\0\0\0\x04\0\0\0\0",
        DecodeError::BadValue("clipboard chunk mark"),
    )?;
    expect_error(
        b"DCLP\0\0\0\0\0\x01\0\0\0\x02-1",
        DecodeError::BadValue("clipboard total is not decimal"),
    )?;
    expect_error(
        b"DCLP\0\0\0\0\0\x01\0\0\0\x0a4294967296",
        DecodeError::BadValue("clipboard total overflow"),
    )?;
    expect_error(b"DKRP\0a\0\0\0\x01\0\x1e", DecodeError::Truncated)?;
    expect_error(b"QINFx", DecodeError::BadValue("trailing bytes"))?;
    expect_error(b"DMDN\0", DecodeError::BadValue("zero mouse button"))?;
    expect_error(b"SECN\0\0\0\x01\xff", DecodeError::BadUtf8)?;
    let mut oversized_string = b"DFTR\x01".to_vec();
    oversized_string.extend_from_slice(&(MAX_STRING_LEN + 1).to_be_bytes());
    expect_error(&oversized_string, DecodeError::BadValue("string length cap"))?;
    for payload in [b"".as_slice(), b"Q", b"QI", b"QIN"] {
        expect_error(payload, DecodeError::Truncated)?;
    }
    for bytes in [&[0u8][..], &[0, 0, 0], &[0, 0, 0, 11, b'S']] {
        let mut d = Decoder::new(MAX_FRAME_LEN);
        d.feed(bytes)?;
        ensure(d.finish() == Err(DecodeError::Truncated), "TCP EOF truncation")?;
    }
    let mut combined = frame(b"ZZZZopaque\0extension");
    combined.extend_from_slice(&frame(b"CALV"));
    let mut d = Decoder::new(MAX_FRAME_LEN);
    prime(&mut d)?;
    d.feed(&combined)?;
    ensure(
        d.next_message()
            == Some(Ok(Message::Unknown {
                code: *b"ZZZZ",
                payload_len: 16,
            })),
        "unknown skip",
    )?;
    ensure(
        d.next_message() == Some(Ok(Message::KeepAlive)),
        "message after unknown",
    )?;
    // Strict semantic text limits, including valid boundary values.
    for (code, cap) in [(b"SECN", 255), (b"LSYN", 64)] {
        let mut payload = code.to_vec();
        payload.extend_from_slice(&((cap + 1) as u32).to_be_bytes());
        payload.extend_from_slice(&vec![b'a'; cap + 1]);
        expect_error(&payload, DecodeError::BadValue("text length cap"))?;
        let valid = if code == b"SECN" {
            Message::SecureInputNotification { name: "a".repeat(cap) }
        } else {
            Message::LanguageSync {
                languages: "a".repeat(cap),
            }
        };
        let mut bytes = Vec::new();
        encode(&valid, &mut bytes)?;
        ensure(decode(&bytes)? == valid, "text boundary")?;
    }
    let mut d = Decoder::new(MAX_FRAME_LEN);
    d.feed(&frame(b"QINF"))?;
    ensure(
        d.next_message() == Some(Err(DecodeError::BadHello)),
        "missing first hello",
    )?;
    let mut d = Decoder::new(MAX_FRAME_LEN);
    prime(&mut d)?;
    d.feed(&frame(b"Barrier\0\x01\0\x06"))?;
    ensure(
        d.next_message() == Some(Err(DecodeError::BadHello)),
        "second hello rejected",
    )?;
    let mut d = Decoder::new(MAX_FRAME_LEN);
    ensure(
        d.feed(&1025u32.to_be_bytes()) == Err(DecodeError::BadHello),
        "hello-back cap at prefix",
    )?;
    let valid = Message::HelloBack {
        product: "Barrier".into(),
        version: v18(),
        name: "n".repeat(1009),
    };
    let mut bytes = Vec::new();
    encode(&valid, &mut bytes)?;
    ensure(decode(&bytes)? == valid, "1024-byte hello-back boundary")?;
    let mut d = Decoder::new(MAX_FRAME_LEN);
    prime(&mut d)?;
    d.feed(&bytes)?;
    ensure(
        d.next_message() == Some(Err(DecodeError::BadHello)),
        "second hello-back rejected",
    )?;
    let mut maximum = b"ZZZZ".to_vec();
    maximum.resize(MAX_FRAME_LEN as usize, 0);
    ensure(
        decode(&frame(&maximum))?
            == Message::Unknown {
                code: *b"ZZZZ",
                payload_len: MAX_FRAME_LEN as usize - 4,
            },
        "inclusive 4 MiB frame cap",
    )?;
    let mut capture = Capture::default();
    capture.accept(ClipboardId(0), 0, &Chunk::Start { total_len: 2 })?;
    capture.accept(ClipboardId(0), 0, &Chunk::Data(vec![1]))?;
    ensure(
        capture.accept(ClipboardId(0), 0, &Chunk::End) == Err(DecodeError::BadValue("clipboard total mismatch")),
        "DCLP aggregate mismatch",
    )?;
    let mut output = vec![0x71];
    let original = output.clone();
    ensure(
        encode(&Message::MouseDown { button: 0 }, &mut output).is_err() && output == original,
        "atomic encode failure",
    )?;
    println!("hostile limits, UTF-8/NUL, hello, DSOP, DCLP size/marks/totals, EOF, Unknown then CALV PASS");
    Ok(())
}

fn clipboard_probe() -> Result {
    let mut bytes = Vec::new();
    marshal_clipboard(
        &[(2, &[0xff, 0, 1]), (0, b"text\0utf8"), (1, b"<b>html</b>")],
        &mut bytes,
    )?;
    let formats = unmarshal_clipboard(&bytes)?;
    ensure(
        formats.iter().map(|(id, _)| *id).collect::<Vec<_>>() == [0, 1, 2],
        "format order",
    )?;
    // Full container exercises the count, ID, length and binary payload path.
    let mut unknown = vec![
        0, 0, 0, 2, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 3, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 2, b'o', b'k',
    ];
    ensure(
        unmarshal_clipboard(&unknown)? == [(0, b"ok".to_vec())],
        "skip unknown format",
    )?;
    unknown.pop();
    ensure(
        unmarshal_clipboard(&unknown) == Err(DecodeError::Truncated),
        "truncated container",
    )?;
    // An 8 MiB container crosses many 32 KiB frames and carries interleaved input.
    let dib = vec![0xa5; 8 * 1024 * 1024];
    bytes.clear();
    marshal_clipboard(&[(2, &dib)], &mut bytes)?;
    let mut capture = Capture::default();
    let mut decoder = Decoder::new(MAX_FRAME_LEN);
    prime(&mut decoder)?;
    let mut wire = Vec::new();
    let mut packets = 0;
    let chunks = std::iter::once(Chunk::Start {
        total_len: bytes.len() as u32,
    })
    .chain(bytes.chunks(CLIPBOARD_CHUNK_LEN).map(|c| Chunk::Data(c.to_vec())))
    .chain(std::iter::once(Chunk::End));
    for chunk in chunks {
        wire.clear();
        encode(&clip(chunk), &mut wire)?;
        encode(&Message::MouseMoveRel { dx: -1, dy: 2 }, &mut wire)?;
        decoder.feed(&wire)?;
        while let Some(message) = decoder.next_message() {
            if let Message::ClipboardData { id, sequence, chunk } = message? {
                packets += 1;
                if let Some(blob) = capture.accept(id, sequence, &chunk)? {
                    ensure(blob == bytes, "large clipboard bytes")?;
                }
            }
        }
    }
    decoder.finish()?;
    println!("clipboard container: sorted formats, unknown skip, truncation; 8 MiB / {packets} DCLP packets with interleaved input PASS");
    Ok(())
}

/// Probe-only observer for real transfers. Production reassembly belongs to
/// session; this checks the peer's advertised total without touching pasteboards.
#[derive(Default)]
struct Capture {
    active: Option<(ClipboardId, u32, usize, Vec<u8>)>,
}
impl Capture {
    fn accept(
        &mut self,
        id: ClipboardId,
        sequence: u32,
        chunk: &Chunk,
    ) -> std::result::Result<Option<Vec<u8>>, DecodeError> {
        match chunk {
            Chunk::Start { total_len } => {
                if *total_len as usize > CLIPBOARD_CAP {
                    return Err(DecodeError::BadValue("clipboard total cap"));
                }
                self.active = Some((id, sequence, *total_len as usize, Vec::new()));
                Ok(None)
            }
            Chunk::Data(data) => {
                let (active_id, active_seq, total, bytes) = self
                    .active
                    .as_mut()
                    .ok_or(DecodeError::BadValue("clipboard data without start"))?;
                if id != *active_id || sequence != *active_seq {
                    return Err(DecodeError::BadValue("interleaved clipboard"));
                }
                if data.len() > total.saturating_sub(bytes.len()) {
                    return Err(DecodeError::BadValue("clipboard total mismatch"));
                }
                bytes.extend_from_slice(data);
                Ok(None)
            }
            Chunk::End => {
                let (active_id, active_seq, total, bytes) = self
                    .active
                    .take()
                    .ok_or(DecodeError::BadValue("clipboard end without start"))?;
                if id != active_id || sequence != active_seq || bytes.len() != total {
                    return Err(DecodeError::BadValue("clipboard total mismatch"));
                }
                Ok(Some(bytes))
            }
        }
    }
}

struct Peer {
    stream: TcpStream,
    decoder: Decoder,
    output: Vec<u8>,
}
impl Peer {
    fn new(stream: TcpStream) -> Result<Self> {
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_millis(100)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        Ok(Self {
            stream,
            decoder: Decoder::new(MAX_FRAME_LEN),
            output: Vec::with_capacity(65536),
        })
    }
    fn send(&mut self, message: &Message) -> Result {
        self.output.clear();
        encode(message, &mut self.output)?;
        self.stream.write_all(&self.output)?;
        println!("TX {}", summary(message));
        Ok(())
    }
    fn receive(&mut self) -> Result<Option<Message>> {
        if let Some(message) = self.decoder.next_message() {
            return Ok(Some(message?));
        }
        let mut bytes = [0u8; 8192];
        match self.stream.read(&mut bytes) {
            Ok(0) => {
                self.decoder.finish()?;
                Err("peer closed TCP".into())
            }
            Ok(n) => {
                self.decoder.feed(bytes.get(..n).ok_or("read count")?)?;
                self.decoder.next_message().transpose().map_err(Into::into)
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

fn allowed_port(port: u16) -> bool {
    (24910..=24919).contains(&port) || (24960..=24969).contains(&port)
}

fn client(args: &[String]) -> Result {
    let address: SocketAddrV4 = arg(args, "--server").ok_or("missing --server")?.parse()?;
    ensure(
        *address.ip() == Ipv4Addr::LOCALHOST && allowed_port(address.port()),
        "only assigned IPv4 loopback ports are allowed",
    )?;
    let name = arg(args, "--name").unwrap_or("openergy-mac");
    let duration: u64 = arg(args, "--duration-secs").unwrap_or("18").parse()?;
    ensure((1..=120).contains(&duration), "duration must be 1-120 seconds")?;
    let mut peer = Peer::new(TcpStream::connect_timeout(&address.into(), Duration::from_secs(2))?)?;
    let deadline = Instant::now() + Duration::from_secs(duration);
    let mut counts = BTreeMap::<&str, usize>::new();
    let mut capture = Capture::default();
    let mut modern = false;
    let mut nonempty_text = 0;
    while Instant::now() < deadline {
        let Some(message) = peer.receive()? else {
            continue;
        };
        println!("RX {}", summary(&message));
        *counts.entry(kind(&message)).or_default() += 1;
        match &message {
            Message::Enter { x, y, .. } | Message::MouseMoveAbs { x, y } => ensure(
                (0..1920).contains(x) && (0..1080).contains(y),
                "out-of-screen lab coordinates",
            )?,
            Message::MouseDown { button } | Message::MouseUp { button } => {
                ensure((1..=5).contains(button), "unexpected lab mouse button")?
            }
            Message::MouseWheel { dx, dy } => ensure(
                i32::from(*dx).abs() <= 12000 && i32::from(*dy).abs() <= 12000,
                "implausible lab wheel delta",
            )?,
            _ => {}
        }
        match message {
            Message::Hello { product, version } => {
                ensure(
                    version.major == 1 && (6..=8).contains(&version.minor),
                    "unsupported lab server version",
                )?;
                modern = version.minor >= 8;
                peer.send(&Message::HelloBack {
                    product,
                    version,
                    name: name.to_owned(),
                })?;
            }
            Message::QueryInfo => peer.send(&info())?,
            Message::Enter { sequence, .. } => {
                peer.send(&Message::ClipboardGrab {
                    id: ClipboardId::CLIPBOARD,
                    sequence,
                })?;
            }
            Message::KeepAlive => peer.send(&Message::KeepAlive)?,
            Message::ClipboardData { id, sequence, chunk } => {
                if let Some(bytes) = capture.accept(id, sequence, &chunk)? {
                    let formats = unmarshal_clipboard(&bytes)?;
                    nonempty_text += usize::from(formats.iter().any(|(id, bytes)| *id == 0 && !bytes.is_empty()));
                    println!(
                        "clipboard complete id={} bytes={} formats={}",
                        id.0,
                        bytes.len(),
                        formats.len()
                    );
                }
            }
            Message::ErrorIncompatible { .. }
            | Message::ErrorBusy
            | Message::ErrorUnknown
            | Message::ErrorBad
            | Message::Bye => return Err("lab server rejected/closed connection".into()),
            _ => {}
        }
    }
    for required in [
        "Hello", "QINF", "CIAK", "CROP", "DSOP", "CALV", "CINN", "DMMV", "DMDN", "DMUP", "DKUP", "DMWM", "CCLP",
        "DCLP", "COUT",
    ] {
        ensure(
            counts.contains_key(required),
            &format!("missing required server message {required}"),
        )?;
    }
    ensure(
        counts.contains_key("DKDL") || counts.contains_key("DKDN"),
        "missing key down",
    )?;
    if modern {
        ensure(counts.contains_key("LSYN"), "missing setup LSYN")?;
    }
    ensure(nonempty_text > 0, "no complete nonempty text clipboard")?;
    println!("client PASS counts={counts:?}, nonempty_text_transfers={nonempty_text}");
    Ok(())
}

fn server(args: &[String]) -> Result {
    let port: u16 = arg(args, "--port").ok_or("missing --port")?.parse()?;
    ensure(
        (24960..=24969).contains(&port),
        "server requires assigned local port 24960-24969",
    )?;
    let product = arg(args, "--product").unwrap_or("Synergy");
    let minor: u16 = arg(args, "--minor").unwrap_or("8").parse()?;
    let hold_ms: u64 = arg(args, "--hold-ms").unwrap_or("8000").parse()?;
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    listener.set_nonblocking(true)?;
    println!("listening 127.0.0.1:{port}; isolated lab input only");
    let deadline = Instant::now() + Duration::from_secs(20);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(e) => return Err(e.into()),
        }
    };
    let mut peer = Peer::new(stream)?;
    peer.send(&Message::Hello {
        product: product.to_owned(),
        version: ProtocolVersion { major: 1, minor },
    })?;
    let deadline = Instant::now() + Duration::from_secs(12);
    let mut version = None;
    let mut configured = None;
    let mut sent = None;
    let mut heartbeat = Instant::now();
    let mut echoed = false;
    while Instant::now() < deadline {
        if let Some(message) = peer.receive()? {
            println!("RX {}", summary(&message));
            match message {
                Message::HelloBack {
                    version: remote,
                    product: echoed_product,
                    name,
                } => {
                    ensure(remote.major == 1 && (6..=8).contains(&remote.minor), "client version")?;
                    ensure(
                        echoed_product == product && name == "lab",
                        "client hello identity/spelling",
                    )?;
                    version = Some(remote);
                    peer.send(&Message::QueryInfo)?;
                    if remote.minor >= 8 {
                        peer.send(&Message::LanguageSync { languages: "en".into() })?;
                    }
                }
                Message::ScreenInfo { w, h, .. } => {
                    ensure(w == 1920 && h == 1080, "unexpected Xvfb geometry")?;
                    peer.send(&Message::InfoAck)?;
                    peer.send(&Message::ResetOptions)?;
                    peer.send(&opts())?;
                    peer.send(&Message::KeepAlive)?;
                    configured = Some(Instant::now());
                }
                Message::KeepAlive => echoed = true,
                Message::ErrorBad | Message::ErrorBusy | Message::ErrorUnknown | Message::Bye => {
                    return Err("client rejected connection".into())
                }
                _ => {}
            }
        }
        if configured.is_some_and(|t| t.elapsed() >= Duration::from_millis(700)) && sent.is_none() {
            send_input(&mut peer, version.ok_or("missing hello-back")?.minor >= 8)?;
            sent = Some(Instant::now());
            println!("input sent; inspect lab pointer=321,234 and container clipboard during hold");
        }
        if heartbeat.elapsed() >= Duration::from_secs(3) {
            peer.send(&Message::KeepAlive)?;
            heartbeat = Instant::now();
        }
        if sent.is_some_and(|t| t.elapsed() >= Duration::from_millis(hold_ms)) {
            peer.send(&Message::Leave)?;
            peer.send(&Message::KeepAlive)?;
            ensure(echoed, "client did not echo keepalive")?;
            println!("server PASS hello-back, DINF=1920x1080, CALV echo; input + clipboard sent, COUT sent");
            return Ok(());
        }
    }
    Err("server handshake/input deadline".into())
}

fn send_input(peer: &mut Peer, modern: bool) -> Result {
    peer.send(&Message::Enter {
        x: 100,
        y: 100,
        sequence: 1,
        modifiers: 0,
    })?;
    peer.send(&Message::MouseMoveAbs { x: 321, y: 234 })?;
    peer.send(&Message::MouseDown { button: 1 })?;
    peer.send(&Message::MouseUp { button: 1 })?;
    peer.send(&Message::KeyDown {
        key: u16::from(b'q'),
        modifiers: 0,
        button: 31,
        lang: modern.then(|| "en".to_owned()),
    })?;
    peer.send(&Message::KeyUp {
        key: u16::from(b'q'),
        modifiers: 0,
        button: 31,
    })?;
    peer.send(&Message::MouseWheel { dx: 0, dy: 120 })?;
    peer.send(&Message::ClipboardGrab {
        id: ClipboardId::CLIPBOARD,
        sequence: 0,
    })?;
    let mut blob = Vec::new();
    marshal_clipboard(&[(0, b"Openergy wire probe clipboard")], &mut blob)?;
    peer.send(&clip(Chunk::Start {
        total_len: blob.len() as u32,
    }))?;
    for chunk in blob.chunks(CLIPBOARD_CHUNK_LEN) {
        peer.send(&clip(Chunk::Data(chunk.to_vec())))?;
    }
    peer.send(&clip(Chunk::End))?;
    Ok(())
}

fn kind(message: &Message) -> &'static str {
    match message {
        Message::Hello { .. } => "Hello",
        Message::HelloBack { .. } => "HelloBack",
        Message::QueryInfo => "QINF",
        Message::Noop => "CNOP",
        Message::Bye => "CBYE",
        Message::InfoAck => "CIAK",
        Message::KeepAlive => "CALV",
        Message::ResetOptions => "CROP",
        Message::Leave => "COUT",
        Message::Enter { .. } => "CINN",
        Message::ScreenSaver { .. } => "CSEC",
        Message::ClipboardGrab { .. } => "CCLP",
        Message::SetOptions { .. } => "DSOP",
        Message::LanguageSync { .. } => "LSYN",
        Message::SecureInputNotification { .. } => "SECN",
        Message::KeyDown { lang: Some(_), .. } => "DKDL",
        Message::KeyDown { .. } => "DKDN",
        Message::KeyRepeat { .. } => "DKRP",
        Message::KeyUp { .. } => "DKUP",
        Message::MouseDown { .. } => "DMDN",
        Message::MouseUp { .. } => "DMUP",
        Message::MouseMoveAbs { .. } => "DMMV",
        Message::MouseMoveRel { .. } => "DMRM",
        Message::MouseWheel { .. } => "DMWM",
        Message::ScreenInfo { .. } => "DINF",
        Message::ClipboardData { .. } => "DCLP",
        Message::FileTransfer { .. } => "DFTR",
        Message::DragInfo { .. } => "DDRG",
        Message::ErrorIncompatible { .. } => "EICV",
        Message::ErrorBusy => "EBSY",
        Message::ErrorUnknown => "EUNK",
        Message::ErrorBad => "EBAD",
        Message::Unknown { .. } => "Unknown",
    }
}

fn summary(message: &Message) -> String {
    let detail = match message {
        Message::Hello { product, version } | Message::HelloBack { product, version, .. } => {
            format!(" product={product} version={}.{}", version.major, version.minor)
        }
        Message::Enter {
            x,
            y,
            sequence,
            modifiers,
        } => format!(" x={x} y={y} sequence={sequence} modifiers={modifiers}"),
        Message::MouseMoveAbs { x, y } => format!(" x={x} y={y}"),
        Message::MouseMoveRel { dx, dy } | Message::MouseWheel { dx, dy } => format!(" dx={dx} dy={dy}"),
        Message::MouseDown { button } | Message::MouseUp { button } => format!(" button={button}"),
        Message::ScreenInfo {
            x,
            y,
            w,
            h,
            warp,
            mx,
            my,
        } => format!(" x={x} y={y} w={w} h={h} warp={warp} mx={mx} my={my}"),
        Message::SetOptions { options } => format!(" pairs={options:?}"),
        Message::ClipboardGrab { id, sequence } => format!(" id={} sequence={sequence}", id.0),
        Message::ClipboardData { id, sequence, chunk } => format!(
            " id={} sequence={sequence} {}",
            id.0,
            match chunk {
                Chunk::Start { total_len } => format!("Start total={total_len}"),
                Chunk::Data(bytes) => format!("Data bytes={}", bytes.len()),
                Chunk::End => "End".into(),
            }
        ),
        Message::LanguageSync { languages } => format!(" bytes={}", languages.len()),
        Message::SecureInputNotification { name } => format!(" bytes={}", name.len()),
        Message::FileTransfer { mark, payload } => format!(" mark={mark} bytes={}", payload.len()),
        Message::DragInfo { count, payload } => format!(" count={count} bytes={}", payload.len()),
        Message::Unknown { code, payload_len } => format!(" code={code:?} bytes={payload_len}"),
        // Never log logical keys, physical key buttons, or clipboard contents.
        _ => String::new(),
    };
    format!("{}{detail}", kind(message))
}
