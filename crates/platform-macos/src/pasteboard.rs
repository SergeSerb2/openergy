//! NSPasteboard and bounded, native-codec PNG/TIFF ↔ Windows DIB conversion.
//! `info` reads metadata only. Content and write APIs are separate entry points.
use crate::ffi::{self, OwnedCf, OwnedObj};
use crate::{error, Result};

pub const TEXT_TYPE: &str = "public.utf8-plain-text";
pub const HTML_TYPE: &str = "public.html";
pub const PNG_TYPE: &str = "public.png";
pub const TIFF_TYPE: &str = "public.tiff";
const DIB_HEADER: usize = 108; // BITMAPV4HEADER, explicit alpha mask and sRGB.

#[derive(Debug, Clone)]
pub struct PasteboardInfo {
    pub change_count: isize,
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    pub text: Option<String>,
    pub html: Option<String>,
    pub dib: Option<Vec<u8>>,
}

pub fn change_count() -> isize {
    let _pool = ffi::Pool::new();
    // Safety: public NSPasteboard changeCount returns NSInteger; receiver lives within this pool.
    unsafe { ffi::msg0(general(), c"changeCount") }
}

/// Reads only changeCount and type names. No dataForType/stringForType call.
pub fn info() -> Result<PasteboardInfo> {
    let _pool = ffi::Pool::new();
    // Safety: native objects remain retained within the pool; static selectors use SDK-matched ABIs.
    unsafe {
        let pasteboard = general();
        let change_count = ffi::msg0(pasteboard, c"changeCount");
        let array: ffi::Id = ffi::msg0(pasteboard, c"types");
        let count: usize = ffi::msg0(array, c"count");
        let mut types = Vec::with_capacity(count);
        for index in 0..count {
            let string: ffi::Id = ffi::msg1(array, c"objectAtIndex:", index);
            types.push(ffi::cf_string(string.cast_const())?);
        }
        Ok(PasteboardInfo { change_count, types })
    }
}

fn general() -> ffi::Id {
    // Safety: public class method returns a borrowed pasteboard in the current pool.
    unsafe { ffi::msg0(ffi::class(c"NSPasteboard"), c"generalPasteboard") }
}

/// Explicit content read. max_bytes caps each input blob, the decoded raster,
/// the output DIB, and the sum of returned wire formats. NSPasteboard has no
/// public length-only data query, so its fetch may allocate before length check.
pub fn read(max_bytes: usize) -> Result<Clipboard> {
    let _pool = ffi::Pool::new();
    let before = info()?;
    let mut remaining = max_bytes;
    let mut output = Clipboard::default();
    for (uti, destination) in [(TEXT_TYPE, &mut output.text), (HTML_TYPE, &mut output.html)] {
        if before.types.iter().any(|value| value == uti) {
            let bytes = read_type(uti, remaining)?;
            let text = String::from_utf8(bytes).map_err(|_| error(format!("{uti} is not UTF-8")))?;
            let text = lf_endings(text);
            remaining =
                remaining
                    .checked_sub(text.len())
                    .ok_or(openergy_platform::PlatformError::ClipboardTooLarge {
                        bytes: max_bytes.saturating_sub(remaining).saturating_add(text.len()),
                        limit: max_bytes,
                    })?;
            *destination = Some(text);
        }
    }
    let image_type = if before.types.iter().any(|value| value == PNG_TYPE) {
        Some(PNG_TYPE)
    } else if before.types.iter().any(|value| value == TIFF_TYPE) {
        Some(TIFF_TYPE)
    } else {
        None
    };
    if let Some(uti) = image_type {
        let bytes = read_type(uti, remaining)?;
        output.dib = Some(image_to_dib(&bytes, remaining)?);
    }
    if change_count() != before.change_count {
        return Err(error("pasteboard changed while reading; retry"));
    }
    Ok(output)
}

fn read_type(uti: &str, maximum: usize) -> Result<Vec<u8>> {
    let name = ffi::ns_string(uti)?;
    // Safety: explicit content-read call uses a live pasteboard and retained NSString type name.
    let data = unsafe { ffi::msg1(general(), c"dataForType:", name.0) };
    ffi::data_bytes(data, maximum)
}

/// Explicit pasteboard replacement. Prepare and validate every representation
/// before clearContents. There is no atomic replace API or rollback guarantee.
pub fn write(clipboard: &Clipboard, max_bytes: usize) -> Result<()> {
    let _pool = ffi::Pool::new();
    let mut total = 0usize;
    for text in [&clipboard.text, &clipboard.html].into_iter().flatten() {
        total = add_bounded(total, text.len(), max_bytes)?;
    }
    if let Some(dib) = &clipboard.dib {
        add_bounded(total, dib.len(), max_bytes)?;
    }
    let mut formats: Vec<(&str, OwnedObj)> = Vec::new();
    for (uti, text) in [(TEXT_TYPE, &clipboard.text), (HTML_TYPE, &clipboard.html)] {
        if let Some(text) = text {
            formats.push((uti, ffi::ns_data(text.as_bytes())?)); // macOS accepts wire LF as-is.
        }
    }
    if let Some(dib) = &clipboard.dib {
        let image = dib_to_image(dib, max_bytes - total)?;
        formats.push((PNG_TYPE, ffi::ns_data(&encode_image(&image, 4, max_bytes)?)?));
        formats.push((TIFF_TYPE, ffi::ns_data(&encode_image(&image, 0, max_bytes)?)?));
    }
    // Build a single item before touching the user's pasteboard.
    // Safety: native objects remain retained within the pool; static selectors use SDK-matched ABIs.
    unsafe {
        if formats.is_empty() {
            // An empty remote clipboard is a clear, not an item with no types.
            ffi::msg0::<isize>(general(), c"clearContents");
            return Ok(());
        }
        let item = OwnedObj::new(
            ffi::msg0(ffi::msg0::<ffi::Id>(ffi::class(c"NSPasteboardItem"), c"alloc"), c"init"),
            "pasteboard item",
        )?;
        for (uti, data) in formats {
            let name = ffi::ns_string(uti)?;
            let success: bool = ffi::msg2(item.0, c"setData:forType:", data.0, name.0);
            if !success {
                return Err(error(format!("cannot prepare pasteboard {uti}")));
            }
        }
        let array: ffi::Id = ffi::msg1(ffi::class(c"NSArray"), c"arrayWithObject:", item.0);
        let pasteboard = general();
        ffi::msg0::<isize>(pasteboard, c"clearContents");
        let success: bool = ffi::msg1(pasteboard, c"writeObjects:", array);
        if !success {
            return Err(error("NSPasteboard writeObjects failed after clearing"));
        }
    }
    Ok(())
}

fn lf_endings(mut text: String) -> String {
    // ASCII CR/LF cannot occur inside a UTF-8 multibyte sequence. Compact the
    // owned input in place instead of making two full-size replacement copies.
    // Safety: edits preserve valid UTF-8 and remove only ASCII bytes.
    let bytes = unsafe { text.as_mut_vec() };
    let mut read = 0;
    let mut write = 0;
    while read < bytes.len() {
        let byte = bytes[read];
        bytes[write] = if byte == b'\r' { b'\n' } else { byte };
        read += 1;
        if byte == b'\r' && bytes.get(read) == Some(&b'\n') {
            read += 1;
        }
        write += 1;
    }
    bytes.truncate(write);
    text
}
fn add_bounded(a: usize, b: usize, maximum: usize) -> Result<usize> {
    a.checked_add(b)
        .filter(|sum| *sum <= maximum)
        .ok_or(openergy_platform::PlatformError::ClipboardTooLarge {
            bytes: a.saturating_add(b),
            limit: maximum,
        })
}
fn bounded_image_len(width: usize, height: usize, overhead: usize, maximum: usize) -> Result<usize> {
    if width == 0 || height == 0 || width > i32::MAX as usize || height > i32::MAX as usize {
        return Err(error("invalid image dimensions"));
    }
    let pixels = width
        .checked_mul(height)
        .and_then(|count| count.checked_mul(4))
        .ok_or_else(|| error("image size overflow"))?;
    add_bounded(pixels, overhead, maximum)?;
    Ok(pixels)
}

/// Top-to-bottom, sRGB straight RGBA8. Invisible RGB is normalized to zero.
/// Decoding first quantizes through an RGBA8 premultiplied native context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// Native metadata check precedes NSBitmapImageRep's potentially lazy decoder.
pub fn decode_image(bytes: &[u8], max_bytes: usize) -> Result<RgbaImage> {
    if bytes.is_empty() {
        return Err(error("empty image"));
    }
    add_bounded(0, bytes.len(), max_bytes)?;
    let _pool = ffi::Pool::new();
    // Safety: native objects remain retained within the pool; static selectors use SDK-matched ABIs.
    unsafe {
        let data = OwnedCf::new(
            ffi::CFDataCreate(ffi::null(), bytes.as_ptr(), bytes.len() as isize),
            "image CFData",
        )?;
        let source = OwnedCf::new(
            ffi::CGImageSourceCreateWithData(data.0, ffi::null()),
            "image metadata source",
        )?;
        let props = OwnedCf::new(
            ffi::CGImageSourceCopyPropertiesAtIndex(source.0, 0, ffi::null()),
            "image properties",
        )?;
        let width = property_dimension(props.0, ffi::kCGImagePropertyPixelWidth)?;
        let height = property_dimension(props.0, ffi::kCGImagePropertyPixelHeight)?;
        let length = bounded_image_len(width, height, DIB_HEADER, max_bytes)?;
        let rep = bitmap_from_data(bytes)?;
        let image: ffi::Ref = ffi::msg0(rep.0, c"CGImage");
        if image.is_null() {
            return Err(error("NSBitmapImageRep has no CGImage"));
        }
        if ffi::CGImageGetWidth(image) != width || ffi::CGImageGetHeight(image) != height {
            return Err(error("image dimensions changed between metadata and decode"));
        }
        let color = OwnedCf::new(
            ffi::CGColorSpaceCreateWithName(ffi::kCGColorSpaceSRGB),
            "sRGB color space",
        )?;
        let mut pixels = vec![0u8; length];
        // ByteOrder32Big | PremultipliedLast produces R,G,B,A bytes on both
        // supported Mac architectures. Context must drop before pixels move.
        let context = OwnedCf::new(
            ffi::CGBitmapContextCreate(pixels.as_mut_ptr().cast(), width, height, 8, width * 4, color.0, 0x4001),
            "RGBA bitmap context",
        )?;
        ffi::CGContextDrawImage(
            context.0,
            ffi::Rect {
                origin: ffi::Point::default(),
                size: ffi::Size {
                    width: width as f64,
                    height: height as f64,
                },
            },
            image,
        );
        drop(context);
        for pixel in pixels.as_chunks_mut::<4>().0 {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel[..3] {
                *channel = (u32::from(*channel) * 255 + alpha / 2)
                    .checked_div(alpha)
                    .unwrap_or(0)
                    .min(255) as u8;
            }
        }
        Ok(RgbaImage { width, height, pixels })
    }
}

unsafe fn property_dimension(props: ffi::Ref, key: ffi::Ref) -> Result<usize> {
    // Safety: props is a live image metadata dictionary; key is an ImageIO singleton.
    let number = unsafe { ffi::CFDictionaryGetValue(props, key) };
    let mut value = 0i64;
    // Safety: non-null image CFNumber writes an SInt64 to the aligned live output pointer.
    if number.is_null() || !unsafe { ffi::CFNumberGetValue(number, 4, (&mut value as *mut i64).cast()) } || value <= 0 {
        return Err(error("image lacks valid pixel dimensions"));
    }
    usize::try_from(value).map_err(|_| error("image dimensions overflow"))
}

fn bitmap_from_data(bytes: &[u8]) -> Result<OwnedObj> {
    let data = ffi::ns_data(bytes)?;
    // Safety: native objects remain retained within the pool; static selectors use SDK-matched ABIs.
    unsafe {
        let allocated = ffi::msg0(ffi::class(c"NSBitmapImageRep"), c"alloc");
        OwnedObj::new(
            ffi::msg1(allocated, c"initWithData:", data.0),
            "NSBitmapImageRep decoder",
        )
    }
}

fn encode_image(image: &RgbaImage, file_type: usize, maximum: usize) -> Result<Vec<u8>> {
    let length = bounded_image_len(image.width, image.height, DIB_HEADER, maximum)?;
    if image.pixels.len() != length {
        return Err(error("RGBA buffer size does not match dimensions"));
    }
    let _pool = ffi::Pool::new();
    let rep = ffi::rgba_bitmap(image.width, image.height)?;
    // Safety: native objects remain retained within the pool; static selectors use SDK-matched ABIs.
    unsafe {
        let bytes: *mut u8 = ffi::msg0(rep.0, c"bitmapData");
        if bytes.is_null() {
            return Err(error("NSBitmapImageRep has no writable data"));
        }
        let stride: isize = ffi::msg0(rep.0, c"bytesPerRow");
        if stride < (image.width * 4) as isize {
            return Err(error("invalid native bitmap stride"));
        }
        for row in 0..image.height {
            std::ptr::copy_nonoverlapping(
                image.pixels.as_ptr().add(row * image.width * 4),
                bytes.add(row * stride as usize),
                image.width * 4,
            );
        }
        let data = ffi::msg2(
            rep.0,
            c"representationUsingType:properties:",
            file_type,
            ffi::empty_dictionary(),
        );
        ffi::data_bytes(data, maximum)
    }
}

pub fn image_to_dib(bytes: &[u8], max_bytes: usize) -> Result<Vec<u8>> {
    let image = decode_image(bytes, max_bytes)?;
    rgba_to_dib(&image, max_bytes)
}

/// Emits BITMAPV4HEADER + bottom-up BGRA8. The wire has no BMP file header.
pub fn rgba_to_dib(image: &RgbaImage, max_bytes: usize) -> Result<Vec<u8>> {
    let length = bounded_image_len(image.width, image.height, DIB_HEADER, max_bytes)?;
    if image.pixels.len() != length {
        return Err(error("RGBA buffer length mismatch"));
    }
    let image_bytes = u32::try_from(length).map_err(|_| error("DIB pixel data exceeds u32"))?;
    let mut dib = vec![0u8; DIB_HEADER + length];
    put_u32(&mut dib, 0, DIB_HEADER as u32);
    put_u32(&mut dib, 4, image.width as u32);
    put_u32(&mut dib, 8, image.height as u32); // positive: bottom-up
    put_u16(&mut dib, 12, 1);
    put_u16(&mut dib, 14, 32);
    put_u32(&mut dib, 16, 3); // BI_BITFIELDS
    put_u32(&mut dib, 20, image_bytes);
    put_u32(&mut dib, 40, 0x00FF_0000);
    put_u32(&mut dib, 44, 0x0000_FF00);
    put_u32(&mut dib, 48, 0x0000_00FF);
    put_u32(&mut dib, 52, 0xFF00_0000);
    put_u32(&mut dib, 56, 0x7352_4742); // LCS_sRGB
    for y in 0..image.height {
        for x in 0..image.width {
            let source = (y * image.width + x) * 4;
            let target = DIB_HEADER + ((image.height - 1 - y) * image.width + x) * 4;
            let rgba = &image.pixels[source..source + 4];
            dib[target..target + 4].copy_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]);
        }
    }
    Ok(dib)
}

struct DibInfo {
    width: usize,
    height: usize,
    offset: usize,
    masks: [u32; 4],
    top_down: bool,
    stride: usize,
    pixel_bytes: usize,
}

fn parse_dib(dib: &[u8], maximum: usize) -> Result<DibInfo> {
    add_bounded(0, dib.len(), maximum)?;
    let header = get_u32(dib, 0)? as usize;
    if ![40, 52, 56, 108, 124].contains(&header) || dib.len() < header {
        return Err(error("unsupported or truncated DIB header"));
    }
    let width = get_u32(dib, 4)? as i32;
    let height = get_u32(dib, 8)? as i32;
    if width <= 0 || height == 0 || height == i32::MIN {
        return Err(error("DIB dimensions are invalid"));
    }
    let bpp = get_u16(dib, 14)?;
    if get_u16(dib, 12)? != 1 || ![24, 32].contains(&bpp) {
        return Err(error("only single-plane 24/32-bpp DIB is supported"));
    }
    let compression = get_u32(dib, 16)?;
    if (compression != 0 && compression != 3) || (bpp == 24 && compression != 0) {
        return Err(error("DIB must be BI_RGB or BI_BITFIELDS"));
    }
    // For 24/32-bit data, a palette is neither required nor accepted. Its size
    // otherwise changes the pixel offset and can conceal truncated pixels.
    if get_u32(dib, 32)? != 0 {
        return Err(error("24/32-bpp DIB palettes are unsupported"));
    }
    let offset = if compression == 3 && header == 40 { 52 } else { header };
    let pixel_bytes = usize::from(bpp / 8);
    let height_abs = height.unsigned_abs() as usize;
    bounded_image_len(width as usize, height_abs, offset, maximum)?;
    let stride = (width as usize)
        .checked_mul(pixel_bytes)
        .and_then(|n| n.checked_add(3))
        .map(|n| n & !3)
        .ok_or_else(|| error("DIB row overflow"))?;
    let length = stride
        .checked_mul(height_abs)
        .ok_or_else(|| error("DIB size overflow"))?;
    add_bounded(offset, length, maximum)?;
    if dib.len() < offset + length {
        return Err(error("truncated DIB pixel data"));
    }
    let declared = get_u32(dib, 20)? as usize;
    if declared != 0 && declared != length {
        return Err(error("DIB biSizeImage does not match dimensions"));
    }
    // BI_RGB's high byte is reserved, not alpha. For BITFIELDS, respect
    // an explicit alpha mask; three-mask headers are opaque.
    let masks = if compression == 0 {
        [0x00FF_0000, 0x0000_FF00, 0x0000_00FF, 0]
    } else {
        [
            get_u32(dib, 40)?,
            get_u32(dib, 44)?,
            get_u32(dib, 48)?,
            if header >= 56 { get_u32(dib, 52)? } else { 0 },
        ]
    };
    let mut used = 0u32;
    for (index, mask) in masks.into_iter().enumerate() {
        if mask == 0 {
            if index < 3 {
                return Err(error("DIB RGB channel mask is zero"));
            } else {
                continue;
            }
        }
        let shifted = mask >> mask.trailing_zeros();
        if shifted & shifted.wrapping_add(1) != 0 || used & mask != 0 {
            return Err(error("DIB masks overlap or are not contiguous"));
        }
        used |= mask;
    }
    // Embedded V5 profiles could change color interpretation and pixel offsets.
    if header == 124 && (get_u32(dib, 112)? != 0 || get_u32(dib, 116)? != 0) {
        return Err(error("embedded DIB color profiles are unsupported"));
    }
    Ok(DibInfo {
        width: width as usize,
        height: height_abs,
        offset,
        masks,
        top_down: height < 0,
        stride,
        pixel_bytes,
    })
}

/// Validates and wraps DIB in the 14-byte BITMAPFILEHEADER required by AppKit.
pub fn dib_to_bmp(dib: &[u8], max_bytes: usize) -> Result<Vec<u8>> {
    let info = parse_dib(dib, max_bytes)?;
    let length = dib.len().checked_add(14).ok_or_else(|| error("BMP length overflow"))?;
    let size = u32::try_from(length).map_err(|_| error("BMP exceeds u32"))?;
    let mut bmp = vec![0u8; 14];
    bmp[..2].copy_from_slice(b"BM");
    put_u32(&mut bmp, 2, size);
    put_u32(&mut bmp, 10, (14 + info.offset) as u32);
    bmp.extend_from_slice(dib);
    Ok(bmp)
}

pub fn dib_to_image(dib: &[u8], max_bytes: usize) -> Result<RgbaImage> {
    let info = parse_dib(dib, max_bytes)?;
    // Parse the bounded DIB directly. A second BMP copy and AppKit decode added
    // allocations and exposed native decoding before these same pixels were
    // read again. PNG/TIFF encoding still uses NSBitmapImageRep afterwards.
    let mut pixels = vec![0u8; info.width * info.height * 4];
    for y in 0..info.height {
        for x in 0..info.width {
            let row = if info.top_down { y } else { info.height - 1 - y };
            let source = info.offset + row * info.stride + x * info.pixel_bytes;
            let value = if info.pixel_bytes == 3 {
                u32::from(dib[source]) | (u32::from(dib[source + 1]) << 8) | (u32::from(dib[source + 2]) << 16)
            } else {
                get_u32(dib, source)?
            };
            let target = (y * info.width + x) * 4;
            for channel in 0..4 {
                let mask = info.masks[channel];
                pixels[target + channel] = if mask == 0 {
                    255
                } else {
                    let shift = mask.trailing_zeros();
                    let maximum = u64::from(mask >> shift);
                    ((u64::from((value & mask) >> shift) * 255 + maximum / 2) / maximum) as u8
                };
            }
            if pixels[target + 3] == 0 {
                pixels[target..target + 3].fill(0);
            }
        }
    }
    Ok(RgbaImage {
        width: info.width,
        height: info.height,
        pixels,
    })
}

pub fn dib_to_png(dib: &[u8], max_bytes: usize) -> Result<Vec<u8>> {
    encode_image(&dib_to_image(dib, max_bytes)?, 4, max_bytes)
}
pub fn dib_to_tiff(dib: &[u8], max_bytes: usize) -> Result<Vec<u8>> {
    encode_image(&dib_to_image(dib, max_bytes)?, 0, max_bytes)
}
pub fn rgba_to_png(image: &RgbaImage, max_bytes: usize) -> Result<Vec<u8>> {
    encode_image(image, 4, max_bytes)
}

fn get_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| error("truncated little-endian u16"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}
fn get_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| error("truncated little-endian u32"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}
fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
