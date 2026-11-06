"""Pin registration and stack-walk evidence to the ARM64 source ELF."""
import contextlib,hashlib,io,json,runpy
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-media-body-provider/inspect-native-tuples.py'))
data,rel,off,string=(e[k] for k in ('data','rel','off','string'))
classname='com/crossapp/tigonhttp/TigonHttpClient$TigonHttpClientServiceHolder'
assert len(classname)==67
assert data[off(0xf5d26,67):off(0xf5d26,67)+67]==classname.encode()
methods=[]
for i in range(10):
    slot=0x74d260+i*24
    methods.append(dict(slot_va=hex(slot),name=string(rel[slot]),signature=string(rel[slot+8]),function_va=hex(rel[slot+16])))
assert methods[0]['name']=='initHybrid' and methods[0]['function_va']=='0x4d6344'
assert 'NoiseChannelManagerHolder;' not in methods[0]['signature']
assert rel[0x76c970+0x30]==0x4f4aa8
assert rel[0x76c9d0+0x30]==0x4f39a8
assert rel[0x76ca30+0x30]==0x4f385c
windows=[]
for name,(a,z) in json.loads((Path(__file__).parent/'native-windows.json').read_text()).items():
    a,z=int(a,16),int(z,16);o=off(a,z-a)
    windows.append(dict(name=name,start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest()))
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),registered_class=classname,registration_class_va='0xf5d26',methods=methods,stage_indices=dict(priority=(0x970-0x700)//0x30,preceding=(0x9a0-0x700)//0x30,base=(0x9d0-0x700)//0x30),windows=windows,scope='Static registration and unconditional stage invocation; actual loaded runtime and property3 producer unobserved'),indent=2))
