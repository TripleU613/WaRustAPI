#!/usr/bin/env python3
"""Independent synthetic protobuf/HMAC vectors for APK Signal versions 3/4."""
import argparse
import hmac
import json
from pathlib import Path


def varint(value):
    out = bytearray()
    while value >= 128:
        out.append((value & 127) | 128)
        value >>= 7
    out.append(value)
    return bytes(out)


def number(tag, value):
    return varint(tag << 3) + varint(value)


def blob(tag, value):
    return varint(tag << 3 | 2) + varint(len(value)) + value


def vectors():
    sender = bytes([5]) + bytes(range(1, 33))
    receiver = bytes([5]) + bytes(range(33, 65))
    ratchet = bytes([5]) + bytes(range(65, 97))
    mac_key = bytes(range(32))
    signals, prekeys = [], []
    for version in (3, 4):
        for counter, previous in ((0, 0), (1, 0), (127, 128), (0xffffffff, 0xffffffff)):
            for length in (0, 16, 32):
                ciphertext = bytes(range(length))
                wire = bytes([version * 17]) + blob(1, ratchet) + number(2, counter) + number(3, previous) + blob(4, ciphertext)
                wire += hmac.digest(mac_key, sender + receiver + wire, 'sha256')[:8]
                signals.append(dict(version=version, counter=counter, previous=previous,
                                    ciphertext=ciphertext.hex(), wire=wire.hex()))
        inner = bytes.fromhex(signals[-1]['wire'])
        for one_time in (None, 0):
            for kem in (None, dict(id=0xffffffff, ciphertext=(bytes(1568)).hex())):
                wire = bytes([version * 17])
                if one_time is not None:
                    wire += number(1, one_time)
                wire += blob(2, ratchet) + blob(3, sender) + blob(4, inner) + number(5, 128) + number(6, 0xffffffff)
                if kem is not None:
                    wire += number(7, kem['id']) + blob(8, bytes.fromhex(kem['ciphertext']))
                prekeys.append(dict(version=version, registration=128, signed=0xffffffff,
                                    one_time=one_time, kem=kem, nested=inner.hex(), wire=wire.hex()))
    return dict(scope='Synthetic serialization and HMAC inputs; not APK execution, encryption or valid KEM ciphertext.',
                sender=sender.hex(), receiver=receiver.hex(), ratchet=ratchet.hex(),
                mac_key=mac_key.hex(), signals=signals, prekeys=prekeys)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    data = json.dumps(vectors(), indent=2) + '\n'
    if args.check:
        if args.output.read_text() != data:
            raise ValueError('Signal wire vectors differ')
    else:
        args.output.write_text(data)
    print('Verified 24 Signal and 8 prekey wire vectors')


if __name__ == '__main__':
    main()
