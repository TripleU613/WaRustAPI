#!/usr/bin/env python3
"""Reproduce pinned ARM64 KEM JNI/ML-KEM traces without executing the APK."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

LIBRARIES = {
    "libwhatsapp.so": "f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a",
    "libwhatsappmerged.so": "565f6c0e5581fe0b9ccc564f2d807d5363b8333abd32513e1722d6880d1c70aa",
}
RANGES = {
    "jni-encapsulate": ("libwhatsapp.so", 0x622f70, 0x6232a0),
    "jni-decapsulate": ("libwhatsapp.so", 0x622cac, 0x622f0c),
    "jni-dispatch": ("libwhatsapp.so", 0x617050, 0x6170b0),
    "ml-kem-exports": ("libwhatsappmerged.so", 0x2baba0, 0x2bac08),
    "ml-kem-dispatch": ("libwhatsappmerged.so", 0x70a6a8, 0x70a6d0),
    "ml-kem-encapsulate": ("libwhatsappmerged.so", 0x70a9dc, 0x70aa78),
}


def collect(root):
    for name, digest in LIBRARIES.items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"pinned library hash mismatch: {name}")
    files = {}
    for name, (library, start, end) in RANGES.items():
        output = subprocess.check_output([
            "llvm-objdump", "-d", f"--start-address={start}",
            f"--stop-address={end}", library,
        ], cwd=root, text=True)
        files[f"{name}.txt"] = output.encode()
    # Instruction anchors corroborate dispatch and widths, not primitive equivalence.
    jni = files["jni-encapsulate.txt"].decode()
    for anchor in ("cmp\tw24, #0x320", "cmp\tw24, #0x4a0",
                   "cmp\tw24, #0x620", "cmp\tw0, #0x20", "blr\tx28"):
        if anchor not in jni:
            raise ValueError(f"missing JNI anchor: {anchor}")
    dispatch = files["jni-dispatch.txt"].decode()
    for level in (512, 768, 1024):
        if f"OQS_KEM_ml_kem_{level}_encaps@plt" not in dispatch:
            raise ValueError(f"missing ML-KEM dispatch: {level}")
    native = files["ml-kem-encapsulate.txt"].decode()
    if "bl\t0x715a7c" not in native or "mov\tw1, #0x20" not in native:
        raise ValueError("missing native 32-byte randomness request")
    manifest = {
        "apk_build": "2.26.37.71",
        "apk_sha256": "1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c",
        "libraries": LIBRARIES,
        "ranges": {name: {"library": lib, "start": hex(start), "end_exclusive": hex(end)}
                   for name, (lib, start, end) in RANGES.items()},
        "files": {name: {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                  for name, data in files.items()},
        "coverage": "Static instruction trace; no target execution or primitive equivalence test.",
    }
    files["manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("library_directory", type=Path)
    parser.add_argument("output_directory", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files = collect(args.library_directory.resolve())
    if args.check:
        for name, data in files.items():
            if (args.output_directory / name).read_bytes() != data:
                raise ValueError(f"trace differs: {name}")
    else:
        args.output_directory.mkdir(parents=True, exist_ok=True)
        for name, data in files.items():
            (args.output_directory / name).write_bytes(data)
    print(f"Verified {len(files)} native KEM trace files")


if __name__ == "__main__":
    main()
