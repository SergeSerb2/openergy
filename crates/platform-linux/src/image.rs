//! Small original PNG/DIB bridge. PNG uses RFC1950/1951 and PNG filters.
//! No image library or process is needed at runtime. Interlaced PNG is rejected.
use crate::native::error;
use openergy_platform::PlatformError;
type Result<T> = std::result::Result<T, PlatformError>;
fn le16(b: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(at..at + 2)
            .ok_or_else(|| error("Truncated DIB"))?
            .try_into()
            .map_err(error)?,
    ))
}
fn le32(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or_else(|| error("Truncated DIB"))?
            .try_into()
            .map_err(error)?,
    ))
}
fn be32(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_be_bytes(
        b.get(at..at + 4)
            .ok_or_else(|| error("Truncated PNG"))?
            .try_into()
            .map_err(error)?,
    ))
}
fn dib_offset(d: &[u8]) -> Result<usize> {
    let header = le32(d, 0)? as usize;
    if ![40, 52, 56, 108, 124].contains(&header) || d.len() < header {
        return Err(error("Unsupported DIB header"));
    }
    let bpp = le16(d, 14)?;
    let colors = le32(d, 32)? as usize;
    let palette = if colors != 0 {
        colors
    } else if bpp <= 8 {
        1usize << bpp
    } else {
        0
    };
    let masks = if header == 40 {
        match le32(d, 16)? {
            3 => 12,
            6 => 16,
            _ => 0,
        }
    } else {
        0
    };
    let offset = header
        .checked_add(masks)
        .and_then(|n| n.checked_add(palette.checked_mul(4)?))
        .ok_or_else(|| error("DIB size overflow"))?;
    if offset > d.len() {
        return Err(error("Truncated DIB palette"));
    }
    Ok(offset)
}
pub fn dib_to_bmp(d: &[u8]) -> Result<Vec<u8>> {
    let offset = dib_offset(d)?;
    let size = u32::try_from(d.len() + 14).map_err(error)?;
    let mut bmp = Vec::with_capacity(d.len() + 14);
    bmp.extend_from_slice(b"BM");
    bmp.extend(size.to_le_bytes());
    bmp.extend([0; 4]);
    bmp.extend(((offset + 14) as u32).to_le_bytes());
    bmp.extend(d);
    Ok(bmp)
}
pub fn bmp_to_dib(bmp: &[u8]) -> Result<Vec<u8>> {
    if bmp.get(..2) != Some(b"BM") {
        return Err(error("Invalid BMP signature"));
    }
    let pixels = le32(bmp, 10)? as usize;
    let data = bmp.get(14..).ok_or_else(|| error("Truncated BMP"))?;
    let offset = dib_offset(data)?;
    if pixels < 14 + offset || pixels > bmp.len() {
        return Err(error("Invalid BMP pixel offset"));
    }
    let mut dib = data[..offset].to_vec();
    dib.extend_from_slice(&bmp[pixels..]);
    Ok(dib)
}
fn rgba_dib(width: usize, height: usize, rgba: &[u8]) -> Result<Vec<u8>> {
    let count = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| error("Image size overflow"))?;
    if count != rgba.len() || count > 256 * 1024 * 1024 || width > i32::MAX as usize || height > i32::MAX as usize {
        return Err(error("Invalid image dimensions"));
    }
    // BITMAPV4HEADER with explicit RGBA masks preserves alpha across Windows/X11.
    let mut dib = vec![0; 108];
    dib[0..4].copy_from_slice(&108u32.to_le_bytes());
    dib[4..8].copy_from_slice(&(width as i32).to_le_bytes());
    dib[8..12].copy_from_slice(&(height as i32).to_le_bytes());
    dib[12..14].copy_from_slice(&1u16.to_le_bytes());
    dib[14..16].copy_from_slice(&32u16.to_le_bytes());
    dib[16..20].copy_from_slice(&3u32.to_le_bytes());
    dib[20..24].copy_from_slice(&(count as u32).to_le_bytes());
    for (i, mask) in [0x00ff0000u32, 0x0000ff00, 0x000000ff, 0xff000000, 0x73524742]
        .iter()
        .enumerate()
    {
        dib[40 + i * 4..44 + i * 4].copy_from_slice(&mask.to_le_bytes());
    }
    for row in rgba.chunks(width * 4).rev() {
        for p in row.chunks(4) {
            dib.extend([p[2], p[1], p[0], p[3]]);
        }
    }
    Ok(dib)
}
pub fn dib_to_png(d: &[u8], max: usize) -> Result<Vec<u8>> {
    let offset = dib_offset(d)?;
    let header = le32(d, 0)? as usize;
    let width = le32(d, 4)? as i32;
    let height = le32(d, 8)? as i32;
    if width <= 0 || height == 0 || height == i32::MIN || le16(d, 12)? != 1 {
        return Err(error("Invalid DIB dimensions"));
    }
    let (w, h) = (width as usize, height.unsigned_abs() as usize);
    let bpp = usize::from(le16(d, 14)?);
    let compression = le32(d, 16)?;
    if ![1, 4, 8, 16, 24, 32].contains(&bpp) || ![0, 3, 6].contains(&compression) {
        return Err(error("Unsupported DIB compression or depth"));
    }
    let length = w
        .checked_mul(h)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n <= max.min(256 * 1024 * 1024))
        .ok_or_else(|| error("Decoded DIB exceeds image conversion limit"))?;
    // The encoder uses stored DEFLATE blocks: one filter byte per row,
    // five bytes per block, six zlib bytes and 57 PNG framing bytes.
    // Check the exact output length before allocating the expanded RGBA data.
    let raw_len = length.checked_add(h).ok_or_else(|| error("PNG size overflow"))?;
    let png_len = raw_len
        .checked_add(
            raw_len
                .div_ceil(65535)
                .checked_mul(5)
                .ok_or_else(|| error("PNG size overflow"))?,
        )
        .and_then(|n| n.checked_add(63))
        .ok_or_else(|| error("PNG size overflow"))?;
    if png_len > max {
        return Err(error("Encoded PNG exceeds image conversion limit"));
    }
    let stride = w
        .checked_mul(bpp)
        .and_then(|n| n.checked_add(31))
        .map(|n| n / 32 * 4)
        .ok_or_else(|| error("DIB stride overflow"))?;
    let pixels = d
        .get(
            offset
                ..offset
                    .checked_add(stride.checked_mul(h).ok_or_else(|| error("DIB size overflow"))?)
                    .ok_or_else(|| error("DIB size overflow"))?,
        )
        .ok_or_else(|| error("Truncated DIB pixels"))?;
    let mut masks = if bpp == 16 {
        [0x7c00, 0x03e0, 0x001f, 0]
    } else {
        [0x00ff0000, 0x0000ff00, 0x000000ff, 0]
    };
    if compression == 3 || compression == 6 {
        for (i, mask) in masks.iter_mut().enumerate() {
            if i < 3 || header >= 56 || compression == 6 {
                *mask = le32(d, 40 + i * 4)?;
            }
        }
    }
    let mut rgba = Vec::with_capacity(length);
    for y in 0..h {
        let row = &pixels[if height > 0 { h - 1 - y } else { y } * stride..][..stride];
        for x in 0..w {
            let p = if bpp <= 8 {
                let bit = x * bpp;
                let index = usize::from((row[bit / 8] >> (8 - bpp - bit % 8)) & ((1 << bpp) - 1) as u8);
                let p = d
                    .get(header + index * 4..header + index * 4 + 4)
                    .filter(|_| header + index * 4 + 4 <= offset)
                    .ok_or_else(|| error("DIB palette index out of bounds"))?;
                [p[2], p[1], p[0], 255]
            } else if bpp == 24 {
                let p = &row[x * 3..x * 3 + 3];
                [p[2], p[1], p[0], 255]
            } else {
                let value = if bpp == 16 {
                    u32::from(u16::from_le_bytes(row[x * 2..x * 2 + 2].try_into().map_err(error)?))
                } else {
                    u32::from_le_bytes(row[x * 4..x * 4 + 4].try_into().map_err(error)?)
                };
                masks.map(|mask| {
                    if mask == 0 {
                        255
                    } else {
                        let shift = mask.trailing_zeros();
                        let range = mask >> shift;
                        (((u64::from(value & mask) >> shift) * 255 + u64::from(range) / 2) / u64::from(range)) as u8
                    }
                })
            };
            rgba.extend(p);
        }
    }
    Ok(encode_png(w as u32, h as u32, &rgba))
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
fn adler(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in bytes {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], value: &[u8]) {
    out.extend((value.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(value);
    out.extend(crc32(&out[start..]).to_be_bytes());
}
fn encode_png(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity(rgba.len() + h as usize);
    for row in rgba.chunks(w as usize * 4) {
        raw.push(0);
        raw.extend(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks = raw.chunks(65535);
    let count = blocks.len();
    for (i, block) in blocks.enumerate() {
        z.push(u8::from(i + 1 == count));
        let len = block.len() as u16;
        z.extend(len.to_le_bytes());
        z.extend((!len).to_le_bytes());
        z.extend(block);
    }
    z.extend(adler(&raw).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::from(w.to_be_bytes());
    ihdr.extend(h.to_be_bytes());
    ihdr.extend([8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}
pub fn png_to_dib(png: &[u8], max: usize) -> Result<Vec<u8>> {
    if png.get(..8) != Some(b"\x89PNG\r\n\x1a\n") {
        return Err(error("Invalid PNG signature"));
    }
    let (mut pos, mut header, mut z, mut palette, mut transparency) =
        (8usize, None, Vec::new(), Vec::new(), Vec::new());
    let mut ended = false;
    while pos < png.len() {
        let len = be32(png, pos)? as usize;
        let end = pos
            .checked_add(12)
            .and_then(|n| n.checked_add(len))
            .filter(|n| *n <= png.len())
            .ok_or_else(|| error("Truncated PNG chunk"))?;
        let kind = &png[pos + 4..pos + 8];
        let value = &png[pos + 8..end - 4];
        if crc32(&png[pos + 4..end - 4]) != be32(png, end - 4)? {
            return Err(error("Invalid PNG CRC"));
        }
        match kind {
            b"IHDR" => {
                if len != 13 || header.is_some() {
                    return Err(error("Invalid PNG IHDR"));
                }
                header = Some(value.to_vec());
            }
            b"IDAT" => {
                if z.len().saturating_add(len) > 256 * 1024 * 1024 {
                    return Err(error("PNG data exceeds limit"));
                }
                z.extend(value);
            }
            b"PLTE" => palette = value.to_vec(),
            b"tRNS" => transparency = value.to_vec(),
            b"IEND" => {
                ended = true;
                break;
            }
            _ if kind[0] & 32 == 0 => return Err(error("Unsupported critical PNG chunk")),
            _ => {}
        }
        pos = end;
    }
    if !ended {
        return Err(error("PNG lacks IEND"));
    }
    let h = header.ok_or_else(|| error("PNG lacks IHDR"))?;
    let w = be32(&h, 0)? as usize;
    let height = be32(&h, 4)? as usize;
    let depth = usize::from(h[8]);
    let color = h[9];
    if w == 0 || height == 0 || h[10..13] != [0, 0, 0] {
        return Err(error("Unsupported PNG dimensions, compression or interlace"));
    }
    let channels = match color {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return Err(error("Unsupported PNG color")),
    };
    if ![1, 2, 4, 8, 16].contains(&depth) || (color != 0 && color != 3 && depth < 8) || (color == 3 && depth == 16) {
        return Err(error("Unsupported PNG depth"));
    }
    let rgba_len = w
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| n.saturating_add(108) <= max.min(256 * 1024 * 1024))
        .ok_or_else(|| error("Decoded PNG exceeds caller limit"))?;
    let stride = w
        .checked_mul(channels * depth)
        .and_then(|n| n.checked_add(7))
        .map(|n| n / 8)
        .ok_or_else(|| error("PNG stride overflow"))?;
    let raw_len = (stride + 1)
        .checked_mul(height)
        .ok_or_else(|| error("PNG size overflow"))?;
    let raw = inflate_zlib(&z, raw_len)?;
    if raw.len() != raw_len {
        return Err(error("PNG scanline length mismatch"));
    }
    let mut previous = vec![0u8; stride];
    let bpp = (channels * depth).div_ceil(8);
    let mut rgba = Vec::with_capacity(rgba_len);
    for line in raw.chunks(stride + 1) {
        let mut row = line[1..].to_vec();
        for i in 0..stride {
            let a = if i >= bpp { row[i - bpp] } else { 0 };
            let b = previous[i];
            let c = if i >= bpp { previous[i - bpp] } else { 0 };
            let predict = match line[0] {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                4 => paeth(a, b, c),
                _ => return Err(error("Invalid PNG filter")),
            };
            row[i] = row[i].wrapping_add(predict);
        }
        for x in 0..w {
            let sample = |c: usize| -> u16 {
                let bit = (x * channels + c) * depth;
                if depth == 16 {
                    u16::from_be_bytes([row[bit / 8], row[bit / 8 + 1]])
                } else {
                    u16::from((row[bit / 8] >> (8 - depth - bit % 8)) & ((1u16 << depth) - 1) as u8)
                }
            };
            let byte = |s: u16| -> u8 {
                if depth == 16 {
                    (s >> 8) as u8
                } else {
                    (u32::from(s) * 255 / ((1u32 << depth) - 1)) as u8
                }
            };
            let p = match color {
                0 => {
                    let s = sample(0);
                    let v = byte(s);
                    let alpha =
                        if transparency.len() == 2 && s == u16::from_be_bytes([transparency[0], transparency[1]]) {
                            0
                        } else {
                            255
                        };
                    [v, v, v, alpha]
                }
                2 => {
                    let s = [sample(0), sample(1), sample(2)];
                    let transparent = transparency.len() == 6
                        && (0..3).all(|i| s[i] == u16::from_be_bytes([transparency[i * 2], transparency[i * 2 + 1]]));
                    [byte(s[0]), byte(s[1]), byte(s[2]), if transparent { 0 } else { 255 }]
                }
                3 => {
                    let i = usize::from(sample(0));
                    let p = palette
                        .get(i * 3..i * 3 + 3)
                        .ok_or_else(|| error("Invalid PNG palette index"))?;
                    [p[0], p[1], p[2], transparency.get(i).copied().unwrap_or(255)]
                }
                4 => {
                    let v = byte(sample(0));
                    [v, v, v, byte(sample(1))]
                }
                _ => [byte(sample(0)), byte(sample(1)), byte(sample(2)), byte(sample(3))],
            };
            rgba.extend(p);
        }
        previous = row;
    }
    rgba_dib(w, height, &rgba)
}
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i32::from(a) + i32::from(b) - i32::from(c);
    let (pa, pb, pc) = (
        (p - i32::from(a)).abs(),
        (p - i32::from(b)).abs(),
        (p - i32::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
struct Bits<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl Bits<'_> {
    fn take(&mut self, n: usize) -> Result<u32> {
        let mut v = 0;
        for i in 0..n {
            let b = self.bytes.get(self.pos / 8).ok_or_else(|| error("Truncated DEFLATE"))?;
            v |= u32::from((b >> (self.pos % 8)) & 1) << i;
            self.pos += 1;
        }
        Ok(v)
    }
}
struct Huffman {
    entries: std::collections::BTreeMap<(usize, u32), usize>,
}
impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self> {
        let mut counts = [0u32; 16];
        for length in lengths {
            if *length > 15 {
                return Err(error("Invalid DEFLATE code length"));
            }
            if *length != 0 {
                counts[usize::from(*length)] += 1;
            }
        }
        let mut code = 0;
        let mut next = [0u32; 16];
        for bits in 1..16 {
            code = (code + counts[bits - 1]) << 1;
            next[bits] = code;
            if code + counts[bits] > 1 << bits {
                return Err(error("Oversubscribed DEFLATE tree"));
            }
        }
        let mut entries = std::collections::BTreeMap::new();
        for (symbol, length) in lengths.iter().enumerate().filter(|(_, n)| **n != 0) {
            let l = usize::from(*length);
            entries.insert((l, next[l]), symbol);
            next[l] += 1;
        }
        Ok(Self { entries })
    }
    fn decode(&self, b: &mut Bits<'_>) -> Result<usize> {
        let mut code = 0;
        for n in 1..16 {
            code = (code << 1) | b.take(1)?;
            if let Some(symbol) = self.entries.get(&(n, code)) {
                return Ok(*symbol);
            }
        }
        Err(error("Invalid DEFLATE symbol"))
    }
}
fn inflate_zlib(z: &[u8], max: usize) -> Result<Vec<u8>> {
    if z.len() < 6
        || z[0] & 15 != 8
        || z[0] >> 4 > 7
        || (u16::from(z[0]) * 256 + u16::from(z[1])) % 31 != 0
        || z[1] & 32 != 0
    {
        return Err(error("Invalid PNG zlib stream"));
    }
    let mut b = Bits {
        bytes: &z[2..z.len() - 4],
        pos: 0,
    };
    let mut out = Vec::new();
    loop {
        let last = b.take(1)? != 0;
        let kind = b.take(2)?;
        if kind == 0 {
            b.pos = b.pos.div_ceil(8) * 8;
            let len = b.take(16)? as usize;
            let inverse = b.take(16)? as u16;
            if len as u16 != !inverse || out.len().saturating_add(len) > max {
                return Err(error("Invalid or oversized DEFLATE stored block"));
            }
            for _ in 0..len {
                out.push(b.take(8)? as u8);
            }
        } else if kind == 1 || kind == 2 {
            let (literal, distance) = if kind == 1 {
                let lengths: Vec<u8> = (0..288)
                    .map(|s| match s {
                        0..=143 => 8,
                        144..=255 => 9,
                        256..=279 => 7,
                        _ => 8,
                    })
                    .collect();
                (Huffman::new(&lengths)?, Huffman::new(&[5; 32])?)
            } else {
                let nl = b.take(5)? as usize + 257;
                let nd = b.take(5)? as usize + 1;
                let nc = b.take(4)? as usize + 4;
                let order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                let mut lengths = [0u8; 19];
                for i in 0..nc {
                    lengths[order[i]] = b.take(3)? as u8;
                }
                let tree = Huffman::new(&lengths)?;
                let mut values = Vec::new();
                while values.len() < nl + nd {
                    let sym = tree.decode(&mut b)?;
                    match sym {
                        0..=15 => values.push(sym as u8),
                        16..=18 => {
                            let (value, n) = match sym {
                                16 => (
                                    *values
                                        .last()
                                        .ok_or_else(|| error("DEFLATE repeat has no previous code"))?,
                                    b.take(2)? + 3,
                                ),
                                17 => (0, b.take(3)? + 3),
                                _ => (0, b.take(7)? + 11),
                            };
                            if values.len() + n as usize > nl + nd {
                                return Err(error("DEFLATE code repeat overflow"));
                            }
                            values.resize(values.len() + n as usize, value);
                        }
                        _ => return Err(error("Invalid DEFLATE code length symbol")),
                    }
                }
                if values[256] == 0 {
                    return Err(error("DEFLATE tree lacks end marker"));
                }
                (Huffman::new(&values[..nl])?, Huffman::new(&values[nl..])?)
            };
            loop {
                let symbol = literal.decode(&mut b)?;
                if symbol < 256 {
                    if out.len() == max {
                        return Err(error("PNG expands beyond limit"));
                    }
                    out.push(symbol as u8);
                } else if symbol == 256 {
                    break;
                } else {
                    const LENGTH: [usize; 29] = [
                        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
                        163, 195, 227, 258,
                    ];
                    const EXTRA: [usize; 29] = [
                        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
                    ];
                    let i = symbol
                        .checked_sub(257)
                        .filter(|i| *i < 29)
                        .ok_or_else(|| error("Invalid DEFLATE length"))?;
                    let len = LENGTH[i] + b.take(EXTRA[i])? as usize;
                    let d = distance.decode(&mut b)?;
                    if d > 29 {
                        return Err(error("Invalid DEFLATE distance"));
                    }
                    let extra = if d < 4 { 0 } else { d / 2 - 1 };
                    let base = if d < 4 { d + 1 } else { ((2 + (d & 1)) << extra) + 1 };
                    let dist = base + b.take(extra)? as usize;
                    if dist > out.len() || out.len().saturating_add(len) > max {
                        return Err(error("DEFLATE distance or size exceeds bounds"));
                    }
                    for _ in 0..len {
                        out.push(out[out.len() - dist]);
                    }
                }
            }
        } else {
            return Err(error("Reserved DEFLATE block"));
        }
        if last {
            break;
        }
    }
    if adler(&out) != be32(z, z.len() - 4)? {
        return Err(error("PNG zlib checksum mismatch"));
    }
    Ok(out)
}
