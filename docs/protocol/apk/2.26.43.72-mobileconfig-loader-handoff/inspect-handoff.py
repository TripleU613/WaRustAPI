"""Pinned loader diagnostic/finalization handoff and constructor state seeding."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
strings={0x181bc:'native_error__parse_main_params_map',0x16e69:'native_error__parse_package_params_maps',0x19e42:'Params List is empty, typically means we failed to parse params map file',0x1aa5f:'No valid schema hash and param list provided'}
for va,value in strings.items():
 if cstring(va)!=value:raise ValueError('diagnostic literal mismatch')
calls={0x5a27c:0x5c6e8,0x5a284:0xbfe18,0x5a290:0xbfc6c,0x5a53c:0x5c6e8,0x5a544:0xbfe18,0x5a574:0xbfc6c,0x5a6e8:0xa9430,0xa953c:0x4c570}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'diagnostics':{hex(k):v for k,v in strings.items()},'record_flow':'loader finalizer -> shared accessor -> local pair -> constructor x1 -> state allocation first pair -> object+0x258','constructor_initial_state':{'object_byte194':'argument w4 & 1','object_byte208':1,'object_byte209':0,'state_bytes40_through46':[1]*7,'state_pointer_field':'object+0x258','record_pair_retained':'shared control count increment when nonnull'},'scope':'selected normal control flow; diagnostics do not independently establish rejection or configuration acceptance; outer admission and persistent state remain unresolved'},indent=2))
