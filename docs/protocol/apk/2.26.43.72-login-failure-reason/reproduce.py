#!/usr/bin/env python3
"""Regenerate the selected pinned initial login failure reason methods without running Android."""
import hashlib
from pathlib import Path
import subprocess
import sys

APK_SHA256 = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
apk = Path(sys.argv[1]).resolve()
if hashlib.sha256(apk.read_bytes()).hexdigest() != APK_SHA256:
    raise SystemExit("APK hash differs from pinned reference")
bundle = Path(__file__).resolve().parent
repo = bundle.parents[3]
selectors = [('required-integer.txt', 'apk-smali', 'LX/0Tp;', 'A04'),
 ('integer-parser.txt', 'apk-smali', 'LX/0Tp;', 'A06'),
 ('required-text.txt', 'apk-smali', 'LX/0Tp;', 'A0L'),
 ('../2.26.43.72-login-success-reader/login-reader.txt', 'apk-smali', 'LX/0ZJ;', 'A13')]
for filename, command, owner, method in selectors:
    args = [str(repo / "target/debug/xtask"), command, str(apk), "--class", owner]
    if method is not None:
        args += ["--method", method]
    if command == "apk-smali":
        args += ["--signatures", "--exceptions"]
    result = subprocess.run(args, check=True, capture_output=True)
    if result.stdout != (bundle / filename).read_bytes():
        raise SystemExit(f"retained output differs: {filename}")
    print(f"matched {filename}")