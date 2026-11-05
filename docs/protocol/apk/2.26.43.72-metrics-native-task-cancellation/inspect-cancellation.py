"""Bind native cancellation/destructor windows and callback thunk destinations."""
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
for pc,target in [(0xbd8f9c,0x727a10),(0xbd8fa4,0x727b28)]:
 ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
 imm=ins&0x3ffffff
 if imm&0x2000000:imm-=0x4000000
 assert pc+4*imm==target
callers=[]
for s in e['sections']:
 if not s[2]&4:continue
 for i,(ins,) in enumerate(struct.iter_unpack('<I',data[s[4]:s[4]+s[5]])):
  if ins&0xfc000000!=0x94000000:continue
  pc=s[3]+i*4;imm=ins&0x3ffffff
  if imm&0x2000000:imm-=0x4000000
  if pc+4*imm==0x727bd0:callers.append(hex(pc))
print(json.dumps({'scope':'static native task cancellation and destructor; not observed disconnect/delivery or replay authority',
 'direct_cancel_helper_callers':callers,'cancel_callback_target':'0x727a10','destructor_target':'0x727b28',
 'queued_callback_table_slot':'0x28','active_task_dictionary_offset':'0x60','endpoint_dictionary_offset':'0x68','windows':rows},indent=2))
