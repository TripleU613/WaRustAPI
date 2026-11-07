"""Pinned backup preparation, copy limitations and continuation; offline only."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring = [elf[k] for k in ('b', 'offset', 'cstring')]
strings = {0x17fa0: '_backup', 0x1b490: 'DONE', 0x17504: 'CacheBackupError',
    0x1c603: 'Failed to backup cache, result: ', 0x173cd: 'FileCopyError',
    0x19ead: 'copyFile', 0x1ad5d: 'rb', 0x1ad5a: 'wb',
    0x1899d: 'FBMobileConfigStorageManager::persistConfigsInternal', 0x25876: ''}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
calls = {0x86818: 0x8b810, 0x868ac: 0xc28ac, 0x86e3c: 0x8b8bc,
    0x86edc: 0xc1f4c, 0x86f0c: 0xc1f4c, 0x86f3c: 0xc1f4c,
    0x86f80: 0x79dfc, 0x79e20: None, 0xc1f94: 0xcd810,
    0xc1fb8: 0xcd810, 0xc1ff0: 0xcd880, 0xc2004: 0xcd870,
    0xc2010: 0xcd820, 0xc2018: 0xcd820, 0xc1af0: 0xcd870,
    0xc1b04: 0xcd820, 0xc28f8: 0xcd8c0}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    if target is None:
        target = 0xc1a9c
        if word >> 26 != 0b000101:
            raise ValueError('expected tail B')
    elif word >> 26 != 0b100101:
        raise ValueError('expected BL')
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if site + 4 * imm != target:
        raise ValueError('selected branch mismatch')
codes = {0: 'marker helper true', 1: 'selected missing state/path/lookup prerequisites',
    2: 'empty backup base', 3: 'selected cleanup failure', 4: 'directory helper false',
    5: 'first copy false', 6: 'second copy false', 7: 'third copy false',
    8: 'marker helper false'}
print(json.dumps({'member_sha256': elf['digest'],
    'strings': {hex(k): v for k, v in strings.items()},
    'selected_calls': {hex(k): hex(v) if v is not None else 'tail B 0xc1a9c' for k, v in calls.items()},
    'backup_codes': codes,
    'continuation': 'all selected normal backup codes reach 0x8706c; codes >=2 log CacheBackupError first',
    'copy': 'rb source, wb destination; 4096-byte fread/fwrite loop until feof; write/close returns not checked',
    'marker': 'empty-string write helper to DONE path; no DONE body text',
    'authority': 'static calls and analyst-transcribed branch meanings, not executed copy or recovery tests',
    'remaining': 'actual source contents, prerequisite helpers, cleanup implementation, restore consumer, durability and activation'
}, indent=2))
