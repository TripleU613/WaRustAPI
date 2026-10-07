#!/usr/bin/env python3
"""Bounded offline ARM64 hello/decoder artifact checks; no account runtime."""

import argparse
import hashlib
import subprocess
import sys
from pathlib import Path


def run_fixture(root, check, max_instructions, host_meminfo=False, trace_worker=False):
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
    elif check == "decoder-full-archive":
        guest_args.append("--full-archive")
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
    trace_count = 0
    progress_step = 0
    trace_points = {0x1EF00: "size-check", 0x1EF40: "size-difference",
                    0x1EF60: "hash-difference", 0x1EFA4: "oversized-file",
                    0x1F044: "failure-log"}
    trace_low_offsets = frozenset(offset & 0xFFF for offset in trace_points)

    def trace_branch(guest, address):
        nonlocal trace_count
        # ARM64 37.71 worker branch targets, re-anchored from the native ELF;
        # trace mode checks its hash before interpreting these offsets.
        maps = [row for row in guest.mem.map_info if "libsuperpack.so" in row[3]]
        if not maps:
            return
        offset = address - min(row[0] for row in maps)
        if offset not in trace_points or trace_count >= 64:
            return
        task = guest.arch.regs.read("x26")

        def field(at, width):
            return int.from_bytes(guest.mem.read(task + at, width), "little")

        # 0x1e544..0x1e564 builds this job; 0x1ef00 compares +0x20,
        # whereas 0x1f04c prints +0x18 as the declared expected size.
        requested = field(0x20, 8)
        produced = guest.arch.regs.read("x27")
        if offset == 0x1EF00 and requested == produced:
            return
        trace_count += 1
        print(
            f"worker branch={trace_points[offset]} offset={offset:#x} "
            f"stream={field(8, 4)} type={field(12, 4)} "
            f"declared={field(0x18, 8)} requested={requested} produced={produced} "
            f"hash_check={field(0x50, 4)}",
            flush=True,
        )

    def check_block(guest, address, size):
        nonlocal instructions, progress_step
        # ARM64 has fixed four-byte instructions. Charge a whole block before
        # execution, conservatively; the count is shared across guest threads.
        instructions += (size + 3) // 4
        if instructions > max_instructions:
            raise RuntimeError("guest instruction budget exhausted")
        if trace_worker:
            if address & 0xFFF in trace_low_offsets:
                trace_branch(guest, address)
            step = instructions // 100_000_000
            if step > progress_step:
                progress_step = step
                print(f"worker progress charged_instructions={instructions}", flush=True)

    ql.hook_block(check_block)
    exits = []

    def record_exit(guest, code):
        exits.append(code)

    ql.os.set_syscall("munmap", munmap_checked)
    ql.os.set_syscall("newfstatat", proc_exe_stat)
    ql.os.set_syscall("exit_group", record_exit, QL_INTERCEPT.ENTER)
    if trace_worker:
        futex_count = 0
        futex_selectors = {}
        previous_futex = None

        def trace_futex(guest, uaddr, op, val, timeout, uaddr2, val3):
            nonlocal futex_count, previous_futex
            if futex_count >= 64:
                return
            selector = futex_selectors.setdefault(uaddr, len(futex_selectors) + 1)
            manager = guest.os.thread_management
            thread = manager.cur_thread.id if manager else None
            state = (thread, selector, op, val, bool(timeout))
            if state == previous_futex:
                return
            previous_futex = state
            futex_count += 1
            print(f"worker futex thread={thread} selector={selector} op={op} "
                  f"val={val} timeout_present={bool(timeout)}", flush=True)

        # Observe only; retain Qiling's syscall and scheduler behavior.
        ql.os.set_syscall("futex", trace_futex, QL_INTERCEPT.ENTER)
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
        "--check", choices=("hello", "decoder-init", "decoder-first-entry", "decoder-full-archive"),
        default="hello",
    )
    parser.add_argument(
        "--host-meminfo", action="store_true",
        help="map real host /proc/meminfo for this local artifact check",
    )
    parser.add_argument("--timeout-seconds", type=int, default=10)
    parser.add_argument("--max-instructions", type=int, default=1_000_000)
    parser.add_argument("--trace-worker", action="store_true", help="trace known 37.71 native worker failure branches, at most 64 records")
    args = parser.parse_args()
    if not 1 <= args.timeout_seconds <= 600:
        parser.error("--timeout-seconds must be between 1 and 600")
    if not 1 <= args.max_instructions <= 10_000_000_000:
        parser.error("--max-instructions must be between 1 and 10000000000")
    root = args.rootfs.resolve()
    if args.trace_worker:
        if args.check not in ("decoder-first-entry", "decoder-full-archive"):
            parser.error("--trace-worker requires an archive reconstruction check")
        decoder = root / "system/lib64/libsuperpack.so"
        # APK 2.26.37.71 lib/arm64-v8a/libsuperpack.so; no other build's
        # register layout is established by these worker offset traces.
        expected = "fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f"
        if not decoder.is_file() or hashlib.sha256(decoder.read_bytes()).hexdigest() != expected:
            parser.error("--trace-worker requires the pinned 37.71 ARM64 decoder")
    required = [
        "bin/arm64_android_hello" if args.check == "hello" else "bin/decoder_probe",
        "system/bin/linker64", "system/lib64/libc.so", "system/lib64/libdl.so",
    ]
    if args.check != "hello":
        required += [
            "system/lib64/libsuperpack.so", "system/lib64/libandroid.so",
            "system/lib64/liblog.so",
        ]
    if args.check in ("decoder-first-entry", "decoder-full-archive"):
        required.append("system/lib64/wa_archive.so")
        if not (root / "decoded").is_dir() or any((root / "decoded").iterdir()):
            parser.error(f"{args.check} requires an empty rootfs/decoded directory")
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
             "run_fixture(Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), "
             "sys.argv[4] == '1', sys.argv[5] == '1')",
             str(root), args.check, str(args.max_instructions),
             "1" if args.host_meminfo else "0", "1" if args.trace_worker else "0"],
            cwd=Path(__file__).resolve().parent,
            timeout=args.timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired:
        parser.exit(1, "loader check failed: wall-clock budget exhausted\n")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
