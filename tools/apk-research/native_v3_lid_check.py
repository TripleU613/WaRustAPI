#!/usr/bin/env python3
"""Execute pinned V3 primary-LID parsing with bounded modeled MCF primitives."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from native_fingerprint_check import LIBRARY_HASH, RETURN


class NativeLid:
    def __init__(self, library):
        from elftools.elf.elffile import ELFFile
        from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
        from unicorn import arm64_const as reg
        self.reg = reg
        self.guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
        with library.open('rb') as stream:
            elf = ELFFile(stream)
            if elf['e_machine'] != 'EM_AARCH64':
                raise ValueError('requires ARM64 ELF')
            for segment in elf.iter_segments():
                if segment['p_type'] == 'PT_LOAD':
                    start = segment['p_vaddr'] & ~4095
                    end = (segment['p_vaddr'] + segment['p_memsz'] + 4095) & ~4095
                    self.guest.mem_map(start, end-start)
                    self.guest.mem_write(segment['p_vaddr'], segment.data())
            for relocation in elf.get_section_by_name('.rela.dyn').iter_relocations():
                if relocation['r_info_type'] == 1027:
                    self.guest.mem_write(relocation['r_offset'], struct.pack('<Q', relocation['r_addend']))
        self.guest.mem_map(0x10000000, 0x10000)
        self.guest.mem_map(0x30000000, 0x100000)
        self.heap = 0x30000000
        self.strings = {}
        self.arrays = {}
        self.instructions = 0
        self.primitives = {}
        # APK 37.71 exact ranges: fingerprint-v3-equality-lid-evidence.txt.
        # Only traced routing/validators/constructors execute; mapped bytes
        # elsewhere are storage, not permission to run another native path.
        self.allowed = [(0x71DA6C, 0x71DADC), (0x71D944, 0x71D9F8),
                        (0x71DDB4, 0x71DED8), (0x71DFA4, 0x71E060),
                        (0x71EBD8, 0x71ECC0), (0x71EB38, 0x71EBD8),
                        (0x71EFE4, 0x71F01C), (0x71F2A4, 0x71F2F8),
                        (0x71F30C, 0x71F4E0), (0x71F4E0, 0x71FA08),
                        (0x71FA08, 0x71FC54), (0x71FD24, 0x71FE94),
                        (0x7204BC, 0x720564), (0x720BAC, 0x720D50)]
        # APK cached constants: producer functions 71e198 + 70*i load
        # relocated (pointer,length) records d0ae98 + 10*i.
        self.constants = {}
        for index in range(13):
            record = 0xD0AE98 + 16*index
            pointer, length = struct.unpack('<QI', self.guest.mem_read(record, 12))
            self.constants[0x71E198 + 0x70*index] = bytes(self.guest.mem_read(pointer, length))
        self.constants.update({0x71E898:b'@', 0x71E908:b'.', 0x71E978:b':', 0x71EAC8:b'0'})

        def hook(cpu, address, size, data):
            self.instructions += 1
            if any(start <= address < end for start, end in self.allowed):
                return
            values = [cpu.reg_read(getattr(reg, f'UC_ARM64_REG_X{i}')) for i in range(4)]
            first, second, third, fourth = values
            self.primitives[hex(address)] = self.primitives.get(hex(address), 0) + 1
            if address in self.constants:
                result = self.string(self.constants[address])
            elif address == 0x71ECC0:  # modeled JID runtime type registration
                result = 1
            elif address == 0xCE84F0:  # modeled instance allocation; actual constructors execute
                if second > 4096:
                    raise ValueError('modeled instance exceeds bound')
                result = self.allocate(second + 0x18)
            elif address == 0xCE8BC0:
                result = len(self.strings[first])
            elif address == 0xCE9A50:
                text = self.strings[first]
                if second >= len(text):
                    raise ValueError('native character index outside synthetic string')
                result = text[second]
            elif address == 0xCE9200:  # modeled split, retaining empty pieces
                result = self.allocate(16)
                self.arrays[result] = [self.string(part) for part in self.strings[first].split(self.strings[second])]
            elif address == 0xCE8800:
                result = len(self.arrays[first])
            elif address == 0xCE8810:
                if second >= len(self.arrays[first]):
                    raise ValueError('native array index outside modeled split')
                result = self.arrays[first][second]
            elif address == 0xCE9150:  # ASCII byte equality, class callback traced separately
                result = int(self.strings[first] == self.strings[second])
            elif address == 0xCE8360:
                result = struct.unpack('<Q', cpu.mem_read(first+0x20, 8))[0]
            elif address == 0xCE9000:
                result = self.string(self.strings[first])
            elif address == 0xCE8940:
                fmt = self.cstring(first)
                if fmt == b'%s@%s':
                    text = self.cstring(second) + b'@' + self.cstring(third)
                elif fmt == b'%s:%d':
                    number = struct.unpack('<i', struct.pack('<I', third & 0xffffffff))[0]
                    text = self.cstring(second) + b':' + str(number).encode()
                else:
                    raise ValueError('unmodeled native format string')
                result = self.string(text)
            elif address == 0xCE80B0:
                # The executed LID validator restricts suffix to 1/2 decimal
                # digits. Default is literal 0. Do not emulate arbitrary strtol.
                text = self.cstring(first)
                if third != 10 or not 1 <= len(text) <= 2 or not text.isdigit():
                    raise ValueError('strtol outside validated synthetic suffix')
                result = int(text)
            elif address == 0xCE8900:
                result = self.allocate(16)
                code = struct.unpack('<i', struct.pack('<I', second & 0xffffffff))[0]
                cpu.mem_write(result, struct.pack('<i', code))
            elif address == 0xCE8390:
                result = first  # modeled retain
            elif address == 0xCE64E0:
                result = 0  # ownership/deallocation not executed
            elif address == 0xCE64F0:
                raise ValueError('native LID invariant abort')
            else:
                raise ValueError(f'unmodeled native LID branch: {address:#x}')
            cpu.reg_write(reg.UC_ARM64_REG_X0, result)
            cpu.reg_write(reg.UC_ARM64_REG_PC, cpu.reg_read(reg.UC_ARM64_REG_LR))
        self.guest.hook_add(UC_HOOK_CODE, hook)

    def allocate(self, size):
        size = max(16, (size+15) & ~15)
        if self.heap + size > 0x30100000:
            raise ValueError('modeled LID heap budget exhausted')
        result = self.heap
        self.heap += size
        return result

    def string(self, value):
        if len(value) > 256 or not all(1 <= byte <= 127 for byte in value):
            raise ValueError('synthetic ASCII input outside fixture bounds')
        pointer = self.allocate(48)
        data = self.allocate(len(value)+1)
        self.guest.mem_write(data, value+b'\0')
        # Merged MCFASCIIStringEqual 0x5316ac reads length +18/data +20.
        # This models an ASCII object; its native constructor does not run.
        self.guest.mem_write(pointer+0x18, struct.pack('<I', len(value)))
        self.guest.mem_write(pointer+0x20, struct.pack('<Q', data))
        self.strings[pointer] = value
        return pointer

    def cstring(self, pointer):
        for count in range(257):
            if self.guest.mem_read(pointer+count, 1) == b'\0':
                return bytes(self.guest.mem_read(pointer, count))
        raise ValueError('synthetic C string exceeded bound')

    def run(self, entry, args):
        from unicorn import UcError
        cpu, reg = self.guest, self.reg
        cpu.reg_write(reg.UC_ARM64_REG_SP, 0x1000F000)
        cpu.reg_write(reg.UC_ARM64_REG_TPIDR_EL0, 0x1000E000)
        cpu.reg_write(reg.UC_ARM64_REG_LR, RETURN)
        for index, value in enumerate(args):
            cpu.reg_write(getattr(reg, f'UC_ARM64_REG_X{index}'), value)
        try:
            cpu.emu_start(entry, RETURN, timeout=10_000_000, count=1_000_000)
        except UcError as error:
            raise ValueError(f'ARM64 LID execution failed: {error}') from error
        if cpu.reg_read(reg.UC_ARM64_REG_PC) != RETURN:
            raise ValueError('LID execution budget exhausted before return')
        return cpu.reg_read(reg.UC_ARM64_REG_X0)

    def parse(self, value):
        error_slot = 0x10000100
        self.guest.mem_write(error_slot, bytes(8))
        # Primary parser 71dfa4 checks category after 71ebd8 flattening;
        # 71f2b8 separately requires server kind 5, returning the same object.
        pointer = self.run(0x71DFA4, [self.string(value), error_slot])
        error = struct.unpack('<Q', self.guest.mem_read(error_slot, 8))[0]
        primary = bool(pointer)
        stable = bool(self.run(0x71F2B8, [pointer])) if primary else False
        result = {'primary_parse_succeeded':primary, 'stable_lid_succeeded':stable,
                  'parse_error': struct.unpack('<i', self.guest.mem_read(error,4))[0] if error else None,
                  'normalized_jid':None, 'kind':None, 'category':None}
        if pointer:
            string = struct.unpack('<Q', self.guest.mem_read(pointer+0x20,8))[0]
            result.update(normalized_jid=self.strings[string].decode(),
                          kind=struct.unpack('<I',self.guest.mem_read(pointer+0x30,4))[0],
                          category=struct.unpack('<I',self.guest.mem_read(pointer+0x18,4))[0])
        return dict(result, instructions=self.instructions, modeled_primitives=self.primitives)


def run_cases(library):
    # Expectations from native primary/category/kind guards and literal
    # validators. String split/equality/format primitives remain modeled.
    records=[]
    cases=[]
    for length in range(1, 22):
        value=b'1'*length+b'@lid'
        cases.append((f'base-length-{length}',value,'1'*length+'@lid' if length<=15 or length==20 else None,-1))
    for suffix in (b'0',b'00',b'1',b'01',b'9',b'99'):
        cases.append(('suffix-'+suffix.decode(),b'123:'+suffix+b'@lid','123@lid' if int(suffix)==0 else None,-2))
    for value in (b'0@lid',b'0123@lid',b'abc@lid',b'-1@lid',b'123:100@lid',b'123:-1@lid',
                  b'123:000@lid',b'123.0@lid',b'123@LID',b'123@unknown',b'synthetic-no-at'):
        cases.append(('invalid-'+value.decode(),value,None,-1))
    for name,value,expected,error in cases:
        result=NativeLid(library).parse(value)
        if expected is not None:
            if not result['stable_lid_succeeded'] or result['normalized_jid']!=expected or result['parse_error'] is not None:
                raise ValueError(f'{name}: native accepted result differed')
        elif result['primary_parse_succeeded'] or result['parse_error']!=error:
            raise ValueError(f'{name}: native rejection differed: {result}')
        records.append(dict(case=name, synthetic_input=value.decode(), **result))
    return {'apk_build':'2.26.37.71','library_sha256':LIBRARY_HASH,
            'scope':'actual primary-JID parser routing, LID validators, user/device decomposition, JID constructors, device-zero flattening and stable-kind gate; modeled MCF split (retains empties), ASCII getters/copies/equality/formatting, cached constants, allocation/retain/release/errors and validated decimal strtol; no native class registry, protobuf converter, JNI, ownership validation or account',
            'cases':records}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library',type=Path,required=True)
    args=parser.parse_args()
    if not args.library.is_file(): parser.error('missing reconstructed native library')
    if hashlib.sha256(args.library.read_bytes()).hexdigest()!=LIBRARY_HASH:
        parser.error('requires the pinned 37.71 reconstructed libwhatsapp.so')
    try:
        result=run_cases(args.library)
    except ImportError as error:
        parser.exit(1,f'missing research dependency: {error.name}\n')
    except (ValueError,KeyError) as error:
        parser.exit(1,f'native V3 LID check failed: {error}\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
