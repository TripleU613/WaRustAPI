"""Pinned text-record construction and synthetic flag/slot projection, not a parser."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
strings={0x1b4a2:'END',0x1bccd:'failed_to_find_end_2',0x192c4:'wrong_token_size_while_parsing',0x18700:'wrong_token_size_while_parsing_param',0x192e3:'empty_line_while_parsing'}
for va,value in strings.items():
 if cstring(va)!=value:raise ValueError('text literal mismatch')
calls={0xbb630:0xbc410,0xbb824:0xc7810,0xbbb34:0xcca30,0xbbb44:0xcca30,0xbbbb8:0xbe52c}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
def project(flags,slot_before=9):
 flags&=0xffffffff;kind=(flags>>2)&7
 if kind>=6:return {'type':kind,'in_model_domain':False,'native_branch':'0xbbe58; outcome not modeled'}
 return {'type':kind,'in_model_domain':True,'record_word3c_slot':slot_before,'counter_after':slot_before+1,'record_byte40':kind,'record_byte48':int((flags&0x80000001)==1),'record_byte49':(flags>>6)&1,'bit5_optional_followup':bool(flags&32)}
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'literals':{hex(k):v for k,v in strings.items()},'record_layout':{'0x00':'group string copy','0x18':'parameter string copy','0x30':'loader+0x178 accumulated parameter word','0x34':'loader+0x180 group ordinal','0x38':'same accumulated parameter word','0x3c':'previous selected type counter','0x40':'(flags >> 2) & 7, requires <6 on this path','0x44':'selected local version/unit word w22; producer conditions retained, semantic name unproven','0x48':'(flags & 0x80000001) == 1','0x49':'flags bit6','0x4c':'loader+0x184 wire config ID'},'lookup_key':'(accumulated_parameter_word OR (config_ID << 12)) mod 2^32','authority':'analyst-derived scalar projection over supplied flags/counters, not a decoder, actual record extraction or APK execution','synthetic_flag_cases':{hex(f):project(f) for f in [0,4,5,0x44,0x64,0x80000005,0x14,0x18]},'remaining':'actual text source, complete line grammar/conversion/errors, finalized vector and alternate binary path remain untraced'},indent=2))
