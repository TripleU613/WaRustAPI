"""Pin class-name slices, JNI tables and registration windows in ARM64 ELF."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,rel,string=[e[k] for k in ['data','off','rel','string']]
rows=[]
for a,n,table,count,expected in [(0xf5dd3,41,0x755fd0,1,'com/facebook/tigon/TigonXplatBodyProvider'),(0xf5f8f,39,0x756048,5,'com/facebook/tigon/TigonXplatBodyStream'),(0xf5f43,41,0x7560f0,5,'com/facebook/tigon/TigonXplatRequestToken'),(0xf6520,36,0x756198,4,'com/facebook/tigon/TigonXplatService')]:
 start=off(a,n);name=data[start:start+n].decode('ascii');assert name==expected
 methods=[{'name':string(rel[q]),'signature':string(rel[q+8]),'entry_va':hex(rel[q+16])} for q in range(table,table+count*24,24)]
 rows.append({'class_name':name,'literal_va':hex(a),'literal_bytes':n,'table_va':hex(table),'count':count,'methods':methods})
assert rows[0]['methods']==[{'name':'initHybrid','signature':'()Lcom/facebook/jni/HybridData;','entry_va':'0x64f83c'}]
assert rows[1]['methods'][3]=={'name':'reportBodyLengthNative','signature':'(I)V','entry_va':'0x64fe3c'}
assert rows[3]['methods'][-1]['name']=='sendRequestBodyBufferIntegerBuffer' and rows[3]['methods'][-1]['entry_va']=='0x650ff0'
windows=[]
for name,a,z in [('registration',0x64cf40,0x64d11c),('dispatch',0x39fbe0,0x39fc30),('dispatch-arguments',0x310600,0x310610),('table-copy',0x728a90,0x728aa0),('single-method-args',0x72e7fc,0x72e808),('array-method-args',0x72baf4,0x72bb00),('near-factory-exclusion',0x64f790,0x64f83c)]:
 start=off(a,z-a);windows.append({'name':name,'start_va':hex(a),'end_va':hex(z),'bytes':z-a,'file_offset':hex(start),'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'scope':'Static registration bindings, not observed loading or successful JNI registration','library_sha256':hashlib.sha256(data).hexdigest(),'bindings':rows,'windows':windows},indent=2))
