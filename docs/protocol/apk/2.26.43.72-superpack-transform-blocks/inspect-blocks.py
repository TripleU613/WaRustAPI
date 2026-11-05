#!/usr/bin/env python3
"""Inspect pinned spo descriptors and block framing; do not decode or execute."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import runpy
import struct

# Reuse the digest-bound APK/member checks and structure-derived metadata cursor.
source = Path(__file__).resolve().parent.parent / '2.26.43.72-superpack-metadata/inspect-metadata.py'
with contextlib.redirect_stdout(io.StringIO()):
    metadata = runpy.run_path(str(source))
blob, data = metadata['blob'], metadata['decoder']
shoff = struct.unpack_from('<Q', data, 40)[0]
size, count = struct.unpack_from('<HH', data, 58)
assert size == 64
sections = [struct.unpack_from('<IIQQQQIIQQ', data, shoff + size * i) for i in range(count)]
def file_offset(address, length=1):
    matches = [s[4] + address - s[3] for s in sections
               if s[1] != 8 and s[5] and s[3] <= address and address + length <= s[3] + s[5]]
    assert len(matches) == 1, (address, matches)
    return matches[0]
relocations = {}
for section in sections:
    if section[1] != 4:
        continue
    assert section[9] == 24
    for offset in range(section[4], section[4] + section[5], 24):
        address, info, addend = struct.unpack_from('<QQq', data, offset)
        if info & 0xffffffff == 1027:
            assert info >> 32 == 0
            relocations[address] = addend
slot = 0x383c0
descriptor = relocations[slot]
assert descriptor == 0x383e0
callbacks = [{'descriptor_offset': hex(i), 'function_va': hex(relocations[descriptor+i]),
              'function_file_offset': hex(file_offset(relocations[descriptor+i], 4))}
             for i in [0, 8, 0x10, 0x18, 0x20, 0x28, 0x48]]
name_offset = file_offset(descriptor + 0x55)
assert data[name_offset:name_offset + 4] == b'spo\0'
pos = metadata['pos']
blocks = []
block_count = struct.unpack_from('<I', blob, 0x18)[0]
for ordinal in range(block_count):
    start = pos
    assert pos + 42 <= len(blob)
    header = blob[pos:pos + 42]
    block_id = struct.unpack_from('<I', header)[0]
    out_size, in_size = struct.unpack_from('<QQ', header, 12)
    kind = header[28]
    auxiliary_count = struct.unpack_from('<I', header, 38)[0]
    pos += 42
    auxiliary_start = pos
    pos += 4 * auxiliary_count
    assert pos + in_size <= len(blob)
    payload = blob[pos:pos + in_size]
    if kind == 1:
        assert payload.startswith(bytes.fromhex('fd377a585a00'))
    blocks.append({'ordinal': ordinal, 'header_archive_offset': hex(start),
                   'header_carrier_file_offset': hex(0x870 + start),
                   'raw_header_hex': header.hex(), 'block_id': block_id,
                   'declared_output_bytes': out_size, 'encoded_bytes': in_size,
                   'kind': kind, 'auxiliary_count': auxiliary_count,
                   'auxiliary_sha256': hashlib.sha256(blob[auxiliary_start:pos]).hexdigest(),
                   'payload_archive_offset': hex(pos),
                   'payload_prefix_hex': payload[:12].hex(),
                   'payload_sha256': hashlib.sha256(payload).hexdigest()})
    pos += in_size
assert pos == len(blob)
print(json.dumps({'scope': 'static descriptors and framing only; no decoded bodies',
                  'descriptor_slot_va': hex(slot),
                  'descriptor_slot_file_offset': hex(file_offset(slot, 8)),
                  'descriptor_va': hex(descriptor),
                  'descriptor_file_offset': hex(file_offset(descriptor, 0x59)),
                  'callbacks': callbacks, 'block_count': block_count,
                  'kind_counts': {str(k): sum(x['kind'] == k for x in blocks)
                                  for k in sorted({x['kind'] for x in blocks})},
                  'final_archive_offset': hex(pos), 'blocks': blocks}, indent=2))
