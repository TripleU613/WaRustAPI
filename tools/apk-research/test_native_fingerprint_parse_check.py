"""Pinned actual protobuf-C unpack/free corpus and build-preflight checks."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_fingerprint_parse_check.py").resolve()


class FingerprintParseChecks(unittest.TestCase):
    def invoke(self, library):
        return subprocess.run(
            [sys.executable, str(HELPER), "--library", str(library)],
            capture_output=True, text=True, timeout=30,
        )

    def test_missing_library_refuses_without_partial_output(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(Path(directory) / "missing.so")
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing reconstructed libwhatsappmerged.so", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_wrong_build_refuses_before_emulation(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / "wrong.so"
            library.write_bytes(b"unrecognized native parser")
            result = self.invoke(library)
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires pinned 37.71", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_actual_native_unpack_and_free_match_committed_corpus(self):
        configured = os.environ.get("TELEWHATSAPP_SCHEMA_LIBRARY")
        if not configured:
            self.skipTest("set TELEWHATSAPP_SCHEMA_LIBRARY for actual native unpacking")
        result = self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode, 0, result.stderr)
        corpus = HELPER.parents[1].parent / "docs/protocol/fingerprint-native-parser-cases.json"
        actual = json.loads(result.stdout)
        self.assertEqual(actual, json.loads(corpus.read_text()))
        self.assertEqual(len(actual["cases"]), 50)
        self.assertTrue(all(case["instructions"] < 1000000 for case in actual["cases"]))
        self.assertTrue(all(case["modeled_allocated_bytes"] <= 1048576 for case in actual["cases"]))


if __name__ == "__main__":
    unittest.main()
