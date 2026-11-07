"""Pinned owner/receiver RTTI and construction aliases; static selected route only."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,r,cstring=[elf[k] for k in ('b','offset','relocs','cstring')]
expected={0xd4d88:0xd3420,0xd3428:0x20ab3,0xd5578:0xd38d8,0xd38e0:0x2215e,0xd5590:0x82c4c}
for slot,target in expected.items():
 if r.get(slot)!=target:raise ValueError('RTTI/vtable mismatch')
if cstring(0x20ab3)!='N12mobileconfig21FBMobileConfigManagerE' or cstring(0x2215e)!='N12mobileconfig28FBMobileConfigStorageManagerE':raise ValueError('owner/receiver name mismatch')
site=0x77818;w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
if imm&(1<<25):imm-=1<<26
if w>>26!=0b100101 or site+4*imm!=0x79c28:raise ValueError('shared alias call mismatch')
print(json.dumps({'member_sha256':elf['digest'],'relative_relocations':{hex(k):hex(v) for k,v in expected.items()},'owner_type':cstring(0x20ab3),'receiver_type':cstring(0x2215e),'geometry':{'storage_object':'allocation+0x20, vtable installed 0x77614','storage_alias':'0x77810..0x77818 -> shared pair at sp+0xf0','manager_object':'allocation+0x18, vtable installed 0x7795c','manager_receiver_pair':'alias x26=allocation+0x58; store pair at +0x48 -> manager+0x88','updater_receiver':'prior proof copies manager+0x88 to updater+0x2d8'},'response_slot':'storage vtable+0x10 = 0xd5590 -> 0x82c4c','scope':'selected static constructor/receiver binding; later replacement, overrides, live traffic and parsing/persistence contract unresolved'},indent=2))
