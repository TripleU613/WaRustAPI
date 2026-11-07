"""Pinned record finalization and a synthetic signed-merge projection."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset=[elf[k] for k in ('b','offset')]
calls={0xbfe78:0x5b7d8,0xbff28:0xbfbc0,0xbff74:0xbfbc0,0xc007c:0x5f9e8,0xc00cc:0xbd714,0xc00e4:0x6f44c,0xc0120:0x75fc8}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
w=struct.unpack_from('<I',b,offset(0xbff1c))[0]
if w&0xff00001f!=0x5400000c:raise ValueError('expected signed GT conditional branch')
def merge(existing,new):
 # Supplied ordered scalar streams only; native map/record ownership is not simulated.
 i=j=0;out=[]
 while i<len(existing) and j<len(new):
  if existing[i]<=new[j]:out.append(['existing',existing[i]]);i+=1
  else:out.append(['new',new[j]]);j+=1
 return out+[['existing',x] for x in existing[i:]]+[['new',x] for x in new[j:]]
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'entry':'0xbfe18','guard':'loader byte+0x70 == 1 and byte+0xe1 low bit clear','branch':'child byte+0x98 == 1 selects paired existing/new merge; otherwise iterator-defined group append','merge_comparison':'signed record word+0x30; existing chosen on equality, with new still pending','authority':'static finalizer binding and analyst-derived scalar merge examples, not native execution or actual map ordering','synthetic_merge_examples':{'interleaved':merge([1,3],[2,4]),'equal_words_retained':merge([2],[2]),'signed_order':merge([-1,2],[0]),'empty_existing':merge([],[1,2])},'remaining':'map comparator/input ordering invariants, producer flags, caller admission, persistence and account ownership unresolved'},indent=2))
