"""Bind credential persistence windows, raw layout and diagnostic literals."""
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
callers=[]
for s in e['sections']:
 if not s[2]&4:continue
 for i,(ins,) in enumerate(struct.iter_unpack('<I',data[s[4]:s[4]+s[5]])):
  if ins&0xfc000000!=0x94000000:continue
  pc=s[3]+i*4;imm=ins&0x3ffffff
  if imm&0x2000000:imm-=0x4000000
  if pc+4*imm==0x6fe6a0:callers.append(hex(pc))
assert callers==['0x729684','0x72a7c4']
print(json.dumps({'scope':'static local credential persistence, not account/message replay or live I/O',
 'direct_path_helper_callers':callers,'reader_requested_bytes':537,
 'header_bytes':25,'header_version':2,
 'header_layout':[{'offset':0,'bytes':1,'field':'version'},
 {'offset':1,'bytes':4,'field':'signed remaining-use count'},
 {'offset':5,'bytes':8,'field':'signed raw expiry'},
 {'offset':13,'bytes':4,'field':'first data length'},
 {'offset':17,'bytes':4,'field':'second data length'},
 {'offset':21,'bytes':4,'field':'optional third data length'}],
 'strings':[{'va':hex(a),'value':string(a)} for a in [0x133fe7,0x127ceb,0x127caa,0x112da0,0x127c0c,0x10afee]],
 'windows':rows},indent=2))
