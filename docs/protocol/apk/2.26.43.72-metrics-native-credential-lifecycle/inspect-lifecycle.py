"""Resolve pinned credential boundaries, switch bytes and log labels."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def raw(a,n):
    start=offset(a,n);return data[start:start+n]
def string(a):
    start=offset(a);end=data.index(0,start)
    return raw(a,end-start).decode('ascii')
windows=[('refresh-scheduler',0x728b24,0x728bfc),('credential-observer-scheduler',0x728bfc,0x728c54),('refresh-worker',0x7298b8,0x729cd0),('credential-completion',0x729cd0,0x72a740),('temporary-cleanup',0x72ad9c,0x72add8),('credential-constructor-setter',0x7212f4,0x721388),('constructor-arguments',0x49b830,0x49b83c),('clock',0x72c4fc,0x72c548),('clock-outline',0xbe8838,0xbe8860),('refresh-thunks',0xbd8fb0,0xbd8fc0),('provider-predicate',0x6fef18,0x6fef40),('scheduler-arguments',0xbe4704,0xbe4710)]
rows=[{'name':name,'start_va':hex(a),'end_va':hex(b),'file_offset':hex(offset(a,b-a)),'bytes':b-a,'sha256':hashlib.sha256(raw(a,b-a)).hexdigest()} for name,a,b in windows]
for pc,target in [(0xbd8fb4,0x729540),(0xbd8fbc,0x7298b8)]:
    ins=struct.unpack('<I',raw(pc,4))[0];assert ins&0xfc000000==0x14000000
    imm=ins&0x3ffffff
    if imm&0x2000000:imm-=0x4000000
    assert pc+4*imm==target
switch=raw(0x2e962e,5)
print(json.dumps({'scope':'static native metrics credential lifecycle; no provider or server execution',
 'constructor_raw_lifetime_increment':172800,'constructor_use_counter':64,
 'selected_mode_zero_counter_override':180,
 'completion_switch':{'table_va':'0x2e962e','bytes_hex':switch.hex(),'destinations':{str(i):hex(0x729d7c+v*4) for i,v in enumerate(switch)}},
 'strings':[{'va':hex(a),'value':string(a)} for a in [0x16dc37,0xf9dcb,0xe99d5,0xaf8bd,0x9f578,0xf9d99,0xe5b10,0xd480c,0xe7119,0x10aa2d,0x10a9d6,0x10aa8e]],'windows':rows},indent=2))
