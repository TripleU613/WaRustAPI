#!/usr/bin/env python3
"""Run the local ARM64 decoder probe under QEMU in a network-isolated rootfs."""

import argparse
import hashlib
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rootfs", required=True, type=Path)
    parser.add_argument("--check", choices=("decoder-init", "decoder-full-archive"), default="decoder-init")
    parser.add_argument("--timeout-seconds", type=int, default=30)
    args = parser.parse_args()
    if not 1 <= args.timeout_seconds <= 600:
        parser.error("--timeout-seconds must be between 1 and 600")
    root = args.rootfs.resolve()
    full = args.check == "decoder-full-archive"
    required = ["bin/decoder_probe", "system/bin/linker64", "system/lib64/libc.so",
                "system/lib64/libdl.so", "system/lib64/libsuperpack.so",
                "system/lib64/libandroid.so", "system/lib64/liblog.so"]
    if full:
        required.append("system/lib64/wa_archive.so")
    for name in required:
        if not (root / name).is_file():
            parser.error(f"missing decoder fixture: {root / name}")
    # Exact inputs extracted from APK 2.26.37.71; do not silently reuse this
    # build's protocol/reconstruction conclusions for another decoder/archive.
    pinned = {"system/lib64/libsuperpack.so":
              "fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f"}
    if full:
        pinned["system/lib64/wa_archive.so"] = (
            "24dfc7c0b9c3a13d1a9bd875304035a9f5ca6aa6f8ff904aef6ec1cfe3418c6b"
        )
    for name, expected in pinned.items():
        if hashlib.sha256((root / name).read_bytes()).hexdigest() != expected:
            parser.error(f"wrong 37.71 artifact hash: {name}")
    destination = root / "decoded"
    if full and destination.exists() and (
        not destination.is_dir() or any(destination.iterdir())
    ):
        parser.error("decoder-full-archive requires an empty rootfs/decoded directory")
    bwrap = shutil.which("bwrap")
    qemu = shutil.which("qemu-aarch64-static")
    if not bwrap or not qemu:
        parser.error("requires bwrap and qemu-aarch64-static")

    # Qiling reads non-executable fixtures; QEMU's Linux ELF loader requires
    # executable mode on the probe/interpreter. Change only these local copies.
    for name in ("bin/decoder_probe", "system/bin/linker64"):
        path = root / name
        path.chmod(path.stat().st_mode | 0o100)
    for name in ("proc", "dev"):
        (root / name).mkdir(mode=0o700, exist_ok=True)
    placeholder = root / "qemu"
    if not placeholder.exists():
        with placeholder.open("xb"):
            pass
        placeholder.chmod(0o600)
    command = [bwrap, "--die-with-parent", "--unshare-net", "--unshare-pid",
               "--ro-bind", str(root), "/"]
    if full:
        destination.mkdir(mode=0o700, exist_ok=True)
        command += ["--bind", str(destination), "/decoded"]
    command += ["--ro-bind", qemu, "/qemu", "--proc", "/proc", "--dev", "/dev",
                "--", "/qemu", "/bin/decoder_probe"]
    if full:
        command.append("--full-archive")
    try:
        result = subprocess.run(command, env={}, timeout=args.timeout_seconds, check=False)
    except subprocess.TimeoutExpired:
        parser.exit(1, "QEMU check failed: wall-clock budget exhausted\n")
    print(f"QEMU process exit: {result.returncode}")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
