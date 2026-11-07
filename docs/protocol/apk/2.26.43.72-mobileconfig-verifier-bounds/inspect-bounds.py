"""Pinned verifier helpers and qualified boundary projections; offline only."""
import contextlib, io, json, runpy, struct
from pathlib import Path
base = Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
    elf = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b, offset = [elf[k] for k in ('b', 'offset')]
limits = struct.unpack_from('<II', b, offset(0x1cd10))
if limits != (64, 1000000):
    raise ValueError('verifier limit literal mismatch')
calls = {0x9aca8: 0x9b050, 0x9ade0: 0x9ac7c, 0x9ae2c: 0x9afc0,
    0x9aee0: 0x9adfc, 0x9aff8: 0x9b050, 0xa2f54: 0x9ac7c,
    0xa2f64: 0x9acec, 0xa2f74: 0x9ad9c, 0xa2f88: 0x9adfc,
    0xa2fb8: 0x9afc0}
for site, target in calls.items():
    word = struct.unpack_from('<I', b, offset(site))[0]
    imm = word & 0x3ffffff
    if imm & (1 << 25):
        imm -= 1 << 26
    if word >> 26 != 0b100101 or site + 4 * imm != target:
        raise ValueError('selected BL mismatch')
def word_span(size, position, aligned=True):
    return (not aligned or position % 4 == 0) and size > 4 and 0 <= position <= size-4
def root(size, position, delta, aligned=True):
    return word_span(size, position, aligned) and 1 <= delta <= 0x7fffffff and position+delta <= size-1
def vector(size, position, count, stride, aligned=True):
    needed = 4 + count * stride
    return word_span(size, position, aligned) and 0 <= count < 0x7fffffff // stride and size > needed and position <= size-needed
print(json.dumps({'member_sha256': elf['digest'], 'counter_limits': list(limits),
    'selected_calls': {hex(k): hex(v) for k, v in calls.items()},
    'projection_authority': 'analyst-derived helper bounds with nonnegative offsets and positive strides; no native execution or complete table verification',
    'synthetic_word_span': {'exact4bytes': word_span(4, 0), 'word_at_tail': word_span(8, 4), 'unaligned': word_span(12, 1)},
    'synthetic_root': {'positive_in_range': root(12, 0, 4), 'zero_delta': root(12, 0, 0), 'negative_delta': root(12, 0, -4), 'target_one_past_end': root(12, 0, 12)},
    'synthetic_vector': {'exact_whole_buffer': vector(12, 0, 2, 4), 'one_spare_byte': vector(13, 0, 2, 4), 'tail_span': vector(16, 4, 2, 4), 'overrun': vector(15, 4, 2, 4), 'count_at_cap': vector(0xffffffff, 0, 0x7fffffff//4, 4)},
    'required_final_scalar': 'root vtable entry 0x1c must exist and contain numeric 123456',
    'remaining': 'field semantics, all nested table verifiers, counter lifecycle, actual fixtures and active-state admission'
}, indent=2))
