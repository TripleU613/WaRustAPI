"""Execute selected credential storage/scheduling with explicit synthetic adapters."""
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
failures, calls, scheduled = [], [], []
manager, credential = 0x10000000, 0x10001000
now = 0
provider_ready = True
def return_value(uc,value):
    uc.reg_write(UC_ARM64_REG_X0,value)
    uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def controlled(uc,address,size,user):
    if address == 0x721278:  # type getter excluded from storage fixture
        return_value(uc,0x1234)
    elif address == 0x72c4fc:  # raw clock supplied, not the clock implementation
        return_value(uc,now)
    elif address == 0x7216b0:  # diagnostic adapter, no logging/runtime
        return_value(uc,0)
    elif address == 0x6fef18:  # provider readiness predicate supplied
        return_value(uc,int(provider_ready))
    elif address in imports:
        name=imports[address];calls.append(name)
        if name == 'MCFRuntimeCreateInstance': return_value(uc,credential)
        elif name == 'MCFRetain': return_value(uc,uc.reg_read(UC_ARM64_REG_X0))
        elif name == 'MCIGlobalExecuteAsyncHelper':
            scheduled.append({'callback_va':hex(uc.reg_read(UC_ARM64_REG_X2)),
                              'context':hex(uc.reg_read(UC_ARM64_REG_X3))})
            return_value(uc,0)
        elif name == 'abort':
            failures.append('native abort');uc.emu_stop()
        else:
            failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
# X register constants are not assumed arithmetically contiguous.
from unicorn.arm64_const import UC_ARM64_REG_X3
machine.hook_add(UC_HOOK_CODE,controlled)
machine.hook_add(UC_HOOK_INTR,interrupted)
def execute(entry,args):
    from unicorn.arm64_const import UC_ARM64_REG_X3
    failures.clear();calls.clear();scheduled.clear()
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    for register,value in zip([UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3],args):
        machine.reg_write(register,value)
    machine.emu_start(entry,0x30000000,timeout=1000000,count=100000)
    if not failures: assert machine.reg_read(UC_ARM64_REG_PC)==0x30000000
results=[]
for raw_time in [0,1700000000,2147483647]:
    now=raw_time
    machine.mem_write(credential,bytes(0x80))
    execute(0x7212f4,[0x10002000,0x10003000,0x10004000])
    assert not failures,failures
    values=struct.unpack('<4QI',machine.mem_read(credential+0x18,36))
    assert values==(0x10002000,0x10003000,0x10004000,raw_time+172800,64),values
    assert calls.count('MCFRetain')==3 and calls.count('MCFRuntimeCreateInstance')==1
    results.append({'case':'constructor','raw_time':raw_time,'stored_slots':list(values),'imports':calls[:]})
for value in [0,1,64,180,2147483647,-1,-2147483648]:
    machine.mem_write(credential+0x38,struct.pack('<i',99))
    execute(0x721368,[credential,value & 0xffffffff])
    stored=struct.unpack('<i',machine.mem_read(credential+0x38,4))[0]
    if value<0: assert failures==['native abort'] and stored==99
    else: assert not failures and stored==value
    results.append({'case':'counter-setter','input':value,'stored':stored,'aborted':bool(failures)})
for name,provider,ready,pending,temp,expected in [
    ('no-provider',False,True,0,False,True),
    ('ready-provider',True,True,0,False,True),
    ('provider-not-ready',True,False,0,False,False),
    ('pending',True,True,1,False,False),
    ('temporary-state',True,True,0,True,False),
    ('pending-and-temporary',True,True,1,True,False),
    ('raw-flag-two',True,True,2,False,True)]:
    machine.mem_write(manager,bytes(0x100))
    machine.mem_write(manager+0x48,struct.pack('<Q',0x10005000 if provider else 0))
    machine.mem_write(manager+0x88,bytes([pending]))
    machine.mem_write(manager+0x98,struct.pack('<Q',0x10006000 if temp else 0))
    provider_ready=ready
    execute(0x728b24,[manager])
    assert not failures,failures
    assert bool(scheduled)==expected,(name,scheduled)
    flag=machine.mem_read(manager+0x88,1)[0]
    assert flag==(1 if expected else pending)
    if expected: assert scheduled==[{'callback_va':'0xbd8fb8','context':hex(manager)}]
    results.append({'case':name,'scheduled':scheduled[:],'flag_after':flag,'imports':calls[:]})
print(json.dumps({'scope':'synthetic storage/scheduling only; no provider/crypto/network execution',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,
 'cases':results},indent=2))
