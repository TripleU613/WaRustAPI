"""Pinned updater callback RTTI/vtable and selected call checks; no execution."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,r,cstring=[elf[k] for k in ('b','offset','relocs','cstring')]
expected={0xd3810:0x219e1,0xd53f8:0xd3808,0xd5430:0x80604,0xd5398:0x7e5ac}
for slot,target in expected.items():
 if r.get(slot)!=target:raise ValueError(f'relocation mismatch at {slot:x}')
# Establish the updater typeinfo through its name pointer and the vtable preceding slot+0x38.
updater_name_slots=[s for s,v in r.items() if v==0x2191d]
if len(updater_name_slots)!=1 or r.get(0xd5358)!=updater_name_slots[0]-8:raise ValueError('updater RTTI mismatch')
name=cstring(0x219e1)
if '28FBMobileConfigDefaultUpdater12queueRequest' not in name:raise ValueError('callable RTTI mismatch')
if cstring(0x1a80b)!='updater deallocated':raise ValueError('diagnostic mismatch')
calls={0x80620:0xcaf88,0x80640:0x76d38,0x76d5c:0xcd210,0x7e468:0x80490}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('BL mismatch')
if struct.unpack_from('<I',b,offset(0x80694))[0]!=0xd63f0100:raise ValueError('indirect invocation changed')
print(json.dumps({'member_sha256':elf['digest'],'relative_relocations':{hex(k):hex(v) for k,v in expected.items()},'callable_rtti':name,'updater_rtti':cstring(0x2191d),'expired_owner_diagnostic':cstring(0x1a80b),'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'callback_entry':'0x80604','owner_lock_helper':'0x76d38','completion_virtual_slot':'0x38','updater_vtable_address_point':'0xd5360','selected_updater_target':'0x7e5ac','boundary':'candidate upstream callback; intermediate dispatch to Java fetch bridge not resolved'},indent=2))
