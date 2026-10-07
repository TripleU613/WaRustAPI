"""Integration checks; set TELEWHATSAPP_LOADER_ROOTFS to the public rootfs."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


HELPER = Path(__file__).with_name("native_loader_check.py").resolve()


class LoaderTests(unittest.TestCase):
    def invoke(self, root, *args):
        return subprocess.run(
            [sys.executable, str(HELPER), "--rootfs", str(root), *args],
            capture_output=True,
            text=True,
            timeout=5,
        )

    def public_root(self):
        configured = os.environ.get("TELEWHATSAPP_LOADER_ROOTFS")
        if not configured:
            self.skipTest("public ARM64 rootfs not configured")
        return Path(configured).resolve()

    def test_missing_fixture_refuses_before_guest_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            result = self.invoke(directory)
        self.assertEqual(result.returncode, 2)
        self.assertIn("missing loader fixture", result.stderr)
        self.assertNotIn("guest exit observed", result.stdout)

    def test_actual_hello_reaches_successful_guest_exit(self):
        result = self.invoke(self.public_root())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Hello, World!", result.stdout)
        self.assertIn("guest exit observed: [0]", result.stdout)

    def test_archive_checks_preserve_partial_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "decoded").mkdir()
            partial = root / "decoded/partial.so"
            partial.write_bytes(b"preserved partial artifact")
            for check in ("decoder-first-entry", "decoder-full-archive"):
                result = self.invoke(root, "--check", check)
                self.assertEqual(result.returncode, 2)
                self.assertIn("requires an empty", result.stderr)
                self.assertEqual(partial.read_bytes(), b"preserved partial artifact")

    def test_trace_refuses_unknown_decoder_before_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "system/lib64").mkdir(parents=True)
            (root / "system/lib64/libsuperpack.so").write_bytes(b"wrong build")
            result = self.invoke(root, "--check", "decoder-full-archive", "--trace-worker")
        self.assertEqual(result.returncode, 2)
        self.assertIn("requires the pinned 37.71", result.stderr)
        self.assertNotIn("worker branch=", result.stdout)

    def test_instruction_guard_stops_actual_threaded_guest(self):
        result = self.invoke(self.public_root(), "--max-instructions", "1")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("guest instruction budget exhausted", result.stderr)
        self.assertNotIn("guest exit observed: [0]", result.stdout)

    def test_wall_guard_kills_infinite_guest(self):
        root = self.public_root()
        if not shutil.which("clang") or not shutil.which("ld.lld"):
            self.skipTest("ARM64 cross compiler not available")
        # A real infinite ARM64 guest detects the threaded-runner timeout gap;
        # this does not mock Qiling or substitute a host sleep for execution.
        source = r'''
extern void __libc_init(void *, void *, int (*)(int,char **,char **), const void *);
static void (*ctors[])(void)={(void (*)(void))-1,0};
int main(int argc,char **argv,char **env) {
    for (;;) __asm__ volatile("nop");
}
void start_c(void *raw) {
    const void *structors[]={ctors,ctors,ctors};
    __libc_init(raw,0,main,structors);
}
__asm__(".global _start\n_start:\nmov x0, sp\nb start_c\n");
'''
        with tempfile.TemporaryDirectory() as directory:
            fixture = Path(directory)
            for name in ("system/bin/linker64", "system/lib64/libc.so", "system/lib64/libdl.so"):
                output = fixture / name
                output.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(root / name, output)
            (fixture / "bin").mkdir()
            source_path = fixture / "loop.c"
            source_path.write_text(source)
            subprocess.run(
                ["clang", "--target=aarch64-linux-android21", "-fuse-ld=lld",
                 "-nostdlib", "-fPIE", "-pie", "-Wl,-e,_start",
                 "-Wl,--dynamic-linker=/system/bin/linker64", "-Wl,-z,max-page-size=4096",
                 f"-L{fixture / 'system/lib64'}", str(source_path),
                 "-l:libc.so", "-l:libdl.so", "-o", str(fixture / "bin/arm64_android_hello")],
                check=True, capture_output=True, timeout=5,
            )
            result = self.invoke(fixture, "--timeout-seconds", "1", "--max-instructions", "1000000000")
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("wall-clock budget exhausted", result.stderr)
        self.assertNotIn("guest exit observed: [0]", result.stdout)


if __name__ == "__main__":
    unittest.main()
