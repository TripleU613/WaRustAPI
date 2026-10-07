#!/usr/bin/env python3
"""Check pinned APK fingerprint hash flow or byte equality on synthetic inputs."""

import argparse
import hashlib
from pathlib import Path


LIBRARY_HASH = "f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a"
# Reconstructed APK 2.26.37.71 libwhatsapp.so, shared V3/V4 helper.
# Caller/input and SHA wrapper evidence: docs/protocol/safety-number.md.
ENTRY, END = 0x713938, 0x7139E4
RETURN = 0x5000000


def check_bytes(library, left, right, alignment=0):
    """Execute the actual equal-length primitive, without modeled callees."""
    from elftools.elf.elffile import ELFFile
    from unicorn import Uc, UcError, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
    from unicorn.arm64_const import (
        UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2,
        UC_ARM64_REG_LR, UC_ARM64_REG_PC,
    )

    if len(left) != len(right) or len(left) > 4096 or alignment not in (0, 1):
        raise ValueError("byte check requires equal bounded lengths and alignment 0/1")
    # APK 37.71 libwhatsapp.so: MCF-data equality 0x71f060 calls
    # 0xc9bbfc..0xc9bd20 after checking lengths. This leaf uses XOR/OR
    # accumulation in 32-byte, 8-byte and tail loops; no runtime calls.
    entry, end, page = 0xC9BBFC, 0xC9BD20, 0xC9B000
    guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
    with library.open("rb") as stream:
        elf = ELFFile(stream)
        if elf["e_machine"] != "EM_AARCH64":
            raise ValueError("expected ARM64 ELF")
        for segment in elf.iter_segments():
            if (segment["p_type"] == "PT_LOAD"
                    and segment["p_vaddr"] <= page
                    and page + 4096 <= segment["p_vaddr"] + segment["p_filesz"]):
                offset = page - segment["p_vaddr"]
                guest.mem_map(page, 4096)
                guest.mem_write(page, segment.data()[offset:offset + 4096])
                break
        else:
            raise ValueError("byte comparator not in a file-backed segment")
    # End each input exactly at an unmapped page, detecting an over-read.
    # Alignment 1 shifts it one byte earlier and leaves one sentinel byte.
    guest.mem_map(0x10000000, 8192)
    guest.mem_map(0x20000000, 8192)
    first = 0x10002000 - len(left) - alignment
    second = 0x20002000 - len(right) - alignment
    if left:
        guest.mem_write(first, left)
        guest.mem_write(second, right)
    instructions = 0

    def hook(cpu, address, size, user_data):
        nonlocal instructions
        if not entry <= address < end:
            raise ValueError(f"unexpected byte-comparator branch: {address:#x}")
        instructions += 1

    guest.hook_add(UC_HOOK_CODE, hook)
    guest.reg_write(UC_ARM64_REG_X0, first)
    guest.reg_write(UC_ARM64_REG_X1, second)
    guest.reg_write(UC_ARM64_REG_X2, len(left))
    guest.reg_write(UC_ARM64_REG_LR, RETURN)
    try:
        guest.emu_start(entry, RETURN, timeout=10_000_000, count=1_000_000)
    except UcError as error:
        raise ValueError(f"ARM64 byte comparison failed: {error}") from error
    if guest.reg_read(UC_ARM64_REG_PC) != RETURN:
        raise ValueError("byte comparison budget exhausted before return")
    result = guest.reg_read(UC_ARM64_REG_X0) & 0xFFFFFFFF
    expected = 0
    for first_byte, second_byte in zip(left, right):
        expected |= first_byte ^ second_byte
    if result != expected:
        raise ValueError("native byte comparison differs from XOR/OR equality")
    return result, instructions


def run_byte_cases(library):
    cases = 0
    # Boundaries exercise each vector/tail transition and 33-byte serialized
    # keys, 64-byte digests and 66-byte concatenations used by fingerprints.
    for length in (0, 1, 7, 8, 9, 15, 16, 31, 32, 33, 63, 64, 65, 66, 4096):
        left = bytes(index % 256 for index in range(length))
        for alignment in (0, 1):
            _, baseline = check_bytes(library, left, left, alignment)
            cases += 1
            # Every byte of small keys/digests; loop edges of the long case.
            positions = range(length) if length <= 66 else (0, 7, 8, 31, 32, length - 1)
            for position in positions:
                right = bytearray(left)
                right[position] ^= 0x80
                result, instructions = check_bytes(library, left, bytes(right), alignment)
                if result != 0x80 or instructions != baseline:
                    raise ValueError("mismatch result or length-driven instruction count differs")
                cases += 1
    print(f"synthetic byte cases={cases}: actual ARM64 XOR/OR results agree; "
          "equal-length instruction counts agree")
    print("No modeled callees. This does not prove end-to-end constant time or peer verification.")


def check_flow(library, seed, keys):
    from elftools.elf.elffile import ELFFile
    from unicorn import Uc, UcError, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
    from unicorn.arm64_const import (
        UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_LR,
        UC_ARM64_REG_PC, UC_ARM64_REG_SP,
    )

    guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
    # Map only the page containing the routine, not the client/runtime ELF.
    with library.open("rb") as stream:
        elf = ELFFile(stream)
        if elf["e_machine"] != "EM_AARCH64":
            raise ValueError("expected ARM64 ELF")
        for segment in elf.iter_segments():
            if (segment["p_type"] == "PT_LOAD"
                    and segment["p_vaddr"] <= ENTRY
                    and END <= segment["p_vaddr"] + segment["p_filesz"]):
                offset = 0x713000 - segment["p_vaddr"]
                guest.mem_map(0x713000, 4096)
                guest.mem_write(0x713000, segment.data()[offset:offset + 4096])
                break
        else:
            raise ValueError("fingerprint routine not in a file-backed segment")
    # These mapped addresses are intercepted before execution. SHA/MCF
    # operations are host models, not the native crypto/runtime implementation.
    for address in (0x711000, 0xCE8000, 0xCE6000):
        guest.mem_map(address, 4096)
    guest.mem_map(0x10000000, 0x100000)
    guest.reg_write(UC_ARM64_REG_SP, 0x100FF000)
    objects = {1: seed, 2: keys}
    contexts = {}
    next_id, rounds, updates, instructions = 3, 0, 0, 0

    def hook(cpu, address, size, user_data):
        nonlocal next_id, rounds, updates, instructions
        instructions += 1
        first = cpu.reg_read(UC_ARM64_REG_X0)
        second = cpu.reg_read(UC_ARM64_REG_X1)
        # 0x711828 creates SHA-512 context; 0x711878 updates with MCF data;
        # 0x711950 finishes into a 64-byte object and resets the context.
        # 0xce8c10/20 create/resize data; 0xce64e0 releases objects.
        if address == 0x711828:
            result = next_id
            next_id += 1
            contexts[result] = hashlib.sha512()
        elif address == 0xCE8C10:
            result = next_id
            next_id += 1
            objects[result] = b""
        elif address == 0xCE8C20:
            if second != 64:
                raise ValueError("unexpected native output length")
            objects[first] = bytes(second)
            result = 0
        elif address == 0x711878:
            contexts[first].update(objects[second])
            updates += 1
            result = 0
        elif address == 0x711950:
            if len(objects[second]) != 64:
                raise ValueError("unexpected native digest destination")
            objects[second] = contexts[first].digest()
            contexts[first] = hashlib.sha512()
            rounds += 1
            result = 0
        elif address == 0xCE64E0:
            result = 0
        elif ENTRY <= address < END:
            return
        else:
            raise ValueError(f"unmodeled native branch: {address:#x}")
        cpu.reg_write(UC_ARM64_REG_X0, result)
        cpu.reg_write(UC_ARM64_REG_PC, cpu.reg_read(UC_ARM64_REG_LR))

    guest.hook_add(UC_HOOK_CODE, hook)
    guest.reg_write(UC_ARM64_REG_X0, 1)
    guest.reg_write(UC_ARM64_REG_X1, 2)
    guest.reg_write(UC_ARM64_REG_LR, RETURN)
    # Single-threaded slice: genuine Unicorn wall/instruction limits, unlike
    # Qiling's multithreaded scheduler. PC proves actual return, not timeout.
    try:
        guest.emu_start(ENTRY, RETURN, timeout=10_000_000, count=1_000_000)
    except UcError as error:
        raise ValueError(f"ARM64 slice execution failed: {error}") from error
    if guest.reg_read(UC_ARM64_REG_PC) != RETURN:
        raise ValueError("native control-flow budget exhausted before return")
    result = objects[guest.reg_read(UC_ARM64_REG_X0)]
    expected = seed
    for _ in range(5200):
        expected = hashlib.sha512(expected + keys).digest()
    if result != expected or rounds != 5200 or updates != 10400:
        raise ValueError("native hash ordering/iteration count differs")
    return result, instructions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--check", choices=("hash", "bytes"), default="hash")
    args = parser.parse_args()
    if not args.library.is_file():
        parser.error("missing reconstructed native library")
    if hashlib.sha256(args.library.read_bytes()).hexdigest() != LIBRARY_HASH:
        parser.error("requires the pinned 37.71 reconstructed libwhatsapp.so")
    cases = ((b"synthetic seed", bytes(range(33))),
             (bytes(range(66)), bytes(range(66))))
    try:
        if args.check == "bytes":
            run_byte_cases(args.library)
            return
        for index, (seed, keys) in enumerate(cases, 1):
            digest, instructions = check_flow(args.library, seed, keys)
            print(f"synthetic case {index}: rounds=5200 updates=10400 "
                  f"instructions={instructions} digest={digest.hex()}")
    except ImportError as error:
        parser.exit(1, f"missing research dependency: {error.name}\n")
    except (ValueError, KeyError) as error:
        parser.exit(1, f"fingerprint control-flow check failed: {error}\n")
    print("Host-modeled SHA/MCF calls; native control flow only. No account verification.")


if __name__ == "__main__":
    main()
