"""Pinned JSON adapter and selected consumer-status handoff, without execution."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset=[elf[k] for k in ('b','offset')]
calls={0xc9560:0xcd5a0,0xc9570:0xcd5b0,0xc9578:0xcc9d0,0xc9588:0xcd620,0xc9590:0xcd5d0,0xc95b0:0xcd5e0,0xc95f8:0xcd630,0xbb51c:0xc950c,0xbb528:0xc60f8,0xbb530:0xc9728,0xc5500:0xc52a4}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
if struct.unpack_from('<I',b,offset(0xbb52c))[0]!=0xaa1503e0:raise ValueError('expected status overwrite mov x0,x21')
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'adapter':'0xc950c','input_length':'strlen of input pointer','parser_allocation':'yajl_alloc with callbacks at 0xd4260','parser_option':{'numeric_option':1,'value':1,'meaning':'not resolved here'},'completion_status_branch':'zero returns retained root after yajl_free; nonzero formats optional diagnostic and releases partial state','selected_consumer':'0xc60f8: null or non-tag4 root returns false; valid-shaped items traverse existing records with a forward cursor','ignored_consumer_status':{'call':'0xbb528','overwrite':'0xbb52c mov x0,x21','next_call':'0xbb530 parsed-root cleanup'},'scope':'static parser/consumer boundary, not full decoder, native execution, rollback proof or configuration acceptance'},indent=2))
