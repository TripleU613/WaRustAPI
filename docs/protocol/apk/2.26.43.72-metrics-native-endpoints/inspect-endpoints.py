"""Bind endpoint windows, JNI provider slot and selected decoded Java constants."""
import contextlib,hashlib,io,json,re,runpy
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def string(a):
 start=offset(a);return data[start:data.index(0,start)].decode('ascii')
index=26
name=string(e['relocations'][0xc076e0+index*8])
signature=string(e['relocations'][0xc077c8+index*8])
assert name=='jnidispatchO' and signature=='(I)Ljava/lang/Object;'
assert 0xc467f0+index*8==0xc468c0
source=(root/'endpoint-constants-excerpt.txt').read_text()
raw=re.findall(r'const-string(?:/jumbo)?  v0, (".*")',source)
assert len(raw)==2
values=[]
for literal in raw:
 literal=re.sub(r'\\u\{([0-9a-fA-F]+)\}',lambda m:'\\u'+m[1].zfill(4),literal)
 encoded=json.loads(literal)
 values.append(''.join(chr(ord(c)^0x12) for c in encoded))
assert values==['https://','/deidentified_telemetry']
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
print(json.dumps({'scope':'static native/Java endpoint construction, not observed endpoints or live transmission',
 'provider':{'method_index':index,'method':name,'signature':signature,'cache_va':'0xc468c0','selector':0},
 'java_prefix':values[0],'java_suffix':values[1],'fallback_native_string_selector':22,
 'strings':[{'va':hex(a),'value':string(a)} for a in [0xf29cb,0xfd458,0xfb2ee,0xb28f2,0xfa6bd,0x1468e4]],
 'windows':rows},indent=2))
