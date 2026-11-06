#!/usr/bin/env python3
"""Reproduce pinned static exports; never execute the Android target."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

BUNDLE = Path(__file__).resolve().parent
REPO = BUNDLE.parents[3]
APK = Path(sys.argv[1]).resolve()
TOOL = REPO / "target/debug/xtask"
assert hashlib.sha256(APK.read_bytes()).hexdigest() == "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"


def call(*args):
    return subprocess.run([str(TOOL), *args[:1], str(APK), *args[1:]], check=True, capture_output=True).stdout


def smali(cls, method):
    return call("apk-smali", "--class", cls, "--method", method, "--signatures", "--exceptions")


def compare(name, data):
    assert data == (BUNDLE / name).read_bytes(), name


for cls, method, name in [
    ("LX/1lZ;", "A1C", "event-forward.txt"),
    ("LX/1lZ;", "A0R", "event-builder.txt"),
    ("LX/1uW;", "A00", "event-selector.txt"),
    ("LX/0ln;", "A0G", "handler-admission.txt"),
    ("LX/0ln;", "A07", "handler-post.txt"),
    ("LX/0ln;", "A00", "handler-select.txt"),
    ("LX/00D;", "A00", "provider-lazy.txt"),
    ("LX/05h;", "<init>", "provider-init.txt"),
    ("LX/05h;", "get", "provider-get.txt"),
    ("LX/00D;", "A02", "provider-entry.txt"),
    ("LX/00D;", "A03", "provider-dispatch.txt"),
]:
    compare(name, smali(cls, method))

for cls, method, name, ranges in [
    ("LX/15E;", "BLn", "consumer-selected.txt", [(0, 0x1a2), (0x386, 0x388)]),
    ("LX/0m2;", "handleMessage", "handler-message-selected.txt", [(0x21c, 0x454), (0x7e0, 0x80c)]),
    ("LX/SQ0;", "<init>", "providers-selected.txt", [(0x406, 0x416)]),
    ("LX/1Uw;", "A01", "factory-low-selected.txt", [(0, 0xc), (0xcd2, 0xd86)]),
    ("LX/1Uw;", "A0S", "factory-selected.txt", [(0, 4), (0xa5c, 0xa68)]),
]:
    lines = smali(cls, method).decode().splitlines()
    selected = ["# Selected DEX excerpt; not a complete method."] + lines[:6]
    if name == "factory-selected.txt":
        switch = next(s for s in lines if "packed-switch " in s)
        assert re.search(r"(?:\[|, )3033:@0xa5c(?:,|\])", switch)
        selected.append("# Selected packed-switch entry: key3033 -> byte0x0a5c; other entries omitted.")
    for line in lines[6:]:
        match = re.match(r"    ([0-9a-f]+)\.\.", line)
        if match and any(lo <= int(match[1], 16) < hi for lo, hi in ranges):
            selected.append(line)
        catch = re.search(r"code_units ([0-9a-f]+)\.\.([0-9a-f]+)", line)
        if catch and any(int(catch[1], 16) * 2 < hi and int(catch[2], 16) * 2 > lo for lo, hi in ranges):
            selected.append(line)
    compare(name, ("\n".join(selected) + "\n").encode())

compare("readreceipts-refs.txt", call("apk-xref", "--anchor", "readreceipts"))
compare("consumer-callers.txt", call("apk-callers", "--class", "LX/15E;", "--method", "BLn"))
compare("consumer-constructor-callers.txt", call("apk-callers", "--class", "LX/15E;", "--method", "<init>"))
compare("event-dispatchers.txt", call("apk-virtual-overrides", "--class", "LX/1lW;", "--method", "CPm", "--signature", "(Landroid/os/Message;)V", "--include-interfaces"))
assert smali("LX/1rH;", "A03") == (REPO / "docs/protocol/apk/2.26.43.72-distribution-ack-admission/native.txt").read_bytes()
assert smali("LX/1Uw;", "A00") == (REPO / "docs/protocol/rea/2.26.43.72-privacy-transport/di-router-dex.txt").read_bytes()
for base, delta, target in [(0x2a, 7, 0x38), (0xe0, 0x153, 0x386), (0x12a, 0x2e, 0x186), (0x2e4, 0xe, 0x300), (0x4b6, 0x16, 0x4e2), (0x4e6, -0x25f, 0x28), (8, 0x665, 0xcd2)]:
    assert base + delta * 2 == target
assert (3033 >> 14) & 0x7f == 0 and 0xaf7 < (3033 & 0x3fff) <= 0xc10
for name, entry in json.loads((BUNDLE / "checksums.json").read_text()).items():
    data = (BUNDLE / name).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"], name
print("Twenty exports/excerpts, two reused methods, branch/provider arithmetic, all checksums and pinned APK digest match.")
