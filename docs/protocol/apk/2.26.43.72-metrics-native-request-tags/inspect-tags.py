"""Resolve digest-bound tag literals, selector tables and mapped windows."""
import contextlib,hashlib,io,json,runpy
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def string(a):
 start=offset(a);return data[start:data.index(0,start)].decode('ascii')
tables=[]
for address,count,base in [(0x2e9638,4,0x72cb30),(0x2e963c,8,0x72cbf0)]:
 values=data[offset(address,count):offset(address,count)+count]
 tables.append({'table_va':hex(address),'bytes_hex':values.hex(),'destinations':{str(i):hex(base+v*4) for i,v in enumerate(values)}})
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
print(json.dumps({'scope':'static request-tag and metadata literals; not observed request execution',
 'tables':tables,'strings':[{'va':hex(a),'value':string(a)} for a in [0x2fd905,0x109977,0x115f0b,0x12bf86,0x8a5c5,0x9fd5b,0xc3713,0xc3702,0xf7dc4,0xf7dc7,0xf779f,0x33bc60,0x16cecc,0x16dc35,0x112ee6,0xc73dc,0xf6592,0xa57b2,0x12b5cd,0x318a4e,0x318a48,0x173673,0x1735d0,0x15269d,0x33bbc2,0x33bbc8,0x890e8,0x89110]],
 'windows':rows},indent=2))
