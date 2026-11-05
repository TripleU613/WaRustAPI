"""Execute selected recovered-buffer header admission with synthetic MCFData adapters."""
import contextlib
import io
import json
from pathlib import Path
import runpy
import struct
from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE, UC_HOOK_INTR
from unicorn.arm64_const import UC_ARM64_REG_SP, UC_ARM64_REG_LR, UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2, UC_ARM64_REG_PC, UC_ARM64_REG_TPIDR_EL0
source = Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(source))
data, sections = evidence['data'], evidence['sections']
machine = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
machine.mem_map(0, 0xc50000)
machine.mem_map(0x10000000, 0x10000)
machine.mem_map(0x20000000, 0x100000)
machine.mem_map(0x30000000, 0x10000)
phoff = struct.unpack_from('<Q',data,32)[0]
phsize, phnum = struct.unpack_from('<HH',data,54)
for index in range(phnum):
    kind,flags,offset,va,pa,filesz,memsz,align = struct.unpack_from('<IIQQQQQQ',data,phoff+index*phsize)
    if kind == 1: machine.mem_write(va,data[offset:offset+filesz])
imports = {}
next_stub = 0x30000100
for section in sections:
    if section[1] != 4: continue
    for offset in range(section[4],section[4]+section[5],24):
        address,info,addend = struct.unpack_from('<QQq',data,offset)
        kind = info & 0xffffffff
        if kind == 1027:
            assert info >> 32 == 0
            machine.mem_write(address,struct.pack('<Q',addend))
        elif kind in [257,1025,1026]:
            symbols = sections[section[6]]
            symbol = struct.unpack_from('<IBBHQQ',data,symbols[4]+(info>>32)*symbols[9])
            if kind == 257 and symbol[3] != 0:
                machine.mem_write(address,struct.pack('<Q',symbol[4]+addend))
                continue
            names = sections[symbols[6]]
            name_offset = struct.unpack_from('<I',data,symbols[4]+(info>>32)*symbols[9])[0]
            position = names[4]+name_offset
            name = data[position:data.index(0,position)].decode('ascii')
            imports[next_stub] = name
            machine.mem_write(next_stub,bytes.fromhex('c0035fd6'))
            machine.mem_write(address,struct.pack('<Q',next_stub+addend))
            next_stub += 4
        else:
            raise RuntimeError(('unsupported relocation',kind))
from unicorn.arm64_const import UC_ARM64_REG_X19, UC_ARM64_REG_X20, UC_ARM64_REG_X21, UC_ARM64_REG_X26, UC_ARM64_REG_X28, UC_ARM64_REG_X29
obj,buffer=0x10000100,0x10001000
blob=b''
failures=[]
def ret(uc,value):uc.reg_write(UC_ARM64_REG_X0,value)
def imported(uc,address,size,user):
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    name=imports[address]
    if name=='MCFDataGetLength':assert x0==obj;ret(uc,len(blob))
    elif name=='MCFDataGetBytePtr':assert x0==obj;ret(uc,buffer)
    elif name=='adler32':assert (x0,x1,x2)==(0,0,0);ret(uc,1)
    else:failures.append('unexpected import '+name);uc.emu_stop()
def byte(uc,address,size,user):
    assert uc.reg_read(UC_ARM64_REG_X0)==obj
    index=uc.reg_read(UC_ARM64_REG_X1);assert index<len(blob)
    ret(uc,blob[index]);uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def boundary(uc,address,size,user):uc.emu_stop()
def interrupted(*args):failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_CODE,byte,begin=0x72c9f4,end=0x72c9f4)
for address in [0x6fe428,0x6fe498]:machine.hook_add(UC_HOOK_CODE,boundary,begin=address,end=address)
machine.hook_add(UC_HOOK_INTR,interrupted)
def header(version=3,length=1,total=25,magic=b'WMT',checksum=0x12345678):
    b=bytearray(max(total,24));b[:3]=magic;b[3]=version
    struct.pack_into('<dIII',b,4,12.5,91,length&0xffffffff,checksum)
    return bytes(b[:total])
cases=[('v0',header(version=0),False,None),('v1',header(version=1),True,20),
 ('v2',header(version=2),True,24),('v3',header(),True,24),('v4',header(version=4),False,None),
 ('short-23',header(total=23),False,None),('bad-W',header(magic=b'XMT'),False,None),
 ('bad-M',header(magic=b'WXT'),False,None),('bad-T',header(magic=b'WMX'),False,None),
 ('zero-payload',header(length=0),False,None),('negative-payload',header(length=-1),False,None),
 ('oversized-payload',header(length=0x7fffffff),False,None),
 ('v2-truncated',header(version=2,length=2,total=25),False,None),
 ('v3-trailing',header(length=1,total=30),True,24),
 ('v1-min24',header(version=1,length=4,total=24),True,20),
 ('v1-short21',header(version=1,length=1,total=21),False,None),
 ('unchecked-checksum-zero',header(checksum=0),True,24),
 ('unchecked-checksum-max',header(checksum=0xffffffff),True,24)]
results=[]
for name,blob,expected,expected_header in cases:
    failures.clear();machine.mem_write(buffer,blob)
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000);machine.reg_write(UC_ARM64_REG_X29,0x200ff050);machine.reg_write(UC_ARM64_REG_X28,obj)
    machine.emu_start(0x6fe34c,0x30000000,timeout=1000000,count=100000)
    assert not failures,failures
    pc=machine.reg_read(UC_ARM64_REG_PC);assert pc in [0x6fe428,0x6fe498]
    accepted=pc==0x6fe498;assert accepted==expected,name
    result={'case':name,'bytes':len(blob),'header_hex':blob[:24].hex(),'accepted':accepted}
    if accepted:
        length=machine.reg_read(UC_ARM64_REG_X21)&0xffffffff
        checksum=machine.reg_read(UC_ARM64_REG_X20)&0xffffffff
        size=machine.reg_read(UC_ARM64_REG_X26)
        assert size==expected_header
        assert length==struct.unpack_from('<I',blob,16)[0]
        assert checksum==(1 if blob[3]==1 else struct.unpack_from('<I',blob,20)[0])
        result.update(header_bytes=size,payload_bytes=length,restored_checksum=checksum)
    results.append(result)
print(json.dumps({'scope':'native header-admission slice only; MCFData byte/length and adler32-initial adapters; no directory, file read, deletion, task creation or network',
 'unicorn_version':'2.1.4','cases':results},indent=2))
