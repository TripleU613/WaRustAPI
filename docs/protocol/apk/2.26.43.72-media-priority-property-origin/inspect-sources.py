"""Verify reconstructed native inputs and scoped symbol/call searches."""
import hashlib,json,re,struct,subprocess,sys
from pathlib import Path
base=Path(__file__).resolve().parent
libdir=Path(sys.argv[1])
prior=json.loads((base.parent/'2.26.43.72-superpack-reconstructed-members/validation.json').read_text())
inputs=[];positive=[]
for m in prior:
    data=(libdir/m['name']).read_bytes();digest=hashlib.sha256(data).hexdigest()
    assert len(data)==m['bytes'] and digest==m['sha256'],m['name']
    inputs.append(dict(name=m['name'],bytes=len(data),sha256=digest))
    if b'TigonPropertyContainer11setProperty' in data or b'TigonPropertyContainer5Group11setProperty' in data:positive.append(m['name'])
windows=[]
for name,(lib,a,z) in json.loads((base/'native-windows.json').read_text()).items():
    data=(libdir/lib).read_bytes();a,z=int(a,16),int(z,16)
    shoff=struct.unpack_from('<Q',data,40)[0];size,count=struct.unpack_from('<HH',data,58);assert size==64
    sections=[struct.unpack_from('<IIQQQQIIQQ',data,shoff+size*i) for i in range(count)]
    offsets=[s[4]+a-s[3] for s in sections if s[1]!=8 and s[5] and s[3]<=a and z<=s[3]+s[5]];assert len(offsets)==1
    o=offsets[0];windows.append(dict(name=name,library=lib,start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest()))
symbols={}
for name in positive+['libar-bundle3.so']:
    output=subprocess.check_output(['llvm-nm','--dynamic','--demangle',str(libdir/name)],text=True)
    symbols[name]=[line for line in output.splitlines() if 'TigonPropertyContainer' in line and 'setProperty' in line]
print(json.dumps(dict(scope='83 pinned reconstructed members; symbol-string/import scan only, not all code or runtime libraries',inputs=inputs,setter_symbol_string_matches=positive,dynamic_setter_symbols=symbols,windows=windows),indent=2))
