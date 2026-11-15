"""Generate a plain PNG fixture in uploads/ for the real browser acceptance run."""
import struct
import zlib
from .xhs_browser.config import PROJECT_ROOT


def chunk(tag, data):
    return struct.pack('!I', len(data)) + tag + data + struct.pack('!I', zlib.crc32(tag + data) & 0xffffffff)


def main():
    width, height = 900, 1200
    pixels = b''.join(b'\0' + bytes([46, 71, 101]) * width for _ in range(height))
    path = PROJECT_ROOT / 'uploads' / 'nodriver-validation.png'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('!IIBBBBB', width, height, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(pixels)) + chunk(b'IEND', b''))
    print(path)


if __name__ == '__main__':
    main()
