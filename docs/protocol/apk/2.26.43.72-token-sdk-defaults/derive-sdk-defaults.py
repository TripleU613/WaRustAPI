"""Bounded static derivation for 09n.A01's four token defaults; no APK execution."""
import hashlib
import json
import re
from pathlib import Path

base = Path(__file__).parent
source = (base / "sdk-generated-defaults.txt").read_text()
specifiers = json.loads((base.parent / "2.26.43.72-token-mobileconfig-binding/specifiers.json").read_text())["selected"]
# These instruction anchors establish the index composition and remainder dispatch.
for anchor in (
    "const-wide/32  v2, #0xffff", "and-long  v0, v5, v2",
    "const/16  v0, #0x36", "ushr-long/2addr  v5, v0",
    "const-wide/16  v0, #0x3f", "and-long/2addr  v5, v0",
    "shl-int/lit8  v2, v0, #0x10", "or-int/2addr  v2, v3",
    "rem-int/lit8  v1, v2, #0x4",
    "002a..002e  if-eq  v1, v0, +0x16",
    "0030..0034  if-eq  v1, v0, +0xd",
):
    if anchor not in source:
        raise ValueError("index/branch coverage changed: " + anchor)
tables = {}
for pc, pairs in re.findall(r"^    ([0-9a-f]+)\.\.[0-9a-f]+  sparse-switch  v2, payload@0x[0-9a-f]+, cases=\[(.*)\], default@0x[0-9a-f]+$", source, re.M):
    rows = [(int(k), int(v, 16)) for k, v in re.findall(r"(\d+):@0x([0-9a-f]+)", pairs)]
    if len(dict(rows)) != len(rows):
        raise ValueError("duplicate switch key")
    tables[pc] = dict(rows)
if {pc: len(rows) for pc, rows in tables.items()} != {"003e":597,"004e":577,"005a":609,"0066":574}:
    raise ValueError("switch coverage changed")
selected = {}
for property_id in (865, 909, 996, 997):
    specifier = int(specifiers[str(property_id)]["specifier_hex"], 16)
    index = (specifier & 0xffff) | (((specifier >> 54) & 0x3f) << 16)
    remainder = index % 4
    pc = {0:"0066",1:"005a",2:"004e",3:"003e"}[remainder]
    target = tables[pc][index]
    match = re.search(rf"^    {target:04x}\.\.([0-9a-f]+)  const-wide/(?:16|32)  v0, #0x([0-9a-f]+)\n    \1\.\.[0-9a-f]+  return-wide  v0$", source, re.M)
    if not match:
        raise ValueError("selected case no longer returns literal directly")
    value = int(match[2], 16)
    if value != (604800 if property_id in (865,996) else 4):
        raise ValueError("selected SDK default changed")
    selected[str(property_id)] = {"specifier_hex":hex(specifier),"default_index":index,"remainder":remainder,"switch_pc":pc,"literal_pc":f"{target:04x}","value":value}
print(json.dumps({"source_sha256":hashlib.sha256(source.encode()).hexdigest(),"switch_entries_checked":sum(map(len,tables.values())),"selected":selected},indent=2))
