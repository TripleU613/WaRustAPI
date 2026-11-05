"""Bind native regular metrics job, Java dispatch cache and callback thunks."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
root=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path(str(root.parent/'2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data,offset=e['data'],e['file_offset']
def string(a):
 start=offset(a);return data[start:data.index(0,start)].decode('ascii')
assert string(e['relocations'][0xc076e0])=='jnidispatchIO'
assert string(e['relocations'][0xc077c8])=='(ILjava/lang/Object;)J'
rows=[]
for w in json.loads((root/'windows.json').read_text()):
 a,b=int(w['start_va'],16),int(w['end_va'],16);start=offset(a,b-a)
 rows.append(dict(w,file_offset=hex(start),bytes=b-a,sha256=hashlib.sha256(data[start:start+b-a]).hexdigest()))
for pc,target in [(0xbd8a6c,0x7275d0),(0xbd8b9c,0x6fea64)]:
 ins=struct.unpack_from('<I',data,offset(pc,4))[0];assert ins&0xfc000000==0x14000000
 imm=ins&0x3ffffff
 if imm&0x2000000:imm-=0x4000000
 assert pc+4*imm==target
print(json.dumps({'scope':'static regular metrics producer and outcome mapping, not live requests or Java execution',
 'jni_method':'jnidispatchIO(ILjava/lang/Object;)J','cache_va':'0xc467f0','selector':6,
 'native_result_mapping':{'java_bridge_true':0,'java_bridge_false':3},
 'declared_diagnostic_binary_bounds':[4,126976],'declared_diagnostic_time_bounds':[0x5e0c5180,0xf486c780],
 'diagnostic_assert_throws':False,'callback_delay_ms':32000,'windows':rows},indent=2))
