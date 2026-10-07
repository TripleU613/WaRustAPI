#!/usr/bin/env python3
"""Bounded offline ARM64 hello/decoder artifact checks; no account runtime."""

import argparse
import subprocess
import sys
from pathlib import Path


def run_fixture(root, check, max_instructions, host_meminfo=False):
    """Worker entry: parent process enforces wall time independently of Qiling."""
    from qiling import Qiling
    from qiling.const import QL_INTERCEPT, QL_VERBOSE
    from qiling.os.posix.syscall.mman import ql_syscall_munmap
    from qiling.os.posix.syscall.stat import (
        Stat, pack_stat_struct, ql_syscall_newfstatat,
    )

    executable = "arm64_android_hello" if check == "hello" else "decoder_probe"
    guest_args = [str(root / "bin" / executable)]
    if check == "decoder-first-entry":
        guest_args.append("--first-entry")
    ql = Qiling(
        guest_args,
        str(root),
        env={},
        verbose=QL_VERBOSE.OFF,
        multithread=True,
    )

    if host_meminfo:
        # Explicit local diagnostic: actual host memory metadata, not an
        # invented Android property or an input to any WhatsApp account flow.
        ql.add_fs_mapper("/proc/meminfo", "/proc/meminfo")

    def proc_exe_stat(guest, dirfd, path, buf, flags):
        # Qiling 1.4.6 newfstatat misses its /proc/self/exe mapping. Resolve
        # only the follow-symlink case to real fixture metadata, not fake data.
        if flags == 0 and guest.os.utils.read_cstring(path) == "/proc/self/exe":
            guest.mem.write(buf, pack_stat_struct(guest, Stat(guest.path)))
            return 0
        return ql_syscall_newfstatat(guest, dirfd, path, buf, flags)

    def munmap_checked(guest, addr, length):
        # Linux munmap permits ranges containing no mapped pages. Preserve the
        # original mapped-file handler; reject invalid alignment/zero length.
        if addr % guest.mem.pagesize or length == 0:
            return -22
        end = guest.mem.align_up(addr + length)
        if not any(addr < upper and end > lower for lower, upper, *_ in guest.mem.map_info):
            return 0
        return ql_syscall_munmap(guest, addr, length)

    instructions = 0

    def check_block(guest, address, size):
        nonlocal instructions
        # ARM64 has fixed four-byte instructions. Charge a whole block before
        # execution, conservatively; the count is shared across guest threads.
        instructions += (size + 3) // 4
        if instructions > max_instructions:
            raise RuntimeError("guest instruction budget exhausted")

    ql.hook_block(check_block)
    exits = []

    def record_exit(guest, code):
        exits.append(code)

    ql.os.set_syscall("munmap", munmap_checked)
    ql.os.set_syscall("newfstatat", proc_exe_stat)
    ql.os.set_syscall("exit_group", record_exit, QL_INTERCEPT.ENTER)
    # Thread.py runs fixed 31337-instruction slices and ignores Qiling.run's
    # timeout/count. The block hook and parent subprocess timeout enforce ours.
    ql.run()
    print(f"guest exit observed: {exits}; charged ARM64 instructions: {instructions}")
    if not exits or exits[-1] != 0:
        raise SystemExit("loader check failed: no successful guest exit_group")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rootfs", required=True, type=Path)
    parser.add_argument(
        "--check", choices=("hello", "decoder-init", "decoder-first-entry"),
        default="hello",
    )
    parser.add_argument(
        "--host-meminfo", action="store_true",
        help="map real host /proc/meminfo for this local artifact check",
    )
    parser.add_argument("--timeout-seconds", type=int, default=10)
    parser.add_argument("--max-instructions", type=int, default=1_000_000)
    args = parser.parse_args()
    if not 1 <= args.timeout_seconds <= 60:
        parser.error("--timeout-seconds must be between 1 and 60")
    if not 1 <= args.max_instructions <= 1_000_000_000:
        parser.error("--max-instructions must be between 1 and 1000000000")
    root = args.rootfs.resolve()
    required = [
        "bin/arm64_android_hello" if args.check == "hello" else "bin/decoder_probe",
        "system/bin/linker64", "system/lib64/libc.so", "system/lib64/libdl.so",
    ]
    if args.check != "hello":
        required += [
            "system/lib64/libsuperpack.so", "system/lib64/libandroid.so",
            "system/lib64/liblog.so",
        ]
    if args.check == "decoder-first-entry":
        required.append("system/lib64/wa_archive.so")
        if not (root / "decoded").is_dir() or any((root / "decoded").iterdir()):
            parser.error("decoder-first-entry requires an empty rootfs/decoded directory")
    for name in required:
        if not (root / name).is_file():
            parser.error(f"missing loader fixture: {root / name}")
    try:
        import importlib.util
        if importlib.util.find_spec("qiling") is None:
            parser.error("Qiling is missing; see tools/apk-research/README.md for setup")
        result = subprocess.run(
            [sys.executable, "-c",
             "import sys; from pathlib import Path; from native_loader_check import run_fixture; "
             "run_fixture(Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), sys.argv[4] == '1')",
             str(root), args.check, str(args.max_instructions), "1" if args.host_meminfo else "0"],
            cwd=Path(__file__).resolve().parent,
            timeout=args.timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired:
        parser.exit(1, "loader check failed: wall-clock budget exhausted\n")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
