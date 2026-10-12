#!/usr/bin/env python3
"""Synthetic independent HMAC-SHA256 vectors for the APK's initial hybrid KDF."""
import argparse
import hashlib
import hmac
import json
from pathlib import Path

LABEL = b'WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024'


def vectors():
    cases = []
    for pattern in range(8):
        values = [bytes([0 if pattern == 0 else 255]) * 32 if pattern < 2
                  else hashlib.sha256(f'synthetic-kdf-{pattern}-{index}'.encode()).digest()
                  for index in range(5)]
        for one_time in (False, True):
            agreements = values[:3] + ([values[3]] if one_time else [])
            material = b'\xff' * 32 + b''.join(agreements) + values[4]
            extracted = hmac.digest(bytes(32), material, 'sha256')
            root = hmac.digest(extracted, LABEL + b'\x01', 'sha256')
            chain = hmac.digest(extracted, root + LABEL + b'\x02', 'sha256')
            cases.append(dict(name=f'pattern-{pattern}-one-time-{one_time}',
                              agreements=[value.hex() for value in values[:3]],
                              one_time=values[3].hex() if one_time else None,
                              kem_shared=values[4].hex(), root=root.hex(), chain=chain.hex()))
    return dict(scope='Synthetic already-computed agreements; Python standard HMAC, not APK execution or DH/KEM verification.',
                label=LABEL.decode(), cases=cases)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    encoded = json.dumps(vectors(), indent=2) + '\n'
    if args.check:
        if args.output.read_text() != encoded:
            raise ValueError('hybrid KDF vectors differ')
    else:
        args.output.write_text(encoded)
    print('Verified 16 hybrid initial KDF vectors')


if __name__ == '__main__':
    main()
