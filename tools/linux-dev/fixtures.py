#!/usr/bin/env python3
"""Generate actual PNG clipboard input with the standard library, then inspect wire DIB."""
import struct, zlib, binascii

def chunk(kind, value):
    return struct.pack('>I', len(value)) + kind + value + struct.pack('>I', binascii.crc32(kind + value))

def image(path, w, h, level):
    # All bytes vary. The small image exercises alpha and DEFLATE dynamic trees.
    row = bytes((i * 37 + i // 7) % 256 for i in range(w * 4))
    raw = b''.join(b'\0' + bytes((v + y) % 256 for v in row) for y in range(h))
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw, level)) + chunk(b'IEND', b'')
    open(path, 'wb').write(png)
    return row

image('/tmp/oe-small.png', 64, 64, 6)
image('/tmp/oe-large.png', 1600, 1640, 0)
print('fixtures: small dynamic-DEFLATE PNG; large PNG bytes=' + str(len(open('/tmp/oe-large.png','rb').read())))
