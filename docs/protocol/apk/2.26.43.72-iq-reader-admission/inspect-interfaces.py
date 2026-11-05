"""Print the pinned APK reader observer's transitive interface declarations."""
import hashlib
from pathlib import Path
import struct
import sys
import zipfile

apk = Path(sys.argv[1])
expected = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
if hashlib.sha256(apk.read_bytes()).hexdigest() != expected:
    raise SystemExit("APK digest differs from the pinned build")
classes = {}
with zipfile.ZipFile(apk) as archive:
    for name in archive.namelist():
        if not (name.startswith("classes") and name.endswith(".dex")):
            continue
        data = archive.read(name)
        def u32(offset):
            return struct.unpack_from("<I", data, offset)[0]
        strings = []
        for index in range(u32(56)):
            offset = u32(u32(60) + 4 * index)
            while data[offset] & 128:
                offset += 1
            offset += 1
            end = data.index(0, offset)
            # Queried descriptors are ASCII; other MUTF-8 strings are unused.
            strings.append(data[offset:end].decode("utf-8", "replace"))
        types = [strings[u32(u32(68) + 4 * index)] for index in range(u32(64))]
        for index in range(u32(96)):
            offset = u32(100) + 32 * index
            class_id, flags, parent, interfaces = struct.unpack_from("<IIII", data, offset)
            parents = [] if not interfaces else [
                types[struct.unpack_from("<H", data, interfaces + 4 + 2 * item)[0]]
                for item in range(u32(interfaces))
            ]
            classes[types[class_id]] = (name, flags, None if parent == 0xffffffff else types[parent], parents)
seen = set()
def show(descriptor):
    if descriptor in seen:
        return
    seen.add(descriptor)
    if descriptor not in classes:
        print(f"{descriptor} external declaration not inspected")
        return
    name, flags, parent, interfaces = classes[descriptor]
    print(f"{name} {descriptor} flags=0x{flags:x} extends={parent} interfaces={interfaces}")
    for item in ([parent] if parent and parent != "Ljava/lang/Object;" else []) + interfaces:
        show(item)
show("LX/1lY;")
