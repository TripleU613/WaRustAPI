#!/usr/bin/env python3
"""Execute pinned APK ML-KEM-1024 primitives against retained NIST vectors offline."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from native_kem_trace import LIBRARIES

RETURN = 0x1003f000


def execute(library, operation, inputs):
    from elftools.elf.elffile import ELFFile
    from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
    from unicorn.arm64_const import (
        UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2, UC_ARM64_REG_X3,
        UC_ARM64_REG_SP, UC_ARM64_REG_LR, UC_ARM64_REG_PC, UC_ARM64_REG_TPIDR_EL0,
    )
    guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
    with library.open('rb') as stream:
        elf = ELFFile(stream)
        if elf['e_machine'] != 'EM_AARCH64':
            raise ValueError('expected ARM64 ELF')
        pages = set()
        for segment in elf.iter_segments():
            if segment['p_type'] == 'PT_LOAD':
                start, size = segment['p_vaddr'], segment['p_memsz']
                for address in range(start & ~4095, (start + size + 4095) & ~4095, 4096):
                    if address not in pages:
                        guest.mem_map(address, 4096)
                        pages.add(address)
                guest.mem_write(start, segment.data())
        for relocation in elf.get_section_by_name('.rela.dyn').iter_relocations():
            if relocation['r_info_type'] == 1027:  # R_AARCH64_RELATIVE
                guest.mem_write(relocation['r_offset'], struct.pack('<Q', relocation['r_addend']))
    guest.mem_map(0x10000000, 0x40000)
    guest.reg_write(UC_ARM64_REG_SP, 0x10030000)
    guest.reg_write(UC_ARM64_REG_TPIDR_EL0, 0x10000000)
    guest.reg_write(UC_ARM64_REG_LR, RETURN)
    heap, once, modeled = 0x10018000, set(), {}
    registers = (UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2, UC_ARM64_REG_X3)

    def callback(cpu, address, size, _):
        nonlocal heap
        args = [cpu.reg_read(register) for register in registers]
        if address in (0x71daf0, 0x71db00, 0x71d900):  # memset, memcpy, __memcpy_chk
            destination, value, length = args[:3]
            if length > 16384 or (address == 0x71d900 and length > args[3]):
                raise ValueError('unbounded or overflowing memory callback')
            data = bytes([value & 255]) * length if address == 0x71daf0 else bytes(cpu.mem_read(value, length))
            if data:
                cpu.mem_write(destination, data)
        elif address == 0x71db20:  # posix_memalign
            destination, alignment, length = args[:3]
            if not 8 <= alignment <= 4096 or alignment & (alignment - 1) or length > 16384:
                raise ValueError('invalid allocation')
            heap = (heap + alignment - 1) & ~(alignment - 1)
            if heap + length > 0x10028000:
                raise ValueError('allocation budget exhausted')
            cpu.mem_write(destination, struct.pack('<Q', heap))
            heap += length
            cpu.reg_write(UC_ARM64_REG_X0, 0)
        elif address == 0x71d580:  # free: isolated guest allocation lifetime
            pass
        elif address == 0x71e0e0:  # pthread_once: execute the actual native initializer
            if args[0] not in once:
                once.add(args[0])
                modeled[hex(address)] = modeled.get(hex(address), 0) + 1
                cpu.reg_write(UC_ARM64_REG_PC, args[1])
                return
        else:
            if not (0x70a000 <= address < 0x71d000 or 0x2b5000 <= address < 0x2bd000):
                raise ValueError(f'unexpected native branch: {address:#x}')
            return
        modeled[hex(address)] = modeled.get(hex(address), 0) + 1
        cpu.reg_write(UC_ARM64_REG_PC, cpu.reg_read(UC_ARM64_REG_LR))

    guest.hook_add(UC_HOOK_CODE, callback)
    outputs = (0x10004000, 0x10008000)
    if operation == 'keyGen':
        entry, lengths = 0x70a6d0, (1568, 3168)
        seed = bytes.fromhex(inputs['d'] + inputs['z'])
        if len(seed) != 64:
            raise ValueError('key generation seed width')
        guest.mem_write(0x10001000, seed)
        args = (*outputs, 0x10001000)
    elif operation == 'encapsulation':
        entry, lengths = 0x70a7d0, (1568, 32)
        public, seed = bytes.fromhex(inputs['ek']), bytes.fromhex(inputs['m'])
        if len(public) != 1568 or len(seed) != 32:
            raise ValueError('encapsulation input width')
        guest.mem_write(0x10001000, public)
        guest.mem_write(0x10002000, seed)
        args = (*outputs, 0x10001000, 0x10002000)
    elif operation == 'decapsulation':
        entry, lengths = 0x2babe8, (32,)
        secret, ciphertext = bytes.fromhex(inputs['dk']), bytes.fromhex(inputs['c'])
        if len(secret) != 3168 or len(ciphertext) != 1568:
            raise ValueError('decapsulation input width')
        guest.mem_write(0x10001000, secret)
        guest.mem_write(0x10002000, ciphertext)
        args = (outputs[0], 0x10002000, 0x10001000)
    else:
        raise ValueError('unsupported operation')
    for register, value in zip(registers, args):
        guest.reg_write(register, value)
    guest.emu_start(entry, RETURN, timeout=20_000_000, count=10_000_000)
    if guest.reg_read(UC_ARM64_REG_PC) != RETURN:
        raise ValueError('native execution budget exhausted before return')
    result = guest.reg_read(UC_ARM64_REG_X0) & 0xffffffff
    return result, [bytes(guest.mem_read(address, length))
                    for address, length in zip(outputs, lengths)], modeled


def check(library, fixtures):
    if hashlib.sha256(library.read_bytes()).hexdigest() != LIBRARIES['libwhatsappmerged.so']:
        raise ValueError('pinned library hash mismatch')
    rows = []
    for case in fixtures['cases']:
        result, outputs, modeled = execute(library, case['operation'], case['input'])
        fields = {'keyGen': ('ek', 'dk'), 'encapsulation': ('c', 'k'), 'decapsulation': ('k',)}
        expected = [bytes.fromhex(case['expected'][field]) for field in fields[case['operation']]]
        if result != 0 or outputs != expected:
            raise ValueError(f"NIST mismatch: {case['operation']} tcId={case['tcId']} return={result}")
        rows.append(dict(operation=case['operation'], tcId=case['tcId'], return_code=result,
                         output_sha256=[hashlib.sha256(value).hexdigest() for value in outputs],
                         modeled_callbacks=modeled))
    return dict(apk_build='2.26.37.71', library_sha256=LIBRARIES['libwhatsappmerged.so'],
                fixture_sha256=hashlib.sha256(json.dumps(fixtures, sort_keys=True).encode()).hexdigest(),
                scope='Actual native ML-KEM-1024 arithmetic and hashing; modeled libc memory/allocation and pthread_once; deterministic inner encapsulation; no JNI, RNG, account, network or hybrid session.',
                cases=rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('library', type=Path)
    parser.add_argument('fixtures', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    result = check(args.library, json.loads(args.fixtures.read_text()))
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(f"Passed {len(result['cases'])} NIST ML-KEM-1024 cases using native arithmetic/hashing")


if __name__ == '__main__':
    main()
