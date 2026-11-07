"""Pinned local file replacement ordering/retries; no native or network I/O."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring = [elf[k] for k in ('b', 'offset', 'cstring')]
strings = {0x1811a: '.tmp', 0x19ed0: 'SchemaUpgradeFailedToRenameFile',
    0x17d09: 'Unable to rename flatbuffer', 0x1a39b: 'fail_to_rename_table_in_upgrade',
    0x1a3d6: 'fail_to_rename_meta_in_upgrade', 0x1a891: 'upgrade_schema_delete_failed'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
calls = {0x85844: 0x5dba4, 0x85858: 0xc36c4, 0x85a2c: 0x8bb80,
    0x85b1c: 0xc2820, 0x85b2c: 0xc2820, 0x8bbac: 0x8bc94,
    0x8bbb8: 0x8bc94, 0x8bbd0: 0x8bc94, 0x8bbdc: 0x8bc94,
    0x8bcf0: 0xc2820, 0x8bd2c: 0xcd310, 0xc285c: 0xcd8b0,
    0x85b50: 0x8acd0, 0x8ad50: 0xc2310, 0x85c04: 0x8ada0}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def ordered(table_first, table_ok, meta_ok):
    if table_first:
        return {'attempted_pairs': ['table'] + (['meta'] if table_ok else []),
                'result': (2 | int(meta_ok)) if table_ok else 0}
    return {'attempted_pairs': ['meta'] + (['table'] if meta_ok else []),
            'result': (3 if table_ok else 1) if meta_ok else 0}
def retry(outcomes):
    attempted = 0
    sleeps = []
    for i, ok in enumerate(outcomes[:3]):
        attempted += 1
        if ok:
            return {'attempts': attempted, 'sleep_ms': sleeps, 'result': True}
        sleeps.append(4 << i)
    return {'attempts': attempted, 'sleep_ms': sleeps, 'result': False}
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'calls': {hex(k): hex(v) for k, v in calls.items()},
    'projection_authority': 'analyst-derived normal branch outcomes, not filesystem execution or network retry policy',
    'synthetic_ordered_results': {f'table_first={first},table={t},meta={m}': ordered(first, t, m) for first in (False, True) for t in (False, True) for m in (False, True)},
    'synthetic_retry': {'first_success': retry([True]), 'second_success': retry([False, True]), 'all_fail': retry([False, False, False])},
    'default_path': 'both rename helpers called once, even when the first fails; both must return true',
    'remaining': 'flag producers, exact paths, OS/filesystem durability, rollback/restart outcomes and active-state installation'
}, indent=2))
