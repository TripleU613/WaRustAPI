#!/usr/bin/env python3
"""Execute pinned native V4 comparison branches on synthetic modeled parameters."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct

from native_fingerprint_check import LIBRARY_HASH, RETURN
from native_schema_check import LIBRARY_HASH as STRING_LIBRARY_HASH

BIONIC_HASH = "4e6d025f3356a3353418981e475561cfa133a0e439a20c9d9bede715ae167697"


class NativeAbort(ValueError):
    """The native routine reached its explicit abort branch."""


def execute(library, known, scanned, version, remap=False, string_libraries=None,
            scheme=4, options=3):
    """Actual 37.71 core/helper/remap instructions; modeled object primitives.

    The internal scheme=3 mode executes V3 core/optional-value/orientation
    helpers/remapper, modeling generic MCFEqual and getter primitives.
    No protobuf parsing, orientation wrapper, JNI, account or human action
    executes. For V4, host ASCII lowercasing models string comparison. With
    pinned string_libraries, actual merged MCFStringCompare and public Bionic
    strncasecmp execute over synthetic MCFString objects. Only built-in cases
    invoke this; the public libc fixture is not an APK-bundled Android runtime.
    """
    if scheme not in (3, 4) or (scheme == 3 and string_libraries):
        raise ValueError('unsupported synthetic comparison scheme/string mode')
    from elftools.elf.elffile import ELFFile
    from unicorn import Uc, UcError, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
    from unicorn.arm64_const import (
        UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2,
        UC_ARM64_REG_X3, UC_ARM64_REG_X4, UC_ARM64_REG_LR, UC_ARM64_REG_PC, UC_ARM64_REG_SP,
        UC_ARM64_REG_TPIDR_EL0,
    )
    guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
    # Core, PN/username helpers and remapper share 0x715000; include actual
    # status-table page. Each modeled target must be mapped for Unicorn fetch.
    pages = (0x715000, 0x302000, 0x714000, 0x702000, 0x6E1000,
             0xCE8000, 0xCE6000, 0xCE9000)
    if scheme == 3:
        pages += (0x6C1000,)  # PN getter target must be mapped before hook fetch.
    with library.open('rb') as stream:
        elf = ELFFile(stream)
        if elf['e_machine'] != 'EM_AARCH64':
            raise ValueError('requires ARM64 ELF')
        for page in pages:
            guest.mem_map(page, 4096)
            for segment in elf.iter_segments():
                if (segment['p_type'] == 'PT_LOAD'
                        and segment['p_vaddr'] <= page
                        and page + 4096 <= segment['p_vaddr'] + segment['p_filesz']):
                    offset = page - segment['p_vaddr']
                    guest.mem_write(page, segment.data()[offset:offset + 4096])
                    break
            else:
                raise ValueError('required comparison page is not file-backed')
    if string_libraries:
        for path, string_pages in ((string_libraries[0], (0x538000, 0x759000, 0x75A000)),
                                   (string_libraries[1], (0x64000, 0xAB000))):
            with path.open('rb') as stream:
                elf = ELFFile(stream)
                if elf['e_machine'] != 'EM_AARCH64':
                    raise ValueError('string fixture requires ARM64 ELF')
                for page in string_pages:
                    guest.mem_map(page, 4096)
                    for segment in elf.iter_segments():
                        if (segment['p_type'] == 'PT_LOAD'
                                and segment['p_vaddr'] <= page
                                and page + 4096 <= segment['p_vaddr'] + segment['p_filesz']):
                            offset = page - segment['p_vaddr']
                            guest.mem_write(page, segment.data()[offset:offset + 4096])
                            break
                    else:
                        raise ValueError('required string page is not file-backed')
        # The native ASCII flag-1 branch resolves GOT strncasecmp here. The
        # strncmp slot remains null: an unexpected flag-0 branch must refuse.
        guest.mem_write(0x759D90, struct.pack('<Q', 0x64E88))
        guest.mem_write(0x75A1A8, struct.pack('<Q', 0))
        guest.mem_map(0x20000000, 0x10000)
    guest.mem_map(0x10000000, 0x10000)
    guest.reg_write(UC_ARM64_REG_SP, 0x1000F000)
    guest.reg_write(UC_ARM64_REG_TPIDR_EL0, 0x1000E000)
    context = 0x10000100
    params = {0x10000200: known[0], 0x10000300: known[1],
              0x10000400: scanned[0], 0x10000500: scanned[1]}
    # Actual core loads the known local/remote holders at context +20/+28.
    guest.mem_write(context + 0x20, struct.pack('<QQ', 0x10000200, 0x10000300))
    strings = {}
    instructions = 0
    comparisons = 0
    object_comparisons = 0

    def pointer(value):
        if value is None:
            return 0
        token = 0x20000000 + len(strings) * (1024 if string_libraries else 16)
        if string_libraries:
            if len(value) > 256 or not all(1 <= byte <= 127 for byte in value):
                raise ValueError('synthetic string outside native ASCII fixture bounds')
            # Synthetic MCFString objects: pointer storage (+19 = 0), ASCII
            # representation (+1a = 1), count +24 and character pointer +28.
            # Actual native constructor is not executed by this runner.
            guest.mem_write(token + 0x19, bytes((0, 1)))
            guest.mem_write(token + 0x24, struct.pack('<IQ', len(value), token + 64))
            if value:
                guest.mem_write(token + 64, value)
        strings[token] = value
        return token

    def hook(cpu, address, size, user_data):
        nonlocal instructions, comparisons, object_comparisons
        instructions += 1
        # Execute the entire core plus PN/username helper instructions.
        if scheme == 3 and (0x71401C <= address < 0x714358
                            or 0x714408 <= address < 0x7144A0):
            return
        if scheme == 4 and 0x7152FC <= address < 0x715688:
            return
        # Executing this separately establishes its exact table lookup.
        if scheme == 4 and remap and 0x7152B8 <= address < 0x7152FC:
            return
        if string_libraries and (0x538A50 <= address < 0x538C34
                                 or 0x64E88 <= address < 0x64EC8):
            return
        first = cpu.reg_read(UC_ARM64_REG_X0)
        second = cpu.reg_read(UC_ARM64_REG_X1)
        if scheme == 4 and address == 0x715264:  # modeled LID equal getter helper
            result = int(params[first]['lid'] == params[second]['lid'])
        elif address in (0x714358, 0x7143B0):  # modeled data getters/equality
            field = 'public' if address == 0x714358 else 'hash'
            result = int(params[first][field] == params[second][field])
            if scheme == 3 and (params[first][field] is None or params[second][field] is None):
                result = 0
        elif address in (0x702A74, 0x6E1814):
            field = 'username' if address == 0x702A74 else ('lid' if scheme == 3 else 'pn')
            result = pointer(params[first][field])
        elif scheme == 3 and address == 0x6C1E4C:
            result = pointer(params[first]['pn'])
        elif scheme == 3 and address == 0xCE9150:
            # V3 invokes generic MCFEqual for PN, retained LID and username.
            # Equality of synthetic byte tokens is modeled, not native JID semantics.
            object_comparisons += 1
            result = int(first != 0 and second != 0 and strings[first] == strings[second])
        elif address == 0x702AB4:
            result = params[first]['hosted'] & 0xFFFFFFFF
        elif scheme == 4 and address == 0xCE8E90:  # MCFASCIIStringEqual, PN exact bytes
            result = int(strings[first] == strings[second])
        elif scheme == 4 and address == 0xCE8970:  # synthetic ASCIIString->MCFString
            result = first
        elif scheme == 4 and address == 0xCE9990:  # MCFStringCompare flag 1
            if cpu.reg_read(UC_ARM64_REG_X2) != 1:
                raise ValueError('unexpected string comparison flags')
            comparisons += 1
            if string_libraries:
                cpu.reg_write(UC_ARM64_REG_PC, 0x538A50)
                return
            result = int(strings[first].lower() != strings[second].lower())
        elif address in (0xCE64E0, 0xCE85B0):  # release / disable logging
            result = 0
        elif address == 0xCE64F0:
            raise NativeAbort('native hosted-state/remapper invariant abort')
        else:
            raise ValueError(f'unmodeled comparison branch: {address:#x}')
        cpu.reg_write(UC_ARM64_REG_X0, result)
        cpu.reg_write(UC_ARM64_REG_PC, cpu.reg_read(UC_ARM64_REG_LR))

    guest.hook_add(UC_HOOK_CODE, hook)
    guest.reg_write(UC_ARM64_REG_X0, context)
    guest.reg_write(UC_ARM64_REG_X1, 0x10000400)
    guest.reg_write(UC_ARM64_REG_X2, 0x10000500)
    guest.reg_write(UC_ARM64_REG_X3, version)
    if scheme == 3:
        guest.reg_write(UC_ARM64_REG_X4, options)
    guest.reg_write(UC_ARM64_REG_LR, RETURN)

    def run(entry):
        try:
            guest.emu_start(entry, RETURN, timeout=10_000_000, count=1_000_000)
        except UcError as error:
            raise ValueError(f'ARM64 comparison execution failed: {error}') from error
        if guest.reg_read(UC_ARM64_REG_PC) != RETURN:
            raise ValueError('comparison budget exhausted before return')
        return struct.unpack('<i', struct.pack('<I', guest.reg_read(UC_ARM64_REG_X0) & 0xFFFFFFFF))[0]

    code = run(0x7140FC if scheme == 3 else 0x7152FC)
    mapped = None
    if remap:
        guest.reg_write(UC_ARM64_REG_LR, RETURN)
        mapped = run(0x7140B8 if scheme == 3 else 0x7152B8)
    if scheme == 3:
        return {'code': code, 'remapped_code': mapped, 'instructions': instructions,
                'modeled_object_comparisons': object_comparisons}
    return {'code':code, 'remapped_code':mapped, 'instructions':instructions,
            ('native_string_comparisons' if string_libraries else 'modeled_string_comparisons'):comparisons}


def run_cases(library, string_libraries=None):
    # Synthetic fields deliberately avoid account/contact inputs. Expected
    # codes come from literal return branches 0x71539c..0x7155cc; hosted
    # expectations from 0x7154f0..0x715584; table from 0x302404.
    base = [dict(lid=b'synthetic-local', public=b'local-key', hash=b'local-hash',
                 pn=b'local-pn', username=b'Local', hosted=0),
            dict(lid=b'synthetic-remote', public=b'remote-key', hash=b'remote-hash',
                 pn=b'remote-pn', username=b'Remote', hosted=1)]
    mapping = {-41:-42,-39:-40,-36:-37,-34:-35,-33:-33,-31:-32,-29:-30,
               -28:-28,-26:-27,-24:-25,-19:-18,-17:-16,-15:-15,-14:-14,-12:-11,0:0}
    records=[]

    def check(name, known, scanned, expected, version=0):
        result = execute(library, known, scanned, version, remap=True, string_libraries=string_libraries)
        if result['code'] != expected or result['remapped_code'] != mapping[expected]:
            raise ValueError(f'{name}: native comparison status/table differed')
        records.append(dict(case=name, version=version, **result))

    for version in (0,1):
        check(f'baseline-v{version}',base,base,0,version)
        for side in (0,1):
            scanned=copy.deepcopy(base); scanned[side]['public' if version==0 else 'hash']=b'different'
            check(f'key-side{side}-v{version}',base,scanned,-14 if side==0 else -15,version)
            scanned=copy.deepcopy(base); scanned[side]['hash' if version==0 else 'public']=b'ignored-other-version'
            check(f'unselected-key-side{side}-v{version}',base,scanned,0,version)
    scanned=copy.deepcopy(base);scanned[1]['lid']=b'different'
    check('remote-lid',base,scanned,-12)
    # Known-local LID was already selected by the separate orientation wrapper.
    scanned=copy.deepcopy(base);scanned[0]['lid']=b'different'
    check('core-does-not-repeat-local-orientation-check',base,scanned,0)
    for side in (0,1):
        for target in ('known','scanned'):
            known=copy.deepcopy(base);scanned=copy.deepcopy(base)
            (known if target=='known' else scanned)[side].update(pn=None,username=None)
            check(f'no-identifiers-{target}-side{side}',known,scanned,-39 if side==0 else -41)
        scanned=copy.deepcopy(base);scanned[side]['pn']=b'different'
        check(f'pn-mismatch-side{side}',base,scanned,-17 if side==0 else -19)
        for missing in ('known','scanned'):
            known=copy.deepcopy(base);scanned=copy.deepcopy(base)
            (known if missing=='known' else scanned)[side]['pn']=None
            check(f'pn-omission-{missing}-side{side}',known,scanned,0)
        for mode,code in [('different',-24 if side==0 else -29),
                          ('known-absent',-28 if side==0 else -33),
                          ('scanned-absent',-26 if side==0 else -31)]:
            known=copy.deepcopy(base);scanned=copy.deepcopy(base)
            if mode=='different':scanned[side]['username']=b'different'
            elif mode=='known-absent':known[side]['username']=None
            else:scanned[side]['username']=None
            check(f'username-{mode}-side{side}',known,scanned,code)
        known=copy.deepcopy(base);scanned=copy.deepcopy(base)
        known[side]['username']=scanned[side]['username']=None
        check(f'both-usernames-absent-side{side}',known,scanned,0)
        scanned=copy.deepcopy(base);scanned[side]['username']=base[side]['username'].swapcase()
        check(f'modeled-case-insensitive-side{side}',base,scanned,0)
    for local in (-1,0,1):
        for remote in (-1,0,1):
            for first in (-1,0,1):
                for second in (-1,0,1):
                    known=copy.deepcopy(base);scanned=copy.deepcopy(base)
                    known[0]['hosted']=local;known[1]['hosted']=remote
                    scanned[0]['hosted']=first;scanned[1]['hosted']=second
                    # Both absent takes the legacy branch and skips known states.
                    expected=0 if first==second==-1 else -34 if local!=first else -36 if remote!=second else 0
                    check(f'hosted-{local}-{remote}-{first}-{second}',known,scanned,expected)
    for side in (0,1):
        scanned=copy.deepcopy(base);scanned[side]['hosted']=2
        if side==1:scanned[0]['hosted']=-1
        try:
            execute(library,base,scanned,0,string_libraries=string_libraries)
        except NativeAbort:
            records.append({'case':f'unknown-scanned-hosted-side{side}','native_abort':True})
        else:
            raise ValueError('unknown scanned hosted state did not abort')
    result = {'apk_build':'2.26.37.71','library_sha256':LIBRARY_HASH,
            'evidence':'observed/offline with modeled dependencies',
            'core_address':'0x7152fc', 'helpers_range':'0x7155d8..0x715688',
            'remapper_address':'0x7152b8', 'status_table_address':'0x302404',
            'scope':'native core, PN/username helpers and status remapper; modeled dependencies; not whole QR comparison',
            'cases':records}
    if string_libraries:
        result['string_library_sha256'] = STRING_LIBRARY_HASH
        result['bionic_fixture_sha256'] = BIONIC_HASH
        result['scope'] += '; actual merged MCFStringCompare and public Bionic strncasecmp; synthetic object construction'
        result['native_ascii_checks'] = check_native_ascii(library, string_libraries)
    return result


def check_native_ascii(library, string_libraries):
    """Full native ASCII-byte/prefix checks without accepting caller strings."""
    base = [dict(lid=b'synthetic-local', public=b'local-key', hash=b'local-hash',
                 pn=b'local-pn', username=b'Local', hosted=0),
            dict(lid=b'synthetic-remote', public=b'remote-key', hash=b'remote-hash',
                 pn=b'remote-pn', username=b'Remote', hosted=1)]
    ascii_bytes = bytes(range(1, 128))
    cases = 0
    max_instructions = 0
    for side in (0, 1):
        variants = [(ascii_bytes, ascii_bytes.swapcase(), 0)]
        # Change each accepted ASCII position to a different case-folded byte.
        # Expected mismatch is independent of the native table or host lower().
        for position, byte in enumerate(ascii_bytes):
            changed = bytearray(ascii_bytes)
            changed[position] = 1 if byte != 1 else 2
            variants.append((ascii_bytes, bytes(changed), -24 if side == 0 else -29))
        mismatch = -24 if side == 0 else -29
        variants.extend([(b'prefix', b'prefixx', mismatch),
                         (b'prefixx', b'prefix', mismatch),
                         (b'', b'', 0), (b'', b'x', mismatch), (b'x', b'', mismatch),
                         (b'A' * 256, b'a' * 256, 0)])
        for known_string, scanned_string, expected in variants:
            known = copy.deepcopy(base)
            scanned = copy.deepcopy(base)
            known[side]['username'] = known_string
            scanned[side]['username'] = scanned_string
            result = execute(library, known, scanned, 0, string_libraries=string_libraries)
            if result['code'] != expected:
                raise ValueError('native ASCII-byte/prefix result differed')
            max_instructions = max(max_instructions, result['instructions'])
            cases += 1
    return {'cases': cases, 'accepted_ascii_byte_values': 127,
            'mismatch_positions_per_side': 127, 'empty_prefix_and_256_byte_cases_per_side': 6,
            'max_instructions': max_instructions,
            'string_object_construction': 'synthetic',
            'string_comparison': 'actual APK merged MCFStringCompare and public Bionic strncasecmp'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library',type=Path,required=True)
    parser.add_argument('--string-library', type=Path)
    parser.add_argument('--libc', type=Path)
    args=parser.parse_args()
    if bool(args.string_library) != bool(args.libc):
        parser.error('--string-library and --libc must be supplied together')
    for path, expected, label in ((args.string_library, STRING_LIBRARY_HASH, '37.71 merged string library'),
                                  (args.libc, BIONIC_HASH, 'public Bionic fixture')):
        if path is not None:
            if not path.is_file():
                parser.error(f'missing {label}')
            if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
                parser.error(f'requires pinned {label}')
    if not args.library.is_file():parser.error('missing reconstructed native library')
    if hashlib.sha256(args.library.read_bytes()).hexdigest()!=LIBRARY_HASH:
        parser.error('requires the pinned 37.71 reconstructed libwhatsapp.so')
    try:
        result=run_cases(args.library, (args.string_library, args.libc) if args.libc else None)
    except ImportError as error:
        parser.exit(1,f'missing research dependency: {error.name}\n')
    except (ValueError,KeyError) as error:
        parser.exit(1,f'native V4 check failed: {error}\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
