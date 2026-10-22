use crate::{
    Chunk, ClipboardId, DecodeError, EncodeError, Message, ProtocolVersion, MAX_FRAME_LEN, MAX_HELLO_BACK_LEN,
    MAX_LANGUAGE_LEN, MAX_SECURE_INPUT_LEN, MAX_STRING_LEN, MAX_VECTOR_LEN, PRODUCT_BARRIER, PRODUCT_SYNERGY,
};
use std::collections::VecDeque;

/// A streaming decoder. Storage grows with received bytes, never a claimed size.
/// Framing errors poison the decoder; message errors are returned by `next_message`.
pub struct Decoder {
    max_frame_len: u32,
    prefix: [u8; 4],
    prefix_len: usize,
    frame_len: Option<usize>,
    frame: Vec<u8>,
    ready: VecDeque<Result<Message, DecodeError>>,
    failed: Option<DecodeError>,
    first_frame: bool,
    legacy_key_repeat: bool,
}

impl Decoder {
    /// A caller can tighten, but cannot raise, the protocol's 4 MiB cap.
    pub fn new(max_frame_len: u32) -> Self {
        Self {
            max_frame_len: max_frame_len.min(MAX_FRAME_LEN),
            prefix: [0; 4],
            prefix_len: 0,
            frame_len: None,
            frame: Vec::new(),
            ready: VecDeque::new(),
            failed: None,
            first_frame: true,
            legacy_key_repeat: false,
        }
    }

    pub fn max_frame_len(&self) -> u32 {
        self.max_frame_len
    }

    pub fn feed(&mut self, mut bytes: &[u8]) -> Result<(), DecodeError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        let result = self.feed_inner(&mut bytes);
        if let Err(error) = &result {
            self.failed = Some(error.clone());
            self.frame.clear();
        }
        result
    }

    fn feed_inner(&mut self, bytes: &mut &[u8]) -> Result<(), DecodeError> {
        while !bytes.is_empty() {
            if self.frame_len.is_none() {
                let n = (4 - self.prefix_len).min(bytes.len());
                let part = bytes.get(..n).ok_or(DecodeError::Truncated)?;
                self.prefix
                    .get_mut(self.prefix_len..self.prefix_len + n)
                    .ok_or(DecodeError::Truncated)?
                    .copy_from_slice(part);
                *bytes = bytes.get(n..).ok_or(DecodeError::Truncated)?;
                self.prefix_len += n;
                if self.prefix_len != 4 {
                    continue;
                }
                let len = u32::from_be_bytes(self.prefix);
                if len > self.max_frame_len {
                    return Err(DecodeError::OversizedFrame(len));
                }
                // Reject impossible lengths from the prefix alone, without
                // waiting for an attacker to provide a body that cannot be valid.
                if self.first_frame {
                    // Hello is 11 B; HelloBack adds at least a 4 B name length.
                    if len != 11 && !(15..=MAX_HELLO_BACK_LEN as u32).contains(&len) {
                        return Err(DecodeError::BadHello);
                    }
                } else if len < 4 {
                    return Err(DecodeError::BadValue("short frame"));
                }
                self.frame_len = Some(len as usize);
                self.prefix_len = 0;
            }
            let len = self.frame_len.ok_or(DecodeError::Truncated)?;
            let n = (len - self.frame.len()).min(bytes.len());
            self.frame
                .try_reserve(n)
                .map_err(|_| DecodeError::BadValue("allocation failed"))?;
            self.frame
                .extend_from_slice(bytes.get(..n).ok_or(DecodeError::Truncated)?);
            *bytes = bytes.get(n..).ok_or(DecodeError::Truncated)?;
            if self.frame.len() == len {
                self.ready
                    .try_reserve(1)
                    .map_err(|_| DecodeError::BadValue("allocation failed"))?;
                let message = decode_frame(&self.frame, self.first_frame, self.legacy_key_repeat);
                if let Ok(Message::Hello { version, .. } | Message::HelloBack { version, .. }) = &message {
                    self.legacy_key_repeat = version.major == 1 && (1..8).contains(&version.minor);
                }
                self.ready.push_back(message);
                self.first_frame = false;
                self.frame.clear();
                self.frame_len = None;
            }
        }
        Ok(())
    }

    pub fn next_message(&mut self) -> Option<Result<Message, DecodeError>> {
        self.ready.pop_front()
    }

    /// Call on TCP EOF: an unfinished prefix or payload is a truncated stream.
    /// Completed messages remain available through `next_message`.
    pub fn finish(&mut self) -> Result<(), DecodeError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if self.prefix_len != 0 || self.frame_len.is_some() {
            self.failed = Some(DecodeError::Truncated);
            return Err(DecodeError::Truncated);
        }
        Ok(())
    }
}

pub(crate) struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.rest.len()
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let part = self.rest.get(..n).ok_or(DecodeError::Truncated)?;
        self.rest = self.rest.get(n..).ok_or(DecodeError::Truncated)?;
        Ok(part)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        self.take(N)?.try_into().map_err(|_| DecodeError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        self.take(1)?.first().copied().ok_or(DecodeError::Truncated)
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn i16(&mut self) -> Result<i16, DecodeError> {
        Ok(i16::from_be_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn blob(&mut self) -> Result<&'a [u8], DecodeError> {
        let n = self.u32()?;
        if n > MAX_STRING_LEN {
            return Err(DecodeError::BadValue("string length cap"));
        }
        self.take(n as usize)
    }

    fn text(&mut self, cap: usize, no_nul: bool) -> Result<String, DecodeError> {
        let n = self.u32()?;
        if n > MAX_STRING_LEN {
            return Err(DecodeError::BadValue("string length cap"));
        }
        if n as usize > cap {
            return Err(DecodeError::BadValue("text length cap"));
        }
        let bytes = self.take(n as usize)?;
        let text = std::str::from_utf8(bytes).map_err(|_| DecodeError::BadUtf8)?;
        if no_nul && bytes.contains(&0) {
            return Err(DecodeError::BadValue("NUL in text"));
        }
        Ok(text.to_owned())
    }

    pub(crate) fn end(self) -> Result<(), DecodeError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::BadValue("trailing bytes"))
        }
    }
}

fn product_ok(bytes: &[u8]) -> bool {
    bytes.eq_ignore_ascii_case(PRODUCT_SYNERGY.as_bytes()) || bytes.eq_ignore_ascii_case(PRODUCT_BARRIER.as_bytes())
}

fn decode_frame(frame: &[u8], first_frame: bool, legacy_key_repeat: bool) -> Result<Message, DecodeError> {
    let mut r = Reader::new(frame);
    if frame.get(..7).is_some_and(product_ok) {
        if !first_frame {
            return Err(DecodeError::BadHello);
        }
        let product = std::str::from_utf8(r.take(7)?)
            .map_err(|_| DecodeError::BadHello)?
            .to_owned();
        let version = ProtocolVersion {
            major: r.u16()?,
            minor: r.u16()?,
        };
        if r.remaining() == 0 {
            return Ok(Message::Hello { product, version });
        }
        if frame.len() > MAX_HELLO_BACK_LEN {
            return Err(DecodeError::BadHello);
        }
        let name = r.text(MAX_HELLO_BACK_LEN - 15, true)?;
        r.end()?;
        return Ok(Message::HelloBack { product, version, name });
    }
    if first_frame {
        return Err(DecodeError::BadHello);
    }
    let code = r.array::<4>()?;
    let message = match &code {
        b"QINF" => Message::QueryInfo,
        b"CNOP" => Message::Noop,
        b"CBYE" => Message::Bye,
        b"COUT" => Message::Leave,
        b"CROP" => Message::ResetOptions,
        b"CIAK" => Message::InfoAck,
        b"CALV" => Message::KeepAlive,
        b"CINN" => Message::Enter {
            x: r.i16()?,
            y: r.i16()?,
            sequence: r.u32()?,
            modifiers: r.u16()?,
        },
        b"CCLP" => Message::ClipboardGrab {
            id: ClipboardId(r.u8()?),
            sequence: r.u32()?,
        },
        b"CSEC" => Message::ScreenSaver { on: r.u8()? != 0 },
        b"DKDN" | b"DKDL" => Message::KeyDown {
            key: r.u16()?,
            modifiers: r.u16()?,
            button: r.u16()?,
            lang: if &code == b"DKDL" {
                Some(r.text(MAX_LANGUAGE_LEN, true)?)
            } else {
                None
            },
        },
        b"DKRP" => Message::KeyRepeat {
            key: r.u16()?,
            modifiers: r.u16()?,
            count: r.u16()?,
            button: r.u16()?,
            // Barrier 1.6 sends no language suffix. Some modern Synergy
            // builds retain the suffix even in their older proxy writers.
            // Only a pre-1.8 hello permits the completely absent field;
            // partial lengths and modern 1.8 packets remain strict.
            lang: if legacy_key_repeat && r.remaining() == 0 {
                String::new()
            } else {
                r.text(MAX_LANGUAGE_LEN, true)?
            },
        },
        b"DKUP" => Message::KeyUp {
            key: r.u16()?,
            modifiers: r.u16()?,
            button: r.u16()?,
        },
        b"DMDN" | b"DMUP" => {
            let button = r.u8()?;
            if button == 0 {
                return Err(DecodeError::BadValue("zero mouse button"));
            }
            if &code == b"DMDN" {
                Message::MouseDown { button }
            } else {
                Message::MouseUp { button }
            }
        }
        b"DMMV" => Message::MouseMoveAbs {
            x: r.i16()?,
            y: r.i16()?,
        },
        b"DMRM" => Message::MouseMoveRel {
            dx: r.i16()?,
            dy: r.i16()?,
        },
        b"DMWM" => Message::MouseWheel {
            dx: r.i16()?,
            dy: r.i16()?,
        },
        b"DINF" => {
            let message = Message::ScreenInfo {
                x: r.i16()?,
                y: r.i16()?,
                w: r.i16()?,
                h: r.i16()?,
                warp: r.i16()?,
                mx: r.i16()?,
                my: r.i16()?,
            };
            if let Message::ScreenInfo { w, h, .. } = &message {
                if *w <= 0 || *h <= 0 {
                    return Err(DecodeError::BadValue("screen size"));
                }
            }
            message
        }
        b"DSOP" => {
            let n = r.u32()?;
            if n > MAX_VECTOR_LEN {
                return Err(DecodeError::BadValue("vector length cap"));
            }
            if n % 2 != 0 {
                return Err(DecodeError::BadValue("odd option count"));
            }
            if n as usize > r.remaining() / 4 {
                return Err(DecodeError::Truncated);
            }
            let mut options = Vec::new();
            options
                .try_reserve_exact(n as usize / 2)
                .map_err(|_| DecodeError::BadValue("allocation failed"))?;
            for _ in 0..n / 2 {
                options.push((r.u32()?, r.u32()? as i32));
            }
            Message::SetOptions { options }
        }
        b"DCLP" => {
            let id = ClipboardId(r.u8()?);
            let sequence = r.u32()?;
            let mark = r.u8()?;
            let bytes = r.blob()?;
            let chunk = match mark {
                1 => {
                    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
                        return Err(DecodeError::BadValue("clipboard total is not decimal"));
                    }
                    let total_len = bytes
                        .iter()
                        .try_fold(0u32, |n, b| n.checked_mul(10)?.checked_add(u32::from(b - b'0')))
                        .ok_or(DecodeError::BadValue("clipboard total overflow"))?;
                    Chunk::Start { total_len }
                }
                2 => Chunk::Data(bytes.to_vec()),
                3 if bytes.is_empty() => Chunk::End,
                3 => return Err(DecodeError::BadValue("nonempty clipboard end")),
                _ => return Err(DecodeError::BadValue("clipboard chunk mark")),
            };
            Message::ClipboardData { id, sequence, chunk }
        }
        b"DFTR" => Message::FileTransfer {
            mark: r.u8()?,
            payload: r.blob()?.to_vec(),
        },
        b"DDRG" => Message::DragInfo {
            count: r.i16()?,
            payload: r.text(MAX_STRING_LEN as usize, false)?,
        },
        b"LSYN" => Message::LanguageSync {
            languages: r.text(MAX_LANGUAGE_LEN, true)?,
        },
        b"SECN" => Message::SecureInputNotification {
            name: r.text(MAX_SECURE_INPUT_LEN, true)?,
        },
        b"EICV" => Message::ErrorIncompatible {
            major: r.u16()?,
            minor: r.u16()?,
        },
        b"EBSY" => Message::ErrorBusy,
        b"EUNK" => Message::ErrorUnknown,
        b"EBAD" => Message::ErrorBad,
        _ => {
            return Ok(Message::Unknown {
                code,
                payload_len: r.remaining(),
            })
        }
    };
    r.end()?;
    Ok(message)
}

fn text_size(text: &str, cap: usize, no_nul: bool) -> Result<usize, EncodeError> {
    if text.len() > cap || text.len() > MAX_STRING_LEN as usize {
        return Err(EncodeError::BadValue("text length cap"));
    }
    if no_nul && text.as_bytes().contains(&0) {
        return Err(EncodeError::BadValue("NUL in text"));
    }
    Ok(4 + text.len())
}

fn blob_size(bytes: &[u8]) -> Result<usize, EncodeError> {
    if bytes.len() > MAX_STRING_LEN as usize {
        return Err(EncodeError::BadValue("string length cap"));
    }
    Ok(4 + bytes.len())
}

/// Encode atomically: errors leave `out` unchanged. No temporary allocation is
/// made for input messages, including a DCLP start's decimal size.
pub fn encode(msg: &Message, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    let len = match msg {
        Message::Hello { product, .. } | Message::HelloBack { product, .. } => {
            if !product_ok(product.as_bytes()) {
                return Err(EncodeError::BadValue("hello product"));
            }
            if let Message::HelloBack { name, .. } = msg {
                11 + text_size(name, MAX_HELLO_BACK_LEN - 15, true)?
            } else {
                11
            }
        }
        Message::QueryInfo
        | Message::Noop
        | Message::Bye
        | Message::InfoAck
        | Message::KeepAlive
        | Message::ResetOptions
        | Message::Leave
        | Message::ErrorBusy
        | Message::ErrorUnknown
        | Message::ErrorBad => 4,
        Message::Enter { .. } => 14,
        Message::ClipboardGrab { .. } => 9,
        Message::ScreenSaver { .. } => 5,
        Message::SetOptions { options } => {
            if options.len() > MAX_VECTOR_LEN as usize / 2 {
                return Err(EncodeError::BadValue("vector length cap"));
            }
            8 + 8 * options.len()
        }
        Message::LanguageSync { languages } => {
            if languages.len() % 2 != 0 {
                return Err(EncodeError::BadValue("odd language bytes"));
            }
            4 + text_size(languages, MAX_LANGUAGE_LEN, true)?
        }
        Message::SecureInputNotification { name } => 4 + text_size(name, MAX_SECURE_INPUT_LEN, true)?,
        Message::KeyDown { lang, .. } => {
            10 + match lang {
                Some(s) => text_size(s, MAX_LANGUAGE_LEN, true)?,
                None => 0,
            }
        }
        Message::KeyRepeat { lang, .. } => 12 + text_size(lang, MAX_LANGUAGE_LEN, true)?,
        Message::KeyUp { .. } => 10,
        Message::MouseDown { button } | Message::MouseUp { button } => {
            if *button == 0 {
                return Err(EncodeError::BadValue("zero mouse button"));
            }
            5
        }
        Message::MouseMoveAbs { .. }
        | Message::MouseMoveRel { .. }
        | Message::MouseWheel { .. }
        | Message::ErrorIncompatible { .. } => 8,
        Message::ScreenInfo { w, h, .. } => {
            if *w <= 0 || *h <= 0 {
                return Err(EncodeError::BadValue("screen size"));
            }
            18
        }
        Message::ClipboardData { chunk, .. } => {
            14 + match chunk {
                Chunk::Start { total_len } => decimal_len(*total_len),
                Chunk::Data(bytes) => blob_size(bytes)? - 4,
                Chunk::End => 0,
            }
        }
        Message::FileTransfer { payload, .. } => 5 + blob_size(payload)?,
        Message::DragInfo { payload, .. } => 6 + text_size(payload, MAX_STRING_LEN as usize, false)?,
        Message::Unknown { .. } => return Err(EncodeError::BadValue("unknown frame has no payload bytes")),
    };
    if len > MAX_FRAME_LEN as usize {
        return Err(EncodeError::BadValue("frame length cap"));
    }
    out.try_reserve(4 + len)
        .map_err(|_| EncodeError::BadValue("allocation failed"))?;
    out.extend_from_slice(&(len as u32).to_be_bytes());
    let mut w = Writer(out);
    match msg {
        Message::Hello { product, version } | Message::HelloBack { product, version, .. } => {
            w.0.extend_from_slice(product.as_bytes());
            w.u16(version.major);
            w.u16(version.minor);
            if let Message::HelloBack { name, .. } = msg {
                w.blob(name.as_bytes());
            }
        }
        Message::QueryInfo => w.code(b"QINF"),
        Message::Noop => w.code(b"CNOP"),
        Message::Bye => w.code(b"CBYE"),
        Message::InfoAck => w.code(b"CIAK"),
        Message::KeepAlive => w.code(b"CALV"),
        Message::ResetOptions => w.code(b"CROP"),
        Message::Leave => w.code(b"COUT"),
        Message::Enter {
            x,
            y,
            sequence,
            modifiers,
        } => {
            w.code(b"CINN");
            w.i16(*x);
            w.i16(*y);
            w.u32(*sequence);
            w.u16(*modifiers);
        }
        Message::ClipboardGrab { id, sequence } => {
            w.code(b"CCLP");
            w.u8(id.0);
            w.u32(*sequence);
        }
        Message::ScreenSaver { on } => {
            w.code(b"CSEC");
            w.u8(u8::from(*on));
        }
        Message::SetOptions { options } => {
            w.code(b"DSOP");
            w.u32(options.len() as u32 * 2);
            for (id, value) in options {
                w.u32(*id);
                w.u32(*value as u32);
            }
        }
        Message::LanguageSync { languages } => {
            w.code(b"LSYN");
            w.blob(languages.as_bytes());
        }
        Message::SecureInputNotification { name } => {
            w.code(b"SECN");
            w.blob(name.as_bytes());
        }
        Message::KeyDown {
            key,
            modifiers,
            button,
            lang,
        } => {
            w.code(if lang.is_some() { b"DKDL" } else { b"DKDN" });
            w.u16(*key);
            w.u16(*modifiers);
            w.u16(*button);
            if let Some(lang) = lang {
                w.blob(lang.as_bytes());
            }
        }
        Message::KeyRepeat {
            key,
            modifiers,
            count,
            button,
            lang,
        } => {
            w.code(b"DKRP");
            w.u16(*key);
            w.u16(*modifiers);
            w.u16(*count);
            w.u16(*button);
            w.blob(lang.as_bytes());
        }
        Message::KeyUp { key, modifiers, button } => {
            w.code(b"DKUP");
            w.u16(*key);
            w.u16(*modifiers);
            w.u16(*button);
        }
        Message::MouseDown { button } => {
            w.code(b"DMDN");
            w.u8(*button);
        }
        Message::MouseUp { button } => {
            w.code(b"DMUP");
            w.u8(*button);
        }
        Message::MouseMoveAbs { x, y } => {
            w.code(b"DMMV");
            w.i16(*x);
            w.i16(*y);
        }
        Message::MouseMoveRel { dx, dy } => {
            w.code(b"DMRM");
            w.i16(*dx);
            w.i16(*dy);
        }
        Message::MouseWheel { dx, dy } => {
            w.code(b"DMWM");
            w.i16(*dx);
            w.i16(*dy);
        }
        Message::ScreenInfo {
            x,
            y,
            w: width,
            h,
            warp,
            mx,
            my,
        } => {
            w.code(b"DINF");
            for n in [x, y, width, h, warp, mx, my] {
                w.i16(*n);
            }
        }
        Message::ClipboardData { id, sequence, chunk } => {
            w.code(b"DCLP");
            w.u8(id.0);
            w.u32(*sequence);
            match chunk {
                Chunk::Start { total_len } => {
                    w.u8(1);
                    w.u32(decimal_len(*total_len) as u32);
                    w.decimal(*total_len);
                }
                Chunk::Data(bytes) => {
                    w.u8(2);
                    w.blob(bytes);
                }
                Chunk::End => {
                    w.u8(3);
                    w.u32(0);
                }
            }
        }
        Message::FileTransfer { mark, payload } => {
            w.code(b"DFTR");
            w.u8(*mark);
            w.blob(payload);
        }
        Message::DragInfo { count, payload } => {
            w.code(b"DDRG");
            w.i16(*count);
            w.blob(payload.as_bytes());
        }
        Message::ErrorIncompatible { major, minor } => {
            w.code(b"EICV");
            w.u16(*major);
            w.u16(*minor);
        }
        Message::ErrorBusy => w.code(b"EBSY"),
        Message::ErrorUnknown => w.code(b"EUNK"),
        Message::ErrorBad => w.code(b"EBAD"),
        Message::Unknown { .. } => {} // Rejected by validation before touching `out`.
    }
    Ok(())
}

fn decimal_len(n: u32) -> usize {
    if n == 0 {
        1
    } else {
        n.ilog10() as usize + 1
    }
}

struct Writer<'a>(&'a mut Vec<u8>);
impl Writer<'_> {
    fn code(&mut self, code: &[u8; 4]) {
        self.0.extend_from_slice(code);
    }
    fn u8(&mut self, n: u8) {
        self.0.push(n);
    }
    fn u16(&mut self, n: u16) {
        self.0.extend_from_slice(&n.to_be_bytes());
    }
    fn i16(&mut self, n: i16) {
        self.0.extend_from_slice(&n.to_be_bytes());
    }
    fn u32(&mut self, n: u32) {
        self.0.extend_from_slice(&n.to_be_bytes());
    }
    fn blob(&mut self, bytes: &[u8]) {
        self.u32(bytes.len() as u32);
        self.0.extend_from_slice(bytes);
    }
    fn decimal(&mut self, n: u32) {
        let mut place = 10u32.pow(decimal_len(n) as u32 - 1);
        loop {
            self.u8(b'0' + ((n / place) % 10) as u8);
            if place == 1 {
                break;
            }
            place /= 10;
        }
    }
}
