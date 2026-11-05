"""Bind selected completion-to-file-removal chain and path literals."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset'];rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);pos=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(pos),bytes=b-a,sha256=hashlib.sha256(data[pos:pos+b-a]).hexdigest()))
pc=0xbd8f2c;ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
imm=ins&0x3ffffff
if imm&0x2000000:imm-=0x4000000
assert pc+imm*4==0x726580
pc=0xbd8f4c;ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
imm=ins&0x3ffffff
if imm&0x2000000:imm-=0x4000000
assert pc+imm*4==0x7269f8
strings=[]
for a,want in [(0x134028,'%s/%s-%d-%d-%d'),(0x2fc21b,'wamt-a'),(0x2fc222,'wamt-o')]:
 pos=offset(a);value=data[pos:data.index(0,pos)].decode('ascii');assert value==want
 strings.append({'virtual_address':hex(a),'file_offset':hex(pos),'value':value})
print(json.dumps({'scope':'static selected construction and completion/deletion chain; full engine and storage lifecycle not covered',
 'deletion_thunk_target':'0x726580','path_strings':strings,'windows':rows},indent=2))
