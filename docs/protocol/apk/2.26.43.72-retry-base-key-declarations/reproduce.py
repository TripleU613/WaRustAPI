#!/usr/bin/env python3
"""Reproduce pinned static schema declarations; do not execute Android."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

APK_HASH = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
METHODS = [('index-declaration.txt', 'LX/1jB;', 'ATY'), ('table-declaration.txt', 'LX/1jB;', 'ATc'), ('column-build-reset.txt', 'LX/0Me;', 'A00'), ('column-autoincrement-flag.txt', 'LX/0Me;', 'A01'), ('column-primary-flag.txt', 'LX/0Me;', 'A02'), ('column-descriptor.txt', 'LX/0Mf;', None), ('column-types.txt', 'LX/0Mg;', '<clinit>'), ('column-type-constructor.txt', 'LX/0Mg;', '<init>'), ('default-blank-check.txt', 'Lcom/whatsapp/infra/core/util/string/StringUtils;', 'A0J'), ('table-registration.txt', 'LX/0Mb;', 'CWj'), ('index-registration.txt', 'LX/0Mb;', 'CWU'), ('provider-constructor.txt', 'LX/1jB;', '<init>')]
PROBES = [('apk-callers', ['--class', 'LX/1jB;', '--method', '<init>']), ('apk-smali', ['--class', 'LX/1jB;', '--method', '<clinit>', '--signatures', '--exceptions'])]

def run(repo,apk,operation,args):
    return subprocess.run([str(repo/"target/debug/xtask"),operation,str(apk),*args],check=True,capture_output=True)

def check(bundle,name,data):
    if data != (bundle/name).read_bytes():
        raise SystemExit(f"retained export differs: {name}")
    print(f"matched {name}")

if __name__ == "__main__":
    apk=Path(sys.argv[1]).resolve()
    if hashlib.sha256(apk.read_bytes()).hexdigest()!=APK_HASH:
        raise SystemExit("APK differs from pinned reference")
    bundle=Path(__file__).resolve().parent
    repo=bundle.parents[3]
    for name,owner,method in METHODS:
        args=["--class",owner,"--signatures","--exceptions"]
        if method is not None: args += ["--method",method]
        check(bundle,name,run(repo,apk,"apk-smali",args).stdout)
    args=["--class","LX/0MY;","--method","CWk","--signature","(Ljava/lang/String;[LX/0Mf;)V","--include-interfaces"]
    check(bundle,"table-sink-overrides.txt",run(repo,apk,"apk-virtual-overrides",args).stdout)
    probes=[]
    for operation,args in PROBES:
        r=run(repo,apk,operation,args)
        probes.append({"operation":operation,"args":args,"returncode":r.returncode,"stdout":r.stdout.decode(),"stderr":r.stderr.decode()})
    check(bundle,"provider-probes.json",(json.dumps(probes,indent=2)+"\n").encode())
