"""Check this pinned descriptor's selected presence indexes, offline only."""
import ast
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parent
text = (root / "video-descriptor.txt").read_text()
line = next(line for line in text.splitlines() if "const-string" in line and r"\u{1}\u{1f}" in line)
literal = line[line.index('"'):]
literal = re.sub(r"\\u\{([0-9a-fA-F]+)\}", lambda match: chr(int(match[1], 16)), literal)
values = list(map(ord, ast.literal_eval(literal)))
# This selected descriptor uses only single-word integers. Refuse new forms;
# do not silently treat UTF16 continuation units as complete integers.
assert all(value < 0xD800 for value in values)
assert values[:10] == [1, 31, 0, 1, 1, 34, 31, 0, 2, 0]
rows = {}
pos = 10
for _ in range(values[1]):
    tag, kind = values[pos:pos + 2]
    pos += 2
    presence = None
    if kind & 0x1000:
        presence = values[pos]
        pos += 1
    assert tag not in rows
    rows[tag] = presence
assert pos == len(values)
assert [rows[tag] for tag in (6, 18, 20)] == [5, 15, 17]
print(json.dumps({
    "mediaKey_tag6": {"presence_index": rows[6], "mask": "0x20"},
    "streamingSidecar_tag18": {"presence_index": rows[18], "mask": "0x8000"},
    "viewOnce_tag20": {"presence_index": rows[20], "mask": "0x20000"},
}, indent=2))
