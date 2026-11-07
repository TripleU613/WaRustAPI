"""Pinned ELF relocation/table binding, not execution or complete callback analysis."""
import argparse,hashlib,json,struct
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('library',type=Path);a=p.parse_args()
b=a.library.read_bytes();digest=hashlib.sha256(b).hexdigest()
if digest!='5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c':raise ValueError('member hash mismatch')
base=Path(__file__).parent
prior=json.loads((base.parent/'2.26.43.72-superpack-reconstructed-members/validation.json').read_text())
rows=prior if isinstance(prior,list) else prior.get('members',[])
if not any(r.get('name')=='libmobileconfig-jni.so' and r.get('sha256')==digest and r.get('bytes')==len(b) for r in rows):raise ValueError('prior member validation mismatch')
if b[:6]!=b'\x7fELF\x02\x01' or struct.unpack_from('<H',b,18)[0]!=183:raise ValueError('ELF identity changed')
u16=lambda o:struct.unpack_from('<H',b,o)[0]
u64=lambda o:struct.unpack_from('<Q',b,o)[0]
phoff=u64(32);phsize=u16(54);phnum=u16(56)
loads=[struct.unpack_from('<IIQQQQQQ',b,phoff+i*phsize) for i in range(phnum) if struct.unpack_from('<I',b,phoff+i*phsize)[0]==1]
def offset(va):
    found=[off+va-vaddr for typ,flags,off,vaddr,paddr,filesz,memsz,align in loads if vaddr<=va<vaddr+filesz]
    if len(found)!=1:raise ValueError('VA mapping changed')
    return found[0]
shoff=u64(40);shsize=u16(58);shnum=u16(60)
relocs={}
for i in range(shnum):
    sh=struct.unpack_from('<IIQQQQIIQQ',b,shoff+i*shsize)
    if sh[1]!=4:continue
    for pos in range(sh[4],sh[4]+sh[5],sh[9]):
        va,info,addend=struct.unpack_from('<QQq',b,pos)
        if info&0xffffffff==1027:relocs[va]=addend
slots=[0xd1f20,0xd1f28,0xd1f30]
values=[relocs[x] for x in slots]
def cstring(va):
    o=offset(va);return b[o:b.index(0,o)].decode()
if [cstring(values[0]),cstring(values[1]),values[2]]!=['onCompletion','(ZLjava/lang/String;)V',0x59574]:raise ValueError('selected JNI binding changed')
class_bytes=b[offset(0x1b7ed):offset(0x1b7ed)+52]
if class_bytes!=b'com/facebook/mobileconfig/MobileConfigFetcherHandler':raise ValueError('class span changed')
word=struct.unpack_from('<I',b,offset(0x595e4))[0]
if word>>26!=0b100101:raise ValueError('expected BL')
imm=word&0x3ffffff
if imm&(1<<25):imm-=1<<26
target=0x595e4+4*imm
if target!=0x5971c or struct.unpack_from('<I',b,offset(0x59748))[0]!=0xd63f0100:raise ValueError('handoff changed')
print(json.dumps({'member_sha256':digest,'member_bytes':len(b),'class_span_va':'0x1b7ed','class_span_bytes':52,'class_name':class_bytes.decode(),'table':[{'slot_va':hex(x),'raw_file_word':hex(u64(offset(x))),'relative_addend':hex(relocs[x])} for x in slots],'method':cstring(values[0]),'signature':cstring(values[1]),'entry':hex(values[2]),'direct_call_va':'0x595e4','direct_call_target':hex(target),'indirect_call_va':'0x59748','indirect_instruction':'blr x8','remaining_target':'runtime stored callable, not resolved here'},indent=2))
