"""Pinned framing gate and hash-prefix projection; no native reader execution."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x17d83: 'verifyBuffer', 0x15ed8: 'empty', 0x1618b: 'verifyAndLoadBody'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
if b[offset(0x24a28):offset(0x24a28)+3] != b'\x02:\x00':
    raise ValueError('short-string colon changed')
if relocs[0xd61d8] != 0xa2ee4:
    raise ValueError('verifier slot changed')
calls = {0xa2a94: 0xa08d0, 0xa2aec: 0xa12bc, 0xa2b20: 0xa2bf4,
    0x881a4: 0xa2a70, 0xa179c: 0xc2310, 0xa8fc4: 0x9e308,
    0x5e888: 0x5dd68, 0x5e89c: 0xccb90, 0xa2f54: 0x9ac7c}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def envelope(data):
    if not data.startswith(bytes.fromhex('3cf1')):
        return {'route': 'raw verifier handoff', 'body_bytes': len(data)}
    n = len(data)
    if n < 11 or data[2] > 1 or struct.unpack_from('<I', data, 4)[0] != n or data[-2:] != bytes.fromhex('3cf1') or n-2 <= data[3]:
        return {'route': 'framing diagnostic'}
    return {'route': 'framed verifier handoff', 'body_offset': data[3], 'body_bytes': n-data[3]-2}
def sample(version=1, body_offset=8):
    body = b'abc'
    return bytes.fromhex('3cf1') + bytes([version, body_offset]) + struct.pack('<I', len(body)+10) + body + bytes.fromhex('3cf1')
bad_size = bytearray(sample()); bad_size[4] ^= 1
bad_footer = bytearray(sample()); bad_footer[-1] = 0
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'calls': {hex(k): hex(v) for k, v in calls.items()},
    'verifier': {'address_point': '0xd61c0', 'slot': '0xd61d8', 'target': '0xa2ee4', 'selected_min_body_bytes': 12},
    'hash_projection': 'prefix before first colon; unchanged when absent',
    'synthetic_hash_prefixes': {s: s.split(':', 1)[0] for s in ['abc:v1', 'abc:v2', ':suffix', 'plain']},
    'projection_authority': 'analyst-derived envelope and ASCII prefix examples; no native verification or acceptance',
    'synthetic_envelopes': {'writer_shape': envelope(sample()), 'version_zero': envelope(sample(0)),
        'version_two': envelope(sample(2)), 'offset_zero': envelope(sample(body_offset=0)),
        'wrong_size': envelope(bad_size), 'wrong_footer': envelope(bad_footer),
        'raw': envelope(b'abc')},
    'remaining': 'provider binding/readiness, all verifier helpers and table schema, diagnostic cleanup effects and active-state admission'
}, indent=2))
