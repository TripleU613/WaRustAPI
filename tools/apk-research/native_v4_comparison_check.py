#!/usr/bin/env python3
"""Execute pinned native V4 comparison branches on synthetic modeled parameters."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct

from native_fingerprint_check import LIBRARY_HASH, RETURN


class NativeAbort(ValueError):
    """The native routine reached its explicit abort branch."""


def execute(library, known, scanned, version, remap=False):
    """Actual 37.71 core/helper/remap instructions; modeled object primitives.

    No protobuf parsing, orientation wrapper, JNI, libc, account or human action
    executes. Host ASCII lowercasing is a synthetic comparison model, not proof
    of Android strncasecmp/locale behavior. Only built-in cases invoke this.
    """
    from elftools.elf.elffile import ELFFile
    from unicorn import Uc, UcError, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
    from unicorn.arm64_const import (
        UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2,
        UC_ARM64_REG_X3, UC_ARM64_REG_LR, UC_ARM64_REG_PC, UC_ARM64_REG_SP,
    )
    guest = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
    # Core, PN/username helpers and remapper share 0x715000; include actual
    # status-table page. Each modeled target must be mapped for Unicorn fetch.
    pages = (0x715000, 0x302000, 0x714000, 0x702000, 0x6E1000,
             0xCE8000, 0xCE6000, 0xCE9000)
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
    guest.mem_map(0x10000000, 0x10000)
    guest.reg_write(UC_ARM64_REG_SP, 0x1000F000)
    context = 0x10000100
    params = {0x10000200: known[0], 0x10000300: known[1],
              0x10000400: scanned[0], 0x10000500: scanned[1]}
    # Actual core loads the known local/remote holders at context +20/+28.
    guest.mem_write(context + 0x20, struct.pack('<QQ', 0x10000200, 0x10000300))
    strings = {}
    instructions = 0
    comparisons = 0

    def pointer(value):
        if value is None:
            return 0
        token = 0x20000000 + len(strings) * 16
        strings[token] = value
        return token

    def hook(cpu, address, size, user_data):
        nonlocal instructions, comparisons
        instructions += 1
        # Execute the entire core plus PN/username helper instructions.
        if 0x7152FC <= address < 0x715688:
            return
        # Executing this separately establishes its exact table lookup.
        if remap and 0x7152B8 <= address < 0x7152FC:
            return
        first = cpu.reg_read(UC_ARM64_REG_X0)
        second = cpu.reg_read(UC_ARM64_REG_X1)
        if address == 0x715264:  # modeled LID equal getter helper
            result = int(params[first]['lid'] == params[second]['lid'])
        elif address in (0x714358, 0x7143B0):  # modeled data getters/equality
            field = 'public' if address == 0x714358 else 'hash'
            result = int(params[first][field] == params[second][field])
        elif address in (0x702A74, 0x6E1814):
            field = 'username' if address == 0x702A74 else 'pn'
            result = pointer(params[first][field])
        elif address == 0x702AB4:
            result = params[first]['hosted'] & 0xFFFFFFFF
        elif address == 0xCE8E90:  # MCFASCIIStringEqual, PN exact bytes
            result = int(strings[first] == strings[second])
        elif address == 0xCE8970:  # synthetic ASCIIString->MCFString
            result = first
        elif address == 0xCE9990:  # MCFStringCompare flag 1
            if cpu.reg_read(UC_ARM64_REG_X2) != 1:
                raise ValueError('unexpected string comparison flags')
            comparisons += 1
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
    guest.reg_write(UC_ARM64_REG_LR, RETURN)

    def run(entry):
        try:
            guest.emu_start(entry, RETURN, timeout=10_000_000, count=1_000_000)
        except UcError as error:
            raise ValueError(f'ARM64 comparison execution failed: {error}') from error
        if guest.reg_read(UC_ARM64_REG_PC) != RETURN:
            raise ValueError('comparison budget exhausted before return')
        return struct.unpack('<i', struct.pack('<I', guest.reg_read(UC_ARM64_REG_X0) & 0xFFFFFFFF))[0]

    code = run(0x7152FC)
    mapped = None
    if remap:
        guest.reg_write(UC_ARM64_REG_LR, RETURN)
        mapped = run(0x7152B8)
    return {'code':code, 'remapped_code':mapped, 'instructions':instructions,
            'modeled_string_comparisons':comparisons}


def run_cases(library):
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
        result = execute(library, known, scanned, version, remap=True)
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
            execute(library,base,scanned,0)
        except NativeAbort:
            records.append({'case':f'unknown-scanned-hosted-side{side}','native_abort':True})
        else:
            raise ValueError('unknown scanned hosted state did not abort')
    return {'apk_build':'2.26.37.71','library_sha256':LIBRARY_HASH,
            'evidence':'observed/offline with modeled dependencies',
            'core_address':'0x7152fc', 'helpers_range':'0x7155d8..0x715688',
            'remapper_address':'0x7152b8', 'status_table_address':'0x302404',
            'scope':'native core, PN/username helpers and status remapper; modeled dependencies; not whole QR comparison',
            'cases':records}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library',type=Path,required=True)
    args=parser.parse_args()
    if not args.library.is_file():parser.error('missing reconstructed native library')
    if hashlib.sha256(args.library.read_bytes()).hexdigest()!=LIBRARY_HASH:
        parser.error('requires the pinned 37.71 reconstructed libwhatsapp.so')
    try:
        result=run_cases(args.library)
    except ImportError as error:
        parser.exit(1,f'missing research dependency: {error.name}\n')
    except (ValueError,KeyError) as error:
        parser.exit(1,f'native V4 check failed: {error}\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
