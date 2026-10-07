#!/usr/bin/env python3
"""Execute pinned APK protobuf-C fingerprint unpacking on built-in synthetic bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from native_schema_check import LIBRARY_HASH
from native_fingerprint_check import RETURN


class NativeParser:
    """Isolated native parser slice with modeled allocation/copy/set operations."""
    def __init__(self, library):
        from elftools.elf.elffile import ELFFile
        from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
        from unicorn import arm64_const as reg
        self.reg = reg
        self.guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
        guest = self.guest
        with library.open('rb') as stream:
            elf = ELFFile(stream)
            if elf.elfclass != 64 or not elf.little_endian or elf['e_machine'] != 'EM_AARCH64':
                raise ValueError('requires little-endian ARM64 ELF')
            # Full file-backed PT_LOAD image is data/instruction storage only;
            # hook permits just unpack/free/internal-helper/generated-init ranges.
            for segment in elf.iter_segments():
                if segment['p_type'] == 'PT_LOAD':
                    start = segment['p_vaddr'] & ~4095
                    end = (segment['p_vaddr'] + segment['p_memsz'] + 4095) & ~4095
                    guest.mem_map(start, end-start)
                    guest.mem_write(segment['p_vaddr'], segment.data())
            for section_name in ('.rela.dyn', '.rela.plt'):
                section = elf.get_section_by_name(section_name)
                if section is None:
                    continue
                symbols = elf.get_section(section['sh_link'])
                for reloc in section.iter_relocations():
                    kind = reloc['r_info_type']
                    if kind == 1027:  # R_AARCH64_RELATIVE, zero image base
                        target = reloc['r_addend']
                    elif kind in (257, 1025, 1026):  # ABS64/GLOB_DAT/JUMP_SLOT
                        symbol = symbols.get_symbol(reloc['r_info_sym'])
                        if symbol['st_shndx'] == 'SHN_UNDEF':
                            continue  # direct memcpy/memset PLT hooks only
                        target = symbol['st_value'] + reloc['r_addend']
                    else:
                        continue
                    guest.mem_write(reloc['r_offset'], struct.pack('<Q', target))
        guest.mem_map(0x10000000, 0x10000)  # scratch/stack/TLS/custom allocator
        guest.mem_map(0x20000000, 0x10000)  # at most 64-KiB synthetic input
        guest.mem_map(0x30000000, 0x100000)  # bounded modeled heap
        guest.mem_map(0x40000000, 0x1000)  # intercepted allocation/free entries
        self.allocator = 0x10000100
        guest.mem_write(self.allocator, struct.pack('<QQQ', 0x40000000, 0x40000004, 0))
        self.heap_next = 0x30000000
        self.live = {}
        self.instructions = 0
        self.allocated = 0

        def hook(cpu, address, size, data):
            self.instructions += 1
            if (0x2D3DB0 <= address < 0x2D3E54 or 0x703158 <= address < 0x7057E0):
                return
            first = cpu.reg_read(reg.UC_ARM64_REG_X0)
            second = cpu.reg_read(reg.UC_ARM64_REG_X1)
            third = cpu.reg_read(reg.UC_ARM64_REG_X2)
            if address == 0x40000000:
                count = max(second, 1)
                count = (count + 15) & ~15
                if count > 0x100000 or self.heap_next + count > 0x30100000:
                    raise ValueError('native parser exceeded modeled heap budget')
                result = self.heap_next
                self.heap_next += count
                self.live[result] = count
                self.allocated += count
            elif address == 0x40000004:
                if second and self.live.pop(second, None) is None:
                    raise ValueError('native parser freed an unknown allocation')
                result = 0
            elif address == 0x71DAF0:  # pinned memset PLT
                if third > 0x100000:
                    raise ValueError('modeled memset exceeds budget')
                if third:
                    cpu.mem_write(first, bytes([second & 255]) * third)
                result = first
            elif address == 0x71DB00:  # pinned memcpy PLT
                if third > 0x100000:
                    raise ValueError('modeled memcpy exceeds budget')
                if third:
                    cpu.mem_write(first, bytes(cpu.mem_read(second, third)))
                result = first
            else:
                raise ValueError(f'unmodeled native parser branch: {address:#x}')
            cpu.reg_write(reg.UC_ARM64_REG_X0, result)
            cpu.reg_write(reg.UC_ARM64_REG_PC, cpu.reg_read(reg.UC_ARM64_REG_LR))
        guest.hook_add(UC_HOOK_CODE, hook)

    def run(self, entry):
        from unicorn import UcError
        self.guest.reg_write(self.reg.UC_ARM64_REG_LR, RETURN)
        try:
            self.guest.emu_start(entry, RETURN, timeout=10_000_000, count=1_000_000)
        except UcError as error:
            raise ValueError(f'native parser execution failed: {error}') from error
        if self.guest.reg_read(self.reg.UC_ARM64_REG_PC) != RETURN:
            raise ValueError('native parser budget exhausted before return')
        return self.guest.reg_read(self.reg.UC_ARM64_REG_X0)

    def integer(self, address):
        return struct.unpack('<I', self.guest.mem_read(address, 4))[0]

    def pointer(self, address):
        return struct.unpack('<Q', self.guest.mem_read(address, 8))[0]

    def individual(self, address):
        if not address:
            return None
        if self.pointer(address) != 0x740B60:
            raise ValueError('unexpected parsed individual descriptor')
        result = {'unknown_fields':self.integer(address+8)}
        for name, flag, value in [('public_key',24,32),('pn_identifier',48,56),
                                  ('lid_identifier',72,80),('username_identifier',96,104),
                                  ('hashed_public_key',128,136)]:
            if not self.integer(address+flag):
                result[name] = None
                continue
            length = self.pointer(address+value)
            pointer = self.pointer(address+value+8)
            if length > 65536:
                raise ValueError('parsed field exceeds synthetic input bound')
            result[name] = bytes(self.guest.mem_read(pointer,length)).hex() if length else ''
        result['hosted_state'] = struct.unpack('<i', self.guest.mem_read(address+124,4))[0] if self.integer(address+120) else None
        return result

    def parse(self, payload):
        if len(payload) > 65536:
            raise ValueError('synthetic payload exceeds 64 KiB')
        guest, reg = self.guest, self.reg
        if payload:
            guest.mem_write(0x20000000, payload)
        guest.reg_write(reg.UC_ARM64_REG_SP, 0x1000F000)
        guest.reg_write(reg.UC_ARM64_REG_TPIDR_EL0, 0x1000E000)
        guest.reg_write(reg.UC_ARM64_REG_X0, self.allocator)
        guest.reg_write(reg.UC_ARM64_REG_X1, len(payload))
        guest.reg_write(reg.UC_ARM64_REG_X2, 0x20000000)
        address = self.run(0x2D3E28)
        if address:
            if self.pointer(address) != 0x740C08:
                raise ValueError('unexpected parsed combined descriptor')
            result = {'accepted':True, 'version':self.integer(address+28) if self.integer(address+24) else None,
                      'unknown_fields':self.integer(address+8),
                      'local':self.individual(self.pointer(address+32)),
                      'remote':self.individual(self.pointer(address+40))}
            guest.reg_write(reg.UC_ARM64_REG_X0,address)
            guest.reg_write(reg.UC_ARM64_REG_X1,self.allocator)
            self.run(0x2D3E44)
        else:
            result = {'accepted':False}
        if self.live:
            raise ValueError('native unpack/free left modeled allocations live')
        result.update(instructions=self.instructions, modeled_allocated_bytes=self.allocated)
        return result


def cases():
    fixture = json.loads((Path(__file__).resolve().parents[2] / 'crates/whatsapp-api/fixtures/fingerprint/native-qr-3771.json').read_text())
    return [('wire-version-0',bytes.fromhex(fixture['version_0_hex'])),
            ('wire-version-1',bytes.fromhex(fixture['version_1_hex'])),
            ('valid-wire-with-unknown-group',bytes.fromhex(fixture['version_0_hex'])+bytes([35,36])),
            ('empty',b''),('missing-version',bytes([0x20,0])),
            ('duplicate-version',bytes([8,0,8,1])),
            ('unknown-enum',bytes([8,0,18,2,40,2])),
            ('duplicate-local-message',bytes([8,0,18,3,26,1,65,18,3,34,1,66])),
            ('truncated',bytes([8])),('invalid-tag-zero',bytes([0])),
            ('duplicate-lid-bytes',bytes([8,0,18,6,26,1,65,26,1,66])),
            ('duplicate-lid-nested',bytes([8,0,18,3,26,1,65,18,3,26,1,66])),
            ('empty-local-message',bytes([8,0,18,0])),
            ('empty-key-presence',bytes([8,0,18,2,10,0])),
            ('unknown-outer-varint',bytes([8,0,32,1])),
            ('unknown-outer-fixed64',bytes([8,0,33])+bytes(8)),
            ('unknown-outer-bytes',bytes([8,0,34,1,65])),
            ('unknown-outer-fixed32',bytes([8,0,37])+bytes(4)),
            ('unknown-outer-empty-group',bytes([8,0,35,36])),
            ('unknown-inner-empty-group',bytes([8,0,18,2,59,60])),
            ('wrong-version-wire-type',bytes([10,1,0])),
            ('wrong-local-wire-type',bytes([8,0,16,0])),
            ('wrong-lid-wire-type',bytes([8,0,18,2,24,0])),
            ('truncated-inner-length',bytes([8,0,18,3,26,2,65])),
            ('overlong-version-zero',bytes([8])+bytes([128])*9+bytes([0])),
            ('overflow-version-64',bytes([8])+bytes([255])*9+bytes([1])),
            ('overflow-version-above-64',bytes([8])+bytes([255])*9+bytes([2])),
            ('eleven-byte-version',bytes([8])+bytes([128])*10+bytes([0])),
            ('overlong-tag',bytes([136,128,128,128,0,0])),
            ('overlong-tag-six-bytes',bytes([136,128,128,128,128,0,0])),
            ('enum-negative-one',bytes([8,0,18,11,40])+bytes([255])*9+bytes([1])),
            ('enum-zero-presence',bytes([8,0,18,2,40,0])),
            ('enum-one-presence',bytes([8,0,18,2,40,1])),
            ('duplicate-enum',bytes([8,0,18,4,40,0,40,1])),
            ('overlong-zero-tag',bytes([128,0,0])),
            ('overflow-tag-field-id',bytes([136,128,128,128,16,0])),
            ('five-byte-empty-length',bytes([8,0,18])+bytes([128])*4+bytes([0])),
            ('six-byte-empty-length',bytes([8,0,18])+bytes([128])*5+bytes([0])),
            ('length-above-int31',bytes([8,0,18,128,128,128,128,8])),
            ('length-wrap-int32',bytes([8,0,18,128,128,128,128,16])),
            ('unknown-outer-above-64-varint',bytes([8,0,32])+bytes([255])*9+bytes([2])),
            ('wrong-version-fixed32',bytes([13])+bytes(4)),
            ('wrong-version-fixed64',bytes([9])+bytes(8)),
            ('wrong-local-fixed32',bytes([8,0,21])+bytes(4)),
            ('wrong-local-fixed64',bytes([8,0,17])+bytes(8)),
            ('wrong-key-varint',bytes([8,0,18,2,8,0])),
            ('wrong-hash-varint',bytes([8,0,18,2,48,0])),
            ('wrong-enum-bytes',bytes([8,0,18,3,42,1,0])),
            ('wrong-enum-fixed32',bytes([8,0,18,5,45])+bytes(4)),
            ('wrong-enum-fixed64',bytes([8,0,18,9,41])+bytes(8))]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library',type=Path,required=True)
    args = parser.parse_args()
    if not args.library.is_file():
        parser.error('missing reconstructed libwhatsappmerged.so')
    if hashlib.sha256(args.library.read_bytes()).hexdigest()!=LIBRARY_HASH:
        parser.error('requires pinned 37.71 reconstructed libwhatsappmerged.so')
    try:
        records = [dict(case=name,payload_hex=payload.hex(),**NativeParser(args.library).parse(payload)) for name,payload in cases()]
    except ImportError as error:
        parser.exit(1,f'missing research dependency: {error.name}\n')
    except (ValueError,KeyError) as error:
        parser.exit(1,f'native fingerprint unpack check failed: {error}\n')
    print(json.dumps({'apk_build':'2.26.37.71','library_sha256':LIBRARY_HASH,
                      'evidence':'observed/offline native protobuf-C; modeled allocator/memcpy/memset',
                      'cases':records},indent=2))

if __name__=='__main__':main()
