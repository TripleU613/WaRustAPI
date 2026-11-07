"""Pinned selected callback branches, preserving full-source identities and catch tables."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--apk',type=Path,required=True)
p.add_argument('--xtask',type=Path,default=Path('target/debug/xtask'))
a=p.parse_args();sha=hashlib.sha256()
with a.apk.open('rb') as f:
    for b in iter(lambda:f.read(1024*1024),b''):sha.update(b)
if sha.hexdigest()!='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e':raise ValueError('APK changed')
result={}
for cls,digest,ranges in [
 ('IE9','b2b33fa0cd84e980606dd9d121124a792fe30efa0ec40ded7b31c31a6acafcdd',[(0x182e,0x185e)]),
 ('ID2','b4c94aff5973ccae4f47675b11a04156c4eaf8ac0df3283298a926d87ecf5d1c',[(0x5e0,0x622),(0xac0,0xac6)])]:
    raw=subprocess.check_output([str(a.xtask.resolve()),'apk-smali',str(a.apk),'--class',f'LX/{cls};','--method','invoke','--signatures','--exceptions'],stderr=subprocess.PIPE)
    if hashlib.sha256(raw).hexdigest()!=digest:raise ValueError('full export changed')
    lines=raw.decode().splitlines();selected=[];entry=[];metadata=[]
    for line in lines:
        m=re.match(r'    ([0-9a-f]+)\.\.([0-9a-f]+)  ',line)
        if m:
            pc=int(m[1],16)
            if pc<0x12:entry.append(line)
            if any(lo<=pc<hi for lo,hi in ranges):selected.append(line)
        elif line.startswith(('    # full signature:','    # exception table:','    # catch ')):
            metadata.append(line)
    result[cls]={'full_source_sha256':digest,'full_source_bytes':len(raw),'entry_and_switch':entry,'selected_byte_ranges':[[hex(lo),hex(hi)] for lo,hi in ranges],'selected_instructions':selected,'signature_and_complete_catch_metadata':metadata}
print(json.dumps(result,indent=2))
