"""Bind recovery callback consumer and loader scheduling windows to pinned ELF."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset'];rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);pos=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(pos),bytes=b-a,sha256=hashlib.sha256(data[pos:pos+b-a]).hexdigest()))
pc=0xbd8f34;ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
imm=ins&0x3ffffff
if imm&0x2000000:imm-=0x4000000
assert pc+imm*4==0x726660
pc=0x6f977c;ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0x9f000000==0x10000000
imm=(((ins>>5)&0x7ffff)<<2)|((ins>>29)&3)
if imm&0x100000:imm-=0x200000
assert pc+imm==0x722ce0
print(json.dumps({'scope':'static selected loader scheduling, consumer checksum and task-handoff chain; not observed startup or replay',
 'loader_thunk_target':'0x726660','consumer_function_target':'0x722ce0','windows':rows},indent=2))
