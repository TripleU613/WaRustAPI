#!/usr/bin/env python3
"""Reproduce selected pinned DEX helpers, excerpts and schema references."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
METHODS = [('status-input.txt', 'LX/Ens;', 'A01'), ('status-state-batch.txt', 'LX/Ens;', 'A02'), ('delete-executor.txt', 'LX/0YM;', 'A02'), ('default-address.txt', 'LX/F3k;', 'A04'), ('rekey-message-key.txt', 'LX/DBb;', 'A0g'), ('runnable-factory.txt', 'LX/Fei;', 'A00'), ('runnable-submit-helper.txt', 'LX/Fei;', 'A01'), ('other-selector-euf.txt', 'LX/Euf;', 'A02'), ('other-selector-eu4.txt', 'LX/Eu4;', 'A01')]
EXCERPTS = [('voip-receipt-excerpt.txt', 'LX/Feo;', 'run', 214697, '104e191915998d307029e4ef785804351506993c85c4cc95d89d726072e1063e', [(0, 14), (3820, 3888), (4324, 4612)]), ('other-selector-feA-excerpt.txt', 'LX/FeA;', 'run', 32141, '0cf58c661b50fab8449a0c8035b85999f9b09a7916dbc6c3bfb2055f0dc67b71', [(2236, 2264)]), ('other-selector-fuy-excerpt.txt', 'LX/FUY;', 'CK4', 23582, 'f54500fc559364c4b6debd30af1d2dc421e26545a5559a4ad280e4614172770e', [(1584, 1616)])]

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
    check(bundle,'factory-callers.txt',run(repo,apk,"apk-callers",["--class","LX/Fei;","--method",'A00']))
    check(bundle,'submit-helper-callers.txt',run(repo,apk,"apk-callers",["--class","LX/Fei;","--method",'A01']))
    check(bundle,'constructor-callers.txt',run(repo,apk,"apk-callers",["--class","LX/Fei;","--method",'<init>']))
    check(bundle,'runnable-constructor.txt',run(repo,apk,"apk-smali",["--class",'LX/Fei;',"--method","<init>","--signatures","--exceptions"]))
    check(bundle,'status-runnable-constructor.txt',run(repo,apk,"apk-smali",["--class",'LX/Feq;',"--method","<init>","--signatures","--exceptions"]))
    probe = run(repo,apk,"apk-smali",["--class","LX/F2A;","--signatures","--exceptions"])
    check(bundle,"f2a-construction-probe.json",(json.dumps({"source_bytes":len(probe),"source_sha256":hashlib.sha256(probe).hexdigest(),"lines_containing_LX_Fei":[line for line in probe.decode().splitlines() if "LX/Fei;" in line]},indent=2)+"\n").encode())
