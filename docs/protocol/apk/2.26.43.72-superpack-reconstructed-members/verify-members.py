"""Check reconstructed files against pinned metadata, XXH64 and ELF layout."""
import contextlib
import ctypes
import ctypes.util
import hashlib
import io
import json
from pathlib import Path
import runpy
import struct
import subprocess
import sys
apk, directory = sys.argv[1:]
root = Path(__file__).resolve().parent.parent
sys.argv = [sys.argv[0], apk]
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(root / '2.26.43.72-superpack-metadata/inspect-metadata.py'))
hash_library = ctypes.CDLL(ctypes.util.find_library('xxhash'))
hash64 = hash_library.XXH64
hash64.restype = ctypes.c_uint64
hash64.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint64]
version = hash_library.XXH_versionNumber
version.restype = ctypes.c_uint
results = []
for member in evidence['records']:
    target = Path(directory) / member['name']
    data = target.read_bytes()
    assert len(data) == member['declared_size'], member['name']
    digest = hash64(data, len(data), 0)
    assert digest == int(member['opaque_u64'], 16), member['name']
    assert data[:6] == bytes.fromhex('7f454c460201')
    assert struct.unpack_from('<HH', data, 16) == (3, 183)
    checked = subprocess.run(['readelf', '--file-header', '--sections', '--program-headers',
                              '--dynamic', '--wide', str(target)], capture_output=True, text=True)
    assert checked.returncode == 0 and not checked.stderr, (member['name'], checked.stderr)
    results.append({'ordinal': member['ordinal'], 'name': member['name'], 'bytes': len(data),
                    'sha256': hashlib.sha256(data).hexdigest(), 'xxh64_seed0': hex(digest),
                    'readelf_layout_clean': True})
print(json.dumps({'xxhash_version_number': version(), 'members': results}, indent=2))
