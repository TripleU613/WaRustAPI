import hashlib,json,struct,sys
from pathlib import Path
p=Path(sys.argv[1]);data=p.read_bytes();assert hashlib.sha256(data).hexdigest()=='d0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d'
shoff=struct.unpack_from('<Q',data,40)[0];size,count=struct.unpack_from('<HH',data,58);assert size==64
ss=[struct.unpack_from('<IIQQQQIIQQ',data,shoff+size*i) for i in range(count)]
def off(a,n=1):
 m=[s[4]+a-s[3] for s in ss if s[1]!=8 and s[5] and s[3]<=a and a+n<=s[3]+s[5]];assert len(m)==1;return m[0]
def string(a):
 o=off(a);e=data.index(0,o);off(a,e-o+1);return data[o:e].decode('ascii')
rel={}
for s in ss:
 if s[1]!=4:continue
 assert s[9]==24
 for pos in range(s[4],s[4]+s[5],24):
  a,i,v=struct.unpack_from('<QQq',data,pos)
  if i&0xffffffff==1027:assert i>>32==0;rel[a]=v
rows=[]
for slot in [0x756048,0x756060,0x756078,0x756090]:
 rows.append({'slot_va':hex(slot),'name':string(rel[slot]),'signature':string(rel[slot+8]),'function_va':hex(rel[slot+16]),'file_offset':hex(off(rel[slot+16],4))})
assert [(r['name'],r['signature'],r['function_va']) for r in rows]==[('transferBytesNative','(Ljava/nio/ByteBuffer;I)I','0x64faa4'),('transferBytesArrayNative','([BII)I','0x64fbe0'),('writeEOMNative','()V','0x64fd80'),('reportBodyLengthNative','(I)V','0x64fe3c')]
windows=[]
for a,z in [(0x64fe3c,0x64ff08),(0x72921c,0x729228),(0x64fbe0,0x64fe3c)]:
 start=off(a,z-a);windows.append({'start_va':hex(a),'end_va':hex(z),'file_offset':hex(start),'bytes':z-a,'sha256':hashlib.sha256(data[start:start+z-a]).hexdigest()})
print(json.dumps({'windows':windows,'library_sha256':hashlib.sha256(data).hexdigest(),'scope':'Static relocated JNI tuples; no registration or target execution','methods':rows},indent=2))
