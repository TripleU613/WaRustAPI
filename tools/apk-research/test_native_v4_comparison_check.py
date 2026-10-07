"""Build preflight and actual bounded synthetic native V4 branch verification."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_v4_comparison_check.py").resolve()


class NativeV4Checks(unittest.TestCase):
    def invoke(self, library):
        return subprocess.run(
            [sys.executable, str(HELPER), "--library", str(library)],
            capture_output=True, text=True, timeout=30,
        )

    def test_missing_fixture_has_no_partial_output(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(Path(directory) / "missing.so")
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing reconstructed native library", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_wrong_build_refuses_before_emulation(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / "other.so"
            library.write_bytes(b"wrong build")
            result = self.invoke(library)
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires the pinned 37.71", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_actual_native_branches_match_committed_observations(self):
        configured = os.environ.get("TELEWHATSAPP_FINGERPRINT_LIBRARY")
        if not configured:
            self.skipTest("set TELEWHATSAPP_FINGERPRINT_LIBRARY for native V4 slice execution")
        result = self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode, 0, result.stderr)
        actual = json.loads(result.stdout)
        fixture = HELPER.parents[1].parent / "docs/protocol/fingerprint-native-comparison-cases.json"
        self.assertEqual(actual, json.loads(fixture.read_text()))
        self.assertTrue(all(case.get("instructions", 0) < 1000000 for case in actual["cases"]))
        self.assertIn("modeled dependencies", actual["scope"])
        self.assertEqual(sum(case.get("native_abort", False) for case in actual["cases"]), 2)


if __name__ == "__main__":
    unittest.main()
