"""Resolve two pinned metrics JNI tuples in the reconstructed ARM64 library."""
import hashlib
import json
from pathlib import Path
import struct
import sys
path = Path(sys.argv[1])
data = path.read_bytes()
assert len(data) == 12783016
assert hashlib.sha256(data).hexdigest() == 'be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5'
assert data[:6] == bytes.fromhex('7f454c460201')
shoff = struct.unpack_from('<Q', data, 40)[0]
size, count = struct.unpack_from('<HH', data, 58)
assert size == 64
sections = [struct.unpack_from('<IIQQQQIIQQ', data, shoff + size*i) for i in range(count)]
def file_offset(address, length=1):
    matches = [s[4]+address-s[3] for s in sections if s[1] != 8 and s[5]
               and s[3] <= address and address+length <= s[3]+s[5]]
    assert len(matches) == 1, (address, matches)
    return matches[0]
def string(address):
    offset = file_offset(address)
    end = data.index(0, offset)
    file_offset(address, end-offset+1)
    return data[offset:end].decode('ascii')
relocations = {}
for section in sections:
    if section[1] != 4: continue
    assert section[9] == 24
    for off in range(section[4], section[4]+section[5], 24):
        address, info, addend = struct.unpack_from('<QQq', data, off)
        if info & 0xffffffff == 1027:
            assert info >> 32 == 0
            relocations[address] = addend
result = []
expected = {'jvidispatchIIIIIOO':'(JJJJLjava/lang/Object;Ljava/lang/Object;)J',
            'jvidispatchIOOOO':'(ILjava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;)J'}
for name, signature in expected.items():
    rows = []
    needle = name.encode('ascii') + b'\0'
    position = data.index(needle)
    assert data.find(needle, position+1) == -1
    mapped = [s[3]+position-s[4] for s in sections if s[1] != 8 and s[5] and s[4] <= position < s[4]+s[5]]
    assert len(mapped) == 1
    for slot, address in relocations.items():
        if address != mapped[0]: continue
        assert slot+8 in relocations and slot+16 in relocations
        function = relocations[slot+16]
        assert string(relocations[slot+8]) == signature
        rows.append({'name':name,'signature':signature,'slot_va':hex(slot),
                     'slot_file_offset':hex(file_offset(slot,24)),
                     'function_va':hex(function),'function_file_offset':hex(file_offset(function,4))})
    assert len(rows) == 1
    result.extend(rows)
table = bytes(data[file_offset(0x2e9553,6):file_offset(0x2e9553,6)+6])
print(json.dumps({'scope':'static relocated tuples; registration execution not observed',
                  'methods':result,'setup_selector_table_va':'0x2e9553',
                  'setup_selector_table_hex':table.hex(),
                  'setup_selector_destinations':{str(i):hex(0x6eeec0+v*4) for i,v in enumerate(table)}},indent=2))
