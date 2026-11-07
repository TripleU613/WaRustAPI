"""Cross-member default executor binding; static only, no callable execution."""
import argparse, contextlib, hashlib, io, json, runpy, struct, sys
from pathlib import Path
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('mobileconfig', type=Path)
p.add_argument('merged', type=Path)
a = p.parse_args()
base = Path(__file__).resolve().parent
sys.argv = [sys.argv[0], str(a.mobileconfig)]
with contextlib.redirect_stdout(io.StringIO()):
    mc = runpy.run_path(str(base.parent / '2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b = a.merged.read_bytes()
digest = hashlib.sha256(b).hexdigest()
if digest != 'd0ab3f2237e92a76ac3519c3577b5836b9e6f65a186c2016cefd43c4923f4b0d' or len(b) != 7871544:
    raise ValueError('merged member identity mismatch')
prior = json.loads((base.parent / '2.26.43.72-superpack-reconstructed-members/validation.json').read_text())
if not any(row.get('name') == 'libwhatsappmerged.so' and row.get('sha256') == digest and row.get('bytes') == len(b) for row in prior):
    raise ValueError('prior reconstruction mismatch')
if b[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', b, 18)[0] != 183:
    raise ValueError('merged ELF identity mismatch')
phoff = struct.unpack_from('<Q', b, 32)[0]
phsize, phnum = struct.unpack_from('<HH', b, 54)
loads = [struct.unpack_from('<IIQQQQQQ', b, phoff+i*phsize) for i in range(phnum)]
def offset(va):
    found = [off+va-vaddr for typ,flags,off,vaddr,paddr,filesz,memsz,align in loads if typ == 1 and vaddr <= va < vaddr+filesz]
    if len(found) != 1:
        raise ValueError('merged mapping mismatch')
    return found[0]
shoff = struct.unpack_from('<Q', b, 40)[0]
shsize, shnum = struct.unpack_from('<HH', b, 58)
sections = [struct.unpack_from('<IIQQQQIIQQ', b, shoff+i*shsize) for i in range(shnum)]
relocs, symbols = {}, {}
for sh in sections:
    if sh[1] == 4:
        for pos in range(sh[4], sh[4]+sh[5], sh[9]):
            va, info, addend = struct.unpack_from('<QQq', b, pos)
            if info & 0xffffffff == 1027:
                relocs[va] = addend
    if sh[1] == 11:
        strings = sections[sh[6]]
        for pos in range(sh[4], sh[4]+sh[5], sh[9]):
            name,info,other,index,value,size = struct.unpack_from('<IBBHQQ', b, pos)
            start = strings[4]+name
            text = b[start:b.index(0,start)].decode()
            if 'InlineSerialExecutor' in text:
                symbols[text] = {'value': hex(value), 'bytes': size, 'defined': index != 0}
vtable = '_ZTVN8facebook6mobile5xplat8executor20InlineSerialExecutorE'
vtt = '_ZTTN8facebook6mobile5xplat8executor20InlineSerialExecutorE'
if symbols.get(vtable) != {'value': '0x757780', 'bytes': 136, 'defined': True} or symbols.get(vtt) != {'value': '0x757808', 'bytes': 64, 'defined': True}:
    raise ValueError('executor export mismatch')
if relocs.get(0x7577e0) != 0x6679f0 or relocs.get(0x757808) != 0x7577d0:
    raise ValueError('selected executor relocation mismatch')
adjustment_site = 0x7577d0-0x48
if adjustment_site in relocs or struct.unpack_from('<q', b, offset(adjustment_site))[0] != 0:
    raise ValueError('default virtual-base adjustment mismatch')
expected = [0xd503245f, 0xf9401830, 0xaa0103e0, 0xd61f0200]
if list(struct.unpack_from('<IIII', b, offset(0x6679f0))) != expected:
    raise ValueError('direct invoke body mismatch')
print(json.dumps({'mobileconfig_sha256': mc['digest'], 'merged_sha256': digest,
    'merged_bytes': len(b), 'executor_exports': symbols,
    'selected_default_binding': {'address_point': '0x7577d0', 'virtual_base_adjustment': 0, 'slot10': '0x7577e0', 'target': '0x6679f0'},
    'instructions': ['bti c', 'ldr x16, [x1, #0x30]', 'mov x0, x1', 'br x16'],
    'callable_handoff': 'selected MobileConfig holder+0x30 contains 0x8b384; default dispatch tail-invokes it directly',
    'authority': 'cross-member static binding, not runtime dispatch or persisted-byte observation',
    'remaining': 'actual dynamic symbol resolution, supplied executor overrides, exceptions, file outcomes and active-table installation'
}, indent=2))
