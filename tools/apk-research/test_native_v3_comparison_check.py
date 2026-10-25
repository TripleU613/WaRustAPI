"""Pinned native V3 core checks, build refusals and deterministic observations."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HELPER = Path(__file__).with_name('native_v3_comparison_check.py').resolve()


class NativeV3Checks(unittest.TestCase):
    def invoke(self, library):
        return subprocess.run([sys.executable, str(HELPER), '--library', str(library)],
                              capture_output=True, text=True, timeout=30)

    def test_missing_fixture_has_no_partial_output(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(Path(directory) / 'missing.so')
        self.assertEqual(result.returncode, 2)
        self.assertIn('missing reconstructed native library', result.stderr)
        self.assertEqual(result.stdout, '')

    def test_wrong_build_refuses_before_emulation(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / 'wrong.so'
            library.write_bytes(b'wrong build')
            result = self.invoke(library)
        self.assertEqual(result.returncode, 2)
        self.assertIn('requires the pinned 37.71', result.stderr)
        self.assertEqual(result.stdout, '')

    def test_actual_native_core_and_remapper(self):
        configured = os.environ.get('WARUSTAPI_FINGERPRINT_LIBRARY')
        if not configured:
            self.skipTest('set WARUSTAPI_FINGERPRINT_LIBRARY for actual V3 execution')
        result = self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode, 0, result.stderr)
        actual = json.loads(result.stdout)
        fixture = HELPER.parents[1].parent / 'docs/protocol/fingerprint-native-v3-comparison-cases.json'
        self.assertEqual(actual, json.loads(fixture.read_text()))
        self.assertEqual(len(actual['cases']), 264)
        self.assertEqual(sum(c.get('native_abort', False) for c in actual['cases']), 1)
        self.assertTrue(all(c.get('instructions', 0) < 1_000_000 for c in actual['cases']))
        self.assertIn('modeled dependencies', actual['scope'])

    def test_shared_runner_rejects_unrecognized_modes_before_emulation(self):
        from native_v4_comparison_check import execute
        with self.assertRaisesRegex(ValueError, 'unsupported synthetic comparison'):
            execute(Path('unused.so'), [], [], 0, scheme=2)
        with self.assertRaisesRegex(ValueError, 'unsupported synthetic comparison'):
            execute(Path('unused.so'), [], [], 0, scheme=3,
                    string_libraries=(Path('unused.so'), Path('unused.so')))


if __name__ == '__main__': unittest.main()
