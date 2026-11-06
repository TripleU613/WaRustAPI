"""Check priority-context bindings against the pinned ARM64 ELF, never execute it."""
import contextlib,hashlib,io,json,runpy,sys
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-media-body-provider/inspect-native-tuples.py'))
data,rel,off,string=(e[k] for k in ('data','rel','off','string'))
rows=[]
for slot,signature,function in [(0x756f40,'(Z)Lcom/facebook/jni/HybridData;',0x65fc1c),(0x756fd0,'()Lcom/facebook/jni/HybridData;',0x65fe28)]:
    assert (string(rel[slot]),string(rel[slot+8]),rel[slot+16])==('initHybrid',signature,function)
    rows.append(dict(slot_va=hex(slot),name='initHybrid',signature=signature,function_va=hex(function)))
for address,name in [(0xf5720,'com/whatsapp/infra/tigon/WAHttpPriorityContextInterceptor'),(0xf5921,'com/whatsapp/infra/tigon/WAHttpPriorityCalculatorHolder')]:
    n=len(name);assert data[off(address,n):off(address,n)+n]==name.encode()
assert string(0xd6fad)=='WAHttpPriorityContextInterceptor'
for a,b in [(0x77a108,0x757030),(0x757038,0x168a74),(0x77a120,0x65ff28),(0x77a138,0x757058),(0x757060,0x168b9d),(0x77a170,0x660000),(0x77a1d8,0x757088),(0x757090,0x168d29),(0x77a0f0,0x65fd6c),(0x77a0a8,0x756fa0),(0x77a0c0,0x65fd28),(0x77a0d8,0x756fb8)]:assert rel[a]==b,(hex(a),hex(b))
assert 'JWAHttpPriorityContextInterceptor' in string(0x168a74)
assert 'makeInterceptor' in string(0x168b9d)
assert string(0x168d29)=='N8whatsapp5tigon21WAHttpPriorityContextE'
windows=[]
for name,(a,z) in json.loads((Path(__file__).parent/'native-windows.json').read_text()).items():
    a,z=int(a,16),int(z,16);o=off(a,z-a)
    windows.append(dict(name=name,start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest()))
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),jni=rows,windows=windows,scope='Static relocation/RTTI bindings, no runtime activation or wire observation'),indent=2))
