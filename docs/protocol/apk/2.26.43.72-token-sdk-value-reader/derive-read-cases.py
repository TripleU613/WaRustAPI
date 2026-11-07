"""Analyst-derived cases for retained SDK metadata branches; no APK execution."""
import hashlib
import json
from pathlib import Path

base = Path(__file__).parent
inputs = {name: (base / name).read_text() for name in (
    "specifier-type.txt", "metadata-sentinels.txt", "source-bit.txt",
    "value-source.txt", "long-read-overrides.txt", "metadata-read.txt", "vector-base.txt",
)}
anchors = {
    "specifier-type.txt": ["#0x30", "ushr-long/2addr", "#0x3f"],
    "metadata-sentinels.txt": ["#0xff (-1)", "#0xfe (-2)", "#0xfd (-3)", "#0x8", "#0x11"],
    "source-bit.txt": ["if-eqz  v1", "const/4  v0, #0x2"],
    "value-source.txt": ["#0x10", "#0x1", "DEFAULT__MISSING_SERVER_VALUE_INVALID_TYPE", "DEFAULT__MISSING_SERVER_VALUE_INVALID_SLOT_ID", "DEFAULT__SERVER_RETURNED_NULL_EMPTY_UNIT_ID"],
    "long-read-overrides.txt": ["#0x11", "#0x8", "ByteBuffer;->getLong(I)J", "return-wide  v11"],
    "metadata-read.txt": ["#0x4", "ByteBuffer;->getInt(I)I"],
    "vector-base.txt": ["add-int/lit8  v0, v2, #0x4"],
}
for name, required in anchors.items():
    if any(anchor not in inputs[name] for anchor in required):
        raise ValueError("retained branch anchors changed: " + name)

sentinels = {((-1 << 8) | 17): "missing_server_value", ((-2 << 8) | 17): "invalid_type", ((-3 << 8) | 17): "invalid_slot"}
def source(meta):
    if meta in sentinels:
        return sentinels[meta]
    if meta == 17:
        return "missing_server_value"
    return {0:"server",1:"server_returned_null",2:"server_returned_null_empty_unit",3:"missing_server_value"}[ (2 if meta & 16 else 0) | (1 if meta & 1 else 0)]

specifiers = json.loads((base.parent / "2.26.43.72-token-mobileconfig-binding/specifiers.json").read_text())["selected"]
selected = {}
for key, row in specifiers.items():
    spec = int(row["specifier_hex"], 16)
    selected[key] = {"type": (spec >> 48) & 63, "config_index": (spec >> 32) & 65535, "slot": spec & 65535}
cases = [{"metadata":m, "source":source(m), "long_reader_uses_default_mask":bool(m & 17)} for m in [*range(32), *sentinels]]
print(json.dumps({"kind":"analyst-derived static branch cases", "input_sha256":{name:hashlib.sha256(text.encode()).hexdigest() for name,text in inputs.items()},"token_specifiers":selected,"cases":cases},indent=2))
