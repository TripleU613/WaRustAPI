"""Validate pinned requester dispatch and local empty-query refusal evidence."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,r,cstring=[elf[k] for k in ('b','offset','relocs','cstring')]
expected={0xd53a8:0x7f648,0xd6458:0xd3fe0,0xd3fe8:0x24c29,0xd6470:0xabf18,0xd4990:0x58788}
for slot,target in expected.items():
 if r.get(slot)!=target:raise ValueError(f'relocation mismatch at {slot:x}')
if cstring(0x24c29)!='N12mobileconfig23FBMobileConfigRequesterE':raise ValueError('requester RTTI mismatch')
if cstring(0x16f64)!='Empty query params':raise ValueError('refusal text mismatch')
def short_string(va):
 o=offset(va);tag=b[o]
 if tag&1 or tag>>1>22 or b[o+1+(tag>>1)]!=0:raise ValueError('expected bounded libc++ short string')
 return b[o+1:o+1+(tag>>1)].decode()
strings={hex(va):short_string(va) for va in (0x24c58,0x24c70)}
if list(strings.values())!=['mobileconfig','POST']:raise ValueError('static string changed')
calls={0xabf60:0xac09c,0xabfbc:0x60148,0xac050:0x5971c}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('BL mismatch')
for site in [0xabfec,0x7e500]:
 if struct.unpack_from('<I',b,offset(site))[0]!=0xd63f0100:raise ValueError('BLR changed')
print(json.dumps({'member_sha256':elf['digest'],'relative_relocations':{hex(k):hex(v) for k,v in expected.items()},'requester_rtti':cstring(0x24c29),'static_short_strings':strings,'local_refusal':cstring(0x16f64),'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'updater_getter':'0x7f648','requester_entry':'0xabf18','fetch_interface_call':'0xabfec','callback_argument':'x4: copied callable storage','refusal_gate':'request-data word +0x9c != 1 and constructed query vector empty','scope':'selected conditional static route; runtime fetcher implementation and full query-vector producer remain unobserved'},indent=2))
