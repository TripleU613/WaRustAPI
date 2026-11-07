#!/usr/bin/env python3
"""Reproduce pinned group constructors without executing Android."""
import hashlib
from pathlib import Path
import subprocess
import sys

apk = Path(sys.argv[1]).resolve()
if hashlib.sha256(apk.read_bytes()).hexdigest() != "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e":
    raise SystemExit("APK differs from pinned reference")
bundle = Path(__file__).resolve().parent
repo = bundle.parents[3]
for filename, owner, method in [
    ("group-constructor.txt", "LX/1bc;", "<init>"),
    ("modern-user-predicate.txt", "LX/0nW;", "A02"),
    ("legacy-prefix-predicate.txt", "LX/0Dl;", "A02"),
    ("../2.26.43.72-message-ack-chat/jid-parser.txt", "LX/0Dd;", "A00"),
]:
    result = subprocess.run([str(repo / "target/debug/xtask"), "apk-smali", str(apk), "--class", owner, "--method", method, "--signatures", "--exceptions"], check=True, capture_output=True)
    if result.stdout != (bundle / filename).read_bytes():
        raise SystemExit(f"retained export differs: {filename}")
    print(f"matched {filename}")
