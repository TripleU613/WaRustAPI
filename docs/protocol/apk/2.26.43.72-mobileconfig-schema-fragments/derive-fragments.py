"""Pinned static anchors and analyst-derived synthetic fragment/checksum examples."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
if cstring(0x1bc30)!=':' or cstring(0x1b1c0)!='[' or cstring(0x1b1b0)!=']':raise ValueError('delimiter mismatch')
calls={0xab828:0xc150c,0xab86c:0xc14e4,0xab9c4:0x9c754,0xab974:0xaa594,0xadb44:0x5e4b8,0xad9c8:0xab954,0xad9e0:0xab954}
for site,target in calls.items():
 word=struct.unpack_from('<I',b,offset(site))[0];imm=word&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if word>>26!=0b100101 or site+4*imm!=target:raise ValueError('call mismatch')
MASK=(1<<32)-1
def signed32(x):
 x&=MASK
 return x-(1<<32) if x&(1<<31) else x
def config_token(word,label):
 word&=MASK
 return str(word) if 1<=word<=1048575 else label
def param_token(word,label):
 word=signed32(word)
 return str(word) if word!=-1 and word<16384 else label
def project(groups,indices,admitted=None):
 # Supplied groups represent valid nonempty 0x50-byte record ranges.
 # admitted stands for the lookup predicate's result; it does not emulate buckets.
 parts=[];acc=0
 for i in indices:
  if i<0 or i>=len(groups):raise ValueError('synthetic index outside modeled table')
  records=groups[i]
  if not records:raise ValueError('empty record range outside this synthetic model')
  if admitted is not None and (records[0][0]&MASK) not in admitted:continue
  first=records[0]
  parts.append(config_token(first[0],first[2])+''.join(':'+param_token(p,n) for c,p,label,n in records))
  for c,p,label,n in records:
   term=((p&MASK)+31*(c&MASK)+16337)&MASK
   acc=((acc*397)^term)&MASK
 # This models literal stream writes, not JSON escaping.
 return {'fragments':parts,'literal_array':'['+','.join('"'+p+'"' for p in parts)+']','checksum_u32':acc,'checksum_decimal':str(acc)}
groups=[[(1,2,'config_one','param_two'),(1,3,'config_one','param_three')],[(1048576,-1,'named_config','named_param')],[(7,0xffffffff,'seven','minus_one'),(7,-2,'seven','negative_two'),(7,16383,'seven','last_numeric'),(7,16384,'seven','named_boundary')]]
examples={
 'empty_selection':project(groups,[]),
 'original_order':project(groups,[0,1]),
 'reverse_order':project(groups,[1,0]),
 'duplicate_selection':project(groups,[0,0]),
 'first_record_filter':project(groups,[0,1],{1048576}),
 'signed_param_boundaries':project(groups,[2]),
 'wrapping_words':project([[(0xffffffff,0xffffffff,'large','fallback')]]*2,[0,1]),
}
print(json.dumps({'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'record_stride':'0x50','selected_index_stride':4,'group_table_stride':'0x18','record_words':{'0x4c':'word used for group token/filter and checksum','0x30':'word used for parameter token and checksum'},'recurrence':'acc = ((acc * 397) XOR (param_word + 31 * config_word + 16337)) mod 2^32','examples_authority':'analyst-derived synthetic transcription, not APK execution or live request data','synthetic_examples':examples},indent=2))
