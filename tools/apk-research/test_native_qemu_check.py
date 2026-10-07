"""Preflight regressions for the isolated local decoder runner."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_qemu_check.py").resolve()


class QemuPreflightTests(unittest.TestCase):
    def test_missing_input_is_named(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                [sys.executable, str(HELPER), "--rootfs", directory],
                capture_output=True, text=True, timeout=5,
            )
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing decoder fixture", result.stderr)
        self.assertIn("bin/decoder_probe", result.stderr)

    def test_wrong_build_refuses_before_preparation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("bin/decoder_probe", "system/bin/linker64",
                         "system/lib64/libc.so", "system/lib64/libdl.so",
                         "system/lib64/libsuperpack.so", "system/lib64/libandroid.so",
                         "system/lib64/liblog.so"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"unrecognized artifact")
                path.chmod(0o600)
            result = subprocess.run(
                [sys.executable, str(HELPER), "--rootfs", str(root)],
                capture_output=True, text=True, timeout=5,
            )
            self.assertEqual((root / "bin/decoder_probe").stat().st_mode & 0o777, 0o600)
            self.assertFalse((root / "qemu").exists())
        self.assertEqual(result.returncode, 2)
        self.assertIn("wrong 37.71 artifact hash", result.stderr)


if __name__ == "__main__":
    unittest.main()
