#!/usr/bin/env python3
"""Reproduce selected pinned DEX helpers, excerpts and schema references."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
METHODS = [('normalize-address.txt', 'LX/0YG;', 'A02'), ('address-phone.txt', 'LX/0YG;', 'A01'), ('missing-mapping.txt', 'LX/0YG;', 'A00'), ('special-phone.txt', 'LX/0x9;', 'A03'), ('special-user.txt', 'LX/0x9;', 'A0A'), ('special-conversion.txt', 'LX/0Dn;', 'A08'), ('stored-mapping.txt', 'LX/0Ws;', 'A0E'), ('address-build.txt', 'LX/F3k;', 'A06'), ('address-type.txt', 'LX/EHs;', 'A00'), ('special-phone-kind.txt', 'LX/0x9;', 'A06'), ('type-A0Z.txt', 'LX/0Dn;', 'A0Z'), ('type-A0W.txt', 'LX/0Dn;', 'A0W'), ('type-A0O.txt', 'LX/0Dn;', 'A0O')]
EXCERPTS = [('delete-trigger-i-excerpt.txt', 'LX/Fei;', 'run', 121131, '41dd3e3edef7acbc2e190433f40547df2d99d0bea48c9d3fbb51bf1bfea320fb', [(0, 14), (6960, 7004)]), ('delete-trigger-q-excerpt.txt', 'LX/Feq;', 'run', 132904, 'b6dd55a4fc8b80ede9cdd8804ad7d92d8878b6a6435049ac536584407b5ae5d7', [(0, 14), (5426, 5474)])]

def excerpt(data, size, digest, ranges):
    if len(data) != size or hashlib.sha256(data).hexdigest() != digest:
        raise SystemExit("complete method differs from pinned export")
    lines = [f"# Bounded excerpt; full source bytes={size}, sha256={digest}",
             "# Other instructions and exception tables omitted. Not a complete method."]
    for line in data.decode().splitlines():
        pc = re.match(r"    ([0-9a-f]+)\.\.", line)
        if pc and any(lo <= int(pc[1],16) < hi for lo,hi in ranges):
            lines.append(line)
        elif line.startswith(("# class:", "# extends:", ".method", "    # full signature:")):
            lines.append(line)
    return ("\n".join(lines)+"\n").encode()

def run(repo, apk, command, args):
    return subprocess.run([str(repo / "target/debug/xtask"), command, str(apk), *args], check=True, capture_output=True).stdout

def check(bundle, name, data):
    if data != (bundle/name).read_bytes():
        raise SystemExit(f"retained export differs: {name}")
    print(f"matched {name}")

if __name__ == "__main__":
    apk = Path(sys.argv[1]).resolve()
    if hashlib.sha256(apk.read_bytes()).hexdigest() != APK_HASH:
        raise SystemExit("APK differs from pinned reference")
    bundle = Path(__file__).resolve().parent
    repo = bundle.parents[3]
    for name, owner, method in METHODS:
        check(bundle, name, run(repo, apk, "apk-smali", ["--class",owner,"--method",method,"--signatures","--exceptions"]))
    for name, owner, method, size, digest, ranges in EXCERPTS:
        data = run(repo, apk, "apk-smali", ["--class",owner,"--method",method,"--signatures","--exceptions"])
        check(bundle,name,excerpt(data,size,digest,ranges))
    check(bundle,"delete-callers.txt",run(repo,apk,"apk-callers",["--class","LX/0WJ;","--method","A0r"]))
    probe = subprocess.run([str(repo / "target/debug/xtask"), "apk-callers", str(apk), "--class", "LX/F2A;", "--method", "A0r"], check=True, capture_output=True)
    check(bundle,"delete-producer-probe.json",(json.dumps({"owner":"LX/F2A;","method":"A0r","stdout_utf8":probe.stdout.decode(),"stderr_utf8":probe.stderr.decode()},indent=2)+"\n").encode())
