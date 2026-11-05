"""Execute regular recovery decision slices after synthetic weak-reference resolution."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3, UC_ARM64_REG_X4, UC_ARM64_REG_X19, UC_ARM64_REG_X20, UC_ARM64_REG_X21, UC_ARM64_REG_D0
manager,provider,other,weak,old,new,queue,global_queue,allocator=[0x10000100+i*0x200 for i in range(9)]
events=[]
failures=[]
ready=0
def val(uc,r):return uc.reg_read(r)
def return_value(uc,value):uc.reg_write(UC_ARM64_REG_X0,value)
def adapted(uc,address,size,user):
    x0,x1,x2,x3,x4=[val(uc,r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3,UC_ARM64_REG_X4]]
    if address in imports:
        name=imports[address]
        if name=='MCFAbsoluteTimeGetCurrent':uc.reg_write(UC_ARM64_REG_D0,struct.unpack('<Q',struct.pack('<d',100.0))[0]);return
        if name=='MCIExecutionGlobalQueueGetExecutionQueue':assert x0==global_queue;return_value(uc,queue);return
        if name in ['MCFRetain','MCFRelease']:
            events.append({'action':name,'object':hex(x0)});return
        if name=='abort':events.append({'action':'abort'});uc.emu_stop();return
        failures.append('unexpected import '+name);uc.emu_stop();return
    if address==0x727294:
        assert x0==manager;events.append({'action':'drain-start'})
    elif address==0x735fdc:
        seconds=struct.unpack('<d',struct.pack('<Q',val(uc,UC_ARM64_REG_D0)))[0]
        assert (x0,x1,x2,x3)==(allocator,0xbd8f88,weak,queue)
        assert seconds==30.0
        events.append({'action':'timer-create','seconds':seconds,'callback':hex(x1),'context':hex(x2)});return_value(uc,new)
    elif address in [0x7360fc,0x73629c]:
        assert x0==(new if address==0x7360fc else old)
        events.append({'action':'timer-start' if address==0x7360fc else 'timer-cancel','object':hex(x0)})
    elif address==0x7273a8:
        assert x0==manager and x1==1;events.append({'action':'drain-end-error'})
    elif address==0x6fef18:
        assert x0==manager;return_value(uc,ready)
    else:raise AssertionError(hex(address))
    uc.reg_write(UC_ARM64_REG_PC,val(uc,UC_ARM64_REG_LR))
def interrupted(*args):failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,adapted,begin=0x30000100,end=next_stub-1)
for address in [0x727294,0x735fdc,0x7360fc,0x73629c,0x7273a8,0x6fef18]:
    machine.hook_add(UC_HOOK_CODE,adapted,begin=address,end=address)
machine.hook_add(UC_HOOK_INTR,interrupted)
results=[]
for same in [False,True]:
 for status in [0,1,2]:
  for busy in [0,1,2]:
   for has_old in [False,True]:
    events.clear();failures.clear();machine.mem_write(manager,bytes(0x80))
    machine.mem_write(manager+0x18,struct.pack('<QQ',allocator,global_queue))
    machine.mem_write(manager+0x28,struct.pack('<Q',provider));machine.mem_write(manager+0x38,struct.pack('<Q',weak))
    machine.mem_write(manager+0x50,bytes([busy]));machine.mem_write(manager+0x58,struct.pack('<Q',old if has_old else 0))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000);machine.reg_write(UC_ARM64_REG_X19,manager)
    machine.reg_write(UC_ARM64_REG_X20,status);machine.reg_write(UC_ARM64_REG_X21,provider if same else other)
    machine.emu_start(0x727820,0x7278bc,timeout=1000000,count=100000)
    assert not failures,failures
    actions=[x['action'] for x in events]
    if same and status==2:assert actions==['abort']
    else:
        assert machine.reg_read(UC_ARM64_REG_PC)==0x7278bc
        if same and status==0 and not busy&1:
            assert actions==['drain-start','timer-create','MCFRetain','MCFRelease','timer-start','MCFRelease']
            assert events[2]['object']==hex(new) and events[3]['object']==hex(old if has_old else 0) and events[5]['object']==hex(new)
        elif same and status==1:assert actions==(['timer-cancel'] if has_old else [])+['drain-end-error']
        else:assert actions==[]
    stored=struct.unpack('<Q',machine.mem_read(manager+0x58,8))[0]
    assert stored==(new if same and status==0 and not busy&1 else old if has_old else 0)
    results.append({'same_provider':same,'raw_status':status,'busy_byte':busy,'old_timer':has_old,'events':events[:],'stored_timer':hex(stored)})
timer=[]
for ready in [0,1,2,3]:
 for busy in [0,1,2,3]:
    events.clear();failures.clear();machine.mem_write(manager+0x50,bytes([busy]))
    machine.reg_write(UC_ARM64_REG_X19,manager);machine.reg_write(UC_ARM64_REG_X0,manager)
    machine.emu_start(0x72796c,0x727984,timeout=1000000,count=10000)
    assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x727984
    assert len(events)==int(bool(ready&1) and not busy&1)
    timer.append({'readiness_result':ready,'busy_byte':busy,'events':events[:]})
print(json.dumps({'scope':'synthetic post-weak-resolution status and timer decision slices; external drain/timer/provider operations intercepted; no real waits, network or storage',
 'unicorn_version':'2.1.4','status_cases':results,'timer_cases':timer},indent=2))
