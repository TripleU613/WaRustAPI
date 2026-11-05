"""Execute native recovered-buffer version/checksum gate with host Adler adapter."""
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
import zlib
from unicorn.arm64_const import UC_ARM64_REG_X19, UC_ARM64_REG_X20, UC_ARM64_REG_X21, UC_ARM64_REG_X22, UC_ARM64_REG_X24, UC_ARM64_REG_X25, UC_ARM64_REG_X29
owner,buffer,obj,pointer=0x10000100,0x10000400,0x10000800,0x10001000
payload=b''
checks=[]
logs=[]
failures=[]
def ret(uc,v):uc.reg_write(UC_ARM64_REG_X0,v)
def imported(uc,address,size,user):
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    name=imports[address]
    if name=='MCFDataGetBytePtr':assert x0==obj;ret(uc,pointer)
    elif name=='MCFDataGetLength':assert x0==obj;ret(uc,len(payload))
    elif name=='adler32':
        if x1==0:assert (x0,x2)==(0,0);ret(uc,1);checks.append('initialize')
        else:
            assert x0==1 and x1==pointer and x2==len(payload)
            checks.append('payload');ret(uc,zlib.adler32(bytes(uc.mem_read(x1,x2)),x0))
    else:failures.append('unexpected import '+name);uc.emu_stop()
def boundary(uc,address,size,user):uc.emu_stop()
def logging(uc,address,size,user):
    assert uc.reg_read(UC_ARM64_REG_X1)==3 and uc.reg_read(UC_ARM64_REG_X2)==0x14a20a
    logs.append(True);uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def interrupted(*args):failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_CODE,boundary,begin=0x722eec,end=0x722eec)
mismatch_hook=machine.hook_add(UC_HOOK_CODE,boundary,begin=0x722fc0,end=0x722fc0)
machine.hook_add(UC_HOOK_CODE,logging,begin=0x7216b0,end=0x7216b0)
machine.hook_add(UC_HOOK_INTR,interrupted)
# The mismatch boundary hook is removed temporarily for the counter slice.
results=[]
for payload in [b'\x00',b'WAM',bytes(range(256))]:
 for version,matching in [(1,False),(2,False),(3,True),(3,False)]:
    checks.clear();logs.clear();failures.clear()
    actual=zlib.adler32(payload);stored=actual if matching else actual^1
    machine.mem_write(pointer,payload);machine.mem_write(buffer,bytes(0x40))
    machine.mem_write(buffer+0x18,struct.pack('<Q',obj));machine.mem_write(buffer+0x30,struct.pack('<I',stored));machine.mem_write(buffer+0x34,bytes([version]))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000);machine.reg_write(UC_ARM64_REG_X29,0x200ff030)
    machine.mem_write(0x200ff008,struct.pack('<Q',0x10002000))
    for r,v in [(UC_ARM64_REG_X19,owner),(UC_ARM64_REG_X20,1),(UC_ARM64_REG_X21,7),(UC_ARM64_REG_X22,0x10002000),(UC_ARM64_REG_X24,2),(UC_ARM64_REG_X25,buffer),(UC_ARM64_REG_X0,9)]:machine.reg_write(r,v)
    machine.emu_start(0x722e80,0x30000000,timeout=1000000,count=100000)
    assert not failures,failures
    accepted=machine.reg_read(UC_ARM64_REG_PC)==0x722eec
    assert accepted==(version<3 or matching)
    assert checks==([] if version<3 else ['initialize','payload'])
    result={'version':version,'payload_hex':payload.hex(),'stored_checksum':stored,'actual_checksum':actual,'gate_passed':accepted,'adler_calls':checks[:]}
    if not accepted:
        machine.mem_write(owner+0xe8,struct.pack('<I',17))
        # Start just after the boundary at the first LDR; execute it explicitly
        # by disabling only that hook, then run the complete mismatch block.
        # A second machine hook can be deleted without changing target bytes.
        machine.hook_del(mismatch_hook)
        machine.emu_start(0x722fc0,0x722fec,timeout=1000000,count=10000)
        mismatch_hook=machine.hook_add(UC_HOOK_CODE,boundary,begin=0x722fc0,end=0x722fc0)
        assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x722fec
        counter=struct.unpack('<I',machine.mem_read(owner+0xe8,4))[0]
        assert counter==18 and len(logs)==1
        result['counter_after_mismatch']=counter
    results.append(result)
print(json.dumps({'scope':'native version/checksum and mismatch-counter slices; host zlib Adler32 adapter; no whole dictionary walk, storage or transport execution',
 'unicorn_version':'2.1.4','cases':results},indent=2))
