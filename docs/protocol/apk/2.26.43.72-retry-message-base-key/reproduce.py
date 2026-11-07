#!/usr/bin/env python3
"""Reproduce complete selected pinned DEX exports without executing Android."""
import hashlib
from pathlib import Path
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
SELECTORS = [('retry-session-gate.txt', 'apk-smali', 'LX/163;', 'A04'), ('save-message-base-key.txt', 'apk-smali', 'LX/0WJ;', 'A0s'), ('compare-message-base-key.txt', 'apk-smali', 'LX/0WJ;', 'A15'), ('session-base-key.txt', 'apk-smali', 'LX/EZK;', 'A00'), ('byte-string-copy.txt', 'apk-smali', 'LX/DBd;', 'A1a'), ('address-remapping.txt', 'apk-smali', 'LX/0YG;', 'A02'), ('session-presence.txt', 'apk-smali', 'LX/0WJ;', 'A14'), ('session-load.txt', 'apk-smali', 'LX/0WJ;', 'A0M'), ('save-location-wrapper.txt', 'apk-smali', 'LX/0WJ;', 'A0n'), ('compare-location-wrapper.txt', 'apk-smali', 'LX/0WJ;', 'A11'), ('voip-peer-failure-call.txt', 'apk-smali', 'LX/FfB;', 'call'), ('save-callers.txt', 'apk-callers', 'LX/0WJ;', 'A0s'), ('compare-callers.txt', 'apk-callers', 'LX/0WJ;', 'A15')]

if __name__ == "__main__":
    apk = Path(sys.argv[1]).resolve()
    if hashlib.sha256(apk.read_bytes()).hexdigest() != APK_HASH:
        raise SystemExit("APK differs from pinned reference")
    bundle = Path(__file__).resolve().parent
    repo = bundle.parents[3]
    for filename, operation, owner, method in SELECTORS:
        args = [str(repo / "target/debug/xtask"), operation, str(apk), "--class", owner, "--method", method]
        if operation == "apk-smali":
            args += ["--signatures", "--exceptions"]
        result = subprocess.run(args, check=True, capture_output=True).stdout
        if result != (bundle / filename).read_bytes():
            raise SystemExit(f"retained export differs: {filename}")
        print(f"matched {filename}")
