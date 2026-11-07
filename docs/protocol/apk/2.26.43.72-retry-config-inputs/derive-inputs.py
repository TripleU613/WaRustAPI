"""Pinned static retry-config entry derivation; no APK execution or account access."""
import hashlib
import json
from pathlib import Path
import re

base = Path(__file__).resolve().parent

def source(name):
    raw = (base / name).read_bytes()
    return raw.decode(), hashlib.sha256(raw).hexdigest()

def prefix(text, end):
    rows = [(int(pc, 16), op, args.strip()) for pc, op, args in
            re.findall(r"^\s+([0-9a-f]+)\.\.\S+\s+(\S+)[ \t]*(.*)$", text, re.M)
            if int(pc, 16) < end]
    allowed = {"const", "const/16", "const/4", "const/high16", "invoke-static",
               "invoke-virtual", "move-object/from16", "move-result-object"}
    if not rows or any(op not in allowed for _, op, _ in rows):
        raise ValueError("changed prefix opcode/control-flow coverage")
    return rows

def writes(rows, register):
    return [(pc, op, args) for pc, op, args in rows
            if not op.startswith("invoke") and args.split(",")[0].strip() == register]

integer, integer_hash = source("integer-default-helper.txt")
rows = prefix(integer, 0x16d8)
assert writes(rows, "v12") == [(0x56, "move-result-object", "v12")]
assert writes([row for row in rows if row[0] < 0x56], "v2") == [(0, "const/4", "v2, #0x0")]
assert any(pc == 0x50 and args == "{v2}, Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;" for pc, _, args in rows)
assert any(pc == 0x16c6 and args == "v0, #0x53e5" for pc, _, args in rows)
assert rows[-1] == (0x16d2, "invoke-virtual", "{v14, v0, v12}, Lcom/google/common/collect/ImmutableMap$Builder;->put(Ljava/lang/Object;Ljava/lang/Object;)Lcom/google/common/collect/ImmutableMap$Builder;")
boolean, boolean_hash = source("boolean-default-helper.txt")
rows = prefix(boolean, 0xe8a)
assert writes(rows, "v1") == [(0, "const/4", "v1, #0x0"), (0x16, "move-result-object", "v1")]
assert any(pc == 0x10 and args == "{v1}, Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;" for pc, _, args in rows)
assert any(pc == 0xe78 and args == "v0, #0x5d4c" for pc, _, args in rows)
assert rows[-1] == (0xe84, "invoke-virtual", "{v4, v0, v1}, Lcom/google/common/collect/ImmutableMap$Builder;->put(Ljava/lang/Object;Ljava/lang/Object;)Lcom/google/common/collect/ImmutableMap$Builder;")
lookup, lookup_hash = source("../2.26.43.72-token-mobileconfig-binding/lookup-map.txt")
pattern = (r"(?:const/16|const)  v2, #0x([0-9a-f]+)\n"
           r"\s+([0-9a-f]+)\.\.\S+  const-wide(?:/high16)?  v0, #0x([0-9a-f]+)"
           r"(?: \([^\n]*\))?\n\s+[0-9a-f]+\.\.\S+  aput-wide  v0, v3, v2")
entries = list(re.finditer(pattern, lookup))
assert len(entries) == lookup.count("aput-wide") == 2065
keys = [int(entry[1], 16) for entry in entries]
assert len(set(keys)) == 2065 and all(key < 0xa078 for key in keys)
assert 21477 not in keys and 23884 not in keys
print(json.dumps({"integer_source_sha256": integer_hash,
                  "boolean_source_sha256": boolean_hash,
                  "lookup_source_sha256": lookup_hash,
                  "literal_lookup_writes_checked": len(entries),
                  "selected_default_entries": {"21477": 0, "23884": False},
                  "concrete_initial_specifiers": {"21477": "0x0", "23884": "0x0"},
                  "scope": "Selected insertions and initial lookup, not effective runtime values or complete buildKeepingLast precedence"}, indent=2))
