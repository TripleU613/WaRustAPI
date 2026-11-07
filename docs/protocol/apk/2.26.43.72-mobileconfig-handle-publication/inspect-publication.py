"""Pinned shared-handle publication and status; no native state mutation."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x191df: 'shared_extension_schema_match', 0x1767c: 'SchemaMismatchError',
           0x19f34: 'FBMobileConfigStorageManager::validateAndCreateMmapHandle'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
slots = {0xd5668: 0x87940, 0xd5648: 0x87388, 0xd55c0: 0x83c7c,
         0xd61d0: 0x886ec, 0xd5890: 0x886ec}
if any(relocs[k] != v for k, v in slots.items()):
    raise ValueError('selected virtual binding mismatch')
calls = {0x879bc: 0x88144, 0x87a0c: 0x531f8, 0x87a30: 0xa8fa4,
    0x87a44: 0x531f8, 0x87b38: 0xa3d78, 0x87c30: 0x87f90,
    0x87c44: 0xccb70, 0x87c5c: 0x75fc8, 0x87cb8: 0x5e4a8,
    0x87cfc: 0xc4338, 0x87f40: 0x88344, 0x83c98: 0xccb70}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def publication(candidate_nonnull, root_nonnull, reason_category):
    appended = [] if candidate_nonnull and root_nonnull else [8 if candidate_nonnull else 4]
    status = (2 if candidate_nonnull and root_nonnull else 3) if reason_category == 5 else (6 if candidate_nonnull and root_nonnull else 7)
    return {'appended_at_probe': appended, 'replaces_shared_pair': True, 'stored_status': status}
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'virtual_bindings': {hex(k): hex(v) for k, v in slots.items()},
    'calls': {hex(k): hex(v) for k, v in calls.items()},
    'probe': '0x886ec tests only object+0x50 nonnull through 0xcabbc',
    'publication': 'candidate pair replaces manager+0x20 under mutex+0x34; status byte+0x30 is stored before unlock',
    'projection_authority': 'analyst-derived publication suffix cases, not execution or complete upstream admission',
    'synthetic_cases': {'normal_root': publication(True, True, 6), 'missing_root': publication(True, False, 6), 'null_candidate': publication(False, False, 6), 'category5_missing_root': publication(True, False, 5)},
    'remaining': 'complete rebuild/candidate constructors, status meanings, observer/owner follow-up and active configuration semantics'
}, indent=2))
