"""Pinned native source-loader anchors and APK parameter-map names; offline only."""
import argparse,contextlib,hashlib,io,json,runpy,struct,sys
from pathlib import Path
from zipfile import ZipFile
p=argparse.ArgumentParser(description=__doc__);p.add_argument('library',type=Path);p.add_argument('apk',type=Path);a=p.parse_args()
base=Path(__file__).resolve().parent
sys.argv=[sys.argv[0],str(a.library)]
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
expected={0x165a3:'params_map_v4_u1.txt',0x16549:'params_map_v4_u2.txt',0x16322:'params_map_kMobileConfigSessionless.txt',0x1636d:'params_map_kMobileConfigSessionbased.txt',0x1b1b2:'/\\'}
for va,value in expected.items():
 if cstring(va)!=value:raise ValueError('filename literal mismatch')
calls={0xa9bf0:0xbb28c,0xa9bfc:0xbfc6c,0xa9cc4:0xbb28c,0xa9ce0:0xbb28c,0xa9dc0:0xbfc6c,0xa9dcc:0x5b7d8,0xbb39c:0xbc0ac,0xbc114:0x990c0}
for site,target in calls.items():
 w=struct.unpack_from('<I',b,offset(site))[0];imm=w&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if w>>26!=0b100101 or site+4*imm!=target:raise ValueError('BL mismatch')
h=hashlib.sha256()
with a.apk.open('rb') as f:
 for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
if h.hexdigest()!='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e':raise ValueError('APK hash mismatch')
with ZipFile(a.apk) as z:
 names=sorted(n for n in z.namelist() if 'params_map' in n)
print(json.dumps({'apk_sha256':h.hexdigest(),'member_sha256':elf['digest'],'native_filename_literals':{hex(k):v for k,v in expected.items()},'apk_parameter_map_members':names,'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'path_source':'configuration+0x130','source_destination':'configuration+0x258 -> first shared-pointer pair','record_accessor':'0xbfc6c selects child+0x50 pair when child byte+0x98 == 1, otherwise loader+0x150 pair','scope':'static file-loading and shared-record handoff; path producer and file-content decoder remain untraced'},indent=2))
