"""Execute regular drain completion gating with controlled error/callback adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3
manager,table,context,error=0x10000100,0x10000200,0x10000300,0x10000400
callback=0x3000f000
machine.mem_write(callback,bytes.fromhex('c0035fd6'))
failures=[]
status_calls=[]
errors=[]
released=[]
def imported(uc,address,size,user):
    x0,x1,x2,x3=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3]]
    if address==callback:
        status_calls.append({'context':hex(x0),'arg1':x1,'arg2':x2,'error':hex(x3),'active_flag_at_callback':uc.mem_read(manager+0x50,1)[0]})
        assert x0==context and x1==0 and x2==0
        return
    name=imports[address]
    if name=='MCFErrorCreate':
        assert x0==0xc30518 and x1==3 and x2==0
        errors.append({'domain_va':hex(x0),'local_code':x1});uc.reg_write(UC_ARM64_REG_X0,error)
    elif name=='MCFRelease':released.append(hex(x0));uc.reg_write(UC_ARM64_REG_X0,0)
    else:failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_CODE,imported,begin=callback,end=callback)
machine.hook_add(UC_HOOK_INTR,interrupted)
cases=[('active-normal',1,0,True,False),('active-error',1,1,True,False),
       ('active-even-argument',1,2,True,False),('active-odd-argument',1,3,True,False),
       ('active-no-table',1,0,False,False),('active-error-no-table',1,1,False,False)]
for flag in [0,2,3]:
    for argument in [0,1]:cases.append((f'inactive-{flag}-{argument}',flag,argument,True,False))
cases += [('repeat-normal',1,0,True,True),('repeat-error',1,1,True,True)]
results=[]
for name,flag,argument,has_table,repeat in cases:
    failures.clear();status_calls.clear();errors.clear();released.clear()
    machine.mem_write(manager,bytes(0x80));machine.mem_write(manager+0x50,bytes([flag]))
    machine.mem_write(manager+0x40,struct.pack('<Q',table if has_table else 0))
    machine.mem_write(table+0x30,struct.pack('<QQ',callback,context))
    for invocation in range(2 if repeat else 1):
        machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
        machine.reg_write(UC_ARM64_REG_LR,0x30000000)
        machine.reg_write(UC_ARM64_REG_X0,manager)
        machine.reg_write(UC_ARM64_REG_X1,argument)
        machine.emu_start(0x7273a8,0x30000000,timeout=1000000,count=100000)
        assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x30000000,(name,failures)
    active=flag==1;error_path=active and bool(argument&1)
    assert len(status_calls)==int(active and has_table)
    assert len(errors)==int(error_path) and len(released)==int(error_path)
    if status_calls:assert status_calls[0]['error']==hex(error if error_path else 0) and status_calls[0]['active_flag_at_callback']==0
    after=machine.mem_read(manager+0x50,1)[0];assert after==(0 if active else flag)
    results.append({'case':name,'before_flag':flag,'raw_argument':argument,'invocations':2 if repeat else 1,
                    'after_flag':after,'status_callbacks':status_calls[:],'created_local_errors':errors[:],'released':released[:]})
print(json.dumps({'scope':'synthetic regular drain completion only; callback/error adapters; no task retry or storage execution',
 'unicorn_version':'2.1.4','instruction_cap_per_invocation':100000,'timeout_microseconds_per_invocation':1000000,'cases':results},indent=2))
