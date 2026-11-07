"""Pinned response-consumer gates; static inspection, no target execution."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring = [elf[k] for k in ('b', 'offset', 'cstring')]
strings = {0x16b7c: 'Invalid json root object', 0x176b4: 'JSONDeserializeError', 0x198cf: 'Failed to parse delta response'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
calls = {0x83184: 0xae568, 0xae5a8: 0xb2ebc, 0xae5bc: 0xcc370, 0xae5e0: 0x5d1ec, 0xae5f4: 0xc9de0, 0x831a0: 0xca4a8, 0x83210: 0x8d670, 0x83388: 0x60484, 0x8347c: 0x60484, 0x83420: 0xae528}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
print(json.dumps({
    'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'selected_calls': {hex(k): hex(v) for k, v in calls.items()},
    'consumer_entry': 'allocates default 0x4c0-byte object before root/RequestData checks; early normal returns retain output',
    'root_gate': 'tag3 or tag4 with zero word+0x10 proceeds; other nonnull kinds set diagnostic',
    'caller_error_gate': 'nonempty context diagnostic OR absent RequestData produces internal result byte10',
    'special_flag_gate': 'byte498==1 AND (byte499&1)==0 AND (byte49a&1)==0 produces internal result byte13',
    'next_processing_call': '0x83210 -> 0x8d670; not evidence of persistence or successful activation',
    'authority': 'static byte identities and analyst transcription, not native execution or HTTP statuses',
    'remaining': 'callback tag mapping, decoded type, field lookup key/global initialization, consumer body, processing/persistence and retry semantics'
}, indent=2))
