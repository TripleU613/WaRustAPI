"""Resolve native timer/status windows, thunks and status diagnostic labels."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def string(a):
 start=offset(a);return data[start:data.index(0,start)].decode('ascii')
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
for pc,target in [(0xbd8fcc,0x72acd8),(0xbd916c,0x7361cc)]:
 ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
 imm=ins&0x3ffffff
 if imm&0x2000000:imm-=0x4000000
 assert pc+4*imm==target
print(json.dumps({'scope':'static connection-status and repeating scheduler timer; no real queues or recovery execution',
 'credential_callback_target':'0x72acd8','generic_timer_event_target':'0x7361cc',
 'selected_interval':30.0,'interval_diagnostic_unit':'seconds','constructor_repeat_byte':1,
 'strings':[{'va':hex(a),'value':string(a)} for a in [0x144d70,0x12464d,0x1455f4,0xc26ff,0xc6cc4,0xd6170,0xec9fa,0x1221cd,0xfd5a8]],
 'windows':rows},indent=2))
