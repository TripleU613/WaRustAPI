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
    def invoke(self, library, extra=()):
        return subprocess.run(
            [sys.executable, str(HELPER), "--library", str(library), *extra],
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

    def test_native_string_options_require_pairing(self):
        result = self.invoke(Path("unused.so"), ["--string-library", "unused-merged.so"])
        self.assertEqual(result.returncode, 2)
        self.assertIn("must be supplied together", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_wrong_native_string_fixtures_refuse_before_emulation(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory) / "wrong.so"
            fixture.write_bytes(b"unrecognized string fixture")
            result = self.invoke(Path("unused.so"), ["--string-library", str(fixture), "--libc", str(fixture)])
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires pinned 37.71 merged string library", result.stderr)
        self.assertEqual(result.stdout, "")

    def test_actual_merged_and_bionic_string_comparison(self):
        library = os.environ.get("TELEWHATSAPP_FINGERPRINT_LIBRARY")
        merged = os.environ.get("TELEWHATSAPP_SCHEMA_LIBRARY")
        libc = os.environ.get("TELEWHATSAPP_BIONIC_LIBRARY")
        if not (library and merged and libc):
            self.skipTest("set main/merged/Bionic library fixture variables for actual string execution")
        result = self.invoke(Path(library).resolve(), ["--string-library", str(Path(merged).resolve()), "--libc", str(Path(libc).resolve())])
        self.assertEqual(result.returncode, 0, result.stderr)
        actual = json.loads(result.stdout)
        fixture_dir = HELPER.parents[1].parent / "docs/protocol"
        expected = json.loads((fixture_dir / "fingerprint-native-string-check.json").read_text())
        for field in ["apk_build", "library_sha256", "string_library_sha256", "bionic_fixture_sha256", "scope", "native_ascii_checks"]:
            self.assertEqual(actual[field], expected[field])
        self.assertEqual(len(actual["cases"]), expected["core_cases"])
        self.assertEqual(max(c.get("instructions", 0) for c in actual["cases"]), expected["core_max_instructions"])
        self.assertEqual(sum(c.get("native_string_comparisons", 0) for c in actual["cases"]), expected["native_string_comparisons_in_core_cases"])
        modeled = json.loads((fixture_dir / "fingerprint-native-comparison-cases.json").read_text())
        self.assertEqual([(c["case"], c.get("code"), c.get("remapped_code")) for c in actual["cases"]],
                         [(c["case"], c.get("code"), c.get("remapped_code")) for c in modeled["cases"]])


if __name__ == "__main__":
    unittest.main()
