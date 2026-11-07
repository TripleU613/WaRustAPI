"""Pinned upgrade dispatch and schema/hash result use; offline only."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x17d25: 'FBMobileConfigStorageManager.upgradeAndSwapFlatbuffer',
    0x1b073: 'FBMobileConfigStorageManager.upgradeSchema',
    0x19fee: 'FBMobileConfigStorageManager::buildLatestConfigWithTranslationTable',
    0x19afb: 'upgrade_failure', 0x1a93a: 'UpdateFailed',
    0x1a8ae: 'mobile_config_sync_fetch_upgrade_failed', 0x1a8d6: 'schema_upgrade_failed',
    0x191b0: 'FBMobileConfigStorageManager::isSchemaMismatch'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
slots = {0xd5610: 0x84a24, 0xd5618: 0x85060, 0xd5620: 0x85164,
         0xd5650: 0x873ec, 0xd5660: 0x878e0}
if any(relocs[k] != v for k, v in slots.items()):
    raise ValueError('selected virtual binding mismatch')
calls = {0x84b20: 0x8bda0, 0x84b64: 0x8bfe0, 0x84f64: 0x8cee8,
    0x850bc: 0x8bda0, 0x85108: 0x8bfe0, 0x8bdec: 0xa2a70,
    0x8be08: 0xa8fa4, 0x8be20: 0xa8fa4, 0x8bea4: 0x8ceb0,
    0x8befc: 0x531f8, 0x852a0: 0x97d08, 0x8531c: 0x8ede0,
    0x853c8: 0x8f3fc, 0x85408: 0x91198, 0x85424: 0x902a4,
    0x85c04: 0x8ada0}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def translation_route(nonempty, mismatch, flag268, mode, build_result):
    if not nonempty:
        return 9
    if (mismatch & 0xffff) < 0x100 or not (flag268 & 1 or (mode & 0xfffffffe) == 2):
        return 10
    return (13 if build_result else 15) + (mode != 3)
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'virtual_bindings': {hex(k): hex(v) for k, v in slots.items()},
    'selected_calls': {hex(k): hex(v) for k, v in calls.items()},
    'main_shortcut': 'schema result bit0 AND (manager268 bit0 OR manager26c in {2,3}) AND helper8bfe0 true; otherwise virtual+a0',
    'main_failed_a0': 'calls cleanup8cee8 with upgrade_failure; sets manager2f8=1 and diagnostic UpdateFailed',
    'translation_gate': 'nonempty selected string AND low16(schema result)>=256 AND (manager268 bit0 OR manager26c in {2,3})',
    'projection_authority': 'analyst-derived dispatch examples, not native loading or activation tests',
    'synthetic_translation_codes': {'empty': translation_route(False, 0, 0, 0, False),
        'only_low_bit': translation_route(True, 1, 1, 0, True),
        'mode3_true': translation_route(True, 0x100, 0, 3, True),
        'mode2_false': translation_route(True, 0x100, 0, 2, False)},
    'remaining': 'file reader validation, normalization helper, build/upgrade bodies, state installation and active configuration admission'
}, indent=2))
