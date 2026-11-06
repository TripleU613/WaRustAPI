import contextlib,io,runpy,json,sys
with contextlib.redirect_stdout(io.StringIO()):e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,ss,rel,off,string=[e[k] for k in ['data','ss','rel','off','string']]
rows=[]
for name in ['N8facebook5tigon23TigonLimitingBodyStreamE','N8facebook5tigon3mns18TigonMNSBodyStreamE','N8facebook5tigon18TigonHucBodyStreamE']:
 pos=data.index(name.encode()+b'\0');sects=[s for s in ss if s[1]!=8 and s[4]<=pos<s[4]+s[5]];assert len(sects)==1;s=sects[0];a=s[3]+pos-s[4]
 refs=[slot for slot,v in rel.items() if v==a]
 for ptr in refs:
  ti=ptr-8
  for slot,v in rel.items():
   if v!=ti:continue
   if int.from_bytes(data[off(slot-8,8):off(slot-8,8)+8],'little')!=0:continue
   if not all(slot+8+i*8 in rel and any(s[2]&4 and s[3]<=rel[slot+8+i*8]<s[3]+s[5] for s in ss) for i in range(7)):continue
   rows.append({'name':name,'name_va':hex(a),'typeinfo_va':hex(ti),'vtable_typeinfo_slot':hex(slot),'address_point':hex(slot+8),'functions':{hex(i*8):hex(rel[slot+8+i*8]) for i in range(7) if slot+8+i*8 in rel}})
assert [(r['name'],r['functions']['0x30']) for r in rows]==[('N8facebook5tigon23TigonLimitingBodyStreamE','0x4b1e3c'),('N8facebook5tigon3mns18TigonMNSBodyStreamE','0x517bc8'),('N8facebook5tigon18TigonHucBodyStreamE','0x64a364')]
windows=[]
import hashlib
for a,z in [(0x4b1e14,0x4b1e50),(0x517bc8,0x517c48),(0x64a364,0x64a448),(0x727afc,0x727b10),(0x72a0dc,0x72a0e8),(0x64e908,0x64ea50)]:
 start=off(a,z-a);windows.append({'start_va':hex(a),'end_va':hex(z),'bytes':z-a,'file_offset':hex(start),'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'scope':'Static RTTI and primary vtable candidates, not observed dynamic dispatch','vtables':rows,'windows':windows},indent=2))
