"""Resolve selected JNI tables using ELF section mappings and RELATIVE relocations."""
import hashlib
from pathlib import Path
import struct
import sys
import json
import zipfile
apk = Path(sys.argv[1])
assert hashlib.sha256(apk.read_bytes()).hexdigest() == "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
with zipfile.ZipFile(apk) as archive:
    data = archive.read("lib/arm64-v8a/libsuperpack.so")
assert hashlib.sha256(data).hexdigest() == "fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f"
assert data[:6] == b"\x7fELF\x02\x01"
shoff = struct.unpack_from("<Q", data, 40)[0]
size, count = struct.unpack_from("<HH", data, 58)
assert size == 64
sections = [struct.unpack_from("<IIQQQQIIQQ", data, shoff + size * i) for i in range(count)]
def file_offset(address, length=1):
    matches = [s[4] + address - s[3] for s in sections if s[1] != 8 and s[3] <= address and address + length <= s[3] + s[5] and s[5]]
    assert len(matches) == 1, (address, matches)
    return matches[0]
relocations = {}
for section in sections:
    if section[1] != 4:
        continue
    assert section[9] == 24
    for offset in range(section[4], section[4] + section[5], 24):
        address, info, addend = struct.unpack_from("<QQq", data, offset)
        if info & 0xffffffff == 1027:
            assert info >> 32 == 0
            relocations[address] = addend

def string(address):
    offset = file_offset(address)
    end = data.index(0, offset)
    assert end - offset < 1024
    file_offset(address, end - offset + 1)
    return data[offset:end].decode("ascii")
result = []
for class_address, table, count in [(0x34dc, 0x380d0, 8), (0x3d76, 0x38018, 5)]:
    rows = []
    for index in range(count):
        slot = table + index * 24
        name, signature, function = [relocations[slot + part] for part in [0, 8, 16]]
        rows.append({"index": index, "slot_va": hex(slot), "slot_file_offset": hex(file_offset(slot, 24)), "name": string(name), "signature": string(signature), "function_va": hex(function), "function_file_offset": hex(file_offset(function, 4))})
    result.append({"class": string(class_address), "table_va": hex(table), "entries": rows})
print(json.dumps(result, indent=2))
