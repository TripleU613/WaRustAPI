"""Resolve pinned metadata path/modes and direct branch references, offline."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import runpy
import struct
source = Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(source))
data, sections, file_offset = evidence['data'], evidence['sections'], evidence['file_offset']
relative = evidence['relocations']
def string(address):
    offset = file_offset(address)
    return data[offset:data.index(0,offset)].decode('ascii')
modes = []
for index in range(3):
    slot = 0xc0c010 + index*8
    address = relative[slot]
    modes.append({'index':index,'slot_va':hex(slot),'slot_file_offset':hex(file_offset(slot,8)),
                  'string_va':hex(address),'value':string(address)})
assert [x['value'] for x in modes] == ['rb','wb','ab']
refs = {0x6fdb7c:[],0x6fdca0:[],0x724fc8:[]}
for section in sections:
    if section[1] != 1 or not section[2] & 4: continue
    for offset in range(section[4],section[4]+section[5]-3,4):
        instruction = struct.unpack_from('<I',data,offset)[0]
        opcode = instruction & 0xfc000000
        if opcode not in [0x14000000,0x94000000]: continue
        immediate = instruction & 0x3ffffff
        if immediate & 0x2000000: immediate -= 0x4000000
        pc = section[3]+offset-section[4]
        target = pc+4*immediate
        if target in refs:
            refs[target].append({'va':hex(pc),'file_offset':hex(offset),'opcode':'BL' if opcode==0x94000000 else 'B'})
header_offset = file_offset(0x18a190,4)
assert data[header_offset:header_offset+4] == b'MTD\x04'
assert string(0x15d33d) == '/wam-metadata' and string(0x165e0f) == 'MTD'
windows = [('snapshot-creation',0x71fdf4,0x71fe70),('path-suffix',0x6fdc64,0x6fdca0),
           ('metadata-file-opener',0x6fdb7c,0x6fdc18),('file-opener',0x733ce0,0x733df0),
           ('metadata-recovery',0x72b1d8,0x72b4c8),('normal-worker-save',0x724280,0x7242d4),
           ('other-normal-save',0x725350,0x725364),('recovery-mode',0x5ce230,0x5ce23c),
           ('write-mode',0x5cf848,0x5cf854),('opener-register-flow',0xbe8448,0xbe8454),('rewind-before-write',0x7343a4,0x734448),('rewind-arguments',0xbe06b0,0xbe06bc),('path-register-flow',0xbe4704,0xbe4710),('url-register-flow',0xbe1aa8,0xbe1ab4)]
rows = []
for name,start,end in windows:
    offset = file_offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(offset),
                 'bytes':end-start,'sha256':hashlib.sha256(data[offset:offset+end-start]).hexdigest()})
print(json.dumps({'scope':'static path/mode/layout evidence; direct branches only, not indirect callers',
                  'path_suffix':string(0x15d33d),'path_suffix_va':'0x15d33d',
                  'magic':string(0x165e0f),'magic_va':'0x165e0f',
                  'record_bytes':0x69c,'version_byte':4,'writer_header_constant_va':'0x18a190','writer_header_bytes_hex':data[header_offset:header_offset+4].hex(),'mode_table':modes,
                  'direct_references':{hex(k):v for k,v in refs.items()},'windows':rows},indent=2))
