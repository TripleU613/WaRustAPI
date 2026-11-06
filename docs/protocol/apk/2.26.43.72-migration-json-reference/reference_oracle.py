#!/usr/bin/env python3
"""Compile pinned AOSP JSON unchanged; execute only a synthetic library fixture."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import urllib.request

HERE = Path(__file__).resolve().parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sources", type=Path, required=True, help="Public reference cache, never account state")
    parser.add_argument("--fetch", action="store_true", help="Fetch missing pinned public sources into this cache")
    parser.add_argument("--check", type=Path, help="Compare complete synthetic report with this fixture")
    args = parser.parse_args()
    manifest = json.loads((HERE / "sources.json").read_text())
    java_home = os.environ.get("JAVA_HOME")
    javac = str(Path(java_home) / "bin/javac") if java_home else shutil.which("javac")
    java = str(Path(java_home) / "bin/java") if java_home else shutil.which("java")
    if not javac or not java:
        raise SystemExit("Java/Javac required; set JAVA_HOME to an installed JDK")
    sources = []
    prior = HERE.parent / "2.26.43.72-mex-null-serialization"
    for name, expected in manifest["files"].items():
        source = args.sources / (name + ".java")
        if not source.exists() and (prior / source.name).exists():
            source = prior / source.name
        if not source.exists():
            if not args.fetch:
                raise SystemExit("Missing public reference source: " + source.name + "; use --fetch once")
            args.sources.mkdir(parents=True, exist_ok=True)
            with urllib.request.urlopen(expected["url"], timeout=30) as response:
                raw = base64.b64decode(response.read(), validate=True)
            if len(raw) != expected["bytes"] or hashlib.sha256(raw).hexdigest() != expected["sha256"]:
                raise SystemExit("Pinned public source mismatch: " + source.name)
            source.write_bytes(raw)
        raw = source.read_bytes()
        if len(raw) != expected["bytes"] or hashlib.sha256(raw).hexdigest() != expected["sha256"]:
            raise SystemExit("Pinned public source mismatch: " + source.name)
        sources.append(source.resolve())
    # Annotation-only compatibility declarations do not change JSON algorithms.
    annotations = {
        "android/compat/annotation/UnsupportedAppUsage.java": "package android.compat.annotation; public @interface UnsupportedAppUsage {}",
        "android/annotation/SystemApi.java": "package android.annotation; public @interface SystemApi { Client client(); enum Client { MODULE_LIBRARIES } }",
    }
    for name in ("NonNull", "Nullable"):
        annotations["libcore/util/" + name + ".java"] = (
            "package libcore.util; import java.lang.annotation.*; "
            "@Target({ElementType.TYPE_USE,ElementType.TYPE_PARAMETER}) public @interface " + name + " {}"
        )
    with tempfile.TemporaryDirectory(prefix="migration-json-reference-") as directory:
        root = Path(directory)
        classes = root / "classes"
        classes.mkdir()
        stubs = []
        for relative, content in annotations.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
            stubs.append(path)
        subprocess.run([javac, "-encoding", "UTF-8", "-d", str(classes),
                        *map(str, sources), *map(str, stubs), str(HERE / "ReferenceJson.java")],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        report = json.loads(subprocess.check_output([java, "-cp", str(classes), "ReferenceJson"], text=True))
    expected_doc = json.loads((HERE.parent / "2.26.43.72-blocklist-migration-obligation" / "queries.json").read_text())["operations"]["MigrateBlocklistLid"]["doc_id"]
    for case in report["cases"]:
        if json.loads(case["body"])["queryId"] != expected_doc:
            raise SystemExit("Reference query ID differs from pinned operation export")
    if args.check:
        if report != json.loads(args.check.read_text()):
            raise SystemExit("Controlled reference report differs from expected fixture")
        print("Four controlled reference cases match; no APK/device/account execution")
    else:
        print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
