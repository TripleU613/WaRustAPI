"""Extract a labelled, hash-bound A0L view for the jobqueue app-set key."""
import hashlib
import json
from pathlib import Path
import re
import sys
source = Path(sys.argv[1]).read_bytes()
lines = source.decode().splitlines()
assert any('LX/1Uw;->A0L(ILX/06h;Ljava/lang/Object;)Ljava/lang/Object;' in line for line in lines)
assert any('7661:@0x292' in line for line in lines)
assert any('0292..0296  sget-object  v1, LX/07q;->A8E:[I' in line for line in lines)
assert any('0296..029a  goto/16  +0x2fa' in line for line in lines)
print(json.dumps({'source_sha256':hashlib.sha256(source).hexdigest(),'source_bytes':len(source),'scope':'A0L selected case7661 and shared return; other cases omitted'}))
for line in lines:
    match = re.match(r'\s+([0-9a-f]+)\.\.', line)
    if match:
        pc = int(match[1], 16)
        if pc < 10 or 0x292 <= pc < 0x29a or 0x88a <= pc < 0x896:
            print(line)
    elif line.startswith('#') or line.startswith('.method') or 'full signature:' in line or 'exception table:' in line or line.startswith('.end method'):
        print(line)
