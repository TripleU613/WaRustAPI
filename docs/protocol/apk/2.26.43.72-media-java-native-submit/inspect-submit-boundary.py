"""Pin selected native internal-header candidate; no media-path assumption."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,rel,string=[e[k] for k in ['data','off','rel','string']]
assert rel[0x788e80]==0xe0240 and string(0xe0240)=='content-length'
assert string(0xf7c89)=='-1' and string(0xdeef7)=='sec-websocket-protocol'
rows=[]
for name,a,z in [('synthesis-candidate',0x4e6230,0x4e62d4),('dictionary-helper',0x310914,0x310920)]:
 start=off(a,z-a);rows.append({'name':name,'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'scope':'Selected native internal dictionary writes, not ordinary-media upload or final wire header','library_sha256':hashlib.sha256(data).hexdigest(),'windows':rows,'literals':{hex(a):string(a) for a in [0xe0240,0xf7c89,0xdeef7]}},indent=2))
