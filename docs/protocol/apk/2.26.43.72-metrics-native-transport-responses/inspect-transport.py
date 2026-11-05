"""Bind native transport callback/status boundaries to the recovered pinned image."""
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
data, file_offset = evidence['data'], evidence['file_offset']
relative = evidence['relocations']
def string(address):
    offset = file_offset(address)
    return data[offset:data.index(0,offset)].decode('ascii')
windows = [('buffer-sender',0x7280dc,0x728b24),('completion-adapter',0x728c54,0x728e18),
           ('response-worker',0x728e18,0x729410),('response-worker-thunk',0xbd8fa8,0xbd8fb0),
           ('has-next-endpoint',0x729410,0x72943c),('regular-error-parser',0x72943c,0x729540),
           ('task-bridge',0x7345a4,0x734900),('regular-task-callback',0x7275d0,0x727740)]
rows = []
for name,start,end in windows:
    offset = file_offset(start,end-start)
    rows.append({'name':name,'virtual_address':hex(start),'file_offset':hex(offset),
                 'bytes':end-start,'sha256':hashlib.sha256(data[offset:offset+end-start]).hexdigest()})
strings = []
for address in [0x8b9b7,0xaf73d,0xaf6f0,0xa3add,0x118534,0xf5667,0xced4d,0xf0030]:
    strings.append({'virtual_address':hex(address),'file_offset':hex(file_offset(address)),
                    'value':string(address)})
outcomes = []
for index in range(4):
    slot = 0xc0aae0+index*8
    address = relative[slot]
    outcomes.append({'code':index,'slot_va':hex(slot),'string_va':hex(address),'name':string(address)})
assert [x['name'] for x in outcomes] == ['success','retryable','non-retryable','failure']
pc = 0xbd8fac
instruction = struct.unpack_from('<I',data,file_offset(pc,4))[0]
assert instruction & 0xfc000000 == 0x14000000
immediate = instruction & 0x3ffffff
if immediate & 0x2000000: immediate -= 0x4000000
assert pc+4*immediate == 0x728e18
print(json.dumps({'scope':'static native metrics transport; no live requests/replies',
                  'ordinary_success_status':200,'distinct_status_branch':401,
                  'response_worker_thunk_target':'0x728e18','regular_callback_outcomes':outcomes,
                  'strings':strings,'windows':rows},indent=2))
