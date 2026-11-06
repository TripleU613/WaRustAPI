"""Verify the static priority header chain against pinned ELF bytes."""
import contextlib,hashlib,io,json,runpy
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-media-body-provider/inspect-native-tuples.py'))
data,rel,off,string=(e[k] for k in ('data','rel','off','string'))
slot=0x74d260
assert string(rel[slot])=='initHybrid'
signature=string(rel[slot+8]);assert 'HttpPriorityCalculatorProvider;' in signature
assert 'NoiseChannelManagerHolder;' not in signature
assert rel[slot+16]==0x4d6344
assert string(0x14d0a0)=='com/facebook/tigon/requestprioritization/calculatorprovider/HttpPriorityCalculatorProvider'
vt=[]
for table,needle,method in [(0x76c430,'buildFullTigonHTTPClientStack',0x4f6864),(0x76c970,'buildFullTigonHTTPClientStack',0x4f4aa8),(0x77a3a0,'TigonHttpPrioritizationService',0x661408)]:
    name=string(rel[rel[table-8]+8]);assert needle in name
    offset=0x10 if table==0x77a3a0 else 0x30
    assert rel[table+offset]==method
    vt.append(dict(vtable_va=hex(table),type_name=name,slot=hex(offset),function_va=hex(method)))
assert string(rel[rel[0x76a320-8]+8]).find('HttpPriorityCalculator')>=0
literals={hex(a):string(a) for a in [0xcbdf5,0xf5308,0xe0154,0xdaf4e,0xf5351,0xd6f87]}
assert list(literals.values())==['Priority','u=',', i',', p',', r=','DefaultHttpPriorityContextInterceptor']
windows=[]
for name,(a,z) in json.loads((Path(__file__).parent/'native-windows.json').read_text()).items():
    a,z=int(a,16),int(z,16);o=off(a,z-a)
    windows.append(dict(name=name,start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest()))
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),jni_slot=hex(slot),jni_signature=signature,vtable_bindings=vt,literals=literals,windows=windows,scope='Static header construction; registration, graph activation and actual wire observation remain separate'),indent=2))
