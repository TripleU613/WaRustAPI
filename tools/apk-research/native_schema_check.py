#!/usr/bin/env python3
"""Read the pinned 37.71 native fingerprint protobuf descriptors offline."""

import argparse
import hashlib
import json
from pathlib import Path
import struct


LIBRARY_HASH = "565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa"
# APK 2.26.37.71 libwhatsappmerged.so, initializer/unpack code at
# 0x2d3db0..0x2d3e54. ABI/source evidence: docs/protocol/safety-number.md.
DESCRIPTORS = (0x740B60, 0x740C08)
TYPES = {6: "uint32", 13: "enum", 15: "bytes", 16: "message"}


def extract(library):
    from elftools.elf.elffile import ELFFile

    with library.open("rb") as stream:
        elf = ELFFile(stream)
        if (elf.elfclass != 64 or not elf.little_endian
                or elf["e_machine"] != "EM_AARCH64"):
            raise ValueError("expected little-endian ELF64 ARM64")
        segments = [(s["p_vaddr"], s.data()) for s in elf.iter_segments()
                    if s["p_type"] == "PT_LOAD"]
        relocations = elf.get_section_by_name(".rela.dyn")
        if relocations is None:
            raise ValueError("missing native relocation table")
        relative = {r["r_offset"]: r["r_addend"]
                    for r in relocations.iter_relocations()
                    if r["r_info_type"] == 1027}  # R_AARCH64_RELATIVE

    def data(address, length):
        for start, contents in segments:
            offset = address - start
            if 0 <= offset and offset + length <= len(contents):
                return contents[offset:offset + length]
        raise ValueError(f"descriptor read outside file-backed ELF: {address:#x}")

    def integer(address):
        return struct.unpack("<I", data(address, 4))[0]

    def pointer(address):
        return relative.get(address, struct.unpack("<Q", data(address, 8))[0])

    def string(address):
        result = bytearray()
        for offset in range(256):
            value = data(address + offset, 1)[0]
            if value == 0:
                return result.decode("ascii")
            result.append(value)
        raise ValueError("unterminated descriptor string")

    rows, enums = [], {}
    for address in DESCRIPTORS:
        if integer(address) != 0x28AAEEF9:
            raise ValueError("wrong ProtobufCMessageDescriptor magic")
        count, fields = integer(address + 48), pointer(address + 56)
        if not 1 <= count <= 16:
            raise ValueError("unexpected fingerprint field count")
        row = {"name": string(pointer(address + 8)), "address": hex(address),
               "size": pointer(address + 40), "fields": []}
        previous = 0
        for index in range(count):
            at = fields + index * 72
            tag, label, kind, quantifier, offset = struct.unpack("<IIIII", data(at + 8, 20))
            if tag <= previous or label != 1 or kind not in TYPES:
                raise ValueError("unsupported/unsorted fingerprint field descriptor")
            previous = tag
            field = {"name": string(pointer(at)), "tag": tag, "label": "optional",
                     "type": TYPES[kind], "address": hex(at),
                     "quantifier_offset": quantifier, "offset": offset}
            if offset >= row["size"] or quantifier >= row["size"]:
                raise ValueError("field offset exceeds native message size")
            target = pointer(at + 32)
            if kind == 16:
                if target not in DESCRIPTORS:
                    raise ValueError("unexpected nested fingerprint descriptor")
                field["message"] = string(pointer(target + 8))
            elif kind == 13:
                if integer(target) != 0x114315AF:
                    raise ValueError("wrong ProtobufCEnumDescriptor magic")
                name = string(pointer(target + 8))
                enum_count, values = integer(target + 40), pointer(target + 48)
                if not 1 <= enum_count <= 16:
                    raise ValueError("unexpected fingerprint enum count")
                enums[name] = {"address": hex(target), "values": [
                    {"name": string(pointer(values + i * 24)),
                     "value": struct.unpack("<i", data(values + i * 24 + 16, 4))[0]}
                    for i in range(enum_count)]}
                field["enum"] = name
            row["fields"].append(field)
        rows.append(row)
    return {"apk_build": "2.26.37.71", "library_sha256": LIBRARY_HASH,
            "evidence": "confirmed/static", "messages": rows, "enums": enums}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", required=True, type=Path)
    args = parser.parse_args()
    if not args.library.is_file():
        parser.error("missing reconstructed libwhatsappmerged.so")
    if hashlib.sha256(args.library.read_bytes()).hexdigest() != LIBRARY_HASH:
        parser.error("requires the pinned 37.71 reconstructed libwhatsappmerged.so")
    try:
        result = extract(args.library)
    except ImportError as error:
        parser.exit(1, f"missing research dependency: {error.name}\n")
    except (ValueError, struct.error) as error:
        parser.exit(1, f"native schema check failed: {error}\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
