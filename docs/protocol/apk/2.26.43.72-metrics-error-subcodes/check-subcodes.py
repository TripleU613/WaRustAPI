"""Execute selected native metrics body-parser branches with typed-object adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3, UC_ARM64_REG_X22
manager,body,root,nested,number,current,spare = [0x10000100+i*0x200 for i in range(7)]
dictionary_type,number_type=101,102
released=[]
refresh=[]
failures=[]
fixture={}
def ret(uc,value):uc.reg_write(UC_ARM64_REG_X0,value)
def imported(uc,address,size,user):
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    name=imports[address]
    if name=='MCIJSONCreateDeserializedObject':
        assert x0==body and x1==0
        ret(uc,root if fixture['root'] else 0)
    elif name=='MCFDictionaryGetTypeID':ret(uc,dictionary_type)
    elif name=='MCFNumberGetTypeID':ret(uc,number_type)
    elif name in ['__MCFCastToClassOrNull','__MCFCastToClassNullable']:
        valid=(x0==dictionary_type and x1 in [root,nested]) or (x0==number_type and x1==number)
        ret(uc,x1 if valid else 0)
    elif name=='MCFDictionaryGetValue':
        if x0==root:
            assert x1==0xc3b6c0
            ret(uc,nested if fixture['nested'] else 0)
        else:
            assert x0==nested and x1==0xc3b6f0
            ret(uc,number if fixture['number'] else 0)
    elif name=='MCFNumberGetValue':
        assert x0==number and x1==0
        uc.mem_write(x2,struct.pack('<I',fixture['code'] & 0xffffffff));ret(uc,1)
    elif name=='MCFRelease':released.append(x0);ret(uc,0)
    else:failures.append('unexpected import '+name);uc.emu_stop()
def refreshing(uc,address,size,user):
    assert uc.reg_read(UC_ARM64_REG_X0)==manager
    refresh.append(True);uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def interrupted(*args):failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_CODE,refreshing,begin=0x728b24,end=0x728b24)
machine.hook_add(UC_HOOK_INTR,interrupted)
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x1000f000)
cases=[('no-body',False,True,True,True,0),('non-dictionary-root',True,False,True,True,0),
       ('missing-error',True,True,False,True,0),('missing-subcode',True,True,True,False,0)]
for code in [0,3067001,3067002,3067003,3067004,3067005,0x7fffffff,0xffffffff]:
    cases.append((f'subcode-{code}',True,True,True,True,code))
results=[]
for name,has_body,has_root,has_nested,has_number,code in cases:
 for has_spare in [False,True]:
    fixture.update(root=has_root,nested=has_nested,number=has_number,code=code)
    released.clear();refresh.clear();failures.clear()
    machine.mem_write(manager,bytes(0x100));machine.mem_write(manager+0x78,struct.pack('<QQ',current,spare if has_spare else 0))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000);machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    machine.reg_write(UC_ARM64_REG_X0,manager);machine.reg_write(UC_ARM64_REG_X1,body if has_body else 0)
    machine.emu_start(0x72943c,0x30000000,timeout=1000000,count=100000)
    assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x30000000,(name,failures)
    recognized=has_body and has_root and has_nested and has_number and code in [3067002,3067003,3067004]
    expected=code-3067002+2 if recognized else (6 if not has_body else 5)
    actual=machine.reg_read(UC_ARM64_REG_X0)&0xffffffff
    slots=struct.unpack('<QQ',machine.mem_read(manager+0x78,16))
    assert actual==expected,(name,actual,expected)
    assert slots==((spare if has_spare else 0,0) if recognized else (current,spare if has_spare else 0))
    assert bool(refresh)==(recognized and not has_spare)
    assert (current in released)==recognized
    results.append({'case':name,'spare_present':has_spare,'raw_result':actual,
                    'released_current':current in released,'refresh_calls':len(refresh),
                    'current_after':hex(slots[0]),'spare_after':hex(slots[1])})
# A separate exact worker slice maps the extracted integer to its diagnostic result.
worker=[]
for code in [0,3067001,3067002,3067003,3067004,3067005,0xffffffff]:
    machine.reg_write(UC_ARM64_REG_X0,code);machine.reg_write(UC_ARM64_REG_X22,5)
    machine.emu_start(0x729140,0x729154,timeout=1000000,count=32)
    result=machine.reg_read(UC_ARM64_REG_X22)&0xffffffff
    assert result==(code-3067002+2 if code in [3067002,3067003,3067004] else 5)
    worker.append({'subcode':code,'raw_result':result})
print(json.dumps({'scope':'synthetic typed-object adapters; native body-parser and worker integer slice only; no JSON library, HTTP or durable storage execution',
 'unicorn_version':'2.1.4','parser_cases':results,'worker_integer_cases':worker},indent=2))
