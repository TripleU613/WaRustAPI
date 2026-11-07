"""Pinned params-map follow-up scheduling, not active-table installation."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring = [elf[k] for k in ('b', 'offset', 'cstring')]
strings = {0x1753f: 'PersistParamsMapError',
    0x18b1b: 'FBMobileConfigStorageManager::scheduleWritingLatestParamsMapToDisk',
    0x188a4: 'Failed to persist params map because mcThreadPool_ is null',
    0x1c6ee: 'writeToFile() returned false for: ', 0x1b185: 'params_map_v4_'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
calls = {0x85c04: 0x8ada0, 0x8ade4: 0x8b030, 0x8adfc: 0x8b0b0,
    0x8ae5c: 0x8b130, 0x8ae84: 0x8b130, 0x8af08: 0x87f90,
    0x8b3cc: 0x8b130, 0x8b3fc: 0x8b130, 0x8b06c: 0xc5e0c,
    0x8b0ec: 0xc5d00, 0x8b244: 0xc1f4c, 0x8b284: 0xc1f4c,
    0x8b2d0: 0xc044c, 0x77584: 0x770bc}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
# Resolve imported GLOB_DAT symbols separately from relative-address addends.
shoff = struct.unpack_from('<Q', b, 40)[0]
shsize, shnum = struct.unpack_from('<HH', b, 58)
sections = [struct.unpack_from('<IIQQQQIIQQ', b, shoff+i*shsize) for i in range(shnum)]
imports = {}
for sh in sections:
    if sh[1] != 4:
        continue
    for pos in range(sh[4], sh[4]+sh[5], sh[9]):
        va, info, addend = struct.unpack_from('<QQq', b, pos)
        if va not in (0xd6ba0, 0xd6ba8):
            continue
        symtab = sections[sh[6]]
        strings_section = sections[symtab[6]]
        name_index = struct.unpack_from('<I', b, symtab[4]+(info>>32)*symtab[9])[0]
        start = strings_section[4]+name_index
        imports[hex(va)] = {'type': info & 0xffffffff, 'name': b[start:b.index(0,start)].decode(), 'addend': addend}
expected = {'0xd6ba0': '_ZTVN8facebook6mobile5xplat8executor20InlineSerialExecutorE',
            '0xd6ba8': '_ZTTN8facebook6mobile5xplat8executor20InlineSerialExecutorE'}
if any(imports.get(k) != {'type': 1025, 'name': v, 'addend': 0} for k, v in expected.items()):
    raise ValueError('executor import binding mismatch')
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'selected_calls': {hex(k): hex(v) for k, v in calls.items()},
    'default_factory_imports': imports,
    'inline_mode': 'manager230==1; optional first path when manager212 bit0 clear, then second path',
    'scheduled_mode': 'other manager230 values; invoke 8b384, manager 8b428, captured flags50/51',
    'missing_pool': 'logs PersistParamsMapError; appends byte3 to supplied result vector',
    'authority': 'static imports/calls and analyst-transcribed branches, not task execution or active-state admission',
    'remaining': 'runtime executor override, external dispatch implementation, task completion/copy outcomes and active-table installation'
}, indent=2))
