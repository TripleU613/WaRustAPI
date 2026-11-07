#!/usr/bin/env python3
"""Reproduce selected pinned DEX helpers, excerpts and schema references."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
METHODS = [('replace-helper.txt', 'LX/0KU;', 'A08'), ('delete-helper.txt', 'LX/0KU;', 'A04'), ('delete-message-store.txt', 'LX/EbJ;', 'A00'), ('delete-message-coordinator.txt', 'LX/0WJ;', 'A0r')]
EXCERPTS = [('fresh-schema-excerpt.txt', 'LX/28o;', 'onCreate', 13253, '5237d0a94cb0fc27671f7694b7cb2412f4ce75a5c8c9a4bc630735fffe713422', [(128, 148)]), ('upgrade-schema-excerpt.txt', 'LX/28o;', 'onUpgrade', 36475, '7205265812dd072f3bf805faadc7c84466694a2a000c91641ca6b6ad9f308c83', [(0, 62), (112, 132), (622, 682), (722, 752)]), ('bulk-delete-excerpt.txt', 'LX/Fc1;', 'run', 72278, 'af6c9f602be2972cb9e95cce8986242dc8b0285a7f852cb8c5fcbe2bbe1c3241', [(0, 14), (3068, 3458)])]

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
    strings = run(repo,apk,"apk-strings",["--contains","message_base_key"])
    check(bundle,"schema-strings.json",(json.dumps({"source_bytes":len(strings),"source_sha256":hashlib.sha256(strings).hexdigest(),"stdout_utf8":strings.decode()},indent=2)+"\n").encode())
    check(bundle,"table-xref.txt",run(repo,apk,"apk-xref",["--anchor","message_base_key"]))
    anchors = [s for s in strings.decode().splitlines() if s.startswith(("CREATE TABLE", "CREATE UNIQUE INDEX"))]
    args = [item for anchor in anchors for item in ("--anchor",anchor)]
    check(bundle,"schema-xref.txt",run(repo,apk,"apk-xref",args))
