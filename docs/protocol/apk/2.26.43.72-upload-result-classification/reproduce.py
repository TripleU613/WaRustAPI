#!/usr/bin/env python3
"""Reproduce selected pinned upload result exports without executing Android."""
import hashlib
from pathlib import Path
import subprocess
import sys

apk = Path(sys.argv[1]).resolve()
if hashlib.sha256(apk.read_bytes()).hexdigest() != "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e":
    raise SystemExit("APK differs from pinned reference")
bundle = Path(__file__).resolve().parent
repo = bundle.parents[3]
selectors = [
    ("transfer-result-gate.txt", ["apk-smali", "--class", "LX/MtZ;", "--method", "Cb6", "--signatures", "--exceptions"]),
    ("result-type.txt", ["apk-smali", "--class", "LX/Lyh;", "--signatures", "--exceptions"]),
    ("result-status-references.txt", ["apk-field-refs", "--class", "LX/Lyh;", "--field", "A06", "--type", "I"]),
    ("upload-runner.txt", ["apk-smali", "--class", "LX/MvO;", "--method", "ARB", "--signatures", "--exceptions"]),
    ("code-bridge.txt", ["apk-smali", "--class", "LX/K2k;", "--method", "A03", "--signatures", "--exceptions"]),
    ("abstract-executor.txt", ["apk-smali", "--class", "LX/K2k;", "--method", "A04", "--signatures", "--exceptions"]),
]
for filename, args in selectors:
    result = subprocess.run([str(repo / "target/debug/xtask"), args[0], str(apk), *args[1:]], check=True, capture_output=True)
    if result.stdout != (bundle / filename).read_bytes():
        raise SystemExit(f"retained export differs: {filename}")
    print(f"matched {filename}")
