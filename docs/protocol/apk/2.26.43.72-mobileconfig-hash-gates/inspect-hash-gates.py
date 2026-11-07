"""Pinned MobileConfig hash gates and later virtual binding; offline only."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x18f07: 'UnknownQueryHash', 0x18f18: 'UnknownOneQueryHash',
           0x1b5d1: 'hasQueryHash=', 0x1b5af: '; queryHash=',
           0x1b5bc: '; previousQueryHash=', 0x1b5ef: 'hasOneQueryHash=',
           0x1b5df: '; oneQueryHash=', 0x1b6a5: '; hasOneQueryHashSupported=',
           0x1ae24: 'generateMetadata', 0x1b13b: 'response_'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
calls = {0x83210: 0x8d670, 0x8d860: 0x60484, 0x8d9c8: 0x60484,
         0x8da20: 0x8e590, 0x8db60: 0x92fd4, 0x8e148: 0x907ac,
         0x8e154: 0x908c8, 0x8e380: 0x7f970, 0x86784: 0x880f8,
         0x867b8: 0x880f8, 0x86804: 0xccb70, 0x87140: None}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    if target is None:
        if word != 0xd63f0100:
            raise ValueError('expected blr x8')
        continue
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
if relocs[0xd5638] != 0x8674c or relocs[0xd5630] != 0x860fc:
    raise ValueError('storage vtable binding mismatch')
def gates(previous_nonempty, query_presence_byte, query_nonempty,
          one_presence_byte, one_nonempty, supported_nonempty):
    query_unknown = previous_nonempty and bool(query_presence_byte & 1) and not query_nonempty
    one_unknown = one_presence_byte == 1 and not one_nonempty
    result = 4 if query_unknown else 5 if one_unknown and supported_nonempty else 'later processing'
    return {'query_unknown_output': query_unknown, 'one_unknown_output': one_unknown, 'selected_result': result}
cases = {
    'both_unknown_query_wins': gates(True, 1, False, 1, False, True),
    'query_presence_absent': gates(True, 0, False, 0, False, True),
    'previous_empty': gates(False, 1, False, 0, False, True),
    'one_unknown_supported': gates(False, 0, False, 1, False, True),
    'one_unknown_unsupported': gates(False, 0, False, 1, False, False),
    'query_presence_odd_byte': gates(True, 3, False, 0, False, True),
    'one_presence_not_exact_one': gates(False, 0, False, 3, False, True),
    'nonempty_strings': gates(True, 1, True, 1, True, True),
}
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'calls': {hex(k): hex(v) if v is not None else 'blr x8' for k, v in calls.items()},
    'conditional_storage_slot_b8': {'vtable': '0xd5580', 'slot': '0xd5638', 'target': '0x8674c'},
    'conditional_storage_slot_b0': {'slot': '0xd5630', 'target': '0x860fc'},
    'projection_authority': 'analyst-derived gate examples, not native execution or server acceptance',
    'synthetic_cases': cases,
    'remaining': 'consumer field producers, metadata body, storage write paths, commit/rollback durability, cache activation and retries'
}, indent=2))
