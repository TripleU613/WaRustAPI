"""Preflight and optional pinned ELF descriptor integration checks."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_schema_check.py").resolve()


class SchemaChecks(unittest.TestCase):
    def invoke(self, library):
        return subprocess.run(
            [sys.executable, str(HELPER), "--library", str(library)],
            capture_output=True, text=True, timeout=10,
        )

    def test_missing_library_is_named(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(Path(directory) / "missing.so")
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing reconstructed libwhatsappmerged.so", result.stderr)

    def test_wrong_build_refuses_without_partial_schema(self):
        with tempfile.TemporaryDirectory() as directory:
            library = Path(directory) / "other.so"
            library.write_bytes(b"unknown artifact")
            result = self.invoke(library)
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires the pinned 37.71", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_pinned_native_descriptors_match_static_fields(self):
        configured = os.environ.get("TELEWHATSAPP_SCHEMA_LIBRARY")
        if not configured:
            self.skipTest("set TELEWHATSAPP_SCHEMA_LIBRARY for actual ELF extraction")
        result = self.invoke(Path(configured).resolve())
        self.assertEqual(result.returncode, 0, result.stderr)
        schema = json.loads(result.stdout)
        messages = {row["name"]: row for row in schema["messages"]}
        fields = messages["whatsapp.FingerprintData"]["fields"]
        self.assertEqual([(row["name"], row["tag"]) for row in fields], [
            ("public_key", 1), ("pn_identifier", 2), ("lid_identifier", 3),
            ("username_identifier", 4), ("hosted_state", 5), ("hashed_public_key", 6),
        ])
        self.assertEqual(messages["whatsapp.CombinedFingerprint"]["size"], 48)
        self.assertEqual(schema["enums"]["whatsapp.HostedState"]["values"], [
            {"name": "E2EE", "value": 0}, {"name": "HOSTED", "value": 1},
        ])


if __name__ == "__main__":
    unittest.main()
