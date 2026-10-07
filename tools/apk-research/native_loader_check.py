#!/usr/bin/env python3
"""Bounded offline check of the public Qiling ARM64 Android hello fixture."""

import argparse
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rootfs", required=True, type=Path)
    args = parser.parse_args()
    root = args.rootfs.resolve()
    for name in (
        "bin/arm64_android_hello",
        "system/bin/linker64",
        "system/lib64/libc.so",
        "system/lib64/libdl.so",
    ):
        if not (root / name).is_file():
            parser.error(f"missing public loader fixture: {root / name}")
    try:
        from qiling import Qiling
        from qiling.const import QL_INTERCEPT, QL_VERBOSE
        from qiling.os.posix.syscall.stat import (
            Stat,
            pack_stat_struct,
            ql_syscall_newfstatat,
        )
    except ImportError:
        parser.error("Qiling is missing; see tools/apk-research/README.md for setup")

    ql = Qiling(
        [str(root / "bin/arm64_android_hello")],
        str(root),
        env={},
        verbose=QL_VERBOSE.OFF,
        multithread=True,
    )

    def proc_exe_stat(guest, dirfd, path, buf, flags):
        # Qiling 1.4.6 newfstatat misses its /proc/self/exe mapping. Resolve
        # only the follow-symlink case to real fixture metadata, not fake data.
        if flags == 0 and guest.os.utils.read_cstring(path) == "/proc/self/exe":
            guest.mem.write(buf, pack_stat_struct(guest, Stat(guest.path)))
            return 0
        return ql_syscall_newfstatat(guest, dirfd, path, buf, flags)

    exits = []

    def record_exit(guest, code):
        exits.append(code)

    ql.os.set_syscall("newfstatat", proc_exe_stat)
    ql.os.set_syscall("exit_group", record_exit, QL_INTERCEPT.ENTER)
    ql.run(timeout=10_000_000, count=1_000_000)
    print(f"guest exit observed: {exits}")
    if not exits or exits[-1] != 0:
        raise SystemExit("loader check failed: no successful guest exit_group")


if __name__ == "__main__":
    main()
