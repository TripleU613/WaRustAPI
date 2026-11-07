"""Pinned native parameter-reader checks and analyst-derived byte projections."""
import argparse,contextlib,hashlib,io,json,runpy,struct,sys
from pathlib import Path
from zipfile import ZipFile
p=argparse.ArgumentParser(description=__doc__);p.add_argument('library',type=Path);p.add_argument('apk',type=Path);a=p.parse_args()
base=Path(__file__).resolve().parent;sys.argv=[sys.argv[0],str(a.library)]
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset=[elf[k] for k in ('b','offset')]
calls={0xbf6b0:0xbf870,0xbf89c:0xcd570,0xbf940:0xcd570,0xbf7f8:0xcd570,0xbf800:0xcd3a0}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('selected BL mismatch')
h=hashlib.sha256()
with a.apk.open('rb') as f:
 for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
if h.hexdigest()!='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e':raise ValueError('APK identity mismatch')
def project(data):
 if len(data)<30:return {'fixed_header_read_complete':False,'header_value_checks':False,'body_admitted':False}
 declared,magic,c6,c8,hashlen=[struct.unpack_from('<H',data,o)[0] for o in (2,4,6,8,28)]
 header=10<=declared<=500 and magic==32767 and c6<=60000 and c8<=60000 and hashlen<=128
 expected=30+hashlen+8*c8
 return {'fixed_header_read_complete':True,'declared_length':declared,'count6':c6,'count8':c8,'hash_length':hashlen,'header_value_checks':header,'hash_bytes_available':len(data)>=30+hashlen,'expected_consumed_bytes':expected,'actual_bytes':len(data),'body_admitted':header and len(data)==expected}
assets={}
with ZipFile(a.apk) as z:
 for u in (1,2,4,6):
  name=f'assets/params_map_v4_u{u}.txt';data=z.read(name)
  assets[name]={'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'projection':project(data)}
 baseline=z.read('assets/params_map_v4_u4.txt')
def word(data,o,value):
 q=bytearray(data);struct.pack_into('<H',q,o,value);return bytes(q)
variants={'baseline':baseline,'version_bytes_changed':b'zz'+baseline[2:],'declared_10':word(baseline,2,10),'declared_500':word(baseline,2,500),'declared_9':word(baseline,2,9),'declared_501':word(baseline,2,501),'magic_zero':word(baseline,4,0),'count6_60001':word(baseline,6,60001),'count6_zero':word(baseline,6,0),'count8_60001':word(baseline,8,60001),'hash_length_129':word(baseline,28,129),'hash_length_zero':word(baseline,28,0),'unused_word26_changed':word(baseline,26,0),'trailing_byte':baseline+b'X','truncated_body':baseline[:-1],'short_fixed_header':baseline[:29]}
print(json.dumps({'apk_sha256':h.hexdigest(),'member_sha256':elf['digest'],'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'assets':assets,'projection_authority':'analyst transcription of selected helper gates under ordinary complete byte reads, not native execution or whole-loader validation','synthetic_u4_variants':{k:project(v) for k,v in variants.items()},'remaining':'caller routing, I/O fault states, record creation and configuration adoption untraced'},indent=2))
