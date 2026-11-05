"""Execute the pinned JNI method-cache loop with controlled lookup adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3, UC_ARM64_REG_X21, UC_ARM64_REG_X22
failures,calls,lookups=[],[],[]
failure_index=None
fake_env,fake_class=0x10000100,0x10000200
expected=[]
for i in range(29):
    name_address=evidence['relocations'][0xc076e0+i*8]
    signature_address=evidence['relocations'][0xc077c8+i*8]
    def text(a):
        start=evidence['file_offset'](a)
        return data[start:data.index(0,start)].decode('ascii')
    expected.append((text(name_address),text(signature_address)))
def read_text(uc,address):
    result=bytearray()
    for n in range(2048):
        byte=uc.mem_read(address+n,1)[0]
        if byte==0:return result.decode('ascii')
        result.append(byte)
    raise AssertionError('unterminated adapter string')
def return_value(uc,value):
    uc.reg_write(UC_ARM64_REG_X0,value)
    uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def imported(uc,address,size,user):
    name=imports[address];calls.append(name)
    if name=='_ZN8facebook9simplejni15registerNativesEP7_JNIEnvPKcPK15JNINativeMethodm':
        assert uc.reg_read(UC_ARM64_REG_X0)==fake_env
        assert read_text(uc,uc.reg_read(UC_ARM64_REG_X1))=='com/whatsapp/wamsys/JniBridge'
        assert uc.reg_read(UC_ARM64_REG_X2)==0xc078b0
        assert uc.reg_read(UC_ARM64_REG_X3)==49
        return_value(uc,0)
    elif name=='_ZN8facebook9simplejni17getStaticMethodIdEP7_JNIEnvP7_jclassPKcS6_':
        i=len(lookups)
        assert uc.reg_read(UC_ARM64_REG_X0)==fake_env and uc.reg_read(UC_ARM64_REG_X1)==fake_class
        pair=(read_text(uc,uc.reg_read(UC_ARM64_REG_X2)),read_text(uc,uc.reg_read(UC_ARM64_REG_X3)))
        assert pair==expected[i],(i,pair)
        result=0 if i==failure_index else 0x10002000+i*8
        lookups.append({'index':i,'name':pair[0],'signature':pair[1],'synthetic_method_id':hex(result)})
        return_value(uc,result)
    elif name=='__wrap___cxa_guard_release':
        assert uc.reg_read(UC_ARM64_REG_X0)==0xc44bd8
        return_value(uc,0)
    else:
        failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_INTR,interrupted)
results=[]
for fail in [None,0,1,12,28]:
    failure_index=fail;failures.clear();calls.clear();lookups.clear()
    machine.mem_write(0xc467f0,bytes(29*8))
    machine.mem_write(0xc4435c,b'\xa5')
    machine.mem_write(0xc44bc0,struct.pack('<Q',fake_class))
    machine.mem_write(0xc44bd0,bytes(8))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_X21,fake_env)
    machine.reg_write(UC_ARM64_REG_X22,0xc44bc0)
    machine.emu_start(0x652824,0x651d04,timeout=1000000,count=100000)
    assert not failures,failures
    assert machine.reg_read(UC_ARM64_REG_PC)==0x651d04
    assert len(lookups)==(29 if fail is None else fail+1)
    flag=machine.mem_read(0xc4435c,1)[0]
    assert flag==int(fail is None)
    assert struct.unpack('<Q',machine.mem_read(0xc44bd0,8))[0]==fake_class
    for row in lookups:
        slot=0xc467f0+row['index']*8
        assert struct.unpack('<Q',machine.mem_read(slot,8))[0]==int(row['synthetic_method_id'],16)
    results.append({'failed_lookup_index':fail,'all_lookup_flag':flag,
                    'outbound_slot_value':hex(struct.unpack('<Q',machine.mem_read(0xc467f0,8))[0]),
                    'lookups':lookups[:],'imports':calls[:]})
print(json.dumps({'scope':'synthetic JNI method-cache loop only; no Java/runtime/native registration executed',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,
 'cases':results},indent=2))
