"""Preflight and optional pinned ARM64 fingerprint control-flow checks."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_fingerprint_check.py").resolve()


class FingerprintChecks(unittest.TestCase):
    def invoke(self, library, check="hash"):
        return subprocess.run(
            [sys.executable, str(HELPER), "--library", str(library), "--check", check],
            capture_output=True, text=True, timeout=30,
        )

    def test_missing_library_is_named(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(Path(directory) / "missing.so")
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing reconstructed native library", result.stderr)

    def test_wrong_build_refused_before_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / "other.so"
            library.write_bytes(b"unrecognized artifact")
            result = self.invoke(library)
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires the pinned 37.71", result.stderr)
        self.assertNotIn("synthetic case", result.stdout)

    def test_wrong_build_byte_check_refused_before_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / "other.so"
            library.write_bytes(b"unrecognized artifact")
            result = self.invoke(library, "bytes")
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires the pinned 37.71", result.stderr)
        self.assertNotIn("synthetic byte cases", result.stdout)

    def test_actual_pinned_byte_comparison(self):
        configured = os.environ.get("WARUSTAPI_FINGERPRINT_LIBRARY")
        if not configured:
            self.skipTest("set WARUSTAPI_FINGERPRINT_LIBRARY for native slice execution")
        result = self.invoke(Path(configured).resolve(), "bytes")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("actual ARM64 XOR/OR results agree", result.stdout)
        self.assertIn("equal-length instruction counts agree", result.stdout)
        self.assertIn("No modeled callees", result.stdout)

    def test_actual_pinned_hash_control_flow(self):
        configured = os.environ.get("WARUSTAPI_FINGERPRINT_LIBRARY")
        if not configured:
            self.skipTest("set WARUSTAPI_FINGERPRINT_LIBRARY for native slice execution")
        result = self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.count("rounds=5200 updates=10400"), 2)
        self.assertIn("native control flow only", result.stdout)


if __name__ == "__main__":
    unittest.main()
