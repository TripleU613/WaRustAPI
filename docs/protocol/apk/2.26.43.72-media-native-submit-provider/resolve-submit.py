"""Resolve pinned JNI tuples and provider vtable chain; no execution."""
import contextlib,hashlib,io,json,runpy
with contextlib.redirect_stdout(io.StringIO()):
 e=runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
data,off,rel,string,ss=[e[k] for k in ['data','off','rel','string','ss']]
def tuple_at(a):return {'slot_va':hex(a),'name':string(rel[a]),'signature':string(rel[a+8]),'function_va':hex(rel[a+16])}
submit=tuple_at(0x7561e0);factory=tuple_at(0x755fd0)
assert submit['name']=='sendRequestBodyBufferIntegerBuffer' and submit['function_va']=='0x650ff0'
assert submit['signature']=='(Lcom/facebook/tigon/iface/TigonRequest;[BILcom/facebook/tigon/TigonBodyProvider;[BILcom/facebook/tigon/TigonCallbacks;Ljava/util/concurrent/Executor;)Lcom/facebook/tigon/TigonXplatRequestToken;'
assert factory['name']=='initHybrid' and factory['signature']=='()Lcom/facebook/jni/HybridData;' and factory['function_va']=='0x64f83c'
tables=[]
for a,count,name in [(0x7788d0,3,'N8facebook5tigon22TigonXplatBodyProviderE'),(0x778780,7,'N8facebook5tigon21TigonJavaBodyProviderE')]:
 assert int.from_bytes(data[off(a-16,8):off(a-16,8)+8],'little')==0
 ti=rel[a-8];assert string(rel[ti+8])==name
 functions={hex(i*8):hex(rel[a+i*8]) for i in range(count)}
 assert all(any(s[2]&4 and s[3]<=rel[a+i*8]<s[3]+s[5] for s in ss) for i in range(count))
 tables.append({'rtti_name':name,'typeinfo_va':hex(ti),'address_point':hex(a),'functions':functions})
assert tables[0]['functions']['0x10']=='0x64fa04' and tables[1]['functions']['0x10']=='0x64e908'
rows=[]
for name,a,z in [('entry',0x650ff0,0x651344),('converter-prefix',0x651860,0x651b64),('provider-factory',0x64f83c,0x64f9c8),('provider-adapter',0x64fa04,0x64faa4),('provider-name',0x64f064,0x64f090),('provider-copy-window',0x3ed468,0x3ed49c),('stream-start-prefix',0x64e908,0x64ec00),('jni-arguments',0x72a1cc,0x72a1d8),('buffer-arguments',0x72e194,0x72e1a0),('adapter-result',0x732ba8,0x732bb4),('adapter-arguments',0x72f908,0x72f914)]:
 start=off(a,z-a);rows.append({'name':name,'start_va':hex(a),'end_va':hex(z),'bytes':z-a,'file_offset':hex(start),'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
assert string(0xf1bfb)=='Java'
print(json.dumps({'scope':'Static JNI/provider chain and selected windows, not observed dispatch or final wire construction','library_sha256':hashlib.sha256(data).hexdigest(),'jni':[submit,factory],'tables':tables,'provider_name':string(0xf1bfb),'windows':rows},indent=2))
