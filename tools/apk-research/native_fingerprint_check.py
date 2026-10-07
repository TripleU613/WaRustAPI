#!/usr/bin/env python3
"""Check pinned APK fingerprint hash control flow with synthetic inputs only."""

import argparse
import hashlib
from pathlib import Path


LIBRARY_HASH = "f41f3fae89dbc18f8469c2c048df3dcf0648c193f9c7d60b878ac2014ec9551a"
# Reconstructed APK 2.26.37.71 libwhatsapp.so, shared V3/V4 helper.
# Caller/input and SHA wrapper evidence: docs/protocol/safety-number.md.
ENTRY, END = 0x713938, 0x7139E4
RETURN = 0x5000000


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
    args = parser.parse_args()
    if not args.library.is_file():
        parser.error("missing reconstructed native library")
    if hashlib.sha256(args.library.read_bytes()).hexdigest() != LIBRARY_HASH:
        parser.error("requires the pinned 37.71 reconstructed libwhatsapp.so")
    cases = ((b"synthetic seed", bytes(range(33))),
             (bytes(range(66)), bytes(range(66))))
    try:
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
