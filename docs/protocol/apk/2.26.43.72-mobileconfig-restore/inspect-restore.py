"""Pinned copyFromBackup callback and selected restore continuation; offline only."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x1c0b2: '.', 0x1c055: '..', 0x1b490: 'DONE',
    0x17f94: 'restore_from_backup', 0x17f74: 'instacrash_recovery_from_backup',
    0x16bd4: 'recover_status',
    0x22b89: 'ZN12mobileconfig28FBMobileConfigStorageManager14copyFromBackupEvE3$_0'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('string mismatch')
if relocs[0xd5598] != 0x834c8 or relocs[0xd5ac0] != 0x8d550:
    raise ValueError('selected vtable target mismatch')
calls = {0x836f8: 0x8cee8, 0x8371c: 0x8b810, 0x83754: 0xc2148,
    0x83778: 0xc28ac, 0x837b8: 0xc2530, 0x8d57c: 0x70a28,
    0x8d590: 0x70a28, 0x8d5a4: 0x70a28, 0x8d5e0: 0xc1f4c,
    0x8cfe0: 0xc2420, 0xc257c: 0xcd8d0, 0xc25c4: 0xcd8e0,
    0xc25c8: 0xcd8f0, 0xc2630: 0xcd900, 0x838e8: 0xc28ac}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def project(entries):
    # Supplied iteration order and helper outcomes, not a native filesystem test.
    ok = True
    attempted = []
    for name, outcome in entries:
        if name in ('.', '..', 'DONE'):
            continue
        if ok:
            attempted.append(name)
            ok = outcome
    return {'attempted': attempted, 'selected_recover_code': 0 if ok else 4}
print(json.dumps({'member_sha256': elf['digest'],
    'strings': {hex(k): v for k, v in strings.items()},
    'storage_method_slot': {'slot': '0xd5598', 'offset': '0x18', 'target': '0x834c8'},
    'callback_slot': {'address_point': '0xd5a90', 'slot': '0xd5ac0', 'target': '0x8d550'},
    'calls': {hex(k): hex(v) for k, v in calls.items()},
    'local_recover_codes': {'0': 'copy latch remains true', '1': 'empty backup base', '2': 'DONE status false', '3': 'destination directory helper false', '4': 'copy latch false', '5': 'pre-restore cleanup helper false'},
    'projection_authority': 'analyst-derived callback projection with supplied order/outcomes, not native restore execution',
    'synthetic_cases': {'marker_only': project([('DONE', True)]),
        'failure_skips_later_copy': project([('first', False), ('DONE', True), ('second', True)]),
        'success': project([('first', True), ('second', True)])},
    'remaining': 'startup caller binding, flag producers, pre-cleanup side effects, iterator/imported I/O exceptions, verified restored bytes and activation'
}, indent=2))
