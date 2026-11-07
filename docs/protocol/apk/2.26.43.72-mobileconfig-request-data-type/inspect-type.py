"""Pinned RequestData RTTI binding; response receiver identity remains unresolved."""
import contextlib,io,json,runpy
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
r=elf['relocs'];c=elf['cstring']
expected={0xd63f8:0xd3e00,0xd3e08:0x24b7c,0xd6400:0xa98a4,0xd6408:0xa987c}
for slot,value in expected.items():
 if r.get(slot)!=value:raise ValueError('RTTI relocation mismatch')
if c(0x24b7c)!='N12mobileconfig25FBMobileConfigRequestDataE':raise ValueError('type name mismatch')
print(json.dumps({'member_sha256':elf['digest'],'relative_relocations':{hex(k):hex(v) for k,v in expected.items()},'rtti_name':c(0x24b7c),'constructor':'0xa9430 installs address point 0xd6400 at 0xa9490','response_call':'0x7e69c invokes slot+0x10 of receiver from updater+0x2d8','boundary':'constructor identity does not bind the separate response receiver; 0x72f00 from another type vtable is not established as this call target'},indent=2))
