"""Independently identify RegisterNatives table index from local JDK21 JNI.h."""
import hashlib,json,re,sys
from pathlib import Path
p=Path(sys.argv[1]);data=p.read_bytes()
assert hashlib.sha256(data).hexdigest()=='99e64ebbe749e6df284f852f11b3c73f6ea97baf15120428f40f887fe0616e61'
s=data.decode().split('struct JNINativeInterface_ {',1)[1].split('\n};',1)[0]
rows=re.findall(r'void\s*\*\s*(reserved\d+)\s*;|\(JNICALL\s*\*\s*(\w+)\)',s)
names=[a or b for a,b in rows]
assert names[:4]==['reserved0','reserved1','reserved2','reserved3'] and names.count('RegisterNatives')==1
index=names.index('RegisterNatives');assert index==215 and index*8==0x6b8
print(json.dumps({'scope':'JNI ABI slot identification using independent local JDK header; not Android execution','source':'Temurin21.0.13 JNI.h','source_sha256':hashlib.sha256(data).hexdigest(),'table_fields':len(names),'method':'RegisterNatives','index':index,'arm64_pointer_bytes':8,'arm64_slot_offset':hex(index*8)},indent=2))
