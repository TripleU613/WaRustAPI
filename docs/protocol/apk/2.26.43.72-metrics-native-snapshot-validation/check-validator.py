"""Execute the pinned local validator on synthetic groups, without APK runtime/I/O."""
import contextlib
import io
import json
from pathlib import Path
import runpy
import struct
from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE, UC_HOOK_INTR
from unicorn.arm64_const import UC_ARM64_REG_SP, UC_ARM64_REG_LR, UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_PC
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
def imported(uc,address,size,user):
    name = imports[address]
    called[name] = called.get(name,0)+1
    if name == 'MCFRetain': return  # identity return
    if name == 'MCFRelease':
        uc.reg_write(UC_ARM64_REG_X0,0)
        return
    failures.append('unsupported import '+name)
    uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt')
    machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_INTR,interrupted)
root,state = 0x10000000,0x10000100
machine.mem_write(root+0x18,struct.pack('<Q',state))
# Native selector order from the traced switch: state +30,+28,+18,+20.
slots = [0x30,0x28,0x18,0x20]
for group,slot in enumerate(slots):
    machine.mem_write(state+slot,struct.pack('<Q',0x10001000+group*0x100))
valid = [0,0,1,1]
cases = [('valid', [valid[:] for _ in slots], 0)]
for group in range(4):
    for field,value,code in [(0,-1,9),(1,-1,10),(2,0,11),(2,65536,12),(3,0,13),(3,65536,14),
                             (0,2147483647,0),(1,2147483647,0),(2,65535,0),(3,65535,0)]:
        groups = [valid[:] for _ in slots]
        groups[group][field] = value
        cases.append((f'g{group}-field{field}-{value}',groups,code))
# A later failing comparison wins inside one group; first failing group stops the loop.
groups = [valid[:] for _ in slots]; groups[0] = [-1,-1,0,65536]
cases.append(('same-group-error-precedence',groups,14))
groups = [valid[:] for _ in slots]; groups[0][2] = 0; groups[1][3] = 65536
cases.append(('first-failing-group',groups,11))
results = []
for name,groups,expected in cases:
    for group,values in enumerate(groups):
        machine.mem_write(0x10001000+group*0x100+0x18,struct.pack('<4i',*values))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    machine.reg_write(UC_ARM64_REG_X0,root)
    machine.reg_write(UC_ARM64_REG_X1,0)  # no error-object allocation
    called.clear(); failures.clear()
    machine.emu_start(0x71fc84,0x30000000,timeout=1000000,count=100000)
    assert not failures, failures
    assert machine.reg_read(UC_ARM64_REG_PC) == 0x30000000
    actual = machine.reg_read(UC_ARM64_REG_X0)
    assert actual == expected, (name,actual,expected)
    results.append({'case':name,'groups':groups,'expected':expected,'actual':actual,'imports':dict(called)})
print(json.dumps({'scope':'synthetic local validator emulation; no Android runtime or I/O',
                  'unicorn_version':'2.1.4','instruction_cap_per_case':100000,
                  'timeout_microseconds_per_case':1000000,'cases':results},indent=2))
