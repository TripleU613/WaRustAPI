#!/usr/bin/env python3
"""Reproduce selected current group serializer methods without Android execution."""
import hashlib
from pathlib import Path
import re
import subprocess
import sys

SELECTORS = [{'file': 'lazy-selector-excerpt.txt',
  'class': 'LX/8cm;',
  'method': 'invoke',
  'mode': 'excerpt',
  'signature': None,
  'source_bytes': 38982,
  'source_sha256': 'e4ed69a0f45e3c8f44953ee13e4d422c907c6ff7ccd48b8ba2c3a7c5c8c0b22e'},
 {'file': 'query-constructor.txt',
  'class': 'LX/BVu;',
  'method': '<init>',
  'mode': 'method',
  'signature': 'LX/BVu;-><init>(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;I)V',
  'source_bytes': 51628,
  'source_sha256': 'b92627f4b1a9374668b8525e36c8f3a746512db2c744e7175e9debaec67ba167'},
 {'file': 'group-element-constructor.txt',
  'class': 'LX/Dyu;',
  'method': '<init>',
  'mode': 'method',
  'signature': 'LX/Dyu;-><init>(LX/1bc;I)V',
  'source_bytes': 22057,
  'source_sha256': '0431fa009111ab82c9b715554f98933cc59070789250bd517063a64e2de0f155'},
 {'file': 'query-accessor.txt',
  'class': 'LX/BVu;',
  'method': 'A00',
  'mode': 'full',
  'signature': None,
  'source_bytes': 625,
  'source_sha256': 'c02a748f5f81458066a5b48d14ad9d6a446ccd35f003b757a0896131e7113b6b'},
 {'file': 'iq-builder.txt',
  'class': 'LX/B1x;',
  'method': 'A0c',
  'mode': 'full',
  'signature': None,
  'source_bytes': 374,
  'source_sha256': 'e7bcf562b6d84960044e5cab5394998726164c1210dda625c0ef3094c1e7f251'},
 {'file': 'iq-attributes.txt',
  'class': 'LX/B23;',
  'method': 'A0b',
  'mode': 'full',
  'signature': None,
  'source_bytes': 1171,
  'source_sha256': 'e72b19409766aa34eb83c484f226cc689600680673b2ca83a31b856b20d848fd'},
 {'file': 'children-check.txt',
  'class': 'LX/0Tk;',
  'method': 'A07',
  'mode': 'full',
  'signature': None,
  'source_bytes': 2556,
  'source_sha256': 'bb68355b50e554e6d117030997f93e85c64c51b0397cceeb4bc2d5aabf6461ea'},
 {'file': 'id-string-check.txt',
  'class': 'LX/0Tk;',
  'method': 'A06',
  'mode': 'full',
  'signature': None,
  'source_bytes': 1859,
  'source_sha256': 'f59ac3f687221f42dc9f8bc9fd1c6d218245d8b2a2d0c5e809eef6fcc31244b9'},
 {'file': 'optional-context.txt',
  'class': 'LX/0Tj;',
  'method': 'A06',
  'mode': 'full',
  'signature': None,
  'source_bytes': 519,
  'source_sha256': 'dda7f9bff8c7cc90d6a8d25b1fd8ca1a17591bb8d105a5b2e30e82b3f4342d1c'},
 {'file': 'enum-attribute.txt',
  'class': 'LX/0Tj;',
  'method': 'A07',
  'mode': 'full',
  'signature': None,
  'source_bytes': 556,
  'source_sha256': '9cdd3626eada7b4c0478e0460d012311a5114301dc20b07ffa9dc02fde9b5784'},
 {'file': 'add-group-child.txt',
  'class': 'LX/B21;',
  'method': 'A1C',
  'mode': 'full',
  'signature': None,
  'source_bytes': 572,
  'source_sha256': 'ebed7c0ad8ed911bdff58da7cbb312fbdc3fe371efa998326ab7b55aef4fcfd6'},
 {'file': 'jid-attribute.txt',
  'class': 'LX/B1y;',
  'method': 'A1F',
  'mode': 'full',
  'signature': None,
  'source_bytes': 486,
  'source_sha256': '73ac2ace31b07d51d4c64a97f54496f0652ff2ed2ffc03362f586c068c92c7bf'},
 {'file': 'server-domain.txt',
  'class': 'LX/7Ks;',
  'method': 'getServer',
  'mode': 'full',
  'signature': None,
  'source_bytes': 294,
  'source_sha256': 'ac7ebbdf4605514a5de01cd31185cba5f23a9474904bfe36267350a46bf5293b'}]

def select(data, entry):
    if len(data) != entry["source_bytes"] or hashlib.sha256(data).hexdigest() != entry["source_sha256"]:
        raise SystemExit(f"complete source differs: {entry['file']}")
    if entry["mode"] == "full":
        return data
    lines = data.decode().splitlines()
    prefix = [line for line in lines if line.startswith(("# class:", "# extends:"))]
    source = f"source bytes={entry['source_bytes']}, sha256={entry['source_sha256']}"
    if entry["mode"] == "method":
        matches = re.findall(r"^\.method .*?^\.end method$", data.decode(), re.M | re.S)
        chosen = [block for block in matches if "    # full signature: " + entry["signature"] + "\n" in block]
        if len(chosen) != 1:
            raise SystemExit("exact complete constructor selector is ambiguous")
        result = ["# Complete selected method; other overloads omitted; " + source, *prefix, chosen[0]]
    else:
        result = ["# Bounded selector10 excerpt; other instructions/exception table omitted; " + source, *prefix]
        for line in lines:
            pc = re.match(r"    ([0-9a-f]+)\.\.[0-9a-f]+  ", line)
            if pc:
                value = int(pc.group(1), 16)
                if value < 0xa or 0x70a <= value < 0x76a:
                    result.append(line)
            elif line.startswith((".method", "    # full signature:")):
                result.append(line)
    return ("\n".join(result) + "\n").encode()

if __name__ == "__main__":
    apk = Path(sys.argv[1]).resolve()
    if hashlib.sha256(apk.read_bytes()).hexdigest() != "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e":
        raise SystemExit("APK differs from pinned reference")
    bundle = Path(__file__).resolve().parent
    repo = bundle.parents[3]
    for entry in SELECTORS:
        data = subprocess.run([str(repo / "target/debug/xtask"), "apk-smali", str(apk), "--class", entry["class"], "--method", entry["method"], "--signatures", "--exceptions"], check=True, capture_output=True).stdout
        if select(data, entry) != (bundle / entry["file"]).read_bytes():
            raise SystemExit(f"retained export differs: {entry['file']}")
        print(f"matched {entry['file']}")
