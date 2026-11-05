"""Check every literal array write before selecting four token specifiers.

Reads retained DEX text only. No APK execution, account access or network.
This is a pinned-method derivation, not a general Dalvik interpreter.
"""
import hashlib
import json
from pathlib import Path
import re

source = Path(__file__).with_name("lookup-map.txt").read_bytes()
text = source.decode("utf-8")
pattern = (r"(?:const/16|const)  v2, #0x([0-9a-f]+)\n"
           r"\s+([0-9a-f]+)\.\.\S+  const-wide(?:/high16)?  v0, #0x([0-9a-f]+)"
           r"(?: \([^\n]*\))?\n\s+[0-9a-f]+\.\.\S+  aput-wide  v0, v3, v2")
writes = list(re.finditer(pattern, text))
if len(writes) != text.count("aput-wide") or len(writes) != 2065:
    raise SystemExit("unrecognized or changed array-write coverage")
entries = {}
for write in writes:
    key, address, value = write.groups()
    key = int(key, 16)
    if key in entries or key >= 0xA078:
        raise SystemExit("duplicate/out-of-bounds literal index")
    entries[key] = {"specifier_hex": "0x" + value, "value_instruction": address}
selected = {str(key): entries[key] for key in (865, 909, 996, 997)}
expected = ("0x8201a700000065", "0x8201bc0000006e", "0x8201e30000007d", "0x8201e40000007e")
if tuple(row["specifier_hex"] for row in selected.values()) != expected:
    raise SystemExit("token mapping changed")
print(json.dumps({"source_sha256": hashlib.sha256(source).hexdigest(),
                  "literal_writes_checked": len(writes), "array_length": 0xA078,
                  "selected": selected}, indent=2))
