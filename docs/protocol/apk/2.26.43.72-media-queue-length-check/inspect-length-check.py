"""Pin selected length-check windows/literals in the recovered ARM64 image."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,string=e['data'],e['off'],e['string']
rows=[]
for name,a,z in [('overflow',0x5432b8,0x54339c),('header-copy',0x542fa0,0x5431e8),('disable-native-lookup',0x3fa81c,0x3fa85c),('queue-caller',0x53c288,0x53c2c4),('queue-late-caller',0x53c530,0x53c554),('queue-arguments',0x2e2bc0,0x2e2bcc)]:
 start=off(a,z-a);rows.append({'name':name,'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
literals=[{'va':hex(a),'value':string(a)} for a in [0xffed2,0xe0226,0xd97d3]]
assert literals[1]['value']=='Request body size exceeds content-length'
assert literals[2]['value']=='getDisableContentLengthHeader'
print(json.dumps({'scope':'Static queue branch and namespace lookup; no target execution or dynamic upload binding','library_sha256':hashlib.sha256(data).hexdigest(),'windows':rows,'literals':literals},indent=2))
