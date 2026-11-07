"""Reuse pinned ELF validation; inspect selected handler-construction evidence only."""
import contextlib,io,json,runpy,sys
from pathlib import Path
base=Path(__file__).resolve().parent
# Prior inspector consumes the same positional library argument and validates the full hash.
with contextlib.redirect_stdout(io.StringIO()):
    elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,relocs,cstring=[elf[k] for k in ('b','offset','relocs','cstring')]
expected={0xd1f10:0x1e108,0xd1f58:0x1e268,0xd4990:0x58788,0xd4988:0xd1f08,0xd4568:0xd1f50,0xd4570:0x59884,0xd4578:0x59898}
for slot,target in expected.items():
    if relocs.get(slot)!=target:raise ValueError(f'relocation mismatch at {slot:x}')
strings={hex(va):cstring(va) for va in (0x18fff,0x1e1a8,0x1e108,0x1e268,0x1e318)}
if strings['0x18fff']!='fetch' or strings['0x1e318']!='com/facebook/mobileconfig/MobileConfigFetcherHandler':raise ValueError('method/class mismatch')
import struct
calls={0x5891c:0x60148,0x58944:0x60148,0x5895c:0x60148,0x58bc0:0x4cc84}
for site,target in calls.items():
    word=struct.unpack_from('<I',b,offset(site))[0];imm=word&0x3ffffff
    if imm&(1<<25):imm-=1<<26
    if word>>26!=0b100101 or site+4*imm!=target:raise ValueError('call mismatch')
print(json.dumps({'member_sha256':elf['digest'],'relative_relocations':{hex(k):hex(v) for k,v in expected.items()},'strings':strings,'selected_direct_calls':{hex(k):hex(v) for k,v in calls.items()},'scope':'static construction and callable storage handoff; concrete captured state and commit behavior unresolved'},indent=2))
