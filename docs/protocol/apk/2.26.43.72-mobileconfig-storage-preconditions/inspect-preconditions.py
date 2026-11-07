"""Pinned storage preconditions and synthetic early-result precedence, offline only."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
strings={0x17ccc:'Invalid StorageManager',0x176b4:'JSONDeserializeError',0x1b372:'Attempted to persist without JSON',0x177d5:'EPAndEmptyMetaSchemaError',0x198ee:'Empty meta schema when handling EP response'}
for va,value in strings.items():
 if cstring(va)!=value:raise ValueError('diagnostic mismatch')
calls={0x82d1c:0x880f8,0x82df4:0x60484,0x82e90:0x60484,0x82f2c:0x60484,0x83008:0xc950c,0x83014:0xcc370,0x830cc:0x60484}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
def early(source_pointer,path_length,hash_length,alternate,response_length,selector,context_mode):
 available=source_pointer or (path_length>0 and hash_length>0) or alternate
 if not available:return 2
 if response_length==0:return 3
 if selector==2 and context_mode==5:return 12
 return 'later parser/consumer path; adoption unproven'
cases={'missing_state_overrides_empty_response':early(False,0,0,False,0,2,5),'empty_response_overrides_ep_gate':early(True,0,0,False,0,2,5),'ep_gate':early(True,0,0,False,4,2,5),'source_pointer_only':early(True,0,0,False,4,0,0),'path_and_hash':early(False,3,3,False,4,0,0),'path_only':early(False,3,0,False,4,0,0),'alternate_state':early(False,0,0,True,4,0,0)}
print(json.dumps({'member_sha256':elf['digest'],'diagnostics':{hex(k):v for k,v in strings.items()},'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'early_result_bytes':{'unavailable_request_data':2,'empty_response':3,'selector2_context_mode5':12},'parser':'strlen-based shared adapter 0xc950c; null parse root copies diagnostics and continues to later processing','tag3_zero_word18':'sets local marker, constructs result byte1 and clears output byte+0x50','projection_authority':'analyst-derived precedence cases, not native execution, HTTP codes or complete response validation','synthetic_cases':cases,'remaining':'root callback kinds, later processing/future, validation and persistence untraced'},indent=2))
