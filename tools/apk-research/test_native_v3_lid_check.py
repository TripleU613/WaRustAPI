"""Build refusal and pinned V3 primary-LID parser branch observations."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HELPER=Path(__file__).with_name('native_v3_lid_check.py').resolve()


class NativeV3LidChecks(unittest.TestCase):
    def invoke(self, library):
        return subprocess.run([sys.executable,str(HELPER),'--library',str(library)],
                              capture_output=True,text=True,timeout=30)

    def test_missing_library_has_no_partial_output(self):
        with tempfile.TemporaryDirectory() as directory:
            result=self.invoke(Path(directory)/'missing.so')
        self.assertEqual(result.returncode,2)
        self.assertIn('missing reconstructed native library',result.stderr)
        self.assertEqual(result.stdout,'')

    def test_wrong_build_refuses_before_emulation(self):
        with tempfile.TemporaryDirectory() as directory:
            library=Path(directory)/'wrong.so';library.write_bytes(b'wrong build')
            result=self.invoke(library)
        self.assertEqual(result.returncode,2)
        self.assertIn('requires the pinned 37.71',result.stderr)
        self.assertEqual(result.stdout,'')

    def test_actual_native_parser_branches_match_observations(self):
        configured=os.environ.get('TELEWHATSAPP_FINGERPRINT_LIBRARY')
        if not configured:
            self.skipTest('set TELEWHATSAPP_FINGERPRINT_LIBRARY for actual V3 LID branches')
        result=self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode,0,result.stderr)
        actual=json.loads(result.stdout)
        fixture=HELPER.parents[1].parent/'docs/protocol/fingerprint-native-v3-lid-cases.json'
        self.assertEqual(actual,json.loads(fixture.read_text()))
        self.assertEqual(len(actual['cases']),38)
        self.assertTrue(all(case['instructions']<1000000 for case in actual['cases']))
        self.assertIn('modeled MCF split',actual['scope'])
        cases={case['case']:case for case in actual['cases']}
        self.assertEqual(cases['suffix-00']['normalized_jid'],'123@lid')
        self.assertEqual(cases['suffix-01']['parse_error'],-2)
        for length in (16,17,18,19,21):
            self.assertFalse(cases[f'base-length-{length}']['primary_parse_succeeded'])
        self.assertTrue(cases['base-length-20']['stable_lid_succeeded'])

    def test_guest_fixture_string_bounds_refuse_before_execution(self):
        configured=os.environ.get('TELEWHATSAPP_FINGERPRINT_LIBRARY')
        if not configured:
            self.skipTest('set library fixture for bounded modeled guest allocation')
        from native_v3_lid_check import NativeLid
        native=NativeLid(Path(configured).resolve())
        for value in (b'a'*257,b'a\0b',b'\x80'):
            with self.assertRaisesRegex(ValueError,'outside fixture bounds'):
                native.string(value)
        with self.assertRaisesRegex(ValueError,'heap budget exhausted'):
            native.allocate(0x100001)
        with self.assertRaisesRegex(ValueError,'unmodeled native LID branch'):
            native.run(0x71373c,[])


if __name__=='__main__':unittest.main()
