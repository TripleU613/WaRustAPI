"""Bind recovered metrics file loader and distinguish bulk deletion."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset'];rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);pos=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(pos),bytes=b-a,sha256=hashlib.sha256(data[pos:pos+b-a]).hexdigest()))
strings=[]
for a,want in [(0x2fc21b,'wamt-a'),(0x2fc222,'wamt-o'),(0xcaafa,'%s/%s')]:
 pos=offset(a);value=data[pos:data.index(0,pos)].decode('ascii');assert value==want
 strings.append({'virtual_address':hex(a),'file_offset':hex(pos),'value':value})
callers=[]
for s in e['sections']:
 if not s[2]&4:continue
 for i,(ins,) in enumerate(struct.iter_unpack('<I',data[s[4]:s[4]+s[5]])):
  if ins&0xfc000000!=0x94000000:continue
  pc=s[3]+i*4;imm=ins&0x3ffffff
  if imm&0x2000000:imm-=0x4000000
  if pc+4*imm==0x6fe1a8:callers.append(hex(pc))
print(json.dumps({'scope':'static file loader, header admission and separate bulk deletion; no full runtime recovery proof',
 'loader_direct_bl_callers':callers,'strings':strings,'windows':rows},indent=2))
