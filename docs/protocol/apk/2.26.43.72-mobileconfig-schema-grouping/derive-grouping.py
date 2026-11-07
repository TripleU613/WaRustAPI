"""Pinned contiguous schema grouping and synthetic table/index-map projection."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset=[elf[k] for k in ('b','offset')]
calls={0xaa4b0:0xc1534,0xc1558:0xaa544,0xc1578:0xb98d0,0xc1584:0xb99fc,0xc1608:0xc16fc,0xc16a4:0xc16fc,0xaa4d0:0x9c80c}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
def project(records,slots):
 # Valid owned record storage only; this is not a malformed native-memory model.
 if slots<0:raise ValueError('synthetic slots must be nonnegative')
 if slots==0 or not records:return {'table':[],'group_id_to_table_index':[]}
 table=[{'range':None,'skip_low_bit':1} for _ in range(slots)];mapping=[0]*slots
 start=0;ranges=[]
 for pos in range(1,len(records)+1):
  if pos==len(records) or records[pos]['word34']!=records[start]['word34']:
   ranges.append((start,pos,records[start]['word34']));start=pos
 for i,(start,end,key) in enumerate(ranges):
  if i>=slots or key<0 or key>=slots:raise ValueError('synthetic table/group-ID bounds refusal; native reaches BRK on selected out-of-bounds path')
  table[i]={'range':[start,end],'skip_low_bit':0,'group_word34':key,'first_wire_word4c':records[start]['word4c']}
  mapping[key]=i
 return {'table':table,'group_id_to_table_index':mapping}
rows=[{'word34':2,'word4c':101},{'word34':2,'word4c':101},{'word34':0,'word4c':202}]
examples={'sparse_group_ids':project(rows,4),'repeated_nonadjacent_group_id':project(rows+[{'word34':2,'word4c':101}],4),'empty_records':project([],4),'zero_slots':project(rows,0)}
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'builder':'0xc1534','grouping_word':'record+0x34','distinct_wire_token_word':'record+0x4c','source_record_stride':'0x50','table_entry_stride':'0x18','table_order':'adjacent-run encounter order, not group-ID sort','map_default':0,'authority':'analyst-derived synthetic projection; actual source vector, slot count and field producers unresolved','synthetic_records':rows,'synthetic_examples':examples},indent=2))
