#!/usr/bin/env python3
"""Reproduce pinned dirty worker dispatch without executing Android."""
import hashlib
from pathlib import Path
import re
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
SOURCE_HASH = "1d435d87f0d71f1cfce2fe7ca08d6c8eb27abfecc7fd75d226b965e4954e251b"
SOURCE_BYTES = 315835

def excerpt(data):
    if len(data) != SOURCE_BYTES or hashlib.sha256(data).hexdigest() != SOURCE_HASH:
        raise SystemExit("complete handler differs from pinned export")
    result = [f"# Bounded dirty collection/scheduling excerpt; full source bytes={SOURCE_BYTES}, sha256={SOURCE_HASH}",
              "# Other instructions and exception tables omitted; this is not a complete method."]
    for line in data.decode().splitlines():
        pc = re.match(r"    ([0-9a-f]+)\.\.[0-9a-f]+  ", line)
        if pc:
            value = int(pc.group(1), 16)
            if 0x2868 <= value < 0x28e4 or 0x5780 <= value < 0x57aa:
                result.append(line)
        elif line.startswith(("# class:", "# extends:", ".method", "    # full signature:")):
            result.append(line)
    return ("\n".join(result) + "\n").encode()

if __name__ == "__main__":
    apk = Path(sys.argv[1]).resolve()
    if hashlib.sha256(apk.read_bytes()).hexdigest() != APK_HASH:
        raise SystemExit("APK differs from pinned reference")
    bundle = Path(__file__).resolve().parent
    repo = bundle.parents[3]
    selectors = [
        ("job.txt", "Lcom/whatsapp/group/batch/FetchTruncatedGroupsJob;", None),
        ("group-parser.txt", "LX/1bd;", "A01"),
        ("mex-dispatch.txt", "LX/7uj;", "A00"),
        ("context-enum.txt", "LX/7e9;", "<clinit>"),
        ("jid-wire-text.txt", "LX/2LD;", "A0e"),
        ("stanza-operation.txt", "LX/8UL;", None),
        ("notification-scheduling-excerpt.txt", "LX/0w9;", "A09"),
    ]
    for filename, owner, method in selectors:
        args = [str(repo / "target/debug/xtask"), "apk-smali", str(apk), "--class", owner, "--signatures", "--exceptions"]
        if method is not None:
            args += ["--method", method]
        result = subprocess.run(args, check=True, capture_output=True).stdout
        if filename == "notification-scheduling-excerpt.txt":
            result = excerpt(result)
        if result != (bundle / filename).read_bytes():
            raise SystemExit(f"retained export differs: {filename}")
        print(f"matched {filename}")
