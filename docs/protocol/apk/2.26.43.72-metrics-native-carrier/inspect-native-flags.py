"""Check exact metrics JNI declaration access flags in the pinned APK."""
import hashlib
from pathlib import Path
import struct
import sys
import zipfile
apk = Path(sys.argv[1])
assert hashlib.sha256(apk.read_bytes()).hexdigest() == "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
with zipfile.ZipFile(apk) as archive:
    for name in archive.namelist():
        if not (name.startswith("classes") and name.endswith(".dex")):
            continue
        data = archive.read(name)
        def u32(offset):
            return struct.unpack_from("<I", data, offset)[0]
        def leb(offset):
            value = shift = 0
            while True:
                b = data[offset]
                offset += 1
                value |= (b & 127) << shift
                if b < 128:
                    return value, offset
                shift += 7
        strings = []
        for i in range(u32(56)):
            _, offset = leb(u32(u32(60) + 4 * i))
            strings.append(data[offset:data.index(0, offset)].decode("utf-8", "replace"))
        types = [strings[u32(u32(68) + 4 * i)] for i in range(u32(64))]
        for i in range(u32(96)):
            base = u32(100) + 32 * i
            if types[u32(base)] != "Lcom/whatsapp/wamsys/JniBridge;":
                continue
            offset = u32(base + 24)
            counts = []
            for _ in range(4):
                n, offset = leb(offset)
                counts.append(n)
            for _ in range(counts[0] + counts[1]):
                _, offset = leb(offset)
                _, offset = leb(offset)
            for count in counts[2:]:
                method_id = 0
                for _ in range(count):
                    delta, offset = leb(offset)
                    flags, offset = leb(offset)
                    code, offset = leb(offset)
                    method_id += delta
                    method_base = u32(92) + 8 * method_id
                    method_name = strings[u32(method_base + 4)]
                    if method_name in {"jvidispatchIIIIIOO", "jvidispatchIOOOO"}:
                        print(f"{name} {method_name} flags=0x{flags:x} native={bool(flags & 0x100)} abstract={bool(flags & 0x400)} code_off={code}")
