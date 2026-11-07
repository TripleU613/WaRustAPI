#!/usr/bin/env python3
"""Regenerate the selected pinned login LID constructor and setter methods without running Android."""
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
selectors = [('setter-definitions.txt', 'apk-smali', '', 'ChR'),
 ('lid-type-predicate.txt', 'apk-smali', 'LX/0Dn;', 'A0Z'),
 ('lid-device-type.txt', 'apk-smali', 'LX/0EM;', None),
 ('lid-user-type.txt', 'apk-smali', 'LX/0Dx;', None),
 ('record-adoption.txt', 'apk-smali', 'LX/09O;', 'A00'),
 ('record-notify.txt', 'apk-smali', 'LX/09O;', 'A02'),
 ('own-record-initialization-gate.txt', 'apk-smali', 'LX/09O;', 'A07'),
 ('nullable-string-parser.txt', 'apk-smali', 'LX/0Dd;', 'A02'),
 ('../2.26.43.72-message-ack-chat/jid-parser.txt', 'apk-smali', 'LX/0Dd;', 'A00'),
 ('../2.26.43.72-message-ack-chat/user-factory.txt', 'apk-smali', 'LX/0De;', 'A02')]
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