"""Pinned selector anchors and analyst-derived synthetic selection matrix."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
if cstring(0x25876)!='' or cstring(0x1b192)!='_':raise ValueError('predicate literals changed')
calls={0xaa7fc:0xab3e4,0xaa834:0xab3e4,0xab43c:0xaa594,0xab4e8:0xab5a8,0xab4fc:0xab5a8,0xab514:0xab65c,0xab634:0x70a28}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
def group(label,count=1,word30=1,param_label='',byte40=1,word44=7,table_byte10=0):
 return {'table_byte10':table_byte10,'records':[{'label':label,'word30':word30,'param_label':param_label,'byte40':byte40,'word44':word44} for _ in range(count)]}
def predicate(records):
 return len(records)==1 and records[0]['label'].startswith('gk_') and records[0]['word30']==1 and records[0]['param_label'] in ('','_')
def select(table,version,side,mode,version_gate=True):
 indices=[];count=0
 for i,g in enumerate(table):
  rows=g['records']
  if g['table_byte10']&1 or not rows:continue
  if version_gate and rows[0]['word44']!=version:continue
  p=predicate(rows)
  if mode==0:admit=bool(side&1) and not p
  elif mode==1:admit=p!=bool(side&1)
  elif mode==2:admit=all(r['byte40']==1 for r in rows)!=bool(side&1)
  else:admit=False
  if admit:indices.append(i);count+=len(rows)
 return {'selected_indices':indices,'selected_record_count':count}
table=[group('gk_enabled'),group('ordinary',word30=2,byte40=0),group('gk_multiple',count=2),group('skipped',table_byte10=1),group('empty',count=0),group('different_version',word44=8),group('gk_underscore',param_label='_',byte40=0)]
examples={f'mode{mode}_side{side}':select(table,7,side,mode) for mode in range(4) for side in (0,1)}
examples['version_gate_disabled']=select(table,7,1,1,False)
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'refresh_mode':'0 if configuration byte +0x194 == 0 else 1 + (byte +0x208 & 1)','first_refresh_container':{'destination':'configuration+0x258 -> +0x18','side':1,'version_gate':1},'second_refresh_container':{'destination':'configuration+0x258 -> +0x20','side':0,'version_gate':1},'authority':'analyst-derived synthetic valid-table projection, not APK execution or extracted actual schema inputs','synthetic_table':table,'synthetic_selection_matrix':examples,'remaining':'schema table construction, field producers, concurrent lifecycle, request-data modes and response persistence untraced'},indent=2))
