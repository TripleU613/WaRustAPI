#!/usr/bin/env python3
"""Regenerate the selected pinned login reader methods without running Android."""
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
selectors = [
    ("login-reader.txt", "apk-smali", "LX/0ZJ;", "A13"),
    ("jid-getter.txt", "apk-smali", "LX/0Tp;", "A0A"),
    ("success-key-helper.txt", "apk-smali", "LX/0ZJ;", "A18"),
    ("location-helper.txt", "apk-smali", "LX/0ZJ;", "A0u"),
    ("success-handler-registration.txt", "apk-smali", "LX/1rA;", "A02"),
    ("empty-success-handler.txt", "apk-smali", "LX/295;", None),
    ("lid-setter-callers.txt", "apk-callers", "LX/09F;", "ChR"),
]
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
for owner in ["LX/295;", "LX/1rN;"]:
    result = subprocess.run(
        [str(repo / "target/debug/xtask"), "apk-smali", str(apk),
         "--class", owner, "--method", "A03", "--signatures", "--exceptions"],
        check=True, capture_output=True,
    )
    if result.stdout:
        raise SystemExit(f"negative selector unexpectedly matched: {owner}.A03")
    print(f"confirmed no matching method: {owner}.A03")
