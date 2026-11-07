"""Pinned file framing and selected two-write cleanup path; no target execution."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset, cstring, relocs = [elf[k] for k in ('b', 'offset', 'cstring', 'relocs')]
strings = {0x17924: 'FBMobileConfigStorageManager.writeToDiskWrapper',
    0x17e7a: 'Failed to write magic code header',
    0x18824: 'Invalid input: bodyPointer is null',
    0x1bd33: 'Invalid input: bodySize is 0',
    0x185b8: 'Failed to write file format version',
    0x16b46: 'Failed to write body offset',
    0x1785e: 'Failed to write magic code footer',
    0x19887: 'Stream in bad state after close',
    0x17610: 'WriteToDiskError', 0x19f86: '.mctable', 0x1ad70: '.meta'}
for va, value in strings.items():
    if cstring(va) != value:
        raise ValueError('diagnostic mismatch')
if b[offset(0x23154):offset(0x23154)+2] != bytes.fromhex('3cf1'):
    raise ValueError('magic mismatch')
if relocs[0xd5630] != 0x860fc:
    raise ValueError('writer virtual binding mismatch')
calls = {0x86154: 0x880f8, 0x8618c: 0x98e40, 0x861a8: 0xcd560,
    0x862f0: 0xcd4f0, 0x86348: 0xcd4f0, 0x863ac: 0xcd560,
    0x86438: 0xcd560, 0x865e4: 0xcd560, 0x86638: 0x9901c,
    0x99030: 0xcd6d0, 0x98eb0: 0x990c0, 0x87208: 0xc2310,
    0xc234c: 0xcd8a0, 0x87224: 0x80244}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def frame(body):
    # Analyst projection of little-endian ARM64 writes, not the native writer.
    return bytes.fromhex('3cf1') + bytes([1, 8]) + struct.pack('<I', (len(body)+10) & 0xffffffff) + body + bytes.fromhex('3cf1')
print(json.dumps({'member_sha256': elf['digest'],
    'diagnostics': {hex(k): v for k, v in strings.items()},
    'selected_calls': {hex(k): hex(v) for k, v in calls.items()},
    'writer': '0x860fc, selected StorageManager virtual slot +0xb0',
    'input': 'body pointer +0x40; unsigned 32-bit size +0x30; both must be nonzero',
    'frame': '3c f1; byte1; byte8; little-endian u32(size+10 modulo 2^32); body; 3c f1',
    'open_mode': 'caller passes 4; wrapper ORs 0x10; filebuf open receives 0x14',
    'checks': 'state mask5 after each write; after explicit close, bit0 must be clear',
    'selected_alternate_path': 'first write false skips second; second write false attempts remove(first path); second true sets result first byte1',
    'projection_authority': 'synthetic body samples are analyst-derived, not native execution or persistence acceptance',
    'synthetic_frames': {'one_byte': frame(b'A').hex(), 'three_bytes': frame(b'abc').hex()},
    'remaining': 'source payload formats, actual paths, flag1 branch, all result meanings, crash recovery, durability and activation'
}, indent=2))
