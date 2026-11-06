"""Verify pinned ELF relocation/literal bindings without executing the target."""
import contextlib, hashlib, io, json, runpy, struct, sys
from pathlib import Path
helper = Path(__file__).resolve().parent.parent / '2.26.43.72-media-body-provider/inspect-native-tuples.py'
with contextlib.redirect_stdout(io.StringIO()):
    e = runpy.run_path(str(helper))
data, rel, off, string = (e[k] for k in ('data', 'rel', 'off', 'string'))
slot = 0x74d0a8
row = (string(rel[slot]), string(rel[slot+8]), rel[slot+16])
assert row == ('initHybrid', '(Ljava/lang/String;Ljava/lang/String;)Lcom/facebook/jni/HybridData;', 0x4d50b8)
assert data[off(0xf575c, 47):off(0xf575c, 47)+47] == b'com/whatsapp/infra/tigon/WAUserAgentInterceptor'
assert string(0xd3b39) == 'https'
assert string(0xd0924) == 'User-Agent'
assert string(0xd6fef) == 'WAUserAgentInterceptor'
assert rel[0x769d30+0x10] == 0x4d5320
assert rel[0x769d60+0x30] == 0x4d5644
assert 'JWAUserAgentInterceptor15makeInterceptor' in string(0x14bfeb)
assert ((0x30075 >> 14) & 0x7f, 0x30075 & 0x3fff) == (12,117)
windows = json.loads((Path(__file__).parent/'native-windows.json').read_text())
rows=[]
for name, (a,z) in windows.items():
    a,z=int(a,16),int(z,16); start=off(a,z-a)
    rows.append(dict(name=name,start_va=hex(a),end_va=hex(z),file_offset=hex(start),bytes=z-a,sha256=hashlib.sha256(data[start:start+z-a]).hexdigest()))
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),jni=dict(name=row[0],signature=row[1],function_va=hex(row[2])),windows=rows,scope='Static bindings, not observed runtime activation'),indent=2))
