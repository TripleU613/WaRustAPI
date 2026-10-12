"""Generate enum/presence vectors with a pinned desktop Java protobuf runtime."""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--classpath', required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
jar_digest = '91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5'
jars = [Path(part) for part in args.classpath.split(os.pathsep) if part.endswith('.jar')]
assert len(jars) == 1 and hashlib.sha256(jars[0].read_bytes()).hexdigest() == jar_digest

def varint(number):
    result = bytearray()
    while number > 127:
        result.append((number & 127) | 128)
        number >>= 7
    result.append(number)
    return bytes(result)

def field(number):
    return b'\x10' + varint(number)

def wrap(mode, body):
    return varint(((12 if mode == 'p' else 51) << 3) | 2) + varint(len(body)) + body

cases = set()
values = list(range(65)) + [127, 128, 255, 0x80000000, 0xffffffff,
    0x100000000, 0x100000001, 0x100000003, 0x100000027,
    0x7fffffffffffffff, 0xffffffffffffffff]
for mode in ['p', 'k']:
    cases.add((mode, b''))
    cases.add((mode, wrap(mode, b'')))
    for value in values:
        current = field(value)
        cases.add((mode, wrap(mode, current)))
        for prior in ([0, 3, 4] if mode == 'p' else [0, 1, 2]):
            previous = field(prior)
            for body in [previous + current, current + previous]:
                cases.add((mode, wrap(mode, body)))
            cases.add((mode, wrap(mode, previous) + wrap(mode, current)))
            cases.add((mode, wrap(mode, current) + wrap(mode, previous)))
    base = field(3 if mode == 'p' else 1)
    unknowns = [b'\x30\x01', b'\x39' + bytes(8), b'\x42\x03abc',
        b'\x4b\x10\x27\x4c', b'\x55' + bytes(4), b'\x4c',
        b'\x4b\x54', b'\x4b', b'\x42\x05abc', b'\x00', b'\x10',
        b'\x10\x80', b'\x0e', b'\x0f', b'\x80' * 10,
        b'\x12\x01\xff', b'\x11' + bytes(8), b'\x15' + bytes(4),
        b'\x13\x10\x00\x14']
    for unknown in unknowns:
        cases.add((mode, wrap(mode, base + unknown)))
        cases.add((mode, wrap(mode, unknown + base)))
    for depth in [1, 2, 97, 98, 99, 100]:
        cases.add((mode, wrap(mode, base + b'\x4b' * depth + field(39) + b'\x4c' * depth)))
    for invalid in [b'\x00', b'\x80', b'\x0c', b'\x0b', b'\x0b\x14']:
        cases.add((mode, invalid))
ordered = sorted(cases)
result = subprocess.run(['java', '-cp', args.classpath, 'ProtocolEnumsOracle'],
    input=''.join(mode + '|' + data.hex() + '\n' for mode, data in ordered),
    capture_output=True, text=True, check=True)
lines = result.stdout.splitlines()
assert len(lines) == len(ordered)
rows = []
for (mode, data), line in zip(ordered, lines):
    rows.append({'mode': mode, 'hex': data.hex(), 'valid': line != 'malformed',
        'value': None if line in ['absent', 'malformed'] else int(line)})
fixture = {'apk_build': '2.26.37.71',
    'apk_sha256': '1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c',
    'scope': 'Projected enum fields/presence and singular-message merge, not full APK schemas or execution',
    'source': 'BZb/BV3 field 2 and CvY/C4F, Cv0/C2L enum verifiers',
    'oracle': 'docs/protocol/ProtocolEnumsOracle.java',
    'runtime': 'protobuf-javalite 4.36.1 on desktop Java; APK runtime version unknown',
    'runtime_url': 'https://repo.maven.apache.org/maven2/com/google/protobuf/protobuf-javalite/4.36.1/protobuf-javalite-4.36.1.jar',
    'runtime_sha256': jar_digest, 'cases': rows}
args.output.write_text(json.dumps(fixture, indent=2) + '\n')
print(f'{len(rows)} independent Java vectors written')
