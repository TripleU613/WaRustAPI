"""Decode only this pinned sticker descriptor's normal-field prefix.

Names/slots below are explicit observations of the retained array/helper writes;
this is not a replacement for the general tool's refused oneof name pairing.
"""
import ast
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parent
text = (root / 'sticker-descriptor.txt').read_text()
line = next(line for line in text.splitlines() if 'const-string' in line and r'\u{1}\u{17}' in line)
literal = line[line.index('"'):]
literal = re.sub(r'\\u\{([0-9a-fA-F]+)\}', lambda match: chr(int(match[1],16)), literal)
values = list(map(ord, ast.literal_eval(literal)))
assert all(value < 0xD800 for value in values)
assert values[:10] == [1,23,1,1,1,26,23,0,0,0]
observed_slots = {
    3: 'url', 4: 'fileSha256', 5: 'fileEncSha256',
    6: 'mediaKey', 7: 'mimetype', 11: 'fileLength', 21: 'isLottie',
}
rows = []
pos = 10
for index in range(22):
    tag, kind, presence = values[pos:pos+3]
    pos += 3
    assert kind & 0x1000 and kind & 0xFF <= 17
    assert presence == index
    slot = 3 + index  # oneof value/case slots0/1, presence word slot2
    if slot in observed_slots:
        rows.append({'name':observed_slots[slot], 'tag':tag, 'object_slot':slot, 'presence_index':presence})
# Leave the new audio oneof unpaired by this normal-prefix checker.
assert values[pos:] == [26,60,0]
assert {row['name']:row['tag'] for row in rows} == {
    'url':1,'fileSha256':2,'fileEncSha256':3,'mediaKey':4,
    'mimetype':5,'fileLength':9,'isLottie':21,
}
print(json.dumps({'selected_normal_fields':rows,'unpaired_tail':[26,60,0]},indent=2))
