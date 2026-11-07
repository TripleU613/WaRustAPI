"""Interpret the retained straight-line 07q array initializer, not Android runtime."""
import hashlib
import json
from pathlib import Path
import re
import sys
source = Path(sys.argv[1]).read_bytes()
registers = {}
arrays = {}
operations = 0
for line in source.decode().splitlines():
    match = re.match(r'\s+([0-9a-f]+)\.\.[0-9a-f]+\s+(\S+)\s*(.*)', line)
    if not match:
        continue
    operations += 1
    _, op, args = match.groups()
    if op.startswith('const'):
        dest, value = re.match(r'(v\d+), #0x([0-9a-f]+)', args).groups()
        registers[dest] = int(value, 16)
    elif op == 'new-array':
        dest, length = re.match(r'(v\d+), (v\d+), \[I', args).groups()
        registers[dest] = [0] * registers[length]
    elif op == 'aput':
        value, array, index = re.match(r'(v\d+), (v\d+), (v\d+)', args).groups()
        registers[array][registers[index]] = registers[value]
    elif op == 'sput-object':
        value, name = re.match(r'(v\d+), LX/07q;->(\w+):\[I', args).groups()
        assert name not in arrays
        arrays[name] = list(registers[value])
    else:
        assert op == 'return-void', (op, args)
assert len(arrays) == 511
assert arrays['A8D'] == [0x300f1, 0x300f2, 0x300f3, 0x300f4, 0x300f5, 0x300f6, 0x300f7, 0x1096, 0xc6b]
assert arrays['A8E'] == [0x1c1af]
print(json.dumps({'source_sha256': hashlib.sha256(source).hexdigest(), 'source_bytes':len(source),
    'scope':'static straight-line initializer interpretation; not runtime execution',
    'operations':operations, 'arrays_checked':len(arrays),
    'selected_arrays':{name:[hex(value) for value in arrays[name]] for name in ['A8D','A8E']},
    'connection_member':{'id':hex(0x300f5), 'root_selector':(0x300f5 >> 14) & 0x7f, 'case':0x300f5 & 0x3fff}}, indent=2))
