"""Bind native integer/field encoding helpers and dispatch table to pinned bytes."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import runpy
source = Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(source))
data, file_offset = evidence['data'], evidence['file_offset']
windows = [('field-encoder',0x72bf48,0x72c184),('signed-width-encoder',0x72c184,0x72c244),
           ('scalar-special-cases',0x72c244,0x72c260),('literal-zero-tag',0x483d04,0x483d0c),
           ('literal-one-tag',0x5d3a0c,0x5d3a14),('append-u16',0x72c958,0x72c9a8),
           ('append-byte',0x72c9a8,0x72c9f4),('patch-header',0x72c280,0x72c330),
           ('worker-encoder-calls',0x724330,0x7243c8)]
rows = []
for name,start,end in windows:
    offset = file_offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(offset),
                 'bytes':end-start,'sha256':hashlib.sha256(data[offset:offset+end-start]).hexdigest()})
position = file_offset(0x2e9633,5)
table = data[position:position+5]
print(json.dumps({'scope':'native local field serialization helpers; send/reply behavior unresolved',
                  'type_dispatch_table_va':'0x2e9633','type_dispatch_table_hex':table.hex(),
                  'type_dispatch_destinations':[hex(0x72bfd0+v*4) for v in table],
                  'integer_width_tags':{'1-byte':3,'2-byte':4,'4-byte':5,'8-byte':6},
                  'scalar_special_tags':{'zero':1,'one':2},'windows':rows},indent=2))
