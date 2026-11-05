"""Bind the static validator, group selector and accessors to pinned bytes."""
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
windows = [('validator',0x71fc84,0x71fd70),('selector',0x71fd70,0x71fdf4),
           ('default-snapshot-constructor',0x71fb58,0x71fc34),
           ('recovery-validation-caller',0x72b438,0x72b4c8),
           ('group-a-getter',0x71de74,0x71de90),('group-b-getter',0x71f6f8,0x71f714),
           ('group-c-getter',0x71fa30,0x71fa4c),('group-d-getter',0x71fa6c,0x71fa8c),
           ('group-slot-30',0x720890,0x7208c4),('group-slot-18',0x7207f4,0x720828),
           ('group-slot-20',0x720828,0x72085c),('group-slot-28',0x72085c,0x720890),
           ('validator-init-outline',0xbec370,0xbec37c),
           ('validator-selector-outline',0x5d09d4,0x5d09e0),
           ('a-field-outline',0xbe58ac,0xbe58b4),('b-field-outline',0xbe5898,0xbe58a0),
           ('c-field-outline',0xbe829c,0xbe82a4),
           ('default-args-1',0xbe3174,0xbe3180),('default-args-2',0xbe0568,0xbe0574)]
rows = []
for name,start,end in windows:
    offset = file_offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(offset),
                 'bytes':end-start,'sha256':hashlib.sha256(data[offset:offset+end-start]).hexdigest()})
position = file_offset(0x2e2c59,4)
table = data[position:position+4]
assert table == bytes.fromhex('000c0408')
print(json.dumps({'scope':'static local snapshot validation; not server response codes',
                  'selector_table_va':'0x2e2c59','selector_table_hex':table.hex(),
                  'selector_destinations':[hex(0x71fda0+v*4) for v in table],
                  'state_group_pointer_offsets':['0x30','0x28','0x18','0x20'],
                  'raw_fields':['0x18','0x1c','0x20','0x24'],
                  'accepted_signed_ranges':[[0,2147483647],[0,2147483647],[1,65535],[1,65535]],
                  'failure_codes':{'A_negative':9,'B_negative':10,'C_below_one':11,
                                   'C_at_least_65536':12,'D_below_one':13,'D_at_least_65536':14},
                  'windows':rows},indent=2))
