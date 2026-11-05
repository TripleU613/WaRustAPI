"""Bind regular drain helper windows and asynchronous job thunk."""
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
pc=0xbd8bbc;ins=struct.unpack_from('<I',data,offset(pc,4))[0]
assert ins&0xfc000000==0x14000000
imm=ins&0x3ffffff
if imm&0x2000000:imm-=0x4000000
assert pc+4*imm==0x6fedf0
print(json.dumps({'scope':'static regular drain status, not server retry/backoff or persistent buffer ownership',
 'drain_flag_offset':'0x50','callback_table_offset':'0x40','status_slot_offset':'0x30',
 'error_domain_object_va':'0xc30518','local_error_code':3,'drain_job_target':'0x6fedf0','windows':rows},indent=2))
