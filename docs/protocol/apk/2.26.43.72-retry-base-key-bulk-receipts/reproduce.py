#!/usr/bin/env python3
"""Reproduce selected pinned DEX helpers, excerpts and schema references."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
METHODS = [('bulk-map-producer.txt', 'LX/F21;', 'A00'), ('received-target-predicate.txt', 'LX/1e4;', 'A03'), ('target-normalization.txt', 'LX/EZd;', 'A00'), ('pending-prekey-confirmation.txt', 'LX/F21;', 'A03'), ('ownership-predicate.txt', 'LX/1dX;', 'A0H'), ('pending-prekey-clear.txt', 'LX/F1Y;', 'A06'), ('target-record.txt', 'LX/Em7;', '<init>'), ('bulk-runnable-constructor.txt', 'LX/Fc1;', '<init>')]
EXCERPTS = []

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
    check(bundle,'bulk-constructor-callers.txt',run(repo,apk,"apk-callers",["--class",'LX/Fc1;',"--method",'<init>']))
    check(bundle,'map-producer-callers.txt',run(repo,apk,"apk-callers",["--class",'LX/F21;',"--method",'A00']))
    check(bundle,'receipt-dispatch-callers.txt',run(repo,apk,"apk-callers",["--class",'LX/F21;',"--method",'A05']))
    dispatch = run(repo,apk,"apk-smali",["--class","LX/F21;","--method","A05","--signatures","--exceptions"])
    if dispatch != (repo/"docs/protocol/apk/2.26.43.72-receipt-null-fact-actor/legacy-apply.txt").read_bytes():
        raise SystemExit("previous complete receipt dispatch differs")
    print("matched previously retained complete F21.A05")
    probe = run(repo,apk,"apk-smali",["--class","LX/0WJ;","--signatures","--exceptions"])
    check(bundle,"coordinator-construction-probe.json",(json.dumps({"source_bytes":len(probe),"source_sha256":hashlib.sha256(probe).hexdigest(),"lines_containing_LX_Fc1":[line for line in probe.decode().splitlines() if "LX/Fc1;" in line]},indent=2)+"\n").encode())
