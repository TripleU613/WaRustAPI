"""Execute credential-file header admission on synthetic local data only."""
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
from unicorn.arm64_const import UC_ARM64_REG_X24
failures,calls=[],[]
blob=b''
result=None
obj,buffer=0x10000100,0x10001000
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x10008000)
machine.mem_write(0x10008028,struct.pack('<Q',0x123456789abcdef))
def imported(uc,address,size,user):
    global result
    if address in [0x72a8bc,0x72a9cc]:
        result=address==0x72a9cc;uc.emu_stop();return
    if address not in imports:return
    name=imports[address];calls.append(name)
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    if name=='MCFDataGetLength':
        assert x0==obj;uc.reg_write(UC_ARM64_REG_X0,len(blob))
    elif name=='MCFSizeTFromInt32':
        assert x0==len(blob)
    elif name=='MCFDataGetBytePtr':
        assert x0==obj;uc.reg_write(UC_ARM64_REG_X0,buffer)
    elif name=='MCFRangeMake':
        assert x0==0 and x1==1;uc.reg_write(UC_ARM64_REG_X0,1<<32)
    elif name=='MCFDataGetBytes':
        assert x0==obj and x1==1<<32;uc.mem_write(x2,blob[:1])
    else:
        failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported)
machine.hook_add(UC_HOOK_INTR,interrupted)
def fixture(version=2,count=64,expiry=1700172800,lengths=(32,64,4),payload=None):
    if payload is None:payload=b'A'*sum(max(0,n) for n in lengths)
    return struct.pack('<BiQiii',version,count,expiry & ((1<<64)-1),*lengths)+payload
cases=[('valid',fixture()),('header-only-zero-lengths',fixture(lengths=(0,0,0))),
 ('trailing-bytes',fixture()+b'TRAIL'),('empty',b''),('short-header',bytes(24)),
 ('wrong-version',fixture(version=1)),('negative-count',fixture(count=-1)),
 ('max-count',fixture(count=2147483647)),('zero-count',fixture(count=0)),
 ('negative-expiry',fixture(expiry=-1)),('zero-expiry',fixture(expiry=0)),
 ('max-expiry',fixture(expiry=2**63-1))]
for i in range(3):
    lengths=[32,64,4];lengths[i]=-1
    cases.append((f'negative-length-{i}',fixture(lengths=lengths,payload=b'A'*100)))
    lengths[i]=2147483647
    cases.append((f'oversized-length-{i}',fixture(lengths=lengths,payload=b'A'*100)))
for n in [1,4,5,68,100]:cases.append((f'truncated-{n}',fixture()[:-n]))
results=[]
for name,value in cases:
    blob=value;result=None;failures.clear();calls.clear()
    machine.mem_write(buffer,blob+bytes(1024))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_X24,obj)
    machine.emu_start(0x72a818,0x72a9cc+4,timeout=1000000,count=100000)
    assert not failures and result is not None,(name,failures,result)
    expected=False
    if len(blob)>=25:
        version,count,expiry,a,b,c=struct.unpack_from('<Biqiii',blob)
        expected=version==2 and count>=0 and expiry>=0 and min(a,b,c)>=0 and a+b+c<=len(blob)-25
    assert result==expected,(name,result,expected)
    results.append({'case':name,'bytes':len(blob),'native_admitted':result,'independent_header_check':expected,'imports':calls[:]})
print(json.dumps({'scope':'synthetic local header admission only; no file I/O, reconstruction or crypto validation',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,
 'cases':results},indent=2))
