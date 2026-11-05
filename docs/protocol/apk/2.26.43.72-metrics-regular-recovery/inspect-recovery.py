"""Hash regular recovery windows and decode the timer callback branch."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);pos=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(pos),bytes=b-a,sha256=hashlib.sha256(data[pos:pos+b-a]).hexdigest()))
pc=0xbd8f8c;ins=struct.unpack_from('<I',data,offset(pc,4))[0]
assert ins&0xfc000000==0x14000000
imm=ins&0x3ffffff
if imm&0x2000000:imm-=0x4000000
assert pc+imm*4==0x727908
print(json.dumps({'scope':'static regular recovery windows, not complete callback registration',
 'timer_thunk_target':'0x727908','windows':rows},indent=2))
