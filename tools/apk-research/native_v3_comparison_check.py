#!/usr/bin/env python3
"""Execute pinned APK V3 comparison core/remapper on built-in synthetic inputs."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from native_fingerprint_check import LIBRARY_HASH
from native_v4_comparison_check import NativeAbort, execute


def run_cases(library):
    # APK 37.71 0x7140fc..0x714358 return branches, 0x714408 presence
    # mapping, and 0x302370 remapping table. Expectations are literal native
    # branches, not outputs computed by a second port of the comparison.
    base = [dict(lid=b'synthetic-local', pn=b'local-pn', username=b'Local',
                 public=b'local-key', hash=b'local-hash', hosted=0),
            dict(lid=b'synthetic-remote', pn=b'remote-pn', username=b'Remote',
                 public=b'remote-key', hash=b'remote-hash', hosted=1)]
    mapping = {0:0, -12:-11, -14:-14, -15:-15, -17:-16, -19:-18,
               -21:-20, -22:-22, -24:-25, -26:-27, -28:-28,
               -29:-30, -31:-32, -33:-33, -34:-35, -36:-37}
    records = []

    def check(name, known, scanned, expected, version=0, options=3):
        result = execute(library, known, scanned, version, remap=True,
                         scheme=3, options=options)
        if result['code'] != expected or result['remapped_code'] != mapping[expected]:
            raise ValueError(f'{name}: native V3 status/table differed: {result["code"]}')
        def inputs(sides):
            return [{field: value.decode('ascii') if isinstance(value, bytes) else value
                     for field, value in side.items()} for side in sides]
        # Built-in synthetic modeled inputs make the independently executed
        # native result reusable by client regression tests; no account data.
        records.append(dict(case=name, version=version, options=options,
                            known=inputs(known), scanned=inputs(scanned), **result))

    for version in (0, 1):
        for options in (1, 3):
            suffix = f'-v{version}-options{options}'
            check('baseline'+suffix, base, base, 0, version, options)
            for side in (0, 1):
                for selected in (False, True):
                    scanned = copy.deepcopy(base)
                    field = ('public' if version == 0 else 'hash') if selected else ('hash' if version == 0 else 'public')
                    scanned[side][field] = b'different'
                    check(f'key-side{side}-selected{selected}'+suffix, base, scanned,
                          (-14 if side == 0 else -15) if selected else 0, version, options)
            # Core remote identity accepts PN OR retained LID; changing one
            # does not disprove the other. Local orientation is wrapper-owned.
            scanned = copy.deepcopy(base); scanned[1]['lid'] = b'different'
            check('remote-lid-different-pn-equal'+suffix, base, scanned, 0, version, options)
            scanned[1]['pn'] = b'different'
            check('remote-both-identifiers-different'+suffix, base, scanned, -12, version, options)
            scanned = copy.deepcopy(base); scanned[0]['lid'] = b'different'
            check('core-does-not-check-local-lid'+suffix, base, scanned, 0, version, options)
            # Keep LIDs unchanged, permitting remote identity match by LID.
            for field, codes in [('pn', [(-17, 0, -17), (-19, -21, -22)]),
                                 ('username', [(-24, -26, -28), (-29, -31, -33)])]:
                for side in (0, 1):
                    for variant, expected in zip(('different', 'scanned-absent', 'known-absent'), codes[side]):
                        known = copy.deepcopy(base); scanned = copy.deepcopy(base)
                        if variant == 'different': scanned[side][field] = b'different'
                        elif variant == 'scanned-absent': scanned[side][field] = None
                        else: known[side][field] = None
                        check(f'{field}-side{side}-{variant}'+suffix, known, scanned, expected, version, options)
                    known = copy.deepcopy(base); scanned = copy.deepcopy(base)
                    known[side][field] = scanned[side][field] = None
                    check(f'{field}-side{side}-both-absent'+suffix, known, scanned, 0, version, options)

    # Exhaust all valid known/scanned hosted pairs. The actual core skips
    # known matching when both scanned states are absent; option 1 skips all.
    for options in (1, 3):
        for kl in (-1, 0, 1):
            for kr in (-1, 0, 1):
                for sl in (-1, 0, 1):
                    for sr in (-1, 0, 1):
                        known = copy.deepcopy(base); scanned = copy.deepcopy(base)
                        known[0]['hosted'], known[1]['hosted'] = kl, kr
                        scanned[0]['hosted'], scanned[1]['hosted'] = sl, sr
                        expected = 0 if options == 1 or (sl == sr == -1) else (-34 if kl != sl else (-36 if kr != sr else 0))
                        check(f'hosted-known{kl},{kr}-scanned{sl},{sr}-options{options}', known, scanned, expected, options=options)
    # Bit 0 is always on at the traced UI caller; exercise off only as an
    # internal core test, without claiming an APK UI setting for it.
    for options in (0, 2):
        scanned = copy.deepcopy(base); scanned[0]['username'] = b'different'
        check(f'internal-username-bit-off-{options}', base, scanned, 0, options=options)
    # Explicit precedence fixtures catch accidental reordering.
    scanned = copy.deepcopy(base)
    scanned[1]['public'] = b'different'; scanned[0]['pn'] = b'different'
    check('remote-key-before-local-pn', base, scanned, -15)
    scanned = copy.deepcopy(base)
    scanned[1]['pn'] = b'different'; scanned[0]['username'] = b'different'
    check('remote-pn-before-local-username', base, scanned, -19)
    scanned = copy.deepcopy(base)
    scanned[1]['username'] = b'different'; scanned[0]['hosted'] = 1
    check('remote-username-before-local-hosted', base, scanned, -29)
    scanned = copy.deepcopy(base)
    scanned[0]['hosted'] = -2
    try:
        execute(library, base, scanned, 0, scheme=3, options=3)
    except NativeAbort:
        records.append(dict(case='invalid-scanned-local-hosted-invariant', native_abort=True))
    else:
        raise ValueError('expected V3 native hosted invariant abort')
    return {'apk_build':'2.26.37.71', 'library_sha256':LIBRARY_HASH,
            'scope':'actual V3 core, optional-value and PN-or-LID helpers/remapper; modeled dependencies (key getters/equality, object getters, MCFEqual, logging); no decoder, JID construction, orientation wrapper, JNI, account or human action',
            'cases':records}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library', type=Path, required=True)
    args = parser.parse_args()
    if not args.library.is_file(): parser.error('missing reconstructed native library')
    if hashlib.sha256(args.library.read_bytes()).hexdigest() != LIBRARY_HASH:
        parser.error('requires the pinned 37.71 reconstructed libwhatsapp.so')
    try:
        result = run_cases(args.library)
    except ImportError as error:
        parser.exit(1, f'missing research dependency: {error.name}\n')
    except (ValueError, KeyError) as error:
        parser.exit(1, f'native V3 check failed: {error}\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__': main()
