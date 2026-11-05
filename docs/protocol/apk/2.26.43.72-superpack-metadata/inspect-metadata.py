#!/usr/bin/env python3
"""Inspect only the pinned archive metadata; never execute or unpack native code."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('apk', type=Path)
args = parser.parse_args()
with args.apk.open('rb') as stream:
    if hashlib.file_digest(stream, 'sha256').hexdigest() != '887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e':
        raise SystemExit('APK digest mismatch')
with zipfile.ZipFile(args.apk) as archive:
    carrier = archive.read('lib/arm64-v8a/libs.so')
    decoder = archive.read('lib/arm64-v8a/libsuperpack.so')
if hashlib.sha256(carrier).hexdigest() != '1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5':
    raise SystemExit('Carrier digest mismatch')
if hashlib.sha256(decoder).hexdigest() != 'fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f':
    raise SystemExit('Decoder digest mismatch')
# Carrier section/symbol mapping retained in the carrier bundle.
file_base, length = 0x870, 0xdb6641
blob = carrier[file_base:file_base + length]
assert len(blob) == length
assert struct.unpack_from('<Q', blob)[0] == 0x05abaf0150c00100
assert struct.unpack_from('<I', blob, 8)[0] == 0x81
member_count = struct.unpack_from('<I', blob, 12)[0]
# 0x1c8dc advances 28 bytes; 0x2d560 reads another 28 plus count*4.
meta = 28
aux_count = struct.unpack_from('<I', blob, meta + 2)[0]
consumed = 28 + aux_count * 4
assert meta + consumed <= len(blob)
pos = meta + consumed
records = []
previous_type = None
for ordinal in range(member_count):
    begin = pos
    end = blob.index(0, pos, min(pos + 0x1001, len(blob)))
    name = blob[pos:end].decode('ascii')
    pos = end + 1
    end = blob.index(0, pos, min(pos + 9, len(blob)))
    raw_type = blob[pos:end].decode('ascii')
    pos = end + 1
    assert pos + 16 <= len(blob)
    size, opaque = struct.unpack_from('<QQ', blob, pos)
    pos += 16
    # 0x1ca20..0x1ca88 reuses previous nonempty type for empty entries.
    effective_type = raw_type if raw_type or previous_type is None else previous_type
    previous_type = effective_type
    records.append({'ordinal': ordinal, 'archive_offset': hex(begin),
                    'carrier_file_offset': hex(file_base + begin), 'name': name,
                    'raw_type': raw_type, 'effective_type': effective_type,
                    'declared_size': size, 'opaque_u64': hex(opaque)})
print(json.dumps({'scope': 'metadata only; no decoded members or payload validation',
                  'archive_bytes': length,
                  'header_words_u32le': list(struct.unpack_from('<7I', blob)),
                  'auxiliary_count': aux_count, 'auxiliary_bytes_consumed': consumed,
                  'member_count': member_count, 'members': records,
                  'payload_archive_offset': hex(pos),
                  'payload_carrier_file_offset': hex(file_base + pos)}, indent=2))
