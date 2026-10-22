#!/usr/bin/env python3
"""Independent pixel comparison after actual X selection transfers, using Python zlib."""
import struct, zlib, binascii

def pixels(path):
    data = open(path, 'rb').read()
    pos, chunks = 8, []
    while pos < len(data):
        n = struct.unpack_from('>I', data, pos)[0]
        kind, value = data[pos+4:pos+8], data[pos+8:pos+8+n]
        assert binascii.crc32(kind + value) == struct.unpack_from('>I', data, pos+8+n)[0]
        if kind == b'IHDR':
            w,h,depth,color,_,_,interlace = struct.unpack('>IIBBBBB',value)
            assert (depth,color,interlace) == (8,6,0)
        if kind == b'IDAT': chunks.append(value)
        pos += 12+n
    raw = zlib.decompress(b''.join(chunks))
    stride = w*4
    assert len(raw) == (stride+1)*h
    assert all(raw[y*(stride+1)] == 0 for y in range(h))
    return w,h,b''.join(raw[y*(stride+1)+1:(y+1)*(stride+1)] for y in range(h))

w,h,original = pixels('/tmp/oe-small.png')
assert pixels('/tmp/oe-roundtrip.png') == (w,h,original)
w,h,original = pixels('/tmp/oe-large.png')
dib = open('/tmp/oe-large.dib','rb').read()
assert struct.unpack_from('<IiiHHI',dib) == (108,w,h,1,32,3)
bgra = bytearray(dib[108:])
bgra[0::4], bgra[2::4] = bgra[2::4], bgra[0::4]
rgba = b''.join(bgra[y*w*4:(y+1)*w*4] for y in reversed(range(h)))
assert rgba == original
print('independent pixels: PNG -> DIB -> PNG preserves RGBA; 10 MB INCR DIB pixels equal source PNG')
