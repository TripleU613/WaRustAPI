"""Execute timer arm/event/cancel and credential callback with controlled adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3,UC_ARM64_REG_D0
manager,timer,manager_weak,timer_weak,queue,provider= [0x10000100+i*0x100 for i in range(6)]
manager_alive=True
timer_alive=True
provider_ready=True
calls=[]
scheduled=[]
failures=[]
work_calls=0
def ret(uc,value):
    uc.reg_write(UC_ARM64_REG_X0,value);uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def controlled(uc,address,size,user):
    global work_calls
    if address in [0x735f44,0x727aec]:ret(uc,0x1234);return
    if address==0x7216b0:ret(uc,0);return
    if address==0x6fef18:ret(uc,int(provider_ready));return
    if address==0x727d40:
        assert uc.reg_read(UC_ARM64_REG_X0)==manager
        work_calls+=1;ret(uc,0);return
    if address not in imports:return
    name=imports[address];calls.append(name)
    x0,x1,x2,x3=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3]]
    if name=='MCFWeakGetTypeID':ret(uc,0x2345)
    elif name in ['__MCFCastToClassNotNull','__MCFCastToClassNullableOwnedObject']:ret(uc,x1)
    elif name=='MCFWeakCopyRef':
        assert x0 in [manager_weak,timer_weak]
        ret(uc,(manager if manager_alive else 0) if x0==manager_weak else (timer if timer_alive else 0))
    elif name in ['pthread_mutex_lock','pthread_mutex_unlock']:
        assert x0==timer+0x24;ret(uc,0)
    elif name=='MCFRelease':ret(uc,0)
    elif name=='MCIExecutionQueueExecuteAsyncHelper':
        assert x0==queue and x2==0xbd9168 and x3==timer_weak
        interval=struct.unpack('<d',struct.pack('<Q',uc.reg_read(UC_ARM64_REG_D0)))[0]
        assert interval==30.0
        scheduled.append({'callback_va':hex(x2),'context':hex(x3),'raw_interval':interval})
        ret(uc,0)
    else:
        failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,controlled)
machine.hook_add(UC_HOOK_INTR,interrupted)
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x10008000)
machine.mem_write(0x10008028,struct.pack('<Q',0x123456789abcdef))
def reset(cancel=0,armed=0,repeat=1,has_provider=True):
    global work_calls
    calls.clear();scheduled.clear();failures.clear();work_calls=0
    machine.mem_write(manager,bytes(0x100));machine.mem_write(timer,bytes(0x100))
    machine.mem_write(manager+0x48,struct.pack('<Q',provider if has_provider else 0))
    machine.mem_write(timer+0x20,bytes([cancel,armed]))
    machine.mem_write(timer+0x50,struct.pack('<QQQQd',timer_weak,0xbd8fc8,manager_weak,queue,30.0))
    machine.mem_write(timer+0x80,bytes([repeat]))
def execute(entry,arg):
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    machine.reg_write(UC_ARM64_REG_X0,arg)
    machine.emu_start(entry,0x30000000,timeout=1000000,count=100000)
    assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x30000000,(hex(entry),failures)
results=[]
for name,alive,owner_alive,has_provider,ready,cancel,repeat in [
 ('ready',True,True,True,True,0,1),('not-ready',True,True,True,False,0,1),
 ('no-provider',True,True,False,True,0,1),('manager-gone',True,False,True,True,0,1),
 ('timer-gone',False,True,True,True,0,1),('canceled',True,True,True,True,1,1),
 ('nonrepeating-zero',True,True,True,True,0,0),('nonrepeating-two',True,True,True,True,0,2)]:
    timer_alive=alive;manager_alive=owner_alive;provider_ready=ready
    reset(cancel=cancel,repeat=repeat,has_provider=has_provider)
    execute(0x7361cc,timer_weak)
    expected_work=int(alive and owner_alive and has_provider and ready and not(cancel&1))
    expected_schedule=int(alive and not(cancel&1) and repeat==1)
    assert work_calls==expected_work and len(scheduled)==expected_schedule,(name,work_calls,scheduled)
    results.append({'case':name,'work_calls':work_calls,'queued_events':scheduled[:],'imports':calls[:]})
for cancel,armed in [(0,0),(1,0),(0,1),(1,1),(2,0)]:
    reset(cancel=cancel,armed=armed)
    execute(0x7360fc,timer)
    expected=int(not(cancel&1) and not(armed&1))
    assert len(scheduled)==expected
    flags=list(machine.mem_read(timer+0x20,2))
    assert flags==[cancel,1 if expected else armed]
    results.append({'case':'arm','before_flags':[cancel,armed],'after_flags':flags,'queued_events':scheduled[:],'imports':calls[:]})
for cancel in [0,2]:
    reset(cancel=cancel,armed=1)
    execute(0x73629c,timer)
    flags=list(machine.mem_read(timer+0x20,2))
    assert flags==[1,1] and not scheduled
    results.append({'case':'cancel','before_cancel_flag':cancel,'after_flags':flags,'queued_events':scheduled[:],'imports':calls[:]})
print(json.dumps({'scope':'synthetic arm/event/cancel and callback gating; fake mutex/weak/scheduler; no actual queues or network',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,'cases':results},indent=2))
