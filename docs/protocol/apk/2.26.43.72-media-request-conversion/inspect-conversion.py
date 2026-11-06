"""Pin native request conversion windows and timeout diagnostic literals."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,string=e['data'],e['off'],e['string']
rows=[]
for name,a,z in [('request-transfer',0x651b64,0x651fa0),('request-property',0x651fa0,0x652080),('timeout-normalizer',0x66377c,0x663844),('timeout-setters',0x66370c,0x66377c),('timeout-prologue',0x725af8,0x725b08),('timeout-epilogue',0x725ab8,0x725ac8),('header-member',0x663844,0x66386c),('header-member-helper',0x7312b4,0x7312c0)]:
 start=off(a,z-a);rows.append({'name':name,'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
assert string(0xfec5f)=='Timeout is too small: ' and string(0xce792)=='validateTimeout'
print(json.dumps({'scope':'Selected static request conversion/timeout handling; no wire or runtime proof','library_sha256':hashlib.sha256(data).hexdigest(),'windows':rows,'timeout_literals':{hex(a):string(a) for a in [0xfec5f,0xd9cc6,0xce792,0xcefbd]}},indent=2))
