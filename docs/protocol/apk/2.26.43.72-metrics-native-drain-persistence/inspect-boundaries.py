"""Bind selected worker/persistence boundaries to the recovered pinned image."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import runpy
import struct
import sys
source = Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(source))
data, sections, file_offset = evidence['data'], evidence['sections'], evidence['file_offset']
windows = [('scheduler',0x7239ec,0x723a40),('pending-worker',0x723a40,0x724fc8),
           ('async-thunk',0xbd8ed8,0xbd8ee0),('snapshot-wrapper',0x724fc8,0x725034),
           ('metadata-write',0x6fdca0,0x6fdf24),('file-write',0x734210,0x734334),
           ('stdio-flush',0x734334,0x7343a4),('event-clock',0x72c4fc,0x72c548)]
rows = []
for name,start,end in windows:
    offset = file_offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(offset),
                 'bytes':end-start,'sha256':hashlib.sha256(data[offset:offset+end-start]).hexdigest()})
pc = 0xbd8edc
instruction = struct.unpack_from('<I',data,file_offset(pc,4))[0]
assert instruction & 0xfc000000 == 0x14000000
immediate = instruction & 0x3ffffff
if immediate & 0x2000000: immediate -= 0x4000000
assert pc + 4*immediate == 0x723a40
strings = []
for address in [0x1536ee,0x1710b6,0x17132e,0x151bd2,0x15d372]:
    offset = file_offset(address)
    strings.append({'virtual_address':hex(address),'file_offset':hex(offset),
                    'value':data[offset:data.index(0,offset)].decode('ascii')})
clock = []
for section in sections:
    if section[1] != 4: continue
    for offset in range(section[4],section[4]+section[5],24):
        address,info,addend = struct.unpack_from('<QQq',data,offset)
        if address != 0xc27d88: continue
        symbols = sections[section[6]]
        names = sections[symbols[6]]
        string_offset = struct.unpack_from('<I',data,symbols[4]+(info>>32)*symbols[9])[0]
        position = names[4]+string_offset
        name = data[position:data.index(0,position)].decode('ascii')
        assert info & 0xffffffff == 1025 and name == 'kMCFAbsoluteTimeIntervalSince1970'
        clock.append({'slot_va':hex(address),'type':1025,'symbol':name,'addend':addend})
assert len(clock) == 1
print(json.dumps({'scope':'static selected boundaries only; no metrics execution',
                  'windows':rows,'async_thunk_destination':'0x723a40',
                  'strings':strings,'clock_offset_relocation':clock[0]},indent=2))
