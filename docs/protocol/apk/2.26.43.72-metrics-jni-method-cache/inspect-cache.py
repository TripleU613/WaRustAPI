"""Resolve JNI_OnLoad, name/signature tables and mapped byte digests."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def string(a):
 start=offset(a);end=data.index(0,start)
 return data[start:end].decode('ascii')
methods=[]
for i in range(29):
 n=e['relocations'][0xc076e0+i*8];s=e['relocations'][0xc077c8+i*8]
 methods.append({'index':i,'name_slot_va':hex(0xc076e0+i*8),'signature_slot_va':hex(0xc077c8+i*8),
                 'name':string(n),'signature':string(s),'cache_slot_va':hex(0xc46310+i*8+0x4e0)})
assert methods[0]['name']=='jnidispatchIO' and methods[0]['signature']=='(ILjava/lang/Object;)J'
assert methods[0]['cache_slot_va']=='0xc467f0'
symbols=[]
for section in e['sections']:
 if section[1]!=11:continue
 assert section[9]==24
 names=e['sections'][section[6]]
 for pos in range(section[4],section[4]+section[5],24):
  n,info,other,index,value,size=struct.unpack_from('<IBBHQQ',data,pos)
  start=names[4]+n;name=data[start:data.index(0,start)].decode('ascii')
  if name=='JNI_OnLoad':symbols.append({'name':name,'entry_va':hex(value),'size':size})
assert symbols==[{'name':'JNI_OnLoad','entry_va':'0x651ca8','size':6704}]
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 assert 0x651ca8<=a<b<=0x651ca8+6704 or w['name']=='outbound-cache-use'
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
assert string(0x1122cd)=='com/whatsapp/wamsys/JniBridge'
print(json.dumps({'scope':'static JNI binding, not observed Java initialization',
 'class_name':string(0x1122cd),'class_lookup_slot_va':'0xc44bc0','outbound_class_slot_va':'0xc44bd0',
 'all_lookup_flag_va':'0xc4435c','symbols':symbols,'methods':methods,'windows':rows},indent=2))
