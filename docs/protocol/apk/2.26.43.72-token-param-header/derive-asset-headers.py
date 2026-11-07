"""Pinned ZIP-only v4 header extraction; not effective configuration or APK execution."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--apk", type=Path, required=True)
args = parser.parse_args()
sha = hashlib.sha256()
with args.apk.open("rb") as f:
    for chunk in iter(lambda:f.read(1024*1024), b""):
        sha.update(chunk)
if sha.hexdigest() != "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e":
    raise ValueError("pinned APK mismatch")
base = Path(__file__).parent
prior = json.loads((base.parent / "2.26.43.72-token-mobileconfig-binding/parameter-assets.json").read_text())
if not isinstance(prior, list):
    raise ValueError("asset inventory shape changed")
rows = []
with zipfile.ZipFile(args.apk) as apk:
    for old in prior:
        name = old["path"]
        data = apk.read(name)
        digest = hashlib.sha256(data).hexdigest()
        if digest != old["sha256"]:
            raise ValueError("prior asset identity changed")
        if len(data) < 30 or data[:2] != b"v4":
            raise ValueError("expected complete v4 header")
        size, magic, a00, a02 = struct.unpack_from("<hhHH", data, 2)
        a01, hash_length = struct.unpack_from("<Hh", data, 26)
        if magic != 32767 or hash_length < 0 or size != 30+hash_length or len(data) < size:
            raise ValueError("invalid selected header")
        schema = data[30:size].decode("utf-8")
        rows.append({"path":name,"bytes":len(data),"sha256":digest,"version":4,
            "header_bytes":size,"magic":magic,"PHU_A00":a00,"PHU_A02":a02,
            "PHU_A01":a01,"schema_hash_bytes":hash_length,"schema_hash":schema,
            "remaining_body_bytes":len(data)-size})
print(json.dumps({"kind":"pinned static packaged parameter headers; body not decoded", "selected":rows},indent=2))
