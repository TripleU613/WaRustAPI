"""Bind metrics error keys, prefix, numeric branch and helper bytes to pinned ELF."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import runpy
import struct
source=Path(__file__).resolve().parent.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):e=runpy.run_path(str(source))
data,offset,rel=e['data'],e['file_offset'],e['relocations']
strings=[]
for obj,want in [(0xc3b6c0,'error'),(0xc3b6f0,'error_subcode'),(0xc3b720,'Error Content = ')]:
    address=rel[obj+0x28];pos=offset(address)
    value=data[pos:data.index(0,pos)].decode('ascii');assert value==want
    strings.append({'object_va':hex(obj),'pointer_slot_va':hex(obj+0x28),
                    'string_va':hex(address),'file_offset':hex(pos),'value':value})
windows=[('response-worker',0x728e18,0x729410),('regular-error-parser',0x72943c,0x729540),
         ('number-extraction',0x71dc40,0x71dc90),('worker-cast-arguments',0xbe4e24,0xbe4e30),
         ('worker-substring-arguments',0xbe19a0,0xbe19ac),('body-cast-arguments',0xbe13d8,0xbe13e4),
         ('body-input-arguments',0x5d0084,0x5d0090)]
rows=[]
for name,start,end in windows:
    pos=offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(pos),'bytes':end-start,
                 'sha256':hashlib.sha256(data[pos:pos+end-start]).hexdigest()})
# Retain the exact instructions independently from the disassembly text.
branch=data[offset(0x729140,20):offset(0x729140,20)+20]
assert [hex(x) for x in struct.unpack('<5I',branch)]==['0x514bb008','0x5131e908','0x11000909','0x71000d1f','0x1a963136']
print(json.dumps({'scope':'static keys and branch bytes; no observed server semantics',
 'recognized_subcodes':[3067002,3067003,3067004],'raw_results':[2,3,4],
 'strings':strings,'windows':rows},indent=2))
