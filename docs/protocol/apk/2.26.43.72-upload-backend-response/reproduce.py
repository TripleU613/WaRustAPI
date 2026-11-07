#!/usr/bin/env python3
"""Reproduce selected upload backend evidence without running Android."""
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
    ("base-constructor-callers.txt", ["apk-callers", "--class", "LX/K2k;", "--method", "<init>"]),
    ("urlconnection-factory.txt", ["apk-smali", "--class", "LX/12F;", "--method", "A03", "--signatures", "--exceptions"]),
    ("urlconnection-open.txt", ["apk-smali", "--class", "LX/0jV;", "--method", "A03", "--signatures", "--exceptions"]),
    ("tigon-builder-constructors.txt", ["apk-smali", "--class", "Lcom/facebook/tigon/iface/TigonRequestBuilder;", "--method", "<init>", "--signatures", "--exceptions"]),
    ("tigon-status-writers.txt", ["apk-field-refs", "--class", "LX/1gK;", "--field", "A00", "--type", "I", "--writes-only"]),
    ("../../rea/2.26.43.72-media-upload-http/urlconnection-execute.txt", ["apk-smali", "--class", "LX/JAS;", "--method", "A04", "--signatures", "--exceptions"]),
    ("../../rea/2.26.43.72-media-upload-http/tigon-execute.txt", ["apk-smali", "--class", "LX/JAT;", "--method", "A04", "--signatures", "--exceptions"]),
]
for filename, args in selectors:
    result = subprocess.run([str(repo / "target/debug/xtask"), args[0], str(apk), *args[1:]], check=True, capture_output=True)
    if result.stdout != (bundle / filename).read_bytes():
        raise SystemExit(f"retained export differs: {filename}")
    print(f"matched {filename}")
