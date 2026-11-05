"""Execute pinned integer payload helpers with a bounded synthetic MCFData adapter."""
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
failures = []
called = {}
data_length = 0
data_object, data_buffer = 0x10000100, 0x10001000
def imported(uc,address,size,user):
    global data_length
    name = imports[address]
    called[name] = called.get(name,0)+1
    if name == 'MCFRetain': return  # identity return
    if name == 'MCFRelease':
        uc.reg_write(UC_ARM64_REG_X0,0)
        return
    if name in ['MCFDataGetLength','MCFDataIncreaseLength','MCFDataGetMutableBytePtr','MCFDataAppendBytes']:
        assert uc.reg_read(UC_ARM64_REG_X0) == data_object
        if name == 'MCFDataGetLength': uc.reg_write(UC_ARM64_REG_X0,data_length)
        elif name == 'MCFDataGetMutableBytePtr': uc.reg_write(UC_ARM64_REG_X0,data_buffer)
        elif name == 'MCFDataIncreaseLength':
            data_length += uc.reg_read(UC_ARM64_REG_X1)
            assert data_length <= 64
        else:
            count = uc.reg_read(UC_ARM64_REG_X2)
            assert data_length+count <= 64
            uc.mem_write(data_buffer+data_length,bytes(uc.mem_read(uc.reg_read(UC_ARM64_REG_X1),count)))
            data_length += count
        return
    failures.append('unsupported import '+name)
    uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt')
    machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_INTR,interrupted)

writer = 0x10000000
machine.mem_write(writer+0x18,struct.pack('<Q',data_object))
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x10008000)
machine.mem_write(0x10008028,struct.pack('<Q',0x123456789abcdef))
values = [-2**63,-2**31-1,-2**31,-32769,-32768,-129,-128,-1,0,1,2,127,128,255,
          32767,32768,65535,65536,2**31-1,2**31,2**63-1]
results = []
for entry in [0x72c184,0x72c244]:
    for value in values:
        data_length = 0
        machine.mem_write(data_buffer,bytes(64))
        machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
        machine.reg_write(UC_ARM64_REG_LR,0x30000000)
        machine.reg_write(UC_ARM64_REG_X0,writer)
        machine.reg_write(UC_ARM64_REG_X1,value & 0xffffffffffffffff)
        called.clear(); failures.clear()
        machine.emu_start(entry,0x30000000,timeout=1000000,count=100000)
        assert not failures, failures
        assert machine.reg_read(UC_ARM64_REG_PC) == 0x30000000
        tag = machine.reg_read(UC_ARM64_REG_X0)
        if entry == 0x72c244 and value in [0,1]:
            expected_tag, expected_bytes = value+1,b''
        else:
            width = next(width for width in [1,2,4,8] if -(1 << (width*8-1)) <= value < (1 << (width*8-1)))
            expected_tag = {1:3,2:4,4:5,8:6}[width]
            expected_bytes = value.to_bytes(width,'little',signed=True)
        actual = bytes(machine.mem_read(data_buffer,data_length)) if data_length else b''
        assert tag == expected_tag and actual == expected_bytes,(hex(entry),value,tag,actual.hex())
        results.append({'entry_va':hex(entry),'value':str(value),'native_tag':tag,
                        'native_bytes_hex':actual.hex(),'matches_independent_encoding':True,
                        'imports':dict(called)})
print(json.dumps({'scope':'synthetic integer payload helpers; no complete event or live traffic',
                  'unicorn_version':'2.1.4','instruction_cap_per_case':100000,
                  'timeout_microseconds_per_case':1000000,'cases':results},indent=2))
