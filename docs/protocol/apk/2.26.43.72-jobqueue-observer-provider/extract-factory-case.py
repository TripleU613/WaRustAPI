"""Retain exact selected factory context; input is complete xtask RNH.A00 text."""
import hashlib
import json
from pathlib import Path
import re
import sys
source = Path(sys.argv[1]).read_bytes()
lines = source.decode().splitlines()
assert any('LX/RNH;->A00(ILX/06h;Ljava/lang/Object;)Ljava/lang/Object;' in line for line in lines)
switch = next(line for line in lines if 'packed-switch ' in line)
assert '245:@0x103a' in switch
selected = []
for line in lines:
    match = re.match(r'\s+([0-9a-f]+)\.\.[0-9a-f]+', line)
    if match:
        pc = int(match[1], 16)
        if pc < 0x16 or 0x103a <= pc < 0x1046:
            selected.append(line)
    elif line.startswith('#') or line.startswith('.method') or 'full signature:' in line or 'exception table:' in line or line.startswith('.end method'):
        selected.append(line)
print(json.dumps({'source_sha256': hashlib.sha256(source).hexdigest(), 'source_bytes': len(source), 'scope': 'selected entry/switch and case245 only; other factory cases omitted'}, sort_keys=True))
print('\n'.join(selected))
