"""Compare the emulated heapsort against independent Python sort invariants."""
import hashlib
import json
from pathlib import Path
import random
import re
import struct
from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
from unicorn.arm64_const import UC_ARM64_REG_SP, UC_ARM64_REG_LR, UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2, UC_ARM64_REG_X3, UC_ARM64_REG_PC
source = (Path(__file__).resolve().parent / 'reconstruct-members.py').read_text()
code = bytes.fromhex(re.search(r"u.mem_write\(0x50003000,bytes.fromhex\('([0-9a-f]+)'\)\)", source).group(1))
random_source = random.Random(4264372)
results = []
for count in [0, 1, 2, 10, 100, 1000]:
    for order in ['random', 'ascending', 'descending']:
        values = [(random_source.randrange(32), i) for i in range(count)]
        if order == 'ascending': values.sort()
        if order == 'descending': values.sort(reverse=True)
        machine = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
        machine.mem_map(0x100000, 0x1000)
        machine.mem_map(0x200000, 0x10000)
        machine.mem_map(0x300000, 0x10000)
        machine.mem_write(0x100000, code)
        machine.mem_write(0x100800, bytes.fromhex('c0035fd6'))
        def compare(uc, address, size, user):
            a = struct.unpack('<Q', uc.mem_read(uc.reg_read(UC_ARM64_REG_X0), 8))[0]
            b = struct.unpack('<Q', uc.mem_read(uc.reg_read(UC_ARM64_REG_X1), 8))[0]
            uc.reg_write(UC_ARM64_REG_X0, ((a > b) - (a < b)) & 0xffffffffffffffff)
        machine.hook_add(UC_HOOK_CODE, compare, begin=0x100800, end=0x100800)
        if values: machine.mem_write(0x300000, b''.join(struct.pack('<QQ', *x) for x in values))
        machine.reg_write(UC_ARM64_REG_SP, 0x20fff0)
        machine.reg_write(UC_ARM64_REG_LR, 0x100ffc)
        for register, value in [(UC_ARM64_REG_X0, 0x300000), (UC_ARM64_REG_X1, count),
                                (UC_ARM64_REG_X2, 16), (UC_ARM64_REG_X3, 0x100800)]:
            machine.reg_write(register, value)
        machine.emu_start(0x100000, 0x100ffc, timeout=5000000, count=10000000)
        assert machine.reg_read(UC_ARM64_REG_PC) == 0x100ffc
        raw = bytes(machine.mem_read(0x300000, count * 16)) if count else b''
        output = list(struct.iter_unpack('<QQ', raw))
        assert [x[0] for x in output] == sorted(x[0] for x in values)
        assert sorted(output) == sorted(values)
        results.append({'count': count, 'order': order, 'sorted_keys': True, 'preserved_records': True,
                        'output_sha256': hashlib.sha256(raw).hexdigest()})
print(json.dumps({'seed': 4264372, 'cases': results}, indent=2))
