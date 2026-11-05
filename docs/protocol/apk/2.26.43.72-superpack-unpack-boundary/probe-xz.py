"""Probe first twelve XZ headers in the verified ARM64 superpack data range.
Outputs metadata only; does not execute target code or restore inner libraries.
"""
import hashlib
import json
import lzma
from pathlib import Path
import sys
import zipfile
apk = Path(sys.argv[1])
assert hashlib.sha256(apk.read_bytes()).hexdigest() == "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
with zipfile.ZipFile(apk) as archive:
    carrier = archive.read("lib/arm64-v8a/libs.so")
assert hashlib.sha256(carrier).hexdigest() == "1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5"
data = carrier[0x870:0x870 + 0xdb6641]
magic = bytes.fromhex("fd377a585a00")
rows = []
offset = 0
while len(rows) < 12:
    offset = data.find(magic, offset)
    if offset < 0:
        break
    start = offset
    offset += 1
    row = {"archive_offset": start}
    try:
        decoder = lzma.LZMADecompressor(format=lzma.FORMAT_XZ, memlimit=128 * 1024 * 1024)
        output = decoder.decompress(data[start:], max_length=64 * 1024 * 1024)
        row.update(eof=decoder.eof, output_bytes=len(output), sha256=hashlib.sha256(output).hexdigest(), elf_magic=output.startswith(b"\x7fELF"))
        if decoder.eof:
            row["stream_bytes"] = len(data) - start - len(decoder.unused_data)
    except lzma.LZMAError as error:
        row["error"] = str(error)
    rows.append(row)
print(json.dumps({"xz_magic_matches": data.count(magic), "probe_limit": 12, "rows": rows}, indent=2))
