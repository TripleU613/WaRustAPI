"""Bind selected native credential-IQ bridges to the pinned reconstructed image."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
assert e['data'][offset(0x2e9553+4)]==0x13
assert 0x6eeec0+0x13*4==0x6eef0c
for pc,target in [(0xbd8a74,0x729cd0),(0xbd8bac,0x6fec84)]:
 instruction=struct.unpack_from('<I',data,offset(pc,4))[0]
 assert instruction&0xfc000000==0x14000000
 imm=instruction&0x3ffffff
 if imm&0x2000000:imm-=0x4000000
 assert pc+4*imm==target
print(json.dumps({'scope':'static bridge and selector evidence; runtime Java method-cache binding remains untraced',
 'selector4_target':'0x6eef0c','direct_callback_target':'0x729cd0','queued_completion_target':'0x6fec84',
 'windows':rows},indent=2))
