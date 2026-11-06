"""Verify selected queue initialization/key bytes in the pinned ARM64 library."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,rel,string=[e[k] for k in ['data','off','rel','string']]
assert rel[0x788e80]==0xe0240 and string(0xe0240)=='content-length'
assert string(0xdc822)=='http2MaxInFlightRequestsPerConnection'
rows=[]
for label,a,z in [('actual-queue-init',0x53b9c4,0x53baec),('header-lookup',0x540018,0x54006c),('key-resolution',0x54edb0,0x54ee7c),('number-conversion',0x642378,0x642418),('header-copy-binding',0x542d90,0x542df8),('excluded-settings',0x51c510,0x51c640),('partial-header-reader',0x53a980,0x53ac80),('queue-core',0x633c80,0x633e00),('header-key-object',0x788e58,0x788e88),('excluded-allocation',0x545b68,0x545bdc)]:
 start=off(a,z-a);rows.append({'label':label,'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'scope':'Static initializer/header binding; no live upload or complete wire-stack proof','library_sha256':hashlib.sha256(data).hexdigest(),'key':{'object_va':'0x788e58','pointer_slot_va':'0x788e80','string_va':'0xe0240','value':string(0xe0240)},'excluded_setting':string(0xdc822),'windows':rows},indent=2))
