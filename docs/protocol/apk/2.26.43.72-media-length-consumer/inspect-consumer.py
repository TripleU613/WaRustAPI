"""Hash mapped selected consumer windows in the pinned reconstructed image."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off=e['data'],e['off']
rows=[]
for label,a,z in [('consumer',0x51aa80,0x51ac1c),('callback-helper',0x72f330,0x72f33c),('consumer-registration',0x5206e0,0x52079c),('consumer-arguments',0x724748,0x724768),('owner-resolver',0x517ecc,0x517f50)]:
 start=off(a,z-a)
 rows.append({'label':label,'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'scope':'Selected static windows, not observed callback execution or exhaustive aliases','library_sha256':hashlib.sha256(data).hexdigest(),'windows':rows},indent=2))
